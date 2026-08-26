//! What has to change on the day `LEGACY_SERVICES` is deleted.
//!
//! §3 #36 of `docs/durum.md` is not blocked on code. `config::LEGACY_SERVICES`
//! — 150 of the 186 embedded defaults — exists so [`handover`] can read a
//! pre-market `.env` and turn `SERVICE_MYSQL_ENABLE=true` into an instance with
//! a version, a port and a volume. It goes when no supported workspace still
//! needs migrating, and *that* is a release decision rather than an engineering
//! one.
//!
//! What is engineering, and what this file is: the deletion must not be an
//! archaeology exercise six months from now. Two things make it mechanical —
//! the keys are one constant instead of 150 lines mixed into 36 others, and the
//! modules that read one are named here. A new reader is a change that makes
//! the eventual deletion bigger, so it has to be written down rather than
//! discovered by whoever attempts it.
//!
//! ## What a reader is, and the two this file used to miss
//!
//! The first version of this gate recognised one spelling: a call to one of the
//! seven `Env` accessors named in [`ACCESSORS`]. It was green, and it was
//! wrong by two modules. `db.rs` keeps a per-engine table of `.env` key names
//! (`password: "SERVICE_MYSQL_ROOT_PASSWORD"`) and passes them to `Env::get`
//! and `Env::bool`; `mail.rs` builds `SERVICE_MAILPIT_ENABLE` from a prefix
//! constant and a suffix. Both read a legacy default on every call, and
//! neither ever writes one of the seven names — so the checklist said four
//! modules while the tree held six, and the document repeated the four.
//!
//! Naming the key is therefore the second spelling, and it is the one that
//! cannot be avoided: a module that reads a `SERVICE_*` default has to say
//! which one somewhere. So a file is a reader if it calls an accessor **or**
//! names a `LEGACY_SERVICES` key — in full, or as the prefix a `format!`
//! completes.
//!
//! The bare family prefix `"SERVICE_"` is excluded, and only that one. It is a
//! prefix of every key, so counting it would make a reader of `template.rs`,
//! whose `PREFIXES` list decides which *variable names* the renderer
//! substitutes and names no service at all. Anything longer names a service,
//! which is the thing deletion day has to go and look at.
//!
//! ## Why file granularity
//!
//! `commands.rs` is fifteen thousand lines with several test modules scattered
//! through it, so "which function" cannot be answered by reading text. The
//! honest unit is the file, and the claim each row makes is a sentence about
//! why that file reads a legacy default at all — which is the thing a reader on
//! deletion day actually needs.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

fn durum() -> String {
    let path = repo_root().join("../docs/durum.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Everything that reads a `SERVICE_*` default out of an [`Env`].
///
/// `service_prefix` is in the list even though it only builds a string: a
/// caller that formats a key by hand and passes it to `get` is doing the same
/// thing one indirection later, and this is the spelling every such caller in
/// this crate happens to use.
const ACCESSORS: [&str; 7] = [
    "service_enabled",
    "service_version",
    "service_versions",
    "service_url",
    "service_host_port",
    "service_credentials",
    "service_prefix",
];

/// The declared readers, and why each one is a reader.
///
/// Ordered by what deletion day does to them: `config.rs` loses the constant
/// and the accessors, `handover.rs` loses its reason to exist, and the other
/// three lose a branch each.
const READERS: [(&str, &str); 5] = [
    (
        "config.rs",
        "defines the constant and the accessors; the deletion starts here",
    ),
    (
        "handover.rs",
        "the migration itself — this is the reason the constant is still here",
    ),
    (
        "commands.rs",
        "the pre-migration branches: `list_services` falls back to `.env` when \
         there is no instance table, and the traefik routes are rendered from \
         the same fallback",
    ),
    (
        "preset.rs",
        "`export` describes a stack, and an unmigrated stack is still described \
         by `.env`",
    ),
    (
        "db.rs",
        "`Kind::keys` is a table of `.env` key names — root password, database, \
         user, enable — and `settings`, `settings_for_instance` and `targets` \
         all fall back through it, because the handover deliberately leaves a \
         migrated instance's credentials in `.env`",
    ),
];

/// Production code only.
///
/// The same indentation-based scan as `platform_matrix_claims.rs`,
/// `readme_claims.rs` and `privacy_claims.rs`, for the same reason: brace
/// counting breaks on a test that writes an unmatched `{` inside a string
/// literal, while `cargo fmt --check` guarantees a top-level item closes with a
/// `}` in column zero.
fn production_regions(src: &str) -> String {
    let mut kept = String::with_capacity(src.len());
    let mut from = 0;

    while let Some(offset) = src[from..].find("\n#[cfg(test)]") {
        let start = from + offset + 1;
        kept.push_str(&src[from..start]);
        match src[start..].find("\n}\n") {
            Some(end) => from = start + end + 3,
            None => return kept,
        }
    }

    kept.push_str(&src[from..]);
    kept
}

/// The keys of `config::LEGACY_SERVICES`, read out of the source text.
///
/// Text rather than `stackvo_desktop_lib::config::LEGACY_SERVICES`, and the
/// reason is deletion day: a test that links the constant stops **compiling**
/// when the constant goes, which is a build error in a file whose whole job is
/// to be readable on that morning. Read as text it simply returns an empty set,
/// the literal rule matches nothing, and what is left is the accessor rule —
/// which is the correct answer once there is no legacy half.
fn legacy_keys() -> BTreeSet<String> {
    let source = read("src/config.rs");
    let start = source
        .find("pub const LEGACY_SERVICES")
        .expect("config.rs still declares LEGACY_SERVICES");
    let body = &source[start..];
    let end = body.find("\n];").map(|e| e + 3).unwrap_or(body.len());

    let mut keys = BTreeSet::new();
    for literal in service_literals(&body[..end]) {
        keys.insert(literal.to_string());
    }
    keys
}

/// Every `"SERVICE_…"` string literal in a chunk of source.
///
/// Deliberately naive — it looks for the opening `"SERVICE_` and takes
/// everything to the next `"`. A key with an escaped quote in it would be a key
/// no shell could set, so there is nothing to be cleverer about.
fn service_literals(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut from = 0;

    while let Some(offset) = text[from..].find("\"SERVICE_") {
        let start = from + offset + 1;
        match text[start..].find('"') {
            Some(end) => {
                found.push(&text[start..start + end]);
                from = start + end + 1;
            }
            None => break,
        }
    }

    found
}

fn is_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!")
}

/// A line that calls one of the accessors, rather than one that mentions it.
///
/// Doc comments name these functions in several modules — `connect.rs` points
/// at `service_credentials` to explain its own masking — and counting those
/// would put files on the list that read nothing. Prose is not a reader.
fn calls_an_accessor(line: &str) -> bool {
    if is_comment(line) {
        return false;
    }
    ACCESSORS.iter().any(|name| line.contains(name))
}

/// A line that names a legacy key outright.
fn names_a_legacy_key(line: &str, keys: &BTreeSet<String>) -> bool {
    if is_comment(line) {
        return false;
    }
    service_literals(line)
        .into_iter()
        .any(|literal| keys.contains(literal))
}

/// The suffixes a file appends to a prefix, as `format!("{}_ENABLE", …)` does.
///
/// Only the shape where the prefix comes first, because that is the shape
/// `Env::service_prefix` is built for — it returns `SERVICE_MYSQL_` and every
/// caller in this crate completes it.
fn appended_suffixes(source: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut from = 0;

    while let Some(offset) = source[from..].find("\"{}") {
        let start = from + offset + 3;
        match source[start..].find('"') {
            Some(end) => {
                let suffix = &source[start..start + end];
                if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_uppercase() || c == '_')
                {
                    found.insert(suffix.to_string());
                }
                from = start + end + 1;
            }
            None => break,
        }
    }

    found
}

/// A file that builds a legacy key out of a prefix literal and a suffix.
///
/// This is the second spelling, and `mail.rs` is why it has to be this exact:
/// the module writes `"SERVICE_MAILPIT"` and completes it with
/// `format!("{}_ENABLE", …)`. A rule that counted any literal which is *a
/// prefix of some legacy key* would have been simpler and would have been
/// wrong in both directions — it called `mail.rs` a reader on the strength of
/// `SERVICE_MAILPIT_URL` existing, a key that module has never read, and it
/// went on calling it one after the enables left the constant. Pairing the
/// prefix with the suffixes the file actually appends answers the real
/// question: does *this* file build a key that *is* in the legacy half.
fn builds_a_legacy_key(source: &str, keys: &BTreeSet<String>) -> bool {
    let suffixes = appended_suffixes(source);
    if suffixes.is_empty() {
        return false;
    }

    source
        .lines()
        .filter(|line| !is_comment(line))
        .flat_map(service_literals)
        .filter(|literal| *literal != "SERVICE_")
        .any(|prefix| {
            suffixes
                .iter()
                .any(|suffix| keys.contains(&format!("{prefix}{suffix}")))
        })
}

/// Every `.rs` under `src/`, including `src/bin/`.
///
/// Recursive, and it had to become so: the first version read `src/` alone, so
/// the two binaries — the CLI and the MCP server, both of them surfaces a user
/// reaches — could have started reading a legacy default without this list
/// noticing. Neither does today. That is a fact the gate should establish, not
/// one it should assume.
fn source_files() -> Vec<String> {
    let root = repo_root().join("src");
    let mut stack = vec![root.clone()];
    let mut found = Vec::new();

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let relative = path
                .strip_prefix(&root)
                .expect("the walk started at src/")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("/");
            found.push(relative);
        }
    }

    found.sort();
    found
}

fn readers_in_tree() -> Vec<String> {
    let keys = legacy_keys();
    let mut found = Vec::new();

    for relative in source_files() {
        let source = production_regions(&read(&format!("src/{relative}")));
        let reads = source
            .lines()
            .any(|line| calls_an_accessor(line) || names_a_legacy_key(line, &keys))
            || builds_a_legacy_key(&source, &keys);
        if reads {
            found.push(relative);
        }
    }

    found.sort();
    found
}

/// The list is the whole list.
///
/// Fails in both directions on purpose. A new reader is the case this exists
/// for; a reader that has *gone* matters just as much, because a row nobody
/// removed is a row somebody will go looking for on deletion day and not find.
#[test]
fn only_the_declared_modules_read_a_legacy_service_default() {
    let found = readers_in_tree();
    let mut declared: Vec<String> = READERS
        .iter()
        .map(|(file, _)| (*file).to_string())
        .collect();
    declared.sort();

    assert_eq!(
        found, declared,
        "the set of modules reading a `SERVICE_*` default has changed.\n\
         Every one of them is work on the day `config::LEGACY_SERVICES` is \
         deleted (§3 #36), so the list in this file is the checklist for that \
         day — add or remove the row, with the sentence that says why."
    );
}

/// Both spellings of "reads a legacy default" are actually being looked for.
///
/// The gate was green for three rounds while missing `db.rs` and `mail.rs`,
/// because it only knew the accessor spelling. This asserts the rule that found
/// them still finds one, so nobody narrows the scan back to accessors and sees
/// green.
///
/// `mail.rs` is now the other half of the same lesson and is asserted from the
/// opposite side. It *was* a reader, on `SERVICE_MAILPIT_ENABLE`, until the
/// decision in §5 pointed `detect` at the catalogue and those keys left the
/// constant. A prefix rule loose enough to catch it originally went on calling
/// it a reader afterwards — `SERVICE_MAILPIT_URL` is still in the legacy half
/// and `"SERVICE_MAILPIT"` is a prefix of it — which is a module handed to
/// whoever runs the deletion with nothing to do. Precision in both directions,
/// or the checklist is a guess with a test around it.
#[test]
fn naming_a_key_counts_as_reading_one() {
    let keys = legacy_keys();
    assert!(
        !keys.is_empty(),
        "no `SERVICE_*` keys were parsed out of config.rs — the scan that finds \
         the second kind of reader is looking at nothing"
    );

    // Names the key outright, and never calls an accessor: without the literal
    // rule this module is invisible.
    let db = production_regions(&read("src/db.rs"));
    assert!(
        db.lines().any(|l| names_a_legacy_key(l, &keys)),
        "db.rs no longer names a legacy key. If it stopped reading `.env` its \
         row in READERS has to go; if it only changed spelling, this gate has \
         to learn the new one — it was written because \
         `\"SERVICE_MYSQL_ROOT_PASSWORD\"` is invisible to the accessor scan."
    );
    assert!(
        !db.lines().any(calls_an_accessor),
        "db.rs now calls an accessor, so the literal rule is no longer what \
         keeps it on the list. Point this test at whatever module the literal \
         rule is now the only thing catching, or the rule can be deleted \
         without anything going red."
    );

    // Builds a prefix and completes it — and completes it into a key that is
    // *not* legacy any more, so it is not a reader.
    let mail = production_regions(&read("src/mail.rs"));
    assert!(
        mail.lines()
            .flat_map(service_literals)
            .any(|l| l != "SERVICE_"),
        "mail.rs no longer names a service prefix at all, so it can no longer \
         show that a loose prefix rule would mis-file it"
    );
    assert!(
        !builds_a_legacy_key(&mail, &keys),
        "mail.rs is building a legacy key again. Either `detect` went back to \
         reading `.env` presence — the thing §5 decided against — or a key it \
         completes has returned to `LEGACY_SERVICES`. Add its row back."
    );

    // And the one literal that must not count. `template.rs` names the family
    // prefix to decide which variables the renderer substitutes; it reads no
    // service default, and a rule that called it a reader would put a module on
    // deletion day's checklist that has nothing to do on it.
    assert!(
        !names_a_legacy_key("    \"SERVICE_\",", &keys),
        "the bare family prefix is being counted as a key, which makes a reader \
         of every module that mentions the prefix"
    );
}

/// The migration still reads the defaults — and is no longer why they are here.
///
/// This test used to be called `the_migration_is_still_the_reason_the_constant_
/// exists`, and the name was the claim: `LEGACY_SERVICES` says in prose that
/// the migration is why it survives, so a `handover.rs` that stopped reading it
/// would leave that sentence as the last thing standing between the constant
/// and its deletion.
///
/// `npm run legacy:rehearse` disproved the claim by doing the deletion. With
/// the legacy half emptied, `handover_equivalence.rs` passes **13 of 13** —
/// every image, port and volume still preserved, the refusals still refusing.
/// The migration does not need these defaults, because it does not need them to
/// be *defaults*: `plan` reads what the `.env` states, and where it states
/// nothing it asks the catalogue (`catalogue.recommended`), which is what ADR
/// 0016 made services dynamic for.
///
/// So the assertion stays — `handover.rs` is a reader and belongs on the
/// checklist — and the sentence it was named after is gone. What actually holds
/// the constant up is measured and lives in the rehearsal's expected list; the
/// shortest version is `mail.rs`, which asks whether a key is *present* rather
/// than what it says.
#[test]
fn the_migration_still_reads_the_defaults() {
    let handover = production_regions(&read("src/handover.rs"));

    for accessor in ["service_enabled", "service_version"] {
        assert!(
            handover.lines().filter(|l| l.contains(accessor)).any(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("///")
            }),
            "handover.rs no longer calls `{accessor}`. If the migration has \
             stopped reading `.env`, it is not a reader and its row in READERS \
             has to go — the deletion-day checklist is shorter by a module."
        );
    }
}

/// The two halves are named where the deletion will be planned.
///
/// The measurement itself is §7's and `platform_matrix_claims.rs` holds the
/// numbers. This holds something smaller and easier to lose: that §3's row
/// names the constant, so a reader who arrives at the item can find the code
/// without grepping for a phrase.
#[test]
fn the_document_names_the_constant_it_is_waiting_to_delete() {
    let doc = durum();

    assert!(
        doc.contains("LEGACY_SERVICES"),
        "docs/durum.md never names `config::LEGACY_SERVICES`, which is the \
         constant §3 #36 is about"
    );
}

// ------------------------------------------------------------ what §3 #36 says

/// The §3 row for #36, as one line.
fn item_36_row(doc: &str) -> &str {
    doc.lines()
        .find(|line| line.starts_with("| 36 |"))
        .expect("docs/durum.md §3 still has a row for #36")
}

/// The §4 bullet for #36, all of it.
///
/// Not the first line. §4's bullets are wrapped prose and a claim can sit on
/// the third line as easily as the first — reading one line found the marker
/// and then failed on a version stated two lines below it, which is a gate
/// failing for its own reason rather than the document's.
fn item_36_bullet(doc: &str) -> String {
    let lines: Vec<&str> = doc.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.starts_with("* **#36**"))
        .expect("docs/durum.md §4 still has a bullet for #36");
    let mut bullet = vec![lines[start]];
    for line in &lines[start + 1..] {
        if line.starts_with("* ") || line.trim().is_empty() || line.starts_with('#') {
            break;
        }
        bullet.push(line);
    }
    bullet.join("\n")
}

/// The §5 line that answered the question #36 was waiting on.
fn item_36_answer(doc: &str) -> &str {
    doc.lines()
        .find(|line| line.contains("`LEGACY_SERVICES` hangi sürümde"))
        .expect("docs/durum.md §5 still carries the version question for #36")
}

/// The size of the deletion, where the deletion is described.
///
/// §7's measurement table holds how many *keys* go; this holds how many
/// *modules* have work on the day, which is the number a person planning it
/// reads off §3. It said four while the tree held six — the same drift §7 was
/// built to stop, one table over. A count is a property of the code even
/// though "not done" is not (§8), so this one is gated too.
///
/// The digit rather than the Turkish word, and the failure message says so: a
/// gate cannot recompute `dört`.
#[test]
fn the_document_states_how_many_modules_the_deletion_touches() {
    let doc = durum();
    let row = item_36_row(&doc);
    let expected = format!("**{}** modül", READERS.len());

    assert!(
        row.contains(&expected),
        "§3 #36 does not say `{expected}`, and READERS lists {} module(s). \
         Anyone planning the deletion reads the size off that row.\nRow:\n{row}",
        READERS.len()
    );
}

// ------------------------------------------------------------ the deletion date

/// The release by which the question has to be answered (§5).
///
/// This was set as "the release migration support ends", and that is not what
/// it is any more. `npm run legacy:rehearse` deletes the constant and runs the
/// suite: `handover_equivalence.rs` passes 13 of 13 without it, so migration
/// was never what the date was protecting. What the constant actually supplies
/// is `.env` *presence* as the answer to "which services does this workspace
/// know about" — `mail.rs` reads exactly that — and whether the catalogue
/// should be that answer instead is a product decision, not a schedule.
///
/// The vocabulary question has since been answered (ADR 0037) and took
/// seventy-two keys with it. What is left is a smaller thing and a real one:
/// roughly twenty-seven of the seventy-eight are live credentials `db.rs` reads
/// for instances that already migrated, so the constant still cannot be deleted
/// wholesale, and separating them costs `SETTINGS` its prefix-only membership
/// rule.
///
/// The date stays because that trade still needs deciding. Never deciding means
/// carrying seventy-eight keys, five reader modules and six test sites
/// indefinitely on a reason nobody has re-examined — which is precisely how the
/// *wrong* reason survived three rounds. A version is the only thing that makes
/// that impossible; what changed is the sentence it forces somebody to read.
const LEGACY_SERVICES_GO_AT: (u64, u64) = (0, 4);

/// The date, as a build failure.
///
/// A date written only in prose is a date that passes. This fails the build on
/// the first commit that bumps the app to 0.4.0 while `LEGACY_SERVICES` is
/// still there — which is exactly when somebody has to decide whether to delete
/// it or to move the date on purpose, and either is fine as long as it is a
/// decision rather than a thing that did not happen.
#[test]
fn the_constant_is_gone_by_the_version_that_was_named_for_it() {
    let conf: serde_json::Value =
        serde_json::from_str(&read("tauri.conf.json")).expect("tauri.conf.json parses");
    let version = conf["version"]
        .as_str()
        .expect("the app declares a version");

    let mut parts = version.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));

    let still_here = read("src/config.rs").contains("pub const LEGACY_SERVICES");
    let (go_major, go_minor) = LEGACY_SERVICES_GO_AT;
    let due = (major, minor) >= (go_major, go_minor);

    assert!(
        !(due && still_here),
        "the app is at {version} and `config::LEGACY_SERVICES` is still \
         declared. §5 answered §3 #36 with {go_major}.{go_minor}: from that \
         release a workspace waiting to be migrated is no longer supported.\n\n\
         The checklist is the READERS table above — {} module(s), each with the \
         sentence saying why it reads a legacy default. Delete them, or move \
         LEGACY_SERVICES_GO_AT and write down what changed the answer.",
        READERS.len()
    );

    // And the other direction: the date has not quietly been moved past the
    // point where it means anything. A constant nobody can reach is not a plan.
    assert!(
        (go_major, go_minor) < (1, 0) || !still_here,
        "the deletion was pushed to {go_major}.{go_minor}, at or past 1.0.0. \
         Carrying the second catalogue into a stable release is the outcome §5 \
         chose against — if it is now the right answer, it needs the paragraph, \
         not the constant."
    );
}

/// The gate and the three places that promise it say the same version.
///
/// This is the half that was missing, and it is the half the item's own
/// argument turns on: §3 #36 says "not prose but a gate", and yet the gate was
/// `LEGACY_SERVICES_GO_AT` in one file and `0.4.0` in three paragraphs of
/// another, with nothing between them. Moving the constant to `(0, 9)` left
/// every sentence in the document saying 0.4.0 and every test green — the
/// deletion would simply have stopped being due, silently, which is the exact
/// outcome the version was introduced to make impossible.
///
/// All three places, not one. §3 is the item, §4 is the order somebody plans
/// from, and §5 is where the question was answered; a document that fixed one
/// and left the others is how #35's stale sentence survived
/// (`durum_sections_agree.rs`).
#[test]
fn the_document_states_the_version_the_gate_is_set_to() {
    let doc = durum();
    let (go_major, go_minor) = LEGACY_SERVICES_GO_AT;
    let version = format!("{go_major}.{go_minor}.0");

    for (where_, line) in [
        ("§3, the item's row", item_36_row(&doc)),
        ("§4, the suggested order", &item_36_bullet(&doc)),
        ("§5, where the question was answered", item_36_answer(&doc)),
    ] {
        assert!(
            line.contains(&version),
            "{where_} does not say {version}, which is what \
             LEGACY_SERVICES_GO_AT is set to. The gate and the promise have to \
             be one fact — a document naming a different release is a date \
             nobody is held to.\nLine:\n{line}"
        );
    }
}

// ------------------------------------------------- what the deletion actually costs

/// The rehearsal's expected sites are sites this tree still has.
///
/// `tools/legacy-deletion-rehearsal.mjs` performs the deletion and compares the
/// failures against a list of files and the sentence saying why each one is on
/// it. That list is the other half of this checklist — the half a grep cannot
/// produce, because a test can lean on an embedded default without naming a
/// key: `preflight.rs` asks `commands.rs` a question and expects phpMyAdmin to
/// be on, and `mail.rs` asks whether a key is present at all.
///
/// The rehearsal is the thing that keeps that list honest, and it takes a full
/// compile plus a full suite, so it is a command rather than a test. What *is*
/// cheap is this: every file it names still exists. A row pointing at a module
/// somebody renamed is a wrong instruction handed to whoever runs the deletion,
/// and it would sit there unnoticed until the morning it mattered.
#[test]
fn the_rehearsal_plans_against_files_this_tree_still_has() {
    let tool = std::fs::read_to_string(repo_root().join("../tools/legacy-deletion-rehearsal.mjs"))
        .expect("the rehearsal tool is readable");

    let mut named = Vec::new();
    let mut from = 0;
    while let Some(offset) = tool[from..].find("file: '") {
        let start = from + offset + "file: '".len();
        let end = start
            + tool[start..]
                .find('\'')
                .expect("an unterminated path literal");
        named.push(tool[start..end].to_string());
        from = end;
    }

    assert!(
        !named.is_empty(),
        "the rehearsal tool names no files. Either its EXPECTED table changed \
         shape or this scan is reading nothing — and a gate that checks nothing \
         reports green."
    );

    for path in &named {
        assert!(
            repo_root().join("..").join(path).exists(),
            "the rehearsal expects {path} to fail when the legacy half goes, and \
             this tree has no such file. Fix the row, or the deletion-day \
             checklist sends somebody to a module that is not there."
        );
    }
}
