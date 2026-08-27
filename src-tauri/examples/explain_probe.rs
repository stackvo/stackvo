//! Is `exec_ts` the moment a recording **started**?
//!
//! `explain.rs` joins a query log, a set of dumps and a mail catcher to one
//! request by the stretch of wall clock that request claims, and it builds that
//! stretch out of two numbers php-spx wrote: `exec_ts`, and the run's own wall
//! time. Every unit test in that module asserts against a `Report` this
//! repository constructed, so all of them prove the arithmetic and none of them
//! prove the premise.
//!
//! The premise is that `exec_ts` is the **start**. If php-spx actually stamps a
//! report at the moment it is written — during the request's shutdown — then
//! the window is the request's duration placed *after* the request, and a slow
//! page would attribute the statements of whatever ran next. That failure is
//! quiet in exactly the way this repository keeps finding: the pane renders, the
//! numbers are plausible, and the answer is about the wrong request.
//!
//! ## What is still riding on it, and what is not
//!
//! A recording **this app started** no longer rides on it at all: `spx.rs` keeps
//! the host clock from both sides of the request and `Window::observed` uses
//! that instead, which brackets the run whatever the field means. That covers
//! the pane's own button and `stackvo spx-record`, which is how recordings are
//! made in practice.
//!
//! What still rides on it is [`explain::Window::of`], the fallback — a recording
//! made in php-spx's own control panel in a browser, and every report already on
//! disk from before the observation existed. The pane says which of the two a
//! reader is looking at, so the premise is visible rather than silent. It is
//! still worth settling, because half the recordings in a workspace can be the
//! browser's.
//!
//! There is no way to settle it from the source tree. php-spx writes the field,
//! and the honest options were to reason about somebody else's C or to measure
//! it. This measures it:
//!
//! ```sh
//! cargo run --example explain_probe -- <project> [path]
//! ```
//!
//! It notes the host clock, asks the site for a page with the profiler trigger
//! on it, notes the clock again, and compares `exec_ts` against both ends. A
//! run long enough to separate them settles it; a run that finishes inside one
//! second cannot, and the probe says so rather than reporting a coin toss as a
//! measurement — which is why `[path]` is worth pointing at something slow.
//!
//! It writes nothing and changes no setting. The profiler has to already be
//! switched on and mounted, because turning it on means recreating a container
//! and that is not a probe's business.

use stackvo_desktop_lib::{explain, spx, workspace};
use std::collections::HashSet;

/// How long a run has to be for the two ends to be distinguishable.
///
/// `exec_ts` is whole seconds, so a request finishing inside one second can be
/// filed under a stamp that is consistent with both readings. Two seconds is the
/// first duration where "the start" and "the end" cannot both be true.
const SEPARABLE_US: u64 = 2_000_000;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let Ok(root) = workspace::resolve().require_root() else {
        println!("no workspace — nothing to measure");
        return;
    };

    let mut args = std::env::args().skip(1);
    let Some(project) = args.next() else {
        println!("usage: cargo run --example explain_probe -- <project> [path]");
        return;
    };
    let path = args.next().unwrap_or_else(|| "/".to_string());

    println!("workspace {}", root.display());
    println!("project   {project}");
    println!("path      {path}\n");

    let status = match spx::status(&root, &project).await {
        Ok(status) => status,
        Err(e) => {
            println!("cannot read the profiler's state — {}", e.message);
            std::process::exit(1);
        }
    };
    if !status.enabled || status.active != Some(true) {
        println!(
            "php-spx is not in {project}'s running container (enabled={}, active={:?}).\n\
             Switch it on in the php-spx card and apply, then run this again.",
            status.enabled, status.active
        );
        std::process::exit(1);
    }
    let Some(domain) = status.domain.as_deref() else {
        println!("{project} has no address to send a request to");
        std::process::exit(1);
    };

    let url = match spx::request_url(domain, &path) {
        Ok(url) => url,
        Err(e) => {
            println!("{}", e.message);
            std::process::exit(1);
        }
    };
    let key = match spx::key(&root) {
        Ok(key) => key,
        Err(e) => {
            println!("no profiler key — {}", e.message);
            std::process::exit(1);
        }
    };
    let config = spx::read_config(&root, &project);

    // Every key already on disk, so the report this request writes can be told
    // from the ones somebody recorded in a browser this morning.
    let before: HashSet<String> = spx::list(&root, &project)
        .into_iter()
        .map(|report| report.key)
        .collect();

    let started = now();
    let code = match spx::send(&url, &spx::trigger_cookie(&key, &config)).await {
        Ok(code) => code,
        Err(e) => {
            println!("the request failed — {}", e.message);
            std::process::exit(1);
        }
    };
    let finished = now();
    println!("{url} answered {code} in {:.3}s\n", finished - started);

    // The pair lands during the request's own shutdown, which can finish after
    // the response has been flushed. The same few tries `spx_record_request`
    // makes, and for the same reason.
    let mut fresh = None;
    for attempt in 0..20 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        fresh = spx::list(&root, &project)
            .into_iter()
            .find(|report| !before.contains(&report.key));
        if fresh.is_some() {
            break;
        }
    }
    let Some(report) = fresh else {
        println!(
            "the request was served and no report appeared. That is the profiler \
             not recording, not this measurement failing."
        );
        std::process::exit(1);
    };

    // Deliberately the DERIVED window, not the one this run just observed: what
    // is being measured is what `exec_ts` means, and reading back the clock
    // this probe itself held would be measuring its own arithmetic.
    let stamp = report.recorded_at as f64;
    let window = explain::Window::of(&report);

    println!("report      {}", report.key);
    println!("request     {}", report.request.as_deref().unwrap_or("—"));
    println!("wall time   {} us", report.wall_time_us);
    println!("exec_ts     {stamp:.0}");
    println!(
        "host start  {started:.3}  (exec_ts - start = {:+.3}s)",
        stamp - started
    );
    println!(
        "host end    {finished:.3}  (exec_ts - end   = {:+.3}s)",
        stamp - finished
    );
    println!("window      {:.3} … {:.3}", window.from, window.to);

    if report.wall_time_us < SEPARABLE_US {
        println!(
            "\nINCONCLUSIVE — the run took {:.3}s, and exec_ts is whole seconds, so \
             both readings fit.\nPoint this at a page that takes more than two \
             seconds and run it again.",
            report.wall_time_us as f64 / 1e6
        );
        std::process::exit(2);
    }

    // Which end is it nearer? With a run longer than the stamp's own
    // resolution, only one of the two can be within a second.
    let near_start = (stamp - started).abs() <= explain::TOLERANCE;
    let near_end = (stamp - finished).abs() <= explain::TOLERANCE;

    match (near_start, near_end) {
        (true, false) => println!(
            "\nCONFIRMED — exec_ts is the start of the run, which is what \
             explain::Window assumes."
        ),
        (false, true) => println!(
            "\nCONTRADICTED — exec_ts is the END of the run. explain::Window \
             places this request's window one whole duration too late, and every \
             statement it joins belongs to whatever ran next. Window::of has to \
             subtract the duration instead of adding it."
        ),
        _ => println!(
            "\nUNSETTLED — exec_ts is near neither end. Either the container's \
             clock differs from this host's, or the field is not a wall-clock \
             stamp at all. Compare `date +%s` here against `date +%s` inside the \
             container before changing anything in explain.rs."
        ),
    }
}

/// The host clock, in the same unit and epoch php-spx writes.
fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
