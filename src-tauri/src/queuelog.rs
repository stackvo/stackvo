//! What the queue did, read out of the worker's own report.
//!
//! ## Why this is not in the bridge
//!
//! [`crate::debugbridge`] catches `dump()` and the request around it by being
//! loaded through `auto_prepend_file`, which runs before the application. Four
//! listeners on Laravel's own queue events would be the obvious way to add
//! jobs to the same file, and there is no moment in that file where they can
//! be attached: the container does not exist yet. Watching the autoloader for
//! the queue's classes looks like the way in and was measured not to be —
//! Composer registers its loader with `$prepend = true`, so it lands in front
//! of anything the prepend file registered and a class it can resolve never
//! reaches a handler behind it.
//!
//! So the job half is read the way the database half is read. [`crate::querylog`]
//! does not instrument anybody's application either; it turns on the server's
//! own general log and parses what the server writes. `queue:work` already
//! reports every job it takes, on its own stdout, in a fixed two-column
//! format:
//!
//! ```text
//! 2026-08-28 18:08:27 App\Jobs\Hello ............................ 40.80ms DONE
//! ```
//!
//! That line is Laravel's own account of what happened, it costs nothing to
//! produce, and it is already sitting in a container this app started.
//!
//! ## Two clocks, and which one is used
//!
//! The line carries a timestamp and it is not the one used here. That one is
//! printed by PHP inside the container, in whatever timezone the image was
//! built with, and a job stamped in UTC lands hours away from a dump stamped
//! by the same wall clock the browser used. The engine's own `--timestamps`
//! prefix is read instead: one clock, the host's, for every moment on the axis
//! — the same correction the query log needed before a query could sit beside
//! a dump.
//!
//! ## What this cannot say
//!
//! `FAIL` is printed for **every attempt that threw**, and the console says
//! nothing about whether another attempt is coming. A job with `--tries=3`
//! that always throws produces three identical rows, which is the truth about
//! what the queue did and not a duplicate. Whether the queue then gave up is
//! written to `failed_jobs`, not to stdout, and inventing that distinction
//! here would be guessing.
//!
//! And this reads the worker **this app started**. Somebody running
//! `php artisan queue:work` in their own terminal is not producing a container
//! log, and no row appears for them. That is a real limit of reading the
//! producer's own record rather than instrumenting the application, and it is
//! the same limit the query log has when the database is not one of ours.

use std::path::Path;

/// How many of the worker's lines are read per poll.
///
/// One job is one or two lines and the pane polls every second, so this is
/// three orders of magnitude more than a poll can miss. It is a bound on a
/// container's output, not a window: everything older than the cursor is
/// dropped by [`ingest`] regardless.
///
/// Read in full on every poll rather than asked for by `since`, and that is a
/// deliberate non-optimisation: the same poll already reads the whole events
/// file, which is bounded at eight megabytes, so four hundred lines of a
/// container's stdout is not the cost in this loop and pretending otherwise
/// would buy a second window to keep in step with the cursor.
pub const TAIL: u32 = 400;

/// One thing the worker finished with a job.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// Seconds since the epoch, from the engine's clock.
    pub at: f64,
    /// Laravel's display name for the job — usually the class.
    pub job: String,
    /// `ok` or `failed`. See the module header on what `failed` does not mean.
    pub outcome: String,
    /// Milliseconds, as the worker measured them.
    pub duration: Option<f64>,
}

/// Which statuses are terminal, and what each one is called here.
///
/// `RUNNING` is deliberately absent. It is the *start* of the thing the next
/// line reports, and a row for it would double every job on the axis while
/// carrying neither an outcome nor a duration — the two facts a job row exists
/// to carry.
fn outcome_of(status: &str) -> Option<&'static str> {
    match status {
        "DONE" => Some("ok"),
        "FAIL" => Some("failed"),
        _ => None,
    }
}

/// `40.80ms` or `1.5s` → milliseconds.
fn milliseconds(token: &str) -> Option<f64> {
    let (number, scale) = if let Some(rest) = token.strip_suffix("ms") {
        (rest, 1.0)
    } else if let Some(rest) = token.strip_suffix('s') {
        (rest, 1000.0)
    } else {
        return None;
    };
    number.parse::<f64>().ok().map(|n| n * scale)
}

/// Everything the worker reported, oldest first.
///
/// Pure, and takes the text rather than the container: the engine is behind an
/// await and the parsing is the part worth testing.
pub fn parse(text: &str) -> Vec<Run> {
    let mut out = Vec::new();

    for line in text.lines() {
        // `<engine instant> <the worker's own line>`. A line with no space is
        // not one of ours.
        let Some((stamp, rest)) = line.split_once(' ') else {
            continue;
        };
        let Some(at) = crate::mail::epoch_of(stamp) else {
            continue;
        };

        let mut parts: Vec<&str> = rest.split_whitespace().collect();

        let Some(status) = parts.pop() else { continue };
        let Some(outcome) = outcome_of(status) else {
            continue;
        };

        let mut duration = None;
        if let Some(last) = parts.last() {
            if let Some(ms) = milliseconds(last) {
                duration = Some(ms);
                parts.pop();
            }
        }

        // The padding between the two columns is a run of dots, and it is its
        // own whitespace-separated token — measured, not assumed.
        while parts.last().is_some_and(|t| t.chars().all(|c| c == '.')) {
            parts.pop();
        }

        // What is left is the worker's own timestamp followed by the job's
        // name. Two tokens of date and time are dropped by position rather
        // than parsed: they are the clock this module does not use, and a
        // parser for them would be a second answer to a question already
        // answered by the engine's prefix.
        if parts.len() >= 2 && parts[0].len() == 10 && parts[1].len() == 8 {
            parts.drain(..2);
        }

        let job = parts.join(" ");
        if job.is_empty() {
            continue;
        }

        out.push(Run {
            at,
            job,
            outcome: outcome.to_string(),
            duration,
        });
    }

    out.sort_by(|a, b| a.at.total_cmp(&b.at));
    out
}

/// Where the last ingested instant is remembered.
///
/// Beside the events file rather than inside it. The alternative — taking the
/// newest job already in the file as the cursor — reads well until somebody
/// clears the pane, at which point the whole of the worker's tail is ingested
/// again and the rows they just dismissed come back.
pub fn cursor_path(root: &Path, project: &str) -> std::path::PathBuf {
    crate::debugbridge::events_dir(root, project).join("jobs.cursor")
}

fn cursor(root: &Path, project: &str) -> Option<f64> {
    std::fs::read_to_string(cursor_path(root, project))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// One job, in the shape the bridge writes and everything downstream reads.
///
/// Written as a [`crate::debugbridge::Event`] on purpose. The alternative is a
/// second file, a second reader, a second cursor and a second row renderer for
/// something that belongs on the same axis as the dump it explains — and the
/// `kind` field was put there, before there was a second value for it, so that
/// exactly this would not be necessary.
fn event_of(run: &Run) -> crate::debugbridge::Event {
    crate::debugbridge::Event {
        at: run.at,
        kind: "job".to_string(),
        label: Some(run.job.clone()),
        // No call site. The worker reports the job, not the line inside it,
        // and a file this module invented would be a link that opens nothing.
        file: None,
        line: None,
        // Not attributed to a request, for the reason the module header of
        // `timeline` gives about queries: a job was dispatched by some request
        // that is no longer running, and joining them by time would be wrong
        // on the first busy minute and wrong silently.
        request: None,
        sapi: None,
        duration: run.duration,
        outcome: Some(run.outcome.clone()),
        value: serde_json::Value::Null,
    }
}

/// Read the worker's tail and append whatever is new to the events file.
///
/// Answers how many rows were added. Appending rather than returning them is
/// what keeps the pane, the timeline, the MCP tools and the request explainer
/// on one reader: none of them learns that a job came from somewhere else.
///
/// Failure is silent by design — no worker, no container, an engine that is
/// down. Every one of those is a normal state for a project that has never
/// queued anything, and none of them is worth an error on a poll that is also
/// asking about dumps.
pub async fn ingest(root: &Path, project: &str) -> usize {
    let container = format!(
        "stackvo-{}",
        crate::worker::container_id(project, crate::worker::Kind::Queue)
    );

    let Ok(text) = crate::engine::logs_tail(&container, TAIL, true).await else {
        return 0;
    };

    let all = parse(&text);

    // No cursor is the first look at this worker, and it is answered by
    // remembering where the worker is rather than by ingesting where it has
    // been. A worker restarts hourly and keeps four hundred lines; "turn
    // capture on" must not mean "read the last hour into the pane". The switch
    // and the clear button both remove this file, so both re-seed here.
    //
    // A worker that has printed nothing seeds at **zero**, not at the clock.
    // That is the difference between "start after the history" and "start
    // after nothing", and getting it wrong swallowed the first job somebody
    // ran after turning capture on: the next poll took the line that job had
    // just written as history to skip past. Zero says the opposite — there is
    // no history, so everything from here is new — and it costs nothing,
    // because there is nothing older than it to replay.
    let Some(since) = cursor(root, project) else {
        let seed = all.last().map(|run| run.at).unwrap_or(0.0);
        let _ = std::fs::write(cursor_path(root, project), seed.to_string());
        return 0;
    };

    let runs: Vec<&Run> = all.iter().filter(|run| run.at > since).collect();
    let Some(newest) = runs.last().map(|run| run.at) else {
        return 0;
    };

    let mut lines = String::new();
    for run in &runs {
        let Ok(line) = serde_json::to_string(&event_of(run)) else {
            continue;
        };
        lines.push_str(&line);
        lines.push('\n');
    }

    use std::io::Write;
    let path = crate::debugbridge::events_path(root, project);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Opened for append, like the container's own `FILE_APPEND`: a worker
    // writing a dump at the same moment interleaves rather than truncates.
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        if file.write_all(lines.as_bytes()).is_err() {
            // The cursor is not moved past rows that were not written down.
            return 0;
        }
    } else {
        return 0;
    }

    let _ = std::fs::write(cursor_path(root, project), newest.to_string());
    runs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact lines a `php:8.4-cli` container printed for two jobs, one of
    /// which threw twice — copied out of `docker logs --timestamps`, not
    /// written from memory.
    const REAL: &str = "\
2026-08-28T18:08:27.635293669Z   2026-08-28 18:08:27 App\\Jobs\\Hello ................................. RUNNING
2026-08-28T18:08:27.675320169Z   2026-08-28 18:08:27 App\\Jobs\\Hello ............................ 40.80ms DONE
2026-08-28T18:08:27.679141336Z   2026-08-28 18:08:27 App\\Jobs\\Boom .................................. RUNNING
2026-08-28T18:08:27.682421586Z   2026-08-28 18:08:27 App\\Jobs\\Boom .............................. 3.35ms FAIL
2026-08-28T18:08:27.699048711Z   2026-08-28 18:08:27 App\\Jobs\\Boom .................................. RUNNING
2026-08-28T18:08:27.700687086Z   2026-08-28 18:08:27 App\\Jobs\\Boom .............................. 1.65ms FAIL
";

    #[test]
    fn the_lines_a_real_worker_printed_parse_into_what_happened() {
        let runs = parse(REAL);

        assert_eq!(runs.len(), 3, "{runs:?}");
        assert_eq!(runs[0].job, "App\\Jobs\\Hello");
        assert_eq!(runs[0].outcome, "ok");
        assert_eq!(runs[0].duration, Some(40.80));

        // Two attempts of the same job, and both are kept: the console says
        // nothing about whether a third is coming, so collapsing them would be
        // inventing an answer.
        assert_eq!(runs[1].job, "App\\Jobs\\Boom");
        assert_eq!(runs[1].outcome, "failed");
        assert_eq!(runs[2].outcome, "failed");
        assert!(runs[2].at > runs[1].at);
    }

    /// `RUNNING` is the start of the row the next line completes. A moment for
    /// it would double every job while carrying neither of the two facts a job
    /// row exists to carry.
    #[test]
    fn a_job_that_only_started_is_not_a_moment_yet() {
        let started =
            "2026-08-28T18:08:27.635293669Z   2026-08-28 18:08:27 App\\Jobs\\Hello ... RUNNING\n";
        assert!(parse(started).is_empty());
    }

    /// The clock used is the engine's, not the one the worker printed. They
    /// disagree by whatever timezone the image was built with, and a job three
    /// hours away from the dump that queued it is not on the same axis.
    #[test]
    fn the_engines_clock_is_the_one_read() {
        let runs = parse(REAL);
        // 2026-08-28T18:08:27.675320169Z
        assert!(
            (runs[0].at - 1_787_940_507.675_32).abs() < 0.001,
            "{}",
            runs[0].at
        );
    }

    #[test]
    fn a_duration_is_read_in_either_unit_and_answered_in_one() {
        assert_eq!(milliseconds("40.80ms"), Some(40.80));
        assert_eq!(milliseconds("1.5s"), Some(1500.0));
        assert_eq!(milliseconds("DONE"), None);
        assert_eq!(milliseconds("ms"), None);
    }

    /// Anything that is not a worker's two-column line is skipped rather than
    /// half-parsed. A container's log carries an application's own output too.
    #[test]
    fn other_output_from_the_same_container_is_left_alone() {
        let noise = "\
2026-08-28T18:08:27.100000000Z [2026-08-28 18:08:27] local.ERROR: something threw
2026-08-28T18:08:27.200000000Z Processing jobs from the [default] queue.
not a log line at all
2026-08-28T18:08:27.300000000Z
";
        assert!(parse(noise).is_empty(), "{:?}", parse(noise));
    }

    /// A job event is written in the bridge's own shape, so nothing
    /// downstream has to learn that it came from somewhere else.
    #[test]
    fn a_job_is_written_as_the_bridge_would_have_written_it() {
        let event = event_of(&parse(REAL)[0]);
        assert_eq!(event.kind, "job");
        assert_eq!(event.label.as_deref(), Some("App\\Jobs\\Hello"));
        assert_eq!(event.outcome.as_deref(), Some("ok"));
        assert_eq!(event.duration, Some(40.80));
        // No call site and no request: the worker reports neither, and a value
        // invented here would be a link that opens nothing and a grouping that
        // is wrong on the first busy minute.
        assert_eq!(event.file, None);
        assert_eq!(event.request, None);

        // And it parses back out of the file as the reader will read it.
        // Everything but the instant survives exactly; the instant is a float
        // through JSON and comes back within a fraction of a microsecond,
        // which is four orders of magnitude finer than the axis it lands on.
        let line = serde_json::to_string(&event).unwrap();
        let back: crate::debugbridge::Event = serde_json::from_str(&line).unwrap();
        assert!(
            (back.at - event.at).abs() < 1e-6,
            "{} {}",
            back.at,
            event.at
        );
        assert_eq!(
            (
                back.kind,
                back.label,
                back.outcome,
                back.duration,
                back.value
            ),
            (
                event.kind.clone(),
                event.label.clone(),
                event.outcome.clone(),
                event.duration,
                event.value.clone()
            )
        );
    }
}
