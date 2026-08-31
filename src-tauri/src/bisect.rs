//! `git bisect`, with the environment the commit was written against.
//!
//! ## The half of a bisect that is a lie
//!
//! `git bisect` moves the code and nothing else. Three months ago the project
//! declared PHP 8.3 and locked `redis` at 7.0; the container running on this
//! machine today is 8.4 and 7.2. So every step of an ordinary bisect through
//! that range is running **old code against a new environment**, and the commit
//! it finally accuses may be innocent — the behaviour changed with the runtime,
//! not with the diff.
//!
//! Nothing in this category does anything about that, because nothing in this
//! category knows what environment a commit wanted. This app does now:
//! `stackvo.json` has always travelled with the repository, and since
//! [`crate::lock`] `stackvo.lock` does too. Both are readable **at a revision**
//! without touching the working tree — `git show <rev>:<path>` — so at every
//! step this can say what that commit expected and how this machine differs.
//!
//! ## It reports the difference and does not install it
//!
//! The obvious next move is to make each step *bring the environment along*,
//! and it is the wrong one. Downgrading a service means replacing a container
//! whose volume holds the developer's data, and a ten-step bisect would do that
//! twenty times. Destroying somebody's database to answer a question about a
//! diff is not a trade this app gets to make on their behalf.
//!
//! So each step returns a sentence: *this commit was locked at redis 7.0 and
//! you are running 7.2 — that difference is inside your bisect.* Acting on it
//! is a decision, and it is theirs. The mechanism to act is already on the
//! Market page, one click away, and it asks first.
//!
//! ## The working tree is somebody else's
//!
//! Three rules fell out of that, and each is a refusal rather than a fallback:
//!
//! **A dirty tree is refused before anything starts.** `git bisect` checks out
//! other commits; doing that over uncommitted work is how an afternoon is lost.
//! Git refuses most of these itself, and this refuses first and by name — a
//! git error surfacing in a desktop app is a sentence nobody can act on.
//!
//! **A revision is validated, not passed through.** `bad` and `good` arrive as
//! free text from a webview and reach a subprocess argument, which is the exact
//! hazard [`crate::git`] wrote its allowlist for: a value beginning with `-` is
//! read by git as an option. See [`is_revision`].
//!
//! **Every move is recorded.** Starting a bisect and marking a step both put
//! the user's checkout on a different commit. That is the same class of act as
//! taking a file out of git's index, and it belongs in [`crate::audit`] for the
//! same reason: somebody has to be able to account for the machine.

use crate::error::{Code, Error, Result};
use std::path::Path;

/// What the person testing the commit decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// The behaviour being hunted is present here.
    Bad,
    /// It is not.
    Good,
    /// This commit cannot be tested — it does not build, or the feature does
    /// not exist yet. Git's own third answer, and leaving it out would make
    /// people mark a commit `good` to get past it, which poisons the search.
    Skip,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Verdict::Bad => "bad",
            Verdict::Good => "good",
            Verdict::Skip => "skip",
        }
    }
}

/// Where a bisect currently is.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Whether one is in progress in this checkout.
    pub running: bool,
    /// The commit checked out right now, short form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Its subject line, so the row is a commit and not a hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Roughly how many steps are left, as git counts them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    /// Set once git has an answer, and then the search is over.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub culprit: Option<String>,
    /// How this machine differs from what the commit under test expected.
    ///
    /// Empty is a real and useful answer: it means the environment is not in
    /// the bisect, so whatever the search accuses is the code.
    pub drift: Vec<Drift>,
}

/// One way this machine differs from what the commit expected.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drift {
    /// Stable key; the UI holds the sentence. The `preflight` arrangement.
    pub id: &'static str,
    /// The runtime, or the service id.
    pub subject: String,
    /// What the commit asked for.
    pub wanted: String,
    /// What is here. `None` when nothing is — "not installed" and "a different
    /// version" send somebody to different places.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
}

// ------------------------------------------------------------- the pure half

/// Is this string safe to hand git as a revision?
///
/// An allowlist, for the reason [`crate::git::parse`] is one: this is
/// webview-supplied text reaching a subprocess argument, and the concrete
/// attack is not exotic — a value beginning with `-` is read by git as an
/// option, and git has options that name programs to run.
///
/// The alphabet is git's own revision syntax and nothing besides:
/// `main`, `v1.2.3`, `HEAD~5`, `abc1234`, `origin/main`, `@{u}`. Anything with
/// a space, a quote, a semicolon or a leading dash is refused — and refused
/// rather than sanitised, because a rewritten revision is a bisect over a range
/// nobody asked for.
pub fn is_revision(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 256
        && !text.starts_with('-')
        && text.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '.' | '_' | '/' | '-' | '~' | '^' | '@' | '{' | '}')
        })
}

/// `Bisecting: 4 revisions left to test after this (roughly 3 steps)` → 3.
///
/// Parsed rather than counted here, because git is the thing that knows: the
/// number depends on the shape of the history, on skips, and on merge parents,
/// and a second estimate computed in this file would disagree with the one on
/// the terminal in exactly the confusing cases.
pub fn parse_steps(text: &str) -> Option<u32> {
    let after = text.split("roughly ").nth(1)?;
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// `abc1234 is the first bad commit` → `abc1234`.
///
/// The end of the search. Recognised on the exact sentence git prints, and
/// `None` for everything else — a bisect that is still running must never be
/// reported as finished, because the finished screen is the one people copy a
/// hash out of.
pub fn parse_culprit(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(hash) = line.strip_suffix(" is the first bad commit") {
            let hash = hash.trim();
            if !hash.is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(hash.to_string());
            }
        }
    }
    None
}

/// How this machine differs from what one commit expected.
///
/// Pure, and the only place the comparison is made. `declared` is the
/// `stackvo.json` at that revision and `locked` its `stackvo.lock`; both are
/// optional because a commit from before either existed has neither, and that
/// is not a finding — it is a range this cannot say anything about, which is
/// better than a range it says the wrong thing about.
pub fn drift(
    declared: Option<&crate::manifest::Manifest>,
    locked: Option<&crate::lock::Lock>,
    runtime_now: Option<&str>,
    instances: &crate::instances::Table,
) -> Vec<Drift> {
    let mut out = Vec::new();

    // The runtime first, because it is the difference K-6 was written about:
    // "three months ago the commit had PHP 8.3 and today's container has 8.4".
    if let (Some(manifest), Some(now)) = (declared, runtime_now) {
        if let Some(wanted) = runtime_version(manifest) {
            if wanted != now {
                out.push(Drift {
                    id: "runtime",
                    subject: manifest.runtime.clone(),
                    wanted: wanted.to_string(),
                    found: Some(now.to_string()),
                });
            }
        }
    }

    // Then the services, which only became answerable when the lock file did.
    for entry in locked.map(|l| l.services.as_slice()).unwrap_or(&[]) {
        match crate::lock::compare(entry, instances) {
            crate::lock::Drift::Same => {}
            crate::lock::Drift::Absent => out.push(Drift {
                id: "service",
                subject: entry.service.clone(),
                wanted: entry.version.clone(),
                found: None,
            }),
            crate::lock::Drift::Off => out.push(Drift {
                id: "serviceOff",
                subject: entry.service.clone(),
                wanted: entry.version.clone(),
                found: None,
            }),
            crate::lock::Drift::Version | crate::lock::Drift::Repackaged => out.push(Drift {
                id: "service",
                subject: entry.service.clone(),
                wanted: entry.version.clone(),
                found: instances
                    .instances
                    .iter()
                    .find(|i| i.service == entry.service && i.enabled)
                    .map(|i| i.version.clone()),
            }),
        }
    }

    out
}

/// The version a manifest states for whichever runtime it names.
fn runtime_version(manifest: &crate::manifest::Manifest) -> Option<&str> {
    match manifest.runtime.as_str() {
        "php" => manifest.php.as_ref().map(|c| c.version.as_str()),
        "node" => manifest.node.as_ref().map(|c| c.version.as_str()),
        _ => manifest.lang.as_ref().map(|c| c.version.as_str()),
    }
}

// -------------------------------------------------------------- the git half

/// Run one git command in the project and return its combined output.
///
/// Combined because git says most of what matters on stderr — `Bisecting: 4
/// revisions left` is written there — and a caller that read only stdout would
/// have to know which of the two each message lands on, which changes between
/// versions.
async fn git(dir: &Path, args: &[&str]) -> Result<(bool, String)> {
    let out = tokio::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| Error::io("running git".to_string(), e))?;

    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.success(), text))
}

/// Refuse before anything moves.
///
/// Git refuses most of these itself and this refuses first, by name and with
/// the repair. A `git` error text arriving in a desktop alert is a sentence
/// written for a terminal, and the person reading it here has no terminal open.
async fn require_clean(dir: &Path) -> Result<()> {
    if !crate::worktree::is_repository(dir) {
        return Err(Error::new(
            Code::Unsupported,
            "this project is not a git repository, so there is no history to bisect".to_string(),
        ));
    }

    let (ok, text) = git(dir, &["status", "--porcelain", "--untracked-files=no"]).await?;
    if !ok {
        return Err(Error::new(Code::Unsupported, text.trim().to_string()));
    }
    if !text.trim().is_empty() {
        return Err(Error::new(
            Code::Conflict,
            "there are uncommitted changes here. A bisect moves the checkout from commit \
             to commit, so commit or stash them first."
                .to_string(),
        ));
    }
    Ok(())
}

/// Read one file at one revision, without touching the working tree.
///
/// `None` for anything that is not there — a commit from before `stackvo.lock`
/// existed simply has no lock, which is a fact about the range and not a
/// failure.
pub async fn show(dir: &Path, rev: &str, file: &str) -> Option<String> {
    if !is_revision(rev) {
        return None;
    }
    let (ok, text) = git(dir, &["show", &format!("{rev}:{file}")]).await.ok()?;
    ok.then_some(text)
}

/// Begin. `bad` is where the behaviour is, `good` is where it is not.
pub async fn start(dir: &Path, bad: &str, good: &str) -> Result<()> {
    for rev in [bad, good] {
        if !is_revision(rev) {
            return Err(Error::new(
                Code::InvalidInput,
                format!("{rev:?} is not a revision"),
            ));
        }
    }
    require_clean(dir).await?;

    // Both named at once rather than in three commands. `git bisect start <bad>
    // <good>` is the form that either takes the whole range or takes none of
    // it; started-then-marked leaves a half-configured bisect behind when the
    // second revision turns out not to exist.
    let (ok, text) = git(dir, &["bisect", "start", bad, good]).await?;
    if !ok {
        // The tree has not moved on a failed start, but a partial one is
        // possible on older gits — resetting is cheap and leaves the checkout
        // where the user left it.
        let _ = git(dir, &["bisect", "reset"]).await;
        return Err(Error::new(Code::Unsupported, text.trim().to_string()));
    }
    Ok(())
}

/// Record a verdict for the commit that is checked out, and move on.
pub async fn mark(dir: &Path, verdict: Verdict) -> Result<()> {
    let (ok, text) = git(dir, &["bisect", verdict.as_str()]).await?;
    if !ok {
        return Err(Error::new(Code::Unsupported, text.trim().to_string()));
    }
    Ok(())
}

/// Stop, and put the checkout back where it was.
///
/// `git bisect reset` returns to the branch the bisect started from, which is
/// the whole reason this is safe to offer: the compensation is git's own and
/// does not have to be reconstructed here.
pub async fn reset(dir: &Path) -> Result<()> {
    let (ok, text) = git(dir, &["bisect", "reset"]).await?;
    if !ok {
        return Err(Error::new(Code::Unsupported, text.trim().to_string()));
    }
    Ok(())
}

/// Where the bisect is, and what is different about this machine.
///
/// Answers on a repository with no bisect running — `running: false` and
/// nothing else — because the pane asking has to be able to open before
/// anything has started.
pub async fn status(dir: &Path, root: &Path, instances: &crate::instances::Table) -> Status {
    let mut status = Status::default();
    if !crate::worktree::is_repository(dir) {
        return status;
    }

    // The file git itself uses to know one is in progress. Asked of the
    // filesystem rather than by running `git bisect log`, which fails with a
    // non-zero status when there is no bisect — an error that means "no" is an
    // error this would have to special-case by message.
    let (_, git_dir) = git(dir, &["rev-parse", "--git-dir"])
        .await
        .unwrap_or_default();
    let git_dir = dir.join(git_dir.trim());
    status.running = git_dir.join("BISECT_LOG").is_file();
    if !status.running {
        return status;
    }

    if let Ok((true, line)) = git(dir, &["log", "-1", "--format=%h %s"]).await {
        let line = line.trim();
        let (hash, subject) = line.split_once(' ').unwrap_or((line, ""));
        status.commit = Some(hash.to_string());
        status.subject = (!subject.is_empty()).then(|| subject.to_string());
    }

    // Git wrote the estimate and the answer into the same place: the log it
    // keeps. Reading it back is how a status call — which did not itself run a
    // bisect step — knows what the last step printed.
    if let Ok(log) = std::fs::read_to_string(git_dir.join("BISECT_LOG")) {
        status.steps = parse_steps(&log);
        status.culprit = parse_culprit(&log);
    }

    // The environment half, and the reason this module exists.
    if let Some(rev) = &status.commit {
        let declared = show(dir, rev, "stackvo.json")
            .await
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .map(|json| crate::manifest::normalize(&json, "", "project"));
        let locked = show(dir, rev, crate::lock::FILE)
            .await
            .and_then(|text| serde_json::from_str::<crate::lock::Lock>(&text).ok());

        // The workspace's `.env`, which is what the generator actually builds
        // the runtime at — not the manifest on disk. The comparison has to be
        // against what is *running*, and the manifest in the working tree is
        // the commit's own copy while a bisect is in progress.
        let now = crate::config::Env::load(root).ok().and_then(|env| {
            declared
                .as_ref()
                .and_then(|m| env.get(&runtime_key(&m.runtime)).map(str::to_string))
        });

        status.drift = drift(
            declared.as_ref(),
            locked.as_ref(),
            now.as_deref(),
            instances,
        );
    }

    status
}

/// The `.env` key holding the version this workspace builds a runtime at.
fn runtime_key(runtime: &str) -> String {
    match runtime {
        "php" => "PHP_VERSION".to_string(),
        other => format!("{}_VERSION", other.to_uppercase()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The allowlist, and the attack it is an allowlist for.
    ///
    /// `git.rs` wrote the reasoning down for URLs and it is the same hazard
    /// here: this is webview text reaching a subprocess argument, and a value
    /// beginning with `-` is read by git as an option — of which git has
    /// several that name a program to run.
    #[test]
    fn a_revision_is_git_syntax_and_nothing_else() {
        for good in [
            "main",
            "HEAD~5",
            "v1.2.3",
            "abc1234",
            "origin/main",
            "@{u}",
            "release/2026-08",
            "HEAD^2",
        ] {
            assert!(is_revision(good), "{good} is a revision");
        }

        for bad in [
            "",
            "--upload-pack=/bin/sh",
            "-n",
            "main; rm -rf /",
            "main file",
            "$(id)",
            "a\0b",
            "ext::sh -c whoami",
        ] {
            assert!(!is_revision(bad), "{bad:?} must not reach git");
        }

        // Refused rather than trimmed. A sanitised revision is a bisect over a
        // range nobody asked for, which is a wrong answer rather than an error.
        assert!(!is_revision(&"a".repeat(257)));
    }

    /// Git's own estimate, read rather than recomputed.
    #[test]
    fn the_step_estimate_comes_from_git() {
        assert_eq!(
            parse_steps("Bisecting: 4 revisions left to test after this (roughly 3 steps)"),
            Some(3)
        );
        assert_eq!(
            parse_steps("Bisecting: 0 revisions left to test after this (roughly 0 steps)"),
            Some(0)
        );
        // Nothing to read is None, not zero: "no estimate" and "no steps left"
        // are opposite things to put on a screen.
        assert_eq!(parse_steps("Bisecting: 1 revision left"), None);
        assert_eq!(parse_steps(""), None);
    }

    /// The end of the search, recognised on the exact sentence and nothing
    /// looser.
    ///
    /// A bisect still running must never read as finished: the finished screen
    /// is the one somebody copies a hash out of and takes to a colleague.
    #[test]
    fn only_the_sentence_that_means_finished_is_read_as_finished() {
        assert_eq!(
            parse_culprit("git bisect bad\n9fceb02d is the first bad commit\n"),
            Some("9fceb02d".to_string())
        );
        assert_eq!(parse_culprit("Bisecting: 3 revisions left"), None);
        assert_eq!(parse_culprit("looking for the first bad commit"), None);
        // A subject line that happens to end that way is not a hash.
        assert_eq!(parse_culprit("fix: whatever is the first bad commit"), None);
    }

    fn manifest(runtime: &str, version: &str) -> crate::manifest::Manifest {
        let json = serde_json::json!({
            "name": "shop",
            "runtime": runtime,
            runtime: { "version": version, "start": "x", "port": 3000, "install": "i" },
        });
        crate::manifest::normalize(&json, "", "shop")
    }

    fn table(rows: &[(&str, &str, bool)]) -> crate::instances::Table {
        let mut table = crate::instances::Table::default();
        for (service, version, enabled) in rows {
            table.instances.push(crate::instances::Instance {
                id: format!("{service}-{}", version.replace('.', "-")),
                service: (*service).to_string(),
                version: (*version).to_string(),
                package: crate::instances::PackageRef {
                    source: "official".into(),
                    sha256: String::new(),
                    installed_at: "2026-01-01T00:00:00Z".into(),
                },
                enabled: *enabled,
                primary: true,
                ports: Default::default(),
                volumes: Default::default(),
                settings: Default::default(),
                secret_refs: Default::default(),
            });
        }
        table
    }

    fn lock(rows: &[(&str, &str)]) -> crate::lock::Lock {
        crate::lock::Lock {
            lock_version: crate::lock::SCHEMA_VERSION,
            at: "2026-08-30T09:14:02Z".into(),
            services: rows
                .iter()
                .map(|(service, version)| crate::lock::Locked {
                    service: (*service).to_string(),
                    version: (*version).to_string(),
                    source: "official".into(),
                    sha256: String::new(),
                })
                .collect(),
        }
    }

    /// The sentence this module was written for.
    ///
    /// *"Three months ago the commit had PHP 8.3 and today's container has
    /// 8.4"* — which means half the bisect is running old code against a new
    /// runtime, and the commit it accuses may be innocent.
    #[test]
    fn the_runtime_the_commit_wanted_is_held_against_the_one_that_is_running() {
        let m = manifest("php", "8.3");

        let found = drift(
            Some(&m),
            None,
            Some("8.4"),
            &crate::instances::Table::default(),
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "runtime");
        assert_eq!(found[0].wanted, "8.3");
        assert_eq!(found[0].found.as_deref(), Some("8.4"));

        // And the same machine on the same version reports nothing, which is
        // the useful answer: the environment is not in the bisect, so whatever
        // the search accuses is the code.
        assert!(drift(
            Some(&m),
            None,
            Some("8.3"),
            &crate::instances::Table::default()
        )
        .is_empty());
    }

    /// The services half, which only became answerable when the lock did.
    ///
    /// Before `stackvo.lock` a commit carried no service versions at all, so
    /// there was nothing to compare — this is the half of K-6 that was blocked
    /// on K-2 and is the reason the two were written in that order.
    #[test]
    fn a_commit_locked_at_an_older_service_says_so() {
        let l = lock(&[("redis", "7.0"), ("mysql", "8.0")]);
        let now = table(&[("redis", "7.2", true), ("mysql", "8.0", true)]);

        let found = drift(None, Some(&l), None, &now);
        assert_eq!(found.len(), 1, "only the one that differs");
        assert_eq!(found[0].subject, "redis");
        assert_eq!(found[0].wanted, "7.0");
        assert_eq!(found[0].found.as_deref(), Some("7.2"));

        // Not installed and installed-but-off are kept apart, because the two
        // repairs are an install and a switch.
        let gone = drift(None, Some(&l), None, &table(&[("mysql", "8.0", true)]));
        assert_eq!(gone[0].id, "service");
        assert!(gone[0].found.is_none());

        let off = drift(
            None,
            Some(&l),
            None,
            &table(&[("redis", "7.0", false), ("mysql", "8.0", true)]),
        );
        assert_eq!(off[0].id, "serviceOff");
    }

    /// A commit from before either file existed.
    ///
    /// Nothing is reported, and that is right: a range this cannot say anything
    /// about is better served by silence than by a finding invented out of an
    /// absent file. The bisect still works — it simply has no environment half
    /// over that range, which is exactly where every other tool always is.
    #[test]
    fn a_commit_that_predates_the_manifest_produces_no_findings() {
        assert!(drift(None, None, Some("8.4"), &table(&[("redis", "7.2", true)])).is_empty());
    }
}
