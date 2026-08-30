//! One request, from the code's side and the database's side, on one axis.
//!
//! The gap this closes was recorded as "dump/mail/log three separate screens,
//! no correlation". Each of those answers a different half of the same
//! question — `dd($user)` says what the code thought it had, the query log says
//! what it actually asked for, and the two were readable only by looking at two
//! screens and comparing clocks by eye.
//!
//! ## Why this is possible now and was not before
//!
//! Two things had to be true. The dump bridge already wrote a `request` field —
//! `GET /api/health`, or the artisan command for a CLI run — and its `Event`
//! already carried a `kind` whose doc comment said, before any of this existed,
//! that it was there "so queries and jobs do not need a second file and a
//! second reader when they arrive". They have arrived. The second is that the query log
//! made the query log readable at all, and made it report seconds since the
//! epoch **as the server computed them** rather than a formatted local time —
//! which is what lets a query sit beside a dump on one axis instead of three
//! hours away from it.
//!
//! ## What correlates, and what only sorts
//!
//! Dumps carry the request they happened in, so several dumps from one page
//! load group together and the group is named. Queries do not: MySQL's general
//! log records the statement and the connection, and nothing in it says which
//! HTTP request caused it. Guessing — "the queries between two dumps belong to
//! the request those dumps belong to" — would be wrong on the first concurrent
//! request and wrong silently, which is the worst kind. So queries are placed
//! on the axis by time and left ungrouped, and the screen says which is which.
//!
//! That is a real limitation and it is the honest one. Attributing a query to a
//! request needs the *application* to say so — a header, a comment appended to
//! the SQL, a Telescope-style collector — and every one of those is code inside
//! somebody's project, which is the thing this feature was built to avoid
//! needing.

use serde::Serialize;

/// What a moment on the axis came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// `dd()` / `dump()`, through the debug bridge.
    Dump,
    /// A statement from the database's own query log.
    Query,
    /// A message the catcher took, which the application believed it sent.
    Mail,
    /// One execution, start to finish — through the same bridge as a dump.
    ///
    /// The one moment on the axis that is a *stretch* rather than an instant,
    /// and it is placed at its end because that is when its duration and its
    /// status became knowable. A request drawn at its start would have to be
    /// drawn without either.
    Request,
    /// One queued job, at the moment the queue finished with it.
    Job,
}

/// Which source a bridge event belongs to.
///
/// A kind this build does not know is read as a dump rather than dropped, and
/// that is the same argument [`crate::debugbridge::Event::value`] makes: a
/// worker that has been running since before an update goes on writing the
/// shape it booted with, and a newer bridge may write a kind an older reader
/// has no case for. Shown in the wrong group beats not shown.
fn source_of(event: &crate::debugbridge::Event) -> Source {
    match event.kind.as_str() {
        "request" => Source::Request,
        "job" => Source::Job,
        _ => Source::Dump,
    }
}

/// One thing that happened, whatever produced it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Moment {
    /// Seconds since the epoch. Both sources report this the same way — see
    /// the module header for why that took a change to make true.
    pub at: f64,
    pub source: Source,
    /// A one-line summary: the dump's label and file, or the statement.
    pub summary: String,
    /// The request this belongs to, where the producer knew one. Always `None`
    /// for a query, and that absence is the point rather than a gap.
    pub request: Option<String>,
    /// For a query, the shape — so a group of them reads as one question.
    pub shape: Option<String>,
}

/// Everything in one window, and what repeated inside it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    /// Oldest first: a timeline read downward is a request read forward.
    pub moments: Vec<Moment>,
    /// Requests the dumps named, in the order they first appear.
    pub requests: Vec<String>,
    /// True when the query log was not recording, so the reader knows the
    /// database half is absent rather than empty.
    pub queries_recording: bool,
}

/// How far back a timeline reaches, in seconds.
///
/// Five minutes. Long enough to hold the page load somebody just did and the
/// one before it; short enough that a session left recording does not turn the
/// screen into a log file. The window is applied to the newest event rather
/// than to the wall clock, so a timeline read ten minutes after the request
/// still shows it.
pub const WINDOW: f64 = 300.0;

/// Every moment the three sources hold, oldest first and unwindowed.
///
/// Split out of [`build`] rather than inlined in it because a second reader
/// arrived with its own window: [`crate::explain`] places one *recorded
/// request* on this axis, and that window is the request's own duration, not
/// the five minutes this module trims to. The ordering, the summaries and the
/// refusal to attribute a query to a request are the same in both — so they
/// live here once, and each caller trims what it asked for.
///
/// Pure, and takes what it needs rather than reading it: the three producers
/// live behind a filesystem, a database and an HTTP API respectively, and a
/// function that reached for them would be untestable in the way that matters.
pub fn collect(
    dumps: &[crate::debugbridge::Event],
    queries: &[crate::querylog::Entry],
    mail: &[crate::mail::MailMessage],
) -> Vec<Moment> {
    let mut moments: Vec<Moment> = Vec::new();

    for event in dumps {
        let source = source_of(event);
        moments.push(Moment {
            at: event.at,
            source,
            summary: match source {
                Source::Request => summarise_request(event),
                Source::Job => summarise_job(event),
                _ => summarise_dump(event),
            },
            request: event.request.clone(),
            shape: None,
        });
    }

    for entry in queries {
        moments.push(Moment {
            at: entry.at,
            source: Source::Query,
            summary: entry.sql.clone(),
            request: None,
            shape: Some(entry.shape.clone()),
        });
    }

    for message in mail {
        // Only the ones whose date could be read. A message the catcher dated
        // in a spelling neither parser knows is left off rather than placed at
        // the epoch — see `mail::epoch_of`, and the note on `at` above.
        let Some(at) = message.at else { continue };
        moments.push(Moment {
            at,
            source: Source::Mail,
            summary: summarise_mail(message),
            // A catcher records the envelope, not the request that produced it,
            // so mail correlates no better than a query does — for the same
            // reason and with the same refusal to guess.
            request: None,
            shape: None,
        });
    }

    // Oldest first. `total_cmp` rather than `partial_cmp().unwrap()`: a NaN
    // reaching here from a malformed file would panic the sort, and a timeline
    // is not worth a crash.
    moments.sort_by(|a, b| a.at.total_cmp(&b.at));
    moments
}

/// The requests the dumps named, in the order they first appear.
///
/// Shared with [`crate::explain`] for the same reason [`collect`] is: a request
/// list built two ways would drift in the one case that matters, which is a
/// page load that produced several dumps.
pub fn requests_of(moments: &[Moment]) -> Vec<String> {
    let mut requests: Vec<String> = Vec::new();
    for moment in moments {
        if let Some(request) = &moment.request {
            if !requests.contains(request) {
                requests.push(request.clone());
            }
        }
    }
    requests
}

/// Build one axis out of the two sources.
///
/// Pure, and takes what it needs rather than reading it: the two producers live
/// behind a filesystem and a database respectively, and a function that reached
/// for both would be untestable in the way that matters — the ordering, the
/// window and the grouping are the logic, and none of them need either.
pub fn build(
    dumps: &[crate::debugbridge::Event],
    queries: &[crate::querylog::Entry],
    mail: &[crate::mail::MailMessage],
    queries_recording: bool,
) -> Timeline {
    let mut moments = collect(dumps, queries, mail);

    // The window, measured from the newest thing there is — not from now.
    // Reading a timeline is something somebody does after the request, and a
    // window anchored to the clock would empty itself while they read.
    if let Some(newest) = moments.last().map(|m| m.at) {
        let floor = newest - WINDOW;
        moments.retain(|m| m.at >= floor);
    }

    let requests = requests_of(&moments);

    Timeline {
        moments,
        requests,
        queries_recording,
    }
}

/// One line for a message: who it went to and what it was about.
fn summarise_mail(message: &crate::mail::MailMessage) -> String {
    let to = message.to.join(", ");
    let subject = if message.subject.trim().is_empty() {
        "(no subject)"
    } else {
        message.subject.trim()
    };
    if to.is_empty() {
        subject.to_string()
    } else {
        format!("{subject} → {to}")
    }
}

/// How long something took, in the unit that keeps it readable.
///
/// Milliseconds up to ten seconds and seconds above it. A queue job that ran
/// for four minutes is `240 s`, not `240000 ms` — a number nobody reads
/// without counting the digits, on the one row where the duration is the whole
/// point.
fn duration(ms: f64) -> String {
    if ms >= 10_000.0 {
        format!("{:.1} s", ms / 1000.0)
    } else {
        format!("{ms:.1} ms")
    }
}

/// One line for a request: what was asked, how it ended, how long it took.
///
/// The request itself is deliberately repeated into the summary rather than
/// left to `Moment::request`. On this axis that field is a *grouping* — every
/// dump raised during the same page load carries it too — so a request row
/// whose summary was only `200 · 12 ms` would be the one row that did not say
/// what it was about.
fn summarise_request(event: &crate::debugbridge::Event) -> String {
    let what = event
        .request
        .clone()
        .or_else(|| event.sapi.clone())
        .unwrap_or_else(|| "request".to_string());

    let mut out = what;
    if let Some(outcome) = &event.outcome {
        out.push_str(&format!(" → {outcome}"));
    }
    if let Some(ms) = event.duration {
        out.push_str(&format!(" · {}", duration(ms)));
    }
    out
}

/// One line for a job: which job, how it ended, how long it ran.
fn summarise_job(event: &crate::debugbridge::Event) -> String {
    let mut out = event.label.clone().unwrap_or_else(|| "job".to_string());
    if let Some(outcome) = &event.outcome {
        out.push_str(&format!(" — {outcome}"));
    }
    if let Some(ms) = event.duration {
        out.push_str(&format!(" · {}", duration(ms)));
    }
    out
}

/// One line for a dump: what it was called and where it was written.
fn summarise_dump(event: &crate::debugbridge::Event) -> String {
    let label = event.label.as_deref().unwrap_or("dump");
    match (&event.file, event.line) {
        (Some(file), Some(line)) => format!("{label} — {file}:{line}"),
        (Some(file), None) => format!("{label} — {file}"),
        _ => label.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debugbridge::Event;
    use crate::querylog::Entry;

    fn dump(at: f64, request: Option<&str>) -> Event {
        Event {
            at,
            kind: "dump".into(),
            label: Some("user".into()),
            file: Some("app/Http/Controllers/Home.php".into()),
            line: Some(42),
            request: request.map(str::to_string),
            sapi: Some("fpm-fcgi".into()),
            duration: None,
            outcome: None,
            value: serde_json::Value::Null,
        }
    }

    fn mail(at: Option<f64>, subject: &str) -> crate::mail::MailMessage {
        crate::mail::MailMessage {
            id: "1".into(),
            from: "app@shop.loc".into(),
            to: vec!["buyer@example.test".into()],
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: Vec::new(),
            subject: subject.into(),
            date: Some("2026-08-15T13:53:36.807Z".into()),
            at,
            snippet: None,
            read: false,
        }
    }

    fn query(at: f64, sql: &str) -> Entry {
        Entry {
            at,
            sql: sql.into(),
            shape: crate::querylog::shape_of(sql),
        }
    }

    /// The whole point: two producers, one axis, in the order things happened.
    #[test]
    fn both_sources_land_on_one_axis_oldest_first() {
        let line = build(
            &[dump(100.0, Some("GET /")), dump(102.0, Some("GET /"))],
            &[query(101.0, "SELECT * FROM users WHERE id = 1")],
            &[],
            true,
        );

        let ats: Vec<f64> = line.moments.iter().map(|m| m.at).collect();
        assert_eq!(ats, vec![100.0, 101.0, 102.0]);
        assert_eq!(line.moments[1].source, Source::Query);
    }

    /// A query is never attributed to a request, and the absence is deliberate
    /// — see the module header. Guessing would be wrong on the first concurrent
    /// request and wrong silently.
    #[test]
    fn a_query_carries_no_request_even_between_two_dumps_that_do() {
        let line = build(
            &[dump(100.0, Some("GET /")), dump(102.0, Some("GET /"))],
            &[query(101.0, "SELECT 1")],
            &[],
            true,
        );

        let q = line
            .moments
            .iter()
            .find(|m| m.source == Source::Query)
            .unwrap();
        assert_eq!(q.request, None);
        assert_eq!(line.requests, vec!["GET /"], "only the dumps name requests");
    }

    /// The window is anchored to the newest event, not to the clock — a
    /// timeline read ten minutes later still holds the request it was opened
    /// for.
    #[test]
    fn the_window_is_measured_from_the_newest_event() {
        let line = build(
            &[
                dump(1_000.0, Some("old")),
                dump(1_000.0 + WINDOW + 1.0, Some("new")),
            ],
            &[],
            &[],
            true,
        );

        assert_eq!(line.moments.len(), 1, "the older one is outside the window");
        assert_eq!(line.requests, vec!["new"]);
    }

    /// Requests appear in the order they first happen, so the list reads the
    /// way the page did.
    #[test]
    fn requests_are_listed_in_the_order_they_first_appear() {
        let line = build(
            &[
                dump(100.0, Some("GET /a")),
                dump(101.0, Some("GET /b")),
                dump(102.0, Some("GET /a")),
            ],
            &[],
            &[],
            true,
        );
        assert_eq!(line.requests, vec!["GET /a", "GET /b"]);
    }

    /// A CLI dump has no request, and that must not become an empty group.
    #[test]
    fn an_event_without_a_request_names_none() {
        let line = build(&[dump(100.0, None)], &[], &[], true);
        assert!(line.requests.is_empty());
        assert_eq!(line.moments.len(), 1);
    }

    /// "The database half is off" and "the database was asked nothing" are
    /// different states, and only one of them has an answer.
    #[test]
    fn not_recording_is_reported_rather_than_shown_as_no_queries() {
        let line = build(&[dump(100.0, None)], &[], &[], false);
        assert!(!line.queries_recording);
        assert!(line.moments.iter().all(|m| m.source == Source::Dump));
    }

    /// The third producer. A page that sent a mail while it ran should show it
    /// between the query that fetched the buyer and the dump that followed.
    #[test]
    fn a_message_lands_on_the_axis_with_the_others() {
        let line = build(
            &[dump(100.0, Some("POST /checkout"))],
            &[query(101.0, "SELECT * FROM users WHERE id = 1")],
            &[mail(Some(102.0), "Your order")],
            true,
        );

        let sources: Vec<Source> = line.moments.iter().map(|m| m.source).collect();
        assert_eq!(sources, vec![Source::Dump, Source::Query, Source::Mail]);
        assert!(line.moments[2].summary.contains("Your order"));
        assert!(line.moments[2].summary.contains("buyer@example.test"));
    }

    /// A date neither parser understood must not become 1970 — on an axis that
    /// is not a missing value, it is a wrong one, and it drags everything else
    /// into a corner.
    #[test]
    fn a_message_with_an_unreadable_date_is_left_off_rather_than_placed_at_zero() {
        let line = build(&[dump(100.0, None)], &[], &[mail(None, "Undated")], true);
        assert_eq!(line.moments.len(), 1);
        assert_eq!(line.moments[0].source, Source::Dump);
    }

    /// A catcher records the envelope, not the request that produced it — so
    /// mail correlates no better than a query, for the same reason.
    #[test]
    fn a_message_names_no_request() {
        let line = build(&[], &[], &[mail(Some(1.0), "x")], true);
        assert_eq!(line.moments[0].request, None);
        assert!(line.requests.is_empty());
    }

    fn event(at: f64, kind: &str) -> Event {
        Event {
            at,
            kind: kind.into(),
            label: None,
            file: None,
            line: None,
            request: None,
            sapi: None,
            duration: None,
            outcome: None,
            value: serde_json::Value::Null,
        }
    }

    /// The kind the bridge writes decides the axis source. Before this there
    /// was one value for the field and everything from the bridge was a dump.
    #[test]
    fn the_bridge_now_lands_on_three_different_rows() {
        let mut request = event(101.0, "request");
        request.request = Some("GET /checkout".into());
        request.outcome = Some("200".into());
        request.duration = Some(23.4);

        let mut job = event(102.0, "job");
        job.label = Some("App\\Jobs\\SendReceipt".into());
        job.outcome = Some("ok".into());
        job.duration = Some(120.0);

        let line = build(
            &[dump(100.0, Some("GET /checkout")), request, job],
            &[],
            &[],
            true,
        );

        let sources: Vec<Source> = line.moments.iter().map(|m| m.source).collect();
        assert_eq!(sources, vec![Source::Dump, Source::Request, Source::Job]);
        assert_eq!(line.moments[1].summary, "GET /checkout → 200 · 23.4 ms");
        assert_eq!(
            line.moments[2].summary,
            "App\\Jobs\\SendReceipt — ok · 120.0 ms"
        );
    }

    /// A row written by a newer bridge than this build knows must still be
    /// shown. Shown in the wrong group beats not shown — a worker keeps the
    /// bridge it booted with for as long as it lives.
    #[test]
    fn a_kind_this_build_does_not_know_is_still_a_moment() {
        let line = build(&[event(100.0, "cache")], &[], &[], true);
        assert_eq!(line.moments.len(), 1);
        assert_eq!(line.moments[0].source, Source::Dump);
    }

    /// A request names itself in its summary as well as in `request`. That
    /// field is a *grouping* — every dump from the same page load carries it —
    /// so a summary of `200 · 12 ms` would be the one row that did not say
    /// what it was about.
    #[test]
    fn a_request_says_what_was_asked_even_though_the_grouping_repeats_it() {
        let mut request = event(100.0, "request");
        request.request = Some("POST /pay".into());
        let line = build(&[request], &[], &[], true);
        assert_eq!(line.moments[0].summary, "POST /pay");
        assert_eq!(line.requests, vec!["POST /pay"]);
    }

    /// Minutes are not read as milliseconds. A job that ran for four minutes
    /// is the row where the duration is the whole point, and `240000.0 ms` is
    /// a number nobody reads without counting the digits.
    #[test]
    fn a_long_duration_changes_unit_rather_than_growing_digits() {
        let mut job = event(100.0, "job");
        job.label = Some("Import".into());
        job.duration = Some(240_000.0);
        let line = build(&[job], &[], &[], true);
        assert!(
            line.moments[0].summary.ends_with("240.0 s"),
            "{}",
            line.moments[0].summary
        );
    }

    #[test]
    fn a_dump_summarises_to_its_label_and_where_it_was_written() {
        let line = build(&[dump(1.0, None)], &[], &[], true);
        assert_eq!(
            line.moments[0].summary,
            "user — app/Http/Controllers/Home.php:42"
        );
    }
}
