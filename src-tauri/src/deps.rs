//! The same discipline `pkg.rs` applies to packages, applied to the project's.
//!
//! ## The asymmetry this closes
//!
//! [`crate::pkg`] verifies every file of every service package against a
//! manifest digest before it runs, refuses a moving tag, and checks a signature
//! over the index it came from. Meanwhile the project sitting beside it pulls
//! four hundred libraries out of `composer.lock` and `package-lock.json` and
//! nothing in this application has ever looked at them.
//!
//! That is the wrong way round. The service packages are a catalogue this
//! project publishes and can vouch for; the dependencies are somebody else's
//! code, in far greater quantity, running with the developer's own permissions.
//!
//! ## Two halves, and only one of them touches a network
//!
//! **What the lock file already says** is read with no network at all, and it
//! answers three questions that turn out to matter:
//!
//! | Finding | Why it is one |
//! | --- | --- |
//! | A dependency fetched over `http://` | [`crate::market`] refuses plain HTTP for this app's *own* catalogue. A project doing it for four hundred libraries is the same hole, larger |
//! | A dependency with no integrity hash | Nothing verifies those bytes. `pkg::verify` exists because that is not acceptable for one package; it is not more acceptable for two hundred |
//! | Where each one comes from | A private registry or a git URL in a lock file is a supply chain nobody wrote down. Counted by host, so the ordinary answer is one line |
//!
//! **Whether any of them has a published advisory** is the half that needs a
//! network, and it is separate for that reason: a different button, a named
//! host, and a sentence in `PRIVACY.md` saying exactly what leaves.
//!
//! ## What is sent, said plainly
//!
//! The advisory query sends **the names and versions of the project's
//! dependencies** to `api.osv.dev`. That is a real disclosure and it is written
//! here, in `PRIVACY.md` and on the button, rather than described as "checking
//! for updates". `privacy_claims.rs` holds the document to it.
//!
//! No identifier is attached, nothing is stored on the far side that this can
//! see, and nothing happens until somebody presses the button — the local half
//! works with no network for ever.
//!
//! ## The two lock files, and the ones deliberately not read
//!
//! `composer.lock` and `package-lock.json`: the two this application's own
//! runtimes reach for most, and the two that are JSON. A `yarn.lock`, a
//! `pnpm-lock.yaml`, a `go.sum` and a `Cargo.lock` are four more formats and
//! four more parsers, and a parser written from memory against a format nobody
//! measured is how a report starts quietly missing half a project. They are
//! named as absent rather than guessed at — the same call `images.rs` made
//! about picking version pins.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Which index a package comes from, in the spelling OSV uses.
///
/// OSV's own names, exactly — `Packagist` capitalised and `npm` not — because
/// this string goes into a query and a helpful normalisation here would be a
/// query that silently matches nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Ecosystem {
    #[serde(rename = "Packagist")]
    Packagist,
    #[serde(rename = "npm")]
    Npm,
}

impl Ecosystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Ecosystem::Packagist => "Packagist",
            Ecosystem::Npm => "npm",
        }
    }

    /// The default index for this ecosystem, so a report can say "everything
    /// came from the usual place" in one line.
    pub fn default_host(self) -> &'static str {
        match self {
            Ecosystem::Packagist => "repo.packagist.org",
            Ecosystem::Npm => "registry.npmjs.org",
        }
    }
}

/// One dependency, as the lock file describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dep {
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    /// Whether the project's own manifest asks for it, as opposed to something
    /// else asking for it on the project's behalf.
    ///
    /// The distinction the roadmap's example sentence turns on — *"three
    /// advisories, two of them in a direct dependency"* — because the two have
    /// different repairs: a direct one is a version you choose, a transitive
    /// one is a version somebody else chooses for you.
    pub direct: bool,
    /// Where the bytes were fetched from, when the lock says.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Whether the lock carries a hash over those bytes.
    pub hashed: bool,
}

impl Dep {
    /// `npm:lodash@4.17.21` — one string per dependency, so advisories can be
    /// matched back to rows without carrying the pair everywhere.
    pub fn key(&self) -> String {
        format!("{}:{}@{}", self.ecosystem.as_str(), self.name, self.version)
    }

    /// The host its source names, when it names one.
    pub fn host(&self) -> Option<&str> {
        let rest = self
            .source
            .as_deref()?
            .split_once("://")
            .map(|(_, rest)| rest)?;
        Some(rest.split(['/', '?', '#']).next().unwrap_or(rest))
    }
}

/// One thing worth saying about the set, with no network involved.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// Stable key; the UI holds the sentence — the `preflight` arrangement.
    pub id: &'static str,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A published advisory against one dependency.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advisory {
    /// `npm:lodash@4.17.21`, matching [`Dep::key`].
    pub package: String,
    /// OSV's own ids — `GHSA-…`, `CVE-…`. Reported rather than summarised: an
    /// id is what somebody searches for, and a severity word this build derived
    /// itself would be a judgement it is in no position to make.
    pub ids: Vec<String>,
    /// Whether the project's own manifest asks for this package.
    pub direct: bool,
}

/// What this project depends on.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// The lock files that were read, by name.
    ///
    /// Named rather than counted so "no findings" can be told apart from "no
    /// lock file was found" — which is the difference between a clean project
    /// and one this cannot see.
    pub locks: Vec<String>,
    pub total: usize,
    pub direct: usize,
    /// Every host the lock files name, with how many packages came from each.
    pub hosts: BTreeMap<String, usize>,
    pub findings: Vec<Finding>,
    /// `None` until somebody asks. Distinct from an empty list, which means the
    /// query ran and found nothing — the two must never look the same.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisories: Option<Vec<Advisory>>,
}

// ------------------------------------------------------------------ parsing

/// The names a `composer.json` asks for directly.
///
/// `composer.lock` does not record which packages the project chose and which
/// were pulled in for it — that fact only exists in `composer.json`. Read from
/// there rather than inferred, because an inference would be wrong for exactly
/// the packages that appear in both places.
pub fn direct_from_composer_json(text: &str) -> BTreeSet<String> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return BTreeSet::new();
    };
    ["require", "require-dev"]
        .iter()
        .filter_map(|key| json.get(*key).and_then(|v| v.as_object()))
        .flat_map(|map| map.keys().cloned())
        // `php`, `ext-mbstring` and `composer-plugin-api` are requirements on
        // the platform, not packages, and no index has them.
        .filter(|name| name.contains('/'))
        .collect()
}

/// The names a `package.json` asks for directly.
pub fn direct_from_package_json(text: &str) -> BTreeSet<String> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return BTreeSet::new();
    };
    ["dependencies", "devDependencies", "optionalDependencies"]
        .iter()
        .filter_map(|key| json.get(*key).and_then(|v| v.as_object()))
        .flat_map(|map| map.keys().cloned())
        .collect()
}

/// Every package in a `composer.lock`, production and dev alike.
///
/// Dev packages are included and not marked apart. They are installed on the
/// machine this report is about, they run in the same container, and a
/// vulnerable test runner is a vulnerable program — "it is only a dev
/// dependency" is a sentence that has introduced real incidents.
pub fn parse_composer_lock(text: &str, direct: &BTreeSet<String>) -> Vec<Dep> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for key in ["packages", "packages-dev"] {
        for entry in json.get(key).and_then(|v| v.as_array()).unwrap_or(&vec![]) {
            let Some(name) = entry.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(version) = entry.get("version").and_then(|v| v.as_str()) else {
                continue;
            };

            // `dist` is what composer actually downloads; `source` is the
            // fallback it uses when told to, and is what a git-sourced package
            // has instead. Asked in that order because it is the order composer
            // asks in.
            let dist = entry.get("dist");
            let source = dist
                .and_then(|d| d.get("url"))
                .or_else(|| entry.get("source").and_then(|s| s.get("url")))
                .and_then(|v| v.as_str())
                .map(str::to_string);

            out.push(Dep {
                ecosystem: Ecosystem::Packagist,
                direct: direct.contains(name),
                name: name.to_string(),
                // A leading `v` is composer's tag spelling, not the version.
                // OSV wants the version, and `v1.2.3` matches nothing.
                version: version.strip_prefix('v').unwrap_or(version).to_string(),
                hashed: dist
                    .and_then(|d| d.get("shasum"))
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| !s.is_empty()),
                source,
            });
        }
    }
    out
}

/// Every package in a `package-lock.json`.
///
/// Lockfile v2 and v3 both carry the flat `packages` map, keyed by install
/// path, and v2 carries the old nested `dependencies` tree beside it for
/// compatibility. The flat map is read when it is there and the tree only when
/// it is not, so a v2 lock is never counted twice.
pub fn parse_package_lock(text: &str, direct: &BTreeSet<String>) -> Vec<Dep> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    if let Some(packages) = json.get("packages").and_then(|v| v.as_object()) {
        for (path, entry) in packages {
            // The empty key is the project itself, and a `link` entry is a
            // workspace pointing at a sibling directory — neither is a
            // dependency anybody fetched.
            if path.is_empty() || entry.get("link").and_then(|v| v.as_bool()) == Some(true) {
                continue;
            }
            // `node_modules/a/node_modules/b` is package `b`. Everything after
            // the last marker, so nesting depth does not change the name.
            let Some((_, name)) = path.rsplit_once("node_modules/") else {
                continue;
            };
            let Some(version) = entry.get("version").and_then(|v| v.as_str()) else {
                continue;
            };
            out.push(Dep {
                ecosystem: Ecosystem::Npm,
                direct: direct.contains(name),
                name: name.to_string(),
                version: version.to_string(),
                source: entry
                    .get("resolved")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                hashed: entry.get("integrity").and_then(|v| v.as_str()).is_some(),
            });
        }
        return out;
    }

    // Lockfile v1: a nested tree, walked.
    fn walk(node: &serde_json::Value, direct: &BTreeSet<String>, out: &mut Vec<Dep>) {
        let Some(map) = node.get("dependencies").and_then(|v| v.as_object()) else {
            return;
        };
        for (name, entry) in map {
            if let Some(version) = entry.get("version").and_then(|v| v.as_str()) {
                out.push(Dep {
                    ecosystem: Ecosystem::Npm,
                    direct: direct.contains(name),
                    name: name.clone(),
                    version: version.to_string(),
                    source: entry
                        .get("resolved")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    hashed: entry.get("integrity").and_then(|v| v.as_str()).is_some(),
                });
            }
            walk(entry, direct, out);
        }
    }
    walk(&json, direct, &mut out);
    out
}

// ----------------------------------------------------------------- findings

/// What is worth saying about a set of dependencies with no network.
pub fn findings(deps: &[Dep]) -> Vec<Finding> {
    let mut out = Vec::new();

    // Plain HTTP, first and one row per package. `market.rs` refuses `http://`
    // for this application's own catalogue with the reasoning written out; a
    // project fetching four hundred libraries that way is the same hole at
    // scale, and whoever is on the path chooses what arrives.
    for dep in deps.iter().filter(|d| {
        d.source
            .as_deref()
            .is_some_and(|s| s.starts_with("http://"))
    }) {
        out.push(Finding {
            id: "insecureSource",
            subject: format!("{}@{}", dep.name, dep.version),
            detail: dep.source.clone(),
        });
    }

    // Nothing verifies these bytes. Counted rather than listed: on a lock file
    // written by an older tool this can be every package, and four hundred rows
    // saying the same thing is a screen nobody reads.
    let unhashed = deps.iter().filter(|d| !d.hashed).count();
    if unhashed > 0 {
        out.push(Finding {
            id: "noIntegrity",
            subject: unhashed.to_string(),
            detail: None,
        });
    }

    // A package that came from somewhere other than the ecosystem's own index.
    // Not a fault — a private mirror is a perfectly ordinary thing to have —
    // but it is a supply chain, and a supply chain nobody has written down is
    // one nobody is watching.
    let mut unusual: BTreeMap<&str, usize> = BTreeMap::new();
    for dep in deps {
        if let Some(host) = dep.host() {
            if host != dep.ecosystem.default_host() {
                *unusual.entry(host).or_default() += 1;
            }
        }
    }
    for (host, count) in unusual {
        out.push(Finding {
            id: "otherIndex",
            subject: host.to_string(),
            detail: Some(count.to_string()),
        });
    }

    out
}

/// Assemble the local half.
pub fn report(locks: Vec<String>, deps: &[Dep]) -> Report {
    let mut hosts: BTreeMap<String, usize> = BTreeMap::new();
    for dep in deps {
        *hosts
            .entry(
                dep.host()
                    .unwrap_or(dep.ecosystem.default_host())
                    .to_string(),
            )
            .or_default() += 1;
    }

    Report {
        locks,
        total: deps.len(),
        direct: deps.iter().filter(|d| d.direct).count(),
        hosts,
        findings: findings(deps),
        advisories: None,
    }
}

// ---------------------------------------------------------------- the query

/// The one host this module can reach, and it is named here so that
/// `PRIVACY.md` and the code cannot drift apart without the gate noticing.
pub const OSV_BATCH: &str = "https://api.osv.dev/v1/querybatch";

/// OSV's documented ceiling for one batch. Chunked rather than truncated: a
/// report that silently stopped at a thousand packages would be a clean result
/// for a large project, which is the worst answer available.
const BATCH: usize = 1000;

/// Ask whether any of these has a published advisory.
///
/// Sends the **names and versions** of the dependencies and nothing else — no
/// identifier, no project name, no path. That disclosure is real, is why this
/// is a separate button, and is written in `PRIVACY.md` in those words.
///
/// A failure is an error rather than an empty list. "Nothing was found" and "I
/// could not ask" must never look the same on a security screen.
pub async fn advisories(deps: &[Dep]) -> crate::error::Result<Vec<Advisory>> {
    use crate::error::{Code, Error};

    let client = reqwest::Client::builder()
        .user_agent(concat!("stackvo/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::new(Code::NetworkError, format!("building the client: {e}")))?;

    let mut out = Vec::new();
    for chunk in deps.chunks(BATCH) {
        let body = serde_json::json!({
            "queries": chunk
                .iter()
                .map(|d| serde_json::json!({
                    "package": { "name": d.name, "ecosystem": d.ecosystem.as_str() },
                    "version": d.version,
                }))
                .collect::<Vec<_>>(),
        });

        let response = client
            .post(OSV_BATCH)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::new(Code::NetworkError, format!("asking api.osv.dev: {e}")))?;

        if !response.status().is_success() {
            return Err(Error::new(
                Code::NetworkError,
                format!("api.osv.dev answered {}", response.status()),
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::new(Code::NetworkError, format!("reading the answer: {e}")))?;

        // Positional: OSV answers one result per query, in order. Zipped rather
        // than looked up by name, because that is the only correspondence the
        // API defines — and a length mismatch is a protocol change rather than
        // an empty answer, so it is refused.
        let results = json.get("results").and_then(|v| v.as_array());
        let Some(results) = results.filter(|r| r.len() == chunk.len()) else {
            return Err(Error::new(
                Code::NetworkError,
                "api.osv.dev answered with a different number of results than queries".to_string(),
            ));
        };

        for (dep, result) in chunk.iter().zip(results) {
            let ids: Vec<String> = result
                .get("vulns")
                .and_then(|v| v.as_array())
                .map(|vulns| {
                    vulns
                        .iter()
                        .filter_map(|v| v.get("id").and_then(|i| i.as_str()))
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();

            if !ids.is_empty() {
                out.push(Advisory {
                    package: dep.key(),
                    direct: dep.direct,
                    ids,
                });
            }
        }
    }

    // Direct first: the two have different repairs, and the one somebody can
    // act on today belongs at the top.
    out.sort_by(|a, b| b.direct.cmp(&a.direct).then(a.package.cmp(&b.package)));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    /// Direct and transitive, which is the distinction the whole report turns
    /// on — a direct dependency is a version you choose, a transitive one is a
    /// version somebody else chooses for you.
    ///
    /// `composer.lock` does not record it, so it comes from `composer.json`,
    /// and the platform requirements there (`php`, `ext-mbstring`) are dropped
    /// because no index has them.
    #[test]
    fn a_composer_lock_says_what_is_installed_and_the_manifest_says_what_was_asked_for() {
        let direct = direct_from_composer_json(
            r#"{"require": {"php": "^8.2", "ext-mbstring": "*", "monolog/monolog": "^3.0"},
                "require-dev": {"phpunit/phpunit": "^11.0"}}"#,
        );
        assert_eq!(direct, set(&["monolog/monolog", "phpunit/phpunit"]));

        let deps = parse_composer_lock(
            r#"{
              "packages": [
                {"name": "monolog/monolog", "version": "v3.5.0",
                 "dist": {"url": "https://api.github.com/repos/Seldaek/monolog/zipball/abc",
                          "shasum": "deadbeef"}},
                {"name": "psr/log", "version": "3.0.0",
                 "dist": {"url": "https://repo.packagist.org/p/psr/log", "shasum": "cafe"}}
              ],
              "packages-dev": [
                {"name": "phpunit/phpunit", "version": "11.0.1",
                 "dist": {"url": "https://repo.packagist.org/p/phpunit", "shasum": ""}}
              ]
            }"#,
            &direct,
        );

        assert_eq!(
            deps.len(),
            3,
            "dev packages count: they run on this machine"
        );
        assert!(deps[0].direct);
        assert!(!deps[1].direct, "psr/log arrived because monolog asked");
        assert!(deps[2].direct);

        // A leading `v` is composer's tag spelling, not the version — and a
        // query for `v3.5.0` matches nothing.
        assert_eq!(deps[0].version, "3.5.0");
        assert_eq!(deps[0].key(), "Packagist:monolog/monolog@3.5.0");

        // An empty shasum is not a hash.
        assert!(deps[0].hashed);
        assert!(!deps[2].hashed);
    }

    /// Lockfile v2 carries both shapes, and reading both would count every
    /// package twice.
    #[test]
    fn a_package_lock_is_read_once_whichever_version_wrote_it() {
        let direct = direct_from_package_json(r#"{"dependencies": {"lodash": "^4.17.21"}}"#);

        let modern = parse_package_lock(
            r#"{
              "lockfileVersion": 3,
              "packages": {
                "": {"name": "shop"},
                "node_modules/lodash": {"version": "4.17.21",
                  "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
                  "integrity": "sha512-x"},
                "node_modules/a/node_modules/tiny": {"version": "1.0.0",
                  "resolved": "https://registry.npmjs.org/tiny/-/tiny-1.0.0.tgz",
                  "integrity": "sha512-y"},
                "packages/ui": {"link": true}
              },
              "dependencies": {"lodash": {"version": "9.9.9"}}
            }"#,
            &direct,
        );

        // The flat map wins, the root entry and the workspace link are not
        // dependencies, and nesting depth does not change a name.
        // In the file's own order: `preserve_order` is on, so a lock reads back
        // the way it was written and a report's rows do not shuffle between
        // runs for no reason.
        let names: Vec<&str> = modern.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["lodash", "tiny"]);
        assert_eq!(
            modern.iter().find(|d| d.name == "lodash").unwrap().version,
            "4.17.21"
        );
        assert!(modern.iter().find(|d| d.name == "lodash").unwrap().direct);

        // v1 has only the tree, and it is walked.
        let old = parse_package_lock(
            r#"{"lockfileVersion": 1, "dependencies": {
                 "lodash": {"version": "4.17.20", "integrity": "sha1-x",
                   "dependencies": {"nested": {"version": "0.1.0"}}}}}"#,
            &direct,
        );
        let names: BTreeSet<&str> = old.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["lodash", "nested"].into_iter().collect());
        assert!(!old.iter().find(|d| d.name == "nested").unwrap().hashed);
    }

    fn dep(name: &str, source: Option<&str>, hashed: bool) -> Dep {
        Dep {
            ecosystem: Ecosystem::Npm,
            name: name.to_string(),
            version: "1.0.0".into(),
            direct: false,
            source: source.map(str::to_string),
            hashed,
        }
    }

    /// The finding `market.rs` already refuses for this app's own catalogue,
    /// applied to the project's four hundred libraries.
    #[test]
    fn a_dependency_fetched_over_plain_http_is_the_first_finding() {
        let deps = [
            dep(
                "safe",
                Some("https://registry.npmjs.org/safe/-/safe-1.0.0.tgz"),
                true,
            ),
            dep("risky", Some("http://mirror.local/risky-1.0.0.tgz"), true),
        ];

        let found = findings(&deps);
        assert_eq!(
            found[0].id, "insecureSource",
            "first, and one row per package"
        );
        assert_eq!(found[0].subject, "risky@1.0.0");

        // And it is reported by name, not counted: whoever is on the path
        // chooses what arrives, so which package it is, is the whole point.
        assert!(found[0].detail.as_deref().unwrap().starts_with("http://"));
    }

    /// Unverified bytes are counted rather than listed.
    ///
    /// On a lock written by an older tool this is every package, and four
    /// hundred identical rows is a screen nobody reads.
    #[test]
    fn packages_nothing_verifies_are_counted() {
        let deps = [
            dep("a", Some("https://registry.npmjs.org/a"), false),
            dep("b", Some("https://registry.npmjs.org/b"), false),
            dep("c", Some("https://registry.npmjs.org/c"), true),
        ];

        let found = findings(&deps);
        let unhashed = found.iter().find(|f| f.id == "noIntegrity").unwrap();
        assert_eq!(unhashed.subject, "2");
    }

    /// A private mirror is not a fault, and it is a supply chain.
    #[test]
    fn an_index_other_than_the_ecosystems_own_is_named_and_counted() {
        let deps = [
            dep(
                "a",
                Some("https://registry.npmjs.org/a/-/a-1.0.0.tgz"),
                true,
            ),
            dep("b", Some("https://npm.corp.example/b/-/b-1.0.0.tgz"), true),
            dep("c", Some("https://npm.corp.example/c/-/c-1.0.0.tgz"), true),
        ];

        let found = findings(&deps);
        let other = found.iter().find(|f| f.id == "otherIndex").unwrap();
        assert_eq!(other.subject, "npm.corp.example");
        assert_eq!(other.detail.as_deref(), Some("2"));

        // The ordinary machine says nothing at all here.
        let usual = [dep("a", Some("https://registry.npmjs.org/a"), true)];
        assert!(!findings(&usual).iter().any(|f| f.id == "otherIndex"));
    }

    /// "No findings" and "no lock file" must not look the same.
    ///
    /// A project this cannot see is not a clean project, and `locks` is what
    /// keeps the two apart — an empty list means nothing was read.
    #[test]
    fn a_project_with_no_lock_file_says_so_rather_than_reporting_nothing_wrong() {
        let empty = report(Vec::new(), &[]);
        assert!(empty.locks.is_empty());
        assert_eq!(empty.total, 0);

        // And `advisories` stays `None` until somebody asks, which is a
        // different thing from an empty list meaning "asked, found nothing".
        assert!(empty.advisories.is_none());

        let read = report(vec!["package-lock.json".into()], &[dep("a", None, true)]);
        assert_eq!(read.locks, ["package-lock.json"]);
        assert_eq!(read.total, 1);
        // A dependency with no `resolved` still counts against the ecosystem's
        // own index, because that is where npm would have got it.
        assert_eq!(read.hosts.get("registry.npmjs.org"), Some(&1));
    }

    /// The host is taken from the URL and nothing else.
    #[test]
    fn a_source_url_yields_its_host_and_no_more() {
        assert_eq!(
            dep("a", Some("https://npm.corp.example/a/-/a-1.0.0.tgz"), true).host(),
            Some("npm.corp.example")
        );
        assert_eq!(
            dep("a", Some("https://npm.corp.example?x=1"), true).host(),
            Some("npm.corp.example")
        );
        // A git source is a URL like any other; a path with no scheme is not.
        assert_eq!(
            dep("a", Some("git+ssh://git@github.com/o/r.git"), true).host(),
            Some("git@github.com")
        );
        assert_eq!(dep("a", Some("../local/tarball.tgz"), true).host(), None);
        assert_eq!(dep("a", None, true).host(), None);
    }
}
