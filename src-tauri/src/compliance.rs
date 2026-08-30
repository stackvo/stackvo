//! Which clause of the policy is actually in force on this machine.
//!
//! ## The gap this closes
//!
//! [`crate::policy`] reads a file an administrator pushed, and `policy_status`
//! reports it back. Every field on that call answers *what the file says*.
//! Nothing answered the question the person who deployed the file has, which
//! is a different one: **is any of it holding here?**
//!
//! The two come apart for one reason, and it is not misconduct. A policy
//! arrives on a machine that was already set up. The registry mirror rewrites
//! image references as files are *generated*, so a project nobody regenerated
//! since Tuesday still pulls from Docker Hub. `allowedPackages` is checked as
//! something is *installed*, so a service installed last month stays installed
//! when the list that would have refused it lands today. `requireSignature`
//! decides what the **next** refresh accepts and says nothing about the index
//! already in the cache.
//!
//! So the honest report is not a re-read of the file. It is a measurement, per
//! clause, of what this machine currently is — and most of what it finds is not
//! somebody breaking a rule, it is a rule that arrived after the fact and has
//! work left to do.
//!
//! ## Four states, and why "no opinion" is not "compliant"
//!
//! | State | Means |
//! | --- | --- |
//! | [`State::Holding`] | Measured, and this machine is inside the clause |
//! | [`State::Bypassed`] | Measured, and something here is outside it |
//! | [`State::Silent`] | The policy says nothing on this subject |
//! | [`State::Unmeasured`] | No evidence either way, and the reason is named |
//!
//! [`State::Silent`] exists so an empty list cannot be counted as a pass. Every
//! list in [`crate::policy::Market`] means *no opinion* when empty — never
//! "none" — and a report that folded that into a green tick would score a
//! machine with no policy at all as fully compliant, which is the single most
//! misleading thing a compliance report can do.
//!
//! [`State::Unmeasured`] covers two things that look different and are the same
//! for this purpose: a fact this app cannot see (the generated tree would not
//! read), and a clause with nothing here to apply to (a pin naming a repository
//! this build never runs). Neither is evidence of compliance, and the whole
//! value of the report is that it never treats absence of evidence as evidence.
//!
//! ## `attestable`, not `compliant`
//!
//! [`crate::verify`] has the same shape and makes the **opposite** call: its
//! `Unknown` does not hold `ready` back, because a check it declined to make is
//! not evidence that the project is broken. That is right for the question
//! verify asks — *can I work?*
//!
//! This module asks a different one — *can somebody sign their name to this?*
//! — and there an unasked question is exactly what an attestation must not
//! swallow. So [`Report::attestable`] is false while anything is `Bypassed`
//! **or** `Unmeasured`, and it is called `attestable` rather than `compliant`
//! so that nobody reads it as a certificate this app is in no position to
//! issue. The layer it reports on is not a security boundary — `policy.rs` says
//! so in its first paragraph — and a report that implied otherwise would be
//! selling the same lock with the same key taped to it.
//!
//! ## A pure function
//!
//! Everything below takes what the caller already measured and returns rows.
//! No file is read here, nothing is spawned, and there is no clock: the same
//! [`Observed`] gives the same [`Report`] on any machine, which is what lets
//! every branch have a test instead of a fixture directory.

use std::collections::BTreeMap;

/// How one clause came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Measured, and this machine is inside the clause.
    Holding,
    /// Measured, and something on this machine is outside it.
    Bypassed,
    /// The policy has no opinion on this subject. Never a pass — see the
    /// module comment on why an empty list is silence rather than a refusal.
    Silent,
    /// No evidence either way. The reason is in `detail`, always.
    Unmeasured,
}

/// One line of the report.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clause {
    /// Stable key; the UI holds the sentence and what to do about it — the
    /// arrangement [`crate::preflight`] states and [`crate::verify`] follows.
    pub id: &'static str,
    /// What the line is about: a key, an image repository, a package, a file.
    pub subject: String,
    pub state: State,
    /// What was measured. Not translated — a version, a path and a key id read
    /// the same in every language, and this is the half somebody pastes into a
    /// ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Clause {
    fn new(id: &'static str, subject: impl Into<String>, state: State) -> Self {
        Self {
            id,
            subject: subject.into(),
            state,
            detail: None,
        }
    }

    fn with(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// Every clause, answered, with the counts that make the summary line.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Whether an administrator has said anything at all on this machine.
    pub active: bool,
    /// Which file this came from, so a finding can be taken to somebody.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub clauses: Vec<Clause>,
    pub holding: usize,
    pub bypassed: usize,
    pub silent: usize,
    pub unmeasured: usize,
    /// False while anything is bypassed **or** unmeasured. See the module
    /// comment: this is not a certificate, it is "nothing here is unaccounted
    /// for".
    pub attestable: bool,
}

// ---------------------------------------------------------------- the inputs

/// One generated file as it currently sits on disk.
#[derive(Debug, Clone)]
pub struct Generated {
    /// The label the generator writes it under — `docker-compose.projects.yml`,
    /// `projects/shop/Dockerfile`.
    pub label: String,
    /// Whether re-applying the mirror to this file's own bytes would change
    /// them.
    ///
    /// Computed by the caller with [`crate::policy::rewrite`] over the text it
    /// read, which is exact and cannot drift: the question "did the mirror
    /// reach this file" is answered by the mirror itself, with its own rules
    /// about build stages, registries already named, and the three references
    /// it deliberately leaves alone. A second parser here would be a second
    /// opinion, and the wrong one would be this module's.
    pub would_change: bool,
}

/// One installed package, and the image its manifest names.
#[derive(Debug, Clone)]
pub struct Package {
    pub service: String,
    pub version: String,
    /// `None` when the manifest would not load — which is itself a fact the
    /// report has to carry rather than skip.
    pub image: Option<String>,
}

/// What a project declares that the `hooks` and `providers` blocks decide about.
#[derive(Debug, Clone)]
pub struct ProjectFacts {
    pub name: String,
    /// Steps that would run a command on the machine rather than in the
    /// project's container.
    pub host_steps: usize,
    /// Steps of any kind.
    pub steps: usize,
    /// Declared providers.
    pub providers: usize,
    /// Of those, ones that offer the direction that writes somewhere else.
    pub push_providers: usize,
}

/// Everything measured, handed in so this module reads no file itself.
pub struct Observed<'a> {
    /// The workspace's own `.env`, parsed **without** the policy layer.
    ///
    /// Without, deliberately. [`crate::config::Env::load`] applies the policy
    /// last, so asking the loaded environment whether the policy took effect
    /// is asking a question whose answer is `true` by construction. The file is
    /// where the two can genuinely disagree, and the disagreement matters: the
    /// app resolves a locked key to the policy's value, and anything reading
    /// the file directly — a `docker compose` run from a terminal, a script,
    /// the Bash generator — gets what the file says.
    pub env_file: &'a BTreeMap<String, String>,
    /// Generated files on disk. `None` when the tree would not read.
    pub generated: Option<&'a [Generated]>,
    /// Every image this build runs, resolved against the policy in force.
    pub images: &'a [crate::images::Listed],
    /// What is installed. `None` when the package tree would not open.
    pub packages: Option<&'a [Package]>,
    /// The catalogue source this workspace last fetched from.
    pub source: Option<&'a crate::market::SourceRef>,
    /// Files this workspace has put in front of a published package.
    pub overrides: &'a [crate::overrides::Override],
    /// The projects, as their manifests declare them.
    pub projects: &'a [ProjectFacts],
}

// ---------------------------------------------------------------- the measure

/// Hold every clause of the policy against what was observed.
pub fn measure(policy: &crate::policy::Policy, observed: &Observed) -> Report {
    let mut clauses = Vec::new();

    settings(policy, observed, &mut clauses);
    mirror(policy, observed, &mut clauses);
    pins(policy, observed, &mut clauses);
    market(policy, observed, &mut clauses);
    hooks(policy, observed, &mut clauses);
    providers(policy, observed, &mut clauses);

    let count = |want: State| clauses.iter().filter(|c| c.state == want).count();
    let bypassed = count(State::Bypassed);
    let unmeasured = count(State::Unmeasured);

    Report {
        active: policy.is_active(),
        source: policy.source().map(|p| p.display().to_string()),
        holding: count(State::Holding),
        silent: count(State::Silent),
        bypassed,
        unmeasured,
        attestable: bypassed == 0 && unmeasured == 0,
        clauses,
    }
}

/// `settings` and `locked`: does the file on disk agree with what was pushed?
///
/// A locked key the workspace also sets to something else is the finding, and
/// it is reported as bypassed rather than as a note. The app itself resolves
/// the policy's value — precedence guarantees that — so nothing *this app* does
/// is wrong. What is wrong is that the machine now has two answers to one
/// question, and the one the file gives is the one every other tool on the
/// machine reads.
fn settings(policy: &crate::policy::Policy, observed: &Observed, out: &mut Vec<Clause>) {
    if policy.settings().is_empty() {
        out.push(Clause::new("settings", "settings", State::Silent));
        return;
    }

    for (key, pushed) in policy.settings() {
        let id = if policy.is_locked(key) {
            "settings.locked"
        } else {
            "settings.managed"
        };
        match observed.env_file.get(key) {
            // The file does not mention it: nothing to disagree with, and the
            // policy's value is the only one on the machine.
            None => out.push(Clause::new(id, key, State::Holding).with(pushed.clone())),
            Some(local) if local == pushed => {
                out.push(Clause::new(id, key, State::Holding).with(pushed.clone()))
            }
            Some(local) => out.push(
                Clause::new(id, key, State::Bypassed)
                    .with(format!("`.env` holds {local}, the policy pushed {pushed}")),
            ),
        }
    }
}

/// `registryPrefix`: did the mirror reach what is already on disk?
///
/// The mirror runs as files are generated, which means it is not a property of
/// the machine — it is a property of the last time somebody pressed regenerate.
/// A file written before the policy arrived names Docker Hub and will keep
/// doing so until it is written again, and on a network where Docker Hub is not
/// reachable that is not a compliance detail, it is why the stack will not
/// start.
fn mirror(policy: &crate::policy::Policy, observed: &Observed, out: &mut Vec<Clause>) {
    let Some(prefix) = policy.registry_prefix() else {
        out.push(Clause::new(
            "registry.mirror",
            "registryPrefix",
            State::Silent,
        ));
        return;
    };

    let Some(files) = observed.generated else {
        out.push(
            Clause::new("registry.mirror", "generated", State::Unmeasured)
                .with("the generated tree could not be read"),
        );
        return;
    };

    let stale: Vec<&Generated> = files.iter().filter(|f| f.would_change).collect();
    if stale.is_empty() {
        out.push(
            Clause::new("registry.mirror", prefix.to_string(), State::Holding)
                .with(format!("{} generated files, all mirrored", files.len())),
        );
        return;
    }

    // One row per file rather than a count. The repair is per file — regenerate
    // that project — and a number tells somebody there is work without telling
    // them where it is.
    for file in stale {
        out.push(
            Clause::new("registry.mirror", file.label.clone(), State::Bypassed)
                .with("generated before the mirror; regenerate to apply it"),
        );
    }
}

/// `imagePins`: is the pin in force, and is it in force over anything?
///
/// A pin naming a repository this build never runs is not a violation and is
/// not compliance either. It is a line that does nothing, which is the same
/// failure `policy.rs` already names for a locked key that is not also set —
/// *"do not change this" without saying to what* — and it gets the same
/// treatment here: reported, never counted as holding.
fn pins(policy: &crate::policy::Policy, observed: &Observed, out: &mut Vec<Clause>) {
    let pinned: Vec<&crate::images::Listed> = observed.images.iter().filter(|i| i.pinned).collect();

    for image in &pinned {
        out.push(
            Clause::new("images.pin", image.repository.clone(), State::Holding)
                .with(image.effective.clone()),
        );
    }

    // A pin over a repository nothing here runs. Found from the policy side,
    // because the images side by construction only lists what this build pulls
    // — so a typo in the file is invisible from there.
    let inert = policy
        .image_pins()
        .keys()
        .filter(|repo| !observed.images.iter().any(|i| &i.repository == *repo));

    for repo in inert {
        out.push(
            Clause::new("images.pin", repo.clone(), State::Unmeasured)
                .with("this build runs no image from that repository"),
        );
    }

    if pinned.is_empty() && policy.image_pins().is_empty() {
        out.push(Clause::new("images.pin", "imagePins", State::Silent));
    }
}

/// The `market` block, held against what is installed and what was fetched.
fn market(policy: &crate::policy::Policy, observed: &Observed, out: &mut Vec<Clause>) {
    let market = policy.market();

    // ---- allowedPackages -------------------------------------------------
    //
    // Checked at install time, so this is the clause a tightened list changes
    // the answer to without anything happening on the machine.
    if market.allowed_packages.is_empty() {
        out.push(Clause::new(
            "market.allowedPackages",
            "allowedPackages",
            State::Silent,
        ));
    } else {
        match observed.packages {
            None => out.push(
                Clause::new("market.allowedPackages", "packages", State::Unmeasured)
                    .with("the installed packages could not be read"),
            ),
            Some(packages) => {
                for package in packages {
                    let state = if market.allows_package(&package.service) {
                        State::Holding
                    } else {
                        State::Bypassed
                    };
                    let clause = Clause::new(
                        "market.allowedPackages",
                        format!("{}@{}", package.service, package.version),
                        state,
                    );
                    out.push(if state == State::Bypassed {
                        clause.with("installed, and the list in force does not name it")
                    } else {
                        clause
                    });
                }
            }
        }
    }

    // ---- allowedRegistries ----------------------------------------------
    if market.allowed_registries.is_empty() {
        out.push(Clause::new(
            "market.allowedRegistries",
            "allowedRegistries",
            State::Silent,
        ));
    } else {
        match observed.packages {
            None => out.push(
                Clause::new("market.allowedRegistries", "packages", State::Unmeasured)
                    .with("the installed packages could not be read"),
            ),
            Some(packages) => {
                for package in packages {
                    let subject = format!("{}@{}", package.service, package.version);
                    match &package.image {
                        // A manifest that will not load is not a pass. It is
                        // the one package whose image nobody can name.
                        None => out.push(
                            Clause::new("market.allowedRegistries", subject, State::Unmeasured)
                                .with("its manifest would not load"),
                        ),
                        Some(image) if market.allows_registry(image) => out.push(
                            Clause::new("market.allowedRegistries", subject, State::Holding)
                                .with(image.clone()),
                        ),
                        Some(image) => out.push(
                            Clause::new("market.allowedRegistries", subject, State::Bypassed)
                                .with(format!("{image} is not from an allowed registry")),
                        ),
                    }
                }
            }
        }
    }

    // ---- allowedSources --------------------------------------------------
    if market.allowed_sources.is_empty() {
        out.push(Clause::new(
            "market.allowedSources",
            "allowedSources",
            State::Silent,
        ));
    } else {
        match observed.source {
            None => out.push(
                Clause::new("market.allowedSources", "source", State::Unmeasured)
                    .with("this workspace has never fetched a catalogue"),
            ),
            Some(source) if market.allows_source(&source.location) => out.push(Clause::new(
                "market.allowedSources",
                source.location.clone(),
                State::Holding,
            )),
            Some(source) => out.push(
                Clause::new(
                    "market.allowedSources",
                    source.location.clone(),
                    State::Bypassed,
                )
                .with("the cached index came from a source the list does not name"),
            ),
        }
    }

    // ---- requireSignature ------------------------------------------------
    //
    // The rule decides what the **next** refresh accepts. What is already in
    // the cache was accepted under whatever rule was in force then, and
    // `verified_by` is the only record of which — written at refresh, absent
    // on a `source.json` from a build that predates the field, which reads as
    // "not verified" for the reason `market.rs` gives: the alternative is a
    // machine claiming a check that never ran.
    if !market.require_signature {
        out.push(Clause::new(
            "market.requireSignature",
            "requireSignature",
            State::Silent,
        ));
    } else {
        match observed.source {
            None => out.push(
                Clause::new("market.requireSignature", "source", State::Unmeasured)
                    .with("this workspace has never fetched a catalogue"),
            ),
            Some(source) => match &source.verified_by {
                Some(key) => out.push(
                    Clause::new(
                        "market.requireSignature",
                        source.location.clone(),
                        State::Holding,
                    )
                    .with(format!("the cached index was verified by {key}")),
                ),
                None => out.push(
                    Clause::new(
                        "market.requireSignature",
                        source.location.clone(),
                        State::Bypassed,
                    )
                    .with("the cached index was accepted before the rule; refresh to re-check it"),
                ),
            },
        }
    }

    // ---- allowOverrides --------------------------------------------------
    //
    // The only clause here whose evidence is a file. `allowOverrides` false
    // stops the app from materialising a new one and does nothing about the
    // ones already there, which keep being read in front of the published
    // bytes — so on this clause the report is the whole enforcement.
    match market.allow_overrides {
        None | Some(true) => out.push(Clause::new(
            "market.allowOverrides",
            "allowOverrides",
            State::Silent,
        )),
        Some(false) if observed.overrides.is_empty() => out.push(
            Clause::new("market.allowOverrides", "overrides", State::Holding)
                .with("no workspace file stands in front of a published one"),
        ),
        Some(false) => {
            for over in observed.overrides {
                out.push(
                    Clause::new(
                        "market.allowOverrides",
                        format!("{}@{} {}", over.service, over.version, over.path),
                        State::Bypassed,
                    )
                    .with("written before the rule, and still read in front of the package"),
                );
            }
        }
    }

    // ---- autoUpdate ------------------------------------------------------
    //
    // Nothing on disk records whether a replacement happened, so there is no
    // measurement to make and saying so is the entry. An `autoUpdate: false`
    // scored green here would be a green tick for a question nobody asked.
    match market.auto_update {
        None => out.push(Clause::new(
            "market.autoUpdate",
            "autoUpdate",
            State::Silent,
        )),
        Some(value) => out.push(
            Clause::new("market.autoUpdate", "autoUpdate", State::Unmeasured).with(format!(
                "set to {value}; nothing on this machine records whether a package was \
                 replaced on its own"
            )),
        ),
    }
}

/// The `hooks` block: how much this machine is actually being stopped from.
///
/// A refusal that stops nothing and a refusal that stops forty steps are both
/// "holding", and only one of them is worth an administrator's attention — so
/// the count is the detail. Nothing here can be bypassed: hooks are gated as
/// they run, from the policy read at that moment.
fn hooks(policy: &crate::policy::Policy, observed: &Observed, out: &mut Vec<Clause>) {
    if !policy.constrains_hooks() {
        out.push(Clause::new("hooks", "hooks", State::Silent));
        return;
    }

    let steps: usize = observed.projects.iter().map(|p| p.steps).sum();
    let host: usize = observed.projects.iter().map(|p| p.host_steps).sum();

    if !policy.hooks().enabled {
        out.push(
            Clause::new("hooks.enabled", "hooks", State::Holding)
                .with(format!("{steps} declared steps will not run")),
        );
    } else if !policy.hooks().allow_host {
        out.push(
            Clause::new("hooks.allowHost", "hooks", State::Holding).with(format!(
                "{host} of {steps} declared steps run on the host and will not"
            )),
        );
    }
}

/// The `providers` block, measured the same way and for the same reason.
fn providers(policy: &crate::policy::Policy, observed: &Observed, out: &mut Vec<Clause>) {
    if !policy.constrains_providers() {
        out.push(Clause::new("providers", "providers", State::Silent));
        return;
    }

    let declared: usize = observed.projects.iter().map(|p| p.providers).sum();
    let pushes: usize = observed.projects.iter().map(|p| p.push_providers).sum();

    if !policy.providers().enabled {
        out.push(
            Clause::new("providers.enabled", "providers", State::Holding)
                .with(format!("{declared} declared providers will not run")),
        );
    } else if !policy.providers().allow_push {
        out.push(
            Clause::new("providers.allowPush", "providers", State::Holding).with(format!(
                "{pushes} of {declared} declared providers offer a push and will not"
            )),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn policy(json: &str) -> crate::policy::Policy {
        crate::policy::Policy::parse(json, Path::new("/etc/stackvo/policy.json"))
    }

    fn nothing() -> Observed<'static> {
        static EMPTY_ENV: std::sync::OnceLock<BTreeMap<String, String>> =
            std::sync::OnceLock::new();
        Observed {
            env_file: EMPTY_ENV.get_or_init(BTreeMap::new),
            generated: None,
            images: &[],
            packages: None,
            source: None,
            overrides: &[],
            projects: &[],
        }
    }

    fn find<'a>(report: &'a Report, id: &str, subject: &str) -> &'a Clause {
        report
            .clauses
            .iter()
            .find(|c| c.id == id && c.subject == subject)
            .unwrap_or_else(|| {
                panic!(
                    "no {id}/{subject} in {:?}",
                    report
                        .clauses
                        .iter()
                        .map(|c| (c.id, c.subject.as_str(), c.state))
                        .collect::<Vec<_>>()
                )
            })
    }

    /// The property the whole module is built on, as one assertion.
    ///
    /// An unmanaged machine satisfies nothing, and a report that scored it as
    /// compliant would be the single most misleading thing this could do — so
    /// every clause is `Silent` and `attestable` is still true, because there
    /// is genuinely nothing outstanding. The two halves are different claims
    /// and the test holds both.
    #[test]
    fn a_machine_with_no_policy_passes_nothing_and_owes_nothing() {
        let report = measure(&crate::policy::Policy::none(), &nothing());

        assert!(!report.active);
        assert_eq!(report.holding, 0, "silence is not a pass");
        assert_eq!(report.bypassed, 0);
        assert_eq!(report.unmeasured, 0);
        assert!(report.silent > 0);
        assert!(report.attestable);
    }

    /// A locked key the workspace also writes, differently.
    ///
    /// Precedence means the app itself resolves the administrator's value, so
    /// nothing this app does is wrong — and the machine still has two answers
    /// to one question, of which the file's is the one every other tool on it
    /// reads. That is the finding, and it is a bypass rather than a note.
    #[test]
    fn a_locked_key_the_file_disagrees_with_is_a_bypass() {
        let policy = policy(
            r#"{"schemaVersion":1,
                "settings":{"DEFAULT_TLD_SUFFIX":"corp.test"},
                "locked":["DEFAULT_TLD_SUFFIX"]}"#,
        );

        let mut env = BTreeMap::new();
        env.insert("DEFAULT_TLD_SUFFIX".to_string(), "loc".to_string());
        let observed = Observed {
            env_file: &env,
            ..nothing()
        };

        let report = measure(&policy, &observed);
        let clause = find(&report, "settings.locked", "DEFAULT_TLD_SUFFIX");
        assert_eq!(clause.state, State::Bypassed);
        assert!(clause.detail.as_ref().unwrap().contains("loc"));
        assert!(!report.attestable);

        // And the file agreeing — or saying nothing at all — holds.
        env.insert("DEFAULT_TLD_SUFFIX".to_string(), "corp.test".to_string());
        let agreed = measure(
            &policy,
            &Observed {
                env_file: &env,
                ..nothing()
            },
        );
        assert_eq!(
            find(&agreed, "settings.locked", "DEFAULT_TLD_SUFFIX").state,
            State::Holding
        );
    }

    /// The mirror is a property of the last regenerate, not of the machine.
    ///
    /// This is the clause that made the module worth writing: a file generated
    /// before the policy arrived still names Docker Hub, and on a network where
    /// Docker Hub is not reachable that is not a compliance detail, it is why
    /// the stack will not start.
    #[test]
    fn a_file_generated_before_the_mirror_is_named_one_by_one() {
        let policy = policy(r#"{"schemaVersion":1,"registryPrefix":"registry.corp/proxy"}"#);

        let files = [
            Generated {
                label: "docker-compose.projects.yml".into(),
                would_change: true,
            },
            Generated {
                label: "projects/shop/Dockerfile".into(),
                would_change: false,
            },
        ];
        let report = measure(
            &policy,
            &Observed {
                generated: Some(&files),
                ..nothing()
            },
        );

        // Per file, because the repair is per file. A count would tell somebody
        // there is work without telling them where it is.
        let stale = find(&report, "registry.mirror", "docker-compose.projects.yml");
        assert_eq!(stale.state, State::Bypassed);
        assert_eq!(report.bypassed, 1);
        assert!(!report.attestable);

        // A tree that would not read is not a pass either.
        let unread = measure(&policy, &nothing());
        assert_eq!(
            find(&unread, "registry.mirror", "generated").state,
            State::Unmeasured
        );
        assert!(!unread.attestable);
    }

    /// A pin over a repository nothing here runs is a line that does nothing.
    ///
    /// The same failure `policy.rs` names for a locked key it does not also set
    /// — "do not change this" without saying to what — and it gets the same
    /// treatment: reported, and never counted as holding.
    #[test]
    fn a_pin_with_nothing_to_apply_to_is_not_compliance() {
        let policy = policy(
            r#"{"schemaVersion":1,
                "imagePins":{"cloudflare/cloudflared":"cloudflare/cloudflared:2024.8.2",
                             "acme/typo":"acme/typo:1"}}"#,
        );

        let images = [crate::images::Listed {
            repository: "cloudflare/cloudflared".into(),
            used_for: "tunnels".into(),
            shipped: "cloudflare/cloudflared:latest".into(),
            effective: "cloudflare/cloudflared:2024.8.2".into(),
            moving: false,
            pinned: true,
        }];

        let report = measure(
            &policy,
            &Observed {
                images: &images,
                ..nothing()
            },
        );

        assert_eq!(
            find(&report, "images.pin", "cloudflare/cloudflared").state,
            State::Holding
        );
        assert_eq!(
            find(&report, "images.pin", "acme/typo").state,
            State::Unmeasured
        );
        assert!(!report.attestable);
    }

    /// A package installed before the list that would have refused it.
    ///
    /// Nothing happened on this machine; the rule moved. The report has to say
    /// so, because `allowedPackages` is checked at install time and will never
    /// look at this package again.
    #[test]
    fn a_package_the_list_no_longer_names_is_still_installed() {
        let policy = policy(r#"{"schemaVersion":1,"market":{"allowedPackages":["mysql"]}}"#);

        let packages = [
            Package {
                service: "mysql".into(),
                version: "8.0".into(),
                image: Some("mysql:8.0".into()),
            },
            Package {
                service: "kafka".into(),
                version: "3.7".into(),
                image: Some("bitnami/kafka:3.7".into()),
            },
        ];
        let report = measure(
            &policy,
            &Observed {
                packages: Some(&packages),
                ..nothing()
            },
        );

        assert_eq!(
            find(&report, "market.allowedPackages", "mysql@8.0").state,
            State::Holding
        );
        assert_eq!(
            find(&report, "market.allowedPackages", "kafka@3.7").state,
            State::Bypassed
        );
    }

    /// `requireSignature` decides what the next refresh accepts, and says
    /// nothing about the index already in the cache.
    ///
    /// A missing `verified_by` reads as "not verified" for the reason
    /// `market.rs` gives — the alternative is a machine claiming a check that
    /// never ran — so turning the rule on does not retroactively make the
    /// cached catalogue trusted, and this is where somebody finds that out.
    #[test]
    fn turning_on_require_signature_does_not_bless_what_is_already_cached() {
        let policy = policy(r#"{"schemaVersion":1,"market":{"requireSignature":true}}"#);

        let unverified = crate::market::SourceRef {
            kind: "https".into(),
            location: "https://mirror.corp/packages".into(),
            verified_by: None,
        };
        let report = measure(
            &policy,
            &Observed {
                source: Some(&unverified),
                ..nothing()
            },
        );
        assert_eq!(
            find(
                &report,
                "market.requireSignature",
                "https://mirror.corp/packages"
            )
            .state,
            State::Bypassed
        );

        let verified = crate::market::SourceRef {
            verified_by: Some("corp-2025".into()),
            ..unverified.clone()
        };
        let after = measure(
            &policy,
            &Observed {
                source: Some(&verified),
                ..nothing()
            },
        );
        let clause = find(
            &after,
            "market.requireSignature",
            "https://mirror.corp/packages",
        );
        assert_eq!(clause.state, State::Holding);
        assert!(clause.detail.as_ref().unwrap().contains("corp-2025"));
    }

    /// An override file written before `allowOverrides: false` arrived.
    ///
    /// The setting stops a new one being materialised and does nothing about
    /// the ones already on disk, which keep being read in front of the
    /// published bytes — so on this clause the report is the whole enforcement.
    #[test]
    fn an_override_that_predates_the_rule_is_still_in_front_of_the_package() {
        let policy = policy(r#"{"schemaVersion":1,"market":{"allowOverrides":false}}"#);

        let overrides = [crate::overrides::Override {
            service: "mysql".into(),
            version: "8.0".into(),
            path: "my.cnf".into(),
        }];
        let report = measure(
            &policy,
            &Observed {
                overrides: &overrides,
                ..nothing()
            },
        );
        assert_eq!(
            find(&report, "market.allowOverrides", "mysql@8.0 my.cnf").state,
            State::Bypassed
        );

        // And with none on disk it holds, rather than being silent — the
        // difference between "the rule is doing its job" and "nobody asked".
        let clean = measure(&policy, &nothing());
        assert_eq!(
            find(&clean, "market.allowOverrides", "overrides").state,
            State::Holding
        );
    }

    /// A refusal that stops nothing and one that stops forty steps are both
    /// holding, and only one of them is worth somebody's attention.
    #[test]
    fn a_refusal_reports_how_much_it_is_actually_refusing() {
        let policy = policy(r#"{"schemaVersion":1,"hooks":{"allowHost":false}}"#);

        let projects = [
            ProjectFacts {
                name: "shop".into(),
                host_steps: 2,
                steps: 5,
                providers: 0,
                push_providers: 0,
            },
            ProjectFacts {
                name: "blog".into(),
                host_steps: 1,
                steps: 1,
                providers: 0,
                push_providers: 0,
            },
        ];
        let report = measure(
            &policy,
            &Observed {
                projects: &projects,
                ..nothing()
            },
        );

        let clause = find(&report, "hooks.allowHost", "hooks");
        assert_eq!(clause.state, State::Holding);
        assert_eq!(
            clause.detail.as_deref(),
            Some("3 of 6 declared steps run on the host and will not")
        );
    }

    /// The distinction `attestable` exists for, stated as an assertion.
    ///
    /// `crate::verify` lets an `Unknown` through because it asks "can I work?".
    /// This does not, because it asks "can somebody sign their name to this?"
    /// — and a report that swallowed its own blind spots would be one people
    /// learn to wave through.
    #[test]
    fn an_unmeasured_clause_is_enough_to_stop_an_attestation() {
        let policy = policy(r#"{"schemaVersion":1,"market":{"autoUpdate":false}}"#);
        let report = measure(&policy, &nothing());

        assert_eq!(report.bypassed, 0, "nothing here is out of compliance");
        assert_eq!(report.unmeasured, 1);
        assert!(
            !report.attestable,
            "a question this app declined to ask must not read as an answer"
        );
    }
}
