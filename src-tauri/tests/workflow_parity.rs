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

/// The container is at least the machine the release job bundles on.
///
/// This is the check whose absence turned a release run into a test
/// environment, and it cost exactly one run to learn.
///
/// `tools/linux/Dockerfile` said it was kept identical to `ci.yml`'s package
/// list, and it was. But `ci.yml` compiles and tests; it never **bundles**, and
/// the bundler is a different program with different needs — it runs
/// `linuxdeploy`, it shells out to `dpkg-deb`, and it copies `/usr/bin/xdg-open`
/// off the build machine. So the one list the image was not being held against
/// was the only list that describes the job it exists to rehearse.
///
/// The direction is the same one this file already argues for CI and the
/// release, one link further along: **the container must be at least the
/// release environment.** More is fine — the image carries `xvfb`, a driver and
/// `cargo-xwin`, none of which a runner needs.
#[test]
fn the_local_container_installs_every_linux_package_the_release_installs() {
    let release = apt_packages(&suite_job(&read(".github/workflows/release.yml")));
    let image = apt_packages(&read("tools/linux/Dockerfile"));

    assert!(
        release.len() > 3,
        "the scan found {} packages in release.yml's build job, so it stopped \
         reading `apt-get install` lines rather than that the release stopped \
         needing them",
        release.len()
    );

    let missing: Vec<&String> = release.difference(&image).collect();
    assert!(
        missing.is_empty(),
        "release.yml installs {missing:?} and tools/linux/Dockerfile does not.\n\n\
         That image is where `tools/linux/run.sh --bundle` answers what the \
         release job answers, and a package present there and absent here means \
         the local run is a rehearsal of a different machine. It has happened: \
         `xdg-utils` was in neither, the AppImage bundler copies \
         /usr/bin/xdg-open out of the build machine, and the failure was found \
         on a runner after the .deb and the .rpm were already written.\n\n\
         The container must be a superset of the release environment, never less."
    );
}

/// The bundling half is answerable without a runner.
///
/// §3 #22 spent a round using a release run as a test environment, and the
/// reason was narrow: every mode in `tools/linux/run.sh` compiled or tested,
/// none of them bundled. On an Apple Silicon machine that container **is**
/// `aarch64-unknown-linux-gnu` — the row that failed — so the failure was
/// always reproducible locally and no command existed to reproduce it.
///
/// `before-push.sh` claims in its first line to ask everything CI asks. That
/// claim is the thing being guarded: it has been false twice, and the file's own
/// comment says why that is worse than absent — people stop reading the runs.
#[test]
fn the_bundle_can_be_built_and_judged_without_a_runner() {
    let runner = read("tools/linux/run.sh");
    for mode in ["--bundle", "--windows-bundle"] {
        assert!(
            runner.contains(&format!("\"${{1:-}}\" = \"{mode}\"")),
            "tools/linux/run.sh has no `{mode}` mode. Then the only place the \
             bundler is ever exercised is a release run, and a release run is \
             not a test environment: it costs twenty minutes on six machines \
             and reports through a screenshot."
        );
    }
    assert!(
        runner.contains("check-installers.mjs"),
        "the local bundle is built and never judged. Then it answers 'it did \
         not crash', and what #22 asks is which formats came out and whether \
         they are named for this architecture."
    );

    let before_push = read("tools/before-push.sh");
    assert!(
        before_push.contains("--bundle"),
        "`tools/before-push.sh --all` does not build a bundle. Its first line \
         says everything CI will ask is asked here first, and the release job \
         asks for a bundle — a claim that is false is worse than absent, which \
         is that file's own argument for existing."
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

    // And it has to be keyed on the suite rather than on the job. A rehearsal
    // runs the suite with `continue-on-error`, so that a red suite still lets
    // the run answer §3 #22's question about the bundler — and while that is
    // true the job status is not `failure()`. An upload written `if: failure()`
    // would skip on exactly the run whose log is hardest to reach: the one
    // where the job goes on for another fifteen minutes afterwards.
    let at = workflow
        .find("- name: Keep the test output")
        .expect("checked above");
    let step = &workflow[at..];
    let step = &step[..step.find("\n      - ").unwrap_or(step.len())];
    assert!(
        step.contains("steps.suite.outcome == 'failure'"),
        "the test log is uploaded on the job's status rather than the suite \
         step's. In a rehearsal the suite is `continue-on-error`, so the job is \
         not failing when this runs and `failure()` skips it:\n{step}"
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

/// The working tree is LF, and stays LF.
///
/// A third thing that only fails on a runner. Git for Windows ships with
/// `core.autocrlf=true` and the Actions Windows runner inherits it, so without
/// `.gitattributes` every test that reads this repository's own source reads a
/// file with `\r\n` in it that it has never seen on the machine it was written
/// on.
///
/// It is not hypothetical and it was not cheap: `cfg_regions.rs` finds which
/// attributes belong to which function by splitting on a blank line, `"\n\n"`
/// does not occur inside `"\r\n\r\n"`, and the search window silently reached
/// back into the function above — so the keystore's real backend and its
/// in-memory fake were reported as carrying the same `cfg` gate. That is a
/// security-shaped assertion failing for a reason that has nothing to do with
/// security.
///
/// Deleting `.gitattributes` is a one-line change that passes everywhere except
/// there, which is exactly the shape of thing this file exists for.
#[test]
fn the_checkout_is_lf_on_every_platform() {
    let path = repo_root().join(".gitattributes");
    let text = std::fs::read_to_string(&path).unwrap_or_default();

    assert!(
        text.contains("eol=lf"),
        ".gitattributes does not pin `eol=lf`. Without it a Windows checkout \
         is CRLF, and every test that reads this repository's own source is \
         reading a file it has never seen — including the ones that split on a \
         blank line to decide which attribute belongs to which function."
    );
}

/// Both suites report everything they found, not the first thing.
///
/// `cargo test` stops at the first test BINARY that fails. On a runner that is
/// the difference between one round and four: a Windows run reports the
/// alphabetically-first broken file and stays silent about what is behind it,
/// so nineteen failures were counted three times and were never nineteen —
/// fixing `agent_install` uncovered a twentieth that had been sitting behind
/// it, waiting its turn, for a full round.
///
/// The cost is the wall-clock of finishing a suite that is already failing. The
/// cost of not paying it is a person watching a ten-minute job to be told one
/// thing.
#[test]
fn both_workflows_run_the_suite_to_the_end() {
    for file in [".github/workflows/ci.yml", ".github/workflows/release.yml"] {
        let workflow = read(file);
        let job = suite_job(&workflow);
        assert!(
            job.contains("cargo test --no-fail-fast"),
            "{file} runs `cargo test` without `--no-fail-fast`, so it stops at \
             the first test binary that fails and reports one failure however \
             many there are. That is how this repository spent four rounds \
             learning about nineteen Windows failures one at a time."
        );
    }
}
