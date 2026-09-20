//! The help documents: pulled from the repository and cached, never bundled.
//!
//! Every card in the interface carries a help button naming a topic; the topic
//! resolves to `docs/help/<locale>/<topic>.md` in the repository. This fetches
//! that file.
//!
//! ## Where it looks, and why in that order
//!
//! 1. **The repository, over HTTPS.** Help text is prose about what a button
//!    does, and a correction to it should not have to wait for a release. This
//!    is the copy that is current, and it is the only copy there is.
//! 2. **The cache.** Whatever was pulled last, written under the app's own
//!    directory. This is what makes the panel work on a plane, behind a VPN
//!    that is refusing, or on the morning GitHub is down.
//!
//! There is no third copy. The documents used to ship inside the installer as
//! well, so that a fresh install with no network still had its help; that copy
//! was as old as the build, it was the one thing the installer carried that the
//! repository could correct, and it made an offline panel indistinguishable
//! from a working one. Now a document that has never been fetched and cannot be
//! fetched is reported as exactly that — `NETWORK_ERROR`, with a hint saying
//! the machine is offline — rather than answered with stale text.
//!
//! So the network decides whether a document is *current*; the cache decides
//! whether it is *available* offline. A fetch that fails for a document the
//! cache holds is silent, and the cached text is shown.
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
///
/// The repository is this one, and "this one" moved: the constant still named
/// `fahrettinaksoy/stackvo-tauri` after the remote became `stackvo/stackvo`, so
/// every fetch answered 404. Nothing showed it, because a failed fetch here was
/// deliberately silent — the panel fell back to the copy the app shipped with,
/// which was right for a slow connection and indistinguishable from a URL that
/// can never work. `published_urls.rs` derives the slug from `.git/config` and
/// fails the build on a third spelling of it; and a 404 is now `NOT_FOUND`
/// naming the topic, which is visible.
const REMOTE_BASE: &str = "https://raw.githubusercontent.com/stackvo/stackvo/main/docs/help";

/// A document is a page of prose. Anything larger is not one, and reading it
/// into a web view would be somebody else's decision about this app's memory.
const MAX_BYTES: usize = 512 * 1024;

/// Long enough for a slow connection, short enough that a panel opened while
/// offline does not sit there. The cache — or the offline notice — is one
/// timeout away.
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

/// The locale a document is read in: the one asked for if it is written in,
/// English otherwise.
///
/// A reader who set a language nobody has written for is better served by a
/// page than by an error about their own settings.
fn locale_for(locale: &str) -> &str {
    if LOCALES.contains(&locale) {
        locale
    } else {
        FALLBACK_LOCALE
    }
}

/// Topics pulled successfully in this run.
///
/// One fetch per topic per run: opening the same card's help twice is common
/// and the second open has nothing to learn. Restarting the app is what asks
/// again, which is also how somebody who was told "we fixed that page" gets it.
///
/// Only a *successful* pull is recorded. A topic opened offline is asked for
/// again on its next open, so that plugging the cable back in is enough — the
/// alternative was a panel that stayed "offline" until the app was restarted.
fn fetched() -> &'static Mutex<std::collections::HashSet<String>> {
    static FETCHED: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    FETCHED.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

/// What asking the repository for a document came back with.
enum Fetched {
    /// The document, checked to be one.
    Document(String),
    /// The repository answered, and it has no such file: an unwritten topic.
    Missing,
    /// No usable answer — offline, a timeout, a proxy that answered with a
    /// login page, a server error. The answer to all of these is the same:
    /// read what is already on disk, and say so if there is nothing.
    Unreachable,
}

/// The document as the repository has it.
///
/// A 404 is kept apart from every other failure, because it is the one that
/// means something different: the network is fine and the document does not
/// exist, which is a fact about this repository rather than about this
/// machine. Nothing is logged at error level for a help page.
async fn fetch(topic: &str, locale: &str) -> Fetched {
    let url = format!("{REMOTE_BASE}/{locale}/{topic}.md");

    let Ok(client) = reqwest::Client::builder()
        .user_agent(concat!("stackvo/", env!("CARGO_PKG_VERSION")))
        .timeout(FETCH_TIMEOUT)
        .build()
    else {
        return Fetched::Unreachable;
    };

    let Ok(response) = client.get(&url).send().await else {
        return Fetched::Unreachable;
    };
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Fetched::Missing;
    }
    if !response.status().is_success() {
        return Fetched::Unreachable;
    }

    match response.text().await {
        Ok(body) if usable(&body) => Fetched::Document(body),
        _ => Fetched::Unreachable,
    }
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

/// Where pulled documents are kept, so the next run has them offline.
fn cache_root() -> Option<PathBuf> {
    Some(crate::appdir::config()?.join("help-cache"))
}

fn cache_path(root: &Path, topic: &str, locale: &str) -> PathBuf {
    root.join(locale).join(format!("{topic}.md"))
}

/// The current document: pulled if it can be, read from the cache if it
/// cannot, and an error naming why if it is in neither place.
pub async fn current(topic: &str, locale: &str) -> Result<String> {
    if !is_slug(topic) {
        return Err(Error::new(
            Code::InvalidInput,
            format!("help topic must be a slug: {topic}"),
        ));
    }
    let wanted = locale_for(locale);
    let root = cache_root();

    // Already pulled this run: the cache is the current text, and there is
    // nothing to ask the network. If the cache went missing underneath — a
    // cleaner emptied the directory — fall through and pull again.
    let key = format!("{wanted}/{topic}");
    let pulled_this_run = fetched()
        .lock()
        .map(|seen| seen.contains(&key))
        .unwrap_or(false);
    if pulled_this_run {
        if let Some(root) = &root {
            if let Ok(text) = read_cached(root, topic, wanted) {
                return Ok(text);
            }
        }
    }

    match fetch(topic, wanted).await {
        Fetched::Document(text) => {
            // Cached before it is returned: the write is what the next offline
            // run reads, and a document shown but not written is one that is
            // pulled again on every single open.
            if let Some(root) = &root {
                let path = cache_path(root, topic, wanted);
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&path, &text);
            }
            if let Ok(mut seen) = fetched().lock() {
                seen.insert(key);
            }
            Ok(text)
        }
        Fetched::Missing => Err(Error::not_found(format!(
            "help document for {topic} in {wanted}"
        ))),
        Fetched::Unreachable => match root {
            Some(root) => read_cached(&root, topic, wanted).map_err(|_| offline(topic)),
            None => Err(offline(topic)),
        },
    }
}

/// The document could not be fetched and has never been fetched on this
/// machine. Said as what it is, so the panel can say "you are offline" rather
/// than "this has not been written".
fn offline(topic: &str) -> Error {
    Error::new(
        Code::NetworkError,
        format!("help document for {topic} could not be fetched, and it is not cached"),
    )
    .with_hint(crate::hints::HELP_NEEDS_NETWORK)
}

/// One document off the cache, in the locale asked for or in English.
///
/// `root` is a parameter rather than [`cache_root`] so that this is testable
/// against a directory a test wrote, without an app directory or an
/// environment variable in the way.
pub fn read_cached(root: &Path, topic: &str, locale: &str) -> Result<String> {
    if !is_slug(topic) {
        return Err(Error::new(
            Code::InvalidInput,
            format!("help topic must be a slug: {topic}"),
        ));
    }
    let wanted = locale_for(locale);

    for candidate in [wanted, FALLBACK_LOCALE] {
        if let Ok(text) = std::fs::read_to_string(cache_path(root, topic, candidate)) {
            return Ok(text);
        }
    }

    Err(Error::not_found(format!(
        "cached help document for {topic} in {wanted}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cache directory of this test's own, holding the documents it names.
    fn cache_with(name: &str, documents: &[(&str, &str, &str)]) -> PathBuf {
        let root = std::env::temp_dir()
            .join("stackvo-help-tests")
            .join(format!("{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (locale, topic, text) in documents {
            let path = cache_path(&root, topic, locale);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        root
    }

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

    /// A slug is refused before it touches the filesystem, whatever the cache
    /// holds.
    #[test]
    fn refuses_a_traversal_even_against_the_cache() {
        let root = cache_with("traversal", &[("en", "project-tunnel", "# Tunnel")]);
        let err = read_cached(&root, "../project-tunnel", "en").unwrap_err();
        assert!(format!("{err:?}").contains("slug"));
    }

    /// A topic written in both is read in both, and each is its own text.
    #[test]
    fn reads_the_locale_that_was_asked_for() {
        let root = cache_with(
            "locales",
            &[
                ("en", "project-tunnel", "# Tunnel\n\nEnglish."),
                ("tr", "project-tunnel", "# Tünel\n\nTürkçe."),
            ],
        );
        let english = read_cached(&root, "project-tunnel", "en").unwrap();
        let turkish = read_cached(&root, "project-tunnel", "tr").unwrap();
        assert_ne!(english, turkish, "one locale is serving the other's text");
        assert!(turkish.starts_with("# Tünel"));
    }

    /// A language nobody has written for still gets a page.
    #[test]
    fn falls_back_rather_than_failing_on_an_unwritten_locale() {
        let root = cache_with("fallback", &[("en", "project-tunnel", "# Tunnel")]);
        let german = read_cached(&root, "project-tunnel", "de").unwrap();
        assert_eq!(german, read_cached(&root, "project-tunnel", "en").unwrap());
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
    fn names_the_topic_when_nothing_is_cached() {
        let root = cache_with("empty", &[]);
        let err = read_cached(&root, "project-nothing-here", "en").unwrap_err();
        assert!(format!("{err:?}").contains("project-nothing-here"));
    }

    /// What the panel reads to say "you are offline" rather than "unwritten":
    /// the code, and a hint that has a translation.
    #[test]
    fn offline_is_a_network_error_with_a_translated_hint() {
        let err = offline("project-tunnel");
        assert_eq!(err.code, Code::NetworkError);
        assert_eq!(err.hint_key, Some(crate::hints::HELP_NEEDS_NETWORK.key));
        assert!(err.message.contains("project-tunnel"));
    }
}
