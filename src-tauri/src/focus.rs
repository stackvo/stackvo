//! Run what this project needs, and stop the rest.
//!
//! The most-wanted verb on a laptop, and the one thing this app already knew
//! enough to do without being told: `stackvo.json` **declares** what a project
//! needs around it — `manifest.services` — and the instance table says what is
//! installed. Between them, "which of these running containers is this project
//! not using" is arithmetic rather than a guess.
//!
//! **Measured August 2026, across seventeen products: none can ask the
//! question.** Herd and ServBay switch services on and off one at a time, and
//! none of them has a concept of "what this project needs" because none has a
//! manifest to declare it in. That is not a feature they have not built; it is
//! a question their shape cannot pose — which is why the claim is worth dating
//! rather than hedging. A competitor that adds a project manifest could add
//! this the same week, and the date is what would make that visible instead of
//! turning this paragraph quietly false.
//!
//! ## Why a plan and an apply rather than one button
//!
//! Stopping five containers is reversible and still not something to do to
//! somebody without showing them the list first — a project that declares
//! nothing declares *nothing*, not "nothing needed", and on that project a
//! naive focus would stop the whole stack. The plan says so in words rather
//! than doing it and leaving the user to work out what happened.
//!
//! `preset`, `worktree` and `release` all split the same way, and `provider`
//! records why the apply re-makes the plan rather than accepting one: "the
//! screen that offered the button may be minutes old".
//!
//! ## What is never stopped
//!
//! Only **service instances**, and only ones that are running. Projects are not
//! touched — a focus that stopped the other projects would be a different verb
//! with a much larger blast radius, and the one asking for it can stop them by
//! name. Traefik is not an instance and so cannot be reached from here, which
//! is the right answer rather than a special case: stopping the router would
//! take the focused project down with everything else.

use std::collections::BTreeSet;

/// One instance, and why the plan reached the verdict it did.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    /// The instance id — `redis-7-2`, not `redis`.
    pub id: String,
    pub service: String,
    /// Whether it is running now. A stopped instance is never in `stop`:
    /// stopping something that is already stopped is noise in a list whose
    /// whole value is being short enough to read.
    pub running: bool,
    /// Why it is kept, for the ones that are. `None` on an instance to stop.
    ///
    /// A reason rather than a boolean because the two reasons are not the same
    /// sentence to a reader: "your project asks for this" and "something your
    /// project asks for needs this" are different, and the second is the one
    /// people are surprised by.
    pub reason: Option<Reason>,
}

/// Why an instance survives a focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    /// Named in the project's own `services` list.
    Declared,
    /// Not named, but something declared depends on it. Kept because stopping
    /// it would break the thing that was asked for, which is the failure mode
    /// this reason exists to prevent.
    Dependency,
}

/// What a focus would do, before it does anything.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// Instances that stay up, declared first.
    pub keep: Vec<Verdict>,
    /// Running instances that would be stopped.
    pub stop: Vec<Verdict>,
    /// True when the project's `services` list is empty.
    ///
    /// The caller has to be able to tell this apart from "everything is already
    /// focused". An empty list means nobody has written one — the manifest
    /// field's own documentation says so — and focusing on it would stop every
    /// service in the workspace on the strength of a field nobody filled in.
    /// So the plan is *computed* and reported, and refusing to apply it is the
    /// caller's decision to make with the user rather than this function's to
    /// make silently.
    pub declares_nothing: bool,
}

/// One installed instance, as much of it as this decision needs.
///
/// A local shape rather than `InstanceRow`: this takes the four fields the
/// arithmetic uses, which is what lets the whole of it be tested without an
/// engine, a package tree or a workspace on disk.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: String,
    pub service: String,
    pub running: bool,
    /// Service ids this instance needs in order to work — resolved by the
    /// caller from the package manifest's `depends_on`.
    pub depends_on: Vec<String>,
}

/// Decide what to keep and what to stop.
///
/// `declared` is the project's `services` list: service ids (`redis`), not
/// instance ids (`redis-7-2`). One declared service keeps **every** instance of
/// it, and that is deliberate: a workspace running MySQL 8.0 and 8.4 side by
/// side has two instances for one id, the manifest cannot say which, and
/// guessing wrong stops the database the project is actually pointed at.
/// Keeping both is the answer that cannot be wrong in the direction that costs
/// somebody their afternoon.
pub fn plan(declared: &[String], candidates: &[Candidate]) -> Plan {
    let declared: BTreeSet<&str> = declared.iter().map(String::as_str).collect();

    // The dependency closure, walked over service ids. Bounded by the candidate
    // count rather than by a recursion depth: a package manifest that declares
    // a cycle is somebody else's file and must not be able to hang this.
    let mut needed: BTreeSet<String> = declared.iter().map(|s| (*s).to_string()).collect();
    for _ in 0..candidates.len() {
        let before = needed.len();
        for candidate in candidates {
            if needed.contains(&candidate.service) {
                for dependency in &candidate.depends_on {
                    needed.insert(dependency.clone());
                }
            }
        }
        if needed.len() == before {
            break;
        }
    }

    let mut keep = Vec::new();
    let mut stop = Vec::new();

    for candidate in candidates {
        let reason = if declared.contains(candidate.service.as_str()) {
            Some(Reason::Declared)
        } else if needed.contains(&candidate.service) {
            Some(Reason::Dependency)
        } else {
            None
        };

        let verdict = Verdict {
            id: candidate.id.clone(),
            service: candidate.service.clone(),
            running: candidate.running,
            reason,
        };

        match reason {
            Some(_) => keep.push(verdict),
            // Only running instances are worth naming as work. A stopped one is
            // already in the state a focus would put it in.
            None if candidate.running => stop.push(verdict),
            None => {}
        }
    }

    // Declared before dependency, so the list reads as "what you asked for,
    // then what that dragged in" rather than in table order.
    keep.sort_by(|a, b| {
        (a.reason != Some(Reason::Declared))
            .cmp(&(b.reason != Some(Reason::Declared)))
            .then_with(|| a.id.cmp(&b.id))
    });
    stop.sort_by(|a, b| a.id.cmp(&b.id));

    Plan {
        keep,
        stop,
        declares_nothing: declared.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, service: &str, running: bool, depends_on: &[&str]) -> Candidate {
        Candidate {
            id: id.into(),
            service: service.into(),
            running,
            depends_on: depends_on.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    fn ids(list: &[Verdict]) -> Vec<&str> {
        list.iter().map(|v| v.id.as_str()).collect()
    }

    #[test]
    fn what_the_project_declares_is_kept_and_the_rest_of_what_is_running_is_stopped() {
        let plan = plan(
            &["mysql".into(), "redis".into()],
            &[
                candidate("mysql-8-0", "mysql", true, &[]),
                candidate("redis-7-2", "redis", true, &[]),
                candidate("elasticsearch-8", "elasticsearch", true, &[]),
                candidate("kafka-3", "kafka", true, &[]),
            ],
        );

        assert_eq!(ids(&plan.keep), ["mysql-8-0", "redis-7-2"]);
        assert_eq!(ids(&plan.stop), ["elasticsearch-8", "kafka-3"]);
        assert!(!plan.declares_nothing);
    }

    /// Stopping something that is already stopped is noise in a list whose only
    /// value is being short enough to read before pressing the button.
    #[test]
    fn an_instance_that_is_already_stopped_is_not_offered_as_work() {
        let plan = plan(
            &["mysql".into()],
            &[
                candidate("mysql-8-0", "mysql", true, &[]),
                candidate("kafka-3", "kafka", false, &[]),
            ],
        );

        assert!(plan.stop.is_empty(), "{:?}", plan.stop);
    }

    /// The failure this is built to prevent: focusing on a project stops the
    /// thing the project's own database needs, and the project comes back up
    /// broken in a way nothing connects to having pressed focus.
    #[test]
    fn something_a_declared_service_depends_on_is_kept_and_says_why() {
        let plan = plan(
            &["kafka".into()],
            &[
                candidate("kafka-3", "kafka", true, &["zookeeper"]),
                candidate("zookeeper-3-9", "zookeeper", true, &[]),
                candidate("redis-7-2", "redis", true, &[]),
            ],
        );

        assert_eq!(ids(&plan.keep), ["kafka-3", "zookeeper-3-9"]);
        assert_eq!(plan.keep[0].reason, Some(Reason::Declared));
        assert_eq!(plan.keep[1].reason, Some(Reason::Dependency));
        assert_eq!(ids(&plan.stop), ["redis-7-2"]);
    }

    /// A dependency of a dependency is still a dependency. One pass over the
    /// list would keep the middle of a chain and stop the end of it.
    #[test]
    fn the_dependency_walk_reaches_the_end_of_a_chain() {
        let plan = plan(
            &["a".into()],
            &[
                candidate("a-1", "a", true, &["b"]),
                candidate("b-1", "b", true, &["c"]),
                candidate("c-1", "c", true, &[]),
                candidate("d-1", "d", true, &[]),
            ],
        );

        assert_eq!(ids(&plan.keep), ["a-1", "b-1", "c-1"]);
        assert_eq!(ids(&plan.stop), ["d-1"]);
    }

    /// A package manifest declaring a cycle is somebody else's file, and it
    /// must not be able to hang the app that reads it.
    #[test]
    fn a_dependency_cycle_terminates_rather_than_spinning() {
        let plan = plan(
            &["a".into()],
            &[
                candidate("a-1", "a", true, &["b"]),
                candidate("b-1", "b", true, &["a"]),
                candidate("z-1", "z", true, &[]),
            ],
        );

        assert_eq!(ids(&plan.keep), ["a-1", "b-1"]);
        assert_eq!(ids(&plan.stop), ["z-1"]);
    }

    /// Two instances of one service and a manifest that cannot say which. Both
    /// are kept, because guessing wrong stops the database the project is
    /// pointed at and the cost is one extra container.
    #[test]
    fn every_instance_of_a_declared_service_is_kept_because_the_manifest_cannot_choose() {
        let plan = plan(
            &["mysql".into()],
            &[
                candidate("mysql-8-0", "mysql", true, &[]),
                candidate("mysql-8-4", "mysql", true, &[]),
            ],
        );

        assert_eq!(ids(&plan.keep), ["mysql-8-0", "mysql-8-4"]);
        assert!(plan.stop.is_empty());
    }

    /// An empty `services` list means nobody wrote one — the manifest field
    /// says so itself — and a focus that acted on it would stop the whole
    /// workspace on the strength of an unfilled field. The plan is computed and
    /// flagged rather than quietly returned as "stop everything".
    #[test]
    fn a_project_that_declares_nothing_is_flagged_rather_than_emptying_the_stack() {
        let plan = plan(
            &[],
            &[
                candidate("mysql-8-0", "mysql", true, &[]),
                candidate("redis-7-2", "redis", true, &[]),
            ],
        );

        assert!(plan.declares_nothing);
        assert!(plan.keep.is_empty());
        assert_eq!(plan.stop.len(), 2, "the arithmetic still has to be right");
    }

    /// Already focused is a real state and has to read as one: nothing to do,
    /// rather than an error or an empty screen.
    #[test]
    fn a_stack_already_focused_plans_no_work() {
        let plan = plan(
            &["mysql".into()],
            &[candidate("mysql-8-0", "mysql", true, &[])],
        );

        assert!(plan.stop.is_empty());
        assert_eq!(ids(&plan.keep), ["mysql-8-0"]);
    }
}
