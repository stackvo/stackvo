//! Tab completion for `stackvo`, generated from the table rather than written.
//!
//! DDEV ships completions and `dde` installs them as part of `system:install`;
//! this had none, and the reason it is worth a module is that the alternative
//! is worse than nothing. A hand-written completion script is a **second copy
//! of the command list**, in a language no test reads, that silently stops
//! matching the first — so a command added to [`crate::cli::COMMANDS`] is
//! invisible to the shell and a command removed from it is still offered.
//! `cli.rs` already refuses to let the CLI drift from `contracts/ipc.json`;
//! letting it drift from a `.bash` file instead would be the same mistake with
//! the gate removed.
//!
//! ## The shell side is four lines, and that is the design
//!
//! The obvious shape — emit a big script per shell, with the commands and flags
//! baked into it — puts the logic in four dialects, three of which nobody here
//! can test. So the script does not know anything. It collects what has been
//! typed and asks the binary:
//!
//! ```text
//! stackvo complete --word <partial> -- <the words before it>
//! ```
//!
//! and prints what comes back, one candidate per line. [`candidates`] is the
//! whole of the logic, it is pure, and it is tested here. Adding a shell is a
//! stub; adding a command is nothing at all.
//!
//! **The current word is passed separately and that is not decoration.** Every
//! shell disagrees about whether the word under the cursor is part of the word
//! list — bash includes an empty string for it, fish does not, PowerShell
//! sometimes does — so a protocol that inferred it from the last element would
//! behave differently in each. Naming it removes the question.
//!
//! ## What it completes, and what it deliberately does not
//!
//! Commands, flags — global and the command's own — and the positionals whose
//! placeholder names a list this app already keeps: `<project>`, `<client>`,
//! `<target>`, `<tool>`, `[shell]` and a literal alternation like `on|off`.
//! Everything else yields nothing, which is not a failure: the stubs are
//! installed with the shell's own file completion left on, so a `<path>` falls
//! through to filenames rather than to silence.
//!
//! **Nothing is completed after a passthrough.** `stackvo artisan migrate
//! --<TAB>` must not offer `--json`: that flag belongs to this binary and the
//! parser stopped at `artisan`, so offering it would suggest a thing that would
//! be handed to artisan instead. It is the same rule [`crate::cli::Command::passthrough`]
//! states, and this is the second place it has to hold.

use crate::cli::{Command, COMMANDS, GLOBAL};

/// The names only a live workspace can supply.
///
/// Passed in rather than read here, so [`candidates`] stays pure and the tests
/// need no projects directory. The caller fills it on a best-effort basis: a
/// workspace that cannot be read yields an empty list and the shell falls back
/// to filenames, which is the correct failure for a completion — an error
/// message printed into somebody's command line is not.
#[derive(Debug, Default)]
pub struct Names {
    pub projects: Vec<String>,
}

/// A shell this can generate a stub for.
///
/// The ids are [`crate::tooling::SHELLS`]' ids and are not restated: the same
/// four shells get a `PATH` line and a completion stub, from one list, so a
/// fifth cannot arrive in one place and be forgotten in the other.
pub fn stub(shell: &str, program: &str) -> Option<String> {
    Some(match shell {
        // `-o default` keeps filename completion when we return nothing, which
        // is what makes an un-modelled `<path>` positional still usable.
        // `${COMP_WORDS[@]:1:COMP_CWORD-1}` is every completed word after the
        // program name; the word under the cursor is passed separately.
        //
        // **The word list is built BEFORE `IFS` is narrowed, and that ordering
        // is the whole of this function.** The obvious spelling puts
        // `local IFS=$'\n'` at the top and expands the slice inside the
        // command substitution — and on **bash 3.2, which is the bash macOS
        // ships**, that collapses `"${COMP_WORDS[@]:1:COMP_CWORD-1}"` into a
        // single argument. `artisan migrate` arrives as one word, no command is
        // recognised in it, and the completion answers with this binary's
        // global flags — the exact thing the passthrough rule exists to
        // prevent, on the exact platform most of these users are on.
        //
        // Nothing in the Rust tests could see it: they call `candidates`
        // directly and it was right the whole time. It took driving the real
        // shell.
        "bash" => format!(
            "_{program}_complete() {{\n  \
               local typed=(\"${{COMP_WORDS[@]:1:COMP_CWORD-1}}\")\n  \
               local out\n  \
               out=$({program} complete --word \"${{COMP_WORDS[COMP_CWORD]}}\" -- \
                 \"${{typed[@]}}\" 2>/dev/null)\n  \
               local IFS=$'\\n'\n  \
               COMPREPLY=( $out )\n\
             }}\n\
             complete -o default -F _{program}_complete {program}\n"
        ),
        // zsh arrays are 1-based and `words[1]` is the program, so the completed
        // words are `words[2,CURRENT-1]` — empty when the command itself is what
        // is being typed. `compadd -- ` stops a candidate that begins with a
        // dash being read as an option to compadd.
        //
        // **Guarded on `compdef` existing, and that guard is the whole reason
        // this stub is not three lines.** `compdef` is defined by `compinit`,
        // which a lot of people never run and which oh-my-zsh runs from the
        // middle of their file. `merge` appends our block, so on a machine
        // where `compinit` has not run an unguarded `compdef` prints
        // `command not found` into every new terminal — the exact failure this
        // module exists to avoid, delivered by the fix for it.
        "zsh" => format!(
            "if (( $+functions[compdef] )); then\n  \
               _{program}_complete() {{\n    \
                 local -a reply_lines\n    \
                 reply_lines=(${{(f)\"$({program} complete --word \"${{words[CURRENT]}}\" -- \
                   \"${{(@)words[2,CURRENT-1]}}\" 2>/dev/null)\"}})\n    \
                 compadd -- $reply_lines\n  \
               }}\n  \
               compdef _{program}_complete {program}\n\
             fi\n"
        ),
        // `commandline -opc` is the tokens before the cursor, program included;
        // `-ct` is the one being typed. `-f` would disable file completion
        // outright, so it is left off and `-k` is not used: order is ours.
        "fish" => format!(
            "function __{program}_complete\n  \
               set -l typed (commandline -opc)\n  \
               set -e typed[1]\n  \
               {program} complete --word (commandline -ct) -- $typed 2>/dev/null\n\
             end\n\
             complete -c {program} -a '(__{program}_complete)'\n"
        ),
        // PowerShell hands the partial word separately already, and includes it
        // in the AST as well when it is not empty — so it is dropped from the
        // element list rather than sent twice.
        "powershell" => format!(
            "Register-ArgumentCompleter -Native -CommandName {program} -ScriptBlock {{\n  \
               param($wordToComplete, $commandAst, $cursorPosition)\n  \
               $typed = @($commandAst.CommandElements | Select-Object -Skip 1 | \
                 ForEach-Object {{ $_.ToString() }})\n  \
               if ($typed.Count -gt 0 -and $typed[-1] -eq $wordToComplete) \
                 {{ $typed = $typed[0..($typed.Count - 2)] }}\n  \
               {program} complete --word $wordToComplete -- @typed 2>$null | ForEach-Object {{\n    \
                 [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)\n  \
               }}\n\
             }}\n"
        ),
        _ => return None,
    })
}

/// What could come next, given what has been typed.
///
/// `typed` is every **completed** word after the program name; `word` is the
/// partial under the cursor, which may be empty.
pub fn candidates(typed: &[String], word: &str, names: &Names) -> Vec<String> {
    let found = command_in(typed);

    // Checked before anything else, because it is true whether or not a command
    // has been named yet: `stackvo --root <TAB>` is a path and so is
    // `stackvo logs --since <TAB>`. Answering either with the command list is
    // the kind of wrong that actively misleads — it offers a word that would be
    // consumed as the flag's value.
    if let Some(previous) = typed.last() {
        if awaits_a_value(previous, found) {
            return Vec::new();
        }
    }

    let Some(command) = found else {
        // No command yet. A flag is still a flag — `stackvo --js<TAB>` — and
        // the global list is the only one that applies before one is chosen.
        let mut out = if word.starts_with('-') {
            flag_names(GLOBAL)
        } else {
            COMMANDS.iter().map(|c| c.name.to_string()).collect()
        };
        out.retain(|item| item.starts_with(word));
        return out;
    };

    // Everything after a passthrough belongs to the program in the container.
    if command.passthrough() {
        return Vec::new();
    }

    let mut out = if word.starts_with('-') {
        let mut flags = flag_names(GLOBAL);
        flags.extend(flag_names(command.flags));
        flags.sort();
        flags.dedup();
        flags
    } else {
        positional(command, filled(typed, command), names)
    };

    out.retain(|item| item.starts_with(word));
    out
}

/// The command among the typed words, if one has been named.
///
/// A flag's *value* is skipped, because `stackvo --root status` and
/// `stackvo --root /some/status` must not both find `status`.
fn command_in(typed: &[String]) -> Option<&'static Command> {
    let mut skip = false;
    for token in typed {
        if skip {
            skip = false;
            continue;
        }
        if let Some(name) = token.strip_prefix("--") {
            // `--flag=value` carries its own value and consumes nothing after.
            if !name.contains('=') && wants_value(name) {
                skip = true;
            }
            continue;
        }
        if token.starts_with('-') {
            continue;
        }
        if let Some(found) = COMMANDS.iter().find(|c| c.name == *token) {
            return Some(found);
        }
    }
    None
}

/// Does this long flag take a value, anywhere in the surface?
///
/// Asked without knowing the command yet, because the command is what this is
/// being used to find. A false positive would swallow the command name that
/// follows; the only flags that take values are declared, so there are none.
fn wants_value(long: &str) -> bool {
    GLOBAL
        .iter()
        .chain(COMMANDS.iter().flat_map(|c| c.flags.iter()))
        .any(|flag| flag.long == long && flag.value.is_some())
}

/// Is `token` a flag whose value is the next word?
///
/// `command` is `None` before one has been named, when only the global flags
/// can apply — `stackvo --root <TAB>`.
fn awaits_a_value(token: &str, command: Option<&Command>) -> bool {
    let own = command.map(|c| c.flags).unwrap_or(&[]);
    let Some(name) = token.strip_prefix("--").filter(|n| !n.contains('=')) else {
        // A short flag with a value is spelled `-p shop`, so it counts too.
        return token
            .strip_prefix('-')
            .and_then(|s| s.chars().next())
            .is_some_and(|c| {
                GLOBAL
                    .iter()
                    .chain(own.iter())
                    .any(|f| f.short == Some(c) && f.value.is_some())
            });
    };
    GLOBAL
        .iter()
        .chain(own.iter())
        .any(|f| f.long == name && f.value.is_some())
}

/// How many positionals have already been given.
fn filled(typed: &[String], command: &Command) -> usize {
    let mut count = 0;
    let mut seen_command = false;
    let mut skip = false;

    for token in typed {
        if skip {
            skip = false;
            continue;
        }
        if token.starts_with('-') {
            if awaits_a_value(token, Some(command)) {
                skip = true;
            }
            continue;
        }
        if !seen_command {
            seen_command = token == command.name;
            continue;
        }
        count += 1;
    }
    count
}

/// The candidates for positional number `index`, from what its placeholder is.
///
/// Driven off the spelling in [`Command::args`] rather than off a second table
/// keyed by command name: the usage line is already the declaration of what a
/// command takes, and a table beside it is a thing to forget.
fn positional(command: &Command, index: usize, names: &Names) -> Vec<String> {
    let Some(slot) = command.args.split_whitespace().nth(index) else {
        return Vec::new();
    };
    let placeholder = slot.trim_matches(|c| c == '<' || c == '>' || c == '[' || c == ']');

    // A literal alternation — `on|off` — is its own answer.
    if placeholder.contains('|') {
        return placeholder.split('|').map(str::to_string).collect();
    }

    match placeholder {
        // `<container>` is `logs`' spelling and a project's container is what
        // it almost always means. A **service** container is not offered, and
        // that cap is deliberate rather than an oversight: listing those needs
        // the engine, and a completion that waits on Docker is a shell that
        // hangs when Docker is down — which is precisely when somebody is
        // typing `stackvo logs`.
        "project" | "container" => names.projects.clone(),
        "client" => crate::agents::CLIENTS
            .iter()
            .map(|c| c.id.to_string())
            .collect(),
        "target" => crate::rules::TARGETS
            .iter()
            .map(|t| t.id.to_string())
            .collect(),
        "tool" => crate::tooling::TOOLS
            .iter()
            .map(|t| t.id.to_string())
            .collect(),
        "shell" => crate::tooling::SHELLS
            .iter()
            .map(|s| s.id.to_string())
            .collect(),
        // `<path>`, `<directory>`, `<report>`, `<id>` and the rest: nothing to
        // offer, and the stubs leave file completion on for exactly this.
        _ => Vec::new(),
    }
}

/// Every spelling of a flag, long and short.
fn flag_names(flags: &[crate::cli::Flag]) -> Vec<String> {
    let mut out = Vec::new();
    for flag in flags {
        out.push(format!("--{}", flag.long));
        if let Some(short) = flag.short {
            out.push(format!("-{short}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    fn names() -> Names {
        Names {
            projects: vec!["shop".into(), "shopfront".into(), "blog".into()],
        }
    }

    #[test]
    fn a_bare_prefix_offers_the_commands_that_start_with_it() {
        let out = candidates(&[], "sta", &names());
        assert!(out.contains(&"status".to_string()), "{out:?}");
        assert!(!out.contains(&"doctor".to_string()), "{out:?}");

        // Nothing typed at all offers every command, not none.
        let all = candidates(&[], "", &names());
        assert_eq!(all.len(), COMMANDS.len());
    }

    /// The one that makes completion worth having: `stackvo logs <TAB>`.
    #[test]
    fn a_project_slot_offers_the_projects() {
        let out = candidates(&words("start"), "", &names());
        assert_eq!(out, vec!["shop", "shopfront", "blog"]);

        let narrowed = candidates(&words("start"), "shop", &names());
        assert_eq!(narrowed, vec!["shop", "shopfront"]);

        // `project <project>` and `logs <container>` are the same question
        // spelled two ways, and both have to answer it.
        assert_eq!(candidates(&words("project"), "b", &names()), vec!["blog"]);
        assert_eq!(candidates(&words("logs"), "b", &names()), vec!["blog"]);
    }

    /// The second positional is a different question from the first, and a
    /// completion that keeps offering projects for it is worse than silence.
    #[test]
    fn the_slot_advances_as_positionals_are_given() {
        // `ide <project> <ide>`: the second slot is not a project.
        let first = candidates(&words("ide"), "", &names());
        assert!(first.contains(&"shop".to_string()), "{first:?}");

        let second = candidates(&words("ide shop"), "", &names());
        assert!(!second.contains(&"shop".to_string()), "{second:?}");
    }

    /// A literal alternation is its own answer and needs no list anywhere.
    #[test]
    fn an_alternation_completes_to_its_own_words() {
        let out = candidates(&words("xdebug shop"), "", &names());
        assert_eq!(out, vec!["on", "off"]);
    }

    #[test]
    fn a_placeholder_that_names_a_list_this_app_keeps_offers_it() {
        let clients = candidates(&words("mcp-install"), "", &names());
        assert!(clients.contains(&"claude-code".to_string()), "{clients:?}");

        let shells = candidates(&words("path-install"), "", &names());
        assert!(shells.contains(&"zsh".to_string()), "{shells:?}");
        assert!(shells.contains(&"fish".to_string()), "{shells:?}");

        let tools = candidates(&words("tool-install"), "", &names());
        assert!(!tools.is_empty(), "the tool catalogue is not empty");
    }

    #[test]
    fn a_dash_asks_for_flags_and_gets_the_globals_plus_the_commands_own() {
        let out = candidates(&words("logs"), "--", &names());
        assert!(out.contains(&"--json".to_string()), "{out:?}");
        assert!(out.contains(&"--follow".to_string()), "{out:?}");
        // A flag of a command this is not is not offered.
        assert!(!out.contains(&"--move".to_string()), "{out:?}");
    }

    /// `stackvo artisan migrate --<TAB>` must not offer `--json`: the parser
    /// stopped at `artisan`, so that flag would reach artisan, not this binary.
    #[test]
    fn nothing_is_offered_after_a_passthrough() {
        for line in ["artisan", "artisan migrate", "composer", "php"] {
            assert!(
                candidates(&words(line), "--", &names()).is_empty(),
                "`{line}` offered flags"
            );
            assert!(
                candidates(&words(line), "", &names()).is_empty(),
                "`{line}` offered positionals"
            );
        }
    }

    /// `stackvo --root <TAB>` is a path, not a command, and `stackvo --root
    /// /x/status` must not be read as the `status` command.
    #[test]
    fn a_flags_value_is_not_a_command_and_not_a_positional() {
        // The word after a value-taking flag has nothing to offer.
        assert!(candidates(&words("--root"), "", &names()).is_empty());

        // And a value that happens to spell a command name is still a value.
        let out = candidates(&words("--root status"), "", &names());
        assert!(
            out.iter().any(|c| c == "logs"),
            "the command list should still be on offer: {out:?}"
        );
    }

    /// `--flag=value` consumes nothing after it, so the next word is the command.
    #[test]
    fn an_equals_flag_does_not_swallow_the_command() {
        let out = candidates(&words("--root=/tmp/x start"), "", &names());
        assert_eq!(out, vec!["shop", "shopfront", "blog"]);
    }

    /// A stub that names the wrong binary completes nothing, silently.
    #[test]
    fn every_shell_gets_a_stub_that_names_the_program() {
        for shell in crate::tooling::SHELLS {
            let script =
                stub(shell.id, "stackvo").unwrap_or_else(|| panic!("no stub for `{}`", shell.id));
            assert!(
                script.contains("stackvo complete --word"),
                "`{}`'s stub does not call the binary: {script}",
                shell.id
            );
            assert!(
                script.ends_with('\n'),
                "`{}` has no final newline",
                shell.id
            );
        }
        assert!(stub("csh", "stackvo").is_none());
    }
}
