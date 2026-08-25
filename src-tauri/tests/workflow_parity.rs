//! Where CI is green and the release is red on the same commit.
//!
//! §3 #2 spent three rounds on a sentence that was wrong: the tag ran, **six of
//! six targets failed at `Verify the contract surface`**, and the remaining
//! work is item W — Windows tests — going green.
//!
//! Nobody had opened the logs. When they were opened the six had **three
//! different causes**, and only one of them was W: the two macOS rows failed on
//! `key_ceremony`, the two Linux rows on `elevate_probe`, the two Windows rows
//! on W's platform assumptions. Three of those were fixed the same evening in
//! commit `c8ec131`, which landed *after* the tag — so the release had been
//! waiting on Windows while two thirds of its failures were already repaired
//! and nobody knew, because a release job's log is six live browser tabs.
//!
//! Two things follow, and this file is one of them. The other is in
//! `release.yml`: the suite runs `--no-fail-fast` and its output is kept.
//!
//! ## What this file guards
//!
//! The release job is the only place in this repository that runs the suite in
//! an environment nobody can reproduce locally. So a package CI installs and
//! the release does not is invisible until a tag is spent finding it — and the
//! list had already drifted: `libdbus-1-dev` was named in CI and not here. That
//! one was harmless, because `libappindicator3-dev` pulls it in anyway. It is
//! guarded for the reason it was harmless: nothing noticed, and nothing would
//! have noticed the next one either.
//!
//! ## The rule, and its direction
//!
//! CI is the environment proven green on every push. The release must be **at
//! least** that environment, never less. The reverse is allowed: the release
//! installs and builds things CI has no reason to — real sidecars, signing
//! certificates, a bundler.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every package named on an `apt-get install` line, across a whole workflow.
///
/// Line-based rather than YAML-aware, and the continuation backslash is the
/// only structure it needs: these are `run: |` blocks, so to a YAML parser they
/// are one opaque string and the work would be the same.
///
/// Read from **one job**, never from a whole file — see [`suite_job`]. The
/// first version of this collected across every job and failed on
/// `webkit2gtk-driver` and `xvfb`, which CI's `driver` job needs to drive a
/// window and the release job has no window to drive. A parity check that
/// demands a superset of everything a repository ever installs is a check that
/// gets an exemption list, and an exemption list is where the real one ends up.
fn apt_packages(workflow: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut collecting = false;

    for line in workflow.lines() {
        let trimmed = line.trim();
        if !collecting && !trimmed.contains("apt-get install") {
            continue;
        }

        // Already inside a continued command, or at the head of a new one:
        // either way what follows is a list of packages. `collecting` is set
        // once, at the bottom, from whether this line ends in a backslash.
        let body = if collecting {
            trimmed
        } else {
            trimmed
                .split_once("apt-get install")
                .map(|(_, rest)| rest)
                .unwrap_or("")
        };

        let more = body.ends_with('\\');
        for word in body.trim_end_matches('\\').split_whitespace() {
            if word.starts_with('-') {
                continue; // -y, and any other flag
            }
            found.insert(word.to_string());
        }
        collecting = more;
    }

    found
}

/// The job that runs the test suite, out of a workflow that has several.
///
/// Jobs are the only keys at two spaces of indentation under `jobs:`, which is
/// enough structure to split on without a YAML parser — the same trade
/// `release_rehearsal.rs` makes for steps, and for the same reason: the
/// alternative is a dependency that has to be kept in step with what the runner
/// actually accepts.
///
/// The comparison has to be per job because a workflow's jobs are different
/// machines. Only the one that runs `cargo test` is answering the same question
/// in both files, and it is the only one whose environment they must share.
fn suite_job(workflow: &str) -> String {
    let mut jobs: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in workflow.lines() {
        let is_job_key = line.starts_with("  ")
            && !line.starts_with("   ")
            && line.trim_end().ends_with(':')
            && !line.trim_start().starts_with('#');
        if is_job_key && !current.is_empty() {
            jobs.push(std::mem::take(&mut current));
        }
        if is_job_key || !current.is_empty() {
            current.push_str(line);
            current.push('\n');
        }
    }
    jobs.push(current);

    let running: Vec<String> = jobs
        .into_iter()
        .filter(|job| job.contains("cargo test"))
        .collect();

    assert_eq!(
        running.len(),
        1,
        "expected exactly one job to run `cargo test`; found {}. This file \
         compares the environments of the two jobs that run the same suite, and \
         it cannot do that if it cannot tell which they are.",
        running.len()
    );
    running.into_iter().next().unwrap()
}

/// The release runs the suite in at least the environment CI proves green.
#[test]
fn the_release_installs_every_linux_package_ci_installs() {
    let ci = apt_packages(&suite_job(&read(".github/workflows/ci.yml")));
    let release = apt_packages(&suite_job(&read(".github/workflows/release.yml")));

    assert!(
        ci.len() > 3,
        "the scan found {} packages in ci.yml's test job, so it stopped reading \
         `apt-get install` lines rather than that CI stopped needing them",
        ci.len()
    );

    let missing: Vec<&String> = ci.difference(&release).collect();
    assert!(
        missing.is_empty(),
        "ci.yml installs {missing:?} and release.yml does not.\n\n\
         The release job runs the same `cargo test` in an environment nobody \
         can reproduce off a runner, so a header CI has and the release lacks \
         does not show up until a tag is spent finding it — and a tag costs a \
         version number and twenty minutes on six machines.\n\n\
         The release environment must be a superset of CI's, never less."
    );
}

/// A failed release leaves its failures behind, in a file.
///
/// What §3 #2 actually recorded after the first real run was "how many failures
/// remain was not read from the log". Six live logs in a browser is why. The
/// suite runs with `--no-fail-fast` so the log reaches the end rather than
/// stopping at the first crate, and the log is uploaded when the step fails.
#[test]
fn a_failed_release_run_keeps_the_test_output() {
    let workflow = read(".github/workflows/release.yml");

    assert!(
        workflow.contains("--no-fail-fast"),
        "the release runs `cargo test` without `--no-fail-fast`, so its log \
         ends at the first test binary that fails and the run cannot say how \
         much else is wrong"
    );
    assert!(
        workflow.contains("PIPESTATUS") || !workflow.contains("cargo test --no-fail-fast 2>&1 |"),
        "the test output is piped to `tee`, and a pipe reports `tee`'s exit \
         code. Without `PIPESTATUS` this step goes green on a failing suite — \
         worse than the failure it was added to describe"
    );
    assert!(
        workflow.contains("name: Keep the test output"),
        "nothing uploads the test log when the release suite fails, so reading \
         six red targets means opening six live logs in a browser"
    );
}

/// The suite runs where the toolchain pin is.
///
/// `rust-toolchain.toml` lives in `src-tauri/` and rustup resolves it from the
/// **working directory**, so a cargo run started at the repository root takes
/// whatever `stable` meant on the runner that morning. `ci.yml` documents the
/// trap twice; this is the file that would walk into it silently, because
/// nobody runs it locally.
#[test]
fn the_release_runs_the_suite_from_the_pinned_toolchains_directory() {
    let workflow = read(".github/workflows/release.yml");
    let at = workflow
        .find("- name: Verify the contract surface")
        .expect("release.yml still verifies the contract surface before it builds a bundle");
    let step = &workflow[at..];
    let step = &step[..step.find("\n      - ").unwrap_or(step.len())];

    assert!(
        step.contains("working-directory: src-tauri"),
        "the release runs its suite from the repository root, where there is no \
         `rust-toolchain.toml` — so the release is gated by a toolchain no \
         developer is running:\n{step}"
    );
}
