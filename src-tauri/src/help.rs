//! The help documents: pulled from the repository, cached, and shipped as a
//! fallback.
//!
//! Every card in the interface carries a help button naming a topic; the topic
//! resolves to `docs/help/<locale>/<topic>.md`. This finds that file.
//!
//! ## Where it looks, and why in that order
//!
//! 1. **The repository, over HTTPS.** Help text is prose about what a button
//!    does, and a correction to it should not have to wait for a release. This
//!    is the copy that is current.
//! 2. **The cache.** Whatever was pulled last, written under the app's own
//!    directory. This is what makes the panel work on a plane, behind a VPN
//!    that is refusing, or on the morning GitHub is down.
//! 3. **The bundled copy**, carried by `bundle.resources`. Present from the
//!    first launch, before anything has ever been pulled — a fresh install with
//!    no network still has its help.
//!
//! So the network decides how *current* the text is and never decides whether
//! there is text at all. A failed fetch is not an error and is not reported: it
//! silently reads what is already here.
//!
//! ## Path safety
//!
//! `topic` and `locale` arrive over IPC, so they are matched against a pattern
//! rather than pushed onto a path or into a URL: `..` in a topic would read any
//! file on the machine, or fetch any file in the repository, into a web view.
//! Only `[a-z0-9-]` is accepted, which is also exactly what the registry in
//! `src/lib/help.js` can produce.
//!
//! ## What leaves the machine
//!
//! One GET per topic per run, carrying the topic name and the locale. It is
//! written down in `PRIVACY.md`, because "which help page did you open" is a
//! thing about the person at the keyboard, and this application says elsewhere
//! that nothing leaves the machine.

use crate::error::{Code, Error, Result};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Where the current text lives. `main`, not a release tag: the point of
/// pulling is that a correction reaches people who are on last month's build.
const REMOTE_BASE: &str =
    "https://raw.githubusercontent.com/fahrettinaksoy/stackvo-tauri/main/docs/help";

/// A document is a page of prose. Anything larger is not one, and reading it
/// into a web view would be somebody else's decision about this app's memory.
const MAX_BYTES: usize = 512 * 1024;

/// Long enough for a slow connection, short enough that a panel opened while
/// offline does not sit there. The bundled copy is one timeout away.
const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Written in one of these, or read in English.
pub const LOCALES: [&str; 2] = ["en", "tr"];

pub const FALLBACK_LOCALE: &str = "en";

/// A topic is a slug and nothing else — no separators, no dots, no traversal.
fn is_slug(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 64
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The directories to look in, in order.
fn roots(app_resource_dir: Option<PathBuf>) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(dir) = app_resource_dir {
        found.push(dir.join("docs").join("help"));
    }
    found.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("help"),
    );
    found.push(Path::new("docs").join("help"));
    found
}

/// Topics already pulled in this run.
///
/// One fetch per topic per run: opening the same card's help twice is common
/// and the second open has nothing to learn. Restarting the app is what asks
/// again, which is also how somebody who was told "we fixed that page" gets it.
fn fetched() -> &'static Mutex<std::collections::HashSet<String>> {
    static FETCHED: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    FETCHED.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

/// The document as the repository has it, or `None` for any reason at all.
///
/// Every failure is the same failure here — offline, 404, a proxy that answered
/// with a login page — because the answer to all of them is the same: read what
/// is already on disk. Nothing is logged at error level for a help page.
async fn fetch(topic: &str, locale: &str) -> Option<String> {
    let url = format!("{REMOTE_BASE}/{locale}/{topic}.md");

    let client = reqwest::Client::builder()
        .user_agent(concat!("stackvo/", env!("CARGO_PKG_VERSION")))
        .timeout(FETCH_TIMEOUT)
        .build()
        .ok()?;

    let response = client.get(&url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let body = response.text().await.ok()?;
    usable(&body).then_some(body)
}

/// Whether a fetched body is a help document at all.
///
/// A document opens with its heading and is a page of prose. A 200 that is
/// neither is a captive portal's login page or a proxy's error page wearing an
/// HTTP success, and caching one would replace a good document with it — on
/// exactly the networks where the person cannot then get the real one back.
fn usable(body: &str) -> bool {
    body.len() <= MAX_BYTES && body.trim_start().starts_with("# ")
}

/// Where a pulled document is kept, so the next run has it offline.
fn cache_path(topic: &str, locale: &str) -> Option<PathBuf> {
    Some(
        crate::appdir::config()?
            .join("help-cache")
            .join(locale)
            .join(format!("{topic}.md")),
    )
}

/// The current document, pulled if it can be, read from disk if it cannot.
pub async fn current(
    app_resource_dir: Option<PathBuf>,
    topic: &str,
    locale: &str,
) -> Result<String> {
    if !is_slug(topic) {
        return Err(Error::new(
            Code::InvalidInput,
            format!("help topic must be a slug: {topic}"),
        ));
    }
    let wanted = if LOCALES.contains(&locale) {
        locale
    } else {
        FALLBACK_LOCALE
    };

    let key = format!("{wanted}/{topic}");
    let first_time = fetched()
        .lock()
        .map(|mut seen| seen.insert(key))
        .unwrap_or(false);

    if first_time {
        if let Some(text) = fetch(topic, wanted).await {
            // Cached before it is returned: the write is what the next offline
            // run reads, and a document shown but not written is one that is
            // pulled again on every single open.
            if let Some(path) = cache_path(topic, wanted) {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&path, &text);
            }
            return Ok(text);
        }
    }

    read(app_resource_dir, topic, wanted)
}

/// One document off disk: the cache first, then whatever the app shipped with.
pub fn read(app_resource_dir: Option<PathBuf>, topic: &str, locale: &str) -> Result<String> {
    if !is_slug(topic) {
        return Err(Error::new(
            Code::InvalidInput,
            format!("help topic must be a slug: {topic}"),
        ));
    }

    // An unknown locale reads English rather than failing: a reader who set a
    // language nobody has written for is better served by a page than by an
    // error about their own settings.
    let wanted = if LOCALES.contains(&locale) {
        locale
    } else {
        FALLBACK_LOCALE
    };

    if let Some(path) = cache_path(topic, wanted) {
        if let Ok(text) = std::fs::read_to_string(&path) {
            return Ok(text);
        }
    }

    for root in roots(app_resource_dir) {
        for candidate in [wanted, FALLBACK_LOCALE] {
            let path = root.join(candidate).join(format!("{topic}.md"));
            if let Ok(text) = std::fs::read_to_string(&path) {
                return Ok(text);
            }
        }
    }

    Err(Error::not_found(format!(
        "no help document for {topic} in {wanted}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_anything_that_is_not_a_slug() {
        for bad in [
            "../../../etc/passwd",
            "project/tunnel",
            "Project-Tunnel",
            "project tunnel",
            "",
        ] {
            assert!(!is_slug(bad), "{bad} was accepted as a topic");
        }
        assert!(is_slug("project-tunnel"));
        assert!(is_slug("page-project-detail"));
    }

    /// The documents this repository actually ships are readable from it.
    #[test]
    fn reads_a_written_document_from_the_repository() {
        let text = read(None, "project-container", "en").expect("the English document");
        assert!(text.starts_with("# "), "a document opens with its heading");
    }

    /// A topic written in both is read in both, and each is its own text.
    #[test]
    fn reads_the_locale_that_was_asked_for() {
        let english = read(None, "project-tunnel", "en").unwrap();
        let turkish = read(None, "project-tunnel", "tr").unwrap();
        assert_ne!(english, turkish, "one locale is serving the other's text");
    }

    /// A language nobody has written for still gets a page.
    #[test]
    fn falls_back_rather_than_failing_on_an_unwritten_locale() {
        let german = read(None, "project-tunnel", "de").unwrap();
        assert_eq!(german, read(None, "project-tunnel", "en").unwrap());
    }

    /// The check that stands between a captive portal and the cache.
    #[test]
    fn accepts_a_document_and_refuses_whatever_else_answered() {
        assert!(usable("# Container\n\nWhat Docker reports."));
        assert!(usable("  # Leading space is still a heading"));

        assert!(!usable("<html><body>Sign in to the network</body></html>"));
        assert!(!usable("Not found"));
        assert!(!usable(""));
        assert!(!usable("#no-space-is-a-fragment-not-a-heading"));
        assert!(!usable(&format!("# Huge\n\n{}", "x".repeat(MAX_BYTES))));
    }

    #[test]
    fn names_the_topic_when_there_is_no_document() {
        let err = read(None, "project-nothing-here", "en").unwrap_err();
        assert!(format!("{err:?}").contains("project-nothing-here"));
    }
}
