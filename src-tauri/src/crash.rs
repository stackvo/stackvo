//! What a panic leaves behind.
//!
//! `[profile.release]` sets `panic = "abort"`, and nothing in this codebase
//! installed a panic hook. The two together produced the worst failure mode the
//! app has: a slice index in our code, or a bug in a dependency, kills the
//! process with **no trace at all**. The user says "it just disappeared", and
//! the rotating log in [`crate::logging`] ends on an ordinary info line, because
//! the panic never reached `tracing` and the process was gone before it could.
//!
//! ## Why a separate file, and not just a log line
//!
//! [`crate::logging`] writes through `tracing_appender::non_blocking`, which
//! hands lines to a background thread. That is the right trade for the millions
//! of ordinary lines and the wrong one for the last line ever written: `abort()`
//! does not unwind, does not run destructors and does not flush that thread, so
//! an `error!` emitted from inside a panic hook is *likely* to be lost — exactly
//! when it is the only thing worth keeping.
//!
//! So the report is written here with a plain, synchronous `fs::write` before
//! the hook returns. The `tracing` line is still emitted, because in a debug
//! build (where panics unwind and the guard is dropped normally) it does arrive,
//! and it is the line that connects the crash to whatever was being logged a
//! moment earlier.
//!
//! Rotation is the second reason. `logging` keeps seven daily files and drops
//! the eighth; a crash from last month is still the answer to "why does this
//! keep happening", so crash reports are pruned on their own count, not by age.
//!
//! ## Symbols
//!
//! A backtrace is only worth capturing if it can be read. `strip = true` in the
//! release profile removed the symbol table along with the debug info, which
//! would have made every frame here a bare address. The profile now strips
//! `"debuginfo"` only: file-and-line is gone, function names survive, and the
//! binary keeps almost all of the size win.

use std::path::{Path, PathBuf};

/// How many reports to keep.
///
/// A crash that happens once is a report; a crash that happens every launch is
/// a pattern, and seeing four of them side by side is what tells the two apart.
/// Small enough that the directory never becomes something to clean up.
const KEEP: usize = 10;

/// Install the panic hook. Call once, as early as possible.
///
/// Chains to the hook that was already installed rather than replacing it, so a
/// debug build keeps its stderr message and its `RUST_BACKTRACE` behaviour. Ours
/// runs first: the previous hook is the one that might abort the process.
pub fn install() {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        // Nothing in here may panic — a panic inside a panic hook aborts with
        // even less to show for it. Every step is best-effort and ignores its
        // own failure.
        let report = render(&Facts::from_hook(info));

        if let Some(path) = write(&report) {
            // Not `tracing`: in a release build the appender thread is about to
            // be aborted out from under us, and this line is what tells a
            // developer running from a terminal where to look.
            eprintln!("stackvo: crash report written to {}", path.display());
        }

        // Best effort, and genuinely useful in a debug build: it lands in the
        // same file as the operations that led up to the panic.
        tracing::error!(target: "stackvo_desktop", crash = %report, "panic");

        previous(info);
    }));
}

/// Everything a report is made of, lifted out of the hook's argument.
///
/// A `PanicHookInfo` can only be obtained inside a live panic hook, and
/// `set_hook` is process-wide — a test that installed one to capture a report
/// would race every other test in the binary. Extracting first makes [`render`]
/// an ordinary function over ordinary data, and leaves [`Facts::from_hook`] as
/// the only part that needs a real panic.
struct Facts {
    message: String,
    location: String,
    thread: String,
    seconds: i64,
    backtrace: String,
}

impl Facts {
    fn from_hook(info: &std::panic::PanicHookInfo<'_>) -> Self {
        // `panic!("{x}")` arrives as a `String`; `panic!("literal")` and the
        // panics the standard library raises arrive as `&str`. Anything else is
        // a payload from `panic_any`, which nothing here uses but a dependency
        // might.
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());

        Self {
            message,
            location: info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "<unknown>".to_string()),
            thread: std::thread::current()
                .name()
                .unwrap_or("<unnamed>")
                .to_string(),
            seconds: unix_seconds(),
            backtrace: std::backtrace::Backtrace::force_capture().to_string(),
        }
    }
}

/// The text of a crash report.
fn render(facts: &Facts) -> String {
    // The same masking the log uses. A panic message is usually ours, but
    // `expect("connecting to mysql://root:hunter2@…")` is the shape that makes a
    // crash report unattachable to an issue, and this file exists to be attached
    // to issues.
    let message = crate::logging::redact(&facts.message);

    format!(
        "StackVo Desktop crash report\n\
         \n\
         version   {version}\n\
         platform  {os} {arch}\n\
         time      {stamp} UTC (unix {seconds})\n\
         thread    {thread}\n\
         location  {location}\n\
         \n\
         message\n\
         {message}\n\
         \n\
         backtrace\n\
         {backtrace}\n",
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        stamp = stamp(facts.seconds),
        seconds = facts.seconds,
        thread = facts.thread,
        location = facts.location,
        backtrace = facts.backtrace,
    )
}

/// Write the report next to the log files and prune older ones.
///
/// Returns where it landed, or None when there is nowhere writable — the same
/// judgement [`crate::logging::init`] makes: no log is a reason to carry on
/// without one, not a reason to make the failure worse.
fn write(report: &str) -> Option<PathBuf> {
    let dir = crate::logging::dir()?;
    std::fs::create_dir_all(&dir).ok()?;

    // The pid disambiguates two processes crashing in the same second, which is
    // exactly what a single-instance app does when a launch loop starts.
    let path = dir.join(format!(
        "crash-{}-{}.txt",
        stamp(unix_seconds()),
        std::process::id()
    ));
    std::fs::write(&path, report).ok()?;

    prune(&dir);
    Some(path)
}

/// Keep the newest [`KEEP`] reports.
///
/// Sorted by name, which for this naming scheme is chronological order — no
/// `stat` call, and no dependence on mtimes that a backup restore rewrites.
fn prune(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut reports: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| is_report(p))
        .collect();

    if reports.len() <= KEEP {
        return;
    }
    reports.sort();
    for old in &reports[..reports.len() - KEEP] {
        let _ = std::fs::remove_file(old);
    }
}

/// Is this one of ours? Narrow on purpose: [`prune`] deletes what this matches,
/// and it runs in a directory that also holds the log files.
/// One report, as the window shows it.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// `crash-<UTC>-<pid>.txt` — the name is the timestamp, so this sorts.
    pub name: String,
    pub path: String,
    pub bytes: u64,
}

/// The reports on disk, newest first.
///
/// ## Why this exists, and why it is not "send a crash report"
///
/// The roadmap asked for sending one. `PRIVACY.md` is explicit that there is no
/// telemetry, no crash reporting service and no server behind the app, and that
/// anything future would be opt-in, off by default, and described there *before*
/// it ships. So there is nowhere to send to, and building one is a product
/// decision rather than a code change.
///
/// What was genuinely missing is smaller and worse: **the app crashes and never
/// says so.** A report is written here, it travels in a diagnostic bundle if
/// somebody happens to build one, and nothing ever tells them a crash happened.
/// So "I would like to report this" never starts — not because reporting is
/// hard, but because they do not know there is anything to report. That is what
/// this answers, and it stays entirely on their machine.
pub fn reports() -> Vec<Report> {
    let Some(dir) = crate::logging::dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut out: Vec<Report> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| is_report(p))
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            Some(Report {
                name,
                path: path.display().to_string(),
                bytes,
            })
        })
        .collect();

    // Newest first. The name carries the timestamp — see `stamp` — so this is
    // chronological without a `stat`, which is the same reason `prune` sorts
    // by name: an mtime is rewritten by a backup restore.
    out.sort_by(|a, b| b.name.cmp(&a.name));
    out
}

/// Where the "you have seen these" marker lives.
///
/// Beside the reports rather than in `preferences.json`: it is a fact about
/// this log directory, and a workspace moved to another machine should not
/// arrive claiming its crashes were already read.
fn seen_marker() -> Option<PathBuf> {
    crate::logging::dir().map(|d| d.join(".crashes-seen"))
}

/// The reports written since the last time somebody was told about them.
///
/// Compared by **name**, not by count: a report that was pruned away would make
/// a count go down and a newer crash then go unmentioned. The marker holds the
/// newest name that has been shown, and anything sorting above it is new.
pub fn unseen() -> Vec<Report> {
    let seen = seen_marker()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    reports()
        .into_iter()
        .take_while(|r| r.name.as_str() > seen.as_str())
        .collect()
}

/// Record that the newest report has been shown.
///
/// Best effort, and deliberately so: a marker that cannot be written means the
/// notice appears again next launch, which is a repeated line rather than a
/// lost crash. The other way round would be worse.
pub fn mark_seen() {
    let Some(path) = seen_marker() else {
        return;
    };
    let newest = reports()
        .first()
        .map(|r| r.name.clone())
        .unwrap_or_default();
    let _ = std::fs::write(path, newest);
}

fn is_report(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("crash-") && n.ends_with(".txt"))
}

fn unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `YYYYMMDD-HHMMSS`, UTC.
///
/// Written out rather than pulled from a formatting crate. `time` is already in
/// the tree but only with `parsing` — turning on `formatting` and `macros` for
/// one filename adds a proc-macro crate to every build, and `local-offset` is
/// refused in a multi-threaded process on Unix anyway, so it could not have
/// given local time here even if it were enabled. UTC, and the report says so.
pub fn stamp(unix_seconds: i64) -> String {
    let days = unix_seconds.div_euclid(86_400);
    let time_of_day = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);

    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60,
    )
}

/// Days since 1970-01-01 to a civil date. Howard Hinnant's `civil_from_days`,
/// which is the algorithm every date library uses; exact for any value a
/// `SystemTime` on this machine can produce.
pub(crate) fn civil_from_days(z: i64) -> (i64, u32, u32) {
    // Shift the epoch to 0000-03-01 so leap day lands at the end of the year and
    // the month arithmetic below needs no special case for February.
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097; // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153; // [0, 11], March = 0
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;

    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// A civil date back to days since 1970-01-01 — the inverse of
/// [`civil_from_days`], and the same author's `days_from_civil`.
///
/// Written because one caller has to read a timestamp this app wrote and answer
/// "how long is left": [`crate::worktree`] gives a sandbox an expiry, and
/// "expires in forty minutes" is the sentence somebody acts on, where "expires
/// at 14:32Z" is arithmetic they have to do. Comparing the strings answers
/// *whether* it has passed, which is why nothing needed this until now.
pub(crate) fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400; // [0, 399]
    let shifted_month = if month > 2 { month - 3 } else { month + 9 } as i64;
    let day_of_year = (153 * shifted_month + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pair, held against each other rather than against a table somebody
    /// typed: a wrong constant in one would have to be wrong in the other in
    /// exactly the same way to survive this.
    #[test]
    fn the_calendar_round_trips_across_every_awkward_boundary() {
        for days in [
            0,       // 1970-01-01
            19_782,  // a leap day
            11_016,  // the leap day the hundred-year rule would have skipped
            -25_509, // a year that is divisible by 100 and is not a leap year
            -25_508, 59, 60, 730, 25_000, 40_000,
        ] {
            let (year, month, day) = civil_from_days(days);
            assert_eq!(
                days_from_civil(year, month, day),
                days,
                "{year:04}-{month:02}-{day:02} does not come back as {days}"
            );
        }
    }

    #[test]
    fn the_epoch_and_a_known_instant_round_trip() {
        assert_eq!(stamp(0), "19700101-000000");
        // 2026-08-06T09:15:30Z.
        assert_eq!(stamp(1_786_007_730), "20260806-091530");
        assert_eq!(stamp(1_786_007_730 - 1), "20260806-091529");
    }

    /// The dates every hand-rolled calendar gets wrong.
    #[test]
    fn leap_days_and_century_rules_hold() {
        // 2024-02-29, an ordinary leap year.
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        // 2000-02-29 — divisible by 100 but also by 400, so it exists.
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        // 1900 was not a leap year: 1900-03-01 follows 1900-02-28.
        assert_eq!(civil_from_days(-25_509), (1900, 2, 28));
        assert_eq!(civil_from_days(-25_508), (1900, 3, 1));
    }

    /// Instants before 1970 are negative, and truncating division would round a
    /// negative quotient towards zero and land a day out.
    #[test]
    fn instants_before_the_epoch_do_not_slip_a_day() {
        assert_eq!(stamp(-1), "19691231-235959");
        assert_eq!(stamp(-86_400), "19691231-000000");
    }

    /// Ordering by filename is what [`prune`] relies on to decide what is old.
    #[test]
    fn stamps_sort_chronologically_as_text() {
        let mut stamps = [stamp(1_786_007_730), stamp(0), stamp(1_700_000_000)];
        stamps.sort();
        assert_eq!(
            stamps,
            [stamp(0), stamp(1_700_000_000), stamp(1_786_007_730)]
        );
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-crash-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// One report per day, named the way [`write`] names them.
    fn seed(dir: &Path, count: usize) -> Vec<String> {
        (0..count)
            .map(|i| {
                let name = format!("crash-{}-1.txt", stamp(1_700_000_000 + i as i64 * 86_400));
                std::fs::write(dir.join(&name), "x").unwrap();
                name
            })
            .collect()
    }

    #[test]
    fn pruning_keeps_the_newest_and_leaves_the_log_alone() {
        let dir = scratch("prune");
        let written = seed(&dir, KEEP + 4);
        // The log files share this directory and must survive.
        std::fs::write(dir.join("stackvo.2026-08-06.log"), "x").unwrap();
        std::fs::write(dir.join("crash-notes.md"), "x").unwrap();

        prune(&dir);

        let left: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(left.iter().filter(|n| n.ends_with(".txt")).count(), KEEP);
        // The four oldest went, and they are the four oldest — not four
        // arbitrary ones.
        for gone in &written[..4] {
            assert!(!left.contains(gone), "{gone} should have been pruned");
        }
        for kept in &written[4..] {
            assert!(left.contains(kept), "{kept} should have survived");
        }
        assert!(left.contains(&"stackvo.2026-08-06.log".to_string()));
        assert!(
            left.contains(&"crash-notes.md".to_string()),
            "pruning is matching on the prefix alone"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_directory_at_the_cap_is_untouched() {
        let dir = scratch("under-cap");
        seed(&dir, KEEP);
        prune(&dir);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), KEEP);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn facts(message: &str) -> Facts {
        Facts {
            message: message.to_string(),
            location: "src/engine.rs:412:9".to_string(),
            thread: "tokio-runtime-worker".to_string(),
            seconds: 1_786_007_730,
            backtrace: "   0: stackvo_desktop_lib::engine::status".to_string(),
        }
    }

    /// The report is the artefact a user attaches to an issue, so the fields
    /// that make it triageable have to be in it.
    #[test]
    fn a_report_carries_what_triage_needs() {
        let report = render(&facts(
            "index out of bounds: the len is 0 but the index is 3",
        ));

        for expected in [
            env!("CARGO_PKG_VERSION"),
            "20260806-091530 UTC",
            "1786007730",
            "tokio-runtime-worker",
            "src/engine.rs:412:9",
            "index out of bounds",
            "stackvo_desktop_lib::engine::status",
            std::env::consts::OS,
        ] {
            assert!(
                report.contains(expected),
                "the report is missing {expected:?}:\n{report}"
            );
        }
    }

    /// A crash report nobody can safely attach is a crash report nobody sends.
    #[test]
    fn a_secret_in_the_panic_message_is_masked() {
        let report = render(&facts("could not connect: MYSQL_ROOT_PASSWORD=hunter2"));
        assert!(report.contains("MYSQL_ROOT_PASSWORD=***"), "{report}");
        assert!(!report.contains("hunter2"));
    }

    /// `panic_any` is rare but a dependency is allowed to use it, and a report
    /// that says nothing at all about the payload is worse than one that says
    /// there was not a readable one.
    #[test]
    fn an_unreadable_payload_still_produces_a_report() {
        let report = render(&facts("<non-string panic payload>"));
        assert!(report.contains("<non-string panic payload>"));
        assert!(
            report.contains("src/engine.rs:412:9"),
            "location still helps"
        );
    }
}
