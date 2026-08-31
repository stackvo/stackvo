//! `stackvo.lock` — the versions this project was actually built against.
//!
//! ## The gap [`crate::verify`] already named
//!
//! `stackvo.json` declares what a project needs *around* it: `"services":
//! ["redis", "mysql"]`. It names no versions, and that is right — a repository
//! should not have to guess what the person cloning it will have installed.
//!
//! The cost is written into `verify.rs` in as many words: *"a version the
//! declaration does not pin is reported as `Ok` with the version it found
//! written beside it, rather than being called a match it has not checked …
//! saying which of those is right needs a lock file, which is a separate
//! item"*. Two machines can both satisfy `redis` while running 7.0 and 7.2, and
//! until now nothing in this app could tell those apart or say which one the
//! project was known to work on.
//!
//! This is that file. It is the same division every ecosystem settled on and
//! for the same reason: **the manifest is intent and the lock is fact**, one
//! written by a person and one written by a machine, and both belong in the
//! repository because the second is what makes the first reproducible.
//!
//! ## What is in it, and the one field that makes it a lock
//!
//! ```json
//! {
//!   "lockVersion": 1,
//!   "at": "2026-08-30T09:14:02Z",
//!   "services": [
//!     { "service": "redis", "version": "7.2",
//!       "source": "official",
//!       "sha256": "9f2c…" }
//!   ]
//! }
//! ```
//!
//! The version alone would be a version list. `sha256` is the digest of the
//! **package manifest** as the registry stated it at install time — already
//! recorded per instance in `instances::PackageRef`, already the thing
//! `pkg::Tree::load` verifies every file against. With it, "redis 7.2" out of
//! somebody else's catalogue is a different answer from "redis 7.2" out of the
//! official one, which is exactly the substitution a lock file exists to catch.
//!
//! ## Written, never inferred
//!
//! Nothing writes this file on its own. A lock the app refreshed quietly would
//! record whatever the machine happened to drift to, which is the opposite of
//! locking: the file would always agree with the machine and could never
//! disagree with it, and a check that cannot fail is worse than no check.
//!
//! So [`resolve`] runs when somebody asks, and [`compare`] is what everything
//! else does with the answer — it reports drift and never repairs it.
//!
//! ## What it deliberately does not lock
//!
//! **The runtime and the server.** `stackvo.json` already carries `php.version`
//! and `server`; they travel with the repository and locking them would be
//! writing a second copy of a fact that already has one place to live.
//!
//! **The images this application itself runs.** Those are a property of the
//! machine, not of any one project — a workspace has one `cloudflared` and ten
//! projects — and they already have their locking mechanism in the policy
//! file's `imagePins`, keyed by repository. See [`crate::images`].
//!
//! **Anything that is not installed.** [`resolve`] cannot lock a service the
//! workspace does not have, and it says which ones rather than writing an entry
//! it invented. The same asymmetry [`crate::policy`] states for a locked key it
//! does not also set: *"do not change this" without saying to what* is not a
//! lock, it is a note.

use crate::instances::Table;
use std::path::{Path, PathBuf};

/// The only schema version this build writes, and the highest it reads.
pub const SCHEMA_VERSION: u32 = 1;

/// The file's name, beside `stackvo.json` in the project's own directory.
pub const FILE: &str = "stackvo.lock";

/// One service, as it was resolved when the lock was written.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Locked {
    pub service: String,
    /// Concrete. `latest` never reaches an instance file — the registry
    /// resolves it at install time — so it never reaches this one either.
    pub version: String,
    /// `official`, or the name of a source a policy allowed.
    pub source: String,
    /// Of the version manifest, as the registry stated it at install time.
    ///
    /// The field that makes this a lock rather than a version list. Copied
    /// from `instances::PackageRef` rather than recomputed: recomputing it here
    /// would be a second answer to a question that already has one, and the two
    /// would disagree the first time a package was re-published.
    pub sha256: String,
}

/// What a project was built against.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lock {
    pub lock_version: u32,
    /// When it was written. RFC 3339, UTC, fixed width — the format
    /// `crate::snapshot::rfc3339` produces and everything else in this
    /// workspace already writes, so two timestamps in one repository sort
    /// against each other.
    pub at: String,
    /// Sorted by service id. Sorted so that re-locking an unchanged machine
    /// produces a byte-identical file: a lock whose diff is noise is one people
    /// stop reading.
    pub services: Vec<Locked>,
}

/// Why one declared service could not be locked.
///
/// Named rather than counted, because the two answers send somebody to
/// different places: one is an install and the other is a switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Unlockable {
    /// The workspace has no instance of it at all.
    NotInstalled,
    /// It is installed and switched off. Locking the version of something the
    /// project does not run would be recording a fact about a service nobody
    /// is using.
    Off,
}

/// One declared service that did not make it into the lock, and why.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub service: String,
    pub reason: Unlockable,
}

/// Hold a project's declaration against the instance table.
///
/// Pure, and separate from writing for the reason every other module here keeps
/// them separate: the caller that shows somebody what a lock *would* say has to
/// be able to ask without writing anything.
pub fn resolve(declared: &[String], instances: &Table, at: String) -> (Lock, Vec<Skipped>) {
    let mut services = Vec::new();
    let mut skipped = Vec::new();

    for service in declared {
        let mine: Vec<&crate::instances::Instance> = instances
            .instances
            .iter()
            .filter(|i| &i.service == service)
            .collect();

        if mine.is_empty() {
            skipped.push(Skipped {
                service: service.clone(),
                reason: Unlockable::NotInstalled,
            });
            continue;
        }

        // The one that is on. A workspace may hold two versions of a service
        // side by side, and the one the project actually reaches is the enabled
        // one — locking the other would record a version nothing runs against.
        match mine.iter().find(|i| i.enabled) {
            Some(on) => services.push(Locked {
                service: service.clone(),
                version: on.version.clone(),
                source: on.package.source.clone(),
                sha256: on.package.sha256.clone(),
            }),
            None => skipped.push(Skipped {
                service: service.clone(),
                reason: Unlockable::Off,
            }),
        }
    }

    services.sort_by(|a, b| a.service.cmp(&b.service));
    skipped.sort_by(|a, b| a.service.cmp(&b.service));

    (
        Lock {
            lock_version: SCHEMA_VERSION,
            at,
            services,
        },
        skipped,
    )
}

/// How one locked service compares with what is here now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Drift {
    /// Same version, same package.
    Same,
    /// Not installed at all.
    Absent,
    /// Installed, and none of them switched on.
    Off,
    /// A different version is the one running.
    Version,
    /// The right version, out of a different package.
    ///
    /// The finding a version list cannot make, and the reason `sha256` is in
    /// the file. A re-published `7.2` is a different `7.2`, and on the day that
    /// matters it is the only thing that explains why one machine works.
    Repackaged,
}

/// What the lock says, against what the workspace has.
///
/// Reports and never repairs — see the module comment on why nothing writes
/// this file by itself.
pub fn compare(entry: &Locked, instances: &Table) -> Drift {
    let mine: Vec<&crate::instances::Instance> = instances
        .instances
        .iter()
        .filter(|i| i.service == entry.service)
        .collect();

    if mine.is_empty() {
        return Drift::Absent;
    }

    let Some(on) = mine.iter().find(|i| i.enabled) else {
        return Drift::Off;
    };

    if on.version != entry.version {
        return Drift::Version;
    }

    // Version first, digest second, and the order is the message. "You are on
    // 7.0 and the lock says 7.2" is a sentence somebody can act on; "the digest
    // differs" said about a version that also differs would bury it.
    if on.package.sha256 != entry.sha256 {
        return Drift::Repackaged;
    }

    Drift::Same
}

// --------------------------------------------------------------------- I/O

pub fn path(project_dir: &Path) -> PathBuf {
    project_dir.join(FILE)
}

/// Read it, or `None` when the project has never been locked.
///
/// Absent is not an error: nearly every project in existence has no lock, and
/// treating that as a failure would make the ordinary case the loud one. A file
/// that is *there* and will not parse **is** an error, because somebody wrote
/// it and is entitled to be told it is not being used.
pub fn read(project_dir: &Path) -> crate::error::Result<Option<Lock>> {
    let file = path(project_dir);
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(crate::error::Error::io(
                format!("reading {}", file.display()),
                e,
            ))
        }
    };

    let lock: Lock = serde_json::from_str(&text).map_err(|e| {
        crate::error::Error::new(
            crate::error::Code::InvalidManifest,
            format!("{} is not readable: {e}", file.display()),
        )
    })?;

    // Refused rather than read as far as it goes. A newer build may have
    // written fields whose absence changes what a comparison means, and a lock
    // half-understood is a lock that reports a match it did not check.
    if lock.lock_version > SCHEMA_VERSION {
        return Err(crate::error::Error::new(
            crate::error::Code::Unsupported,
            format!(
                "{} was written by a newer version of StackVo (lock version {}, this build reads {SCHEMA_VERSION})",
                file.display(),
                lock.lock_version
            ),
        ));
    }

    Ok(Some(lock))
}

/// Write it, pretty and newline-terminated.
///
/// Pretty because it goes in a repository and somebody will read it in a diff;
/// newline-terminated for the same reason. `serde_json` writes object keys in
/// declaration order and [`resolve`] sorted the array, so the same workspace
/// re-locked twice produces the same bytes.
pub fn write(project_dir: &Path, lock: &Lock) -> crate::error::Result<PathBuf> {
    let file = path(project_dir);
    let mut text = serde_json::to_string_pretty(lock).map_err(|e| {
        crate::error::Error::new(
            crate::error::Code::IoError,
            format!("serialising the lock: {e}"),
        )
    })?;
    text.push('\n');
    std::fs::write(&file, text)
        .map_err(|e| crate::error::Error::io(format!("writing {}", file.display()), e))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::{Instance, PackageRef};

    fn table(rows: &[(&str, &str, bool, &str)]) -> Table {
        let mut table = Table::default();
        for (service, version, enabled, sha) in rows {
            table.instances.push(Instance {
                id: format!("{service}-{}", version.replace('.', "-")),
                service: (*service).to_string(),
                version: (*version).to_string(),
                package: PackageRef {
                    source: "official".into(),
                    sha256: (*sha).to_string(),
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

    /// The convention `instances.rs` uses: a directory named after the test and
    /// this process, wiped before use — no dev-dependency for four lines.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-lock-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("creating the scratch directory");
        dir
    }

    fn at() -> String {
        "2026-08-30T09:14:02Z".to_string()
    }

    /// What a lock is for, in one assertion: the declaration says `redis` and
    /// the file says which one.
    #[test]
    fn a_lock_records_the_version_the_declaration_left_open() {
        let declared = vec!["redis".to_string(), "mysql".to_string()];
        let (lock, skipped) = resolve(
            &declared,
            &table(&[("redis", "7.2", true, "aaa"), ("mysql", "8.0", true, "bbb")]),
            at(),
        );

        assert!(skipped.is_empty());
        assert_eq!(lock.lock_version, SCHEMA_VERSION);
        // Sorted, so re-locking an unchanged machine is a diff nobody has to
        // read past.
        assert_eq!(
            lock.services
                .iter()
                .map(|l| l.service.as_str())
                .collect::<Vec<_>>(),
            ["mysql", "redis"]
        );
        assert_eq!(lock.services[1].version, "7.2");
        assert_eq!(lock.services[1].sha256, "aaa");
    }

    /// It cannot lock what is not there, and it says which rather than
    /// inventing an entry.
    ///
    /// The same asymmetry `policy.rs` states for a locked key it does not also
    /// set: "do not change this" without saying to what is a note, not a lock.
    /// The two reasons are kept apart because the repairs are — one is an
    /// install and the other is a switch.
    #[test]
    fn what_it_could_not_lock_is_named_with_the_reason() {
        let declared = vec![
            "redis".to_string(),
            "mysql".to_string(),
            "kafka".to_string(),
        ];
        let (lock, skipped) = resolve(
            &declared,
            &table(&[
                ("redis", "7.2", true, "aaa"),
                ("mysql", "8.0", false, "bbb"),
            ]),
            at(),
        );

        assert_eq!(lock.services.len(), 1, "only the one that is on");
        assert_eq!(
            skipped
                .iter()
                .map(|s| (s.service.as_str(), s.reason))
                .collect::<Vec<_>>(),
            [
                ("kafka", Unlockable::NotInstalled),
                ("mysql", Unlockable::Off)
            ]
        );
    }

    /// A workspace holding two versions locks the one that runs.
    #[test]
    fn two_versions_side_by_side_lock_the_enabled_one() {
        let (lock, skipped) = resolve(
            &["redis".to_string()],
            &table(&[
                ("redis", "7.0", false, "old"),
                ("redis", "7.2", true, "new"),
            ]),
            at(),
        );

        assert!(skipped.is_empty());
        assert_eq!(lock.services[0].version, "7.2");
        assert_eq!(lock.services[0].sha256, "new");
    }

    /// The finding a version list cannot make.
    ///
    /// Same version, different package — a re-published `7.2` is a different
    /// `7.2`, and on the day that matters it is the only thing that explains
    /// why one machine works. This is what `sha256` is in the file for.
    #[test]
    fn the_right_version_out_of_a_different_package_is_drift() {
        let entry = Locked {
            service: "redis".into(),
            version: "7.2".into(),
            source: "official".into(),
            sha256: "the-one-it-was-built-against".into(),
        };

        assert_eq!(
            compare(
                &entry,
                &table(&[("redis", "7.2", true, "the-one-it-was-built-against")])
            ),
            Drift::Same
        );
        assert_eq!(
            compare(&entry, &table(&[("redis", "7.2", true, "somebody-elses")])),
            Drift::Repackaged
        );

        // Version first, digest second, and the order is the message: a machine
        // on the wrong version is told that, not told about a digest.
        assert_eq!(
            compare(&entry, &table(&[("redis", "7.0", true, "somebody-elses")])),
            Drift::Version
        );
        assert_eq!(
            compare(&entry, &table(&[("redis", "7.2", false, "x")])),
            Drift::Off
        );
        assert_eq!(compare(&entry, &Table::default()), Drift::Absent);
    }

    /// Re-locking an unchanged machine writes the same bytes.
    ///
    /// Not a nicety. A lock whose diff is noise is one people stop reading, and
    /// a file people stop reading is one that stops being reviewed in a pull
    /// request — which is the only place a lock actually does its job.
    #[test]
    fn the_same_workspace_locked_twice_is_byte_identical() {
        let dir = scratch("identical");
        let declared = vec!["redis".to_string(), "mysql".to_string()];
        let instances = table(&[("mysql", "8.0", true, "bbb"), ("redis", "7.2", true, "aaa")]);

        let (first, _) = resolve(&declared, &instances, at());
        write(&dir, &first).unwrap();
        let a = std::fs::read_to_string(path(&dir)).unwrap();

        // Declared in the other order, and the table in the other order: the
        // sort is what makes those two facts stop mattering.
        let (second, _) = resolve(
            &["mysql".to_string(), "redis".to_string()],
            &table(&[("redis", "7.2", true, "aaa"), ("mysql", "8.0", true, "bbb")]),
            at(),
        );
        write(&dir, &second).unwrap();
        let b = std::fs::read_to_string(path(&dir)).unwrap();

        assert_eq!(a, b);
        assert!(a.ends_with('\n'), "it goes in a repository");
        assert_eq!(read(&dir).unwrap().unwrap(), first);
    }

    /// Absent is not an error; unreadable is; from the future is.
    ///
    /// The three answers are different sentences. A project with no lock has
    /// not failed to write one, and treating that as a failure would make the
    /// ordinary case the loud one. A lock a newer build wrote is refused rather
    /// than read as far as it goes — a lock half-understood is a lock that
    /// reports a match it did not check.
    #[test]
    fn a_lock_from_the_future_is_refused_rather_than_half_read() {
        let dir = scratch("future");
        assert!(read(&dir).unwrap().is_none());

        std::fs::write(path(&dir), "{ not json").unwrap();
        assert!(read(&dir).is_err());

        std::fs::write(
            path(&dir),
            r#"{"lockVersion":99,"at":"2026-08-30T09:14:02Z","services":[]}"#,
        )
        .unwrap();
        let error = read(&dir).unwrap_err().to_string();
        assert!(error.contains("newer version"), "{error}");
    }
}
