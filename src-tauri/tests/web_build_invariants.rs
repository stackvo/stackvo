//! What a browser could not be handed, checked as a property of the tree.
//!
//! This file used to be `platform_matrix_claims.rs`, and most of it was a gate
//! over a table of counts in a status document: how many commands there are,
//! how many front-end files reach for Tauri, how many lines of Rust. That
//! document is gone, and the four tests that only compared a number in prose
//! against a number in the tree went with it — a measurement whose only reader
//! was the sentence stating it is a measurement nobody needed.
//!
//! Three did not depend on the document, and they are the ones worth keeping:
//! each states something about *this repository* that a reader planning a web
//! build would have to know, and each fails when the tree stops being that way.
//!
//! * `invoke` appears in exactly one file, so the transport is one function.
//! * The four commands with no meaning in a browser are still declared.
//! * A fifth has not quietly joined them.
//!
//! The last is the direction that actually goes wrong. A named list can only
//! catch a rename; it cannot catch an addition, and an addition is what happens
//! when somebody writes a command that relabels the tray.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every `.js` and `.vue` under `src/`, tests excluded.
fn front_end_files() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![repo_root().join("src")];

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
            let name = entry.file_name().to_string_lossy().to_string();
            if (name.ends_with(".js") || name.ends_with(".vue")) && !name.ends_with(".spec.js") {
                found.push(path);
            }
        }
    }

    found
}

/// One function is the whole data path.
///
/// This is the claim a web version would be planned from — change the body of
/// `call()` and no other front-end file moves — so it is checked as a property
/// of the tree rather than written down as one. A second `invoke(` anywhere
/// makes it false.
#[test]
fn invoke_appears_in_exactly_one_file() {
    let offenders: Vec<String> = front_end_files()
        .into_iter()
        .filter(|path| !path.ends_with("lib/ipc.js"))
        .filter(|path| read(path).contains("invoke("))
        .map(|path| path.display().to_string())
        .collect();

    assert!(
        offenders.is_empty(),
        "`invoke(` is supposed to appear only in src/lib/ipc.js — the whole \
         transport argument depends on it. It also appears in: {offenders:?}"
    );
}

/// The four commands that have no meaning in a browser are still declared.
///
/// Named rather than counted, because "roughly four of them" was the previous
/// version of this sentence and nobody could check it. If one is renamed, this
/// says so rather than the set quietly becoming three.
///
/// [`no_fifth_command_has_quietly_become_desktop_only`] covers the other
/// direction, which is the one that actually happens.
#[test]
fn the_desktop_only_commands_are_still_called_that() {
    let contract: serde_json::Value =
        serde_json::from_str(&read(&repo_root().join("contracts/ipc.json")))
            .expect("the contract is valid JSON");
    let commands = contract["commands"].as_object().expect("commands object");

    for name in [
        "tray_relabel",
        "window_close_action",
        "updater_status",
        "updates_check",
    ] {
        assert!(
            commands.contains_key(name),
            "`{name}` is one of the four commands a web build cannot have, and \
             the contract no longer declares it"
        );
    }
}

/// The source with every top-level `#[cfg(test)]` item removed.
///
/// The same indentation-based scan as `readme_claims.rs` and
/// `privacy_claims.rs`, for the same reason: brace counting breaks on a test
/// that writes an unmatched `{` inside a string literal, while `cargo fmt
/// --check` guarantees a top-level item closes with a `}` in column zero.
fn production_regions(src: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;

    for line in src.lines() {
        if line.starts_with("#[cfg(test)]") {
            skipping = true;
            continue;
        }
        if skipping {
            if line == "}" {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    out
}

/// The desktop-only set is derived, not remembered.
///
/// The test above holds four names. It cannot fail when a **fifth** appears,
/// and that is the only direction this ever goes: somebody adds a command that
/// relabels the tray or drives the window, and the set a web build would have
/// to answer for quietly becomes a list of four out of five.
///
/// So the set is computed. A command has no web counterpart when it reaches the
/// tray, the updater or a window, either in its own body or through a helper in
/// this same file: `window_close_action` does nothing itself and hands off to
/// `apply_close`, so a scan that did not follow one call would have missed it.
///
/// ## Why three and not four
///
/// `updates_check` is one of the contract's three `frontend-plugin` commands —
/// it is `tauri-plugin-updater`'s, not this crate's, so it has no
/// `#[tauri::command]` here to find. Asserting it separately would be asserting
/// that a dependency still exists.
///
/// ## Comments are stripped first, and the first version did not
///
/// It reported `locale_get`, which does nothing of the kind: it calls
/// `preferred_locale`, whose comment explains that the tray reads the same
/// preference. Same lesson as `policy_claims.rs` — a gate that reads prose is
/// answering a question about the documentation.
#[test]
fn no_fifth_command_has_quietly_become_desktop_only() {
    let source = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands.rs"));
    let source = production_regions(&source);
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    // Every top-level function in the file, by name, so a command that hands
    // off can be followed one step.
    let mut bodies: BTreeMap<String, String> = BTreeMap::new();
    for (offset, _) in code
        .match_indices("\nfn ")
        .chain(code.match_indices(" fn "))
    {
        let after = &code[offset..];
        let Some(rest) = after.split_once("fn ").map(|(_, rest)| rest) else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let start = offset + (after.len() - rest.len());
        let end = code[start..]
            .find("\n}\n")
            .map(|i| start + i)
            .unwrap_or(code.len());
        bodies
            .entry(name)
            .or_insert_with(|| code[start..end].to_string());
    }

    /// The window, the tray and the updater: the three things a browser tab is
    /// not. Spelled as they appear in this crate rather than as Tauri names
    /// them, because a command reaches them through this app's own modules.
    const DESKTOP: [&str; 6] = [
        "tray",
        "updater",
        "Updater",
        "TrayIcon",
        "WebviewWindow",
        "get_webview_window",
    ];

    fn reaches(name: &str, bodies: &BTreeMap<String, String>, seen: &mut BTreeSet<String>) -> bool {
        if !seen.insert(name.to_string()) {
            return false;
        }
        let Some(body) = bodies.get(name) else {
            return false;
        };
        // The function's own name is removed first. `updater_offer` decides
        // whether an install should be offered a release — pure arithmetic over
        // a manifest, nothing a browser tab lacks — and it matched the needle
        // `updater` in its own signature. A scan that reads a name as evidence
        // of what the code does would have moved it onto the list of commands a
        // web build cannot have, which is exactly the list a web port plans from.
        //
        // `updater_status` still matches, and for the right reason: its body
        // reads `plugins.updater` out of the compiled-in configuration.
        let body = body.replace(name, "");
        if DESKTOP.iter().any(|needle| body.contains(needle)) {
            return true;
        }
        // One call deep, and then as deep as that goes. Cheap because the set
        // of functions in one file is small and `seen` stops the cycles.
        bodies
            .keys()
            .filter(|callee| callee.as_str() != name && body.contains(&format!("{callee}(")))
            .any(|callee| reaches(callee, bodies, seen))
    }

    let mut found = BTreeSet::new();
    for chunk in code.split("\n#[tauri::command").skip(1) {
        let Some(rest) = chunk.split_once("fn ").map(|(_, rest)| rest) else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        if reaches(&name, &bodies, &mut BTreeSet::new()) {
            found.insert(name);
        }
    }

    let declared: BTreeSet<String> = ["tray_relabel", "updater_status", "window_close_action"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert_eq!(
        found, declared,
        "the set of commands that reach the tray, the window or the updater has \
         changed. `the_desktop_only_commands_are_still_called_that` holds the \
         named list — add the name there and here, or find out why this one \
         needs a window."
    );
}

/// The CSP and the i18n bundle have to agree, and for a long time they did not.
///
/// `tauri.conf.json` declares `default-src 'self'` and names no `script-src`,
/// so the webview refuses `new Function`. vue-i18n's **full** build compiles a
/// message by calling exactly that, on the first `t()` — which happens while
/// `App.vue` renders. The throw reached `main.js`'s error handler, the root
/// rendered a comment node, and the packaged application opened a window with
/// nothing in it.
///
/// ## Why this is a source test rather than a build one
///
/// Nothing cheap could see it. `vite preview` serves over http and applies no
/// CSP, so the Playwright suite renders that bundle nineteen times out of
/// nineteen and `npm run screenshots` photographs a working application. The
/// only surface that enforces the policy is the real webview, so the only
/// failing test was `tests/driver/`, which needs Linux, a container and twenty
/// minutes — and it could only say *"the app root never rendered any
/// children"*.
///
/// So this holds the two files against each other instead. It is a weaker
/// claim than "the bundle contains no `eval`" and it is the one that can be
/// made in milliseconds, from a fresh clone, with nothing built.
#[test]
fn a_csp_that_forbids_eval_is_matched_by_an_i18n_build_that_needs_none() {
    let conf = read(&repo_root().join("src-tauri/tauri.conf.json"));
    let csp: String = serde_json::from_str::<serde_json::Value>(&conf)
        .expect("tauri.conf.json is JSON")
        .pointer("/app/security/csp")
        .and_then(|v| v.as_str())
        .expect("tauri.conf.json declares a CSP")
        .to_string();

    // `script-src` when it is named, otherwise whatever `default-src` says —
    // which is the fallback the CSP specification defines and the reason a
    // policy with no `script-src` still governs scripts.
    let directive = csp
        .split(';')
        .map(str::trim)
        .find(|d| d.starts_with("script-src"))
        .or_else(|| {
            csp.split(';')
                .map(str::trim)
                .find(|d| d.starts_with("default-src"))
        })
        .expect("the CSP governs scripts one way or the other");

    if directive.contains("'unsafe-eval'") {
        // Then there is nothing to hold together, and saying so beats a test
        // that quietly checks nothing after somebody widens the policy.
        return;
    }

    let vite = read(&repo_root().join("vite.config.js"));
    assert!(
        vite.contains("'vue-i18n': 'vue-i18n/dist/vue-i18n.runtime"),
        "the CSP forbids eval ({directive:?}) and vue-i18n is not aliased to its \
         runtime build, so the full build's `compileToFunction` will call \
         `new Function` on the first translation and the window will render empty"
    );
    assert!(
        vite.contains("__INTLIFY_JIT_COMPILATION__: true"),
        "vue-i18n is aliased to the runtime build, whose message compiler is \
         switched on by `__INTLIFY_JIT_COMPILATION__` — undefined, it compiles \
         nothing and every translation renders as its own key"
    );
    assert!(
        vite.contains("__INTLIFY_DROP_MESSAGE_COMPILER__: false"),
        "the messages in src/i18n/locales/ are plain objects compiled at \
         runtime, so the compiler cannot be dropped"
    );
}
