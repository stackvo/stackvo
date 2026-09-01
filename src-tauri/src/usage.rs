//! What each project actually cost, in CPU and in memory held.
//!
//! ## The measurement that turns a weakness into a position
//!
//! Docker is expensive, and this app's own README says so in a table rather
//! than arguing with it. Every rival built on containers has the same cost and
//! none of them measures it — DDEV, Lando and Laradock all leave "why is my
//! laptop hot" to Activity Monitor, where the answer is nine processes called
//! `com.docker.backend`. Being the one product that can say *"`shop` has held
//! 4.2 GB·hours and used 38 minutes of CPU today"* is a better place to stand
//! than denying the cost.
//!
//! ## No new measurement, and that is the whole reason this is small
//!
//! `sample_container_stats` has been reading CPU and memory once a minute
//! since the port, for the sparkline. It threw the numbers away after two
//! hours, because a sparkline is all it was for. This accumulates the same
//! samples instead of discarding them.
//!
//! ## Time is the part that is easy to get wrong
//!
//! A total is a rate multiplied by an interval, and the interval is the thing
//! nobody has. The sampler runs on a sixty-second timer, so sixty is the
//! tempting constant — and it is wrong on every machine that has ever been
//! shut, because a laptop closed on Friday and opened on Monday would bill
//! whatever was running for the weekend at its Friday rate.
//!
//! So the interval is measured — the gap since *that container's* last sample —
//! and a gap longer than [`MAX_GAP`] contributes **nothing**. The sample still
//! counts, and the clock still moves forward; only the time is refused. That
//! makes the worst case "today's total is a few minutes short" rather than "the
//! weekend is on the bill", and [`crate::stats_store`] settled the same
//! question the same way for the same reason.
//!
//! ## Shared services are not divided between projects
//!
//! `shop` and `blog` both use `mysql-8-4`, and any split of its memory between
//! them would be invented. It gets its own row and says what it is. A number
//! somebody could act on has to be one they can check.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The shape written today. Stamped so a later change has somewhere to branch.
const SCHEMA_VERSION: u64 = 1;

/// How many days are kept.
///
/// Thirty. Long enough to answer "was last Tuesday unusual", short enough that
/// the file stays a few tens of kilobytes on a workspace with twenty
/// containers — this is a record of a laptop, not a metrics system, and one
/// that grew without bound would be a second thing to have to clean up.
pub const RETENTION_DAYS: usize = 30;

/// The longest gap that is still billed, in seconds.
///
/// Five minutes: comfortably more than the sixty-second interval and its
/// jitter, and far less than any real interruption. A machine that slept, an
/// app that was quit, an engine that was down — all of them land past this and
/// contribute nothing rather than a fabricated stretch of time.
pub const MAX_GAP: u64 = 5 * 60;

/// What one container has used, since midnight UTC.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Seconds of one CPU. Directly comparable to what `time` reports, which
    /// is the number a developer already has a feel for.
    pub cpu_seconds: f64,
    /// Gigabyte-seconds of resident memory. Held rather than consumed —
    /// memory is not spent, it is occupied, and the unit says so.
    pub gb_seconds: f64,
    /// How many readings went into the two above. A total from four samples
    /// and a total from four hundred are different claims.
    pub samples: u64,
}

impl Usage {
    pub fn gb_hours(&self) -> f64 {
        self.gb_seconds / 3600.0
    }

    pub fn cpu_minutes(&self) -> f64 {
        self.cpu_seconds / 60.0
    }
}

/// One day, by container name.
pub type Day = BTreeMap<String, Usage>;

/// The whole record.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ledger {
    #[serde(default)]
    pub schema_version: u64,
    /// `YYYY-MM-DD` (UTC) -> that day's totals. A `BTreeMap` so the keys sort
    /// chronologically as text, which is what makes pruning the oldest a
    /// `pop_first` rather than a date comparison.
    #[serde(default)]
    pub days: BTreeMap<String, Day>,
    /// When each container was last sampled, so the next interval can be
    /// measured rather than assumed. Kept outside the days, because the gap
    /// that matters most is the one across midnight.
    #[serde(default)]
    pub last: BTreeMap<String, u64>,
}

/// `YYYY-MM-DD`, UTC, out of the same formatter everything else here uses.
pub fn day_of(unix_seconds: i64) -> String {
    crate::audit::rfc3339_of(unix_seconds)[..10].to_string()
}

impl Ledger {
    /// Add one reading.
    ///
    /// `cpu_percent` is Docker's own figure, which is percent of *one* core and
    /// so goes past 100 on a container using several — dividing by a hundred
    /// gives CPU-seconds per second, which is what makes the total comparable
    /// to `time`.
    pub fn record(&mut self, container: &str, now: u64, cpu_percent: f64, memory_bytes: u64) {
        let previous = self.last.insert(container.to_string(), now);

        let day = self.days.entry(day_of(now as i64)).or_default();
        let entry = day.entry(container.to_string()).or_default();
        entry.samples += 1;

        // The first reading of a container has no interval behind it, and one
        // after a gap has an interval nobody can vouch for. Both move the clock
        // forward and bill nothing.
        let Some(previous) = previous else { return };
        let gap = now.saturating_sub(previous);
        if gap == 0 || gap > MAX_GAP {
            return;
        }

        let seconds = gap as f64;
        entry.cpu_seconds += (cpu_percent / 100.0) * seconds;
        entry.gb_seconds += (memory_bytes as f64 / 1_000_000_000.0) * seconds;
    }

    /// Drop the days past [`RETENTION_DAYS`], and any container that has not
    /// been seen inside them.
    pub fn prune(&mut self) {
        while self.days.len() > RETENTION_DAYS {
            self.days.pop_first();
        }

        // A container deleted a month ago would otherwise keep its `last`
        // entry for ever — the same unbounded growth `sample_container_stats`
        // already fixed for the sample history.
        let seen: std::collections::BTreeSet<&String> =
            self.days.values().flat_map(|day| day.keys()).collect();
        let live: Vec<String> = self
            .last
            .keys()
            .filter(|k| seen.contains(k))
            .cloned()
            .collect();
        self.last.retain(|name, _| live.contains(name));
    }

    pub fn day(&self, date: &str) -> Day {
        self.days.get(date).cloned().unwrap_or_default()
    }
}

/// Where the record lives. `None` when the OS config directory is unknown.
pub fn path() -> Option<PathBuf> {
    crate::appdir::config().map(|dir| dir.join("usage.json"))
}

/// Read it back. A missing or unreadable file is an empty record.
///
/// The same stance [`crate::stats_store`] takes, and for a weaker reason that
/// makes it easier: nothing here may keep the app from starting, and the cost
/// of starting over is a chart that begins today.
pub fn load_from(path: &Path) -> Ledger {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ledger::default();
    };
    let mut ledger: Ledger = match serde_json::from_str(&text) {
        Ok(ledger) => ledger,
        Err(e) => {
            tracing::warn!(error = %e, "the usage record could not be read; starting a new one");
            return Ledger::default();
        }
    };
    if ledger.schema_version > SCHEMA_VERSION {
        tracing::warn!(
            found = ledger.schema_version,
            understood = SCHEMA_VERSION,
            "the usage record was written by a newer build; starting a new one"
        );
        return Ledger::default();
    }
    ledger.prune();
    ledger
}

pub fn save_to(path: &Path, ledger: &Ledger) -> crate::error::Result<()> {
    let stored = Ledger {
        schema_version: SCHEMA_VERSION,
        days: ledger.days.clone(),
        last: ledger.last.clone(),
    };
    crate::atomic::write(path, &serde_json::to_string(&stored)?)
}

// ------------------------------------------------------------------ the report

/// What a container is, for the purpose of reading the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Project,
    Service,
    /// The stack's own — the router, the mail catcher. Named rather than
    /// hidden: they are part of what Docker costs on this machine, and a total
    /// that quietly left them out would understate the answer.
    Stack,
}

/// One line of the answer.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub name: String,
    pub kind: Kind,
    pub cpu_seconds: f64,
    pub gb_hours: f64,
    pub samples: u64,
    /// The budget set for this project, in CPU minutes. Absent when none is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_cpu_minutes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_gb_hours: Option<f64>,
    /// True when either budget is set and has been passed.
    pub over_budget: bool,
}

/// A day, answered.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub date: String,
    pub rows: Vec<Row>,
    pub cpu_seconds: f64,
    pub gb_hours: f64,
}

/// A budget, as the preferences file holds it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Budget {
    pub cpu_minutes: Option<f64>,
    pub gb_hours: Option<f64>,
}

impl Budget {
    /// Only a budget that was set can be passed. A budget of zero is treated as
    /// no budget rather than as "always over": a cleared number field arrives
    /// as zero, and an alert that fires the moment it is cleared is one nobody
    /// will leave switched on.
    fn passed(&self, cpu_seconds: f64, gb_hours: f64) -> bool {
        self.cpu_minutes
            .is_some_and(|b| b > 0.0 && cpu_seconds / 60.0 > b)
            || self.gb_hours.is_some_and(|b| b > 0.0 && gb_hours > b)
    }
}

/// Turn one day into rows somebody can read.
///
/// `projects` and `services` are passed in rather than looked up here, so this
/// stays a pure function over three sets — and so the classification comes from
/// the same lists the rest of the app uses rather than from a second guess at
/// what a container name means.
pub fn report(
    date: &str,
    day: &Day,
    projects: &std::collections::BTreeSet<String>,
    services: &std::collections::BTreeSet<String>,
    budgets: &BTreeMap<String, Budget>,
) -> Report {
    let mut rows: Vec<Row> = day
        .iter()
        .map(|(name, usage)| {
            let kind = if projects.contains(name) {
                Kind::Project
            } else if services.contains(name) {
                Kind::Service
            } else {
                Kind::Stack
            };
            let gb_hours = usage.gb_hours();
            // Only a project has a budget. A shared service is not any one
            // person's to be over on, which is the same reason its usage is not
            // divided between the projects that use it.
            let budget = (kind == Kind::Project)
                .then(|| budgets.get(name).copied())
                .flatten()
                .unwrap_or_default();

            Row {
                name: name.clone(),
                kind,
                cpu_seconds: usage.cpu_seconds,
                gb_hours,
                samples: usage.samples,
                budget_cpu_minutes: budget.cpu_minutes.filter(|b| *b > 0.0),
                budget_gb_hours: budget.gb_hours.filter(|b| *b > 0.0),
                over_budget: budget.passed(usage.cpu_seconds, gb_hours),
            }
        })
        .collect();

    // Heaviest first, by CPU. The question this answers is "what is using the
    // machine", and the answer belongs at the top.
    rows.sort_by(|a, b| {
        b.cpu_seconds
            .partial_cmp(&a.cpu_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });

    Report {
        date: date.to_string(),
        cpu_seconds: rows.iter().map(|r| r.cpu_seconds).sum(),
        gb_hours: rows.iter().map(|r| r.gb_hours).sum(),
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ledger() -> Ledger {
        Ledger::default()
    }

    /// A total is a rate times an interval, and the interval is measured.
    #[test]
    fn a_minute_at_fifty_percent_is_thirty_cpu_seconds() {
        let mut l = ledger();
        // 2026-08-19T09:00:00Z and a minute later.
        l.record("shop", 1_787_130_000, 50.0, 1_000_000_000);
        l.record("shop", 1_787_130_060, 50.0, 1_000_000_000);

        let day = l.day(&day_of(1_787_130_000));
        let usage = day.get("shop").expect("a row for shop");

        assert!((usage.cpu_seconds - 30.0).abs() < 1e-9, "{usage:?}");
        // One gigabyte held for a minute.
        assert!((usage.gb_seconds - 60.0).abs() < 1e-9, "{usage:?}");
        assert!((usage.gb_hours() - (1.0 / 60.0)).abs() < 1e-9);
        assert_eq!(usage.samples, 2, "both readings are counted");
    }

    /// The first reading of a container has no interval behind it.
    #[test]
    fn one_reading_bills_nothing_and_still_counts() {
        let mut l = ledger();
        l.record("shop", 1_787_130_000, 100.0, 4_000_000_000);

        let usage = l.day(&day_of(1_787_130_000))["shop"];
        assert_eq!(usage.cpu_seconds, 0.0);
        assert_eq!(usage.gb_seconds, 0.0);
        assert_eq!(usage.samples, 1);
    }

    /// The rule the whole module turns on: a laptop that slept is not billed
    /// for the night.
    #[test]
    fn a_gap_longer_than_the_cap_contributes_nothing() {
        let mut l = ledger();
        let friday: i64 = 1_787_130_000;
        l.record("shop", friday as u64, 100.0, 8_000_000_000);
        // Three days later, still at 100%.
        l.record("shop", (friday + 3 * 86_400) as u64, 100.0, 8_000_000_000);

        let monday = l.day(&day_of(friday + 3 * 86_400));
        assert_eq!(
            monday["shop"].cpu_seconds, 0.0,
            "the weekend was put on the bill"
        );
        assert_eq!(monday["shop"].samples, 1);

        // And the clock moved: the next ordinary sample bills its own minute.
        l.record(
            "shop",
            (friday + 3 * 86_400 + 60) as u64,
            100.0,
            8_000_000_000,
        );
        let monday = l.day(&day_of(friday + 3 * 86_400));
        assert!((monday["shop"].cpu_seconds - 60.0).abs() < 1e-9);
    }

    #[test]
    fn each_day_is_counted_on_its_own() {
        let mut l = ledger();
        // 2026-08-19T23:59:30Z, then midnight, then a minute past it.
        let before_midnight: i64 = 1_787_183_970;
        l.record("shop", before_midnight as u64, 100.0, 1_000_000_000);
        l.record("shop", (before_midnight + 30) as u64, 100.0, 1_000_000_000);
        l.record("shop", (before_midnight + 90) as u64, 100.0, 1_000_000_000);

        let first = day_of(before_midnight);
        let second = day_of(before_midnight + 90);
        assert_ne!(first, second, "the fixture must straddle midnight");

        // The interval that crossed midnight is billed to the day it ended in,
        // which is the day the sample belongs to. Splitting it would be more
        // precise and would need a rule for a sample that spans a whole day.
        assert_eq!(l.day(&first)["shop"].cpu_seconds, 0.0);
        assert!((l.day(&second)["shop"].cpu_seconds - 90.0).abs() < 1e-9);
    }

    #[test]
    fn the_record_keeps_a_month_and_forgets_what_left_with_it() {
        let mut l = ledger();
        for day in 0..RETENTION_DAYS + 5 {
            l.record("shop", 1_700_000_000 + (day as u64) * 86_400, 10.0, 1_000);
        }
        l.record("gone", 1_700_000_000, 10.0, 1_000);
        l.prune();

        assert_eq!(l.days.len(), RETENTION_DAYS);
        assert!(
            !l.last.contains_key("gone"),
            "a container whose days all aged out kept its clock entry"
        );
        assert!(l.last.contains_key("shop"));
    }

    fn set(names: &[&str]) -> std::collections::BTreeSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn the_report_names_what_each_row_is_and_puts_the_heaviest_first() {
        let day: Day = [
            (
                "shop".to_string(),
                Usage {
                    cpu_seconds: 10.0,
                    gb_seconds: 3600.0,
                    samples: 5,
                },
            ),
            (
                "mysql-8-4".to_string(),
                Usage {
                    cpu_seconds: 60.0,
                    gb_seconds: 7200.0,
                    samples: 5,
                },
            ),
            (
                "traefik".to_string(),
                Usage {
                    cpu_seconds: 1.0,
                    gb_seconds: 60.0,
                    samples: 5,
                },
            ),
        ]
        .into_iter()
        .collect();

        let out = report(
            "2026-08-30",
            &day,
            &set(&["shop"]),
            &set(&["mysql-8-4"]),
            &BTreeMap::new(),
        );

        assert_eq!(
            out.rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            vec!["mysql-8-4", "shop", "traefik"],
            "heaviest first"
        );
        assert_eq!(out.rows[0].kind, Kind::Service);
        assert_eq!(out.rows[1].kind, Kind::Project);
        // The stack's own containers are named, not hidden: they are part of
        // what Docker costs here.
        assert_eq!(out.rows[2].kind, Kind::Stack);
        // Both totals are every row, traefik included — the same claim the
        // `Kind::Stack` assertion above makes, carried through to the numbers.
        // Written as the fixture's own figures rather than as one constant, so
        // that a changed row is read as a changed row.
        assert!((out.cpu_seconds - (10.0 + 60.0 + 1.0)).abs() < 1e-9);
        assert!((out.gb_hours - (3600.0 + 7200.0 + 60.0) / 3600.0).abs() < 1e-9);
    }

    #[test]
    fn a_budget_is_only_passed_when_one_was_set() {
        let day: Day = [(
            "shop".to_string(),
            Usage {
                cpu_seconds: 2400.0,
                gb_seconds: 18_000.0,
                samples: 40,
            },
        )]
        .into_iter()
        .collect();
        let projects = set(&["shop"]);
        let none = std::collections::BTreeSet::new();

        let unset = report("2026-08-30", &day, &projects, &none, &BTreeMap::new());
        assert!(!unset.rows[0].over_budget, "no budget cannot be exceeded");

        // A cleared field arrives as zero, and an alert that fires the moment
        // somebody clears the box is one they will switch off for ever.
        let zeroed = BTreeMap::from([(
            "shop".to_string(),
            Budget {
                cpu_minutes: Some(0.0),
                gb_hours: Some(0.0),
            },
        )]);
        assert!(!report("2026-08-30", &day, &projects, &none, &zeroed).rows[0].over_budget);

        // Forty CPU minutes against a thirty-minute budget.
        let cpu = BTreeMap::from([(
            "shop".to_string(),
            Budget {
                cpu_minutes: Some(30.0),
                gb_hours: None,
            },
        )]);
        let out = report("2026-08-30", &day, &projects, &none, &cpu);
        assert!(out.rows[0].over_budget);
        assert_eq!(out.rows[0].budget_cpu_minutes, Some(30.0));
        assert_eq!(out.rows[0].budget_gb_hours, None);

        // Under it, and either budget on its own is enough to be over.
        let generous = BTreeMap::from([(
            "shop".to_string(),
            Budget {
                cpu_minutes: Some(60.0),
                gb_hours: Some(1.0),
            },
        )]);
        assert!(
            report("2026-08-30", &day, &projects, &none, &generous).rows[0].over_budget,
            "five gigabyte-hours against a one gigabyte-hour budget"
        );
    }

    /// A shared service is nobody's to be over on, which is the same reason its
    /// usage is not divided between the projects that use it.
    #[test]
    fn a_service_carries_no_budget_even_when_one_is_written_against_its_name() {
        let day: Day = [(
            "mysql-8-4".to_string(),
            Usage {
                cpu_seconds: 9_000.0,
                gb_seconds: 90_000.0,
                samples: 40,
            },
        )]
        .into_iter()
        .collect();

        let out = report(
            "2026-08-30",
            &day,
            &std::collections::BTreeSet::new(),
            &set(&["mysql-8-4"]),
            &BTreeMap::from([(
                "mysql-8-4".to_string(),
                Budget {
                    cpu_minutes: Some(1.0),
                    gb_hours: Some(1.0),
                },
            )]),
        );

        assert!(!out.rows[0].over_budget);
        assert_eq!(out.rows[0].budget_cpu_minutes, None);
    }

    #[test]
    fn a_record_survives_being_written_and_read_back() {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-usage-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("usage.json");

        let mut l = ledger();
        l.record("shop", 1_787_130_000, 50.0, 1_000_000_000);
        l.record("shop", 1_787_130_060, 50.0, 1_000_000_000);
        save_to(&file, &l).unwrap();

        let back = load_from(&file);
        assert_eq!(back.schema_version, SCHEMA_VERSION);
        assert!((back.day(&day_of(1_787_130_000))["shop"].cpu_seconds - 30.0).abs() < 1e-9);
        // The clock comes back too, or the first sample after a restart would
        // be billed nothing every time the app is opened.
        assert_eq!(back.last.get("shop"), Some(&1_787_130_060));

        // Damage is a fresh start, never a failure to open the app.
        std::fs::write(&file, "{ not json").unwrap();
        assert!(load_from(&file).days.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
