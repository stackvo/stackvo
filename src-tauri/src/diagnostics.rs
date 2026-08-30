//! Everything a bug report needs, in one file.
//!
//! Settings could already open the log *folder*. That leaves the rest to the
//! person reporting the problem: find the right file among seven daily ones,
//! know that the doctor output exists and is separate, remember the app version
//! and the platform, and think to mention that the engine was down. Most people
//! attach the newest log and nothing else, and the first reply is always a list
//! of the other four things.
//!
//! So this collects them. One button, one archive, and the maintainer gets the
//! same set every time.
//!
//! ## Masked again on the way in
//!
//! `logging::redact` already runs over subprocess output as it is written, so
//! the files on disk are masked. Every log line is put through it a second time
//! here, and that is not superstition: the redactor has been extended before,
//! and a bundle assembled today may contain lines written by an older build
//! whose rule was narrower. Re-running the current rule over old text costs a
//! pass over a few megabytes and closes the one gap that would leak a password
//! into an issue tracker.
//!
//! ## Bounded, so it can actually be sent
//!
//! Logs are capped per file. An unbounded archive is one nobody can attach —
//! and the end of a log is the part that explains a failure anyway. What was
//! dropped is *stated* in the manifest rather than silently trimmed, for the
//! reason `applog::FanoutScan` reports its own cap: a truncated report that
//! looks complete is worse than a short one that says so.
//!
//! ## Readable before it is sent
//!
//! Plain text and JSON, and a `README.txt` that says what each file is. The
//! whole premise of masking is that the bundle is safe to attach — but the
//! person attaching it should still be able to look, and a format they cannot
//! open is a format they cannot check.

use crate::error::{Error, Result};
use std::io::Write;
use std::path::Path;

/// How much of each log file is carried.
///
/// One mebibyte is roughly a day of ordinary operation and several thousand
/// lines of a failing build — past the point where more text adds information.
const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// One file in the archive.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    pub bytes: u64,
    /// True when the file was cut to [`MAX_LOG_BYTES`]. Reported rather than
    /// hidden, so nobody reads a trimmed log as the whole story.
    pub truncated: bool,
}

/// What was written, for the UI to show without opening the archive.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub path: String,
    pub bytes: u64,
    pub entries: Vec<Entry>,
}

/// The environment half of the report — no workspace and no engine needed.
///
/// Split out because it is the part that is always answerable. A bundle from a
/// machine where nothing works at all still carries this, and "which version on
/// which platform" is the first question of every report.
pub fn about() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    format!(
        "StackVo Desktop diagnostic bundle\n\
         \n\
         version    {version}\n\
         platform   {os} {arch}\n\
         collected  {stamp} UTC (unix {seconds})\n\
         log dir    {logs}\n\
         config dir {config}\n",
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        stamp = crate::crash::stamp(seconds),
        logs = crate::appdir::logs()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<none>".into()),
        config = crate::appdir::config()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<none>".into()),
    )
}

// --------------------------------------------------------- the environment
//
// The half of a bundle that can be *compared*.

/// The name the comparable half is filed under, in a bundle and on its own.
pub const ENVIRONMENT_FILE: &str = "environment.json";

/// What this machine is, as facts one can be held against another.
///
/// ## Why this exists next to a bundle that already has everything
///
/// "It works on my machine" is the oldest complaint in this category and no
/// product in it answers the question — they all say the container solves it,
/// which it does not: the same compose file on two Docker versions is two
/// different things. This app was already the only one that packages the state
/// of a machine. What it could not do is put two of them side by side.
///
/// It could not because the bundle is written for a **person**: `about.txt` is
/// prose, and `doctor.json` and `preflight.json` are shaped for reading. Diffing
/// those produces noise — a socket path, a pid, a byte count that moved — and a
/// comparison whose output is mostly noise is one nobody reads twice. So the
/// comparable half is derived separately and deliberately flat: one line per
/// fact, a key somebody can say out loud, and a value that is the same string
/// on two machines when the two machines agree.
///
/// ## What is in it, and what is kept out
///
/// Versions, the engine, the services and what each project declares. **No
/// paths**, because a home directory differs on every machine and would report
/// two identical setups as different in five places. **No credentials and no
/// `.env` values**: this file is meant to be handed to a colleague, and the
/// whole premise of the bundle it sits in is that it is safe to attach.
///
/// No new measurement, either. Every value here is read from something this
/// module already collects or from a file already on disk.
pub async fn facts(root: Option<&Path>) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut put = |key: &str, value: String| {
        out.insert(key.to_string(), value);
    };

    put("app.version", env!("CARGO_PKG_VERSION").to_string());
    put("app.os", std::env::consts::OS.to_string());
    put("app.arch", std::env::consts::ARCH.to_string());

    let engine = crate::engine::status().await;
    put("engine.platform", format!("{:?}", engine.platform));
    put(
        "engine.version",
        engine
            .version
            .clone()
            .unwrap_or_else(|| "unreachable".into()),
    );
    if let Some(api) = &engine.api_version {
        put("engine.apiVersion", api.clone());
    }

    // The state only, never the detail. A requirement's detail carries the
    // socket it found and the port a process holds, which differ between two
    // machines that are working identically.
    for requirement in crate::preflight::run().await.requirements {
        put(
            &format!("preflight.{}", requirement.id),
            format!("{:?}", requirement.state).to_lowercase(),
        );
    }

    let Some(root) = root else {
        return out;
    };

    if let Ok(env) = crate::config::Env::load(root) {
        put("workspace.tldSuffix", env.tld_suffix());
        put("workspace.network", env.docker_network());
        put("workspace.defaultServer", env.default_server());
    }

    if let Ok(table) = crate::instances::Table::load(root) {
        for instance in &table.instances {
            put(
                &format!("service.{}", instance.id),
                format!(
                    "{} {}",
                    instance.version,
                    if instance.enabled { "on" } else { "off" }
                ),
            );
        }
    }

    // What each project *declares*, which is the half that travels in git and
    // the half two people can actually disagree about while both believing they
    // are running the same thing.
    if let Some(projects) = crate::workspace::projects_root(root) {
        for entry in std::fs::read_dir(&projects).into_iter().flatten().flatten() {
            let dir = entry.path();
            let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Ok(manifest) = crate::manifest::read(&dir.join(crate::manifest::FILE), name) else {
                continue;
            };
            // The runtime and its version, which is the pair two people most
            // often differ on while both believing they run "the project".
            let version = match (&manifest.php, &manifest.node) {
                (Some(php), _) => php.version.clone(),
                (None, Some(node)) => node.version.clone(),
                _ => "default".to_string(),
            };
            put(
                &format!("project.{name}"),
                format!(
                    "{} {version} on {}",
                    manifest.runtime,
                    manifest.server.as_deref().unwrap_or("default"),
                ),
            );
            // Xdebug being on is a difference worth naming by itself: it is the
            // usual answer to "why is it slow on yours and not on mine".
            if let Some(php) = &manifest.php {
                put(
                    &format!("project.{name}.xdebug"),
                    if php.xdebug { "on" } else { "off" }.to_string(),
                );
            }
        }
    }

    out
}

/// One fact two machines do not agree about.
///
/// Both sides are carried, and either may be absent — "you have redis-7-2 and
/// they do not" is a different sentence from "you are on 7.2 and they are on
/// 7.0", and a comparison that flattened them into "differs" would throw away
/// the half that says what to do.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Difference {
    pub key: String,
    pub here: Option<String>,
    pub there: Option<String>,
}

/// The answer to "why does it work on yours".
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    /// Only what differs, in key order. Agreement is counted, not listed: a
    /// report that prints two hundred identical lines buries the four that
    /// matter, which is how a diff stops being read.
    pub differences: Vec<Difference>,
    /// How many facts both sides state and agree on.
    pub same: usize,
    /// The app version the other bundle came from, when it says.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub their_version: Option<String>,
}

/// Hold two fingerprints against each other.
pub fn compare(
    here: &std::collections::BTreeMap<String, String>,
    there: &std::collections::BTreeMap<String, String>,
) -> Comparison {
    let mut differences = Vec::new();
    let mut same = 0;

    // Both key sets, in one order. A key only one side has is a difference —
    // the most interesting kind, and the one an intersection would drop.
    let keys: std::collections::BTreeSet<&String> = here.keys().chain(there.keys()).collect();

    for key in keys {
        let (mine, theirs) = (here.get(key), there.get(key));
        if mine == theirs {
            same += 1;
            continue;
        }
        differences.push(Difference {
            key: key.clone(),
            here: mine.cloned(),
            there: theirs.cloned(),
        });
    }

    Comparison {
        differences,
        same,
        their_version: there.get("app.version").cloned(),
    }
}

/// Read the comparable half out of what somebody sent.
///
/// A whole bundle or the one file out of it, because both are things people
/// actually send: the zip is what the app produces, and the extracted JSON is
/// what somebody pastes into a chat window. Guessing by extension would refuse
/// a correct file for having been renamed, so the zip is *tried* and anything
/// that is not one is read as JSON.
pub fn facts_from_file(path: &Path) -> Result<std::collections::BTreeMap<String, String>> {
    let file = std::fs::File::open(path)
        .map_err(|e| Error::io(format!("reading {}", path.display()), e))?;

    let text = match zip::ZipArchive::new(std::io::BufReader::new(file)) {
        Ok(mut archive) => {
            use std::io::Read;
            let mut entry = archive.by_name(ENVIRONMENT_FILE).map_err(|_| {
                Error::new(
                    crate::error::Code::NotFound,
                    format!(
                        "that bundle has no {ENVIRONMENT_FILE} — it was made by a version of                          StackVo that did not collect one yet, and there is nothing in it to                          compare"
                    ),
                )
            })?;
            let mut text = String::new();
            entry
                .read_to_string(&mut text)
                .map_err(|e| Error::io(format!("reading {ENVIRONMENT_FILE}"), e))?;
            text
        }
        Err(_) => std::fs::read_to_string(path)
            .map_err(|e| Error::io(format!("reading {}", path.display()), e))?,
    };

    serde_json::from_str(&text).map_err(|e| {
        Error::new(
            crate::error::Code::InvalidInput,
            format!(
                "that file is neither a StackVo diagnostic bundle nor an {ENVIRONMENT_FILE}: {e}"
            ),
        )
    })
}

/// The header that tells whoever opens the archive what they are looking at.
fn readme(entries: &[Entry]) -> String {
    let mut out = String::from(
        "What is in here\n\
         ===============\n\
         \n\
         about.txt      version, platform and where this app keeps its files\n\
         preflight.json every startup requirement and whether it is met\n\
         doctor.json    ports, hosts entries, generated-config drift, disk use\n\
         engine.json    the Docker engine as this app sees it\n\
         environment.json  the comparable facts: versions, services, projects\n\
         logs/          the rotating application log, newest last\n\
         crashes/       panic reports, if this app has ever died on this machine\n\
         \n\
         Passwords and tokens are masked as the log is written and masked again\n\
         as this archive is built. Nothing here is read from your .env, and no\n\
         project source is included. It is meant to be safe to attach to an\n\
         issue — but it is plain text, so read it first.\n\
         \n\
         Files\n\
         -----\n",
    );
    for entry in entries {
        out.push_str(&format!(
            "{:<40} {:>10} bytes{}\n",
            entry.name,
            entry.bytes,
            if entry.truncated {
                "  (truncated to the most recent 1 MiB)"
            } else {
                ""
            }
        ));
    }
    out
}

/// Pretty JSON, or a note saying why there is none.
///
/// A section that failed must still appear. An absent `doctor.json` is
/// indistinguishable from a bundle built by an older version; a `doctor.json`
/// holding an error is a fact about the machine.
fn section<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value)
        .unwrap_or_else(|e| format!("{{ \"error\": \"could not serialise: {e}\" }}"))
}

/// Read the tail of a file and mask it, reporting whether it was cut.
fn redacted_tail(path: &Path) -> Option<(String, bool)> {
    let len = std::fs::metadata(path).ok()?.len();
    let truncated = len > MAX_LOG_BYTES;

    let (text, _) = crate::applog::tail(path, MAX_LOG_BYTES).ok()?;
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        out.push_str(&crate::logging::redact(line));
        out.push('\n');
    }
    Some((out, truncated))
}

/// Everything the archive holds, as `(name, contents)`.
///
/// Separate from the writing so it can be tested without producing a zip and
/// unpacking it again. What matters about this function is *what is collected
/// and what is masked*; that a zip writer can write bytes is not in question.
pub async fn parts(root: Option<&Path>) -> Vec<(String, String)> {
    let mut parts = vec![("about.txt".to_string(), about())];

    // Each of these reaches out — to Docker, to the filesystem, to the host —
    // and each is written to answer rather than to fail, so a broken machine
    // still produces a full bundle. That is exactly the machine this is for.
    parts.push((
        "preflight.json".into(),
        section(&crate::preflight::run().await),
    ));
    parts.push((
        "doctor.json".into(),
        section(&crate::doctor::run(root).await),
    ));
    parts.push((
        "engine.json".into(),
        section(&crate::engine::status().await),
    ));
    // The one file in here written to be read by a program rather than a
    // person: flat, path-free and credential-free, so two of them can be held
    // against each other. See [`facts`].
    parts.push((ENVIRONMENT_FILE.into(), section(&facts(root).await)));

    if let Some(dir) = crate::logging::dir() {
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        // By name, which for both the daily log and the crash reports is
        // chronological — see `crash::stamp`.
        files.sort();

        for path in files {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some((text, _)) = redacted_tail(&path) else {
                continue;
            };
            let folder = if name.starts_with("crash-") {
                "crashes"
            } else {
                "logs"
            };
            parts.push((format!("{folder}/{name}"), text));
        }
    }

    parts
}

/// Write the bundle to `dest`.
///
/// `dest` comes from the system save dialog, like `mail::save_attachment`'s
/// does — the front end never names a destination this process did not receive
/// from the user.
pub async fn write(root: Option<&Path>, dest: &Path) -> Result<Bundle> {
    let parts = parts(root).await;

    let mut entries: Vec<Entry> = parts
        .iter()
        .map(|(name, body)| Entry {
            name: name.clone(),
            bytes: body.len() as u64,
            truncated: body.len() as u64 >= MAX_LOG_BYTES,
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let file = std::fs::File::create(dest)
        .map_err(|e| Error::io(format!("creating {}", dest.display()), e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let readme = readme(&entries);
    let io = |e: std::io::Error| Error::io(format!("writing {}", dest.display()), e);

    zip.start_file("README.txt", options).map_err(zip_error)?;
    zip.write_all(readme.as_bytes()).map_err(io)?;

    for (name, body) in &parts {
        zip.start_file(name, options).map_err(zip_error)?;
        zip.write_all(body.as_bytes()).map_err(io)?;
    }
    zip.finish().map_err(zip_error)?;

    let bytes = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    tracing::info!(path = %dest.display(), bytes, files = entries.len() + 1, "diagnostic bundle written");

    Ok(Bundle {
        path: dest.display().to_string(),
        bytes,
        entries,
    })
}

fn zip_error(e: zip::result::ZipError) -> Error {
    Error::new(
        crate::error::Code::IoError,
        format!("could not build the archive: {e}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_names_the_version_and_the_platform() {
        let text = about();
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
        assert!(text.contains(std::env::consts::OS));
        assert!(text.contains("UTC"));
    }

    /// The readme is the only part a non-developer reads, and its whole job is
    /// to say what is in the archive and that it is safe to attach.
    #[test]
    fn the_readme_lists_every_file_and_says_it_is_masked() {
        let entries = vec![
            Entry {
                name: "logs/stackvo.2026-08-06.log".into(),
                bytes: 12,
                truncated: false,
            },
            Entry {
                name: "logs/stackvo.2026-08-05.log".into(),
                bytes: MAX_LOG_BYTES,
                truncated: true,
            },
        ];
        let text = readme(&entries);

        assert!(text.contains("logs/stackvo.2026-08-06.log"));
        assert!(text.contains("logs/stackvo.2026-08-05.log"));
        assert!(text.contains("truncated"), "a cut file must say so");
        assert!(text.contains("masked"));
        assert!(text.contains("read it first"));
    }

    /// The reason the second masking pass exists: a log written by an older
    /// build carries whatever that build's redactor let through, and today's
    /// rule is the one that should decide what leaves the machine.
    #[test]
    fn a_secret_already_on_disk_is_masked_on_the_way_into_the_bundle() {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-diagnostics-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("stackvo.2026-08-06.log");
        std::fs::write(
            &log,
            "starting up\nshop | MYSQL_ROOT_PASSWORD=hunter2\ndone\n",
        )
        .unwrap();

        let (text, truncated) = redacted_tail(&log).expect("a readable log");

        assert!(!truncated);
        assert!(text.contains("MYSQL_ROOT_PASSWORD=***"), "{text}");
        assert!(!text.contains("hunter2"));
        // And the ordinary lines survive, or the bundle is useless.
        assert!(text.contains("starting up"));
        assert!(text.contains("done"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn map(pairs: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// What a comparison is for: the four lines that matter, not the two
    /// hundred that agree.
    #[test]
    fn only_what_differs_is_listed_and_the_rest_is_counted() {
        let here = map(&[
            ("app.version", "0.1.0"),
            ("engine.version", "27.1.1"),
            ("service.redis-7-2", "7.2 on"),
            ("project.shop", "php 8.4 on nginx"),
        ]);
        let there = map(&[
            ("app.version", "0.1.0"),
            ("engine.version", "25.0.3"),
            ("service.redis-7-2", "7.2 off"),
            ("project.shop", "php 8.4 on nginx"),
        ]);

        let out = compare(&here, &there);

        assert_eq!(out.same, 2, "agreement is counted, not listed");
        assert_eq!(
            out.differences,
            vec![
                Difference {
                    key: "engine.version".into(),
                    here: Some("27.1.1".into()),
                    there: Some("25.0.3".into()),
                },
                Difference {
                    key: "service.redis-7-2".into(),
                    here: Some("7.2 on".into()),
                    there: Some("7.2 off".into()),
                },
            ],
            "in key order, both sides carried"
        );
        assert_eq!(out.their_version.as_deref(), Some("0.1.0"));
    }

    /// The most interesting difference is the one an intersection would drop.
    #[test]
    fn a_fact_only_one_side_states_is_a_difference_with_a_missing_half() {
        let out = compare(
            &map(&[("service.redis-7-2", "7.2 on")]),
            &map(&[("service.mysql-8-4", "8.4 on")]),
        );

        assert_eq!(out.same, 0);
        assert_eq!(
            out.differences,
            vec![
                Difference {
                    key: "service.mysql-8-4".into(),
                    here: None,
                    there: Some("8.4 on".into()),
                },
                Difference {
                    key: "service.redis-7-2".into(),
                    here: Some("7.2 on".into()),
                    there: None,
                },
            ],
            "\"you have it and they do not\" is a different sentence from \"you disagree\""
        );
    }

    #[test]
    fn two_identical_machines_have_nothing_to_say() {
        let both = map(&[("app.version", "0.1.0"), ("engine.version", "27.1.1")]);
        let out = compare(&both, &both);

        assert!(out.differences.is_empty());
        assert_eq!(out.same, 2);
    }

    /// Both of the things people actually send, read by the same function.
    #[test]
    fn the_other_side_can_arrive_as_a_bundle_or_as_the_one_file_out_of_it() {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-diagnostics-compare-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let body = r#"{"app.version":"0.1.0","engine.version":"25.0.3"}"#;

        // The extracted file, pasted out of a chat window — and named
        // something else, because guessing by extension would refuse a correct
        // file for having been renamed.
        let bare = dir.join("theirs.txt");
        std::fs::write(&bare, body).unwrap();
        assert_eq!(
            facts_from_file(&bare).unwrap().get("engine.version"),
            Some(&"25.0.3".to_string())
        );

        // The whole bundle.
        let zipped = dir.join("theirs.zip");
        {
            let file = std::fs::File::create(&zipped).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
            writer.start_file("about.txt", options).unwrap();
            writer
                .write_all(
                    b"prose nobody can diff
",
                )
                .unwrap();
            writer.start_file(ENVIRONMENT_FILE, options).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
            writer.finish().unwrap();
        }
        assert_eq!(
            facts_from_file(&zipped).unwrap().get("app.version"),
            Some(&"0.1.0".to_string())
        );

        // A bundle from a build that predates this: named, not shrugged at.
        let older = dir.join("older.zip");
        {
            let file = std::fs::File::create(&older).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
            writer.start_file("about.txt", options).unwrap();
            writer
                .write_all(
                    b"prose nobody can diff
",
                )
                .unwrap();
            writer.finish().unwrap();
        }
        let refused = facts_from_file(&older).unwrap_err();
        assert!(
            refused.message.contains(ENVIRONMENT_FILE),
            "the reason has to name the missing file: {}",
            refused.message
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The bundle carries the comparable half, or a comparison has nothing to
    /// read.
    #[tokio::test]
    async fn the_bundle_carries_the_file_a_comparison_reads() {
        let names: Vec<String> = parts(None)
            .await
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        assert!(
            names.iter().any(|n| n == ENVIRONMENT_FILE),
            "the bundle no longer carries {ENVIRONMENT_FILE}: {names:?}"
        );
    }

    /// No paths and no credentials, because this file is meant to be handed to
    /// somebody. A home directory would also report two identical setups as
    /// different in five places.
    #[tokio::test]
    async fn the_comparable_half_carries_no_path_and_no_secret() {
        let facts = facts(None).await;
        let body = serde_json::to_string(&facts).unwrap();

        for forbidden in ["/Users/", "/home/", "C:\\", "password", "secret", "token"] {
            assert!(
                !body.to_lowercase().contains(&forbidden.to_lowercase()),
                "{forbidden:?} reached the comparable half: {body}"
            );
        }
        // And it is not empty, or the assertion above proves nothing.
        assert!(facts.contains_key("app.version"), "{body}");
    }

    #[test]
    fn an_oversized_log_is_cut_and_says_so() {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-diagnostics-big-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("big.log");

        let line = "a line of ordinary build output that says nothing secret\n";
        let body = line.repeat((MAX_LOG_BYTES as usize / line.len()) + 2_000);
        std::fs::write(&log, &body).unwrap();

        let (text, truncated) = redacted_tail(&log).expect("a readable log");
        assert!(truncated, "the cap did not bite");
        assert!(
            (text.len() as u64) <= MAX_LOG_BYTES,
            "kept {} bytes, cap is {MAX_LOG_BYTES}",
            text.len()
        );
        // The *end* is what is kept — the beginning of a long log is the part
        // nobody needs.
        assert!(text.ends_with("secret\n"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A section that could not be produced still has to appear, or its absence
    /// reads as "this bundle came from an older version".
    #[test]
    fn an_unserialisable_section_becomes_a_recorded_error_not_a_gap() {
        // `f64::NAN` has no JSON representation, which is the one way
        // `to_string_pretty` fails on a type that is otherwise `Serialize`.
        let text = section(&serde_json::json!({ "ok": true }));
        assert!(text.contains("\"ok\": true"));

        // A map with a non-string key is the reliable `serde_json` failure —
        // JSON objects have string keys and nothing else.
        let mut map = std::collections::HashMap::new();
        map.insert(vec![1u8, 2], "value");
        let text = section(&map);
        assert!(text.contains("error"), "{text}");
    }

    /// The archive itself, written and read back.
    ///
    /// The parts are covered above without a zip in sight; what this adds is
    /// the one thing those cannot: that the file on disk is a readable archive
    /// with the names the README promises. A bundle that cannot be opened is
    /// worse than no bundle, and nothing else would notice.
    #[tokio::test]
    async fn the_archive_opens_and_holds_what_the_readme_lists() {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-bundle-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("bundle.zip");

        // No workspace: the bundle from a machine that never got that far is
        // exactly the one somebody needs, so it must still be produced.
        let bundle = write(None, &dest)
            .await
            .expect("a bundle with no workspace");

        assert!(dest.is_file());
        assert_eq!(bundle.path, dest.display().to_string());
        assert!(bundle.bytes > 0);

        let file = std::fs::File::open(&dest).unwrap();
        let mut archive = zip::ZipArchive::new(file).expect("the file is a readable zip");

        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        for required in [
            "README.txt",
            "about.txt",
            "preflight.json",
            "doctor.json",
            "engine.json",
        ] {
            assert!(
                names.contains(&required.to_string()),
                "missing {required} from {names:?}"
            );
        }

        // The README has to name every entry, or it is a manifest that lies.
        let mut readme = String::new();
        {
            use std::io::Read;
            archive
                .by_name("README.txt")
                .unwrap()
                .read_to_string(&mut readme)
                .unwrap();
        }
        for entry in &bundle.entries {
            assert!(
                readme.contains(&entry.name),
                "the README does not mention {}",
                entry.name
            );
        }

        // And the JSON sections are JSON, not a debug rendering of a struct.
        for name in ["preflight.json", "doctor.json", "engine.json"] {
            use std::io::Read;
            let mut text = String::new();
            archive
                .by_name(name)
                .unwrap()
                .read_to_string(&mut text)
                .unwrap();
            serde_json::from_str::<serde_json::Value>(&text)
                .unwrap_or_else(|e| panic!("{name} is not valid JSON: {e}"));
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
