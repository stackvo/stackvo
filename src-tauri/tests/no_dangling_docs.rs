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
//! section number. Deleting it would have left every one
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
//! | every decision in it    | the module header of the code it decided       |
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
//!
//! ## The second document
//!
//! `docs/durum.md` was deleted later and for a different reason: it was a
//! numbered backlog and a numbered decision register, and a number is read as
//! settled. Items were being planned around rather than re-examined, and the
//! citation outlived the thinking behind it.
//!
//! It is guarded here for the same reason the first one is. There were roughly
//! six hundred references to that document — its path, its numbered decisions
//! and its numbered sections — and a cleanup that large is only worth doing
//! once. Without a gate the first citation written from memory puts it back.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

/// The two deleted documents, and the phase labels that only meant anything
/// inside the first.
///
/// Spelled without a directory or an extension so a citation is caught however
/// it was written — as a link, as a bare filename, or in the middle of a
/// sentence.
const GONE: [&str; 2] = ["servis-market-mimarisi", "durum.md"];
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
fn the_deleted_documents_are_gone_and_stay_gone() {
    for (path, why) in [
        (
            "docs/servis-market-mimarisi.md",
            "a design document outlives its usefulness the moment the thing it \
             describes exists",
        ),
        (
            "docs/durum.md",
            "a numbered backlog is read as settled, and its items were being \
             planned around rather than re-examined",
        ),
    ] {
        assert!(
            !repo_root().join(path).exists(),
            "{path} is back. It was deleted because {why}; if there is something \
             to say, say it where the code is."
        );
    }
}

#[test]
fn nothing_cites_the_deleted_design_document() {
    let mut offenders = Vec::new();

    for path in searched() {
        // This file names them on purpose: it is the gate and has to say what
        // it is gating. (`docs/isler.md` used to be the second exemption, as
        // the report that recorded the deletions; it was itself retired after
        // v0.2.0, its remaining items moved to issues #98–#104, and
        // `CHANGELOG.md` — exempted elsewhere — now carries that record.)
        if path.ends_with("no_dangling_docs.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for gone in GONE {
            if text.contains(gone) {
                offenders.push(format!("{} ({gone})", relative(&path)));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "these cite a document that does not exist: {offenders:?}\n\
         For the package design: the format is \
         contracts/package-version.schema.json, the allowlist is \
         contracts/compose-policy.json, and the threat model is SECURITY.md.\n\
         For the status document: say what the code does and why, in the module \
         header beside it. There is no numbered item to cite any more, which is \
         the point."
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
         {offenders:?}\nSay what the code does and why. The plan these date \
         themselves by is finished, and the git history is where its order \
         lives."
    );
}
