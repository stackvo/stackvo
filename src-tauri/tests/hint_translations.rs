//! The Rust catalogues and the locale files, held equal.
//!
//! Four catalogues, one rule. `hints.rs` came first and is described below;
//! `quickcmd.rs`, `oauth.rs` and `tooling.rs` arrived later with the same gap
//! and are held by the same tests at the bottom of this file.
//!
//! `src/hints.rs` declares every suggestion this app makes; `src/i18n/locales/`
//! translates them. Nothing joins the two — one is Rust, the other is
//! JavaScript, and they are read by different runtimes at different times. That
//! is the shape of gap this project keeps closing with a test rather than a
//! convention, for the reason `readme_claims.rs` exists: a number or a key that
//! is only correct because somebody remembered is correct until they do not.
//!
//! Three failures, all silent without this:
//!
//!   * a hint added to the catalogue and never translated — the Turkish user
//!     gets English, which is the exact defect this whole change removed;
//!   * a hint deleted from the catalogue and left in the locales — dead weight
//!     that reads as coverage;
//!   * the English drifting between `hints.rs` and `en.js` — the fallback and
//!     the translation quietly saying different things, and no user ever seeing
//!     both to notice.
//!
//! ## Why the locale files are parsed rather than imported
//!
//! They are ES modules and this is a Rust test. Running `node` to print them as
//! JSON would work and would make `cargo test` need a Node toolchain — which
//! `cargo test` does not need today and should not start needing for this. The
//! `errorHints` block is generated in a fixed shape (`key: '…',` one per line),
//! so a narrow reader over exactly that block is enough, and it fails loudly
//! rather than silently if the shape ever changes: `no_keys_parsed` below.

use std::collections::{BTreeMap, BTreeSet};

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("a repository root above src-tauri")
        .to_path_buf()
}

/// `key: '…'` pairs out of an object body, whatever Prettier did to the layout.
///
/// Deliberately not line-based, which is what this was first and why it broke:
/// `prettier --write` runs on every commit and moves a value that overruns the
/// print width onto its own line. A line reader then found twelve fewer keys and
/// reported them as missing translations — a test failing for a reason that had
/// nothing to do with what it checks. Scanning for the *pair* rather than the
/// *line* is indifferent to where the newlines land.
fn parse_entries(body: &str) -> BTreeMap<String, String> {
    let bytes: Vec<char> = body.chars().collect();
    let mut out = BTreeMap::new();
    let mut i = 0;

    while i < bytes.len() {
        // An identifier followed by a colon — or a *quoted* one, which is what
        // Prettier writes for a key that is not a valid identifier. Half the
        // quick-command ids have a hyphen in them (`migrate-status`,
        // `optimize-clear`), so a reader that knew only the bare form would
        // silently drop thirteen of twenty-six and report the rest as complete.
        let key: String = if bytes[i] == '\'' || bytes[i] == '"' {
            let quote = bytes[i];
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != quote {
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }
            i = j + 1;
            bytes[start..j].iter().collect()
        } else {
            if !(bytes[i].is_ascii_alphabetic() || bytes[i] == '_') {
                i += 1;
                continue;
            }
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '_') {
                i += 1;
            }
            bytes[start..i].iter().collect()
        };

        // Whitespace, then a colon, then whitespace, then a quote. Anything
        // else and this was not an entry — a word inside a comment, most often.
        let mut j = i;
        while j < bytes.len() && bytes[j].is_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != ':' {
            continue;
        }
        j += 1;
        while j < bytes.len() && bytes[j].is_whitespace() {
            j += 1;
        }
        // Either quote. Prettier picks whichever needs fewer escapes, so
        // `"Edit it from the project's Manifest tab instead."` is double-quoted
        // while its neighbours are single-quoted — a reader that knew only one
        // style dropped exactly the entries with an apostrophe in them.
        if j >= bytes.len() || (bytes[j] != '\'' && bytes[j] != '"') {
            continue;
        }
        let quote = bytes[j];
        j += 1;

        // The value, honouring backslash escapes so a `\'` does not end it.
        let mut value = String::new();
        while j < bytes.len() && bytes[j] != quote {
            if bytes[j] == '\\' && j + 1 < bytes.len() {
                j += 1;
            }
            value.push(bytes[j]);
            j += 1;
        }
        if j >= bytes.len() {
            break; // unterminated string; the caller's emptiness check reports it
        }

        out.insert(key, value);
        i = j + 1;
    }
    out
}

/// One top-level block of a locale file, as key → text.
fn locale_block(locale: &str, name: &str) -> BTreeMap<String, String> {
    let path = repo_root()
        .join("src/i18n/locales")
        .join(format!("{locale}.js"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));

    let start = text
        .find(&format!("\n  {name}: {{"))
        .unwrap_or_else(|| panic!("{} has no {name} block", path.display()));
    let body = &text[start..];
    let end = body
        .find("\n  },")
        .unwrap_or_else(|| panic!("{} has an unterminated {name} block", path.display()));
    let body = &body[..end];

    let out = parse_entries(body);

    assert!(
        !out.is_empty(),
        "no keys parsed out of {}'s {name} block — its shape changed and this \
         test would otherwise pass by finding nothing to disagree with",
        path.display()
    );
    out
}

/// The `errorHints` block, which is what the four tests above were written for.
fn locale_hints(locale: &str) -> BTreeMap<String, String> {
    locale_block(locale, "errorHints")
}

fn catalogue() -> BTreeMap<String, String> {
    stackvo_desktop_lib::hints::ALL
        .iter()
        .map(|h| (h.key.to_string(), h.english.to_string()))
        .collect()
}

/// The failure this whole change exists to prevent, in test form.
#[test]
fn every_hint_is_translated_in_every_locale() {
    let expected: BTreeSet<String> = catalogue().keys().cloned().collect();

    for locale in ["en", "tr"] {
        let actual: BTreeSet<String> = locale_hints(locale).keys().cloned().collect();

        let missing: Vec<&String> = expected.difference(&actual).collect();
        assert!(
            missing.is_empty(),
            "{locale}.js is missing {} hint translation(s): {missing:?}\n\
             A user in this locale would be shown English for these.",
            missing.len()
        );

        let stale: Vec<&String> = actual.difference(&expected).collect();
        assert!(
            stale.is_empty(),
            "{locale}.js translates {} hint(s) nothing raises any more: {stale:?}\n\
             Delete them — a translation with no hint reads as coverage that is not there.",
            stale.len()
        );
    }
}

/// The fallback and the English translation are the same sentence.
///
/// They are two copies by necessity — Rust cannot read the locale file and the
/// webview cannot read the catalogue — so the only question is whether anything
/// notices when they diverge. This does.
#[test]
fn the_english_locale_matches_the_catalogue_word_for_word() {
    let catalogue = catalogue();
    let english = locale_hints("en");

    let mut drifted = Vec::new();
    for (key, rust) in &catalogue {
        let Some(js) = english.get(key) else {
            continue; // reported by the test above, with a better message
        };
        if rust != js {
            drifted.push(format!("  {key}\n    hints.rs: {rust}\n    en.js:    {js}"));
        }
    }

    assert!(
        drifted.is_empty(),
        "{} hint(s) say different things in Rust and in en.js:\n{}",
        drifted.len(),
        drifted.join("\n")
    );
}

/// A translation that is the English text is almost always a placeholder
/// somebody meant to come back to. A handful are legitimately identical — a
/// bare list of database names has nothing to translate — so this reports
/// rather than forbids, and the threshold is what makes it useful.
#[test]
fn the_turkish_locale_is_actually_translated() {
    let english = locale_hints("en");
    let turkish = locale_hints("tr");

    let untranslated: Vec<&String> = turkish
        .iter()
        .filter(|(key, tr)| english.get(*key).is_some_and(|en| en == *tr))
        .map(|(key, _)| key)
        .collect();

    assert!(
        untranslated.len() <= 2,
        "{} Turkish hints are identical to the English: {untranslated:?}",
        untranslated.len()
    );
}

/// Every hint in the catalogue is reachable — a `Hint` const nothing passes to
/// `with_hint` is a translation being maintained for a message no user sees.
#[test]
fn every_hint_in_the_catalogue_is_actually_raised() {
    let src = repo_root().join("src-tauri/src");
    let mut used = BTreeSet::new();

    for entry in std::fs::read_dir(&src).expect("src/").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs")
            && path.file_name().is_some_and(|n| n != "hints.rs")
        {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            for hint in stackvo_desktop_lib::hints::ALL {
                // The constant's Rust name, which is what a call site writes.
                let name = constant_name(hint.key);
                if text.contains(&format!("hints::{name}")) {
                    used.insert(hint.key);
                }
            }
        }
    }

    let unused: Vec<&str> = stackvo_desktop_lib::hints::ALL
        .iter()
        .map(|h| h.key)
        .filter(|k| !used.contains(k))
        .collect();

    assert!(
        unused.is_empty(),
        "{} hint(s) are declared and never raised: {unused:?}",
        unused.len()
    );
}

// ------------------------------------------------ the three later catalogues

/// A Rust catalogue that carries prose into the window: what it is called, the
/// locale block that translates it, and its rows as id → English.
struct Catalogue {
    /// The Rust file, for the failure message — that is where a fix goes.
    module: &'static str,
    /// The top-level key in `en.js` / `tr.js`.
    block: &'static str,
    rows: BTreeMap<String, String>,
}

/// The five that arrived after `hints.rs` with the same gap.
///
/// Keyed by the id each catalogue already carries, and that is the whole design:
/// `Spec.id`, `Provider.id`, `Tool.id` and `Recipe.name` are stable — the IPC
/// surface sends them and the tests name them — so there is no second name to
/// typo and no mapping to hold level. What is left to check is only whether the
/// two sides have the same set of ids, which is what this file does.
///
/// `provider::Edit` is the exception and carries its own key, because a
/// recipe's edit list has no ids of its own and positions are not names: the
/// two database recipes need the same instruction, and keying by position
/// would have been two translations of one sentence.
fn later_catalogues() -> Vec<Catalogue> {
    vec![
        Catalogue {
            module: "quickcmd.rs",
            block: "quickCommands",
            rows: stackvo_desktop_lib::quickcmd::CATALOG
                .iter()
                .map(|s| (s.id.to_string(), s.about.to_string()))
                .collect(),
        },
        Catalogue {
            module: "oauth.rs",
            block: "oauthNotes",
            rows: stackvo_desktop_lib::oauth::PROVIDERS
                .iter()
                .map(|p| (p.id.to_string(), p.note.to_string()))
                .collect(),
        },
        Catalogue {
            module: "tooling.rs",
            block: "toolingWhy",
            rows: stackvo_desktop_lib::tooling::TOOLS
                .iter()
                .map(|t| (t.id.to_string(), t.why.to_string()))
                .collect(),
        },
        Catalogue {
            module: "tooling.rs",
            block: "toolingOwn",
            rows: stackvo_desktop_lib::tooling::OWN
                .iter()
                .map(|(id, about)| (id.to_string(), about.to_string()))
                .collect(),
        },
        Catalogue {
            module: "provider.rs",
            block: "providerRecipes",
            rows: stackvo_desktop_lib::provider::RECIPES
                .iter()
                .map(|r| (r.name.to_string(), r.about.to_string()))
                .collect(),
        },
        Catalogue {
            module: "provider.rs",
            block: "providerRecipeEdits",
            // Flattened and de-duplicated: `connection` is shared by both
            // database recipes, which is the reason these carry a key at all.
            rows: stackvo_desktop_lib::provider::RECIPES
                .iter()
                .flat_map(|r| r.edit)
                .map(|e| (e.key.to_string(), e.english.to_string()))
                .collect(),
        },
    ]
}

/// The reader has to be finding rows, or every assertion below passes by
/// comparing two empty sets. Counts rather than "more than zero", because the
/// number is the thing that was measured: 30 quick commands, 7 providers, 4
/// tools, 2 of this repository's own binaries, 3 shipped recipes and 5 edit
/// instructions between them — fifty-one sentences printed in English to
/// everyone. (Six recipe edits are written; five are distinct, because both
/// database recipes name the same one.)
#[test]
fn the_later_catalogues_are_read_at_all() {
    let counts: Vec<(&str, usize)> = later_catalogues()
        .iter()
        .map(|c| (c.block, c.rows.len()))
        .collect();

    assert_eq!(
        counts,
        vec![
            ("quickCommands", 30),
            ("oauthNotes", 7),
            ("toolingWhy", 4),
            ("toolingOwn", 2),
            ("providerRecipes", 3),
            ("providerRecipeEdits", 5),
        ],
        "a catalogue changed size. That is fine — update the number — but it \
         has to be noticed, because a reader finding nothing is a test that \
         cannot fail."
    );
}

/// The same three failures `every_hint_is_translated_in_every_locale` catches,
/// for the catalogues that were written without a locale block at all.
#[test]
fn every_catalogue_row_is_translated_in_every_locale() {
    for catalogue in later_catalogues() {
        let expected: BTreeSet<String> = catalogue.rows.keys().cloned().collect();

        for locale in ["en", "tr"] {
            let actual: BTreeSet<String> = locale_block(locale, catalogue.block)
                .keys()
                .cloned()
                .collect();

            let missing: Vec<&String> = expected.difference(&actual).collect();
            assert!(
                missing.is_empty(),
                "{locale}.js `{}` is missing {} row(s) from {}: {missing:?}\n\
                 A user in this locale is shown English for these.",
                catalogue.block,
                missing.len(),
                catalogue.module
            );

            let stale: Vec<&String> = actual.difference(&expected).collect();
            assert!(
                stale.is_empty(),
                "{locale}.js `{}` translates {} row(s) {} no longer offers: {stale:?}\n\
                 Delete them — a translation with no row reads as coverage that is not there.",
                catalogue.block,
                stale.len(),
                catalogue.module
            );
        }
    }
}

/// The fallback and the English translation are the same sentence.
///
/// Two copies by necessity: the back end sends its English to the CLI, the MCP
/// surface and the log, and the window reads the locale file. The only question
/// is whether anything notices when they diverge.
#[test]
fn the_english_locale_matches_the_later_catalogues_word_for_word() {
    let mut drifted = Vec::new();

    for catalogue in later_catalogues() {
        let english = locale_block("en", catalogue.block);
        for (id, rust) in &catalogue.rows {
            let Some(js) = english.get(id) else {
                continue; // reported by the test above, with a better message
            };
            if rust != js {
                drifted.push(format!(
                    "  {}.{id}\n    {}: {rust}\n    en.js:   {js}",
                    catalogue.block, catalogue.module
                ));
            }
        }
    }

    assert!(
        drifted.is_empty(),
        "{} catalogue row(s) say different things in Rust and in en.js:\n{}",
        drifted.len(),
        drifted.join("\n")
    );
}

/// And that the Turkish is Turkish. Every row here is a full sentence with a
/// verb in it, so unlike the hints there is no legitimately-identical case and
/// the threshold is zero.
#[test]
fn the_turkish_later_catalogues_are_actually_translated() {
    let mut untranslated = Vec::new();

    for catalogue in later_catalogues() {
        let english = locale_block("en", catalogue.block);
        let turkish = locale_block("tr", catalogue.block);

        for (id, tr) in &turkish {
            if english.get(id).is_some_and(|en| en == tr) {
                untranslated.push(format!("{}.{id}", catalogue.block));
            }
        }
    }

    assert!(
        untranslated.is_empty(),
        "{} row(s) are still the English text: {untranslated:?}",
        untranslated.len()
    );
}

/// `startDockerOrSetHost` → `START_DOCKER_OR_SET_HOST`.
fn constant_name(key: &str) -> String {
    let mut out = String::new();
    for c in key.chars() {
        if c.is_ascii_uppercase() && !out.is_empty() {
            out.push('_');
        }
        out.push(c.to_ascii_uppercase());
    }
    out
}
