//! The README, held to the same standard as the code.
//!
//! This project's own thesis is that "probably the same" is not a shipping
//! standard: `contracts/ipc.json` is checked against `lib.rs` and `commands.rs`,
//! `mcp.rs` cross-checks every tool against the command it names, and the Bash
//! generator was replaced against byte-for-byte fixtures. The README was the one
//! surface outside that culture — and by the time anyone measured it, both of
//! its measurable claims were wrong:
//!
//!   * "Thirty-four commands take an `AppHandle`" — the real count was 48.
//!   * "Two tools change things" — `--allow-writes` actually unlocks 7,
//!     including `stack_down`, which stops the entire stack.
//!
//! The second one is why this file exists rather than a commit that fixes the
//! numbers. A reader deciding whether to hand an assistant `--allow-writes` was
//! being told the flag unlocked Xdebug and a certificate reissue. That is a
//! security documentation defect, and the defence against it recurring is not
//! care — it is this test.
//!
//! ## Why the numbers are counted rather than generated
//!
//! `mcp.rs` already argued that generating the tool *list* is the wrong move:
//! dispatch cannot be generated, so a generated list advertises tools that fail
//! when called. The same holds a step further out. What is checkable is that the
//! prose agrees with the code, so that is what is checked — the README stays
//! hand-written, and a number in it that stops being true fails the build.

/// How the `AppHandle` count is arrived at, so a future reader does not have to
/// reverse-engineer the regex-free scan below.
///
/// A command is a `#[tauri::command]` (or `#[tauri::command(async)]`) attribute
/// followed by a function whose argument list mentions `AppHandle`. Attributes
/// inside a `#[cfg(test)]` module do not count: three of them exist purely to
/// exercise the registration machinery and are not part of the shipped surface.
mod scan {
    /// The byte ranges of every top-level `#[cfg(test)]` item.
    ///
    /// **Found by indentation, not by counting braces.** Brace counting is the
    /// obvious approach and it is wrong here, which this file learned the
    /// expensive way: a test that writes deliberately truncated JSON —
    /// `"{\"theme\": \"dark\", trunca"` — puts an unmatched `{` inside a string
    /// literal, the count never returns to zero, the last test module stops
    /// being recognised as a test module, and three test-only commands start
    /// counting as shipped surface. The scan reported 146 commands instead of
    /// 143 and the guard below caught it, which is the only reason this comment
    /// exists rather than a wrong number in the README.
    ///
    /// Skipping string and char literals properly would mean writing a Rust
    /// lexer. The cheaper invariant is already enforced by CI: `cargo fmt
    /// --check` runs on every push, and rustfmt closes a top-level item with a
    /// `}` in column zero. Nothing inside a literal can look like that, because
    /// rustfmt indents every line it owns.
    fn test_regions(src: &str) -> Vec<(usize, usize)> {
        let mut regions = Vec::new();
        let mut from = 0;

        // Anchored to the start of a line: a `#[cfg(test)]` on an inner item is
        // indented, and only the top-level ones enclose whole commands.
        while let Some(offset) = src[from..].find("\n#[cfg(test)]") {
            let start = from + offset + 1;

            // The first column-zero `}` after it closes the item.
            let end = match src[start..].find("\n}") {
                Some(i) => start + i + 2,
                // Unterminated: treat the rest of the file as inside, which is
                // the safe direction — it can only *exclude* commands, and the
                // total assertion below turns that into a visible failure.
                None => src.len(),
            };

            regions.push((start, end));
            from = end;
        }
        regions
    }

    /// The balanced argument list of the first `fn` after `at`.
    fn argument_list(src: &str, at: usize) -> Option<&str> {
        let rest = &src[at..];
        let f = rest.find(" fn ")?;
        let open = at + f + rest[f..].find('(')?;

        let mut depth = 0usize;
        for (i, c) in src[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&src[open..=open + i]);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The name of every shipped command, for the registration cross-check.
    pub fn command_names(src: &str) -> Vec<String> {
        let regions = test_regions(src);
        let shipped = |p: usize| !regions.iter().any(|&(s, e)| (s..=e).contains(&p));

        let mut names = Vec::new();
        let mut from = 0;
        while let Some(offset) = src[from..].find("#[tauri::command") {
            let at = from + offset;
            from = at + 1;
            if !shipped(at) {
                continue;
            }
            if let Some(name) = function_name(src, at) {
                names.push(name);
            }
        }
        names
    }

    /// The identifier after the first ` fn ` following `at`.
    fn function_name(src: &str, at: usize) -> Option<String> {
        let rest = &src[at..];
        let f = rest.find(" fn ")? + " fn ".len();
        let tail = &rest[f..];
        let end = tail.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
        Some(tail[..end].to_string())
    }

    /// `(commands, commands taking an AppHandle)` in shipped code.
    pub fn commands(src: &str) -> (usize, usize) {
        let regions = test_regions(src);
        let shipped = |p: usize| !regions.iter().any(|&(s, e)| (s..=e).contains(&p));

        let (mut total, mut with_handle) = (0, 0);
        let mut from = 0;
        while let Some(offset) = src[from..].find("#[tauri::command") {
            let at = from + offset;
            from = at + 1;
            if !shipped(at) {
                continue;
            }
            total += 1;
            if argument_list(src, at).is_some_and(|args| args.contains("AppHandle")) {
                with_handle += 1;
            }
        }
        (total, with_handle)
    }
}

fn readme() -> String {
    // `CARGO_MANIFEST_DIR` is `src-tauri`; the README is one level up. Read
    // rather than `include_str!` so a failure says "the README moved" instead of
    // failing to compile.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("a repository root above src-tauri")
        .join("README.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every `commands::<name>,` in `lib.rs`'s `generate_handler!` list.
///
/// This is the *registration*, as opposed to the implementation the scanner
/// above finds. Tauri only routes what is named here, and a command written in
/// `commands.rs` and left out of this list **compiles and silently does
/// nothing** — the frontend gets "command not found" at runtime, on a screen
/// nobody opened during development.
fn registered() -> Vec<String> {
    let source = include_str!("../src/lib.rs");
    let start = source
        .find("generate_handler!")
        .expect("lib.rs registers its commands with generate_handler!");
    let body = &source[start..];
    let end = body.find("])").unwrap_or(body.len());

    body[..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix("commands::"))
        .filter_map(|name| name.split(',').next())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Registration and implementation agree — checked here, in Rust, with nothing
/// fetched from anywhere.
///
/// This is what suite E in `tools/validate-contracts.mjs` does, and that job
/// **checks out an external repository to run**. The readiness review flagged
/// the dependency as a risk rather than a defect, and it is: the day
/// `stackvo/stackvo` goes private, is renamed or rate-limits, the contract gate
/// disappears and nothing says so. This half of it needs no network, no
/// checkout and no Node, so it keeps working when that job cannot.
///
/// It also replaces a hardcoded `143` that used to sit below. That number was a
/// sanity check on the scanner and it made every new command look like a
/// scanner fault — noise, where this is signal.
#[test]
fn every_implemented_command_is_registered_and_the_reverse() {
    use std::collections::BTreeSet;

    let source = include_str!("../src/commands.rs");
    let implemented: BTreeSet<String> = scan::command_names(source).into_iter().collect();
    let registered: BTreeSet<String> = registered().into_iter().collect();

    let unregistered: Vec<&String> = implemented.difference(&registered).collect();
    assert!(
        unregistered.is_empty(),
        "{} command(s) are implemented but never registered in lib.rs: {unregistered:?}\n\
         These compile, and fail at runtime with \"command not found\".",
        unregistered.len()
    );

    let unimplemented: Vec<&String> = registered.difference(&implemented).collect();
    assert!(
        unimplemented.is_empty(),
        "{} command(s) are registered in lib.rs but not implemented: {unimplemented:?}",
        unimplemented.len()
    );

    assert!(
        implemented.len() > 100,
        "the scan found only {} commands, so it is not reading the file it thinks it is",
        implemented.len()
    );
}

/// The claim that was wrong by 14, in the paragraph explaining why MCP cannot
/// reach the rest of the command surface.
#[test]
fn the_readme_states_the_real_app_handle_count() {
    let source = include_str!("../src/commands.rs");
    let (total, with_handle) = scan::commands(source);

    // The scanner is trusted because the test above proved it agrees with the
    // registration list, not because of a number written down here.
    let claim = format!("{with_handle} of the {total} commands take");
    assert!(
        readme().contains(&claim),
        "the README should say {claim:?} — it is now the measured number"
    );
}

/// The claim that mattered: what `--allow-writes` actually hands over.
#[test]
fn the_readme_states_every_tool_allow_writes_unlocks() {
    use stackvo_desktop_lib::mcp;

    let readme = readme();
    let writers: Vec<&str> = mcp::TOOLS
        .iter()
        .filter(|t| t.writes)
        .map(|t| t.name)
        .collect();

    let claim = format!(
        "{} of the {} tools change things",
        writers.len(),
        mcp::TOOLS.len()
    );
    assert!(readme.contains(&claim), "the README should say {claim:?}");

    // Naming them is the part that has security value. A count alone would have
    // gone from "two" to "seven" and still not told anyone that `stack_down` is
    // in the set.
    for tool in writers {
        // The README drops the `stackvo_` prefix every tool carries, because the
        // prose is already about StackVo's tools.
        let short = tool.strip_prefix("stackvo_").unwrap_or(tool);
        assert!(
            readme.contains(&format!("`{short}`")),
            "`{short}` is unlocked by --allow-writes but the README does not name it"
        );
    }
}

/// Both numbers came from the same paragraph pair, and the paragraphs are only
/// honest together: a reader who sees "17 tools" needs the same table to have
/// been counted. Cheap, and it catches a tool added without a `writes` flag
/// decision being made at all.
#[test]
fn every_mcp_tool_declares_whether_it_writes() {
    use stackvo_desktop_lib::mcp;

    let readable = mcp::visible(false).count();
    let all = mcp::visible(true).count();

    assert_eq!(all, mcp::TOOLS.len(), "a tool is visible in neither mode");
    assert_eq!(
        all - readable,
        mcp::TOOLS.iter().filter(|t| t.writes).count(),
        "the write gate and the `writes` flag disagree"
    );
}

/// What the README says about the generator, against what `generate_with` does.
///
/// Both of the claims below were wrong at once, and had been for as long as the
/// takeover was finished. The README said the Rust generator "runs _alongside_
/// the Bash one; it does not replace it", listed `bash` as **the default**, and
/// closed with "Bash runs in every mode" — while `GeneratorEngine::Bash`
/// returns `Unsupported` to every caller and `#[default]` sits on `Rust`.
///
/// This is the class the numbers were already guarded against, one step out: a
/// count is checkable and so is "which variant carries `#[default]`". The prose
/// around it is still review's problem. What this settles is that the README
/// cannot name a default the enum does not have, or promise a mode that errors.
#[test]
fn the_readme_names_the_generator_default_the_enum_actually_carries() {
    let commands = include_str!("../src/commands.rs");
    let enum_start = commands
        .find("pub enum GeneratorEngine")
        .expect("commands.rs declares GeneratorEngine");
    let body = &commands[enum_start..enum_start + 800];

    // The variant that follows `#[default]` is the one a caller gets when it
    // sends no mode at all — which is what "the default" means in the README.
    let after_default = body
        .split_once("#[default]")
        .map(|(_, rest)| rest)
        .expect("one GeneratorEngine variant is #[default]");
    let default_variant = after_default
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("///") && !l.starts_with('#'))
        .expect("a variant after #[default]")
        .trim_end_matches(',')
        .to_string();
    assert_eq!(
        default_variant, "Rust",
        "the enum's default moved; the README's mode table has to move with it"
    );

    let readme = readme();
    assert!(
        !readme.contains("Bash runs in every mode"),
        "the README says Bash runs in every mode; `GeneratorEngine::Bash` returns Unsupported"
    );
    assert!(
        !readme.contains("`bash`   | What StackVo does today. The default."),
        "the README still lists `bash` as the default"
    );
    assert!(
        readme.contains("**The default**, and the only writer."),
        "the README no longer says which mode writes; `rust` is the one that does"
    );
}

/// The README's Windows paragraph, against the CI matrix.
///
/// It said those blocks "have never been compiled" long after `windows-latest`
/// joined the matrix and the four failures that run surfaced were fixed. A
/// reader deciding whether this app is worth trying on Windows was reading the
/// state of the tree from before the work, which is the same defect class as
/// the `--allow-writes` count above: the document understated what shipped.
///
/// Only the compiled/not-compiled half is checkable. Whether anybody has *run*
/// it on a Windows machine is not a fact in this tree, so the README says so in
/// prose and this test stays out of it.
#[test]
fn the_readme_does_not_deny_a_windows_build_the_matrix_performs() {
    let workflow = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("a repository root")
            .join(".github/workflows/ci.yml"),
    )
    .expect("a CI workflow");

    if workflow.contains("windows-latest") {
        assert!(
            !readme().contains("have never been compiled"),
            "CI builds on windows-latest; the README still says those blocks have never been compiled"
        );
    }
}

/// The README now counts six things that were finished and undocumented, and a
/// count in prose is exactly what this file exists to hold.
///
/// Each row names a surface by size — seven release commands, six worktree
/// commands, seven import sources — and every one of those is a fact this tree
/// can settle. The import count in particular has drifted before: `imports.rs`
/// still had a header calling it "**Two** of them" when `ALL` carried seven.
#[test]
fn the_readme_counts_the_surfaces_it_advertises() {
    let readme = readme();
    let commands = include_str!("../src/commands.rs");

    let count = |prefix: &str| {
        commands
            .match_indices(&format!("pub fn {prefix}"))
            .count()
            .saturating_add(
                commands
                    .match_indices(&format!("pub async fn {prefix}"))
                    .count(),
            )
    };

    let release = count("release_");
    assert_eq!(
        release, 7,
        "commands.rs now has {release} release_* commands"
    );
    assert!(
        readme.contains("seven IPC commands"),
        "the README no longer says how many release commands there are"
    );

    let worktree = count("worktree_");
    assert_eq!(
        worktree, 6,
        "commands.rs now has {worktree} worktree_* commands"
    );
    assert!(
        readme.contains("six commands"),
        "the README no longer says how many worktree commands there are"
    );

    // `imports::ALL` is the declared list; its length is the number a reader is
    // told, and the two have disagreed before.
    let imports = include_str!("../src/imports.rs");
    let declared = imports
        .split_once("pub const ALL: [Source; ")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(n, _)| n.trim().to_string())
        .expect("imports.rs declares ALL with a length");
    assert_eq!(declared, "7", "imports::ALL now holds {declared} sources");
    assert!(
        readme.contains("imports from **seven** other local environments"),
        "the README no longer states the import count"
    );
}

/// The instruments that separate this product have to be reachable from an
/// assistant, not just from the window.
///
/// A review counted the surface and found the gap precisely: `request_explain`,
/// `request_timeline`, `query_log` and `profiler_flame` were written, tested,
/// registered as IPC commands and exposed on no MCP tool. So the README could
/// truthfully say the server answers "why is shop.loc not loading?" while "why
/// is it slow" — the harder question, whose answer this repository is unusual
/// for having — could not be asked at all.
///
/// This holds the four by the command they implement rather than by tool name,
/// because the tool can be renamed and the gap would be the same gap.
#[test]
fn the_four_measuring_commands_are_reachable_from_an_assistant() {
    let mcp = include_str!("../src/mcp.rs");

    // The tool table is the authority: `command:` and `also:` together are what
    // `Tool::commands()` reports, and what the websurface intersects over.
    let table = mcp
        .split_once("pub const TOOLS: &[Tool] = &[")
        .map(|(_, rest)| rest)
        .expect("mcp.rs declares a tool table");

    for command in [
        "request_explain",
        "request_timeline",
        "query_log",
        "profiler_flame",
    ] {
        assert!(
            table.contains(&format!("command: \"{command}\"")),
            "no MCP tool implements {command}; it is one of the four instruments \
             that were built and unreachable"
        );
    }
}
