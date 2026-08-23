//! The two commands this app ships beside itself, and the three files that
//! have to agree about them.
//!
//! `tooling.rs` offers to link `stackvo` and `stackvo-mcp` onto the user's
//! `PATH`, and `agents.rs` registers the second with six assistants. Both used
//! to find nothing on an installed app, because nothing bundled them — the
//! instruction was "clone the repository and run cargo", which is not something
//! you can tell somebody who downloaded a `.dmg`.
//!
//! They are `externalBin` now, and that spreads one fact across three places:
//! the list in `tauri.conf.json` that the bundler reads, the list in
//! `tools/sidecars.mjs` that builds the files, and the list in `tooling.rs`
//! that the interface offers to link. A sidecar added to one and not the others
//! is built and never shipped, or shipped and never offered — both silent.
//!
//! ## Why this is source-reading rather than a bundle check
//!
//! The thing worth checking is whether a bundle *would* carry them, and no test
//! on a developer's machine can produce one: `tauri build` takes minutes, needs
//! the platform's bundler, and in CI runs in a different job. What goes wrong
//! is not the bundler; it is the three lists drifting apart. That is a text
//! comparison, and a text comparison runs everywhere in milliseconds.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// The `externalBin` entries, as the bundler reads them.
fn declared() -> Vec<String> {
    let conf: serde_json::Value =
        serde_json::from_str(&read("src-tauri/tauri.conf.json")).expect("tauri.conf.json parses");

    conf["bundle"]["externalBin"]
        .as_array()
        .expect("bundle.externalBin is an array — without it nothing ships beside the app")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("every externalBin entry is a string")
                .to_string()
        })
        .collect()
}

#[test]
fn the_bundle_carries_both_commands() {
    let declared = declared();

    for name in ["stackvo", "stackvo-mcp"] {
        assert!(
            declared.iter().any(|p| p == &format!("binaries/{name}")),
            "tauri.conf.json does not bundle {name}. The Tooling pane offers to \
             link it onto the user's PATH; on an installed app it would find \
             nothing.\nDeclared: {declared:?}"
        );
    }
}

#[test]
fn the_builder_and_the_bundler_name_the_same_binaries() {
    let script = read("tools/sidecars.mjs");

    // The `SIDECARS` array, read the way it is written rather than by parsing
    // JavaScript: a single line of string literals, which is also why the
    // script keeps it on one line.
    let line = script
        .lines()
        .find(|l| l.contains("export const SIDECARS"))
        .expect("tools/sidecars.mjs declares SIDECARS");

    for entry in declared() {
        let name = entry
            .strip_prefix("binaries/")
            .expect("every externalBin entry lives in binaries/");
        assert!(
            line.contains(&format!("'{name}'")),
            "tauri.conf.json bundles {name} and tools/sidecars.mjs does not build \
             it — the bundler would fail on a file nothing writes.\n{line}"
        );
    }
}

#[test]
fn the_pane_offers_exactly_what_the_bundle_carries() {
    let tooling = read("src-tauri/src/tooling.rs");

    for entry in declared() {
        let name = entry.strip_prefix("binaries/").unwrap();
        assert!(
            tooling.contains(&format!("(\"{name}\", ")),
            "{name} is bundled and tooling::OWN does not list it, so nothing \
             links it onto PATH"
        );
    }
}

/// The build hook that stands between a placeholder and a release.
///
/// `tools/sidecars.mjs` writes a text placeholder so that `tauri-build`'s
/// existence check passes and cargo can build the real binaries — the cycle is
/// written up in that file. The placeholder is harmless exactly as long as
/// something refuses to bundle one, and that something is `--verify` in
/// `beforeBuildCommand`. Removing it would leave every `tauri build` free to
/// ship a `stackvo` that is a shell script exiting 1.
#[test]
fn a_placeholder_cannot_reach_a_bundle() {
    let conf: serde_json::Value =
        serde_json::from_str(&read("src-tauri/tauri.conf.json")).expect("tauri.conf.json parses");

    let before = conf["build"]["beforeBuildCommand"]
        .as_str()
        .expect("beforeBuildCommand is a string");

    assert!(
        before.contains("sidecars.mjs --verify"),
        "beforeBuildCommand no longer verifies the sidecars, so a `tauri build` \
         on any path can bundle a placeholder: {before}"
    );

    let dev = conf["build"]["beforeDevCommand"]
        .as_str()
        .expect("beforeDevCommand is a string");
    assert!(
        dev.contains("sidecars.mjs"),
        "beforeDevCommand no longer builds the sidecars, so `tauri dev` fails on \
         a missing externalBin before it starts: {dev}"
    );
}

/// The placeholder must never be mistakable for a build.
#[test]
fn the_placeholder_announces_itself_and_fails() {
    let script = read("tools/sidecars.mjs");

    assert!(
        script.contains("exit 1"),
        "the placeholder no longer exits non-zero — one that escaped into a \
         bundle would fail silently instead of loudly"
    );
    assert!(
        script.contains("LEAST_REAL_BYTES"),
        "the size floor that tells a placeholder from a build is gone"
    );
}
