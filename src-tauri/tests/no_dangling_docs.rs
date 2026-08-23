//! A comment may not send a reader to a file that is not there.
//!
//! `docs/servis-market-mimarisi.md` was a design report: it said what the
//! package system should become, in what order, and at what risk. All of it was
//! built, and its own closing section named the condition for deleting it — *a
//! design document, once the thing it describes exists, is a second source of
//! truth and it drifts.*
//!
//! What kept it alive past that point was not its content but its **citations**:
//! thirteen Rust modules, its tests and three contract files pointed at it by
//! section number (`§4.4`, `§9`, `Faz 2`). Deleting it would have left every one
//! of them addressing nothing, which is worse than a stale document — a reader
//! who cannot find the reference does not learn that the reference was wrong,
//! they learn that this repository's comments cannot be followed.
//!
//! So the citations moved to the things that are now the source, and this is
//! what stops them coming back:
//!
//! | What it said            | Where it lives now                           |
//! | ----------------------- | -------------------------------------------- |
//! | the version manifest    | `contracts/package-version.schema.json`      |
//! | the compose allowlist   | `contracts/compose-policy.json`              |
//! | the index and its chain | `contracts/registry.schema.json`             |
//! | the threat model        | `SECURITY.md`                                |
//! | every decision in it    | `docs/durum.md` §6, decisions 0011–0016, 0021, 0031, 0032 |
//!
//! ## Why the phase numbers go too
//!
//! `Faz 1`…`Faz 7` were a delivery plan, and a delivery plan is a thing that
//! stops being true by succeeding. A module doc opening with "Faz 2 of
//! <deleted file>" tells a reader the order work happened in, which the git
//! history already carries, in place of telling them what the module does. Both
//! spellings are refused here for that reason and not only because the file is
//! gone.
//!
//! ## What this deliberately does not do
//!
//! It does not check every path named in every comment — that would be a
//! link-checker, and `architecture_claims.rs` already runs one over the document
//! that carries most of them. This holds one specific promise: **this** file was
//! deleted, and nothing points at it.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

/// The file itself, and the phase labels that only meant anything inside it.
const GONE: &str = "servis-market-mimarisi";
const PHASES: [&str; 8] = [
    "Faz 0", "Faz 1", "Faz 2", "Faz 3", "Faz 4", "Faz 5", "Faz 6", "Faz 7",
];

/// Where a citation could plausibly be written: source, tests, contracts, and
/// the documents that are read as current rather than as a record.
///
/// `CHANGELOG.md` is **not** here, and that is the one exception worth stating.
/// It is a record of what was delivered and when; an entry that described the
/// document while it existed is still a true account of that release, and
/// rewriting it would be editing history to keep a test quiet. The entry that
/// records the deletion is what tells a reader scanning from the top.
fn searched() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();

    for dir in [
        root.join("src-tauri/src"),
        root.join("src-tauri/tests"),
        root.join("src-tauri/examples"),
        root.join("contracts"),
        root.join("docs"),
        root.join("src"),
        root.join("tests"),
        root.join("tools"),
    ] {
        collect(&dir, &mut out);
    }

    for file in [
        "README.md",
        "ARCHITECTURE.md",
        "SECURITY.md",
        "CONTRIBUTING.md",
    ] {
        let path = root.join(file);
        if path.is_file() {
            out.push(path);
        }
    }

    assert!(
        out.len() > 100,
        "only {} files found to search — the walk has stopped matching, and a \
         scan that reads nothing agrees with anything",
        out.len()
    );
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        if path.is_dir() {
            collect(&path, out);
            continue;
        }
        let text = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(
            text,
            "rs" | "js" | "vue" | "mjs" | "ts" | "json" | "md" | "sh"
        ) {
            out.push(path);
        }
    }
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

#[test]
fn the_deleted_design_document_is_gone_and_stays_gone() {
    let path = repo_root().join("docs/servis-market-mimarisi.md");
    assert!(
        !path.exists(),
        "docs/servis-market-mimarisi.md is back. It was deleted because a design \
         document outlives its usefulness the moment the thing it describes \
         exists; if there is something to say, say it where the code is."
    );
}

#[test]
fn nothing_cites_the_deleted_design_document() {
    let mut offenders = Vec::new();

    for path in searched() {
        // Two files name it on purpose. This one is the gate and has to say
        // what it is gating; `docs/durum.md` §1 carries the tombstone — the
        // sentence that says the document was deleted and where each part of it
        // went. A tombstone is the opposite of a dangling pointer: it is what a
        // reader who half-remembers the file needs to find.
        if path.ends_with("no_dangling_docs.rs") || path.ends_with("durum.md") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(GONE) {
            offenders.push(relative(&path));
        }
    }

    assert!(
        offenders.is_empty(),
        "these point at docs/servis-market-mimarisi.md, which does not exist: \
         {offenders:?}\nThe format is contracts/package-version.schema.json, the \
         allowlist is contracts/compose-policy.json, the threat model is \
         SECURITY.md, and the decisions are docs/durum.md §6."
    );
}

#[test]
fn nothing_still_dates_itself_by_a_delivery_phase() {
    let mut offenders = Vec::new();

    for path in searched() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // This test names the phases in its own doc comment, which is the one
        // place they are supposed to appear.
        if path.ends_with("no_dangling_docs.rs") {
            continue;
        }
        for phase in PHASES {
            if text.contains(phase) {
                offenders.push(format!("{} ({phase})", relative(&path)));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "these date themselves by a delivery phase of a plan that finished: \
         {offenders:?}\nSay what the code does, and cite the decision in \
         docs/durum.md §6 for why."
    );
}
