//! The rules file that tells an assistant *when* to use the MCP server.
//!
//! [`crate::agents`] registers `stackvo-mcp` with the assistants on this
//! machine, which makes the tools reachable. Reachable is not the same as used.
//! An assistant asked "why is shop.loc not loading?" in a project it has never
//! seen will read the source, guess at nginx, and suggest editing a generated
//! file — because nothing told it that `stackvo_doctor` exists and answers that
//! exact question in one call.
//!
//! ServBay ships this as "AI Rule" and files it beside the MCP documentation
//! as a first-class feature; EnvKit installs a skill; Lerd's
//! `mcp:enable-global` "writes context files so assistants understand available
//! capabilities". It was the one part of the competitive picture this
//! repository had no answer to at all.
//!
//! ## Six files, not six clients
//!
//! A row here is **a file**, not a product, because the products share files:
//! Codex and Zed both read `AGENTS.md`, and listing them separately would offer
//! two buttons that write the same bytes to the same path and disagree about
//! which one is installed. [`TARGETS`] is keyed on the path for that reason.
//!
//! ## The same three rules `agents.rs` follows
//!
//! **Read, replace one region, write back.** The file is `CLAUDE.md` — the
//! user's own instructions to their own assistant. Everything outside our
//! markers comes back byte for byte, including whatever they wrote yesterday.
//!
//! **Markers, not a whole file.** [`BEGIN`] and [`END`] are HTML comments, so
//! they are invisible in every renderer these files pass through and legal in
//! all of them. A file with no markers is appended to, never replaced.
//!
//! **One backup beside it.** `.stackvo-backup`, rewritten rather than
//! accumulated, exactly as the client-configuration writer does it.
//!
//! ## Why the text is English
//!
//! Everything on this surface is. The rule file is read by a model, ends up in
//! a repository that may have contributors who do not share the author's
//! locale, and — like the hints that reach the MCP client — has one canonical
//! wording that a test can check. `ARCHITECTURE.md` states the same rule for
//! the MCP surface generally.

use crate::error::{Code, Error, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The markers our region lives between.
///
/// HTML comments because every one of these files is Markdown, and a Markdown
/// renderer drops them. A user who opens `CLAUDE.md` in a preview sees the
/// rules and not the plumbing.
pub const BEGIN: &str = "<!-- stackvo:rules:begin -->";
pub const END: &str = "<!-- stackvo:rules:end -->";

/// The suffix of the copy taken before a file is rewritten. The same one
/// [`crate::agents`] uses, for the same reason.
pub const BACKUP_SUFFIX: &str = ".stackvo-backup";

/// Where a rules file is read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    /// Inside one repository. Every client supports this, and it is the shape
    /// that survives being cloned onto somebody else's machine.
    Workspace,
    /// The user's home, applying to every session. Only some clients read one.
    Global,
}

impl Scope {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "workspace" => Ok(Scope::Workspace),
            "global" => Ok(Scope::Global),
            other => Err(Error::new(
                Code::InvalidInput,
                format!("`{other}` is not a scope — use `workspace` or `global`"),
            )),
        }
    }
}

/// One rules file, and the clients that read it.
pub struct Target {
    pub id: &'static str,
    /// Shown in the pane. Product names, so not translated — the same decision
    /// [`crate::agents::Client::label`] makes.
    pub label: &'static str,
    /// Relative to the directory the rules are being written into.
    pub workspace: &'static str,
    /// The home-relative path this client reads globally, when it reads one.
    ///
    /// `None` is the common case and is not a gap in this code: Cursor, VS Code
    /// and Windsurf scope their rules to a workspace by design, and writing a
    /// file none of them opens would be the failure `agents.rs` was built to
    /// avoid.
    pub global: Option<&'static str>,
    /// What has to sit above our block for the client to apply the file at all.
    ///
    /// Cursor's `.mdc` and VS Code's `.instructions.md` are inert without it —
    /// a rule with no `alwaysApply` is one the model may never be shown. It is
    /// written only when the file is created, because a user who narrowed
    /// `applyTo` to their PHP directories meant that.
    pub front_matter: Option<&'static str>,
}

/// Every rules file this writes.
///
/// `AGENTS.md` carries two products because both read it — see the module note
/// on why a row is a file rather than a client.
pub const TARGETS: &[Target] = &[
    Target {
        id: "claude",
        label: "Claude Code",
        workspace: "CLAUDE.md",
        global: Some(".claude/CLAUDE.md"),
        front_matter: None,
    },
    Target {
        id: "agents",
        label: "Codex & Zed (AGENTS.md)",
        workspace: "AGENTS.md",
        // Codex honours CODEX_HOME, so this one is resolved rather than joined
        // — see `global_path`.
        global: Some(".codex/AGENTS.md"),
        front_matter: None,
    },
    Target {
        id: "cursor",
        label: "Cursor",
        workspace: ".cursor/rules/stackvo.mdc",
        global: None,
        front_matter: Some(
            "---\ndescription: StackVo local development environment\nalwaysApply: true\n---\n",
        ),
    },
    Target {
        id: "copilot",
        label: "VS Code & GitHub Copilot",
        workspace: ".github/instructions/stackvo.instructions.md",
        global: None,
        front_matter: Some("---\napplyTo: '**'\n---\n"),
    },
    Target {
        id: "windsurf",
        label: "Windsurf",
        workspace: ".windsurf/rules/stackvo.md",
        global: None,
        front_matter: Some("---\ntrigger: always_on\n---\n"),
    },
    Target {
        id: "gemini",
        label: "Gemini CLI",
        workspace: "GEMINI.md",
        global: Some(".gemini/GEMINI.md"),
        front_matter: None,
    },
];

pub fn target(id: &str) -> Option<&'static Target> {
    TARGETS.iter().find(|t| t.id == id)
}

/// Where this target's global file is on this machine.
///
/// `None` when the client reads no global file, or when there is no home
/// directory to resolve against — the one case where there is no answer rather
/// than a wrong one.
pub fn global_path(id: &str) -> Option<PathBuf> {
    let target = target(id)?;
    let relative = target.global?;
    let home = dirs::home_dir()?;

    // Codex's home is configurable and the machine this was written on sets it.
    // Joining `~/.codex` would write a file that installation does not read,
    // which is exactly the failure `agents::config_candidates` documents.
    if id == "agents" {
        let base = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| home.join(".codex"));
        return Some(base.join("AGENTS.md"));
    }

    Some(home.join(relative))
}

/// The file this target writes for one scope.
pub fn path_for(id: &str, scope: Scope, dir: Option<&Path>) -> Option<PathBuf> {
    let target = target(id)?;
    match scope {
        Scope::Workspace => Some(dir?.join(target.workspace)),
        Scope::Global => global_path(id),
    }
}

// ----------------------------------------------------------------- the rules

/// What every rules file says.
///
/// Written once and rendered into six files, so the guidance cannot drift
/// between the assistant somebody uses at work and the one they use at home.
///
/// The content is what a model gets wrong without being told, in the order it
/// gets it wrong: it does not know the tools exist; it edits generated files
/// because they look like the real ones; it proposes `docker compose` by hand;
/// and it treats a write tool as free. Everything here is checked by a test in
/// this module or is a name the MCP table also carries.
pub fn text() -> String {
    let mut out = String::new();
    out.push_str(
        "## StackVo local environment\n\
         \n\
         This machine runs its local development stack under **StackVo**: Docker containers \
         generated from `.env` and each project's `stackvo.json`, reachable over HTTPS at \
         per-project domains. A `stackvo` MCP server is registered with this assistant.\n\
         \n\
         ### Ask the stack before you read the code\n\
         \n\
         For anything about the *environment* rather than the *application*, call a tool first. \
         Reading source to guess at the environment is slower and usually wrong.\n\
         \n\
         | Question | Call |\n\
         | --- | --- |\n\
         | Is anything working at all? | `stackvo_overview` |\n\
         | Why did the stack fail to start / what holds port 3306? | `stackvo_doctor` |\n\
         | Why does one site not load? | `stackvo_project`, then `stackvo_hosts` and \
         `stackvo_certificates` |\n\
         | What did the application log say? | `stackvo_log_files`, then `stackvo_log_read` |\n\
         | Why did the container die? | `stackvo_logs` |\n\
         | What is running, and on which version? | `stackvo_services`, \
         `stackvo_service_instances` |\n\
         | How do I connect to the database? | `stackvo_service_connection` |\n\
         | Did the application send that mail? | `stackvo_mail`, then `stackvo_mail_message` |\n\
         | Why is everything slow? | `stackvo_system`, then `stackvo_container_stats` |\n\
         | Why is *this page* slow? | `stackvo_profiler` |\n\
         | Why does my breakpoint not hit? | `stackvo_ide_debug` |\n\
         \n\
         `stackvo_project` already carries the certificate check, the Xdebug state and the PHP \
         limits **as the running container reports them** — so a question about an upload \
         failing at a limit that looks raised in `php.ini` is answered there, not by reading \
         `php.ini`.\n\
         \n\
         ### Never edit generated files\n\
         \n\
         Anything under the generated directory — `docker-compose*.yml`, the per-project \
         `Dockerfile`, the nginx and PHP configuration — is derived output and is overwritten \
         without warning. Change the **input** instead: the project's `stackvo.json`, or `.env`. \
         Then regenerate. A change made in the output survives until the next generate and no \
         longer.\n\
         \n\
         ### Do not drive Docker by hand\n\
         \n\
         Do not propose `docker compose up`, `docker run`, or editing the hosts file directly. \
         The stack has an order — generate, then bring profiles up — and the tools and the \
         `stackvo` CLI keep it. A container started outside that order holds a name and a port \
         that the next generate expects to own.\n\
         \n\
         ### Writing tools\n\
         \n\
         The server is **read-only unless it was started with `--allow-writes`**. When the \
         writing tools are present, say what you are about to do and get agreement before \
         calling one. These are not free:\n\
         \n\
         - `stackvo_stack_down` stops **everything**, including every project.\n\
         - `stackvo_service_stop` stops a shared service — every project using it loses it.\n\
         - `stackvo_generate` overwrites all generated output.\n\
         - `stackvo_xdebug_set` needs a rebuild afterwards before it takes effect.\n\
         \n\
         Every writing call you make here is recorded in StackVo's audit trail with what it \
         was done to and how it ended, refusals included, and most of them carry what would \
         put them back — so a person can reverse one from the app. That is not a reason to be \
         careless: an undo is a sequence and some acts have none.\n\
         \n\
         The tools you can see are the whole of what this server will do. It may have been \
         started for one project only, or for a fixed length of time, and then the writing \
         tools it offers are fewer than the twelve above — that is a decision somebody made \
         deliberately. If a call is refused for being out of scope or out of time, say so and \
         stop; do not work around it, and do not propose restarting the server with wider \
         flags unless you are asked to.\n\
         \n\
         Before anything that could change data — a migration, a seeder, a destructive query — \
         take a snapshot with `stackvo_snapshot_take` first. Restoring one is deliberately not a \
         tool; it is done in the app, by a person.\n\
         \n\
         ### Credentials\n\
         \n\
         No tool on this surface returns a password, and there is no argument that asks one to. \
         Passwords are in the project's own `.env` and in the keystore. If you need one, ask the \
         user rather than looking for a tool that will hand it over.\n",
    );
    out
}

/// The block as it is written into a file, markers included.
pub fn block() -> String {
    format!("{BEGIN}\n{}\n{END}\n", text().trim_end())
}

// ------------------------------------------------------------- pure editing
//
// Strings in, strings out, no disk — the same shape `agents::insert` takes, and
// for the same reason: this is where the promise not to destroy somebody's file
// is kept, so it is where the tests can reach.

/// The bounds of our region in `text`, if it has one.
fn region(text: &str) -> Option<(usize, usize)> {
    let start = text.find(BEGIN)?;
    let end = text[start..].find(END)? + start + END.len();
    Some((start, end))
}

/// `text` with our block inserted or replaced, and nothing else changed.
pub fn merge(text: &str, front_matter: Option<&str>) -> String {
    let block = block();

    if let Some((start, end)) = region(text) {
        let mut out = String::with_capacity(text.len() + block.len());
        out.push_str(&text[..start]);
        out.push_str(block.trim_end());
        out.push_str(&text[end..]);
        return out;
    }

    if text.trim().is_empty() {
        // A new file. This is the only moment the front matter is written: it
        // configures *when the file applies*, and a user who narrowed that
        // meant it.
        return match front_matter {
            Some(matter) => format!("{matter}\n{block}"),
            None => block,
        };
    }

    // An existing file with no block of ours. Appended, never prepended: what
    // is already at the top of somebody's CLAUDE.md is the part they consider
    // most important.
    let mut out = text.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(&block);
    out
}

/// `text` without our region, and without the blank line it left behind.
pub fn strip(text: &str) -> String {
    let Some((start, end)) = region(text) else {
        return text.to_string();
    };

    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..start]);
    out.push_str(text[end..].trim_start_matches('\n'));

    // A file that held nothing but our block becomes empty rather than a stray
    // newline; a file that held more keeps its own trailing newline.
    let trimmed = out.trim_end_matches([' ', '\t', '\n']);
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

/// Whether `text` carries our block, and whether it is the current wording.
pub fn state(text: &str) -> (bool, bool) {
    let Some((start, end)) = region(text) else {
        return (false, false);
    };
    (true, text[start..end].trim() == block().trim())
}

// -------------------------------------------------------------------- status

/// One rules file on this machine.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetStatus {
    pub id: String,
    pub label: String,
    pub scope: Scope,
    /// Absolute, so a reader can open it — including when this refuses to.
    pub path: String,
    pub exists: bool,
    /// The file carries a StackVo block.
    pub installed: bool,
    /// That block is the wording this version would write. False on a block
    /// left by an older release, which is the case a "Update" button exists for.
    pub current: bool,
}

/// Every rules file, in both scopes. Never writes.
///
/// `dir` is the directory workspace rules would go into — a project, or the
/// workspace root. `None` leaves the workspace half out rather than inventing a
/// directory to report on.
pub fn status(dir: Option<&Path>) -> Vec<TargetStatus> {
    let mut out = Vec::new();

    for target in TARGETS {
        for scope in [Scope::Workspace, Scope::Global] {
            let Some(path) = path_for(target.id, scope, dir) else {
                continue;
            };
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let (installed, current) = state(&text);

            out.push(TargetStatus {
                id: target.id.to_string(),
                label: target.label.to_string(),
                scope,
                path: path.display().to_string(),
                exists: path.is_file(),
                installed,
                current,
            });
        }
    }

    out
}

// --------------------------------------------------------------------- write

/// Read, edit and write one rules file, keeping a copy of what was there.
fn rewrite(path: &Path, edit: impl FnOnce(&str) -> String) -> Result<String> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let updated = edit(&existing);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }

    // Only when there was something to lose — an empty `.stackvo-backup` reads
    // as a configuration that was eaten.
    if !existing.is_empty() {
        let mut name = path.file_name().unwrap_or_default().to_os_string();
        name.push(BACKUP_SUFFIX);
        let backup = path.with_file_name(name);
        std::fs::write(&backup, &existing)
            .map_err(|e| Error::io(format!("writing {}", backup.display()), e))?;
    }

    crate::atomic::write(path, &updated)?;
    Ok(path.display().to_string())
}

/// Write the rules into one target. Returns the file written.
pub fn apply(id: &str, scope: Scope, dir: Option<&Path>) -> Result<String> {
    let Some(target) = target(id) else {
        return Err(Error::new(
            Code::InvalidInput,
            format!("unknown rules target {id}"),
        ));
    };
    let Some(path) = path_for(id, scope, dir) else {
        return Err(Error::new(
            Code::NotFound,
            format!(
                "{} has no {scope:?} rules file on this machine",
                target.label
            ),
        ));
    };

    rewrite(&path, |text| merge(text, target.front_matter))
}

/// Take the block back out. The file itself is left, minus our region — it is
/// the user's file and may hold nothing else only by coincidence.
pub fn remove(id: &str, scope: Scope, dir: Option<&Path>) -> Result<String> {
    let Some(path) = path_for(id, scope, dir) else {
        return Err(Error::new(
            Code::NotFound,
            format!("no rules file is known for {id}"),
        ));
    };
    if !path.is_file() {
        return Ok(path.display().to_string());
    }

    rewrite(&path, strip)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A counter rather than a timestamp, for the reason `agentctx` documents:
    /// two tests on two threads can read the same nanosecond.
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    fn scratch() -> PathBuf {
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("stackvo-rules-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The promise the whole module exists to keep.
    #[test]
    fn everything_outside_the_markers_survives() {
        let original = "# My rules\n\nAlways use tabs.\n";
        let merged = merge(original, None);

        assert!(merged.starts_with("# My rules\n\nAlways use tabs.\n"));
        assert!(merged.contains(BEGIN) && merged.contains(END));

        // And a second pass replaces the block rather than stacking another.
        let again = merge(&merged, None);
        assert_eq!(again.matches(BEGIN).count(), 1, "{again}");
        assert!(again.starts_with("# My rules\n\nAlways use tabs.\n"));
    }

    /// Removing ours gives back what was there, which is the only definition of
    /// "non-destructive" that means anything.
    #[test]
    fn removing_the_block_restores_the_file() {
        let original = "# My rules\n\nAlways use tabs.\n";
        assert_eq!(strip(&merge(original, None)), original);
    }

    /// A file that held only our block goes back to empty rather than to a
    /// lone newline — the next `merge` on it must take the new-file path.
    #[test]
    fn a_file_that_was_only_ours_ends_up_empty() {
        assert_eq!(strip(&merge("", None)), "");
    }

    /// The front matter is what makes Cursor's and VS Code's files apply at
    /// all, and it is written exactly once.
    #[test]
    fn front_matter_is_written_on_creation_and_never_again() {
        let matter = "---\nalwaysApply: true\n---\n";
        let created = merge("", Some(matter));
        assert!(created.starts_with(matter), "{created}");

        // The user narrowed it. A second apply must not put ours back.
        let narrowed = created.replace("alwaysApply: true", "globs: src/**");
        let again = merge(&narrowed, Some(matter));
        assert!(again.contains("globs: src/**"), "{again}");
        assert!(!again.contains("alwaysApply: true"), "{again}");
        // Lines that are exactly `---`, because the rules themselves contain a
        // Markdown table whose separator row is full of them.
        let fences = again.lines().filter(|line| *line == "---").count();
        assert_eq!(fences, 2, "one front matter block");
    }

    #[test]
    fn state_distinguishes_absent_from_stale() {
        assert_eq!(state("# nothing here\n"), (false, false));
        assert_eq!(state(&merge("", None)), (true, true));

        let stale = format!("{BEGIN}\nold wording\n{END}\n");
        assert_eq!(state(&stale), (true, false));
    }

    /// Every target that declares front matter has it fenced properly, because
    /// a malformed one is not ignored by these clients — it is rendered into
    /// the prompt as text.
    #[test]
    fn declared_front_matter_is_well_formed() {
        for target in TARGETS {
            let Some(matter) = target.front_matter else {
                continue;
            };
            assert!(matter.starts_with("---\n"), "{}", target.id);
            assert!(matter.ends_with("---\n"), "{}", target.id);
        }
    }

    /// One row per file. Two rows writing one path would be two buttons
    /// disagreeing about whether the rules are installed.
    #[test]
    fn no_two_targets_write_the_same_workspace_file() {
        let mut seen = std::collections::HashSet::new();
        for target in TARGETS {
            assert!(
                seen.insert(target.workspace),
                "{} writes a path another target already writes",
                target.id
            );
            assert!(!target.workspace.starts_with('/'), "{}", target.id);
        }
    }

    /// The rules name tools, and a rule naming a tool that does not exist sends
    /// an assistant to call something the server will refuse. This is the check
    /// that keeps the two files honest with each other.
    #[test]
    fn every_tool_the_rules_name_is_a_tool_that_exists() {
        let text = text();
        let mut named = 0;

        for fragment in text.split('`') {
            if !fragment.starts_with("stackvo_") {
                continue;
            }
            named += 1;
            assert!(
                crate::mcp::TOOLS.iter().any(|t| t.name == fragment),
                "the rules name `{fragment}`, which is not a tool"
            );
        }

        assert!(named > 10, "the rules stopped naming tools: {named}");
    }

    /// The safety paragraph is the reason this file is worth writing at all —
    /// an assistant that reads the tool table and skips the warnings is the
    /// state we were already in.
    #[test]
    fn the_rules_warn_about_the_tools_that_take_the_stack_down() {
        let text = text();
        for claim in [
            "stackvo_stack_down",
            "--allow-writes",
            "stackvo_snapshot_take",
            "Never edit generated files",
        ] {
            assert!(text.contains(claim), "the rules no longer say {claim:?}");
        }
    }

    #[test]
    fn applying_and_removing_round_trips_on_disk() {
        let dir = scratch();
        std::fs::write(dir.join("CLAUDE.md"), "# Mine\n").unwrap();

        let written = apply("claude", Scope::Workspace, Some(&dir)).unwrap();
        assert!(written.ends_with("CLAUDE.md"));

        let after = std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
        assert_eq!(state(&after), (true, true));
        // The copy of what was there.
        assert_eq!(
            std::fs::read_to_string(dir.join("CLAUDE.md.stackvo-backup")).unwrap(),
            "# Mine\n"
        );

        remove("claude", Scope::Workspace, Some(&dir)).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap(),
            "# Mine\n"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A nested path is created rather than reported missing: nobody has
    /// `.github/instructions/` until something makes it.
    #[test]
    fn a_nested_target_creates_its_directories() {
        let dir = scratch();
        apply("copilot", Scope::Workspace, Some(&dir)).unwrap();

        let path = dir.join(".github/instructions/stackvo.instructions.md");
        assert!(path.is_file());
        assert!(std::fs::read_to_string(&path).unwrap().contains("applyTo"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_reports_both_scopes_and_leaves_workspace_out_without_a_directory() {
        let global_only = status(None);
        assert!(global_only.iter().all(|s| s.scope == Scope::Global));
        assert!(!global_only.is_empty(), "some client reads a global file");

        let dir = scratch();
        let both = status(Some(&dir));
        assert_eq!(
            both.iter().filter(|s| s.scope == Scope::Workspace).count(),
            TARGETS.len()
        );
        assert!(both.iter().all(|s| !s.installed));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_workspace_apply_with_no_directory_is_refused_rather_than_guessed() {
        let error = apply("claude", Scope::Workspace, None).unwrap_err();
        assert_eq!(error.code, Code::NotFound);
    }

    #[test]
    fn scopes_parse_and_anything_else_is_an_error() {
        assert_eq!(Scope::parse("workspace").unwrap(), Scope::Workspace);
        assert_eq!(Scope::parse("global").unwrap(), Scope::Global);
        assert!(Scope::parse("machine").is_err());
    }
}
