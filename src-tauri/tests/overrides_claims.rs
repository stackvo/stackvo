//! An override that only some screens honour is worse than no override.
//!
//! `crate::overrides` (ADR 0031) lets a workspace put its own copy of a
//! package file in front of the published one. The mechanism is one line in
//! `pkg::Tree::file`, and it only works for a tree that was opened **with** the
//! overrides directory attached — which `market::catalogue` is the one place
//! that does.
//!
//! The way that stops being true is quiet and looks like a tidy-up: somebody
//! adds a screen, reaches for `pkg::Tree::open(&market::dir(root))` because it
//! is the shorter spelling and it is what every other call site used to say,
//! and that screen alone reports the published bytes. The compose file renders
//! from the workspace's fragment while the connection string, the settings
//! sheet and the doctor describe a different one, and the symptom is a service
//! that behaves unlike everything said about it — the most expensive class of
//! bug this repository has a name for.
//!
//! So the rule is held here rather than in a comment: outside `pkg` itself, its
//! tests, and the helper that layers the overrides on, nothing opens a tree for
//! a workspace directly.
//!
//! ## What this cannot check
//!
//! Whether the override is the *right* file to have taken over, and whether the
//! edit in it is any good. Both are somebody's judgement, and a test that
//! pretended otherwise would be the lie `platform_matrix_claims.rs` refuses to
//! tell about its four manual counts.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every `.rs` directly under `src/`, with its name.
fn sources() -> Vec<(String, String)> {
    let dir = repo_root().join("src");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("src/ is readable")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let text = std::fs::read_to_string(&path).expect("a readable source file");
            (name, text)
        })
        .collect()
}

/// Production code only — a `#[cfg(test)]` module is allowed a bare tree, and
/// wants one: a test of the package *format* has no workspace and no overrides.
fn production(text: &str) -> String {
    match text.find("\n#[cfg(test)]\n") {
        Some(at) => text[..at].to_string(),
        None => text.to_string(),
    }
}

/// The one helper that attaches the overrides, and the only place allowed to
/// open a tree over a workspace's market directory.
#[test]
fn only_the_catalogue_helper_opens_a_tree_for_a_workspace() {
    let mut offenders = Vec::new();

    for (name, text) in sources() {
        // `pkg` defines the type; `market` is where the helper lives and is the
        // one legitimate caller of the bare constructor.
        if name == "pkg.rs" || name == "market.rs" {
            continue;
        }
        if production(&text).contains("Tree::open(&crate::market::dir(") {
            offenders.push(name);
        }
    }

    assert!(
        offenders.is_empty(),
        "these open a package tree without the workspace's overrides — use \
         `market::catalogue(root)`: {offenders:?}"
    );
}

/// The helper is still the thing that attaches them.
///
/// The rule above is only worth anything while `market::catalogue` actually
/// layers the overrides on. Deleting that one line would leave every call site
/// spelled correctly and the feature switched off everywhere at once.
#[test]
fn the_catalogue_helper_attaches_the_overrides() {
    let market = read("src/market.rs");
    assert!(
        market.contains("pub fn catalogue(root: &Path)"),
        "market::catalogue is gone; the call sites in the test above point at nothing"
    );
    assert!(
        market.contains("with_overrides(crate::overrides::dir(root))"),
        "market::catalogue no longer layers the workspace's overrides onto the tree"
    );
    assert!(
        market.contains("allows_overrides()"),
        "market::catalogue no longer consults policy.market.allowOverrides"
    );
}

/// The manifest is not an overridable file, and the contract says so.
///
/// The load-bearing rule of the whole feature: the manifest declares the image,
/// the ports and the volumes, so a workspace that could override it could run
/// one thing while the catalogue reported another. It is enforced in
/// `overrides::overridable`, which builds its list out of manifest *fields* —
/// and that is exactly the kind of thing a later refactor generalises into
/// "every file in the package directory" without noticing what it has opened.
#[test]
fn the_overridable_list_is_built_from_manifest_fields_and_never_from_a_directory() {
    let text = read("src/overrides.rs");
    let list = text
        .split("pub fn overridable(")
        .nth(1)
        .expect("overrides::overridable is still there");
    let body = &list[..list.find("\n}\n").expect("a function body")];

    for field in [
        "manifest.compose.file",
        "manifest.files",
        "manifest.companions",
    ] {
        assert!(
            body.contains(field),
            "overridable() no longer reads {field}; the list has stopped coming from the manifest"
        );
    }
    for stray in ["read_dir", "manifest.json"] {
        assert!(
            !body.contains(stray),
            "overridable() names {stray:?} — the list must come from manifest fields, never \
             from what happens to be on disk, and never include the manifest itself"
        );
    }
}

/// Nothing under `market/packages/` is written by this feature.
///
/// The property that keeps a reinstall safe and the hash chain intact. An
/// override that wrote into the package would break `pkg::verify` on the next
/// read, which is the failure the whole design exists to avoid — and the
/// tempting shortcut, because the path is right there.
#[test]
fn an_override_is_never_written_into_the_package_tree() {
    let text = production(&read("src/overrides.rs"));
    assert!(
        !text.contains("packages_dir") && !text.contains("market::dir"),
        "overrides.rs reaches into the package tree; every path it writes must be under \
         `overrides::dir(root)`"
    );
}
