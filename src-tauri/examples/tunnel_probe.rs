//! How far can eight tunnel providers be checked without eight accounts?
//!
//! The Share pane offers eight providers and four of them want a token nobody
//! here has. That is the same shape as [`stripe_probe`](stripe_probe.rs), and
//! it gets the same answer: half of "it cannot be verified" is true and half
//! of it is an excuse, and this probe is the line between them.
//!
//! **What it verifies for real, for every provider:**
//!
//! * the image runs and the tunnel client is the program inside it;
//! * the arguments [`tunnel::run_args`] builds are arguments that client
//!   accepts — a flag it has removed, a subcommand it has renamed or a target
//!   shape it refuses all show up here as a usage error rather than as a pane
//!   that spins for ever;
//! * for the four anonymous ones, that a real public URL comes back and that
//!   [`tunnel::find_url`] picks it out of the client's own banner;
//! * for the four that need a token, that a deliberately invalid one produces
//!   a complaint [`tunnel::find_failure`] recognises — so the pane says "that
//!   token was rejected" instead of leaving a spinner running.
//!
//! **What it cannot verify, and does not pretend to:** that a tunnel opened
//! with a *real* account carries traffic. That needs four accounts, and no
//! assertion written here stands in for one. What it does mean is that when
//! somebody pastes a real token in, the only untested step left is the
//! provider's own answer to it.
//!
//! ```sh
//! cargo run --example tunnel_probe             # every provider
//! cargo run --example tunnel_probe -- ngrok    # one of them
//! ```
//!
//! The tokens used are literal strings like `probe_invalid_token`; they are
//! not credentials and are valid nowhere. Nothing on this machine is read —
//! in particular the keystore is not touched, and neither is any project.

use stackvo_desktop_lib::tunnel::{self, Provider};
use std::process::Command;

/// Everything the probe makes, named so it cannot collide with a real tunnel.
const NET: &str = "stackvo-tunnel-probe-net";
const TARGET: &str = "stackvo-probe";
const SIDECAR: &str = "stackvo-tunnel-probe";
const TARGET_IMAGE: &str = "nginx:alpine";

/// How long a client gets to say something.
///
/// A quick tunnel answers in seconds and a rejected token faster still, but
/// localtunnel has no published image and fetches its client at start — so the
/// slowest honest case is a package download, not a handshake. An image pull
/// is waited for separately.
const WAIT_SECONDS: u64 = 90;

fn docker(args: &[&str]) -> (bool, String) {
    let out = Command::new("docker")
        .args(args)
        .output()
        .expect("docker is on PATH");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), text)
}

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();
    let providers: Vec<&Provider> = tunnel::PROVIDERS
        .iter()
        .filter(|p| only.is_empty() || only.iter().any(|o| o == p.id))
        .collect();
    assert!(!providers.is_empty(), "no provider matched {only:?}");

    println!("Pulling images. The first run of this is the slow one.");
    for image in std::iter::once(TARGET_IMAGE).chain(providers.iter().map(|p| p.image)) {
        let (ok, _) = docker(&["pull", "-q", image]);
        println!("  {image} {}", if ok { "✓" } else { "— PULL FAILED" });
    }

    // A network and a target of our own: the probe must not touch a project.
    let _ = docker(&["network", "create", NET]);
    let _ = docker(&["rm", "-f", TARGET]);
    let (up, out) = docker(&[
        "run",
        "-d",
        "--name",
        TARGET,
        "--network",
        NET,
        TARGET_IMAGE,
    ]);
    assert!(up, "could not start the probe target: {out}");

    let mut results: Vec<(String, bool, String)> = Vec::new();

    for provider in &providers {
        println!("\n─── {} ─────────────────────────────", provider.id);
        let verdict = probe(provider);
        println!("  {}", verdict.1);
        results.push((provider.id.to_string(), verdict.0, verdict.1));
    }

    let _ = docker(&["rm", "-f", TARGET]);
    let _ = docker(&["network", "rm", NET]);

    println!("\n════ summary ════");
    for (id, ok, line) in &results {
        println!("  {:<14} {}  {line}", id, if *ok { "PASS" } else { "FAIL" });
    }

    let failed = results.iter().filter(|(_, ok, _)| !ok).count();
    if failed > 0 {
        eprintln!(
            "\n{failed} provider(s) did not behave as `tunnel.rs` claims. The invocation \
             in the table is wrong, or the client changed under it."
        );
        std::process::exit(1);
    }
    println!("\nEvery provider accepted its arguments and was understood.");
}

/// Run one provider's real argument list and read what came back.
fn probe(provider: &Provider) -> (bool, String) {
    let _ = docker(&["rm", "-f", SIDECAR]);

    // The real arguments, from the real function. Only the container name is
    // changed, so a probe cannot collide with a tunnel somebody is using.
    // A reserved name was added to the plan, and the probe sends one wherever
    // the provider can take it: a flag the client has removed or renamed shows
    // up here as a usage error, which is the whole reason this file exists.
    let reserved = provider.reserved.map(|shape| {
        if shape.dotted {
            "stackvo-probe.example.com".to_string()
        } else {
            "stackvo-probe".to_string()
        }
    });
    let mut args = tunnel::run_args(
        provider,
        &tunnel::Plan {
            project: "probe",
            domain: Some("probe.loc"),
            port: 80,
            network: NET,
            reserved: reserved.as_deref(),
            guard: None,
        },
    );
    for arg in args.iter_mut() {
        if arg == "stackvo-tunnel-probe" {
            *arg = SIDECAR.to_string();
        }
    }

    // The invalid token, put in this process's environment so the container
    // inherits it exactly the way `runner::run_operation` hands over a real
    // one — and so the argv below stays free of credentials.
    if let Some(var) = provider.token_env {
        unsafe { std::env::set_var(var, "probe_invalid_token") };
    }

    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    println!("  argv: docker {}", argv.join(" "));
    assert!(
        !argv.iter().any(|a| a.contains("probe_invalid_token")),
        "the token reached the argument list, which the operation console prints"
    );

    let (started, out) = docker(&argv);
    if !started {
        return (false, format!("docker refused to start it: {}", out.trim()));
    }

    // Poll rather than sleep the whole way: a quick tunnel answers in seconds
    // and a rejected token answers faster still.
    let mut log = String::new();
    for _ in 0..WAIT_SECONDS / 3 {
        std::thread::sleep(std::time::Duration::from_secs(3));
        log = docker(&["logs", SIDECAR]).1;
        let done =
            tunnel::find_url(provider, &log).is_some() || tunnel::find_failure(&log).is_some();
        if done {
            break;
        }
    }

    let url = tunnel::find_url(provider, &log);
    let failure = tunnel::find_failure(&log);
    let usage = usage_error(&log);

    let (state, _) = docker(&["inspect", "-f", "{{.State.Status}}", SIDECAR]);
    let _ = state;
    let _ = docker(&["rm", "-f", SIDECAR]);

    // A usage error is the finding this probe exists for: the client is there,
    // it read the arguments, and it does not accept them.
    if let Some(line) = usage {
        return (false, format!("the client rejected its arguments: {line}"));
    }

    if provider.anonymous {
        match (url, failure) {
            (Some(url), _) => (true, format!("public URL assigned: {url}")),
            // The provider said no, in words the pane will show. That is not
            // this repository being wrong — Pinggy's free tier refuses a
            // fourth anonymous tunnel from one address, and a probe that
            // called that a defect would send somebody to fix working code.
            (None, Some(line)) => (
                true,
                format!("no tunnel, and the provider said why: {line}"),
            ),
            (None, None) => (
                false,
                format!(
                    "silence for {} seconds — no URL and nothing `find_failure` \
                     recognises, which is the one outcome the pane cannot explain. \
                     Last lines: {}",
                    WAIT_SECONDS,
                    tail(&log)
                ),
            ),
        }
    } else {
        match failure {
            Some(line) => (
                true,
                format!("invalid token refused, and the pane can say so: {line}"),
            ),
            None => (
                false,
                format!(
                    "an invalid token produced nothing `find_failure` recognises. Last lines: {}",
                    tail(&log)
                ),
            ),
        }
    }
}

/// A client complaining about the command line rather than about a credential.
///
/// Kept apart from [`tunnel::find_failure`] on purpose: the pane's job is to
/// show a user why their tunnel did not open, and this probe's job is to catch
/// the case where the reason is *this repository's* argument list.
fn usage_error(log: &str) -> Option<String> {
    const NEEDLES: &[&str] = &[
        "unknown flag",
        "unknown command",
        "flag provided but not defined",
        "incorrect usage",
        "unknown shorthand",
        "invalid argument",
        "accepts at most",
        "unrecognized option",
        "executable file not found",
    ];
    log.lines()
        .find(|line| {
            let lowered = line.to_ascii_lowercase();
            NEEDLES.iter().any(|n| lowered.contains(n))
        })
        .map(|line| line.trim().to_string())
}

fn tail(log: &str) -> String {
    let lines: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(4);
    lines[start..].join(" ⏎ ")
}
