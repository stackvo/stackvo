//! Why this request was slow — the three instruments on one request.
//!
//! Three separate panes already existed — SPX (`crate::spx`), the query log
//! (`crate::querylog`) and the axis (`crate::timeline`) — and what was missing
//! was the view that puts them around **one request**, plus N+1 detection. The
//! thing this module is built on: *no new measurement is needed*. Every number here was already
//! being recorded, by instruments that already exist, and was readable only by
//! opening three panes and comparing clocks by eye.
//!
//! ## The common key is a recording, and the join is its window
//!
//! `spx::Report` is the only artefact in this system that names one request
//! (`GET /checkout`), says when it started, and says how long it took. That
//! makes it the key: a report *is* a request, and everything else is placed
//! against the stretch of wall clock the report claims.
//!
//! This is a join by time, and saying so is the whole of the honesty here.
//! `timeline.rs` refused to attribute a query to a request and its reasoning
//! still stands — MySQL's general log records the statement and the connection
//! and nothing about which HTTP request caused it. Nothing below pretends
//! otherwise. What changed is that there is now a **stated** window instead of
//! a reader's eye: everything the site did in that stretch is shown as
//! everything the site did in that stretch, and when something else was
//! recorded across the same stretch it is named — see [`Explanation::overlaps`].
//!
//! ## The tolerance is a truncation, not a guess about clocks
//!
//! php-spx writes `exec_ts` as whole seconds. A request that began at
//! `…07.940` is filed under `…07`, so the true start is somewhere in a
//! one-second band and the true end is a second past where the arithmetic puts
//! it. [`TOLERANCE`] is that band, applied at both ends. It is deliberately not
//! larger: a second of slack on a 30 ms request already widens the window by a
//! factor of sixty, and every millisecond added past the truncation is a
//! statement about somebody else's clock that nothing here measured.
//!
//! ## What only a joined view can say
//!
//! Two of these findings are impossible in any one of the three panes, and they
//! are the reason this module is not a layout:
//!
//! * **The database was asked and the profile does not show it.** The query log
//!   holds statements inside the window and the trace names no driver frame at
//!   all. That is not a slow page, it is a profile that cannot answer the
//!   question — php-spx's `builtins` switch is off, so `PDOStatement::execute`
//!   was never sampled and its time is sitting inside whichever userland
//!   function called it. See [`Finding::NoDriverFrames`].
//! * **The N+1 is in *this* request.** `querylog::repeats` counts shapes across
//!   the whole session, which answers "what does this app do repeatedly". The
//!   question people actually arrive with is "what did *this page load* do three
//!   hundred times", and that is the same function over the windowed slice.
//!
//! ## Nothing here reads anything
//!
//! [`explain`] is pure and takes what it needs. The four producers live behind
//! a gzip file, a database, a directory and an HTTP API, and a function that
//! reached for them would be untestable in exactly the way that matters: the
//! window, the split, the ranking and the refusal to over-claim are the logic,
//! and none of them need a container.

use crate::querylog::{Entry, Repeat};
use crate::spx::{Analysis, Hotspot, Report};
use crate::timeline::Moment;
use serde::Serialize;

/// The slack applied to each end of a recording's window, in seconds.
///
/// One second, and it is `exec_ts`'s own resolution rather than an allowance
/// for drift. See the module header.
pub const TOLERANCE: f64 = 1.0;

/// How many times one shape has to repeat *inside one request* to be a finding.
///
/// Deliberately [`crate::querylog::N_PLUS_ONE`] rather than a number of this
/// module's own. A screen that called three repeats an N+1 in one pane and four
/// in another would be two tools, and the reader would have no way to tell
/// which one they were looking at.
pub const N_PLUS_ONE: usize = crate::querylog::N_PLUS_ONE;

/// The share of a run inside database driver frames that makes it database-bound.
///
/// Thirty per cent. Not a majority, because the case worth naming is the one
/// where the fix is a query and the profile still looks like PHP: a page that
/// spends a third of itself waiting on a database is a page whose next
/// improvement is in the database, and by the time it is past half nobody
/// needed a tool to tell them.
pub const DATABASE_BOUND: f64 = 30.0;

/// The share one function has to hold on its own to be worth naming.
///
/// Twenty per cent of the run in a single function's *own* body. Exclusive
/// rather than inclusive, because every request has a controller holding 99%
/// inclusive and that is the shape of a call stack, not a finding.
pub const HOTSPOT_SHARE: f64 = 20.0;

/// How many hotspots and how many statements cross the boundary.
///
/// The same ceiling `spx::HOTSPOTS` uses, for the same reason: this is a
/// screen, and a list nobody scrolls to the end of is a list that cost bytes to
/// deliver. The counts above the lists are computed over everything.
pub const SHOWN: usize = crate::spx::HOTSPOTS;

// ------------------------------------------------------------------ the window

/// Where a window came from, which decides how much it can be trusted.
///
/// The distinction is the difference between a number this app watched happen
/// and a number it worked out, and it is on the wire because the screen has to
/// be able to say which — see [`Window::of`] for what the second one rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Basis {
    /// This app sent the request and held the clock on both sides of it.
    Observed,
    /// Worked out from `exec_ts` and the run's own wall time.
    Derived,
}

/// The stretch of wall clock a recording claims, in seconds since the epoch.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub from: f64,
    pub to: f64,
    pub basis: Basis,
}

impl Window {
    /// A report's window, worked out from what php-spx wrote.
    ///
    /// `wall_time_us` is microseconds — php-spx names the field `wall_time_ms`
    /// and it is not milliseconds, which `spx::Report` measured and documents.
    /// Reading it as milliseconds here would open a window a thousand times too
    /// wide and quietly attribute the whole session to one request.
    ///
    /// This is the fallback, and it carries one premise nothing in this tree
    /// could settle: that `exec_ts` is the moment the run **started** rather
    /// than the moment the file was written, during the request's shutdown. If
    /// that is wrong the window sits one whole duration late, and quietly — the
    /// pane renders and the numbers stay plausible. `examples/explain_probe.rs`
    /// asks a live container the question. [`Window::observed`] makes it
    /// unnecessary for every recording this app starts itself, which is the
    /// path the pane's own button takes.
    pub fn of(report: &Report) -> Window {
        let started = report.recorded_at as f64;
        let seconds = report.wall_time_us as f64 / 1_000_000.0;
        Window {
            from: started - TOLERANCE,
            to: started + seconds + TOLERANCE,
            basis: Basis::Derived,
        }
    }

    /// The window this app watched a recording across.
    ///
    /// No tolerance, because there is nothing here to round: `from` is the host
    /// clock before the request was sent and `to` is the host clock once the
    /// report was on disk, which is after php-spx finished writing it. The pair
    /// brackets the request by construction, and it does so whatever `exec_ts`
    /// turns out to mean.
    ///
    /// It is wider than the run — it includes the connection, the response and
    /// however long the poll took to notice the file — and wider in the safe
    /// direction: a window that is too generous shows statements that were not
    /// this request's, which the screen already says is possible, where one
    /// that is too tight hides the ones that were.
    pub fn observed(observed: &crate::spx::Observed) -> Window {
        Window {
            from: observed.from,
            to: observed.to,
            basis: Basis::Observed,
        }
    }

    pub fn holds(&self, at: f64) -> bool {
        at >= self.from && at <= self.to
    }

    /// Do two windows share any wall clock at all?
    pub fn meets(&self, other: &Window) -> bool {
        self.from <= other.to && other.from <= self.to
    }
}

// ------------------------------------------------------------------ the split

/// Where the run's time went, in two parts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
    /// Microseconds inside a database driver's own body.
    ///
    /// The sum of the **exclusive** time of every frame [`is_database`] names.
    /// Exclusive and not inclusive, because inclusive would count a
    /// `PDOStatement::execute` reached from inside `PDO::query` twice, and
    /// because the number this answers to is "how long was PHP waiting on the
    /// database", which is time in the driver's own body by definition.
    pub database_us: f64,
    pub database_percent: f64,
    /// The rest of the run: everything that is not a driver frame.
    pub php_us: f64,
    pub php_percent: f64,
    /// The frames that were summed, so the number above is auditable.
    ///
    /// Present rather than implied: a share nobody can decompose is a share
    /// nobody can argue with, and the matcher below is a list of names that
    /// somebody's own `pg_export` could join by accident.
    pub drivers: Vec<Hotspot>,
}

/// Is this frame the database, as php-spx names frames?
///
/// php-spx writes methods as `Class::method` and functions by their bare name,
/// so this is a match on names and nothing cleverer. It covers what
/// `querylog::supports` covers and no more — the pane exists to explain a
/// request against the *statement log*, and counting a Redis call as database
/// time would put a number on screen that the list underneath cannot account
/// for.
///
/// Two shapes are deliberately absent:
///
/// * **Framework layers.** `Illuminate\Database\Connection::run` is where a
///   Laravel query goes through, and it is not the database — its own body is
///   the framework's bookkeeping and the wait happens further down, in the
///   driver. Counting it would double-count against the driver frame beneath.
/// * **Caches and queues.** Redis, Memcached and AMQP are not what the query
///   log records, so time in them cannot be checked against anything on this
///   screen.
pub fn is_database(function: &str) -> bool {
    // Class methods, by the class that owns them.
    const CLASSES: &[&str] = &[
        "PDO::",
        "PDOStatement::",
        "mysqli::",
        "mysqli_stmt::",
        "mysqli_result::",
        "SQLite3::",
        "SQLite3Stmt::",
        "SQLite3Result::",
    ];
    // Bare functions, by prefix. `pg_` and `mysqli_` are the procedural halves
    // of the same two extensions; `sqlite_` is the older one.
    const PREFIXES: &[&str] = &["pg_", "mysqli_", "sqlite_"];
    // Mongo has no procedural half and one namespace, so it is a prefix on the
    // namespace rather than a class list — `Manager`, `Server`, `Query` and
    // `BulkWrite` all sit under it and all of them are the driver.
    const NAMESPACES: &[&str] = &["MongoDB\\Driver\\"];

    CLASSES.iter().any(|c| function.starts_with(c))
        || NAMESPACES.iter().any(|n| function.starts_with(n))
        // A prefix match only where there is no `::`: `Foo::pg_thing` is
        // somebody's method and `pg_query` is the extension's function.
        || (!function.contains("::")
            && PREFIXES.iter().any(|p| function.starts_with(p)))
}

/// Split one analysis into driver time and everything else.
///
/// Takes the **whole** hotspot list rather than the truncated one a screen
/// shows. A request whose driver frames all sit below the twenty-fifth function
/// by exclusive share is exactly the request this split is for, and computing
/// it from a top-25 would report it as pure PHP.
pub fn split(analysis: &Analysis) -> Split {
    let whole = analysis.wall_time_us as f64;

    let drivers: Vec<Hotspot> = analysis
        .hotspots
        .iter()
        .filter(|h| is_database(&h.function))
        .cloned()
        .collect();

    let database_us: f64 = drivers.iter().map(|h| h.exclusive_us).sum();
    // Clamped rather than trusted. The shares come from a replay that may have
    // been truncated, and a rounding path that produced 100.4% would render as
    // a negative PHP share — a number that is not merely wrong but reads as a
    // different kind of bug.
    let database_percent = if whole > 0.0 {
        (database_us / whole * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    Split {
        database_us,
        database_percent,
        php_us: (whole - database_us).max(0.0),
        php_percent: 100.0 - database_percent,
        drivers,
    }
}

// ---------------------------------------------------------------- the findings

/// What this request's evidence supports saying about it.
///
/// A kind and its numbers, never a sentence. The three surfaces render these in
/// their own words and two of them render them in two languages; a sentence
/// built here would be English in a Turkish window, which is the defect
/// `tests/language-of-parts.spec.js` exists to keep out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// One statement shape ran [`N_PLUS_ONE`] times or more inside the window.
    NPlusOne,
    /// [`DATABASE_BOUND`] or more of the run was inside driver frames.
    DatabaseBound,
    /// One function held [`HOTSPOT_SHARE`] or more of the run in its own body.
    Hotspot,
    /// Statements landed in the window and the trace names no driver frame.
    ///
    /// The cross-check only a joined view can make. Almost always php-spx's
    /// `builtins` switch: with internal functions unprofiled, the wait inside
    /// `PDOStatement::execute` is charged to whichever userland function called
    /// it, so a database-bound request profiles as a PHP-bound one.
    NoDriverFrames,
    /// The query log was not recording, so the database half is absent.
    ///
    /// Absent rather than empty, and the distinction is the finding: without it
    /// an unrecorded request and a request that asked nothing are one picture.
    QueriesUnrecorded,
    /// The log was recording and holds statements, and none are in the window.
    ///
    /// Worth naming rather than showing as an empty list: it is what a window
    /// that landed in the wrong place looks like, and it is also what a request
    /// that genuinely touched no database looks like. The reader is told which
    /// two possibilities they are choosing between instead of being handed one.
    QueriesOutsideWindow,
    /// Another recording claims part of the same wall clock.
    ///
    /// Everything joined by time is then shared between them, and the reader
    /// has to be told before they read a number as belonging to one request.
    Overlaps,
    /// The trace half of the pair could not be read.
    TraceMissing,
    /// The trace was longer than the cap, so the shares describe its beginning.
    Truncated,
}

/// One thing worth saying, with the numbers that support it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub kind: Kind,
    /// The shape, the function, or nothing — whichever the kind is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// A count where the kind counts something: repeats, or overlaps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    /// A share of the run where the kind is about one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<f64>,
}

impl Finding {
    fn bare(kind: Kind) -> Finding {
        Finding {
            kind,
            subject: None,
            count: None,
            percent: None,
        }
    }

    /// Where this kind sits in the list, lowest first.
    ///
    /// Answers before caveats, and within the answers the ones that name a fix.
    /// An N+1 is first because it is the one finding that says what to change;
    /// the caveats are last because they qualify the answers above them and a
    /// reader who stops early has still read the answer.
    fn rank(&self) -> u8 {
        match self.kind {
            Kind::NPlusOne => 0,
            Kind::DatabaseBound => 1,
            Kind::Hotspot => 2,
            Kind::NoDriverFrames => 3,
            Kind::QueriesUnrecorded => 4,
            Kind::QueriesOutsideWindow => 5,
            Kind::Overlaps => 6,
            Kind::TraceMissing => 7,
            Kind::Truncated => 8,
        }
    }
}

// ------------------------------------------------------------- the explanation

/// One request, and everything three instruments recorded across it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    /// php-spx's own report key — the thing this whole view is keyed on.
    pub key: String,
    /// `GET /checkout`, when the run was a request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    /// The command line, when it was a CLI run instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub cli: bool,
    /// Unix seconds, as php-spx filed the run.
    pub recorded_at: i64,
    /// Microseconds. See `spx::Report::wall_time_us` for why it is not `_ms`.
    pub wall_time_us: u64,
    pub window: Window,
    /// The trace half was read, so the shares below mean something.
    ///
    /// False is not an error: the metadata half is enough to say what the
    /// request was and when, and the query half is untouched by it. Half an
    /// explanation is worth more than an error page — the rule
    /// `commands::request_timeline` already follows for an unreachable database.
    pub trace_read: bool,
    /// The trace was cut at the cap, so the shares are about the run's start.
    pub truncated: bool,
    /// Distinct functions the trace named, before [`SHOWN`] trimmed the list.
    pub functions: usize,
    /// Where the time went, when there was a trace to say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<Split>,
    /// The heaviest functions by their own body, longest first, capped.
    pub hotspots: Vec<Hotspot>,
    /// The statements the log holds inside the window, oldest first.
    pub queries: Vec<Entry>,
    /// How many there were before [`SHOWN`] trimmed the list.
    pub query_count: usize,
    /// Shapes repeated [`N_PLUS_ONE`] times or more **inside this request**.
    pub repeats: Vec<Repeat>,
    /// Was the log recording at all? False means absent, not empty.
    pub queries_recording: bool,
    /// Statements the log holds outside this window.
    ///
    /// The denominator that makes an empty `queries` readable: none in the
    /// window out of none recorded is a quiet session, none in the window out
    /// of four hundred recorded is a request that asked nothing.
    pub queries_elsewhere: usize,
    /// Dumps, statements and mail on one axis, trimmed to the window.
    pub moments: Vec<Moment>,
    /// The requests the dumps inside the window named, first appearance first.
    ///
    /// A dump *does* carry its request, so this is the one attribution on the
    /// screen that is not a join by time — and where it disagrees with
    /// `request` above, two requests were in flight.
    pub requests: Vec<String>,
    /// Other recordings claiming part of the same wall clock, newest first.
    pub overlaps: Vec<String>,
    /// The profiler is currently set to sample PHP's own functions.
    ///
    /// Read from the project's config, which describes **now** rather than the
    /// moment this recording was made — so it is offered as context for
    /// [`Kind::NoDriverFrames`] and never as a claim about the recording.
    pub builtins: bool,
    /// What the evidence supports saying, most useful first.
    pub findings: Vec<Finding>,
}

/// Put one recording, one query log and one axis around a single request.
///
/// `analysis` is `None` when the trace half could not be read; everything that
/// does not depend on it still comes back. `observed` is the stretch this app
/// watched the request across, when it was this app that sent it — and where
/// there is one it replaces the arithmetic entirely, premise and all.
/// `others` is every other recording the project holds — the report itself is
/// filtered out by key rather than by the caller having to remember to remove
/// it.
#[allow(clippy::too_many_arguments)]
// ------------------------------------------------------------------- replay

/// Two recordings of one request, before and after.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Replay {
    pub before: crate::spx::Report,
    pub after: crate::spx::Report,
    /// `after - before`, in SPX's own microseconds. Negative is faster.
    ///
    /// A number and not a verdict. One run against one run is not a benchmark —
    /// a cold opcache, a cold query cache and whatever else the machine was
    /// doing are all inside it — and a field called `faster` would invite a
    /// conclusion the measurement cannot carry. The two reports are returned
    /// whole so the difference can be read where it came from.
    pub wall_time_us: i64,
    pub peak_memory: i64,
}

/// The path a recording could be replayed at, or why it cannot be.
///
/// ## What a recording actually holds
///
/// `spx::Report` is the only artefact in this system that names one request,
/// and what it names is the *line*: `GET /checkout`. Not its headers, not its
/// body, not the session it ran under — nothing records those, and that is the
/// whole boundary of what a replay can honestly be here.
///
/// So a GET is replayable and everything else is refused **by name**. A POST
/// re-sent without its body and its session is not the request that was
/// recorded; against any framework with CSRF it answers 419 and against one
/// without it, it does something the person did not ask for. Producing a
/// result that looks like an answer and is not is worse than saying no.
pub fn replayable(report: &crate::spx::Report) -> std::result::Result<String, String> {
    if report.cli {
        return Err(format!(
            "this recording is a command run{}, not a request — replaying it would mean              running the command again, which is a different act from re-requesting a page",
            report
                .command
                .as_deref()
                .map(|c| format!(" ({c})"))
                .unwrap_or_default()
        ));
    }

    let Some(line) = report
        .request
        .as_deref()
        .map(str::trim)
        .filter(|l| !l.is_empty())
    else {
        return Err("this recording does not name a request, so there is nothing to send".into());
    };

    // `GET /checkout?step=2`. Anything else is left alone rather than guessed
    // at: a line this cannot read is one nobody should be re-sending.
    let (method, path) = match line.split_once(' ') {
        Some((method, path)) => (method.trim(), path.trim()),
        None => ("", line),
    };

    if !method.eq_ignore_ascii_case("GET") {
        return Err(format!(
            "this recording is `{line}`. Only the request line was recorded — not its body,              its headers or its session — and a {} replayed without them is a different              request, which is usually answered with a redirect or a 419 rather than the page",
            if method.is_empty() { "request" } else { method }
        ));
    }
    if !path.starts_with('/') {
        return Err(format!(
            "this recording names `{line}`, which is not a path this app will send"
        ));
    }

    Ok(path.to_string())
}

pub fn explain(
    report: &Report,
    analysis: Option<&Analysis>,
    observed: Option<&crate::spx::Observed>,
    queries: &[Entry],
    queries_recording: bool,
    dumps: &[crate::debugbridge::Event],
    mail: &[crate::mail::MailMessage],
    others: &[Report],
    builtins: bool,
) -> Explanation {
    // Watched beats worked out. The arithmetic is the fallback for a recording
    // somebody made in a browser, where this app held no clock at all.
    let window = match observed {
        Some(observed) => Window::observed(observed),
        None => Window::of(report),
    };

    // ---- the database half, trimmed to the window
    let inside: Vec<Entry> = queries
        .iter()
        .filter(|entry| window.holds(entry.at))
        .cloned()
        .collect();
    let query_count = inside.len();
    let queries_elsewhere = queries.len().saturating_sub(query_count);

    // The same counter the whole-session pane uses, over this request's slice.
    // Deliberately `querylog::repeats` and not a second implementation: two
    // functions that both decide what an N+1 is would eventually disagree, and
    // the disagreement would show up as one pane finding what the other did
    // not.
    let repeats = crate::querylog::repeats(&inside);

    // ---- the axis, trimmed to the same window
    let mut moments = crate::timeline::collect(dumps, &inside, mail);
    moments.retain(|moment| window.holds(moment.at));
    let requests = crate::timeline::requests_of(&moments);

    // ---- the code half
    let split = analysis.map(split);
    let mut hotspots: Vec<Hotspot> = analysis.map(|a| a.hotspots.clone()).unwrap_or_default();
    let functions = analysis.map(|a| a.functions).unwrap_or(0);
    let truncated = analysis.is_some_and(|a| a.truncated);
    hotspots.truncate(SHOWN);

    // ---- what else was recorded across the same stretch
    let overlaps: Vec<String> = others
        .iter()
        .filter(|other| other.key != report.key && Window::of(other).meets(&window))
        .map(|other| {
            other
                .request
                .clone()
                .or_else(|| other.command.clone())
                .unwrap_or_else(|| other.key.clone())
        })
        .collect();

    let findings = findings(
        &repeats,
        split.as_ref(),
        &hotspots,
        query_count,
        queries_recording,
        queries_elsewhere,
        &overlaps,
        analysis.is_some(),
        truncated,
    );

    let mut queries = inside;
    queries.truncate(SHOWN);

    Explanation {
        key: report.key.clone(),
        request: report.request.clone(),
        command: report.command.clone(),
        cli: report.cli,
        recorded_at: report.recorded_at,
        wall_time_us: report.wall_time_us,
        window,
        trace_read: analysis.is_some(),
        truncated,
        functions,
        split,
        hotspots,
        queries,
        query_count,
        repeats,
        queries_recording,
        queries_elsewhere,
        moments,
        requests,
        overlaps,
        builtins,
        findings,
    }
}

/// Rank what the evidence supports saying.
#[allow(clippy::too_many_arguments)]
fn findings(
    repeats: &[Repeat],
    split: Option<&Split>,
    hotspots: &[Hotspot],
    query_count: usize,
    queries_recording: bool,
    queries_elsewhere: usize,
    overlaps: &[String],
    trace_read: bool,
    truncated: bool,
) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();

    // Every repeated shape, not only the worst. A page with three N+1s in it
    // has three things to fix, and a list that named one would send somebody
    // back for the second after the next recording.
    for repeat in repeats {
        out.push(Finding {
            kind: Kind::NPlusOne,
            subject: Some(repeat.shape.clone()),
            count: Some(repeat.count as u64),
            percent: None,
        });
    }

    if let Some(split) = split {
        if split.database_percent >= DATABASE_BOUND {
            out.push(Finding {
                kind: Kind::DatabaseBound,
                subject: None,
                count: None,
                percent: Some(split.database_percent),
            });
        }

        // The database's own frames are excluded here. They are already the
        // finding above, and naming `PDOStatement::execute` as a hotspot beside
        // "this request is database-bound" is the same sentence twice.
        if let Some(top) = hotspots
            .iter()
            .find(|h| !is_database(&h.function) && h.exclusive_percent >= HOTSPOT_SHARE)
        {
            out.push(Finding {
                kind: Kind::Hotspot,
                subject: Some(top.function.clone()),
                count: None,
                percent: Some(top.exclusive_percent),
            });
        }

        // The cross-check. Statements landed in the window and the trace has no
        // driver frame to account for them, which means the profile cannot
        // answer the question it appears to be answering.
        if query_count > 0 && split.drivers.is_empty() {
            out.push(Finding {
                kind: Kind::NoDriverFrames,
                subject: None,
                count: Some(query_count as u64),
                percent: None,
            });
        }
    }

    if !queries_recording {
        out.push(Finding::bare(Kind::QueriesUnrecorded));
    } else if query_count == 0 && queries_elsewhere > 0 {
        out.push(Finding {
            kind: Kind::QueriesOutsideWindow,
            subject: None,
            count: Some(queries_elsewhere as u64),
            percent: None,
        });
    }

    if !overlaps.is_empty() {
        out.push(Finding {
            kind: Kind::Overlaps,
            subject: None,
            count: Some(overlaps.len() as u64),
            percent: None,
        });
    }

    if !trace_read {
        out.push(Finding::bare(Kind::TraceMissing));
    }
    if truncated {
        out.push(Finding::bare(Kind::Truncated));
    }

    // Stable within a rank: the repeats arrive from `querylog::repeats` already
    // ordered most-repeated first, and a sort that reshuffled equal ranks would
    // make two reads of one recording disagree.
    out.sort_by_key(|finding| finding.rank());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debugbridge::Event;

    fn report(key: &str, at: i64, wall_us: u64) -> Report {
        Report {
            key: key.to_string(),
            recorded_at: at,
            cli: false,
            request: Some(format!("GET /{key}")),
            command: None,
            wall_time_us: wall_us,
            peak_memory: 0,
            call_count: 0,
            bytes: 0,
        }
    }

    // ---- what may be replayed ------------------------------------------

    #[test]
    fn a_recorded_get_is_replayable_at_the_path_it_names() {
        let mut r = report("k", 0, 100);
        r.request = Some("GET /checkout?step=2".into());
        assert_eq!(replayable(&r).unwrap(), "/checkout?step=2");

        // The query string is part of the request and travels with it — a
        // replay that dropped it would be a different page.
        r.request = Some("GET /".into());
        assert_eq!(replayable(&r).unwrap(), "/");
    }

    /// The boundary of the whole feature, and the reason it is a refusal
    /// rather than an attempt: a recording names the request *line* and
    /// nothing else.
    #[test]
    fn anything_but_a_get_is_refused_with_the_reason() {
        let mut r = report("k", 0, 100);

        r.request = Some("POST /checkout".into());
        let why = replayable(&r).unwrap_err();
        assert!(why.contains("POST /checkout"), "{why}");
        assert!(
            why.contains("session") && why.contains("419"),
            "the refusal has to say what would actually happen: {why}"
        );

        r.cli = true;
        r.command = Some("artisan queue:work".into());
        let why = replayable(&r).unwrap_err();
        assert!(why.contains("artisan queue:work"), "{why}");

        r.cli = false;
        r.command = None;
        r.request = None;
        assert!(replayable(&r).is_err(), "there is nothing to send");

        r.request = Some("GET http://elsewhere.example/".into());
        assert!(
            replayable(&r).is_err(),
            "a recording naming somewhere else is not this app's to re-send"
        );
    }

    fn entry(at: f64, sql: &str) -> Entry {
        Entry {
            at,
            sql: sql.to_string(),
            shape: crate::querylog::shape_of(sql),
        }
    }

    fn hotspot(function: &str, exclusive_percent: f64, whole_us: f64) -> Hotspot {
        Hotspot {
            function: function.to_string(),
            calls: 1,
            exclusive_us: exclusive_percent / 100.0 * whole_us,
            exclusive_percent,
            inclusive_us: exclusive_percent / 100.0 * whole_us,
            inclusive_percent: exclusive_percent,
        }
    }

    fn analysis(wall_us: u64, hotspots: Vec<Hotspot>) -> Analysis {
        Analysis {
            key: "k".to_string(),
            wall_time_us: wall_us,
            call_count: 0,
            functions: hotspots.len(),
            events: 0,
            truncated: false,
            hotspots,
        }
    }

    fn kinds(explanation: &Explanation) -> Vec<Kind> {
        explanation.findings.iter().map(|f| f.kind).collect()
    }

    // ------------------------------------------------------------- the window

    /// The unit `wall_time_us` is in, as a test rather than as a comment.
    ///
    /// php-spx calls the field `wall_time_ms` and it holds microseconds. Read as
    /// milliseconds, a 400 ms request would open a window four hundred seconds
    /// wide — which is not an error anywhere, it is the whole session
    /// attributed to one page load.
    #[test]
    fn the_window_reads_wall_time_as_microseconds() {
        let window = Window::of(&report("a", 1_000, 400_000));
        assert_eq!(window.from, 1_000.0 - TOLERANCE);
        assert_eq!(window.to, 1_000.0 + 0.4 + TOLERANCE);
    }

    #[test]
    fn the_window_carries_a_second_of_slack_at_each_end() {
        let window = Window::of(&report("a", 100, 0));
        assert!(window.holds(99.5), "the truncated second before the stamp");
        assert!(window.holds(100.9), "the truncated second after it");
        assert!(!window.holds(98.0));
        assert!(!window.holds(102.0));
    }

    #[test]
    fn two_windows_meet_when_they_share_any_wall_clock() {
        let first = Window::of(&report("a", 100, 2_000_000));
        let second = Window::of(&report("b", 103, 1_000_000));
        let far = Window::of(&report("c", 200, 1_000_000));

        assert!(first.meets(&second));
        assert!(second.meets(&first), "meeting is symmetric");
        assert!(!first.meets(&far));
    }

    /// The premise the arithmetic rests on, and the one an observation removes.
    ///
    /// `Window::of` assumes `exec_ts` is the moment the run started. Nothing in
    /// this tree settles that — `examples/explain_probe.rs` asks a live
    /// container — so a recording this app sent itself carries the clock it
    /// watched instead, and the payload says which of the two the reader is
    /// looking at.
    #[test]
    fn an_observed_window_replaces_the_arithmetic_premise_and_all() {
        let report = report("a", 1_000, 940_000);
        let observed = crate::spx::Observed {
            from: 1_200.5,
            to: 1_202.1,
        };

        let derived = Window::of(&report);
        assert_eq!(derived.basis, Basis::Derived);

        let watched = Window::observed(&observed);
        assert_eq!(watched.basis, Basis::Observed);
        assert_eq!(watched.from, 1_200.5);
        assert_eq!(watched.to, 1_202.1);
        assert!(
            !watched.holds(1_000.5),
            "the arithmetic's window is somewhere else entirely, and the \
             observation does not inherit it"
        );
    }

    /// No slack on an observation. There is nothing here to round.
    #[test]
    fn an_observed_window_carries_no_tolerance() {
        let watched = Window::observed(&crate::spx::Observed {
            from: 100.0,
            to: 102.0,
        });
        assert!(!watched.holds(99.9));
        assert!(!watched.holds(102.1));
        assert!(watched.holds(100.0) && watched.holds(102.0));
    }

    #[test]
    fn the_statements_a_request_asked_are_the_ones_inside_the_watched_stretch() {
        let observed = crate::spx::Observed {
            from: 5_000.0,
            to: 5_002.0,
        };

        let explanation = explain(
            // A stamp the arithmetic would put four thousand seconds away, so
            // the two windows cannot be confused for one another.
            &report("a", 1_000, 940_000),
            None,
            Some(&observed),
            &[
                entry(4_999.0, "SELECT 1"),
                entry(5_000.5, "SELECT * FROM carts WHERE id = 1"),
                entry(5_001.5, "SELECT * FROM carts WHERE id = 2"),
                entry(5_003.0, "SELECT 2"),
            ],
            true,
            &[],
            &[],
            &[],
            true,
        );

        assert_eq!(explanation.window.basis, Basis::Observed);
        assert_eq!(explanation.query_count, 2);
        assert_eq!(explanation.queries_elsewhere, 2);
        assert!(explanation.queries.iter().all(|e| e.sql.contains("carts")));
    }

    #[test]
    fn without_an_observation_the_window_is_the_arithmetic_and_says_so() {
        let explanation = explain(
            &report("a", 1_000, 940_000),
            None,
            None,
            &[],
            true,
            &[],
            &[],
            &[],
            true,
        );

        assert_eq!(explanation.window.basis, Basis::Derived);
        assert_eq!(explanation.window, Window::of(&report("a", 1_000, 940_000)));
    }

    // -------------------------------------------------------------- the split

    #[test]
    fn driver_frames_are_recognised_by_the_names_php_spx_writes() {
        for name in [
            "PDO::query",
            "PDOStatement::execute",
            "mysqli::real_query",
            "mysqli_query",
            "pg_query_params",
            "SQLite3Stmt::execute",
            "MongoDB\\Driver\\Manager::executeQuery",
        ] {
            assert!(is_database(name), "{name} is the database");
        }
    }

    /// The three shapes the matcher must not claim, each for its own reason.
    #[test]
    fn it_does_not_claim_frameworks_caches_or_somebody_elses_prefix() {
        for name in [
            // The framework layer above the driver: counting it would
            // double-count against the driver frame beneath it.
            "Illuminate\\Database\\Connection::run",
            "Doctrine\\DBAL\\Connection::executeQuery",
            // Not what the query log records, so its time cannot be checked
            // against anything on the same screen.
            "Redis::get",
            "Memcached::set",
            // A method whose name happens to start with a driver prefix.
            "Report::pg_export",
            "Mailer::mysqli_stub",
        ] {
            assert!(!is_database(name), "{name} is not the database");
        }
    }

    #[test]
    fn the_split_sums_exclusive_time_and_the_two_halves_make_the_whole() {
        let split = split(&analysis(
            1_000_000,
            vec![
                hotspot("PDOStatement::execute", 40.0, 1_000_000.0),
                hotspot("App\\Controller::index", 35.0, 1_000_000.0),
                hotspot("pg_query", 5.0, 1_000_000.0),
            ],
        ));

        assert_eq!(split.drivers.len(), 2);
        assert!((split.database_percent - 45.0).abs() < 1e-6);
        assert!((split.php_percent - 55.0).abs() < 1e-6);
        assert!((split.database_us + split.php_us - 1_000_000.0).abs() < 1e-6);
    }

    /// A recording whose driver frames all sit below the shown list.
    ///
    /// The reason `split` takes the whole analysis rather than the trimmed list
    /// a screen renders: a hundred small `PDOStatement::execute` calls can each
    /// be under a per-function share that keeps them off a top-25 while together
    /// holding half the request.
    #[test]
    fn the_split_sees_driver_frames_that_no_top_list_would_show() {
        let mut hotspots = vec![hotspot("App\\Controller::index", 50.0, 1_000_000.0)];
        for i in 0..60 {
            hotspots.push(hotspot(
                &format!("PDOStatement::execute#{i}"),
                0.5,
                1_000_000.0,
            ));
        }

        let split = split(&analysis(1_000_000, hotspots));
        assert_eq!(split.drivers.len(), 60);
        assert!((split.database_percent - 30.0).abs() < 1e-6);
    }

    #[test]
    fn a_run_with_no_wall_time_splits_to_zero_rather_than_dividing_by_it() {
        let split = split(&analysis(
            0,
            vec![hotspot("PDOStatement::execute", 40.0, 0.0)],
        ));
        assert_eq!(split.database_percent, 0.0);
        assert!(split.database_percent.is_finite());
        assert!(split.php_percent.is_finite());
    }

    // ------------------------------------------------------------ the joining

    #[test]
    fn only_the_statements_inside_the_window_are_this_requests() {
        let explanation = explain(
            &report("a", 1_000, 500_000),
            None,
            None,
            &[
                entry(996.0, "SELECT 1"),
                entry(1_000.2, "SELECT * FROM users WHERE id = 1"),
                entry(1_000.4, "SELECT * FROM users WHERE id = 2"),
                entry(1_010.0, "SELECT 2"),
            ],
            true,
            &[],
            &[],
            &[],
            true,
        );

        assert_eq!(explanation.query_count, 2);
        assert_eq!(explanation.queries_elsewhere, 2);
        assert!(explanation.queries.iter().all(|e| e.sql.contains("users")));
    }

    /// The N+1 is the one inside this request, not the one in the session.
    #[test]
    fn repeats_are_counted_over_the_window_and_not_the_whole_log() {
        let mut queries: Vec<Entry> = Vec::new();
        // Four of one shape, spread across the session and outside the window.
        for i in 0..4 {
            queries.push(entry(500.0 + i as f64, "SELECT * FROM logs WHERE id = 1"));
        }
        // Three of another, inside it.
        for i in 0..3 {
            queries.push(entry(
                1_000.1 + i as f64 * 0.1,
                "SELECT * FROM orders WHERE user_id = 7",
            ));
        }

        let explanation = explain(
            &report("a", 1_000, 500_000),
            None,
            None,
            &queries,
            true,
            &[],
            &[],
            &[],
            true,
        );

        assert_eq!(explanation.repeats.len(), 1);
        assert!(explanation.repeats[0].shape.contains("orders"));
        assert_eq!(explanation.repeats[0].count, 3);
        assert_eq!(kinds(&explanation)[0], Kind::NPlusOne);
    }

    #[test]
    fn dumps_keep_the_request_they_named_and_queries_are_still_not_given_one() {
        let dump = Event {
            at: 1_000.3,
            kind: "dump".to_string(),
            label: Some("user".to_string()),
            file: Some("app/Http/Controllers/Home.php".to_string()),
            line: Some(21),
            request: Some("GET /a".to_string()),
            sapi: Some("fpm-fcgi".to_string()),
            duration: None,
            outcome: None,
            value: serde_json::Value::Null,
        };

        let explanation = explain(
            &report("a", 1_000, 500_000),
            None,
            None,
            &[entry(1_000.4, "SELECT 1")],
            true,
            &[dump],
            &[],
            &[],
            true,
        );

        assert_eq!(explanation.moments.len(), 2);
        assert_eq!(explanation.requests, vec!["GET /a".to_string()]);
        let query = explanation
            .moments
            .iter()
            .find(|m| m.source == crate::timeline::Source::Query)
            .expect("the statement is on the axis");
        assert!(
            query.request.is_none(),
            "a statement still carries no request — the window is the join, and \
             the screen says so"
        );
    }

    #[test]
    fn a_recording_across_the_same_stretch_is_named_rather_than_ignored() {
        let explanation = explain(
            &report("a", 1_000, 2_000_000),
            None,
            None,
            &[],
            true,
            &[],
            &[],
            &[
                report("b", 1_001, 100_000),
                report("c", 5_000, 100_000),
                // The report itself, as `spx::list` would hand it over.
                report("a", 1_000, 2_000_000),
            ],
            true,
        );

        assert_eq!(explanation.overlaps, vec!["GET /b".to_string()]);
        assert!(kinds(&explanation).contains(&Kind::Overlaps));
    }

    // ------------------------------------------------------------ the findings

    #[test]
    fn a_database_bound_request_says_so_and_names_no_driver_as_a_hotspot() {
        let analysis = analysis(
            1_000_000,
            vec![
                hotspot("PDOStatement::execute", 60.0, 1_000_000.0),
                hotspot("App\\Controller::index", 25.0, 1_000_000.0),
            ],
        );

        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            Some(&analysis),
            None,
            &[entry(1_000.1, "SELECT 1")],
            true,
            &[],
            &[],
            &[],
            true,
        );

        let kinds = kinds(&explanation);
        assert!(kinds.contains(&Kind::DatabaseBound));
        assert!(
            kinds.contains(&Kind::Hotspot),
            "the userland function above the share is still named"
        );
        let hotspot = explanation
            .findings
            .iter()
            .find(|f| f.kind == Kind::Hotspot)
            .expect("named");
        assert_eq!(hotspot.subject.as_deref(), Some("App\\Controller::index"));
    }

    /// The cross-check no single pane can make.
    #[test]
    fn statements_with_no_driver_frame_are_reported_as_an_unanswerable_profile() {
        let analysis = analysis(
            1_000_000,
            vec![hotspot("App\\Controller::index", 90.0, 1_000_000.0)],
        );

        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            Some(&analysis),
            None,
            &[entry(1_000.1, "SELECT 1"), entry(1_000.2, "SELECT 2")],
            true,
            &[],
            &[],
            &[],
            false,
        );

        let finding = explanation
            .findings
            .iter()
            .find(|f| f.kind == Kind::NoDriverFrames)
            .expect("the profile cannot account for the statements");
        assert_eq!(finding.count, Some(2));
        assert!(
            !explanation.builtins,
            "the context the pane needs to explain it"
        );
    }

    #[test]
    fn a_profile_with_no_statements_in_it_does_not_claim_the_profile_is_broken() {
        let analysis = analysis(
            1_000_000,
            vec![hotspot("App\\Controller::index", 90.0, 1_000_000.0)],
        );

        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            Some(&analysis),
            None,
            &[],
            true,
            &[],
            &[],
            &[],
            false,
        );

        assert!(!kinds(&explanation).contains(&Kind::NoDriverFrames));
    }

    #[test]
    fn an_unrecorded_log_is_absent_rather_than_empty() {
        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            None,
            None,
            &[],
            false,
            &[],
            &[],
            &[],
            true,
        );

        let kinds = kinds(&explanation);
        assert!(kinds.contains(&Kind::QueriesUnrecorded));
        assert!(
            !kinds.contains(&Kind::QueriesOutsideWindow),
            "a log that was off cannot have statements somewhere else"
        );
    }

    #[test]
    fn a_recording_log_with_nothing_in_the_window_says_which_two_cases_apply() {
        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            None,
            None,
            &[entry(500.0, "SELECT 1")],
            true,
            &[],
            &[],
            &[],
            true,
        );

        let finding = explanation
            .findings
            .iter()
            .find(|f| f.kind == Kind::QueriesOutsideWindow)
            .expect("named rather than shown as an empty list");
        assert_eq!(finding.count, Some(1));
    }

    #[test]
    fn a_missing_trace_costs_the_shares_and_nothing_else() {
        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            None,
            None,
            &[entry(1_000.1, "SELECT * FROM users WHERE id = 1")],
            true,
            &[],
            &[],
            &[],
            true,
        );

        assert!(!explanation.trace_read);
        assert!(explanation.split.is_none());
        assert!(explanation.hotspots.is_empty());
        assert_eq!(explanation.query_count, 1, "the database half survives");
        assert!(kinds(&explanation).contains(&Kind::TraceMissing));
    }

    /// Answers first, caveats last — a reader who stops early has the answer.
    #[test]
    fn findings_are_ranked_with_the_fix_above_the_qualifications() {
        let analysis = analysis(
            1_000_000,
            vec![hotspot("PDOStatement::execute", 60.0, 1_000_000.0)],
        );

        let mut queries = Vec::new();
        for i in 0..5 {
            queries.push(entry(
                1_000.1 + i as f64 * 0.05,
                "SELECT * FROM orders WHERE id = 1",
            ));
        }

        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            Some(&analysis),
            None,
            &queries,
            true,
            &[],
            &[],
            &[report("b", 1_000, 100_000)],
            true,
        );

        let kinds = kinds(&explanation);
        assert_eq!(kinds[0], Kind::NPlusOne);
        assert_eq!(kinds[1], Kind::DatabaseBound);
        assert_eq!(*kinds.last().expect("some"), Kind::Overlaps);
    }

    #[test]
    fn the_lists_are_capped_and_the_counts_are_not() {
        let mut queries = Vec::new();
        for i in 0..(SHOWN + 20) {
            queries.push(entry(1_000.1, &format!("SELECT {i}")));
        }

        let explanation = explain(
            &report("a", 1_000, 1_000_000),
            None,
            None,
            &queries,
            true,
            &[],
            &[],
            &[],
            true,
        );

        assert_eq!(explanation.queries.len(), SHOWN);
        assert_eq!(explanation.query_count, SHOWN + 20);
    }
}
