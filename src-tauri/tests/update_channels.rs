//! The beta channel exists end to end, and the stable one cannot be hurt by it.
//!
//! `channel.rs` declared `Channel::Beta` before anything could be on it, and
//! wrote down why a *setting* had to wait: the updater plugin walks its
//! endpoint list until one answers, so a second endpoint is a fallback and not
//! a selector, and a channel nobody publishes to is a setting that silently
//! stops updates. v0.2.0 shipped on 3 September 2026, so the wait is over —
//! and this file holds the pieces that now have to agree, across a Rust
//! module, a workflow, a JavaScript tool and a preferences key.
//!
//! ## The shape
//!
//! * **Tag rule.** A hyphen in the tag (`v0.3.0-beta.1`) makes the release a
//!   pre-release. GitHub's `releases/latest` never resolves to one, and the
//!   stable endpoint is exactly that URL — so a stable install cannot be
//!   offered a beta by any route.
//! * **Pointer.** Pre-releases have no `releases/latest`, so the workflow keeps
//!   one: a release tagged `beta`, itself a pre-release, holding `beta.json`.
//!   The `channel` job refreshes it when a person publishes — never from the
//!   tag run, whose release is a draft nobody can download from.
//! * **Client.** The app rewrites the plugin's endpoint list at launch from a
//!   preference: `[beta.json, latest.json]` for beta, `[latest.json]` for
//!   everybody else. Beta accepts stable manifests, so the fallback is an
//!   update and not a refusal.
//!
//! Text tests, for the reason `release_rehearsal.rs` gives: Actions cannot be
//! run here, and what survives without a runner is that the strings the parts
//! exchange are the same strings.

use std::path::{Path, PathBuf};

use stackvo_desktop_lib::channel::{Channel, PREFERENCE, ROLLING_TAG};

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

fn workflow() -> String {
    read(".github/workflows/release.yml")
}

fn stable_endpoint() -> String {
    let conf: serde_json::Value =
        serde_json::from_str(&read("src-tauri/tauri.conf.json")).expect("tauri.conf.json parses");
    let endpoints = conf["plugins"]["updater"]["endpoints"]
        .as_array()
        .expect("the updater declares its endpoints");
    assert_eq!(
        endpoints.len(),
        1,
        "the beta endpoint must not be declared in tauri.conf.json: a second \
         entry there is a fallback for EVERY install, stable ones included. It \
         is derived at launch by `channel::configure` for the installs that \
         asked for it."
    );
    endpoints[0]
        .as_str()
        .expect("an endpoint is a string")
        .to_string()
}

/// The job that maintains the pointer, out of a workflow with several.
fn channel_job() -> String {
    let text = workflow();
    let at = text
        .find("\n  channel:")
        .expect("release.yml has no `channel` job — nothing maintains beta.json");
    text[at..].to_string()
}

/// A hyphen makes a pre-release, and a pre-release can never be `latest`.
#[test]
fn a_tag_with_a_hyphen_is_a_pre_release_and_stable_never_sees_it() {
    let text = workflow();
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("prerelease:"))
        .expect("tauri-action is no longer told whether the release is a pre-release");

    assert!(
        line.contains("contains(github.ref_name, '-')"),
        "`prerelease` is `{}`. It must be decided by the tag — a hyphen, \
         semver's own marker — because that flag is what keeps a beta out of \
         `releases/latest`, and `releases/latest/download/latest.json` is the \
         stable endpoint. A pre-release published without it is offered to \
         every stable install.",
        line.trim()
    );
    assert!(
        stable_endpoint().contains("/releases/latest/download/"),
        "the stable endpoint no longer reads `releases/latest`, so the \
         pre-release flag no longer protects it — say here what does"
    );
}

/// The pointer moves when a person publishes, and only then.
#[test]
fn beta_json_is_refreshed_on_publish_and_never_from_the_tag_run() {
    let text = workflow();
    assert!(
        text.contains("release:\n    types: [published]"),
        "release.yml does not run on `release: published`. The tag run ends in \
         a draft, and a draft's assets cannot be downloaded — the only moment \
         `beta.json` may move is when somebody presses Publish."
    );

    let job = channel_job();
    assert!(
        job.contains("if: github.event_name == 'release'"),
        "the `channel` job is not gated on the publish event:\n{job}"
    );
    assert!(
        job.contains("!github.event.release.draft"),
        "the `channel` job would run for a draft:\n{job}"
    );

    // And the build does not start over when the event fires: a second run
    // on the same tag would open a second draft named after it.
    for job_name in ["preflight", "build", "checksums"] {
        let at = text
            .find(&format!("\n  {job_name}:"))
            .unwrap_or_else(|| panic!("no `{job_name}` job"));
        let head = &text[at..at + 600];
        assert!(
            head.contains("if: github.event_name != 'release'"),
            "`{job_name}` runs on the publish event too, which rebuilds a \
             release that already exists"
        );
    }
}

/// The rolling release is a pre-release, or the stable channel dies.
///
/// A release that is not a pre-release becomes `releases/latest` the moment
/// it is newer than the current one, and the rolling release carries no
/// `latest.json`. Without the flag, the first run of the `channel` job would
/// turn the stable endpoint into a 404 for every install in the field.
#[test]
fn the_rolling_release_can_never_become_latest() {
    let job = channel_job();
    let create = job
        .lines()
        .find(|l| l.contains(&format!("gh release create {ROLLING_TAG}")))
        .expect("the `channel` job never creates the rolling release");
    assert!(
        create.contains("--prerelease"),
        "the rolling release is created without `--prerelease`: `{}`. It would \
         become `releases/latest`, and the stable endpoint would answer 404.",
        create.trim()
    );
    assert!(
        job.contains(&format!(
            "gh release upload {ROLLING_TAG} {} --clobber",
            Channel::Beta.manifest_name()
        )),
        "the `channel` job does not replace {} on the `{ROLLING_TAG}` release",
        Channel::Beta.manifest_name()
    );
}

/// The app asks the URL the workflow publishes to. Same repository, same
/// tag, same file name — derived on one side, typed on the other, held
/// together here.
#[test]
fn the_app_asks_for_the_file_the_workflow_publishes() {
    let stable = stable_endpoint();
    let beta = Channel::Beta
        .endpoint(&stable)
        .expect("the beta endpoint cannot be derived from the configured stable one");

    let base = stable
        .strip_suffix("/releases/latest/download/latest.json")
        .expect("the stable endpoint is GitHub's `latest` pointer");
    assert_eq!(
        beta,
        format!(
            "{base}/releases/download/{ROLLING_TAG}/{}",
            Channel::Beta.manifest_name()
        )
    );

    // And the same derivation in the tool that asks it from a checkout, so
    // `npm run updates:check -- --channel beta` asks what the app asks.
    let tool = read("tools/check-updater-endpoint.mjs");
    assert!(
        tool.contains(&format!(
            "/releases/download/{ROLLING_TAG}/{}",
            Channel::Beta.manifest_name()
        )),
        "tools/check-updater-endpoint.mjs derives a different beta URL from the app's"
    );
    assert!(
        tool.contains("--channel"),
        "`npm run updates:check` cannot be pointed at the beta channel"
    );
}

/// Stable is always in the list, and always last.
#[test]
fn a_beta_install_falls_back_to_stable_and_a_stable_install_asks_stable_only() {
    let stable = stable_endpoint();
    let beta = Channel::Beta.endpoints(&stable);
    assert_eq!(beta.len(), 2);
    assert_eq!(beta[1], stable);
    assert_eq!(Channel::Stable.endpoints(&stable), vec![stable.clone()]);

    // The rule that makes the fallback an update rather than a refusal.
    assert!(Channel::Beta.accepts(Channel::Stable));
    assert!(!Channel::Stable.accepts(Channel::Beta));
}

/// The rule for what `beta.json` names is in a file with tests, and the
/// workflow calls that file.
#[test]
fn the_pointer_rule_is_a_tested_tool_and_the_workflow_uses_it() {
    let job = channel_job();
    for mode in ["newest", "stamp"] {
        assert!(
            job.contains(&format!("node tools/beta-manifest.mjs {mode}")),
            "the `channel` job does not call `beta-manifest.mjs {mode}`, so the \
             rule it applies is written in YAML where nothing can test it"
        );
    }
    assert!(repo_root().join("tools/beta-manifest.mjs").exists());
    assert!(
        repo_root().join("tests/beta-manifest.spec.js").exists(),
        "the pointer tool has no tests"
    );
}

/// One preference, spelled once.
///
/// The switch writes it from the webview and the launch reads it from Rust.
/// Two spellings would be a beta switch that saves under one name and is read
/// under another — a setting that silently does nothing.
#[test]
fn the_switch_writes_the_key_the_launch_reads() {
    let updates = read("src/lib/updates.js");
    assert!(
        updates.contains(&format!("CHANNEL_PREFERENCE = '{PREFERENCE}'")),
        "src/lib/updates.js does not spell the preference `{PREFERENCE}`"
    );
    let settings = read("src/views/Settings.vue");
    assert!(
        settings.contains("CHANNEL_PREFERENCE") && settings.contains("channelOf("),
        "Settings.vue does not save the channel through the shared constant, or \
         does not hand the channel to the check"
    );
    assert!(
        read("src-tauri/src/lib.rs").contains("channel::configure(context.config_mut())"),
        "lib.rs no longer chooses the endpoint list before the plugin is built, \
         so the preference is read by nothing"
    );
}

/// The person who ticks the switch is told what it does.
#[test]
fn the_help_and_the_readme_describe_the_beta_channel() {
    for doc in [
        "docs/help/en/page-settings-updates.md",
        "docs/help/tr/page-settings-updates.md",
        "README.md",
    ] {
        let text = read(doc).to_ascii_lowercase();
        assert!(
            text.contains("beta"),
            "{doc} says nothing about the beta channel, and the switch is on the \
             card it describes"
        );
    }
}
