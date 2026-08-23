//! The completion stubs against real shells (A-3').
//!
//!   cargo run --example completion_probe
//!
//! `completions.rs` is pure and its unit tests call [`candidates`] directly, so
//! they settle what the *answer* is. They cannot settle whether the four-line
//! stub in each shell asks the question correctly, and that is where the bug
//! was: the bash stub narrowed `IFS` before expanding `"${COMP_WORDS[@]:1:…}"`,
//! and **on bash 3.2 — the bash macOS ships — that collapses the slice into a
//! single argument**. `artisan migrate` arrived as one word, no command was
//! recognised in it, and `stackvo artisan migrate --<TAB>` offered this
//! binary's own global flags. Every Rust test passed the whole time.
//!
//! So this drives the real thing. It writes each stub, sources it in that
//! shell, sets the variables the line editor would have set, calls the
//! completion function and reads back what it put in the reply array.
//!
//! Unix only, and the `#[cfg]` is at the top rather than around the checks —
//! `tui_probe`'s reasoning: a probe that "passed" on Windows by doing nothing
//! would be worse than one that says it did not run.

#[cfg(unix)]
fn main() -> std::process::ExitCode {
    unix::run()
}

#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("completion_probe needs a unix shell; nothing was checked.");
    std::process::ExitCode::FAILURE
}

#[cfg(unix)]
mod unix {
    use std::path::{Path, PathBuf};
    use std::process::{Command, ExitCode};

    /// One line typed so far, and what the completion must answer with.
    struct Case {
        /// What the user has typed, `stackvo` included. The **last** element is
        /// the word under the cursor and may be empty.
        line: &'static [&'static str],
        what: &'static str,
        expect: Expect,
    }

    enum Expect {
        /// Exactly these, in this order.
        Exactly(&'static [&'static str]),
        /// Nothing at all — the shell falls back to filenames.
        Nothing,
        /// At least one, and every one of these among them.
        Containing(&'static [&'static str]),
    }

    /// The cases, and every one of them is a rule stated somewhere else.
    const CASES: &[Case] = &[
        Case {
            line: &["stackvo", "sta"],
            what: "a prefix narrows the command list",
            expect: Expect::Exactly(&["status", "start"]),
        },
        Case {
            line: &["stackvo", "mcp-install", ""],
            what: "a <client> slot offers the clients",
            expect: Expect::Containing(&["claude-code", "cursor"]),
        },
        Case {
            line: &["stackvo", "path-install", ""],
            what: "a [shell] slot offers the shells",
            expect: Expect::Containing(&["zsh", "bash", "fish", "powershell"]),
        },
        Case {
            line: &["stackvo", "xdebug", "shop", ""],
            what: "an alternation completes to its own words",
            expect: Expect::Exactly(&["on", "off"]),
        },
        // The one this probe was written for.
        Case {
            line: &["stackvo", "artisan", "migrate", "--"],
            what: "NOTHING is offered after a passthrough",
            expect: Expect::Nothing,
        },
        Case {
            line: &["stackvo", "--root", ""],
            what: "a flag's value is not a command",
            expect: Expect::Nothing,
        },
        Case {
            line: &["stackvo", "logs", "--"],
            what: "a command's own flags are offered beside the globals",
            expect: Expect::Containing(&["--follow", "--tail", "--json"]),
        },
    ];

    pub fn run() -> ExitCode {
        let Some(binary) = binary() else {
            eprintln!(
                "no `stackvo` binary — run `cargo build --bin stackvo` first.\nNothing was checked."
            );
            return ExitCode::FAILURE;
        };

        // **Which binary, and how old.** `cargo run --example` builds the
        // example and not the `stackvo` bin, so a probe run straight after
        // editing `completions.rs` reads the *previous* stub and reports every
        // check green. That happened on the first run of this probe, against
        // the very bug it was written for. The line below is what makes it
        // visible; `--bin stackvo` in the invocation above is what makes it
        // not happen.
        println!("binary: {}{}", binary.display(), staleness(&binary));

        let dir = std::env::temp_dir().join(format!("stackvo-completion-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        // On `PATH` under the name the stubs use, because that is how a real
        // shell reaches it and a stub naming an absolute path would be a
        // different stub from the one people install.
        let _ = std::os::unix::fs::symlink(&binary, dir.join("stackvo"));

        let mut failures = 0usize;
        let mut ran = 0usize;

        for shell in ["bash", "zsh"] {
            if which(shell).is_none() {
                println!("— {shell}: not on this machine, skipped");
                continue;
            }
            println!("\n{shell}  ({})", version(shell));

            let stub = stub_for(&binary, shell);
            let stub_path = dir.join(format!("stub.{shell}"));
            std::fs::write(&stub_path, &stub).expect("writing the stub");

            for case in CASES {
                ran += 1;
                let got = drive(shell, &dir, &stub_path, case.line);
                if check(&got, &case.expect) {
                    println!("  ok    {}", case.what);
                } else {
                    failures += 1;
                    println!("  FAIL  {}", case.what);
                    println!("        typed:    {:?}", case.line);
                    println!("        expected: {}", describe(&case.expect));
                    println!("        got:      {got:?}");
                }
            }
        }

        let _ = std::fs::remove_dir_all(&dir);

        if ran == 0 {
            eprintln!("\nNeither bash nor zsh is on this machine. Nothing was checked.");
            return ExitCode::FAILURE;
        }
        println!("\n{ran} checks, {failures} failed.");
        if failures == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }

    /// Run one completion the way the line editor would.
    ///
    /// The harness sets the variables each shell's completion system sets and
    /// calls the stub's own function, so what is exercised is the stub — not a
    /// re-implementation of it that could be right where the stub is wrong.
    fn drive(shell: &str, dir: &Path, stub: &Path, line: &[&str]) -> Vec<String> {
        let program = line[0];
        let words = &line[..line.len() - 1];
        let current = line[line.len() - 1];
        let cursor = words.len();

        let script = match shell {
            "bash" => format!(
                "export PATH=\"{dir}:$PATH\"\n\
                 source {stub}\n\
                 COMP_WORDS=({words} {current})\n\
                 COMP_CWORD={cursor}\n\
                 COMPREPLY=()\n\
                 _{program}_complete\n\
                 printf '%s\\n' \"${{COMPREPLY[@]}}\"\n",
                dir = dir.display(),
                stub = quote(&stub.display().to_string()),
                words = words.iter().map(|w| quote(w)).collect::<Vec<_>>().join(" "),
                current = quote(current),
                cursor = cursor,
            ),
            // zsh's stub calls `compadd`, which only works inside the completion
            // system. The body is what is under test, so it is run with the same
            // `words`/`CURRENT` the widget would have set and the reply array is
            // printed instead of added.
            "zsh" => format!(
                "export PATH=\"{dir}:$PATH\"\n\
                 words=({words} {current})\n\
                 CURRENT={cursor}\n\
                 compadd() {{ shift; printf '%s\\n' \"$@\"; }}\n\
                 autoload -Uz compinit 2>/dev/null\n\
                 compdef() {{ :; }}\n\
                 source {stub}\n\
                 _{program}_complete\n",
                dir = dir.display(),
                stub = quote(&stub.display().to_string()),
                words = words.iter().map(|w| quote(w)).collect::<Vec<_>>().join(" "),
                current = quote(current),
                // zsh arrays are 1-based, so the word under the cursor is at
                // `words.len() + 1`.
                cursor = cursor + 1,
            ),
            other => panic!("no harness for `{other}`"),
        };

        let out = Command::new(shell)
            .arg("-c")
            .arg(&script)
            .current_dir(dir)
            .output()
            .expect("running the shell");

        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn check(got: &[String], expect: &Expect) -> bool {
        match expect {
            Expect::Nothing => got.is_empty(),
            Expect::Exactly(want) => {
                got.len() == want.len() && got.iter().zip(*want).all(|(g, w)| g == w)
            }
            Expect::Containing(want) => {
                !got.is_empty() && want.iter().all(|w| got.iter().any(|g| g == w))
            }
        }
    }

    fn describe(expect: &Expect) -> String {
        match expect {
            Expect::Nothing => "nothing".to_string(),
            Expect::Exactly(want) => format!("exactly {want:?}"),
            Expect::Containing(want) => format!("all of {want:?}"),
        }
    }

    /// Ask the binary for its own stub, rather than calling the library.
    ///
    /// The thing people install is what `stackvo completions <shell>` prints,
    /// so that is what gets driven.
    fn stub_for(binary: &Path, shell: &str) -> String {
        let out = Command::new(binary)
            .args(["completions", shell])
            .output()
            .expect("asking for the stub");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// ` (N seconds older than completions.rs)`, when it is.
    ///
    /// Silent when the binary is newer than the source, which is the ordinary
    /// case and needs no line of its own.
    fn staleness(binary: &Path) -> String {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/completions.rs");
        let Ok(built) = binary.metadata().and_then(|m| m.modified()) else {
            return String::new();
        };
        let Ok(edited) = source.metadata().and_then(|m| m.modified()) else {
            return String::new();
        };
        match edited.duration_since(built) {
            Ok(behind) if behind.as_secs() > 0 => format!(
                "  ** {}s OLDER than completions.rs — run `cargo build --bin stackvo` **",
                behind.as_secs()
            ),
            _ => String::new(),
        }
    }

    fn binary() -> Option<PathBuf> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        ["debug", "release"]
            .iter()
            .map(|profile| root.join(profile).join("stackvo"))
            .find(|path| path.is_file())
    }

    fn which(program: &str) -> Option<PathBuf> {
        std::env::var_os("PATH")
            .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
            .map(|dir| dir.join(program))
            .find(|path| path.is_file())
    }

    fn version(shell: &str) -> String {
        Command::new(shell)
            .arg("--version")
            .output()
            .ok()
            .map(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("?")
                    .to_string()
            })
            .unwrap_or_else(|| "?".into())
    }

    /// Single quotes, with the one escape a POSIX shell has.
    fn quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
