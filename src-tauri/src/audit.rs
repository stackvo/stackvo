//! A record of the things that cannot be taken back.
//!
//! `logging.rs` already writes a diagnostic log, and this is deliberately not
//! that. The two differ in every property that matters:
//!
//! | | diagnostic log | this |
//! | --- | --- | --- |
//! | answers | "why did it fail?" | "what was done, and when?" |
//! | contents | operations, transitions, errors | privileged and irreversible acts only |
//! | rotation | daily, oldest of 7 deleted | **none** |
//! | audience | whoever is debugging | whoever has to account for the machine |
//!
//! The rotation row is the whole point, and it is why a second file exists
//! rather than a log level. A record that deletes itself after seven days
//! cannot answer "when was this host entry added", which is precisely the
//! question asked three weeks later. An earlier review asked for a
//! separate, unrotated trail; a filter over a rotating one would have been the
//! same file with a different name.
//!
//! ## What is in it, and what is deliberately not
//!
//! Only acts that change something outside this app and cannot be undone by
//! pressing the button again:
//!
//!   * a write to `/etc/hosts`, which needs elevation;
//!   * a change to certificate trust, which edits a system store;
//!   * deleting a project, which is `remove_dir_all` over somebody's source;
//!   * writing `.env`, which reconfigures every container in the stack;
//!   * restoring a database, which replaces data that was there;
//!   * loading an image bundle, which installs something from elsewhere.
//!   * moving a credential into the OS keystore, and taking it back out, which
//!     changes where a password lives and what can still read it.
//!   * importing a site from another tool, which copies somebody's source tree
//!     into the workspace and, when asked to move, removes the original;
//!   * registering the MCP server with an assistant, and unregistering it,
//!     which edits a file belonging to another application and — with
//!     `--allow-writes` — hands that assistant the ability to stop the stack.
//!   * registering a project's own MCP server into that project's client
//!     configuration, which edits a file the repository usually commits and
//!     hands an assistant a command that runs inside a container;
//!   * generating a worktree's own Passport signing keys, which mints the
//!     credentials that branch's tokens are signed with;
//!   * declaring a browser container for Dusk, which edits the project's
//!     manifest and writes an environment file into it;
//!   * teaching that container to trust this machine's CA, which runs a
//!     command inside somebody else's image as root;
//!   * writing the AI rules into a project or a home directory, which puts text
//!     into a file the user owns — one that is usually committed, and that
//!     every future session of that assistant reads as instructions.
//!   * writing an IDE's debug configuration into a project, which is the same
//!     kind of act on the same kind of file — `launch.json` is committed, and
//!     what it configures is a debugger that attaches to running code.
//!   * writing the `PATH` entry into a shell startup file, which edits a file
//!     every shell the user opens reads, and puts a directory this app writes
//!     into ahead of the rest of their `PATH`.
//!   * installing a host tool, which is the only act here that fetches an
//!     executable over the network and leaves it somewhere a shell will run it.
//!   * taking a project's `.env` out of git, which edits that repository's
//!     index and its `.gitignore` — and is done in answer to a credential
//!     having been committed, which is exactly the kind of act somebody has to
//!     be able to date afterwards.
//!   * arming a capture window, which is a decision to write a project's
//!     request cookies and bodies — session tokens and form input — to disk so
//!     that a POST can be replayed. It is the sharpest entry in this list,
//!     because what the permission produces **is** the credential; the entry
//!     carries the project and the length and never a captured value.
//!   * moving a project's checkout to bisect it, which detaches HEAD and walks
//!     the working tree through other people's commits. It is reversible —
//!     `git bisect reset` is git's own compensation — and "my files are not
//!     what they were" is still the loudest question a developer can have about
//!     their own machine, so the answer has to be somewhere.
//!
//! Starting a container is not here, and neither is reading anything. An audit
//! trail that records routine traffic is one nobody reads, and a trail nobody
//! reads is not evidence — it is a file. The bar is "would somebody have to
//! account for this?", not "did something happen?".
//!
//! ## The one place that bar is widened, and why
//!
//! Every **writing call an assistant makes over MCP** is recorded, refusals
//! included — including `project_start`, which the paragraph above excludes.
//! The exclusion was never about the act; it was about the actor. Starting a
//! container from the window is done by the person reading this trail, and
//! they watched it happen. The same act asked for by an assistant is the one
//! nobody saw, and *"`stackvo_stack_down` was called at 14:32"* was a sentence
//! this app could not produce about the only caller that is not a person.
//!
//! Those lines carry one more thing: [`crate::undo`]'s plan, worked out
//! **before** the call ran, so the app can offer to put the act back. A
//! compensation computed later would be computed against a machine that has
//! since changed — what a `stack_down` stopped exists only before it stopped
//! it.
//!
//! ## No identity field, and that is honest rather than lazy
//!
//! This is a single-user desktop app running as the person at the keyboard. An
//! `actor` column would be the OS account repeated on every line, which reads
//! as an authorisation record and is not one — the app has no accounts, no
//! roles and nothing to authenticate against. The account that owns the file is
//! the answer, and the file lives in that account's own directory.
//!
//! ## Failures are recorded too
//!
//! An attempt that was refused is exactly as interesting as one that worked —
//! more so, in the cases anyone reviews these for. `outcome` carries which.

use std::path::{Path, PathBuf};

/// How an act ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// It happened.
    Ok,
    /// The app declined before trying — a locked setting, a failed validation.
    Refused,
    /// It was attempted and did not succeed, including a cancelled password
    /// prompt: the user was asked for elevation and said no, which is a thing
    /// somebody may need to know happened.
    Failed,
}

/// One line of the trail.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Entry {
    /// RFC 3339, UTC. Sorts lexicographically, which is what makes `grep` and
    /// `sort` usable on this file without a parser.
    pub at: String,
    /// A stable verb. Same string for the same act for ever — this is the
    /// column somebody filters on, so renaming one silently splits a history.
    pub action: &'static str,
    /// What it was done to: a domain, a project name, a path, an image tag.
    pub subject: String,
    pub outcome: Outcome,
    /// Free text for the one detail that makes the line worth reading. Never a
    /// payload: the same rule `logging.rs` states, for the same reason — a
    /// trail carrying `.env` values is one nobody can hand to anybody.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// What would put this back, worked out **before** the act — or the
    /// sentence saying why nothing would. See [`crate::undo`].
    ///
    /// Absent on every line written by the app itself, and that is the
    /// boundary rather than an omission: those acts were asked for by the
    /// person reading this screen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub undo: Option<crate::undo::Undo>,
    /// The `at` of the line this one put back.
    ///
    /// The file is append-only, so an undone act cannot be edited to say so.
    /// The undo says it instead, and the reader joins the two — which also
    /// means the record still holds both halves: that it happened, and that
    /// somebody reversed it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub undoes: Option<String>,
}

/// Where the trail lives.
///
/// Beside the logs rather than in the config directory: it is a record, not a
/// setting, and somebody collecting evidence looks where the logs are. It is
/// **not** managed by the log rotation, which only ever touches the files it
/// wrote itself.
pub fn path() -> Option<PathBuf> {
    crate::appdir::logs().map(|dir| dir.join("audit.jsonl"))
}

/// JSON Lines, one object per line.
///
/// Not a JSON array: an array has to be rewritten to be appended to, so a
/// crash mid-write costs the whole history rather than one line. Append-only
/// also means no read-modify-write, so two writers cannot lose each other's
/// entry. `atomic::write` is the right tool for a file that is replaced and
/// exactly the wrong one for a file that grows.
pub fn append_to(path: &Path, entry: &Entry) {
    use std::io::Write;

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let Ok(mut line) = serde_json::to_string(entry) else {
        return;
    };
    line.push('\n');

    // Errors are dropped, and this is the one place in the app where that is
    // the right answer rather than a shortcut. The alternative is failing a
    // `/etc/hosts` write because its audit line could not be stored, which
    // turns a record-keeping problem into an outage. The trail is evidence
    // about the app, not a participant in it.
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

/// The most recent entries, newest first.
///
/// ## Why this did not exist
///
/// The trail was written from eighteen places and read from none: of 309 IPC
/// commands, not one named `audit`. So the record that exists "for whoever has
/// to account for the machine" could only be produced by that person knowing it
/// was JSON Lines, knowing which directory the logs go in, and opening it in a
/// text editor. Writing a record nobody can be shown is most of the cost of a
/// record and none of the benefit.
///
/// ## Reading it backwards, and tolerating a bad line
///
/// The file is append-only and never rotated, so it is the one file in this app
/// that only grows — which makes "read it all and take the last N" the wrong
/// shape on the machine where this matters most. It is read as a tail: the last
/// `limit` lines, then reversed.
///
/// A line that does not parse is **skipped rather than fatal**, and that is the
/// same judgement `append_to` makes one function up. A trail is evidence about
/// the app rather than a participant in it: a half-written final line from a
/// process killed mid-append must not make the other nine thousand unreadable.
/// The count of skipped lines is returned rather than swallowed, because "this
/// file has damage in it" is itself something the person reading a trail needs
/// to be told.
pub fn tail_of(path: &Path, limit: usize) -> Trail {
    let Ok(text) = std::fs::read_to_string(path) else {
        // No file is the normal state: nothing irreversible has been done yet.
        return Trail::default();
    };

    let mut entries = Vec::new();
    let mut unreadable = 0;
    let mut total = 0;

    // Counted in one pass and parsed in another over the same borrow: the file
    // is read once and only the tail is turned into structs, which is the whole
    // reason for the shape.
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    total += lines.len();

    for line in lines.iter().rev().take(limit) {
        match serde_json::from_str::<Record>(line) {
            Ok(entry) => entries.push(entry),
            Err(_) => unreadable += 1,
        }
    }

    // An undone act cannot be edited to say so — the file is append-only — so
    // the join happens here. The undo is always *newer* than the act it
    // reverses, which is why a window that holds the act holds its undo too.
    let reversed: std::collections::BTreeSet<String> = entries
        .iter()
        .filter(|e| e.outcome == Outcome::Ok)
        .filter_map(|e| e.undoes.clone())
        .collect();
    for entry in &mut entries {
        entry.undone = reversed.contains(&entry.at);
    }

    Trail {
        entries,
        total,
        unreadable,
    }
}

/// One line, read back.
///
/// A separate shape from [`Entry`] rather than the same one deriving both
/// halves, because `Entry::action` is a `&'static str` **on purpose** — the
/// doc comment on it says the verb has to be the same string for the same act
/// for ever, and a `&'static str` is how the compiler holds that promise: the
/// only values that can reach it are literals in this crate.
///
/// Deserialising into it would require the borrow to be `'static`, which it
/// cannot be for text read off disk at runtime. Widening the field to `String`
/// to make one struct do both jobs would trade an invariant the write side
/// depends on for a struct definition the read side did not need.
///
/// So: the writer keeps its guarantee, and the reader carries an owned copy of
/// whatever was written — including a verb this build no longer emits, which is
/// exactly what reading an unrotated historical record means.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub at: String,
    pub action: String,
    pub subject: String,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo: Option<crate::undo::Undo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undoes: Option<String>,
    /// Whether a later line says this one was put back. **Derived on read**,
    /// never stored: the file is append-only, so the only honest way to know is
    /// to look for the undo that names it.
    #[serde(default)]
    pub undone: bool,
}

/// What the trail holds, and what could not be read.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trail {
    /// Newest first — the order somebody scanning for "what just happened"
    /// reads in, and the opposite of the order the file is written in.
    pub entries: Vec<Record>,
    /// Every line in the file, not just the ones returned. A screen showing
    /// fifty of nine thousand has to say so, or it reads as the whole history.
    pub total: usize,
    /// Lines that did not parse. Nearly always zero, and when it is not, the
    /// person reading the trail is the one who needs to know.
    pub unreadable: usize,
}

/// What an undo did.
///
/// Two numbers rather than a boolean: an undo is a sequence, and "four of six"
/// is the answer somebody needs when the fifth refused.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Undone {
    pub steps: usize,
    pub done: usize,
}

/// The trail, wherever it happens to live.
pub fn tail(limit: usize) -> Trail {
    path().map(|p| tail_of(&p, limit)).unwrap_or_default()
}

/// Record an act, wherever the trail happens to live.
pub fn record(action: &'static str, subject: impl Into<String>, outcome: Outcome) {
    record_with(action, subject, outcome, None);
}

/// As [`record_with`], carrying what would put the act back.
///
/// A separate function rather than two more arguments on [`record_with`],
/// which eighteen call sites use and none of which has a compensation to
/// offer: an act done by the person at the keyboard is one they can see they
/// did. This is for the surface where that is not true.
pub fn record_undoable(
    action: &'static str,
    subject: impl Into<String>,
    outcome: Outcome,
    detail: Option<String>,
    undo: Option<crate::undo::Undo>,
) {
    let Some(path) = path() else { return };
    append_to(
        &path,
        &Entry {
            at: now_rfc3339(),
            action,
            subject: subject.into(),
            outcome,
            detail,
            undo,
            undoes: None,
        },
    )
}

/// Record that one earlier line was put back.
pub fn record_undone(
    subject: impl Into<String>,
    outcome: Outcome,
    detail: Option<String>,
    undoes: &str,
) {
    let Some(path) = path() else { return };
    append_to(
        &path,
        &Entry {
            at: now_rfc3339(),
            action: "undo",
            subject: subject.into(),
            outcome,
            detail,
            // An undo carries no undo of its own. Offering one would put a
            // button in front of somebody that re-does the thing they just
            // reversed, labelled as if it were putting something back.
            undo: None,
            undoes: Some(undoes.to_string()),
        },
    )
}

/// As [`record`], with the one detail worth keeping.
pub fn record_with(
    action: &'static str,
    subject: impl Into<String>,
    outcome: Outcome,
    detail: Option<String>,
) {
    let Some(path) = path() else { return };
    append_to(
        &path,
        &Entry {
            at: now_rfc3339(),
            action,
            subject: subject.into(),
            outcome,
            detail,
            undo: None,
            undoes: None,
        },
    )
}

/// UTC, to the second, RFC 3339.
///
/// The calendar arithmetic is `crash::civil_from_days`, reused rather than
/// written again — it is the algorithm every date library uses and there is no
/// second correct version of it. The *format* differs from `crash::stamp` on
/// purpose: that one names files, where colons are illegal on Windows, and this
/// one is read by whoever reviews the trail. Both sort lexicographically, which
/// is what makes `sort` work on this file without a parser.
pub(crate) fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    rfc3339_of(secs)
}

pub(crate) fn rfc3339_of(unix_seconds: i64) -> String {
    let (year, month, day) = crate::crash::civil_from_days(unix_seconds.div_euclid(86_400));
    let time_of_day = unix_seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60,
    )
}

/// The inverse of [`rfc3339_of`], for the one caller that has to subtract two
/// timestamps rather than compare them.
///
/// Fixed-width and UTC by construction — this only ever reads strings this app
/// wrote — so it is an offset read rather than a parser, and anything that is
/// not that shape is `None` rather than a guess. A trail is evidence: a line
/// whose timestamp cannot be read is one to report, not one to interpret.
pub(crate) fn seconds_of_rfc3339(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() != 20 || bytes[4] != b'-' || bytes[10] != b'T' || bytes[19] != b'Z' {
        return None;
    }
    let at = |from: usize, to: usize| text.get(from..to)?.parse::<i64>().ok();

    let days = crate::crash::days_from_civil(
        at(0, 4)?,
        u32::try_from(at(5, 7)?).ok()?,
        u32::try_from(at(8, 10)?).ok()?,
    );
    Some(days * 86_400 + at(11, 13)? * 3600 + at(14, 16)? * 60 + at(17, 19)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-audit-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join("audit.jsonl")
    }

    pub(super) fn entry(action: &'static str, subject: &str, outcome: Outcome) -> Entry {
        Entry {
            at: "2026-08-10T09:00:00Z".into(),
            action,
            subject: subject.into(),
            outcome,
            detail: None,
            undo: None,
            undoes: None,
        }
    }

    /// The two directions of one format, held against each other.
    #[test]
    fn a_timestamp_this_app_wrote_reads_back_as_the_second_it_was() {
        for seconds in [0_i64, 1, 1_756_500_000, 1_772_000_000, 253_370_764_800] {
            let text = rfc3339_of(seconds);
            assert_eq!(
                seconds_of_rfc3339(&text),
                Some(seconds),
                "{text} did not come back as {seconds}"
            );
        }

        // Anything that is not the shape this app writes is refused rather than
        // half-read: a partial parse of a damaged line is a wrong answer where
        // "I could not read it" is a true one.
        for bad in [
            "",
            "2026-08-30",
            "2026-08-30T09:00:00",
            "2026-08-30T09:00:00+02:00",
            "not a timestamp at all",
        ] {
            assert_eq!(seconds_of_rfc3339(bad), None, "{bad:?} was read anyway");
        }
    }

    /// The join that makes an append-only file able to say "this was put back".
    #[test]
    fn an_undone_act_is_marked_by_the_line_that_undid_it() {
        let path = temp("undone");

        let mut down = entry("stackvo_stack_down", "the stack", Outcome::Ok);
        down.undo = Some(crate::undo::Undo::Steps {
            steps: vec![crate::undo::Step {
                tool: "stackvo_project_start".into(),
                arguments: serde_json::json!({ "name": "shop" }),
            }],
        });
        append_to(&path, &down);

        let mut other = entry("stackvo_project_stop", "blog", Outcome::Ok);
        other.at = "2026-08-10T09:00:01Z".into();
        append_to(&path, &other);

        let mut undone = entry("undo", "the stack", Outcome::Ok);
        undone.at = "2026-08-10T09:05:00Z".into();
        undone.undoes = Some("2026-08-10T09:00:00Z".into());
        append_to(&path, &undone);

        let trail = tail_of(&path, 10);
        let by_at = |at: &str| {
            trail
                .entries
                .iter()
                .find(|e| e.at == at)
                .expect("the line is in the trail")
                .clone()
        };

        assert!(
            by_at("2026-08-10T09:00:00Z").undone,
            "the act its undo names is not marked"
        );
        assert!(
            !by_at("2026-08-10T09:00:01Z").undone,
            "a line nothing undid was marked anyway"
        );
        // The plan survives the round trip, which is what makes the button in
        // the pane something other than a label.
        assert_eq!(
            by_at("2026-08-10T09:00:00Z").undo.expect("a plan").steps()[0].tool,
            "stackvo_project_start"
        );
        // And the record still holds both halves — that it happened, and that
        // somebody reversed it. An edit in place would have kept only one.
        assert_eq!(trail.total, 3);
    }

    /// A trail written before this field existed still reads.
    #[test]
    fn a_line_from_an_older_build_reads_without_the_new_fields() {
        let path = temp("older");
        std::fs::write(
            &path,
            r#"{"at":"2026-01-01T00:00:00Z","action":"hosts_apply","subject":"shop.loc","outcome":"ok"}
"#,
        )
        .unwrap();

        let trail = tail_of(&path, 10);
        assert_eq!(trail.unreadable, 0, "an older line was counted as damage");
        assert_eq!(trail.entries.len(), 1);
        assert!(trail.entries[0].undo.is_none());
        assert!(!trail.entries[0].undone);
    }

    /// The property that makes this a trail rather than a status file.
    #[test]
    fn entries_accumulate_instead_of_replacing_each_other() {
        let path = temp("append");

        append_to(&path, &entry("hosts_apply", "shop.loc", Outcome::Ok));
        append_to(&path, &entry("project_delete", "blog", Outcome::Ok));
        append_to(&path, &entry("hosts_apply", "api.loc", Outcome::Failed));

        let lines: Vec<_> = std::fs::read_to_string(&path)
            .expect("readable")
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("each line is one object"))
            .collect();

        assert_eq!(lines.len(), 3, "nothing overwrote anything");
        assert_eq!(lines[0]["action"], "hosts_apply");
        assert_eq!(lines[1]["subject"], "blog");
        assert_eq!(
            lines[2]["outcome"], "failed",
            "a refused or failed act is recorded, not only a successful one"
        );
    }

    /// Every line stands alone, so a torn write costs one entry.
    #[test]
    fn the_file_is_json_lines_rather_than_a_json_array() {
        let path = temp("jsonl");
        append_to(&path, &entry("env_set", "DEFAULT_PHP_VERSION", Outcome::Ok));

        let text = std::fs::read_to_string(&path).expect("readable");
        assert!(text.ends_with('\n'), "each entry is a complete line");
        assert!(
            !text.trim_start().starts_with('['),
            "an array would have to be rewritten to be appended to"
        );
        assert_eq!(text.lines().count(), 1);
    }

    /// A detail is optional and absent rather than null when there is none.
    #[test]
    fn an_absent_detail_does_not_appear_in_the_line() {
        let path = temp("detail");
        append_to(&path, &entry("cert_apply", "shop.loc", Outcome::Ok));
        append_to(
            &path,
            &Entry {
                detail: Some("user cancelled the prompt".into()),
                ..entry("hosts_apply", "shop.loc", Outcome::Failed)
            },
        );

        let lines: Vec<serde_json::Value> = std::fs::read_to_string(&path)
            .expect("readable")
            .lines()
            .map(|l| serde_json::from_str(l).expect("valid"))
            .collect();

        assert!(lines[0].get("detail").is_none());
        assert_eq!(lines[1]["detail"], "user cancelled the prompt");
    }

    /// An unwritable trail must not become an outage.
    #[test]
    fn a_trail_that_cannot_be_written_is_silent_rather_than_fatal() {
        // A path whose parent is a file, not a directory: `create_dir_all`
        // fails and so does the open.
        let dir = std::env::temp_dir().join("stackvo-audit-unwritable");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let blocker = dir.join("not-a-dir");
        std::fs::write(&blocker, "").expect("writable");

        // Returns rather than panicking. The act it was recording — writing
        // `/etc/hosts` — must not fail because its receipt could not be filed.
        append_to(
            &blocker.join("audit.jsonl"),
            &entry("hosts_apply", "shop.loc", Outcome::Ok),
        );
    }
}

/// The read side. The trail was write-only until this existed, so these are the
/// first tests that treat it as a file somebody gets shown.
#[cfg(test)]
mod read_tests {
    use super::tests::entry;
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-audit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir.join("audit.jsonl")
    }

    /// Newest first, which is the opposite of the order the file is written in.
    /// A record read in write order puts the thing that just happened at the
    /// bottom of a list that only grows.
    #[test]
    fn the_trail_comes_back_with_the_most_recent_act_first() {
        let path = scratch("order");
        for subject in ["first.loc", "second.loc", "third.loc"] {
            append_to(&path, &entry("hosts_apply", subject, Outcome::Ok));
        }

        let trail = tail_of(&path, 10);
        let subjects: Vec<&str> = trail.entries.iter().map(|e| e.subject.as_str()).collect();
        assert_eq!(subjects, ["third.loc", "second.loc", "first.loc"]);
        assert_eq!(trail.total, 3);
        assert_eq!(trail.unreadable, 0);
    }

    /// The cap is a tail, and `total` is the file. A screen that showed the cap
    /// as the history would understate what the machine has been through, which
    /// is the one direction a record must not be wrong in.
    #[test]
    fn the_limit_trims_the_list_and_the_total_still_counts_the_file() {
        let path = scratch("limit");
        for i in 0..50 {
            append_to(&path, &entry("env_write", &format!("key{i}"), Outcome::Ok));
        }

        let trail = tail_of(&path, 5);
        assert_eq!(trail.entries.len(), 5);
        assert_eq!(trail.total, 50);
        assert_eq!(trail.entries[0].subject, "key49", "not the newest five");
    }

    /// A half-written final line is what a process killed mid-append leaves,
    /// and it must not take the other entries with it. `append_to` makes the
    /// same judgement in the other direction — it drops the error rather than
    /// failing the act being recorded.
    #[test]
    fn a_damaged_line_is_skipped_and_counted_rather_than_losing_the_file() {
        let path = scratch("damage");
        append_to(&path, &entry("cert_apply", "shop.loc", Outcome::Ok));
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .expect("the file exists");
            f.write_all(b"{\"at\":\"2026-08-2")
                .expect("a truncated line");
            f.write_all(b"\n").expect("a newline");
        }
        append_to(&path, &entry("project_delete", "old", Outcome::Ok));

        let trail = tail_of(&path, 10);
        assert_eq!(trail.entries.len(), 2, "a bad line took good ones with it");
        assert_eq!(trail.unreadable, 1, "the damage was not reported");
        assert_eq!(trail.total, 3);
    }

    /// Nothing irreversible having been done yet is the normal state of a new
    /// workspace, so an absent file is an empty trail rather than an error.
    #[test]
    fn no_file_is_an_empty_trail_and_not_a_failure() {
        let trail = tail_of(Path::new("/nonexistent/stackvo/audit.jsonl"), 10);
        assert!(trail.entries.is_empty());
        assert_eq!(trail.total, 0);
        assert_eq!(trail.unreadable, 0);
    }

    /// A verb this build no longer writes still reads back. The reader carries
    /// an owned `String` precisely so an unrotated historical record stays
    /// readable across a rename that `Entry`'s `&'static str` would refuse.
    #[test]
    fn a_verb_this_build_no_longer_emits_still_reads_back() {
        let path = scratch("retired");
        std::fs::write(
            &path,
            "{\"at\":\"2024-01-01T00:00:00Z\",\"action\":\"a_verb_since_renamed\",\
             \"subject\":\"shop\",\"outcome\":\"ok\"}\n",
        )
        .expect("a historical line");

        let trail = tail_of(&path, 10);
        assert_eq!(trail.entries.len(), 1);
        assert_eq!(trail.entries[0].action, "a_verb_since_renamed");
    }
}
