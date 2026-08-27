//! The CLI against `contracts/ipc.json`, held the way `mcp.rs` is held.
//!
//! The CLI sat unbuilt because it is a **third surface** — the desktop reaches
//! the core one way, an assistant another, and a third consumer is a third
//! thing that can drift from the contract while every existing test passes.
//! Whether that was acceptable was the open question. It is, on the condition
//! this file enforces: every command names the contract command it implements,
//! and the pair is checked.
//!
//! What each check buys:
//!
//! * **A command naming nothing.** A command renamed in `ipc.json` leaves
//!   `stackvo doctor` describing something that no longer exists, and without
//!   this the first person to find out is the one who ran it.
//! * **A read that writes.** `COMMANDS` splits into "Reads" and "Changes the
//!   stack" in `--help`, and somebody reads that list before typing into a
//!   machine they care about. A command under the wrong heading is worse than
//!   no heading.
//! * **A write that reads.** The converse, and the reason it matters is the
//!   audit trail: writes are recorded, reads are not, so a mutation filed
//!   under "reads" is one that happens without a record.
//!
//! Deliberately not checked here: that the human rendering says the right
//! thing. That is settled in `cli.rs`'s own tests against fixture values,
//! because it is a claim about text rather than about the surface.

use stackvo_desktop_lib::cli::{self, COMMANDS};

fn contract() -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .join("contracts/ipc.json");
    let text = std::fs::read_to_string(&path).expect("contracts/ipc.json is readable");
    serde_json::from_str(&text).expect("the contract is valid JSON")
}

#[test]
fn every_command_names_a_real_contract_command() {
    let ipc = contract();
    let commands = ipc["commands"]
        .as_object()
        .expect("the contract declares commands");

    // `contracts()` rather than `contract()`: a screen names several and every
    // one is checked. A shell command names none and is covered by the
    // assertions further down.
    for command in COMMANDS {
        for contract in command.contracts() {
            assert!(
                commands.contains_key(contract),
                "`stackvo {}` names `{contract}`, which is not in contracts/ipc.json",
                command.name,
            );
        }
    }
}

#[test]
fn reading_commands_are_backed_by_reading_contract_commands() {
    let ipc = contract();

    // `Local` is the second exception to "every command names a contract
    // command" and it is held to its own boundary below, exactly as the shell
    // commands are. Filtered here rather than given an `if` inside the loop,
    // so the assertion below stays true of everything it does look at.
    let backed = COMMANDS
        .iter()
        .filter(|c| !c.writes)
        .filter(|c| !matches!(c.backing, cli::Backing::Local));

    for command in backed {
        assert!(
            !command.contracts().is_empty(),
            "`stackvo {}` is listed under Reads and names no contract command",
            command.name
        );
        for contract in command.contracts() {
            let kind = ipc["commands"][contract]["kind"]
                .as_str()
                .unwrap_or("unknown");
            assert!(
                matches!(kind, "query" | "stream"),
                "`stackvo {}` is listed under Reads but `{contract}` is declared `{kind}`",
                command.name,
            );
        }
    }
}

#[test]
fn writing_commands_are_backed_by_mutating_contract_commands() {
    let ipc = contract();

    // A screen is judged by whether it can change anything, not by whether
    // everything it drives does: `tui` lists as well as toggles, and demanding
    // that every name be a mutation would push the list out of the
    // declaration — which is the one place a reader sees what it touches.
    for command in COMMANDS.iter().filter(|c| c.writes) {
        let names = command.contracts();
        if names.is_empty() {
            continue; // a shell command, checked below
        }
        let kinds: Vec<&str> = names
            .iter()
            .map(|c| ipc["commands"][c]["kind"].as_str().unwrap_or("unknown"))
            .collect();
        assert!(
            kinds.iter().any(|k| matches!(*k, "mutation" | "operation")),
            "`stackvo {}` is listed as changing the stack but drives only {kinds:?}",
            command.name,
        );
    }
}

/// The gate needs something to gate.
///
/// Both halves asserted because a table that drifted to all-reads or all-writes
/// would pass the two tests above while making the split in `--help` — the one
/// thing a person reads before typing — meaningless.
#[test]
fn the_surface_has_both_kinds() {
    assert!(COMMANDS.iter().any(|c| c.writes));
    assert!(COMMANDS.iter().any(|c| !c.writes));
}

/// The MCP server and the CLI expose the same core; where they name the same
/// contract command, they must agree about whether it writes.
///
/// Not a requirement that they expose the same set — they do not, and should
/// not: `logs --follow` is a CLI answer and a useless one over JSON-RPC. What
/// would be indefensible is `compose_down` being a write in one surface and a
/// read in the other, because then one of the two is lying to somebody.
#[test]
fn the_two_surfaces_agree_about_what_writes() {
    for command in COMMANDS {
        let Some(contract) = command.contract() else {
            continue;
        };
        let Some(tool) = stackvo_desktop_lib::mcp::TOOLS
            .iter()
            .find(|t| t.command == contract)
        else {
            continue;
        };

        assert_eq!(
            command.writes, tool.writes,
            "`stackvo {}` and `{}` both drive `{contract}` and disagree about whether it writes",
            command.name, tool.name
        );
    }
}

// -------------------------------------------------- the shell commands
//
// `Backing::HostShell` is the one exception to "every command names a contract
// command", and an exception nobody checks is a hole. These four assertions are
// what keeps it a boundary rather than an escape hatch: a future command that
// wanted to skip the contract would have to be a container passthrough that
// writes, which is exactly the class the exception was opened for.

/// A screen names more than one command, or it is not a screen.
///
/// `Backing::Surface` exists because "which command does `stackvo tui`
/// implement" has no single honest answer. One that named a single command
/// should have been a `Contract`, and the looser rule above — any one of the
/// names may be the mutation — would then be a hole rather than a shape.
#[test]
fn a_screen_drives_more_than_one_command() {
    for command in COMMANDS {
        if let stackvo_desktop_lib::cli::Backing::Surface(names) = command.backing {
            assert!(
                names.len() > 1,
                "`stackvo {}` is declared a screen over one command",
                command.name
            );
        }
    }
}

/// A shell command runs **in a container**, never on the host.
///
/// This is the whole justification for the exception. `stackvo php` is defended
/// as "less dangerous than the `docker exec` you would otherwise type"; the
/// moment one of these spawned a host process that sentence stops being true,
/// and nothing else in the tree would notice.
#[test]
fn every_shell_command_runs_inside_the_container() {
    let target = cli::Target {
        name: "shop".into(),
        container: "stackvo-shop".into(),
        running: true,
        runtime: "php".into(),
        mount: Some("/var/www/html"),
        workdir: Some("/var/www/html".into()),
    };

    for command in COMMANDS.iter().filter(|c| c.passthrough()) {
        let argv = cli::exec_argv(&target, &["placeholder".to_string()], false);

        assert_eq!(
            argv.first().map(String::as_str),
            Some("exec"),
            "`stackvo {}` does not go through `docker exec`",
            command.name
        );
        assert!(
            argv.contains(&"stackvo-shop".to_string()),
            "`stackvo {}` names no container",
            command.name
        );
    }
}

/// Every shell command is classified as writing.
///
/// Not because every call changes something — `php -v` does not — but because
/// the surface cannot tell, and `--help`'s headings have to be true of every
/// call under them. It is also what keeps the read-only test above able to
/// `expect()` a contract on everything it looks at.
#[test]
fn shell_commands_are_classified_as_writing() {
    for command in COMMANDS.iter().filter(|c| c.passthrough()) {
        assert!(
            command.writes,
            "`stackvo {}` runs an arbitrary program and is not marked as writing",
            command.name
        );
        assert!(command.contract().is_none());
    }
}

// ------------------------------------------------ the local commands (A-3')
//
// `Backing::Local` is the second exception to "every command names a contract
// command", and an exception nobody checks is a hole — the same sentence the
// shell block above opens with, and the same reason it is followed by
// assertions rather than by a comment.

/// A `Local` command reaches nothing and changes nothing.
///
/// That pair is the whole of the boundary. The two that exist answer from
/// `COMMANDS` itself — the stub for a shell, and the candidates for a half-typed
/// line — and the moment one of them writes, or drives a contract command, the
/// argument for letting it skip the gate stops holding.
#[test]
fn local_commands_reach_nothing_and_change_nothing() {
    let local: Vec<_> = COMMANDS
        .iter()
        .filter(|c| matches!(c.backing, cli::Backing::Local))
        .collect();

    assert!(
        !local.is_empty(),
        "the exception exists and nothing uses it — delete Backing::Local"
    );

    for command in local {
        assert!(
            !command.writes,
            "`stackvo {}` skips the contract gate and is marked as writing",
            command.name
        );
        assert!(
            command.contracts().is_empty(),
            "`stackvo {}` is Local and names a contract command — make it Contract",
            command.name
        );
        assert!(
            !command.passthrough(),
            "`stackvo {}` cannot be both Local and a container command",
            command.name
        );
    }
}

/// The completion surface offers every command, including itself.
///
/// The failure this catches is the quiet one: a command added to the table and
/// never offered by the shell is a command nobody discovers. Asserted through
/// `candidates` rather than by reading the table twice, because the table is
/// what `candidates` reads — what is being checked is that nothing filters it.
#[test]
fn every_command_is_reachable_by_tab() {
    let names = stackvo_desktop_lib::completions::Names::default();
    let offered = stackvo_desktop_lib::completions::candidates(&[], "", &names);

    for command in COMMANDS {
        assert!(
            offered.iter().any(|c| c == command.name),
            "`stackvo {}` is in the table and never offered by tab completion",
            command.name
        );
    }
}

/// Every runtime this app generates a container for can be run in one.
///
/// `php` and `node` had a row from the start and the six other runtimes in
/// `manifest::LANG_RUNTIMES` did not, which made "run it in the container" read
/// as a PHP feature. The rule that fixed it is only worth as much as the thing
/// that holds it: a seventh runtime added to that list and forgotten here would
/// be a project this app can build and cannot open a `python -V` in.
///
/// The row is matched by its **prefix**, not by its name, because the name is a
/// spelling choice and the program is the claim. Rust's toolchain is `cargo`.
#[test]
fn every_runtime_can_be_run_in_its_own_container() {
    let programs: Vec<&str> = COMMANDS
        .iter()
        .filter(|c| c.passthrough())
        .filter_map(|c| c.prefix.first().copied())
        .collect();

    for runtime in stackvo_desktop_lib::manifest::LANG_RUNTIMES {
        // The one runtime whose binary is not the thing you type. `rust` is not
        // a program; `cargo` is what its own start command runs.
        let program = if runtime == "rust" { "cargo" } else { runtime };
        assert!(
            programs.contains(&program),
            "`{runtime}` is a runtime this app generates and `stackvo {program}` does not exist"
        );
    }

    for runtime in ["php", "node"] {
        assert!(
            programs.contains(&runtime),
            "`stackvo {runtime}` went missing"
        );
    }
}

/// Every package manager the manifest can pin can be run.
///
/// Same rule, same reason: `npm` had a row and the two Corepack also pins did
/// not, so a project that declared `pnpm` had no way to run it.
#[test]
fn every_package_manager_the_manifest_pins_can_be_run() {
    let programs: Vec<&str> = COMMANDS
        .iter()
        .filter(|c| c.passthrough())
        .filter_map(|c| c.prefix.first().copied())
        .collect();

    for manager in stackvo_desktop_lib::manifest::NODE_PACKAGE_MANAGERS {
        assert!(
            programs.contains(&manager),
            "the manifest can pin `{manager}` and `stackvo {manager}` does not exist"
        );
    }
}

/// No two container commands run the same thing.
///
/// A duplicate is not a compile error and not a runtime error — it is a second
/// row in `--help` that does what the first one does, which is how a list stops
/// being读able. `find` returns the first, so the second is simply unreachable.
#[test]
fn no_two_container_commands_run_the_same_program() {
    let mut seen: Vec<&[&str]> = Vec::new();
    for command in COMMANDS.iter().filter(|c| c.passthrough()) {
        if command.prefix.is_empty() {
            continue; // `exec` and `shell` name no program of their own.
        }
        assert!(
            !seen.contains(&command.prefix),
            "`stackvo {}` runs `{}`, which another row already runs",
            command.name,
            command.prefix.join(" ")
        );
        seen.push(command.prefix);
    }
}

/// The passthrough rule, at the parser.
///
/// `artisan migrate --force` is the most common artisan call there is, and a
/// parser that read `--force` as its own would refuse it. Asserted on the
/// parsed arguments rather than on the parser's shape, because what matters is
/// that the flag *arrives*.
#[test]
fn a_shell_commands_own_flags_survive_the_parser() {
    let argv: Vec<String> = "artisan migrate --force --seed"
        .split_whitespace()
        .map(str::to_string)
        .collect();

    let parsed = cli::parse(&argv).expect("a passthrough line parses");
    assert_eq!(parsed.command, Some("artisan"));
    assert_eq!(parsed.args, ["migrate", "--force", "--seed"]);

    // And StackVo's own flags still work — before the command, which is where
    // the parser stops looking.
    let argv: Vec<String> = "--project shop artisan migrate --force"
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let parsed = cli::parse(&argv).expect("globals before the command parse");
    assert_eq!(parsed.value("project"), Some("shop"));
    assert_eq!(parsed.args, ["migrate", "--force"]);
}

/// The prefix in the table is what actually runs.
///
/// `--help` prints `php artisan` for `stackvo artisan`; this is the assertion
/// that the two cannot drift, which is the same promise `quickcmd::Spec` makes
/// with its `display` field.
#[test]
fn the_advertised_prefix_is_the_argv_that_runs() {
    let target = cli::Target {
        name: "shop".into(),
        container: "stackvo-shop".into(),
        running: true,
        runtime: "php".into(),
        mount: Some("/var/www/html"),
        workdir: None,
    };

    for command in COMMANDS.iter().filter(|c| !c.prefix.is_empty()) {
        let argv = cli::container_argv(command, &["migrate".to_string()])
            .unwrap_or_else(|e| panic!("`stackvo {}`: {}", command.name, e.message));

        assert!(
            argv.starts_with(
                &command
                    .prefix
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            ),
            "`stackvo {}` advertises `{:?}` and runs `{argv:?}`",
            command.name,
            command.prefix
        );
        assert_eq!(argv.last().map(String::as_str), Some("migrate"));

        // And it survives the trip through `docker exec` intact.
        let full = cli::exec_argv(&target, &argv, false);
        assert!(full.ends_with(&argv), "{full:?} lost the command");
    }
}

/// A parse of every command's own spelled usage.
///
/// `--help` prints `xdebug <project> on|off`; this checks that a line shaped
/// like it actually parses, so the usage cannot describe a command line the
/// parser refuses.
#[test]
fn every_commands_own_usage_parses() {
    for command in COMMANDS.iter().filter(|c| !c.passthrough()) {
        let wanted = placeholders(command);
        let mut argv = vec![command.name.to_string()];
        argv.extend(std::iter::repeat_n("placeholder".to_string(), wanted));

        let parsed = cli::parse(&argv)
            .unwrap_or_else(|e| panic!("`stackvo {}` does not parse: {}", command.name, e.message));

        assert_eq!(parsed.command, Some(command.name));
        assert_eq!(parsed.args.len(), wanted);
    }
}

/// How many placeholder arguments to hand a command to exercise its usage line.
///
/// The maximum, except for a **variadic** one, where the maximum is
/// `usize::MAX` and `0..max` is a loop that does not finish. That is not a
/// hypothetical: adding the first variadic command turned two tests in this
/// file into an eighteen-quintillion-iteration loop that pinned two cores and
/// grew a Vec until the process was killed — a hang, which is the one failure
/// that does not look like one.
///
/// Its *spelled* count is the right number anyway: what these tests check is
/// that the usage line parses, and the usage line for a variadic command spells
/// one argument and an ellipsis.
fn placeholders(command: &cli::Command) -> usize {
    if command.arity.1 == usize::MAX {
        return command.args.split_whitespace().count();
    }
    command.arity.1
}

/// Every flag a command declares is one the parser will take from it.
///
/// A `Flag` in the table that the parser rejects would be advertised in
/// `--help` and refused when typed — the exact failure this whole file is
/// about, one layer down.
#[test]
fn every_declared_flag_is_accepted_by_the_command_that_declares_it() {
    for command in COMMANDS.iter().filter(|c| !c.passthrough()) {
        for flag in command.flags {
            let mut argv = vec![command.name.to_string()];
            argv.extend(std::iter::repeat_n(
                "placeholder".to_string(),
                placeholders(command),
            ));
            argv.push(format!("--{}", flag.long));
            if flag.value.is_some() {
                argv.push("1".to_string());
            }

            let parsed = cli::parse(&argv).unwrap_or_else(|e| {
                panic!(
                    "`stackvo {} --{}` is advertised and refused: {}",
                    command.name, flag.long, e.message
                )
            });
            assert!(
                parsed.on(flag.long),
                "--{} parsed but did not arrive",
                flag.long
            );
        }
    }
}
