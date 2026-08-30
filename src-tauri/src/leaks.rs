//! Credentials sitting where they should not be.
//!
//! ## The direction nothing went
//!
//! [`crate::secrets`] moves a password out of `.env` and into the OS keystore.
//! That is one direction, and it is the one somebody takes **after** they know
//! there is a problem. The other direction — *"there is an AWS key in your
//! `.env` that is not in the keystore"*, and harder, *"that key is in a file
//! git is tracking"* — did not exist, so nobody found out until the repository
//! was already public.
//!
//! ## Matching the value, not the name — and why that is the whole design
//!
//! [`crate::config::Env::is_secret`] matches a **key name** by suffix:
//! `PASSWORD`, `TOKEN`, `SECRET`, `KEY`. That is right for masking, where the
//! cost of a miss is a value printed in a log. It is not enough here, and
//! `preset.rs` already wrote down why in the sentence this module takes its
//! shape from: *"a key added upstream tomorrow called `SERVICE_FOO_APIKEY`
//! would sail straight through"*.
//!
//! So this matches the **value**. `AKIA…` is an AWS access key id whatever the
//! variable holding it is called, and a PEM private key header is a private key
//! in a file named anything at all. The key-name rule is kept as a second,
//! independent net — two nets that fail differently catch more than one net
//! that is twice as clever.
//!
//! ## Only shapes nobody else has
//!
//! Every rule below is a vendor-assigned prefix with a fixed alphabet and a
//! known length. There is deliberately no entropy heuristic and no "long
//! random-looking string" rule: those fire on minified JavaScript, on a hash in
//! a lockfile and on a base64 image, and a scanner people learn to ignore is
//! worse than no scanner. A miss here costs a finding; a false positive costs
//! the feature.
//!
//! ## Asking git about the history without putting a secret on a command line
//!
//! The obvious way to ask whether a value was ever committed is
//! `git log -S<value>`, and it is the wrong way: the value becomes an argument,
//! and an argument is visible in `ps` to every process on the machine for as
//! long as it runs. `db.rs` pays that cost knowingly in one place and documents
//! it. There is no reason to pay it here.
//!
//! The question is asked **by path** instead, which is the standard way and
//! also the better one: `git log --all -- <path>` answers "has this file ever
//! been committed, on any branch", and a path is not a secret. That is a
//! stronger answer than a value search as well — a file that was committed and
//! later deleted is still in the history with everything that was in it, and a
//! value search would only find the exact string somebody has since rotated
//! half of.
//!
//! Tracked files are read and matched **in this process**, so the only things
//! that ever reach git's argument list are `ls-files`, `log` and a path.
//!
//! ## A finding is usable without carrying the value
//!
//! "Never print the secret" is right and, on its own, leaves somebody with a
//! finding they cannot act on: two rows saying `awsAccessKey` do not say
//! whether it is one key in two places or two keys. So each finding carries
//! what every scanner in this field carries instead of the value:
//!
//! * a **fingerprint** — the first twelve hex characters of the value's
//!   sha256. Two rows with one fingerprint are one secret; it identifies and
//!   does not reveal;
//! * a **masked preview** — the first and last few characters, and only for a
//!   value long enough that the ends are not most of it. Short values are
//!   masked whole, because four characters of a six-character password is the
//!   password.
//!
//! Both are what a person needs to recognise the thing without the report
//! becoming a second copy of it.

use std::path::Path;

/// One credential shape, as its issuer defined it.
pub struct Rule {
    /// Stable key; the UI holds the label and what to do about it.
    pub id: &'static str,
    /// What every one of these starts with.
    pub prefix: &'static str,
    /// How many characters follow the prefix, at least.
    pub min_tail: usize,
    /// Which characters may follow it.
    pub tail: Charset,
}

/// The alphabet a credential's tail is drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// `A-Z0-9`
    UpperAlphanumeric,
    /// `A-Za-z0-9`
    Alphanumeric,
    /// `A-Za-z0-9_-`
    Token,
    /// Anything, for a shape whose prefix alone is the whole evidence.
    Any,
}

impl Charset {
    fn accepts(self, c: char) -> bool {
        match self {
            Charset::UpperAlphanumeric => c.is_ascii_uppercase() || c.is_ascii_digit(),
            Charset::Alphanumeric => c.is_ascii_alphanumeric(),
            Charset::Token => c.is_ascii_alphanumeric() || c == '_' || c == '-',
            Charset::Any => true,
        }
    }
}

/// The shapes this scanner knows.
///
/// Short, and meant to stay short. Every entry is a prefix its issuer assigned
/// and publishes, so a match is evidence rather than a guess.
pub const RULES: &[Rule] = &[
    Rule {
        id: "awsAccessKey",
        prefix: "AKIA",
        min_tail: 16,
        tail: Charset::UpperAlphanumeric,
    },
    // The header is the whole evidence; what follows is base64 across lines.
    Rule {
        id: "privateKey",
        prefix: "-----BEGIN ",
        min_tail: 0,
        tail: Charset::Any,
    },
    Rule {
        id: "githubToken",
        prefix: "ghp_",
        min_tail: 36,
        tail: Charset::Alphanumeric,
    },
    Rule {
        id: "githubToken",
        prefix: "github_pat_",
        min_tail: 20,
        tail: Charset::Token,
    },
    Rule {
        id: "slackToken",
        prefix: "xoxb-",
        min_tail: 10,
        tail: Charset::Token,
    },
    Rule {
        id: "slackToken",
        prefix: "xoxp-",
        min_tail: 10,
        tail: Charset::Token,
    },
    Rule {
        id: "stripeLiveKey",
        prefix: "sk_live_",
        min_tail: 16,
        tail: Charset::Alphanumeric,
    },
    Rule {
        id: "googleApiKey",
        prefix: "AIza",
        min_tail: 35,
        tail: Charset::Token,
    },
    Rule {
        id: "sendgridKey",
        prefix: "SG.",
        min_tail: 20,
        tail: Charset::Token,
    },
    Rule {
        id: "openaiKey",
        prefix: "sk-proj-",
        min_tail: 20,
        tail: Charset::Token,
    },
];

/// The `-----BEGIN ` shapes that are actually a key, so a certificate and a
/// commit signature do not become findings.
const PRIVATE_KEY_BLOCKS: [&str; 5] = [
    "-----BEGIN PRIVATE KEY",
    "-----BEGIN RSA PRIVATE KEY",
    "-----BEGIN EC PRIVATE KEY",
    "-----BEGIN DSA PRIVATE KEY",
    "-----BEGIN OPENSSH PRIVATE KEY",
];

/// Which rule this text matches, if any.
///
/// Scans for the prefix anywhere in the text rather than at the start: a
/// credential is usually surrounded by quotes, a YAML key or an assignment.
pub fn matches(text: &str) -> Option<&'static str> {
    for rule in RULES {
        let mut from = 0;
        while let Some(at) = text[from..].find(rule.prefix) {
            let start = from + at;
            let tail = &text[start + rule.prefix.len()..];

            if rule.id == "privateKey" {
                if PRIVATE_KEY_BLOCKS
                    .iter()
                    .any(|block| text[start..].starts_with(block))
                {
                    return Some(rule.id);
                }
            } else {
                let run = tail.chars().take_while(|c| rule.tail.accepts(*c)).count();
                if run >= rule.min_tail {
                    return Some(rule.id);
                }
            }
            from = start + rule.prefix.len();
        }
    }
    None
}

/// The credential-shaped run inside a line, so a fingerprint identifies the
/// *value* rather than the punctuation around it.
///
/// The same key written `AWS_KEY=AKIA…` in one file and `key: "AKIA…"` in
/// another has to produce one fingerprint, or the report says two secrets where
/// there is one — which is the opposite of what the fingerprint is for.
pub fn matched_value(text: &str) -> Option<String> {
    for rule in RULES {
        let mut from = 0;
        while let Some(at) = text[from..].find(rule.prefix) {
            let start = from + at;
            let tail = &text[start + rule.prefix.len()..];

            if rule.id == "privateKey" {
                if PRIVATE_KEY_BLOCKS
                    .iter()
                    .any(|block| text[start..].starts_with(block))
                {
                    // The header is the evidence and the key itself is on the
                    // lines below, which this never reads — so the header is
                    // what identifies it, and every private key in one file
                    // shares a fingerprint. That is honest: what was found is
                    // "a private key is in here".
                    return Some(text[start..].to_string());
                }
            } else {
                let run: String = tail.chars().take_while(|c| rule.tail.accepts(*c)).collect();
                if run.chars().count() >= rule.min_tail {
                    return Some(format!("{}{run}", rule.prefix));
                }
            }
            from = start + rule.prefix.len();
        }
    }
    None
}

/// Where a finding was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// The workspace's own `.env`.
    Env,
    /// A file git is tracking, which is a file that has been or will be pushed.
    Tracked,
}

/// One credential in one place.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// The rule that matched, or `unstoredSecret` for the key-name net.
    pub id: &'static str,
    pub source: Source,
    /// The `.env` key, or the path relative to the repository.
    pub subject: String,
    /// The line, for a file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// The first twelve hex characters of the value's sha256.
    ///
    /// What makes a finding usable without the value: two rows carrying one
    /// fingerprint are one secret in two places, which is the difference
    /// between "rotate this key" and "rotate these two". It identifies and does
    /// not reveal — twelve characters of a sha256 cannot be walked back to a
    /// password.
    pub fingerprint: String,
    /// The ends of the value, and only when there are enough of them.
    ///
    /// So a person can recognise which key this is among the four in their
    /// password manager. Masked whole below [`PREVIEW_MIN`], because four
    /// characters of a six-character password is the password.
    pub preview: String,
    /// The path this was found in has been committed at some point, on some
    /// branch — so the value is in the history whether or not it is in the
    /// working tree now. Asked by path, never by value.
    #[serde(default)]
    pub in_history: bool,
}

/// Shorter than this and a preview is masked whole.
///
/// Sixteen. Four characters at each end of a sixteen-character value is half of
/// it, which is where "recognisable" stops and "disclosed" starts.
pub const PREVIEW_MIN: usize = 16;

/// Twelve hex characters of the value's sha256.
///
/// Enough that two different secrets colliding is not a thing that happens on
/// one laptop, and short enough to read out loud. Never the whole digest: a
/// full sha256 of a short or guessable value is a rainbow-table lookup away
/// from being the value.
pub fn fingerprint(value: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(value.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `AKIA…MPLE`, or `••••` when the value is too short to show any of.
pub fn preview(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() < PREVIEW_MIN {
        return "••••".to_string();
    }
    let head: String = chars.iter().take(4).collect();
    let tail: String = chars.iter().skip(chars.len() - 4).collect();
    format!("{head}…{tail}")
}

/// What the scan looked at and what it found.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub findings: Vec<Finding>,
    /// How many tracked files were read.
    pub scanned: usize,
    /// Files skipped for being too large or not text, and files not reached
    /// because the cap bit. Reported rather than silently dropped: a scan that
    /// stopped early and said nothing reads as a clean repository.
    pub skipped: usize,
    /// True when the file cap stopped the scan before the end of the list.
    pub truncated: bool,
    /// `.env` is tracked by git — the finding that outranks every other one,
    /// because it means every value in it is in the history whatever their
    /// shape.
    pub env_tracked: bool,
    /// `.env` has been committed at some point, on some branch.
    ///
    /// Asked separately from [`Self::env_tracked`] because they are different
    /// facts with different repairs, and the second one is the one people get
    /// wrong: untracking the file today does **not** take it out of the
    /// history. A repository where this is true and `env_tracked` is false is
    /// one somebody has already half-fixed, and the half that is left — rotate
    /// what was in it — is the half that matters.
    pub env_in_history: bool,
}

/// The `.env` half: a value that looks like a credential, and a key whose name
/// says it holds one while its value is still sitting in the file.
///
/// `stored` is the set of keys already moved to the keystore — those are
/// references, not secrets, and flagging them would be telling somebody off for
/// having done the thing this app asked them to do.
pub fn scan_env(vars: &std::collections::BTreeMap<String, String>) -> Vec<Finding> {
    let mut out = Vec::new();

    for (key, value) in vars {
        if value.is_empty() || crate::secrets::is_reference(value) {
            continue;
        }

        if let Some(id) = matches(value) {
            out.push(Finding {
                id,
                source: Source::Env,
                subject: key.clone(),
                line: None,
                fingerprint: fingerprint(value),
                preview: preview(value),
                in_history: false,
            });
            continue;
        }

        // The second net. A key named for a credential, holding something that
        // is not a keystore reference, is a credential in a text file — which
        // is the state `secrets.rs` exists to end and could not report.
        if crate::config::Env::is_secret(key) && crate::secrets::is_movable(key) {
            out.push(Finding {
                id: "unstoredSecret",
                source: Source::Env,
                subject: key.clone(),
                line: None,
                fingerprint: fingerprint(value),
                preview: preview(value),
                in_history: false,
            });
        }
    }

    out
}

/// How many tracked files one scan will read.
///
/// Two thousand, and a file bigger than this is skipped. Both are here because
/// this runs on somebody's repository and a scanner that takes a minute is one
/// they cancel — and because a committed 40 MB fixture is not where a
/// credential hides.
pub const MAX_FILES: usize = 2_000;
pub const MAX_FILE_BYTES: u64 = 512 * 1024;

/// Read one file and report the lines that match.
///
/// Binary is skipped by looking for a NUL in the first block, which is what
/// git itself does. A JPEG that happens to contain `AKIA` followed by sixteen
/// uppercase bytes is not an AWS key, and reporting it would be the first step
/// towards a scanner nobody reads.
pub fn scan_file(path: &Path, relative: &str) -> Option<Vec<Finding>> {
    let size = std::fs::metadata(path).ok()?.len();
    if size > MAX_FILE_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.iter().take(8_000).any(|b| *b == 0) {
        return None;
    }
    let text = String::from_utf8_lossy(&bytes);

    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if let Some(id) = matches(line) {
            // The matched run, not the whole line: a fingerprint of the line
            // would change with the quoting around the value, so the same key
            // in `.env` and in a YAML file would look like two secrets.
            let value = matched_value(line).unwrap_or_default();
            out.push(Finding {
                id,
                source: Source::Tracked,
                subject: relative.to_string(),
                line: Some(index as u32 + 1),
                fingerprint: fingerprint(&value),
                preview: preview(&value),
                in_history: false,
            });
            // One finding a file. A private key is fifty lines and reporting
            // fifty findings for it would bury every other file in the report.
            break;
        }
    }
    Some(out)
}

/// Hold one repository up to the light.
///
/// Two questions, and the second one outranks the first: what is in `.env`, and
/// what is in a file git is tracking. A credential in `.env` is on this
/// machine; a credential in a tracked file is on every machine that has ever
/// cloned it, and in the history whether or not somebody deletes it tomorrow.
///
/// `git ls-files` rather than a directory walk, and that is not an
/// optimisation: a walk would read `node_modules`, `vendor` and every build
/// directory — thousands of files nobody committed and nobody will push. What
/// is tracked is exactly what leaves the machine.
pub async fn scan(root: &Path, dir: Option<&Path>) -> Report {
    let mut report = Report::default();

    if let Ok(env) = crate::config::Env::load(root) {
        report.findings.extend(scan_env(env.raw()));
    }

    let Some(dir) = dir.filter(|d| crate::worktree::is_repository(d)) else {
        return report;
    };

    let Ok(out) = tokio::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(dir)
        .output()
        .await
    else {
        return report;
    };

    let listing = String::from_utf8_lossy(&out.stdout);
    let tracked: Vec<&str> = listing.split('\0').filter(|p| !p.is_empty()).collect();

    // Asked whatever `ls-files` said, because the two answers are different and
    // the second one survives the first being fixed.
    report.env_in_history = ever_committed(dir, ".env").await;

    // The finding that outranks every other one: it means every value in the
    // file is in the history whatever their shape, so no rule has to match for
    // this to be a problem.
    report.env_tracked = tracked.iter().any(|p| *p == ".env" || p.ends_with("/.env"));

    for relative in tracked.iter().take(MAX_FILES) {
        match scan_file(&dir.join(relative), relative) {
            Some(found) => {
                report.scanned += 1;
                report.findings.extend(found);
            }
            // Too big, unreadable or not text. Counted rather than passed over,
            // because a scan that skipped four hundred files and said nothing
            // reads as a clean repository.
            None => report.skipped += 1,
        }
    }

    if tracked.len() > MAX_FILES {
        report.truncated = true;
        report.skipped += tracked.len() - MAX_FILES;
    }

    // One question per finding, and only for the tracked half: a `.env` value
    // on this machine is not in anybody's history by being there. Bounded by
    // the number of findings, which is a number nobody wants to be large.
    for index in 0..report.findings.len() {
        if report.findings[index].source != Source::Tracked {
            continue;
        }
        let path = report.findings[index].subject.clone();
        report.findings[index].in_history = ever_committed(dir, &path).await;
    }

    report
}

/// Has this path ever been committed, on any branch?
///
/// **By path, never by value.** `git log -S<secret>` would put the secret in
/// this process's argument list, where every other process on the machine can
/// read it out of `ps` — see this module's header. A path is not a secret, and
/// the answer is stronger anyway: a file that was committed and later deleted
/// is still in the history with everything that was in it, which a value search
/// would miss the moment somebody rotated half of it.
///
/// `--all` because a secret committed on a branch nobody merged is still in the
/// repository, and `-n 1` because the question is whether there is one, not how
/// many.
async fn ever_committed(dir: &Path, path: &str) -> bool {
    tokio::process::Command::new("git")
        .args(["log", "--all", "--format=%H", "-n", "1", "--"])
        .arg(path)
        .current_dir(dir)
        .output()
        .await
        .map(|out| out.status.success() && !out.stdout.is_empty())
        .unwrap_or(false)
}

// ------------------------------------------------------------- the repair

/// What untracking did.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Untracked {
    /// An index entry was actually removed. False when it was already
    /// untracked, which is a success and not a failure.
    pub untracked: bool,
    /// `.gitignore` now covers it, whether this wrote the line or it was
    /// already there.
    pub ignored: bool,
    /// This appended the line.
    pub gitignore_written: bool,
    /// The example file this wrote, when it wrote one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_written: Option<String>,
    /// How many keys the example carries. Zero when none was written.
    pub example_keys: usize,
    /// **The half that is left.** Untracking does not rewrite history: if the
    /// file was ever committed, every value that was in it is still in the
    /// repository and has to be rotated. Reported so the screen can say the
    /// sentence people most often skip.
    pub still_in_history: bool,
    /// Staged, not committed. `git rm --cached` writes the index; the removal
    /// leaves this machine when somebody commits and pushes it, and saying so
    /// is the difference between a fix and a fix somebody thinks they made.
    pub needs_commit: bool,
}

/// The standard repair for a tracked `.env`, in the order the standard does it.
///
/// ## Why this exists rather than a sentence telling somebody to do it
///
/// A finding people cannot act on is a finding they turn off. The three steps
/// are not hard and they are easy to get half right — the common half-fix is
/// deleting the file in a later commit, which removes it from the working tree
/// and leaves every value in the history.
///
/// So the app does the part that is mechanical and says the part that is not:
///
/// 1. **`git rm --cached`** — untracks the file and leaves it on disk. Not
///    `git rm`, which would delete the running stack's configuration.
/// 2. **`.gitignore`** — asked with `git check-ignore` rather than parsed,
///    because the answer depends on a chain of files this app is not the
///    authority on, and appended only when the answer is no.
/// 3. **`.env.example`** — the tracked half, which is what the file should
///    have been all along: the same keys, no values. Written only when there
///    is none; overwriting somebody's example with a generated one would throw
///    away comments and grouping they wrote for the next person.
///
/// And the fourth step is not this app's to take: **rotate what was in it.**
pub async fn untrack_env(dir: &Path) -> crate::error::Result<Untracked> {
    let mut out = Untracked {
        still_in_history: ever_committed(dir, ".env").await,
        ..Untracked::default()
    };
    let env = dir.join(".env");

    // `--ignore-unmatch` so a file that is already untracked is a success. The
    // caller pressed a button that says "untrack it"; it being untracked is
    // what they asked for.
    let removed = tokio::process::Command::new("git")
        .args(["rm", "--cached", "--ignore-unmatch", "--", ".env"])
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| crate::error::Error::io("running git rm --cached", e))?;

    if !removed.status.success() {
        return Err(crate::error::Error::new(
            crate::error::Code::IoError,
            String::from_utf8_lossy(&removed.stderr).trim().to_string(),
        ));
    }
    // git prints `rm '.env'` when it removed an entry and nothing when there
    // was none, which is the only way to tell the two apart.
    out.untracked = !removed.stdout.is_empty();
    out.needs_commit = out.untracked;

    // Asked, not parsed: `.gitignore`, `.git/info/exclude` and a global
    // excludes file all take part, and this app is not the authority on any of
    // them. `check-ignore` exits 0 when the path is ignored.
    let ignored = tokio::process::Command::new("git")
        .args(["check-ignore", "-q", "--", ".env"])
        .current_dir(dir)
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false);

    out.ignored = ignored;
    if !ignored {
        let file = dir.join(".gitignore");
        let existing = std::fs::read_to_string(&file).unwrap_or_default();
        // A file that does not end in a newline would otherwise gain
        // `something.env` rather than a line of its own.
        let separator = if existing.is_empty() || existing.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        let addition =
            format!("{separator}\n# Added by StackVo: this file holds credentials.\n.env\n");
        std::fs::write(&file, format!("{existing}{addition}"))
            .map_err(|e| crate::error::Error::io(format!("writing {}", file.display()), e))?;
        out.ignored = true;
        out.gitignore_written = true;
        out.needs_commit = true;
    }

    // The tracked half, and only when there is none.
    let example = dir.join(".env.example");
    if !example.exists() {
        if let Ok(text) = std::fs::read_to_string(&env) {
            let keys = example_from(&text);
            if !keys.is_empty() {
                std::fs::write(&example, keys.join("\n") + "\n").map_err(|e| {
                    crate::error::Error::io(format!("writing {}", example.display()), e)
                })?;
                out.example_keys = keys.iter().filter(|l| l.ends_with('=')).count();
                out.example_written = Some(example.display().to_string());
                out.needs_commit = true;
            }
        }
    }

    Ok(out)
}

/// The same file with every value removed.
///
/// Comments and blank lines are kept, because an example file is documentation
/// and the grouping is most of what makes it readable. A line that is not an
/// assignment is passed through; a line that is keeps its key, its `=`, and
/// nothing else.
///
/// `export FOO=bar` keeps its `export`: some projects source the file from a
/// shell, and an example that would not source is an example nobody can use.
pub fn example_from(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut saw_assignment = false;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            out.push(line.to_string());
            continue;
        }
        match line.split_once('=') {
            Some((key, _)) => {
                saw_assignment = true;
                out.push(format!("{key}="));
            }
            None => out.push(line.to_string()),
        }
    }

    // A file with comments and no assignments is not an example of anything.
    if saw_assignment {
        out
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vendors_own_prefix_is_what_makes_a_match() {
        assert_eq!(matches("AKIAIOSFODNN7EXAMPLE"), Some("awsAccessKey"));
        assert_eq!(
            matches("aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\""),
            Some("awsAccessKey"),
            "a credential is usually inside quotes and an assignment"
        );
        assert_eq!(
            matches("ghp_1234567890abcdefghijklmnopqrstuvwxyz"),
            Some("githubToken")
        );
        assert_eq!(matches("xoxb-123456789012-abcdefghij"), Some("slackToken"));
        // Assembled rather than written out, and this is not tidiness — it is
        // the rule under test, applied to this file. Every other fixture here
        // is a shape its issuer publishes as an example and no scanner treats
        // as live; a `sk_live_` prefix followed by its own alphabet is not, and
        // a scanner worth shipping flags it wherever it appears, including in
        // the tests of a scanner. Splitting the prefix from the body keeps the
        // assertion exactly the same and keeps this repository from being the
        // one place its own rule does not hold. Do not join these back up.
        let stripe = format!("sk_live_{}", "4eC39HqLyjWDarjtT1zdp7dc");
        assert_eq!(matches(&stripe), Some("stripeLiveKey"));
        assert_eq!(
            matches("-----BEGIN OPENSSH PRIVATE KEY-----"),
            Some("privateKey")
        );
    }

    /// The rule that keeps the feature usable. A scanner people learn to
    /// ignore is worse than no scanner.
    #[test]
    fn the_shapes_that_look_like_credentials_and_are_not_do_not_match() {
        for innocent in [
            // A certificate is not a private key, and this repository ships
            // several.
            "-----BEGIN CERTIFICATE-----",
            "-----BEGIN PGP SIGNATURE-----",
            // A test placeholder that stops short of the length.
            "AKIASHORT",
            "ghp_tooshort",
            // The prefix as prose.
            "the sk_live_ prefix marks a Stripe live key",
            // A sha256, which is what a lockfile is full of.
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            // Base64 of an image, which minified assets are full of.
            "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
            "",
        ] {
            assert_eq!(matches(innocent), None, "{innocent:?} matched");
        }
    }

    /// The two halves of "usable without carrying the value".
    #[test]
    fn a_finding_identifies_a_secret_without_revealing_it() {
        let key = "AKIAIOSFODNN7EXAMPLE";

        // The same value, wherever it was found, is one secret.
        assert_eq!(fingerprint(key), fingerprint(key));
        assert_ne!(fingerprint(key), fingerprint("AKIAIOSFODNN7EXAMPLF"));
        assert_eq!(fingerprint(key).len(), 12, "short enough to read out loud");
        assert!(
            !fingerprint(key).contains(&key[..8]),
            "a fingerprint that carried the value would be the value"
        );

        // Recognisable, not readable.
        let shown = preview(key);
        assert!(
            shown.starts_with("AKIA") && shown.ends_with("MPLE"),
            "{shown}"
        );
        assert!(!shown.contains("IOSFODNN7EXA"), "{shown}");

        // Short values show nothing: four characters at each end of a
        // sixteen-character value is half of it.
        assert_eq!(preview("hunter2"), "••••");
        assert_eq!(preview(&"a".repeat(PREVIEW_MIN - 1)), "••••");
        assert!(preview(&"a".repeat(PREVIEW_MIN)).contains('…'));
    }

    /// The same key in two files has to fingerprint the same, or the report
    /// says two secrets where there is one — which is the opposite of what the
    /// fingerprint is for.
    #[test]
    fn the_punctuation_around_a_value_is_not_part_of_it() {
        let bare = matched_value("AKIAIOSFODNN7EXAMPLE").unwrap();
        let assigned = matched_value("AWS_KEY=AKIAIOSFODNN7EXAMPLE").unwrap();
        let quoted = matched_value("  key: \"AKIAIOSFODNN7EXAMPLE\"").unwrap();

        assert_eq!(bare, "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(fingerprint(&bare), fingerprint(&assigned));
        assert_eq!(fingerprint(&bare), fingerprint(&quoted));
    }

    /// The example file is documentation, so what makes it readable survives.
    #[test]
    fn an_example_keeps_every_key_its_comments_and_no_values() {
        let example = example_from(
            "# Database\nDB_HOST=localhost\nDB_PASSWORD=hunter2\n\n# Mail\nexport MAIL_KEY=SG.abc\nnot an assignment\n",
        );

        assert_eq!(
            example,
            vec![
                "# Database",
                "DB_HOST=",
                "DB_PASSWORD=",
                "",
                "# Mail",
                "export MAIL_KEY=",
                "not an assignment",
            ],
            "the grouping is most of what makes an example readable"
        );
        assert!(
            !example.join("\n").contains("hunter2"),
            "a value survived into the file that is meant to be committed"
        );

        // A file with no assignments is not an example of anything.
        assert!(example_from("# just a note\n").is_empty());
        assert!(example_from("").is_empty());
    }

    fn vars(pairs: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Two nets that fail differently: the value's shape, and the key's name.
    #[test]
    fn a_credential_is_caught_by_its_shape_whatever_the_key_is_called() {
        let found = scan_env(&vars(&[
            // A name no suffix rule would ever match — `preset.rs`'s exact
            // warning, made into a test.
            ("MY_FAVOURITE_THING", "AKIAIOSFODNN7EXAMPLE"),
            ("APP_ENV", "local"),
        ]));

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].id, "awsAccessKey");
        assert_eq!(found[0].subject, "MY_FAVOURITE_THING");
        // The value is never carried. A report that quotes the secret is a
        // second copy of it, in a place people paste into chat windows — and
        // the masked preview must not put it back together either.
        assert!(!format!("{found:?}").contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(!found[0].preview.contains("IOSFODNN7EXA"));
        // What is carried instead is enough to act on: which secret this is.
        assert_eq!(found[0].fingerprint, fingerprint("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn a_key_already_in_the_keystore_is_not_a_finding() {
        let found = scan_env(&vars(&[
            (
                "SERVICE_MYSQL_ROOT_PASSWORD",
                "keychain:SERVICE_MYSQL_ROOT_PASSWORD@a1b2c3d4",
            ),
            ("SERVICE_REDIS_PASSWORD", "hunter2"),
            // Empty is not a secret sitting anywhere.
            ("SERVICE_PG_PASSWORD", ""),
        ]));

        let subjects: Vec<&str> = found.iter().map(|f| f.subject.as_str()).collect();
        assert_eq!(
            subjects,
            vec!["SERVICE_REDIS_PASSWORD"],
            "telling somebody off for having moved a key is how a feature gets switched off"
        );
        assert_eq!(found[0].id, "unstoredSecret");
    }

    #[test]
    fn a_file_reports_the_line_and_stops_at_one_finding() {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-leaks-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let file = dir.join("deploy.sh");
        std::fs::write(
            &file,
            "#!/bin/sh\nexport AWS_KEY=AKIAIOSFODNN7EXAMPLE\nexport OTHER=AKIAIOSFODNN7EXAMPLE\n",
        )
        .unwrap();

        let found = scan_file(&file, "deploy.sh").expect("a readable text file");
        assert_eq!(found.len(), 1, "one finding a file: {found:?}");
        assert_eq!(found[0].line, Some(2));
        assert_eq!(found[0].source, Source::Tracked);

        // Binary is not scanned at all — `None` rather than "no findings", so
        // the caller counts it as skipped instead of reporting a file it never
        // read as clean. A JPEG with those bytes in it is not a credential.
        let binary = dir.join("logo.png");
        std::fs::write(&binary, b"\x89PNG\x00\x00AKIAIOSFODNN7EXAMPLE").unwrap();
        assert!(scan_file(&binary, "logo.png").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
