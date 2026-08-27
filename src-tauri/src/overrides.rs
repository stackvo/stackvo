//! A workspace taking over one file of a package it did not write.
//!
//! The last of the three extension points the package system was designed to
//! have. The other two are here already:
//! authoring a package is [`crate::authoring`], and the organisation's half of a
//! third-party source is `policy.market.allowedSources`. This is the one in
//! between — a package that is *nearly* right, and a person who needs one line
//! of it different.
//!
//! ## Why the old answer stopped working
//!
//! Before packages, that person edited `core/templates/services/redis/…` and
//! [`crate::skeleton`] made the edit win over the bytes in the binary. That
//! directory is gone, and the replacement is a verified tree: every file a
//! package ships is hashed by its manifest and [`crate::pkg::verify`] checks it
//! on every read. So the same edit now produces a package that refuses to load,
//! complaining about bytes rather than about the line just typed — the exact
//! obstacle [`crate::authoring`] was written for, arriving from the other side.
//!
//! Re-sealing is the wrong tool here. It is right for a package you are
//! *writing*; applied to one you fetched it rewrites somebody else's manifest
//! so it describes your edit, and the next `market_install` silently undoes it
//! with no record that it ever existed. An override has to survive a reinstall,
//! and it has to be visible.
//!
//! ## So the edit lives outside the package, and the package stays intact
//!
//! `<root>/overrides/<service>/<version>/<the file's own relative path>`.
//!
//! Nothing under `market/packages/` is touched, so the hash chain is exactly as
//! it was and a reinstall is exactly as safe. [`crate::pkg::Tree`] consults this
//! directory first when a renderer asks it for a file, which is the whole of the
//! mechanism — and it is [`crate::skeleton`]'s mechanism, with the one change
//! that matters: skeleton's overrides live *at* the path they replace, and these
//! live beside it, because that path is now covered by a hash.
//!
//! ## Templates, never the manifest
//!
//! [`overridable`] is derived from the manifest and holds three things: the
//! compose fragment, the config templates, and each companion's fragment. The
//! manifest itself is **not** overridable, and that is the load-bearing rule.
//!
//! The manifest is where the image, the ports, the volumes and the settings are
//! declared; the render context is built from it and so is every check made
//! against it. A workspace that could override the manifest could change the
//! image a package runs while the catalogue still reported the published one,
//! and every statement this app makes about what is installed would become a
//! statement about what was installed. A template cannot do that: whatever it
//! says, it is substituted from a context the manifest defines and then passed
//! through [`crate::compose_policy`] — the same allowlist a downloaded fragment
//! goes through, on the same code path, after substitution. An override is
//! ordinary content arriving from a more trusted place, not a way past a gate.
//!
//! ## Reverting deletes, and does not restore
//!
//! [`revert`] removes the workspace's copy and the package's own file takes over
//! on the next read. Writing the pristine bytes back into the override instead
//! would leave a file on disk that means nothing — which is the state
//! [`crate::skeleton`] documents at length as the one it exists to stop
//! producing, because it makes "overridden" mean "was installed".

use crate::error::{Code, Error, Result};
use crate::pkg::{Catalogue, Manifest};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// `<root>/overrides`.
pub fn dir(root: &Path) -> PathBuf {
    root.join("overrides")
}

/// What a file in a package is for, so a list is readable without opening each.
pub const COMPOSE: &str = "compose";
pub const CONFIG: &str = "config";
pub const COMPANION: &str = "companion";

/// One file of one package version, and whether this workspace has taken it
/// over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverridableFile {
    /// The relative path the manifest names it by, and the id every other call
    /// takes.
    pub path: String,
    /// [`COMPOSE`], [`CONFIG`] or [`COMPANION`].
    pub kind: String,
    /// For a companion, whose. `None` for the service's own files.
    pub companion: Option<String>,
    /// There is a copy under `overrides/`, so that copy is what renders.
    pub overridden: bool,
    /// Absolute, and always the path an override *would* live at — so a caller
    /// can open the file in the user's own editor without asking twice.
    pub at: String,
}

/// One file this workspace has taken over, found by walking `overrides/`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    pub service: String,
    pub version: String,
    pub path: String,
}

/// The files of this package a workspace may take over, in a stable order.
///
/// Derived from the manifest rather than from the directory, and the two are
/// not the same list: a package may ship a file no manifest field names, and a
/// file nothing reads is not a file overriding changes anything about. The
/// manifest is also the thing whose hashes make an edit-in-place refuse to
/// load, so it is the honest definition of "which files does this apply to".
pub fn overridable(manifest: &Manifest) -> Vec<(String, &'static str, Option<String>)> {
    let mut out: Vec<(String, &'static str, Option<String>)> =
        vec![(manifest.compose.file.clone(), COMPOSE, None)];
    for file in &manifest.files {
        out.push((file.template.clone(), CONFIG, None));
    }
    for companion in &manifest.companions {
        out.push((
            companion.compose.file.clone(),
            COMPANION,
            Some(companion.name.clone()),
        ));
    }

    // A manifest that names one file twice — a config template also used as a
    // companion's fragment — would otherwise produce two rows that are one
    // file, and reverting the second would report success on something already
    // gone. First mention wins, so the kind shown is the more specific one.
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|(path, _, _)| seen.insert(path.clone()));
    out
}

/// The overridable files of one installed version, with their current state.
pub fn listing(
    root: &Path,
    service: &str,
    version: &str,
    manifest: &Manifest,
) -> Result<Vec<OverridableFile>> {
    let base = version_dir(root, service, version)?;
    overridable(manifest)
        .into_iter()
        .map(|(path, kind, companion)| {
            crate::pkg::checked_relative(&path, "package file")?;
            let at = base.join(&path);
            Ok(OverridableFile {
                overridden: at.is_file(),
                at: at.display().to_string(),
                path,
                kind: kind.to_string(),
                companion,
            })
        })
        .collect()
}

/// Copy the package's own file into `overrides/` so it can be edited.
///
/// Returns where it landed, because the caller's next move is to open it in the
/// user's editor — the same return `template_override` gives, for the same
/// reason.
///
/// Refuses when a copy is already there. That file is somebody's edit, and this
/// is the call that would replace it with the published bytes.
pub fn materialize(
    root: &Path,
    catalogue: &dyn Catalogue,
    service: &str,
    version: &str,
    manifest: &Manifest,
    relative: &str,
) -> Result<PathBuf> {
    // The organisation's half of this gate. A note rather than a lock (ADR
    // 0009) — anybody who can write the policy can widen it — but a machine
    // running a catalogue somebody vetted has a real reason to say that the
    // vetted bytes are the ones that run.
    let market = crate::policy::current().market();
    if !market.allows_overrides() {
        return Err(Error::new(
            Code::Forbidden,
            format!(
                "this machine does not allow package files to be overridden ({})",
                crate::policy::current().origin()
            ),
        )
        .with_hint(crate::hints::OVERRIDES_REFUSED_BY_POLICY));
    }

    checked(manifest, relative)?;

    let target = version_dir(root, service, version)?.join(relative);
    if target.exists() {
        return Err(Error::new(
            Code::AlreadyExists,
            format!("{service}@{version}: {relative} is already overridden in this workspace"),
        )
        .with_hint(crate::hints::REVERT_OVERRIDE_FIRST));
    }

    // Through the catalogue rather than off the disk, so the bytes copied are
    // bytes `pkg::verify` has just agreed match the manifest. Starting an
    // override from a file that was already wrong would bake the corruption in
    // and remove the only thing that would have reported it.
    let text = catalogue.file(service, version, relative).ok_or_else(|| {
        Error::not_found(format!("{service}@{version}: {relative}"))
            .with_hint(crate::hints::PACKAGE_CONTENT_CHANGED)
    })?;

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    std::fs::write(&target, text)
        .map_err(|e| Error::io(format!("writing {}", target.display()), e))?;

    Ok(target)
}

/// Delete the workspace's copy; the package's own file takes over again.
///
/// Not gated on policy, and that asymmetry is the same one
/// `market.requireSignature` has: this call can only move a workspace *towards*
/// what the publisher shipped, so a policy that forbade it would be forbidding
/// the safe direction.
pub fn revert(root: &Path, service: &str, version: &str, relative: &str) -> Result<()> {
    crate::pkg::checked_relative(relative, "override")?;
    let base = version_dir(root, service, version)?;
    let target = base.join(relative);

    match std::fs::remove_file(&target) {
        Ok(()) => {}
        // Already back on the published file. The user asked for a state, not
        // for an operation, and they are in it.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error::io(format!("removing {}", target.display()), e)),
    }

    // The directories that file was in now hold nothing and say nothing.
    // `remove_dir` refuses a non-empty one, which is exactly the guard wanted:
    // a directory still holding another override stays. Started at the file's
    // own parent rather than at the version directory, because a template two
    // levels down (`files/my.cnf.tpl`) leaves `files/` behind otherwise — an
    // empty directory that reads as an override until somebody looks inside.
    if let Some(parent) = target.parent() {
        prune_empty(parent, &dir(root));
    }
    Ok(())
}

/// The file this workspace has put in front of a package's own, if any.
///
/// Read by [`crate::pkg::Tree`] and by nothing else. `None` for every reason —
/// no override, an unreadable one, a service or version that is not a usable
/// directory name — because the caller's fallback is the published file, which
/// is the right answer to all of them.
pub fn read(overrides_root: &Path, service: &str, version: &str, relative: &str) -> Option<String> {
    crate::pkg::checked_relative(relative, "override").ok()?;
    let path = overrides_root
        .join(label(service).ok()?)
        .join(label(version).ok()?)
        .join(relative);
    std::fs::read_to_string(path).ok()
}

/// Everything this workspace has taken over, sorted.
///
/// A directory walk rather than a question asked of each installed package, and
/// deliberately: an override whose package has since been uninstalled still
/// exists, still costs disk, and would otherwise be invisible — which is the
/// state `doctor` is for.
pub fn all(root: &Path) -> Vec<Override> {
    let base = dir(root);
    let mut out = Vec::new();

    for service_dir in dirs_in(&base) {
        let Some(service) = name_of(&service_dir) else {
            continue;
        };
        for version_dir in dirs_in(&service_dir) {
            let Some(version) = name_of(&version_dir) else {
                continue;
            };
            for path in files_under(&version_dir, &version_dir) {
                out.push(Override {
                    service: service.clone(),
                    version: version.clone(),
                    path,
                });
            }
        }
    }

    out.sort();
    out
}

// ---------------------------------------------------------------- internals

/// A component that is about to become a directory name.
///
/// `service` and `version` reach here from `instances.json` and from the
/// webview, and are joined onto a path. `pkg::checked_relative` guards the
/// file's own path and says nothing about these two, so they are checked here —
/// at the point of the join, which is the check that is still there after
/// somebody refactors the caller.
fn label(value: &str) -> Result<&str> {
    let bad = |why: &str| {
        Err(Error::new(
            Code::InvalidInput,
            format!("{value:?} {why} and cannot name a directory"),
        )
        .with_hint(crate::hints::PACKAGE_PATHS_STAY_INSIDE))
    };
    if value.is_empty() {
        return bad("is empty");
    }
    if value.starts_with('.') {
        return bad("starts with a dot");
    }
    if value.contains(['/', '\\', ':']) {
        return bad("contains a path separator");
    }
    Ok(value)
}

fn version_dir(root: &Path, service: &str, version: &str) -> Result<PathBuf> {
    Ok(dir(root).join(label(service)?).join(label(version)?))
}

/// A path this workspace is willing to write under `overrides/`.
///
/// Membership in [`overridable`] is the check, and it is an exact match against
/// a list the manifest states — which no amount of `..` can talk its way into,
/// and which is why the manifest is not in it.
fn checked(manifest: &Manifest, relative: &str) -> Result<()> {
    if overridable(manifest).iter().any(|(p, _, _)| p == relative) {
        return crate::pkg::checked_relative(relative, "override");
    }
    Err(Error::new(
        Code::InvalidInput,
        format!("{relative:?} is not a file of this package that can be overridden"),
    )
    .with_hint(crate::hints::ONLY_PACKAGE_TEMPLATES))
}

/// Depth-first up to `stop`, so a parent is tried after its children.
fn prune_empty(from: &Path, stop: &Path) {
    let mut cursor = from.to_path_buf();
    while cursor.starts_with(stop) && cursor != stop {
        if std::fs::remove_dir(&cursor).is_err() {
            return;
        }
        let Some(parent) = cursor.parent() else {
            return;
        };
        cursor = parent.to_path_buf();
    }
}

fn dirs_in(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}

fn name_of(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.starts_with('.'))
        .map(str::to_string)
}

/// Every file under `dir`, at any depth, as a path relative to `base`.
fn files_under(dir: &Path, base: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(files_under(&path, base));
            continue;
        }
        if let Ok(relative) = path.strip_prefix(base) {
            // The separator on the wire is the one the manifest writes, on
            // every platform: these strings are compared against manifest
            // fields, and a Windows workspace producing `files\my.cnf.tpl`
            // would match nothing and report an override nobody could revert.
            let text = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if !text.is_empty() {
                out.push(text);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkg::Tree;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("stackvo-overrides-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    const FRAGMENT: &str = "image: \"{{ image }}\"\n";
    const CONF: &str = "maxmemory 256mb\n";

    /// A package on disk whose hashes are right, so the tree will read it.
    fn plant(root: &Path, service: &str, version: &str) {
        let dir = root
            .join("packages/databases")
            .join(service)
            .join("versions")
            .join(version);
        std::fs::create_dir_all(dir.join("files")).unwrap();
        std::fs::write(dir.join("compose.yml"), FRAGMENT).unwrap();
        std::fs::write(dir.join("files/my.cnf.tpl"), CONF).unwrap();

        std::fs::write(
            dir.parent().unwrap().parent().unwrap().join("package.json"),
            format!(
                r#"{{"apiVersion":"stackvo.dev/package/v1","service":"{service}",
                     "category":"databases"}}"#
            ),
        )
        .unwrap();

        std::fs::write(
            dir.join("manifest.json"),
            format!(
                r#"{{
                  "apiVersion": "stackvo.dev/package/v1",
                  "service": "{service}",
                  "version": "{version}",
                  "image": {{ "repository": "{service}", "tag": "{version}" }},
                  "instancing": {{ "multiple": true }},
                  "files": [
                    {{ "name": "my_cnf", "template": "files/my.cnf.tpl",
                       "target": "/etc/my.cnf", "sha256": "{}" }}
                  ],
                  "compose": {{ "file": "compose.yml", "sha256": "{}" }},
                  "support": {{ "status": "supported" }}
                }}"#,
                crate::pkg::sha256_hex(CONF.as_bytes()),
                crate::pkg::sha256_hex(FRAGMENT.as_bytes()),
            ),
        )
        .unwrap();
    }

    fn tree(market: &Path) -> Tree {
        Tree::open(market).unwrap()
    }

    /// The point of the whole module: the workspace's bytes are what renders,
    /// and the package is still intact underneath.
    #[test]
    fn an_override_is_what_a_renderer_reads_and_the_package_still_verifies() {
        let root = scratch("wins");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let plain = tree(&market);
        let manifest = plain.load("mysql", "8.0").unwrap();
        materialize(&root, &plain, "mysql", "8.0", &manifest, "compose.yml").unwrap();
        std::fs::write(
            dir(&root).join("mysql/8.0/compose.yml"),
            "image: \"mine\"\n",
        )
        .unwrap();

        let layered = tree(&market).with_overrides(dir(&root));
        assert_eq!(
            layered.file("mysql", "8.0", "compose.yml").as_deref(),
            Some("image: \"mine\"\n")
        );
        // The package's own bytes are untouched, so the hash chain is intact
        // and a reinstall is exactly as safe as it was.
        assert!(layered.load("mysql", "8.0").is_ok());
    }

    #[test]
    fn reverting_puts_the_published_file_back_and_leaves_no_empty_directories() {
        let root = scratch("revert");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let plain = tree(&market);
        let manifest = plain.load("mysql", "8.0").unwrap();
        materialize(&root, &plain, "mysql", "8.0", &manifest, "files/my.cnf.tpl").unwrap();
        assert_eq!(all(&root).len(), 1);

        revert(&root, "mysql", "8.0", "files/my.cnf.tpl").unwrap();

        let layered = tree(&market).with_overrides(dir(&root));
        assert_eq!(
            layered.file("mysql", "8.0", "files/my.cnf.tpl").as_deref(),
            Some(CONF)
        );
        assert!(all(&root).is_empty());
        // `overrides/mysql/8.0/files` and everything above it up to
        // `overrides/` are gone; `overrides/` itself stays.
        assert!(!dir(&root).join("mysql").exists());
    }

    /// Asking twice for a state somebody is already in is not an error.
    #[test]
    fn reverting_something_that_was_never_overridden_succeeds() {
        let root = scratch("idempotent");
        assert!(revert(&root, "mysql", "8.0", "compose.yml").is_ok());
    }

    /// The load-bearing rule. The manifest declares the image, the ports and
    /// the volumes; a workspace that could rewrite it could run one thing while
    /// the catalogue reported another.
    #[test]
    fn the_manifest_itself_cannot_be_overridden() {
        let root = scratch("manifest");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let plain = tree(&market);
        let manifest = plain.load("mysql", "8.0").unwrap();
        let refused = materialize(&root, &plain, "mysql", "8.0", &manifest, "manifest.json");

        assert!(refused.is_err(), "the manifest was accepted as overridable");
        assert!(!dir(&root).join("mysql/8.0/manifest.json").exists());
    }

    #[test]
    fn a_file_the_manifest_does_not_ship_is_refused() {
        let root = scratch("stranger");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let plain = tree(&market);
        let manifest = plain.load("mysql", "8.0").unwrap();
        for path in ["../../../etc/passwd", "files/../../escape", "README.md"] {
            assert!(
                materialize(&root, &plain, "mysql", "8.0", &manifest, path).is_err(),
                "{path} was accepted"
            );
        }
    }

    /// A service or a version is a directory name here, and neither is checked
    /// by the path guard that covers the file.
    #[test]
    fn a_service_or_version_that_walks_out_is_refused() {
        let root = scratch("walk");
        assert!(revert(&root, "..", "8.0", "compose.yml").is_err());
        assert!(revert(&root, "mysql", "../..", "compose.yml").is_err());
        assert!(read(&dir(&root), "..", "8.0", "compose.yml").is_none());
    }

    #[test]
    fn taking_over_a_file_twice_is_refused_rather_than_overwriting_the_edit() {
        let root = scratch("twice");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let plain = tree(&market);
        let manifest = plain.load("mysql", "8.0").unwrap();
        materialize(&root, &plain, "mysql", "8.0", &manifest, "compose.yml").unwrap();
        std::fs::write(dir(&root).join("mysql/8.0/compose.yml"), "mine\n").unwrap();

        assert!(materialize(&root, &plain, "mysql", "8.0", &manifest, "compose.yml").is_err());
        assert_eq!(
            std::fs::read_to_string(dir(&root).join("mysql/8.0/compose.yml")).unwrap(),
            "mine\n",
            "the edit was replaced by the published bytes"
        );
    }

    #[test]
    fn the_listing_names_every_template_and_only_templates() {
        let root = scratch("listing");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let plain = tree(&market);
        let manifest = plain.load("mysql", "8.0").unwrap();
        let rows = listing(&root, "mysql", "8.0", &manifest).unwrap();

        let paths: Vec<&str> = rows.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, vec!["compose.yml", "files/my.cnf.tpl"]);
        assert!(rows.iter().all(|r| !r.overridden));
        assert_eq!(rows[0].kind, COMPOSE);
        assert_eq!(rows[1].kind, CONFIG);
    }

    /// A tree with no overrides directory attached behaves exactly as before.
    #[test]
    fn a_workspace_with_no_overrides_reads_the_published_files() {
        let root = scratch("none");
        let market = root.join("market");
        plant(&market, "mysql", "8.0");

        let layered = tree(&market).with_overrides(dir(&root));
        assert_eq!(
            layered.file("mysql", "8.0", "compose.yml").as_deref(),
            Some(FRAGMENT)
        );
        assert!(all(&root).is_empty());
    }
}
