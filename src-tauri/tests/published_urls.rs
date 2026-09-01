//! Every URL this app hard-codes that names *this* repository names the right
//! one.
//!
//! `updater_endpoint.rs` beside this file checks one such URL and explains at
//! length why it is derived from `.git/config` rather than typed: a constant is
//! a second copy of a fact, and the copy is the one that goes stale. That
//! reasoning was right and it was applied to exactly one constant.
//!
//! There were two. `help.rs` fetches its documents from
//! `raw.githubusercontent.com/<owner>/<repo>/main/docs/help`, and after the
//! remote moved to `stackvo/stackvo` that constant still said
//! `fahrettinaksoy/stackvo-tauri`. Every help fetch answered 404.
//!
//! **Nothing showed it, and nothing could.** A failed help fetch is silent by
//! design — the panel falls back to the copy the app shipped with, which is the
//! correct behaviour on a slow connection and is indistinguishable from a URL
//! that can never work. So the feature looked perfect on the machine it was
//! written on, where the bundled documents are also the current ones, and would
//! have kept looking perfect on every machine while the whole point of pulling
//! — that a correction reaches somebody on last month's build — silently did
//! not happen.
//!
//! One URL was guarded and one was not, so the rule is now the class rather
//! than the instance: scan the crate, find every GitHub repository it names,
//! and require each to be this repository or a declared exception.
//!
//! ## What is deliberately not checked
//!
//! That any of them answer. No network — for the reason `updater_endpoint.rs`
//! gives: a gate that fails when GitHub is slow is a gate people learn to
//! ignore. What is checkable offline is which repository the URL names, and
//! that is the half that was wrong both times.

use std::collections::BTreeSet;
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

/// Repositories other than this one that the app names on purpose.
///
/// Three entries, and each earns it. An allowlist rather than a pattern, so
/// adding a fourth is a decision somebody writes down — which is what happened
/// with the second and third: `spx.rs` was written, this test failed, and the
/// reason had to be typed out before it passed. `tooling.rs` went the same way.
const OTHER_REPOSITORIES: [(&str, &str); 4] = [
    (
        "roadrunner-server/roadrunner",
        "RoadRunner's own install script, curled inside the Dockerfile `generator.rs` writes \
         for a Laravel Octane project — see `roadrunner_postamble`. Unlike mkcert below this \
         one is NOT pinned: it is fetched from `master` and resolves the right build for the \
         image's architecture, which a hardcoded release asset does not. That is a real \
         difference in kind and it is recorded here rather than smoothed over — the address \
         moving would change what is installed, not refuse to install. It is bounded by \
         where it runs: a generated image, built from a manifest that asked for RoadRunner",
    ),
    (
        "stackvo/stackvo-service-packages",
        "the service catalogue, released separately from the app — market.rs suggests it as \
         the default source",
    ),
    (
        "NoiseByNorthwest/php-spx",
        "the sampling profiler's own source. It is not on PECL, so spx.rs clones and builds \
         it — see that module for why the extension cannot come through the manifest",
    ),
    (
        "FiloSottile/mkcert",
        "mkcert's own release, which tooling.rs fetches on request. The URL is pinned to one \
         version and every asset's SHA-256 is compiled in beside it, so this address moving \
         is a build that refuses to install rather than one that installs something else",
    ),
];

/// `owner/repo` from git's own config.
///
/// The same reader `updater_endpoint.rs` uses, and duplicated rather than
/// shared because integration tests are separate binaries with no common
/// module — the alternative is a `tests/common/` that exists to hold twelve
/// lines. `None` in a checkout with no origin, which is a source tarball or a
/// CI clone done some other way, and is reported rather than failed on.
fn origin_slug() -> Option<String> {
    let config = std::fs::read_to_string(repo_root().join(".git/config")).ok()?;
    let url = config
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("url = "))?;
    let path = url
        .trim_end_matches(".git")
        .rsplit_once("github.com")
        .map(|(_, rest)| rest.trim_start_matches([':', '/']))?
        .to_string();
    (path.matches('/').count() == 1).then_some(path)
}

/// `src` without its `#[cfg(test)]` regions.
///
/// A fixture URL is not a claim about anything: `market.rs`'s tests parse
/// `https://github.com/o/r` to prove the parser handles a slug, and failing on
/// it would teach people to write the placeholder somewhere the scanner cannot
/// see instead of somewhere it can.
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

/// Every `owner/repo` a GitHub URL in `text` names, with the line it is on.
///
/// Textual, and it does not need to be more: these are string literals in
/// source and JSON, and a URL assembled from parts would not be a hard-coded
/// URL — which is the thing being checked.
fn slugs(text: &str) -> Vec<(usize, String)> {
    const HOSTS: [&str; 2] = ["https://github.com/", "https://raw.githubusercontent.com/"];
    let mut found = Vec::new();

    for (number, line) in text.lines().enumerate() {
        for host in HOSTS {
            let mut rest = line;
            while let Some(at) = rest.find(host) {
                let tail = &rest[at + host.len()..];
                let slug: String = tail
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || "._-/".contains(*c))
                    .collect();

                let mut parts = slug.split('/');
                if let (Some(owner), Some(repo)) = (parts.next(), parts.next()) {
                    if !owner.is_empty() && !repo.is_empty() {
                        found.push((
                            number + 1,
                            format!("{owner}/{}", repo.trim_end_matches(".git")),
                        ));
                    }
                }
                rest = &rest[at + host.len()..];
            }
        }
    }

    found
}

/// Every file that can carry one: the crate's own sources and its Tauri
/// configuration.
fn sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("src/ is readable")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();

    let mut out: Vec<(String, String)> = files
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("a source file is readable");
            let name = format!(
                "src-tauri/src/{}",
                path.file_name().and_then(|s| s.to_str()).unwrap_or("?")
            );
            (name, production_regions(&text))
        })
        .collect();

    out.push((
        "src-tauri/tauri.conf.json".to_string(),
        read("src-tauri/tauri.conf.json"),
    ));
    out
}

#[test]
fn every_github_url_names_this_repository_or_a_declared_one() {
    let Some(slug) = origin_slug() else {
        eprintln!("no github origin in .git/config — nothing to check against");
        return;
    };

    let allowed: BTreeSet<&str> = OTHER_REPOSITORIES.iter().map(|(name, _)| *name).collect();
    let mut wrong: Vec<String> = Vec::new();

    for (file, text) in sources() {
        for (line, found) in slugs(&text) {
            if found == slug || allowed.contains(found.as_str()) {
                continue;
            }
            wrong.push(format!("  {file}:{line} names {found}"));
        }
    }

    assert!(
        wrong.is_empty(),
        "this checkout came from `{slug}`, and {} hard-coded URL(s) name a \
         different repository:\n{}\n\nEither this repository moved and the \
         constant did not — which is silent, because a 404 from an update \
         check or a help fetch is never shown — or the URL genuinely points \
         somewhere else and belongs in OTHER_REPOSITORIES with the reason.",
        wrong.len(),
        wrong.join("\n")
    );
}

/// The scanner has to be finding things, or the test above passes on a typo in
/// the scanner rather than on a clean tree.
#[test]
fn the_scan_finds_the_urls_that_are_known_to_be_there() {
    let all: Vec<String> = sources()
        .iter()
        .flat_map(|(_, text)| slugs(text))
        .map(|(_, slug)| slug)
        .collect();
    let found: BTreeSet<&str> = all.iter().map(String::as_str).collect();

    // Occurrences, not distinct repositories. Once both stale constants are
    // corrected there are only two repositories in the whole crate, so a
    // threshold on the distinct count would have to be `2` — which a scanner
    // that had stopped matching everything except the catalogue would also
    // reach. The call sites are what has to still be seen: the updater
    // endpoint, the help base, the About link and the catalogue.
    assert!(
        all.len() >= 4,
        "only {} repository URL(s) found across the crate — the updater \
         endpoint, the help base, the About link and the package catalogue are \
         all still string literals, so a smaller number is the scanner failing: \
         {all:?}",
        all.len()
    );
    assert!(
        found.contains("stackvo/stackvo-service-packages"),
        "the catalogue URL was not found: {found:?}"
    );
}

/// And the exception list is not a place things go to be forgotten.
#[test]
fn every_declared_exception_is_still_named_somewhere() {
    let found: BTreeSet<String> = sources()
        .iter()
        .flat_map(|(_, text)| slugs(text))
        .map(|(_, slug)| slug)
        .collect();

    for (name, reason) in OTHER_REPOSITORIES {
        assert!(
            found.contains(name),
            "OTHER_REPOSITORIES holds `{name}` ({reason}), which nothing names \
             any more. Remove it — an allowlist entry with no call site is a \
             permission nobody asked for."
        );
    }
}

/// The help documents are fetched from a path that exists in this repository.
///
/// The slug is only half of the URL. `main/docs/help` is the other half, and a
/// documents directory moved without the constant would be the same silent 404
/// with a correct owner on it.
#[test]
fn the_help_base_points_at_the_directory_the_documents_are_in() {
    let source = read("src-tauri/src/help.rs");
    let base = source
        .lines()
        .find(|line| line.contains("raw.githubusercontent.com"))
        .expect("help.rs declares a remote base");

    assert!(
        base.contains("/main/docs/help"),
        "the help base does not read from docs/help on the default branch: {base}"
    );
    assert!(
        repo_root().join("docs/help").is_dir(),
        "docs/help is not in this repository, so the URL above cannot answer"
    );

    // The branch, not a tag: the whole point of pulling is that a correction
    // reaches somebody who is on last month's build, and a tag would pin them
    // to the documents that shipped with it.
    assert!(
        !base.contains("/releases/") && !base.contains("/tags/"),
        "the help base is pinned to a release, which defeats fetching it: {base}"
    );
}
