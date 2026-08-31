//! Does this machine match what the repository says the project needs?
//!
//! ## The half of onboarding nobody built
//!
//! Every tool in this category does the *setting up* half: DDEV has
//! `config.yaml` and hooks, and this app has [`crate::preset`]. None of them
//! does the **checking** half — the one that answers the question somebody
//! actually has an hour after cloning, which is not "how do I set this up" but
//! *"I did set it up; why does it still not work?"*
//!
//! The repository already declares what it needs. `stackvo.json` names the
//! runtime and its version, the web server, and — since the services field —
//! which of the twenty backing services the project expects to find around it.
//! What was missing is the sentence back: **your setup does not match, and here
//! is the line.**
//!
//! ## Nothing here measures anything new
//!
//! Every fact this compares is already computed by something else, which is why
//! this module is a pure function and not a probe:
//!
//! | Fact | Where it already comes from |
//! | --- | --- |
//! | The declaration | `manifest::Manifest`, already schema-validated |
//! | Whether the manifest is even readable | `Project::manifest_valid` |
//! | Whether the image was ever built here | `Project::built` |
//! | Whether the generated tree is older than the manifest | `Project::generated_stale` |
//! | Whether the domain resolves | `Project::domain_configured` |
//! | Which services exist, at which versions, and whether they are on | `instances::Table` |
//!
//! ## Every check carries an id, and the sentence lives in the UI
//!
//! The same arrangement [`crate::preflight`] states for the same reason: the id
//! is stable and the label is translated, so a check gains a Turkish
//! explanation by being added to a locale file rather than by having English
//! prose compiled into the binary and displayed to somebody who does not read
//! it.
//!
//! ## What "matches" can and cannot mean today
//!
//! It means *the thing the repository named is here, installed and switched
//! on*. On its own it does not mean byte-identical: two machines can both
//! satisfy `redis` and be running 7.0 and 7.2, and a declaration that names no
//! version cannot tell them apart.
//!
//! **[`crate::lock`] is what closes that**, and it is opt-in by design. When
//! the project has a `stackvo.lock`, a declared service is held against the
//! version *and the package digest* it was locked at, and three answers this
//! module could not previously give become available: the wrong version, the
//! right version out of a different package, and a lock entry for a service the
//! manifest no longer declares. Without one, the behaviour is exactly what it
//! was — the version found is reported beside an [`State::Ok`] rather than
//! being called a match nothing checked.

use crate::instances::Table;
use crate::manifest::Manifest;

/// How one check came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// This machine satisfies the line.
    Ok,
    /// The declaration names something that is not here at all.
    Missing,
    /// It is here and is not what was asked for — installed but switched off,
    /// generated output older than the manifest.
    Different,
    /// The declaration names something this app cannot check. Distinct from
    /// `Missing` on purpose: "I do not know" and "it is not there" send
    /// somebody to different places, and reporting the first as the second is
    /// how a verifier trains people to ignore it.
    Unknown,
}

/// One line of the answer.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    /// Stable key; the UI holds the label and what to do about it.
    pub id: &'static str,
    /// What the line is about — a service id, the project's name.
    pub subject: String,
    pub state: State,
    /// The facts. Not translated: this is what was found, and a version number
    /// reads the same in every language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Everything the declaration asked for, answered.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub project: String,
    pub checks: Vec<Check>,
    /// True when nothing is `Missing` or `Different`.
    ///
    /// `Unknown` deliberately does **not** hold this back. A check this app
    /// cannot make is not evidence that something is wrong, and a verifier that
    /// says "not ready" for a question it declined to ask is one people learn
    /// to override.
    pub ready: bool,
}

/// What the caller already knows about the project, without this module having
/// to reach for any of it again.
pub struct Declared<'a> {
    pub name: &'a str,
    pub manifest: &'a Manifest,
    pub manifest_valid: bool,
    pub built: bool,
    pub generated_stale: bool,
    pub domain_configured: bool,
    /// The project's `stackvo.lock`, when it has one.
    ///
    /// `None` is the overwhelmingly common case and is not a finding: a project
    /// without a lock has not failed to write one, it has not asked for one.
    /// What changes when it is present is how much a `service` check is allowed
    /// to claim — see the module comment.
    pub lock: Option<&'a crate::lock::Lock>,
    /// What the project's own `composer.json` says it needs of the platform.
    ///
    /// `None` for a project that has no `composer.json`, which is most of them:
    /// this is the Laravel half of the question and a Go project has no
    /// opinion about PHP.
    pub platform: Option<&'a Platform>,
}

/// The `require` block's demands on the platform, rather than on packages.
///
/// Read from `composer.json` — the file that states them — rather than from
/// `composer.lock`, which records what was resolved. The distinction matters
/// here: a lock file is the answer, and the question is what the project asked.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Platform {
    /// The constraint as written: `^8.3`, `>=8.1 <9.0`.
    pub php: Option<String>,
    /// The extension names with `ext-` stripped, so they are spelled the way
    /// `php.extensions` spells them.
    pub extensions: Vec<String>,
}

/// Read the platform requirements out of a `composer.json`.
///
/// `require` only, not `require-dev`. A dev requirement is a tool for the test
/// suite; failing the project's readiness on one would call a working
/// installation broken.
pub fn platform_of(composer_json: &str) -> Platform {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(composer_json) else {
        return Platform::default();
    };
    let Some(require) = json.get("require").and_then(|v| v.as_object()) else {
        return Platform::default();
    };

    Platform {
        php: require
            .get("php")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        extensions: require
            .keys()
            // `ext-pdo_mysql` is spelled `pdo_mysql` in a manifest, and
            // comparing the two spellings is how a check reports a missing
            // extension that is right there.
            .filter_map(|name| name.strip_prefix("ext-"))
            .map(str::to_ascii_lowercase)
            .collect(),
    }
}

/// The platform half of the answer: what `composer.json` asks for, against what
/// the manifest gives.
///
/// ## The failure this catches
///
/// `composer.json` says `"php": "^8.3"`. `stackvo.json` says `8.2`. The image
/// builds without complaint, and `composer install` falls over **inside the
/// container** with a platform requirement error — which names PHP and does not
/// name the file that has to change. The developer is looking at a composer
/// error and the fix is one line of a manifest.
///
/// ## Nothing new is measured
///
/// [`crate::detect::first_version`] already turns `^8.3` into `8.3`, and it is
/// already used to *choose* a PHP line at adoption. What was missing is that
/// the two were never compared again afterwards. This is a pure function over
/// two files.
///
/// ## And what it refuses to claim
///
/// A constraint with no `major.minor` in it — `*`, or a bare `^8` — yields
/// [`State::Unknown`] rather than a guess. `first_version` says so itself: a
/// bare major is not a version this app can pin, and reporting "not satisfied"
/// over a constraint nobody read is how a check teaches people to override it.
fn platform_checks(platform: &Platform, manifest: &Manifest) -> Vec<Check> {
    let Some(php) = &manifest.php else {
        // No PHP block: this project is not served by PHP, so its
        // `composer.json` — if it even has one — is not describing what runs
        // here.
        return Vec::new();
    };
    let mut out = Vec::new();

    if let Some(constraint) = &platform.php {
        let wanted = crate::detect::first_version(constraint);
        out.push(Check {
            id: "phpVersion",
            subject: constraint.clone(),
            state: match &wanted {
                None => State::Unknown,
                Some(wanted) if *wanted == php.version => State::Ok,
                Some(_) => State::Different,
            },
            // Both numbers, always — including on a pass. "8.3 asked, 8.3 given"
            // is what makes the line worth reading rather than a green tick
            // whose subject nobody can reconstruct.
            detail: Some(match wanted {
                Some(wanted) => format!(
                    "composer.json wants {wanted}, stackvo.json gives {}",
                    php.version
                ),
                None => format!("stackvo.json gives {}", php.version),
            }),
        });
    }

    // One row per missing extension, named. Counted would be the wrong shape
    // here: the repair is per extension and the name is the whole of it.
    for extension in &platform.extensions {
        let present = php
            .extensions
            .iter()
            .any(|have| have.eq_ignore_ascii_case(extension));
        if !present {
            out.push(Check {
                id: "phpExtension",
                subject: extension.clone(),
                state: State::Missing,
                detail: Some("required by composer.json".to_string()),
            });
        }
    }

    out
}

/// Hold the declaration against the machine.
///
/// `catalogue` is every service id this build knows, so a declaration naming
/// something that has never existed is told apart from one naming a service
/// nobody has installed yet — two different sentences with two different
/// repairs.
pub fn verify(declared: &Declared, instances: &Table, catalogue: &[String]) -> Report {
    let mut checks = Vec::new();

    // The manifest first, because every other line below is read out of it. A
    // manifest that does not validate makes the rest of this report a set of
    // answers about a file nobody can trust.
    checks.push(Check {
        id: "manifest",
        subject: declared.name.to_string(),
        state: if declared.manifest_valid {
            State::Ok
        } else {
            State::Different
        },
        detail: None,
    });

    for service in &declared.manifest.services {
        // The lock's answer when it has one, the presence answer otherwise.
        // Split rather than branched inside one function: the two ask different
        // questions of the same table, and a single function with a branch
        // through the middle is one nobody can read either half of.
        match declared
            .lock
            .and_then(|lock| lock.services.iter().find(|l| &l.service == service))
        {
            Some(locked) => checks.push(locked_check(locked, instances)),
            None => checks.push(service_check(service, instances, catalogue)),
        }
    }

    // A lock entry for something the manifest no longer declares. Not the
    // machine's fault and not silently ignored either: the file in the
    // repository is now describing a project that has moved on, and the repair
    // is to re-lock rather than to install anything.
    if let Some(lock) = declared.lock {
        for entry in &lock.services {
            if !declared.manifest.services.contains(&entry.service) {
                checks.push(Check {
                    id: "lockExtra",
                    subject: entry.service.clone(),
                    state: State::Different,
                    detail: Some(format!("locked at {}, no longer declared", entry.version)),
                });
            }
        }
    }

    // The Laravel half: what `composer.json` demands of the platform, against
    // what the manifest gives it. Above `built`, because a mismatch here is why
    // the build that follows will fail.
    if let Some(platform) = declared.platform {
        checks.extend(platform_checks(platform, declared.manifest));
    }

    checks.push(Check {
        id: "built",
        subject: declared.name.to_string(),
        state: if declared.built {
            State::Ok
        } else {
            State::Missing
        },
        detail: None,
    });

    checks.push(Check {
        id: "generated",
        subject: declared.name.to_string(),
        state: if declared.generated_stale {
            State::Different
        } else {
            State::Ok
        },
        detail: None,
    });

    // Only when the manifest names one. A project with no domain has not
    // failed to configure it; it has not asked for one.
    if let Some(domain) = &declared.manifest.domain {
        checks.push(Check {
            id: "domain",
            subject: domain.clone(),
            state: if declared.domain_configured {
                State::Ok
            } else {
                State::Missing
            },
            detail: None,
        });
    }

    let ready = !checks
        .iter()
        .any(|c| matches!(c.state, State::Missing | State::Different));

    Report {
        project: declared.name.to_string(),
        checks,
        ready,
    }
}

/// One **locked** service, against what is installed.
///
/// The three answers a declaration alone cannot produce. `Repackaged` is the
/// one the digest is in the file for: the right version out of a different
/// package is the substitution a version list cannot see, and on the day it
/// matters it is the only thing that explains why one machine works.
fn locked_check(locked: &crate::lock::Locked, instances: &Table) -> Check {
    use crate::lock::Drift;

    let (id, state, detail) = match crate::lock::compare(locked, instances) {
        Drift::Same => ("service", State::Ok, Some(locked.version.clone())),
        Drift::Absent => ("service", State::Missing, Some(locked.version.clone())),
        Drift::Off => ("serviceOff", State::Different, Some(locked.version.clone())),
        Drift::Version => (
            "lockedVersion",
            State::Different,
            Some(format!(
                "locked at {}, running {}",
                locked.version,
                running(&locked.service, instances).unwrap_or_else(|| "?".into())
            )),
        ),
        // The version matches, so naming it again would say nothing. What is
        // worth eight characters is which package it is now, because that is
        // the fact somebody has to go and look up.
        Drift::Repackaged => (
            "lockedPackage",
            State::Different,
            Some(format!(
                "{} from a different package ({}…)",
                locked.version,
                locked.sha256.chars().take(8).collect::<String>()
            )),
        ),
    };

    Check {
        id,
        subject: locked.service.to_string(),
        state,
        detail,
    }
}

/// The version of a service that is switched on here, if any.
fn running(service: &str, instances: &Table) -> Option<String> {
    instances
        .instances
        .iter()
        .find(|i| i.service == service && i.enabled)
        .map(|i| i.version.clone())
}

/// One declared service, against what is installed.
fn service_check(service: &str, instances: &Table, catalogue: &[String]) -> Check {
    let installed: Vec<&crate::instances::Instance> = instances
        .instances
        .iter()
        .filter(|i| i.service == service)
        .collect();

    // Nothing installed. Which sentence that is depends on whether the
    // catalogue has ever heard of it: "install it" and "this build does not
    // know that name" are different problems and only one of them is the
    // person's to fix.
    if installed.is_empty() {
        let known = catalogue.iter().any(|id| id == service);
        return Check {
            id: if known { "service" } else { "unknownService" },
            subject: service.to_string(),
            state: if known {
                State::Missing
            } else {
                State::Unknown
            },
            detail: None,
        };
    }

    // Installed and on is the answer, whichever version it is. Saying which
    // version *should* be there needs a lock file; until there is one, the
    // version found is reported rather than judged.
    match installed.iter().find(|i| i.enabled) {
        Some(on) => Check {
            id: "service",
            subject: service.to_string(),
            state: State::Ok,
            detail: Some(on.version.clone()),
        },
        None => Check {
            id: "serviceOff",
            subject: service.to_string(),
            state: State::Different,
            detail: Some(
                installed
                    .iter()
                    .map(|i| i.version.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Through `normalize`, like `agentctx`'s fixture, so what is verified here
    /// is a manifest the app would actually have produced.
    fn manifest(services: &[&str], domain: Option<&str>) -> Manifest {
        let mut json = serde_json::json!({
            "name": "shop",
            "runtime": "php",
            "services": services,
        });
        if let Some(domain) = domain {
            json["domain"] = serde_json::json!(domain);
        }
        crate::manifest::normalize(&json, "", "shop")
    }

    fn declared<'a>(manifest: &'a Manifest) -> Declared<'a> {
        Declared {
            name: "shop",
            manifest,
            manifest_valid: true,
            built: true,
            generated_stale: false,
            domain_configured: true,
            lock: None,
            platform: None,
        }
    }

    /// A manifest with a PHP block, which is what the platform checks compare
    /// against — the `manifest` helper above deliberately has none.
    fn php_manifest(version: &str, extensions: &[&str]) -> Manifest {
        crate::manifest::normalize(
            &serde_json::json!({
                "name": "shop",
                "runtime": "php",
                "php": { "version": version, "extensions": extensions },
            }),
            "",
            "shop",
        )
    }

    fn table(rows: &[(&str, &str, bool)]) -> Table {
        let mut table = Table::default();
        for (service, version, enabled) in rows {
            table.instances.push(crate::instances::Instance {
                id: format!("{service}-{}", version.replace('.', "-")),
                service: service.to_string(),
                version: version.to_string(),
                package: crate::instances::PackageRef {
                    source: "official".into(),
                    sha256: String::new(),
                    installed_at: "2026-01-01T00:00:00Z".into(),
                },
                enabled: *enabled,
                primary: true,
                ports: Default::default(),
                volumes: Default::default(),
                settings: Default::default(),
                secret_refs: Default::default(),
            });
        }
        table
    }

    fn catalogue() -> Vec<String> {
        ["mysql", "redis", "elasticsearch"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn state_of(report: &Report, id: &str, subject: &str) -> State {
        report
            .checks
            .iter()
            .find(|c| c.id == id && c.subject == subject)
            .unwrap_or_else(|| panic!("no {id} check for {subject}: {:?}", report.checks))
            .state
    }

    #[test]
    fn a_machine_that_satisfies_the_declaration_is_ready() {
        let m = manifest(&["mysql", "redis"], Some("shop.loc"));
        let report = verify(
            &declared(&m),
            &table(&[("mysql", "8.4", true), ("redis", "7.2", true)]),
            &catalogue(),
        );

        assert!(report.ready, "{:?}", report.checks);
        assert!(report.checks.iter().all(|c| c.state == State::Ok));
        // The version found travels with the line, because "redis is on" and
        // "redis 7.2 is on" are different amounts of help.
        assert_eq!(
            report
                .checks
                .iter()
                .find(|c| c.subject == "redis")
                .and_then(|c| c.detail.clone()),
            Some("7.2".to_string())
        );
    }

    /// The three ways a declared service can fail, and they are three
    /// different sentences.
    #[test]
    fn a_service_that_is_absent_off_or_unknown_says_which() {
        let m = manifest(&["mysql", "redis", "clickhouse"], None);
        let report = verify(
            &declared(&m),
            &table(&[("redis", "7.2", false)]),
            &catalogue(),
        );

        // In the catalogue, nothing installed.
        assert_eq!(state_of(&report, "service", "mysql"), State::Missing);
        // Installed, switched off — and the versions that are there are named,
        // because "install redis" would be the wrong instruction.
        assert_eq!(state_of(&report, "serviceOff", "redis"), State::Different);
        assert_eq!(
            report
                .checks
                .iter()
                .find(|c| c.id == "serviceOff")
                .and_then(|c| c.detail.clone()),
            Some("7.2".to_string())
        );
        // Not a service this build has ever heard of. Not the person's to fix,
        // so not counted against them.
        assert_eq!(
            state_of(&report, "unknownService", "clickhouse"),
            State::Unknown
        );
        assert!(!report.ready);
    }

    /// `Unknown` is not evidence that something is wrong, and a verifier that
    /// said "not ready" for a question it declined to ask is one people learn
    /// to override.
    #[test]
    fn a_question_this_app_cannot_answer_does_not_hold_the_verdict_back() {
        let m = manifest(&["clickhouse"], None);
        let report = verify(&declared(&m), &table(&[]), &catalogue());

        assert_eq!(
            state_of(&report, "unknownService", "clickhouse"),
            State::Unknown
        );
        assert!(report.ready, "an unanswerable question failed the project");
    }

    #[test]
    fn the_project_itself_is_checked_as_well_as_what_it_declares() {
        let m = manifest(&[], Some("shop.loc"));

        let mut d = declared(&m);
        d.built = false;
        assert_eq!(
            state_of(&verify(&d, &table(&[]), &catalogue()), "built", "shop"),
            State::Missing
        );

        let mut d = declared(&m);
        d.generated_stale = true;
        assert_eq!(
            state_of(&verify(&d, &table(&[]), &catalogue()), "generated", "shop"),
            State::Different
        );

        let mut d = declared(&m);
        d.domain_configured = false;
        assert_eq!(
            state_of(&verify(&d, &table(&[]), &catalogue()), "domain", "shop.loc"),
            State::Missing
        );

        let mut d = declared(&m);
        d.manifest_valid = false;
        let report = verify(&d, &table(&[]), &catalogue());
        assert_eq!(state_of(&report, "manifest", "shop"), State::Different);
        assert!(!report.ready);
    }

    /// A project with no domain has not failed to configure one.
    #[test]
    fn a_project_that_asks_for_no_domain_is_not_asked_about_one() {
        let m = manifest(&[], None);
        let mut d = declared(&m);
        d.domain_configured = false;

        let report = verify(&d, &table(&[]), &catalogue());
        assert!(!report.checks.iter().any(|c| c.id == "domain"));
        assert!(report.ready);
    }

    // ------------------------------------------------------- with a lock

    fn lock(rows: &[(&str, &str, &str)]) -> crate::lock::Lock {
        crate::lock::Lock {
            lock_version: crate::lock::SCHEMA_VERSION,
            at: "2026-08-30T09:14:02Z".into(),
            services: rows
                .iter()
                .map(|(service, version, sha)| crate::lock::Locked {
                    service: (*service).to_string(),
                    version: (*version).to_string(),
                    source: "official".into(),
                    sha256: (*sha).to_string(),
                })
                .collect(),
        }
    }

    /// The upgrade a lock buys, as one assertion.
    ///
    /// Without one, `redis` installed and on is `Ok` whichever version it is —
    /// which is honest and is as far as a declaration naming no version can go.
    /// With one, the same table is `Different` and says which two versions are
    /// in play, and that is the sentence somebody can act on.
    #[test]
    fn the_same_machine_reads_differently_once_there_is_a_lock() {
        let m = manifest(&["redis"], None);
        let instances = table(&[("redis", "7.0", true)]);

        let unlocked = verify(&declared(&m), &instances, &catalogue());
        assert_eq!(state_of(&unlocked, "service", "redis"), State::Ok);
        assert!(unlocked.ready);

        let l = lock(&[("redis", "7.2", "")]);
        let mut d = declared(&m);
        d.lock = Some(&l);
        let report = verify(&d, &instances, &catalogue());
        assert_eq!(
            state_of(&report, "lockedVersion", "redis"),
            State::Different
        );
        assert!(!report.ready);
        assert_eq!(
            report
                .checks
                .iter()
                .find(|c| c.id == "lockedVersion")
                .and_then(|c| c.detail.clone()),
            Some("locked at 7.2, running 7.0".to_string()),
            "both versions, because one of them alone is not actionable"
        );
    }

    /// The right version out of a different package.
    ///
    /// The answer no version list can give, and the reason the digest is in the
    /// file at all. `table` writes an empty sha256, so a lock carrying one is
    /// enough to make the two disagree.
    #[test]
    fn a_republished_version_is_not_the_version_that_was_locked() {
        let m = manifest(&["redis"], None);
        let l = lock(&[("redis", "7.2", "0123456789abcdef")]);
        let mut d = declared(&m);
        d.lock = Some(&l);

        let report = verify(&d, &table(&[("redis", "7.2", true)]), &catalogue());
        let check = report
            .checks
            .iter()
            .find(|c| c.id == "lockedPackage")
            .expect("the digest disagreed and nothing said so");
        assert_eq!(check.state, State::Different);
        assert!(
            check.detail.as_ref().unwrap().contains("01234567"),
            "the package it should be is the fact somebody has to go and look up"
        );
    }

    /// A lock entry for a service the manifest no longer declares.
    ///
    /// Nothing to install. The file in the repository has fallen behind the
    /// project, and the repair is to re-lock — which is a different sentence
    /// from every other line on this report.
    #[test]
    fn a_lock_that_names_a_service_the_manifest_dropped_says_so() {
        let m = manifest(&["redis"], None);
        let l = lock(&[("redis", "7.2", ""), ("kafka", "3.7", "")]);
        let mut d = declared(&m);
        d.lock = Some(&l);

        let report = verify(&d, &table(&[("redis", "7.2", true)]), &catalogue());
        assert_eq!(state_of(&report, "service", "redis"), State::Ok);
        assert_eq!(state_of(&report, "lockExtra", "kafka"), State::Different);
        assert!(!report.ready);
    }

    /// A declared service the lock does not name keeps the old behaviour.
    ///
    /// The lock has nothing to say about it, and inventing a finding out of
    /// that would punish somebody for locking a project before installing the
    /// rest of it. `project_lock` already reported what it could not lock, at
    /// the moment it could not lock it.
    #[test]
    fn a_service_outside_the_lock_is_answered_the_way_it_always_was() {
        let m = manifest(&["redis", "mysql"], None);
        let l = lock(&[("redis", "7.2", "")]);
        let mut d = declared(&m);
        d.lock = Some(&l);

        let report = verify(
            &d,
            &table(&[("redis", "7.2", true), ("mysql", "8.0", true)]),
            &catalogue(),
        );
        assert_eq!(state_of(&report, "service", "redis"), State::Ok);
        assert_eq!(state_of(&report, "service", "mysql"), State::Ok);
        assert!(report.ready);
    }

    /// The failure this exists for: composer wants 8.3, the manifest gives 8.2,
    /// the image builds fine and `composer install` dies inside the container
    /// naming PHP but not the file that has to change.
    #[test]
    fn the_php_the_project_asks_for_is_held_against_the_php_it_is_given() {
        let m = php_manifest("8.2", &["mbstring"]);
        let mut d = declared(&m);
        let platform =
            platform_of(r#"{ "require": { "php": "^8.3", "laravel/framework": "^12.0" } }"#);
        d.platform = Some(&platform);

        let report = verify(&d, &table(&[]), &catalogue());
        assert_eq!(state_of(&report, "phpVersion", "^8.3"), State::Different);
        assert!(!report.ready);

        // Both numbers on the line, so the reader does not have to go and find
        // the second one.
        let detail = report
            .checks
            .iter()
            .find(|c| c.id == "phpVersion")
            .and_then(|c| c.detail.clone())
            .unwrap();
        assert!(detail.contains("8.3") && detail.contains("8.2"), "{detail}");
    }

    /// A matching pair passes and still says both numbers.
    #[test]
    fn a_matching_php_line_is_ok_and_still_reports_what_it_compared() {
        let m = php_manifest("8.3", &[]);
        let mut d = declared(&m);
        let platform = platform_of(r#"{ "require": { "php": ">=8.3 <9.0" } }"#);
        d.platform = Some(&platform);

        let report = verify(&d, &table(&[]), &catalogue());
        assert_eq!(state_of(&report, "phpVersion", ">=8.3 <9.0"), State::Ok);
        assert!(report.ready);
    }

    /// A constraint with no `major.minor` is `Unknown`, not a guess and not a
    /// failure — a check that says "not satisfied" over something nobody read
    /// is one people learn to override.
    #[test]
    fn a_constraint_this_cannot_read_is_not_reported_as_a_mismatch() {
        for constraint in ["*", "^8"] {
            let m = php_manifest("8.4", &[]);
            let mut d = declared(&m);
            let platform = platform_of(&format!(r#"{{ "require": {{ "php": "{constraint}" }} }}"#));
            d.platform = Some(&platform);

            let report = verify(&d, &table(&[]), &catalogue());
            assert_eq!(state_of(&report, "phpVersion", constraint), State::Unknown);
            // `Unknown` never holds `ready` back.
            assert!(report.ready, "{constraint}");
        }
    }

    /// The second half: `ext-*` against `php.extensions`, one row per missing
    /// name, and `require-dev` left out of it.
    #[test]
    fn each_missing_extension_is_named_and_dev_requirements_are_not_counted() {
        let m = php_manifest("8.3", &["mbstring", "pdo_mysql"]);
        let mut d = declared(&m);
        let platform = platform_of(
            r#"{
                "require": { "php": "^8.3", "ext-mbstring": "*", "ext-pdo_mysql": "*", "ext-intl": "*" },
                "require-dev": { "ext-xdebug": "*" }
            }"#,
        );
        d.platform = Some(&platform);

        let report = verify(&d, &table(&[]), &catalogue());

        // `ext-` is stripped before the comparison. `ext-pdo_mysql` and
        // `pdo_mysql` are the same extension in two files' spellings, and
        // reporting it as missing would send somebody looking for something
        // that is right there.
        let missing: Vec<&str> = report
            .checks
            .iter()
            .filter(|c| c.id == "phpExtension")
            .map(|c| c.subject.as_str())
            .collect();
        assert_eq!(missing, ["intl"]);
    }

    /// A project with no PHP block is not asked PHP questions, whatever its
    /// `composer.json` happens to say.
    #[test]
    fn a_project_that_is_not_php_gains_no_platform_lines() {
        let m = manifest(&[], None);
        let mut d = declared(&m);
        let platform = platform_of(r#"{ "require": { "php": "^8.3", "ext-intl": "*" } }"#);
        d.platform = Some(&platform);

        let report = verify(&d, &table(&[]), &catalogue());
        assert!(!report.checks.iter().any(|c| c.id.starts_with("php")));
    }
}
