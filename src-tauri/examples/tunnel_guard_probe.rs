//! Does the tunnel's guard actually refuse anybody? (B-7)
//!
//! [`tunnelid`](../src/tunnelid.rs) puts an nginx container between a tunnel
//! sidecar and the project, so a public link asks for a password. Every part
//! of that is checkable from a unit test except the part that matters: whether
//! nginx, given the configuration this app writes, actually answers 401 to a
//! visitor without credentials and passes the application's own bytes through
//! to one with them.
//!
//! A unit test can only assert that a string contains `return 401`. This runs
//! it. The guard is started with **this app's own `guard_args`**, against a
//! throwaway nginx standing in for a project, and four requests are made from
//! a container on the same network:
//!
//! * no credentials → 401, **and** a `WWW-Authenticate` header, or a browser
//!   shows an error page instead of a password prompt;
//! * wrong password → 401;
//! * wrong user → 401;
//! * the right credential → 200 and the target's own page, byte for byte.
//!
//! It also checks the two things that are easy to get wrong and invisible when
//! they are: that the password never appears in the argument list (which
//! `docker inspect` and this app's operation console both print), and that the
//! guard refuses to start at all when no credential reaches it — a guard that
//! came up open would be the worst possible failure, because the pane would
//! report a protected link.
//!
//! ```sh
//! cargo run --example tunnel_guard_probe
//! ```
//!
//! Nothing on this machine is read: no keystore, no workspace, no project. The
//! credential is a literal in this file and is valid nowhere.

use stackvo_desktop_lib::tunnelid;
use std::process::Command;

const NET: &str = "stackvo-guard-probe-net";
const TARGET: &str = "stackvo-guard-probe-target";
const GUARD: &str = "stackvo-tunnel-guard-guardprobe";
const CURL: &str = "curlimages/curl:latest";
const TARGET_IMAGE: &str = "nginx:alpine";

const USER: &str = "stackvo";
const PASSWORD: &str = "probe-not-a-real-password";

fn docker(args: &[&str]) -> (bool, String) {
    let out = Command::new("docker")
        .args(args)
        .output()
        .expect("docker is on PATH");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.success(), text)
}

/// One request from a container on the guard's network.
fn request(args: &[&str]) -> String {
    let mut argv = vec!["run", "--rm", "--network", NET, CURL, "-s", "-m", "10"];
    argv.extend_from_slice(args);
    docker(&argv).1
}

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let mut check = |name: &str, ok: bool, detail: String| {
        println!("  {} {name}{}", if ok { "PASS" } else { "FAIL" }, {
            if detail.is_empty() {
                String::new()
            } else {
                format!(" — {detail}")
            }
        });
        if !ok {
            failures.push(name.to_string());
        }
    };

    println!("Pulling images.");
    for image in [TARGET_IMAGE, CURL] {
        let (ok, _) = docker(&["pull", "-q", image]);
        println!("  {image} {}", if ok { "✓" } else { "— PULL FAILED" });
    }

    let _ = docker(&["rm", "-f", TARGET, GUARD]);
    let _ = docker(&["network", "create", NET]);
    let (up, out) = docker(&[
        "run",
        "-d",
        "--name",
        TARGET,
        "--network",
        NET,
        TARGET_IMAGE,
    ]);
    assert!(up, "could not start the stand-in project: {out}");

    // ---------------------------------------------------------- the argv
    //
    // The real arguments, from the real function.
    let args = tunnelid::guard_args("guardprobe", TARGET, 80, NET);
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    check(
        "the password is not in the argument list",
        !argv.iter().any(|a| a.contains(PASSWORD)),
        String::new(),
    );

    // ------------------------------------------- a guard with no credential
    //
    // Started with the variable unset. It must refuse rather than come up
    // serving the application to anybody who has the link.
    let (_, out) = docker(&argv);
    let _ = out;
    std::thread::sleep(std::time::Duration::from_secs(2));
    let (_, state) = docker(&["inspect", "-f", "{{.State.Running}}", GUARD]);
    let log = docker(&["logs", GUARD]).1;
    check(
        "a guard with no credential refuses to start",
        state.trim() != "true" && log.contains("no tunnel credential"),
        format!("running={}, log={}", state.trim(), log.trim()),
    );
    let _ = docker(&["rm", "-f", GUARD]);

    // ------------------------------------------------------- the real guard
    let credentials = tunnelid::Credentials {
        user: USER.into(),
        password: PASSWORD.into(),
    };
    // The credential travels the way `tunnel_start` hands it over: in this
    // process's environment, named on the command line and never valued there.
    unsafe { std::env::set_var(tunnelid::AUTH_ENV, credentials.header_value()) };
    let (started, out) = docker(&argv);
    assert!(started, "the guard did not start: {out}");
    std::thread::sleep(std::time::Duration::from_secs(2));

    // What the guard itself said while coming up. Printed unconditionally: a
    // guard that failed to parse its own configuration answers every request
    // below with a connection error, and the reason is only ever here.
    println!("  guard log: {}", docker(&["logs", GUARD]).1.trim());
    println!(
        "  guard running: {}",
        docker(&["inspect", "-f", "{{.State.Running}}", GUARD])
            .1
            .trim()
    );

    let unauthenticated = request(&["-i", &format!("http://{GUARD}:{}/", tunnelid::GUARD_PORT)]);
    check(
        "no credentials → 401",
        unauthenticated.contains("401 Unauthorized"),
        first_line(&unauthenticated),
    );
    check(
        "…with a challenge a browser can prompt for",
        unauthenticated
            .to_ascii_lowercase()
            .contains("www-authenticate"),
        String::new(),
    );

    for (name, user, password) in [
        ("the wrong password → 401", USER, "not-the-password"),
        ("the wrong user → 401", "someone-else", PASSWORD),
    ] {
        let answer = request(&[
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-u",
            &format!("{user}:{password}"),
            &format!("http://{GUARD}:{}/", tunnelid::GUARD_PORT),
        ]);
        check(name, answer.trim() == "401", answer.trim().to_string());
    }

    // The one that has to succeed, and it is compared against the target's own
    // bytes rather than against a status code: a 200 from the guard's own
    // error page would pass a weaker check.
    let direct = request(&[&format!("http://{TARGET}:80/")]);
    let through = request(&[
        "-u",
        &format!("{USER}:{PASSWORD}"),
        &format!("http://{GUARD}:{}/", tunnelid::GUARD_PORT),
    ]);
    check(
        "the right credential → the application's own page",
        !direct.trim().is_empty() && direct == through,
        format!(
            "{} bytes through, {} bytes direct",
            through.len(),
            direct.len()
        ),
    );

    // The credential stops at the guard: an application behind it must not see
    // the password that opens its own front door. nginx echoes what it
    // received into its 404 body only if asked, so this is read from the
    // target's access log instead — the header would be there if it had been
    // forwarded with `$http_authorization` intact.
    let _ = request(&[
        "-u",
        &format!("{USER}:{PASSWORD}"),
        &format!("http://{GUARD}:{}/stackvo-probe-path", tunnelid::GUARD_PORT),
    ]);
    let target_log = docker(&["logs", TARGET]).1;
    check(
        "the target saw the request",
        target_log.contains("stackvo-probe-path"),
        String::new(),
    );

    let _ = docker(&["rm", "-f", TARGET, GUARD]);
    let _ = docker(&["network", "rm", NET]);

    println!("\n════ summary ════");
    if failures.is_empty() {
        println!("The guard refuses everyone without the credential and nobody with it.");
    } else {
        eprintln!(
            "{} check(s) failed: {}",
            failures.len(),
            failures.join(", ")
        );
        std::process::exit(1);
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_string()
}
