//! What can leave this machine, as far as Docker can actually say.
//!
//! ## The question, and whose it is
//!
//! An administrator who set `registryPrefix` to point every pull at the
//! organisation's mirror has a follow-up: **who bypassed it.** The same person
//! wants to know which of the containers on this laptop can reach the internet
//! at all. Neither question had an answer here, and neither has one anywhere
//! else in this category — a local binary has no containers, so it has no
//! network namespaces to separate one program's traffic from another's.
//!
//! ## Three answers, and the third is weaker than the other two
//!
//! **Can it reach outside?** Exact, and not a heuristic. A Docker network
//! created with `internal: true` gets no gateway installed, so a container
//! whose every network is internal *cannot route out* — that is a property of
//! the network, asked of the daemon, not an inference from behaviour.
//!
//! **Where did its image come from?** Exact. Every running container names the
//! reference it was created from, and the registry host is the first component
//! of that reference under Docker's own rule ([`crate::images::registry_of`]).
//! This is the mirror-bypass answer, and it is the one an administrator
//! actually asked for.
//!
//! **How much has it sent?** A number, and the caveat travels with it: Docker's
//! counters are per interface and count **all** traffic, including the chatter
//! between containers on the StackVo network. So `sent` is a floor on "did
//! anything leave this container", never a measure of internet traffic. It is
//! reported because a container that has sent nothing at all is a useful thing
//! to be sure of, and withheld from any sentence that would read as a
//! destination.
//!
//! ## What this deliberately does not do, and it is the interesting half
//!
//! **It does not say where anything connected to.** Docker keeps no connection
//! log. Answering *"which host did this container talk to"* needs either a
//! packet capture inside the container's network namespace or a proxy standing
//! in front of it, and both are things this application will not install
//! quietly on somebody's machine to fill in a report.
//!
//! That refusal is named on the screen rather than left as a gap, for the
//! reason [`crate::compliance`] gives about its own `unmeasured`: a report that
//! lets its blind spots read as silence is one people learn to trust further
//! than it deserves. The measurement that would close it is a decision — a
//! visible, consenting one — and it is not made here.

use serde::Serialize;
use std::collections::BTreeMap;

/// Whether a container can route off this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reach {
    /// At least one of its networks has a gateway. Docker will route.
    Outside,
    /// Every network it is on was created `internal`. Docker installs no
    /// gateway, so there is nowhere for a packet to go.
    Contained,
    /// It is on no network this could inspect — a daemon that would not answer,
    /// or `network_mode: none`. Distinct from `Contained` on purpose: "it
    /// cannot" and "I could not tell" are different claims and only one of them
    /// is evidence.
    Unknown,
}

/// One container, and what it can reach.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub container: String,
    /// The reference it was created from, verbatim.
    pub image: String,
    /// The registry host that reference names. `docker.io` when it names none,
    /// because that is where an unqualified reference is pulled from — writing
    /// "none" would omit the one host most worth naming.
    pub registry: String,
    /// Whether the policy's mirror is what produced `registry`.
    ///
    /// The row an administrator reads first. `false` on a managed machine means
    /// this container was created before the mirror arrived, or from a
    /// reference the mirror leaves alone.
    pub mirrored: bool,
    pub reach: Reach,
    pub networks: Vec<String>,
    /// Cumulative bytes out since the container started. See the module comment
    /// on why this is a floor rather than a measurement of egress.
    pub sent: u64,
    pub received: u64,
}

/// Every container, and the honest summary of what is not here.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub rows: Vec<Row>,
    /// The mirror in force, when there is one. Present so the `mirrored` column
    /// can be read as a comparison rather than as an opinion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_prefix: Option<String>,
    /// Distinct registry hosts across every row, sorted.
    ///
    /// The single most useful line for the person who set a mirror: on a
    /// machine where it holds, this is one entry long.
    pub registries: Vec<String>,
    /// How many containers cannot route out at all.
    pub contained: usize,
}

/// What the caller measured, one container at a time.
///
/// Handed in so this module reads nothing itself and every branch has a test —
/// the arrangement [`crate::compliance`] and [`crate::verify`] both use.
pub struct Observed<'a> {
    pub container: &'a str,
    pub image: &'a str,
    /// The networks it is attached to, and whether each was created
    /// `internal`. `None` for a network the daemon would not describe, which is
    /// what makes [`Reach::Unknown`] reachable.
    pub networks: &'a [(String, Option<bool>)],
    pub sent: u64,
    pub received: u64,
}

/// Docker's default, and the host an unqualified reference is pulled from.
pub const DEFAULT_REGISTRY: &str = "docker.io";

/// Hold every container against the policy in force.
pub fn measure(observed: &[Observed], registry_prefix: Option<&str>) -> Report {
    let mut rows: Vec<Row> = observed
        .iter()
        .map(|o| {
            let registry = crate::images::registry_of(o.image)
                .unwrap_or(DEFAULT_REGISTRY)
                .to_string();
            Row {
                container: o.container.to_string(),
                image: o.image.to_string(),
                // Compared against the prefix's own host rather than against
                // the whole prefix: a mirror written as
                // `registry.corp/proxy` puts `registry.corp` in front of the
                // reference, and a row that demanded the path too would report
                // every mirrored image as a bypass.
                mirrored: registry_prefix
                    .map(|prefix| registry == prefix.split('/').next().unwrap_or(prefix))
                    .unwrap_or(false),
                reach: reach(o.networks),
                networks: o.networks.iter().map(|(name, _)| name.clone()).collect(),
                sent: o.sent,
                received: o.received,
                registry,
            }
        })
        .collect();

    rows.sort_by(|a, b| a.container.cmp(&b.container));

    let mut registries: Vec<String> = rows.iter().map(|r| r.registry.clone()).collect();
    registries.sort();
    registries.dedup();

    Report {
        contained: rows.iter().filter(|r| r.reach == Reach::Contained).count(),
        registries,
        registry_prefix: registry_prefix.map(str::to_string),
        rows,
    }
}

/// One container's networks → whether Docker will route for it.
///
/// The asymmetry is deliberate and is the whole reliability of the column: a
/// single non-internal network is enough for [`Reach::Outside`], because one
/// gateway is all a packet needs. Nothing short of *every* network being known
/// internal earns [`Reach::Contained`] — a network this could not describe
/// leaves the answer [`Reach::Unknown`], never "contained", because a
/// containment claim that rests on a failed lookup is the one wrong answer this
/// report must not give.
fn reach(networks: &[(String, Option<bool>)]) -> Reach {
    if networks.is_empty() {
        return Reach::Unknown;
    }
    if networks
        .iter()
        .any(|(_, internal)| *internal == Some(false))
    {
        return Reach::Outside;
    }
    if networks.iter().all(|(_, internal)| *internal == Some(true)) {
        return Reach::Contained;
    }
    Reach::Unknown
}

/// Ask the daemon, once per network rather than once per container.
///
/// A workspace has three networks and thirty containers, so the naive shape is
/// thirty inspect calls for three answers. Cached in a map the caller owns, so
/// the cost is the number of distinct networks and nothing here has to be a
/// singleton.
pub async fn internal_flags(
    names: &[String],
    cache: &mut BTreeMap<String, Option<bool>>,
) -> Vec<(String, Option<bool>)> {
    let mut out = Vec::new();
    for name in names {
        if !cache.contains_key(name) {
            cache.insert(name.clone(), crate::engine::network_is_internal(name).await);
        }
        out.push((name.clone(), cache[name]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net(rows: &[(&str, Option<bool>)]) -> Vec<(String, Option<bool>)> {
        rows.iter().map(|(n, i)| ((*n).to_string(), *i)).collect()
    }

    /// The asymmetry that makes the column worth having.
    ///
    /// One gateway is all a packet needs, so a single routable network is
    /// enough for `Outside`. Nothing short of *every* network being known
    /// internal earns `Contained` — a containment claim resting on a lookup
    /// that failed is the one wrong answer this report must not give.
    #[test]
    fn containment_is_claimed_only_when_every_network_is_known_internal() {
        assert_eq!(reach(&net(&[("stackvo", Some(false))])), Reach::Outside);
        assert_eq!(reach(&net(&[("walled", Some(true))])), Reach::Contained);

        // One way out is a way out.
        assert_eq!(
            reach(&net(&[("walled", Some(true)), ("stackvo", Some(false))])),
            Reach::Outside
        );

        // A network the daemon would not describe. Not "contained" — the
        // difference between "it cannot" and "I could not tell".
        assert_eq!(
            reach(&net(&[("walled", Some(true)), ("mystery", None)])),
            Reach::Unknown
        );
        assert_eq!(reach(&net(&[])), Reach::Unknown);
    }

    /// An unqualified reference is pulled from Docker Hub, and the report says
    /// so by name.
    ///
    /// Reporting "none" would omit the one host an administrator most wants to
    /// see in the list.
    #[test]
    fn a_reference_with_no_host_is_reported_as_docker_hub() {
        let observed = [
            Observed {
                container: "stackvo-shop",
                image: "mysql:8.0",
                networks: &[],
                sent: 0,
                received: 0,
            },
            Observed {
                container: "stackvo-blog",
                image: "registry.corp/proxy/mysql:8.0",
                networks: &[],
                sent: 0,
                received: 0,
            },
        ];

        // Rows are sorted by container name, so blog comes before shop.
        let report = measure(&observed, None);
        assert_eq!(report.rows[0].container, "stackvo-blog");
        assert_eq!(report.rows[0].registry, "registry.corp");
        assert_eq!(report.rows[1].registry, DEFAULT_REGISTRY);
        assert_eq!(report.registries, ["docker.io", "registry.corp"]);
    }

    /// The mirror-bypass answer, which is the whole reason an administrator
    /// opens this.
    ///
    /// Compared against the prefix's **host**, not the whole prefix: a mirror
    /// written `registry.corp/proxy` puts `registry.corp` in front of a
    /// reference, and a row that demanded the path back would report every
    /// correctly mirrored image as a bypass.
    #[test]
    fn a_container_that_did_not_come_through_the_mirror_is_the_row_that_stands_out() {
        let observed = [
            Observed {
                container: "a-mirrored",
                image: "registry.corp/proxy/redis:7.2",
                networks: &[],
                sent: 0,
                received: 0,
            },
            Observed {
                container: "b-bypassed",
                image: "redis:7.2",
                networks: &[],
                sent: 0,
                received: 0,
            },
        ];

        let report = measure(&observed, Some("registry.corp/proxy"));
        assert!(report.rows[0].mirrored);
        assert!(!report.rows[1].mirrored, "pulled straight from Docker Hub");
        assert_eq!(
            report.registry_prefix.as_deref(),
            Some("registry.corp/proxy")
        );

        // With no mirror in force nothing is "mirrored", and that is silence
        // rather than a finding: an unmanaged machine has no rule to bypass.
        let unmanaged = measure(&observed, None);
        assert!(unmanaged.rows.iter().all(|r| !r.mirrored));
    }

    /// The summary line the person who set a mirror actually reads.
    #[test]
    fn one_registry_across_the_whole_machine_is_the_answer_a_mirror_is_for() {
        let observed = [
            Observed {
                container: "a",
                image: "registry.corp/proxy/redis:7.2",
                networks: &net(&[("walled", Some(true))]),
                sent: 0,
                received: 0,
            },
            Observed {
                container: "b",
                image: "registry.corp/proxy/mysql:8.0",
                networks: &net(&[("walled", Some(true))]),
                sent: 0,
                received: 0,
            },
        ];

        let report = measure(&observed, Some("registry.corp/proxy"));
        assert_eq!(report.registries.len(), 1);
        assert_eq!(report.contained, 2);
    }
}
