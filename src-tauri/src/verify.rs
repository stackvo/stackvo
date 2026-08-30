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
//! on*. It does not mean byte-identical: two machines can both satisfy
//! `redis` and be running 7.0 and 7.2. Saying which of those is right needs a
//! lock file, which is a separate item — so a version the declaration does not
//! pin is reported as [`State::Ok`] with the version it found written beside
//! it, rather than being called a match it has not checked.

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
        checks.push(service_check(service, instances, catalogue));
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
        }
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
}
