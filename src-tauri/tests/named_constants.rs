//! The three values that were written in fourteen places, held to one each.
//!
//! Not tidiness. Each of the three had **two derivations that disagreed**, and
//! the disagreement was measurable rather than theoretical:
//!
//!   * `DEFAULT_TLD_SUFFIX` — `certs::suffix` trimmed, lower-cased and treated
//!     an empty value as absent; ten call sites did a bare
//!     `unwrap_or("stackvo.loc")`. A `.env` carrying the key with nothing after
//!     it gave every project a domain ending in a bare dot while the
//!     certificate was still issued for `stackvo.loc`.
//!   * `DOCKER_DEFAULT_NETWORK` — the same shape, five sites.
//!   * `SUPPORTED_SERVERS_DEFAULT` — read in production in exactly one place,
//!     the new-project wizard's preselection, while five render sites said
//!     `unwrap_or("nginx")`. So `=caddy` gave a wizard project Caddy, an
//!     adopted project nginx, and a manifest with its `server` line deleted
//!     nginx. One setting, three answers.
//!
//! What this file defends is the shape of the fix: the string exists once, and
//! everything else asks for it. A literal reappearing is the second derivation
//! coming back, and the second derivation is the bug.
//!
//! ## Read as text, and only the production half
//!
//! The values are `&'static str` constants, so a test could compare them to
//! themselves and prove nothing. What has to be checked is whether the literal
//! is *written* somewhere else, which is a question about the source.
//!
//! Tests are excluded deliberately, and they are where most of these live: a
//! test naming `stackvo.loc` is stating the expected value, which is the thing
//! a gate wants written down rather than derived. Comments are excluded for the
//! reason `theme-colours.spec.js` excludes them — this repository writes the
//! bug down in the file the fix lives in, and a scanner that read the
//! explanation would fail on the sentence explaining why it must not.

use std::collections::BTreeMap;

fn src_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// A module's production half, with comments stripped.
///
/// `#[cfg(test)]` is the boundary and `//!`/`//`/`/* */` are the noise. Crude
/// on purpose: a string literal containing `//` would be mangled, and none of
/// the three values here is a URL.
fn production(text: &str) -> String {
    let body = match text.find("\n#[cfg(test)]") {
        Some(i) => &text[..i],
        None => text,
    };

    let mut out = String::with_capacity(body.len());
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Every `src/*.rs`, by file name, production only.
fn modules() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(src_dir()).expect("src/").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            out.insert(name, production(&text));
        }
    }
    out
}

/// The two values that are settings and nothing else, and their one home.
///
/// `"nginx"` is deliberately not here, and the difference is worth stating:
/// those two strings are *only ever* the value of a setting, while `"nginx"` is
/// also the **name of a web server**. It legitimately appears in `Server::parse`,
/// in the list of valid names, in the nginx-specific rendering, in the mapping
/// from an older spelling, and in the directives for directory listing. Banning
/// it outright would be a gate demanding a worse codebase, so the rule for it
/// is the shape below instead.
const OWNED: [(&str, &str); 2] = [
    ("\"stackvo.loc\"", "config.rs"),
    ("\"stackvo-net\"", "config.rs"),
];

/// The fallback idiom, which is where a default gets a second spelling.
///
/// `unwrap_or("nginx")` says "the default server is nginx" in a module that
/// cannot see `SUPPORTED_SERVERS_DEFAULT`. It was written five times, and the
/// result was one setting with three answers — a wizard project on Caddy, an
/// adopted project on nginx, and a manifest with its `server` line deleted on
/// nginx. `Env::default_server` is the answer, and `Manifest::server_or` is
/// where a project that did not choose gets it.
const FALLBACKS: [&str; 3] = [
    "unwrap_or(\"nginx\")",
    "unwrap_or_else(|| \"nginx\"",
    "unwrap_or(\"stackvo",
];

#[test]
fn each_value_is_written_in_exactly_one_module() {
    let modules = modules();
    assert!(
        modules.len() > 50,
        "only {} modules were read — the scan is not finding the tree, and \
         every assertion below would pass by looking at nothing",
        modules.len()
    );

    for pattern in FALLBACKS {
        let mut found: Vec<&str> = modules
            .iter()
            .filter(|(_, text)| text.contains(pattern))
            .map(|(name, _)| name.as_str())
            .collect();
        found.sort();

        assert!(
            found.is_empty(),
            "`{pattern}` is a default spelled where the setting cannot reach it, \
             in: {found:?}\n\
             Use `Env::default_server` / `Env::tld_suffix` / `Env::docker_network`, \
             or `Manifest::server_or` for a project that did not choose."
        );
    }

    for (literal, owner) in OWNED {
        let mut elsewhere: Vec<(&str, usize)> = modules
            .iter()
            .filter(|(name, _)| name.as_str() != owner)
            .map(|(name, text)| (name.as_str(), text.matches(literal).count()))
            .filter(|(_, n)| *n > 0)
            .collect();
        elsewhere.sort();

        assert!(
            elsewhere.is_empty(),
            "{literal} is written outside {owner}: {elsewhere:?}\n\
             It is a setting with a named constant and one derivation — \
             `config::DEFAULT_TLD_SUFFIX`, `DEFAULT_NETWORK`, `DEFAULT_SERVER` \
             and the `Env` methods beside them. A second literal is a second \
             answer, which is the defect these replaced."
        );

        let owned = modules.get(owner).expect("the owning module");
        assert_eq!(
            owned.matches(literal).count(),
            1,
            "{literal} is written {} times in {owner}; it is supposed to be the \
             one place, once",
            owned.matches(literal).count()
        );
    }
}

/// The constant and the shipped default are the same string.
///
/// They are two things — a `pub const` and a row in `EMBEDDED` — and the row
/// points at the constant, so this cannot fail today. It is here for the day
/// somebody edits the table by hand: the row is what `.env`-less workspaces
/// get, and the constant is what the fallbacks use, and a workspace where those
/// differ has one answer on disk and another in memory.
#[test]
fn the_embedded_defaults_are_the_constants() {
    let env = stackvo_desktop_lib::config::Env::parse("");

    assert_eq!(
        env.get("DEFAULT_TLD_SUFFIX"),
        Some(stackvo_desktop_lib::config::DEFAULT_TLD_SUFFIX)
    );
    assert_eq!(
        env.get("DOCKER_DEFAULT_NETWORK"),
        Some(stackvo_desktop_lib::config::DEFAULT_NETWORK)
    );
    assert_eq!(
        env.get("SUPPORTED_SERVERS_DEFAULT"),
        Some(stackvo_desktop_lib::config::DEFAULT_SERVER)
    );
}
