//! A rehearsal publishes nothing, and stays that way.
//!
//! The two ARM rows could not be verified because no tag had ever
//! been pushed, and no tag could be pushed because the signing preflight fails
//! without a secret that is somebody's decision to add. The `rehearsal` input
//! cut that knot: it builds all six targets, runs the suite on each, drops the
//! bundles on the run page, and publishes nothing.
//!
//! "Publishes nothing" is the whole value of it, and it is the part that erodes.
//! A rehearsal that quietly opened a draft release would be worse than no
//! rehearsal — it is *run from a branch*, so the release would be named after
//! whatever branch somebody was on. That already happened once: before the
//! input existed, a manual run passed `tagName: ${{ github.ref_name }}` and
//! drafted a release called after a feature branch.
//!
//! ## Why this is a text test and not a run
//!
//! GitHub Actions cannot be run here — that is the whole of what is left in
//! #22, and this file does not pretend otherwise. What it holds is the claim
//! that survives without a runner: **every step that could publish is gated.**
//! A new `gh release create` or a second upload action added without a gate
//! fails here, on the machine of whoever added it, rather than on the first
//! rehearsal after it.
//!
//! `updater_endpoint.rs` reads this file the same way and for the same reason.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn workflow() -> String {
    let path = repo_root().join(".github/workflows/release.yml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// One step: the lines from its `- name:`/`- uses:` to the next step's.
///
/// Coarse on purpose. The question is which `if:` governs which action, and a
/// block that ran a couple of lines long would only make this test *more*
/// willing to call a step gated — so the sloppiness is checked against, below,
/// by asserting the gated steps are the ones actually expected.
///
/// One thing it is not sloppy about, because in this file the comments *are*
/// the reasoning: a run of comment lines is held back and given to the step it
/// introduces rather than to the one it follows. Attached the naive way, a
/// paragraph explaining why the next step exists reads as part of the previous
/// step — and a test looking for the step that runs a command finds the step
/// above it, whose comment merely names it.
fn steps(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut pending = String::new();

    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with("- ") && indent >= 6 {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            current.push_str(&std::mem::take(&mut pending));
        } else if trimmed.is_empty() || trimmed.starts_with('#') {
            // Held: it belongs to whichever comes next, a step or more of this
            // one.
            pending.push_str(line);
            pending.push('\n');
            continue;
        } else {
            current.push_str(&std::mem::take(&mut pending));
        }

        current.push_str(line);
        current.push('\n');
    }

    current.push_str(&pending);
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Actions and commands that put something where the public can reach it.
///
/// Named as needles rather than as a list of step names, because a step can be
/// renamed and the thing it runs cannot.
const PUBLISHES: [&str; 4] = [
    "actions/attest-build-provenance",
    "softprops/action-gh-release",
    "gh release create",
    "gh release upload",
];

/// Nothing that publishes runs in a rehearsal.
#[test]
fn every_publishing_step_is_gated_on_this_not_being_a_rehearsal() {
    let text = workflow();
    let mut ungated = Vec::new();
    let mut gated = 0usize;

    for step in steps(&text) {
        let publishes = PUBLISHES.iter().any(|needle| step.contains(needle));
        if !publishes {
            continue;
        }
        if step.contains("!inputs.rehearsal") {
            gated += 1;
        } else {
            let label = step
                .lines()
                .find(|l| l.contains("name:") || l.contains("uses:"))
                .unwrap_or("<unnamed step>")
                .trim()
                .to_string();
            ungated.push(label);
        }
    }

    assert!(
        gated > 0,
        "no publishing step is gated on `!inputs.rehearsal` — either the \
         needles in PUBLISHES no longer match anything in release.yml, in \
         which case this test is checking nothing, or the gating is gone"
    );
    assert!(
        ungated.is_empty(),
        "{} step(s) can publish during a rehearsal:\n{}\n\nA rehearsal runs \
         from a BRANCH. Anything it publishes is named after that branch, \
         which is how a draft release called after a feature branch got \
         created before the `rehearsal` input existed.",
        ungated.len(),
        ungated
            .iter()
            .map(|l| format!("  {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// `tauri-action` is gated by its arguments rather than by an `if:`.
///
/// It is the step that *builds*, so it has to run in a rehearsal — an empty
/// `tagName` is what makes it build without touching a release. The expression
/// is the fragile part and the comment above it in the workflow says why:
/// written the obvious way round, `${{ inputs.rehearsal && '' || github.ref_name }}`,
/// the empty string is falsy, `||` fires, and the rehearsal publishes under the
/// branch name — the exact bug the input was added to prevent, reintroduced by
/// a tidier-looking line.
#[test]
fn the_builder_is_told_not_to_publish_by_an_expression_with_no_falsy_trap() {
    let text = workflow();
    assert!(
        text.contains("tauri-apps/tauri-action"),
        "the builder step is gone, and this test is about how it is configured"
    );

    for field in ["tagName", "releaseName"] {
        let line = text
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{field}:")))
            .unwrap_or_else(|| panic!("release.yml no longer sets `{field}`"));

        assert!(
            line.contains("!inputs.rehearsal &&"),
            "`{field}` is `{}`. It must be written `!inputs.rehearsal && <value> || ''` \
             — the other order has a falsy left operand on the publishing path, \
             so `''` makes `||` fire and the rehearsal publishes under the \
             branch name.",
            line.trim()
        );
        assert!(
            line.trim_end().ends_with("|| '' }}"),
            "`{field}` does not fall through to an empty string, which is what \
             makes tauri-action build without touching a release",
        );
    }
}

/// A rehearsal is worth running only if it leaves the bundles behind.
///
/// The two ARM rows are the reason the input exists: a `.deb` and an `.msi`
/// that exist are the only proof that the runner label resolved and the
/// bundler ran there. Without the upload the run is green and produces nothing
/// to look at, which answers a different question than the one #22 asks.
///
/// And it has to survive a red job. Two steps in this file are now allowed to
/// fail after the bundles exist — the suite, which is `continue-on-error` in a
/// rehearsal, and the check on what was produced — so an upload carrying the
/// implied `success()` would skip on precisely the runs worth reading.
#[test]
fn a_rehearsal_keeps_what_it_built() {
    let text = workflow();
    let keeps = steps(&text)
        .into_iter()
        .find(|step| step.contains("actions/upload-artifact") && step.contains("inputs.rehearsal"))
        .expect(
            "no artifact upload runs in a rehearsal. Then a rehearsal proves the \
             six targets COMPILED and nothing about whether they bundled — and \
             the bundler is where an ARM runner actually differs.",
        );

    let condition = keeps
        .lines()
        .find(|l| l.trim_start().starts_with("if:"))
        .unwrap_or("")
        .to_string();
    assert!(
        condition.contains("always()"),
        "the rehearsal upload is `{}`. Every `if:` without a status function \
         carries an implied `success()`, and a rehearsal that failed — at the \
         suite, or at the check on what it produced — is the one whose bundles \
         somebody wants.",
        condition.trim()
    );
}

/// The rehearsal builds without a signing key, and says so to the bundler.
///
/// This is the wall the rehearsal would have hit on its first run, having
/// already done everything #22 asks about. `bundle.createUpdaterArtifacts` is
/// true and `plugins.updater.pubkey` is set, so `tauri build` signs the
/// updater-enabled bundles after producing them — reading the key from
/// `TAURI_SIGNING_PRIVATE_KEY`, unconditionally.
///
/// A repository that has not decided where a private key lives does not get
/// tauri's clean "a public key has been found, but no private key" error.
/// Actions sets an absent secret to the **empty string**, which is set, so the
/// guard passes and the run dies decoding a zero-length key instead — after
/// `bundle_project` has already written every installer to disk.
///
/// `--no-sign` is the whole fix, and it is correct rather than a workaround: a
/// rehearsal publishes nothing, so there is nothing for a signature to protect.
#[test]
fn a_rehearsal_tells_the_bundler_not_to_sign() {
    let text = workflow();
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("args:"))
        .expect("release.yml no longer passes `args:` to tauri-action");

    assert!(
        line.contains("--no-sign"),
        "tauri-action is called with `{}`. Without `--no-sign` a rehearsal \
         bundles all six targets and then fails signing them with a key it \
         does not have — at the one step #22 exists to reach, for a reason \
         that has nothing to do with packaging.",
        line.trim()
    );
    assert!(
        line.contains("inputs.rehearsal &&"),
        "`--no-sign` is not conditional on this being a rehearsal: `{}`. A \
         published release must be signed, and an unsigned one is invisible to \
         the updater on the user's machine rather than here.",
        line.trim()
    );
}

/// A rehearsal reaches its bundler even when the suite is red.
///
/// The first real run of this workflow died at the suite on all six targets and
/// reached no bundler at all — so eighteen minutes on `windows-11-arm` taught
/// #22 nothing about the only thing it was asking. The two questions are
/// independent: whether the tests pass there, and whether a package can be
/// produced there.
///
/// The trade has one edge, and this test guards it. `continue-on-error` on its
/// own finishes the job **green**, and a green tick over a failing suite is a
/// worse thing to own than an unanswered question — so something later must
/// read that step's outcome and fail on it.
#[test]
fn a_rehearsal_reaches_its_bundler_even_when_the_suite_fails() {
    let text = workflow();

    let suite = steps(&text)
        .into_iter()
        .find(|step| step.contains("cargo test --no-fail-fast"))
        .expect("release.yml no longer runs the suite");
    assert!(
        suite.contains("id: suite"),
        "the suite step has no `id:`, so nothing downstream can name its \
         outcome — and every gate below depends on being able to"
    );
    assert!(
        suite.contains("continue-on-error: ${{ inputs.rehearsal"),
        "the suite is not `continue-on-error` in a rehearsal, so a red suite \
         stops the job before the bundler runs. That is right for a tag and \
         wrong for a rehearsal, whose only question is downstream of it:\n{suite}"
    );

    let votes = steps(&text)
        .into_iter()
        .any(|step| step.contains("steps.suite.outcome == 'failure'") && step.contains("exit 1"));
    assert!(
        votes,
        "nothing fails the job when the suite failed. `continue-on-error` \
         alone makes a rehearsal with a broken suite finish green, which is a \
         worse claim than the one it was added to avoid."
    );
}

/// The run says what the bundler produced, rather than leaving a zip to read.
///
/// The upload answers "there is a directory"; #22 asks whether each format a
/// platform owes came out of it, and whether what came out is for the
/// architecture the row is named after. `bundle.targets` is `"all"`, so Linux
/// owes a `.deb`, an `.rpm` and an `.AppImage` from three separate bundlers —
/// one of which downloads `linuxdeploy-aarch64.AppImage` and executes it, a
/// step with no equivalent on the x86 rows.
///
/// The judgement half of the checker is tested without a bundler in
/// `tests/installer-formats.spec.js`; this is the half that says it runs.
#[test]
fn the_run_says_what_the_bundler_produced() {
    let text = workflow();
    let step = steps(&text)
        .into_iter()
        .find(|step| step.contains("node tools/check-installers.mjs"))
        .expect(
            "nothing checks what the bundler produced. Then the answer to #22 \
             is a zip file somebody downloads once and reads with their eyes — \
             which is not a check: it is not repeated and it has no verdict.",
        );

    assert!(
        step.contains("--target ${{ matrix.target }}"),
        "the check is not told which target it is looking at, so it cannot say \
         whether an ARM row produced an ARM package — the failure hardest to \
         see from an artifact listing:\n{step}"
    );
    assert!(
        step.contains("inputs.rehearsal && '--unsigned'"),
        "`--unsigned` is not conditional on this being a rehearsal. A rehearsal \
         builds with `--no-sign` so there are no signatures to find; a \
         published release has them, and an artifact without one installs by \
         hand and is invisible to the updater:\n{step}"
    );
    assert!(
        step.contains("!cancelled()"),
        "the check carries an implied `success()`, so it is skipped on a \
         rehearsal whose suite failed — the run where 'did the packaging half \
         work anyway' is the only question left:\n{step}"
    );
    assert!(
        !step.contains("steps.tauri.outcome == 'success'"),
        "the check runs only when the bundler succeeded, and the first \
         rehearsal is what disproved that gate: `ubuntu-24.04-arm` wrote \
         StackVo_0.1.0_arm64.deb and StackVo-0.1.0-1.aarch64.rpm and then \
         failed on the AppImage — the two packages #22 was waiting to see were \
         on disk, and this step skipped itself:\n{step}"
    );
}

/// Every platform's bundler is given an icon in the format it demands.
///
/// The first rehearsal's Windows rows — **both** of them, x86_64 and ARM — died
/// on `Couldn't find a .ico icon`, and the two files were on disk the whole
/// time: `icons/icon.ico` and `icons/icon.icns` are in the repository.
/// `bundle.icon` named only `icons/icon.png`, so the bundler was never told
/// about them.
///
/// It stayed invisible because the failure is asymmetric. macOS *generates* an
/// `.icns` from a PNG when it has to, so two of the three platforms were fine
/// and the third was a hard error — and the third is the one nobody runs. Read
/// off the run, `windows-11-arm` looked like an ARM problem; it had nothing to
/// do with ARM, and `windows-latest` was failing exactly the same way.
///
/// Both halves are checked. A list that names `icon.ico` and does not ship it
/// fails the same build, one step later.
#[test]
fn each_platform_gets_an_icon_in_the_format_its_bundler_demands() {
    let conf: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("src-tauri/tauri.conf.json"))
            .expect("tauri.conf.json is readable"),
    )
    .expect("tauri.conf.json is JSON");

    let icons: Vec<String> = conf["bundle"]["icon"]
        .as_array()
        .expect("bundle.icon is a list")
        .iter()
        .map(|v| v.as_str().expect("an icon path is a string").to_string())
        .collect();

    for (extension, platform, consequence) in [
        (
            ".ico",
            "Windows",
            "the MSI and NSIS bundlers stop with `Couldn't find a .ico icon`,              after the application has finished compiling",
        ),
        (
            ".icns",
            "macOS",
            "the bundler generates one from a PNG, so this is the platform              where the omission does NOT fail — which is how it stayed hidden",
        ),
    ] {
        assert!(
            icons.iter().any(|i| i.ends_with(extension)),
            "bundle.icon names no `{extension}`, so {platform} is bundled              without one: {consequence}.\n\nbundle.icon is {icons:?}"
        );
    }

    for icon in &icons {
        let path = repo_root().join("src-tauri").join(icon);
        assert!(
            path.exists(),
            "bundle.icon names {icon} and there is no such file. The bundler              reads this list, so a name without a file fails the build one step              after a list with no name at all."
        );
    }
}

/// The AppImage bundler copies a binary off the build machine, and it has to be
/// there.
///
/// Found the only way it could be found. `tauri-bundler` copies
/// `/usr/bin/xdg-open` into the AppDir when the application can open a link —
/// this one can, through `tauri-plugin-opener` — and it copies it from the
/// runner. `ubuntu-latest` ships `xdg-utils`; `ubuntu-24.04-arm` does not. The
/// same commit produced an AppImage on one and `xdg-open binary not found` on
/// the other, *after* the `.deb` and the `.rpm` were already written.
///
/// Guarded because of how it will look to whoever reads the list next: it is
/// not a `-dev` header and nothing else in the file needs it, so it reads like
/// something that wandered in. It is the only reason the two Linux rows are the
/// same machine.
#[test]
fn the_appimage_bundler_gets_the_binary_it_copies_off_the_runner() {
    let text = workflow();
    assert!(
        text.contains("xdg-utils"),
        "release.yml does not install `xdg-utils`. The AppImage bundler copies \
         /usr/bin/xdg-open out of the build machine, `ubuntu-24.04-arm` does \
         not ship it, and the failure lands after the .deb and the .rpm are \
         already written — so it reads as an ARM bundler problem and is a \
         runner image problem."
    );
}

/// The matrix target reaches the toolchain that actually builds.
///
/// `dtolnay/rust-toolchain@stable` installs the target into **stable**, and
/// nothing in this job builds with stable: `src-tauri/rust-toolchain.toml` pins
/// 1.96.1 and rustup resolves that pin from the working directory.
/// `stable-<host>` and `1.96.1-<host>` are separate installations even on the
/// day they are the same compiler.
///
/// Free on the four rows where the target is the host — a host's own std is
/// always there — and the entire build on `x86_64-apple-darwin`, which
/// cross-compiles from an arm64 runner. The same class of bug as the one
/// `workflow_parity.rs` already guards, and invisible in the same way: nobody
/// runs this job locally.
#[test]
fn the_pinned_toolchain_is_given_the_matrix_target() {
    let text = workflow();
    let step = steps(&text)
        .into_iter()
        .find(|step| step.contains("rustup target add"))
        .expect(
            "nothing adds the matrix target to the pinned toolchain. \
             `dtolnay/rust-toolchain@stable` adds it to `stable`, and the \
             build runs under the 1.96.1 named in src-tauri/rust-toolchain.toml.",
        );

    assert!(
        step.contains("working-directory: src-tauri"),
        "`rustup target add` runs from the repository root, where there is no \
         `rust-toolchain.toml` — so it adds the target to whatever the default \
         toolchain is, which is the toolchain that does not build this:\n{step}"
    );
}

/// Six targets, and the two ARM rows on native ARM runners.
///
/// The matrix is six. Cross-compiling the ARM rows on x86 runners would be a
/// different claim wearing the same number: the bundler runs native tools, and
/// "it cross-compiled" is not "it produced a package on that architecture".
#[test]
fn the_matrix_is_six_targets_with_the_arm_rows_on_arm_runners() {
    let text = workflow();
    let targets: Vec<&str> = text
        .lines()
        .filter_map(|l| l.trim().strip_prefix("target: "))
        .collect();

    assert_eq!(
        targets.len(),
        6,
        "release.yml builds {} target(s); README.md says six:\n{targets:#?}",
        targets.len()
    );

    for (target, runner) in [
        ("aarch64-unknown-linux-gnu", "ubuntu-24.04-arm"),
        ("aarch64-pc-windows-msvc", "windows-11-arm"),
    ] {
        assert!(
            targets.contains(&target),
            "`{target}` is no longer in the matrix — that row is half of what \
             #22 is about"
        );
        assert!(
            text.contains(runner),
            "`{target}` is built without `{runner}`. A cross-compiled ARM \
             bundle is a different claim: the bundler runs native tools, and \
             #22 asks whether a package can be produced on that architecture."
        );
    }
}
