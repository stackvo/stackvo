//! Telescope, Horizon and Pulse — the way in, and the precondition nobody says.
//!
//! ## What was already true, and why nobody used it
//!
//! All three of these ship a web dashboard, all three open in `local` without
//! authentication, and this application already serves the project on its own
//! domain over a certificate the browser trusts. So `https://shop.loc/horizon`
//! works today. Not one line anywhere on the project page says it exists, so
//! nobody clicks it.
//!
//! A link would have been the cheap half. It is not the useful half.
//!
//! ## Each of the three goes quietly empty for a reason this app can see
//!
//! | Dashboard | Why it stays empty, with nothing broken on screen |
//! | --- | --- |
//! | **Horizon** | The queue connection has to be `redis`. Its metrics graphs stay flat until `horizon:snapshot` runs every five minutes — a scheduled job, not a worker |
//! | **Telescope** | `telescope:install` and `migrate` have to have run; without a daily `telescope:prune`, `telescope_entries` grows until the disk notices |
//! | **Pulse** | Its storage wants MySQL, MariaDB or PostgreSQL and refuses SQLite; with Redis ingest it wants a Redis connection **separate from the queue's**; and `pulse:check` is a long process, so it is a worker ([`crate::worker::Kind::Pulse`]) rather than a schedule entry |
//!
//! Every one of those is a sentence somebody needs *before* they conclude the
//! dashboard is broken.
//!
//! ## The honest limit, said in the same breath as the observation
//!
//! **This application reads `.env` and `composer.lock`. It does not read
//! `config/*.php`, and a project that has run `config:cache` can make both of
//! them lie.**
//!
//! So nothing here is a verdict. Every [`Observation`] carries the key it read
//! and the value it found — *"`.env` says `QUEUE_CONNECTION=sync`, and Horizon
//! only works with `redis`"* — and the caveat about a compiled configuration
//! sits beside it rather than at the top of a pane, because a warning at the
//! top of a screen and a row at the bottom are two things a reader has to join
//! up themselves.
//!
//! A check that calls something broken without having measured it is the check
//! people learn to ignore. That lesson is `verify`'s, and this module is held
//! to it.
//!
//! ## What is deliberately not claimed
//!
//! * **Whether the migrations have run.** That is a question about a database
//!   this does not query. Telescope's precondition is stated as a precondition,
//!   not reported as a state.
//! * **Whether Redis Cluster is in use.** Horizon does not support it, and the
//!   `.env` key that would say so is not one this could read without guessing
//!   at what `config/database.php` does with it. It belongs in the help
//!   document, where a sentence is a sentence, rather than in a row that looks
//!   measured.

use serde::Serialize;
use std::path::Path;

/// One of the three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Board {
    Horizon,
    Telescope,
    Pulse,
}

impl Board {
    pub const ALL: [Board; 3] = [Board::Horizon, Board::Telescope, Board::Pulse];

    pub fn as_str(self) -> &'static str {
        match self {
            Board::Horizon => "horizon",
            Board::Telescope => "telescope",
            Board::Pulse => "pulse",
        }
    }

    /// The composer package that puts it in a project.
    pub fn package(self) -> &'static str {
        match self {
            Board::Horizon => "laravel/horizon",
            Board::Telescope => "laravel/telescope",
            Board::Pulse => "laravel/pulse",
        }
    }

    /// The path each one registers, which is also each one's own default and
    /// the only value this app could know without reading `config/*.php`.
    ///
    /// Said as the default rather than as the address: a project that moved its
    /// dashboard has moved it in a file this does not read, and the pane says
    /// as much beside the link.
    pub fn path(self) -> &'static str {
        match self {
            Board::Horizon => "/horizon",
            Board::Telescope => "/telescope",
            Board::Pulse => "/pulse",
        }
    }
}

/// One thing worth saying, with the evidence it was read from.
///
/// `id` is a stable key and the UI holds the sentence — the `preflight`
/// arrangement, kept because a sentence assembled in Rust is a sentence that
/// cannot be translated.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub id: &'static str,
    /// The `.env` key this was read from, so the row says where it looked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// What that key said. Absent means the key is not in the file at all,
    /// which is a different fact from an empty value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A scheduled command a dashboard needs, and whether the project has it.
///
/// Offered as a [`crate::cron::Job`] rather than as prose because the project
/// already has a table of exactly this shape, with a writer, a log and a last
/// run. Adding one goes through `scheduler_save` — the single writer for the
/// whole list — rather than through a second verb of its own.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Needed {
    pub id: &'static str,
    pub job: crate::cron::Job,
    /// Something in the project's schedule already runs this artisan command.
    ///
    /// Matched on the command rather than on the label, for the reason
    /// `boost.rs` matches a registration on what it runs: the label is the
    /// reader's, and renaming a job must not make this offer it again.
    pub scheduled: bool,
}

/// One dashboard, everything this app can say about it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardStatus {
    pub board: Board,
    /// The version `composer.lock` names, when it names one. `None` means the
    /// package is not installed and nothing below applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed: Option<String>,
    /// Where it answers, on the project's own domain. `None` before the project
    /// has a domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub observations: Vec<Observation>,
    pub needs: Vec<Needed>,
    /// The long processes this dashboard needs, and whether each is up.
    pub workers: Vec<WorkerNeed>,
}

/// A worker sidecar a dashboard depends on.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerNeed {
    pub kind: crate::worker::Kind,
    pub running: bool,
}

/// Scout: the service is on, and the index is empty.
///
/// ## The purest form of this module's pattern
///
/// `meilisearch` and `typesense` are catalogue services, so switching one on is
/// a click in the Market. What nothing says is the **next step**: an empty
/// Meilisearch returns *nothing* for every search, so the application looks
/// broken while **every container is green**.
///
/// ## Why this is a sentence and not a button
///
/// The commands are `scout:import "App\Models\Post"` and
/// `scout:sync-index-settings`. The first cannot be offered from a fixed
/// catalogue, because it takes a **model class name this application cannot
/// know** — and a button that filled in something it had guessed and ran it is
/// exactly what `quickcmd`'s catalogue refuses. So the right shape is the one
/// [`crate::hints`] carries: one sentence, said where somebody is already
/// looking, pointing at the mechanism that does exist — a project declares its
/// own commands in `stackvo.json`, and `quickcmd` has accepted those all along.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scout {
    /// The version `composer.lock` names for `laravel/scout`.
    pub installed: String,
    /// `SCOUT_DRIVER` as the project's own `.env` spells it.
    pub driver: String,
}

/// The three, as this project stands.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Whether a `.env` was found at all.
    ///
    /// Reported rather than implied: with no file, every observation below is
    /// absent because nothing was read, not because nothing is wrong — and the
    /// two must not look the same. The same distinction
    /// [`crate::deps::Report::locks`] draws.
    pub read_env: bool,
    pub boards: Vec<BoardStatus>,
    /// Present only when `laravel/scout` is installed **and** the driver is one
    /// of the two this application can install — see [`Scout`].
    ///
    /// Both halves, because either alone would be the wrong sentence: Scout on
    /// the `database` driver has no index to fill, and a Meilisearch running
    /// for something other than Scout is not this note's business.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scout: Option<Scout>,
}

// -------------------------------------------------------------- the reading

/// One value out of a project's `.env`, trimmed of quotes and lower-cased.
///
/// Lower-cased because every value this module compares is a driver name, and
/// `REDIS` and `redis` are the same answer. Not [`crate::config::Env::parse`],
/// for the reason `detect::env_pairs` gives: that one merges this application's
/// own defaults over the file, which would report values the project never set.
pub fn env_value(dir: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(dir.join(".env")).ok()?;
    value_in(&text, key)
}

/// The same, over text already in hand — which is what the tests drive.
///
/// The **last** assignment wins, because that is what every dotenv reader does
/// with a repeated key and reporting the first would name a value the
/// application is not using.
pub fn value_in(text: &str, key: &str) -> Option<String> {
    let mut found = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case(key) {
            found = Some(
                value
                    .trim()
                    .trim_matches(['"', '\''])
                    .to_ascii_lowercase()
                    .to_string(),
            );
        }
    }
    found
}

// ------------------------------------------------------------- the schedule

/// `horizon:snapshot`, every five minutes.
///
/// Laravel's own interval, from Horizon's own documentation, and the reason the
/// metrics page is blank in a fresh installation: nothing writes a snapshot
/// until something runs this.
fn snapshot_job() -> crate::cron::Job {
    crate::cron::Job {
        label: "Horizon snapshot".to_string(),
        cron: "*/5 * * * *".to_string(),
        exec: vec!["php".into(), "artisan".into(), "horizon:snapshot".into()],
        enabled: true,
    }
}

/// `telescope:prune`, daily.
///
/// Telescope records every request, query, job and exception. Without this the
/// table grows for as long as the project is open, and the symptom is a full
/// disk rather than a slow dashboard.
fn prune_job() -> crate::cron::Job {
    crate::cron::Job {
        label: "Telescope prune".to_string(),
        cron: "0 3 * * *".to_string(),
        exec: vec![
            "php".into(),
            "artisan".into(),
            "telescope:prune".into(),
            "--hours=48".into(),
        ],
        enabled: true,
    }
}

/// Does this project's schedule already run this artisan command?
fn already_scheduled(schedule: &[crate::cron::Job], verb: &str) -> bool {
    schedule
        .iter()
        .any(|job| job.exec.iter().any(|a| a == verb))
}

// ---------------------------------------------------------------- the report

/// Assemble it.
///
/// `deps` is the set [`crate::deps`] has already parsed, so this and the
/// dependency card cannot disagree about what is installed. `workers` is what
/// the engine says is up. Everything else is two of the project's own files.
pub fn report(
    dir: &Path,
    domain: Option<&str>,
    deps: &[crate::deps::Dep],
    schedule: &[crate::cron::Job],
    running: &[crate::worker::Kind],
) -> Report {
    let env = std::fs::read_to_string(dir.join(".env")).ok();
    let value = |key: &str| env.as_deref().and_then(|text| value_in(text, key));
    let installed = |package: &str| {
        deps.iter()
            .find(|d| d.ecosystem == crate::deps::Ecosystem::Packagist && d.name == package)
            .map(|d| d.version.clone())
    };

    let boards = Board::ALL
        .iter()
        .map(|board| {
            let installed = installed(board.package());
            let mut observations = Vec::new();
            let mut needs = Vec::new();
            let mut workers = Vec::new();

            // Nothing below means anything for a package the project does not
            // have. Reported as a row all the same, so that "you could install
            // this" and "this is installed and misconfigured" are different
            // screens rather than the same absence.
            if installed.is_some() {
                match board {
                    Board::Horizon => {
                        let queue = value("QUEUE_CONNECTION");
                        if queue.as_deref() != Some("redis") {
                            observations.push(Observation {
                                id: "queueNotRedis",
                                key: Some("QUEUE_CONNECTION".to_string()),
                                value: queue,
                            });
                        }
                        needs.push(Needed {
                            id: "horizonSnapshot",
                            scheduled: already_scheduled(schedule, "horizon:snapshot"),
                            job: snapshot_job(),
                        });
                        workers.push(WorkerNeed {
                            kind: crate::worker::Kind::Horizon,
                            running: running.contains(&crate::worker::Kind::Horizon),
                        });
                    }

                    Board::Telescope => {
                        needs.push(Needed {
                            id: "telescopePrune",
                            scheduled: already_scheduled(schedule, "telescope:prune"),
                            job: prune_job(),
                        });
                    }

                    Board::Pulse => {
                        // Pulse's own connection first, and the application's
                        // only when Pulse does not name one — which is the
                        // order Pulse itself resolves them in.
                        let (key, storage) = match value("PULSE_DB_CONNECTION") {
                            Some(v) => ("PULSE_DB_CONNECTION", Some(v)),
                            None => ("DB_CONNECTION", value("DB_CONNECTION")),
                        };
                        if storage.as_deref() == Some("sqlite") {
                            observations.push(Observation {
                                id: "storageSqlite",
                                key: Some(key.to_string()),
                                value: storage,
                            });
                        }

                        let ingest = value("PULSE_INGEST");
                        if ingest.as_deref() == Some("redis") {
                            if value("PULSE_REDIS_CONNECTION").is_none() {
                                observations.push(Observation {
                                    id: "ingestSharesTheQueuesRedis",
                                    key: Some("PULSE_REDIS_CONNECTION".to_string()),
                                    value: None,
                                });
                            }
                            workers.push(WorkerNeed {
                                kind: crate::worker::Kind::PulseWork,
                                running: running.contains(&crate::worker::Kind::PulseWork),
                            });
                        }

                        workers.push(WorkerNeed {
                            kind: crate::worker::Kind::Pulse,
                            running: running.contains(&crate::worker::Kind::Pulse),
                        });
                    }
                }
            }

            BoardStatus {
                board: *board,
                url: installed
                    .as_ref()
                    .and(domain)
                    .map(|domain| format!("https://{domain}{}", board.path())),
                installed,
                observations,
                needs,
                workers,
            }
        })
        .collect();

    Report {
        read_env: env.is_some(),
        boards,
        scout: installed("laravel/scout").and_then(|version| {
            value("SCOUT_DRIVER")
                .filter(|driver| driver == "meilisearch" || driver == "typesense")
                .map(|driver| Scout {
                    installed: version,
                    driver,
                })
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker::Kind;

    fn dep(name: &str) -> crate::deps::Dep {
        crate::deps::Dep {
            ecosystem: crate::deps::Ecosystem::Packagist,
            name: name.to_string(),
            version: "1.0.0".to_string(),
            direct: true,
            source: None,
            hashed: true,
        }
    }

    fn dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-dashboards-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn board(report: &Report, which: Board) -> &BoardStatus {
        report
            .boards
            .iter()
            .find(|b| b.board == which)
            .expect("every board is reported")
    }

    /// The last assignment wins, a comment is not a value, and a key that is
    /// not in the file is `None` rather than an empty string.
    #[test]
    fn a_repeated_key_reads_as_the_value_the_application_would_use() {
        let text = "# QUEUE_CONNECTION=redis\nQUEUE_CONNECTION=sync\nQUEUE_CONNECTION=\"redis\"\n";
        assert_eq!(value_in(text, "QUEUE_CONNECTION").as_deref(), Some("redis"));
        assert_eq!(value_in(text, "queue_connection").as_deref(), Some("redis"));
        assert_eq!(value_in(text, "DB_CONNECTION"), None);
        assert_eq!(
            value_in("DB_CONNECTION=\n", "DB_CONNECTION").as_deref(),
            Some("")
        );
    }

    /// A package that is not installed produces a row with nothing attached —
    /// no URL, no observation, no schedule entry.
    #[test]
    fn nothing_is_said_about_a_dashboard_the_project_does_not_have() {
        let dir = dir();
        let found = report(&dir, Some("shop.loc"), &[], &[], &[]);

        assert!(!found.read_env, "there is no .env in this fixture");
        for status in &found.boards {
            assert!(status.installed.is_none());
            assert!(status.url.is_none());
            assert!(status.observations.is_empty());
            assert!(status.needs.is_empty());
            assert!(status.workers.is_empty());
        }
    }

    /// Horizon's two: the connection it needs, and the snapshot its graphs
    /// need.
    #[test]
    fn horizon_reports_the_queue_connection_and_asks_for_the_snapshot() {
        let dir = dir();
        std::fs::write(dir.join(".env"), "QUEUE_CONNECTION=sync\n").unwrap();

        let found = report(
            &dir,
            Some("shop.loc"),
            &[dep("laravel/horizon")],
            &[],
            &[Kind::Horizon],
        );
        let horizon = board(&found, Board::Horizon);

        assert!(found.read_env);
        assert_eq!(horizon.url.as_deref(), Some("https://shop.loc/horizon"));
        assert_eq!(horizon.observations.len(), 1);
        assert_eq!(horizon.observations[0].id, "queueNotRedis");
        // The value is carried, not summarised: the row says what the file
        // says, which is what makes it an observation rather than a verdict.
        assert_eq!(horizon.observations[0].value.as_deref(), Some("sync"));
        assert_eq!(horizon.needs.len(), 1);
        assert!(!horizon.needs[0].scheduled);
        assert_eq!(horizon.workers[0].kind, Kind::Horizon);
        assert!(horizon.workers[0].running);

        // With redis there is nothing to observe, and the snapshot is still
        // needed — it is not a symptom of a wrong connection.
        std::fs::write(dir.join(".env"), "QUEUE_CONNECTION=redis\n").unwrap();
        let found = report(&dir, None, &[dep("laravel/horizon")], &[], &[]);
        let horizon = board(&found, Board::Horizon);
        assert!(horizon.observations.is_empty());
        assert_eq!(horizon.needs.len(), 1);
        // No domain, no link — rather than a link to nowhere.
        assert!(horizon.url.is_none());
    }

    /// A job already in the schedule is not offered again, and it is matched on
    /// what it runs rather than on what somebody called it.
    #[test]
    fn a_scheduled_command_is_not_offered_twice_under_any_label() {
        let dir = dir();
        let renamed = crate::cron::Job {
            label: "Nightly tidy".to_string(),
            cron: "0 4 * * *".to_string(),
            exec: vec!["php".into(), "artisan".into(), "telescope:prune".into()],
            enabled: true,
        };

        let found = report(
            &dir,
            Some("shop.loc"),
            &[dep("laravel/telescope")],
            &[renamed],
            &[],
        );
        let telescope = board(&found, Board::Telescope);
        assert_eq!(telescope.needs.len(), 1);
        assert!(telescope.needs[0].scheduled);
    }

    /// Pulse's storage refuses SQLite, its own connection is asked for first,
    /// and Redis ingest adds a second worker.
    #[test]
    fn pulse_reads_its_own_connection_before_the_applications() {
        let dir = dir();
        std::fs::write(dir.join(".env"), "DB_CONNECTION=sqlite\n").unwrap();

        let found = report(&dir, Some("shop.loc"), &[dep("laravel/pulse")], &[], &[]);
        let pulse = board(&found, Board::Pulse);
        assert_eq!(pulse.observations.len(), 1);
        assert_eq!(pulse.observations[0].id, "storageSqlite");
        assert_eq!(pulse.observations[0].key.as_deref(), Some("DB_CONNECTION"));
        // Without Redis ingest there is one worker, and it is the recorder.
        assert_eq!(pulse.workers.len(), 1);
        assert_eq!(pulse.workers[0].kind, Kind::Pulse);

        // Pulse's own connection wins where it is set, so a project that keeps
        // Pulse on MySQL while the application is on SQLite is not reported.
        std::fs::write(
            dir.join(".env"),
            "DB_CONNECTION=sqlite\nPULSE_DB_CONNECTION=mysql\nPULSE_INGEST=redis\n",
        )
        .unwrap();
        let found = report(&dir, Some("shop.loc"), &[dep("laravel/pulse")], &[], &[]);
        let pulse = board(&found, Board::Pulse);
        assert_eq!(
            pulse.observations.iter().map(|o| o.id).collect::<Vec<_>>(),
            ["ingestSharesTheQueuesRedis"]
        );
        assert_eq!(
            pulse.workers.iter().map(|w| w.kind).collect::<Vec<_>>(),
            [Kind::PulseWork, Kind::Pulse]
        );
    }

    /// The offered jobs are ones `cron.rs` will actually accept.
    #[test]
    fn the_offered_jobs_pass_the_schedules_own_validation() {
        for job in [snapshot_job(), prune_job()] {
            crate::cron::validate(std::slice::from_ref(&job))
                .unwrap_or_else(|e| panic!("{}: {e}", job.label));
        }
    }

    /// Both halves have to be true, and either alone is the wrong sentence.
    #[test]
    fn the_scout_note_needs_the_package_and_a_driver_this_app_installs() {
        let dir = dir();

        // The package with no driver named: Scout's own default is `database`,
        // which has no index to fill.
        std::fs::write(dir.join(".env"), "APP_ENV=local\n").unwrap();
        assert!(report(&dir, None, &[dep("laravel/scout")], &[], &[])
            .scout
            .is_none());

        // The driver without the package: a Meilisearch running for something
        // else is not this note's business.
        std::fs::write(dir.join(".env"), "SCOUT_DRIVER=meilisearch\n").unwrap();
        assert!(report(&dir, None, &[], &[], &[]).scout.is_none());

        // `database` is a driver and not one of the two.
        std::fs::write(dir.join(".env"), "SCOUT_DRIVER=database\n").unwrap();
        assert!(report(&dir, None, &[dep("laravel/scout")], &[], &[])
            .scout
            .is_none());

        for driver in ["meilisearch", "typesense"] {
            std::fs::write(dir.join(".env"), format!("SCOUT_DRIVER={driver}\n")).unwrap();
            let scout = report(&dir, None, &[dep("laravel/scout")], &[], &[])
                .scout
                .unwrap_or_else(|| panic!("{driver}"));
            assert_eq!(scout.driver, driver);
            assert_eq!(scout.installed, "1.0.0");
        }
    }
}
