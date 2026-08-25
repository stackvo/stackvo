//! Whether a container can carry an editor at all — §3 R-3.
//!
//! [`crate::ide`] wires an IDE on the *host* to a debugger in the container.
//! This is the other half of that idea and a much larger one: the editor itself
//! running **inside** the image — language server, extensions, terminal,
//! `composer` and `artisan` all in there, and no PHP on the machine at all.
//!
//! R-1 is the address that opens it. This is the question that has to be
//! answered before the address is worth having, and it is not one question:
//!
//! * **libc.** VS Code ships a server for musl and JetBrains does not, so
//!   `node:X-alpine` is fine for one and refused by the other. Reported rather
//!   than judged here, because which of the two is being opened is R-1's and
//!   R-2's business.
//! * **The source.** This is the one that is silently wrong. A PHP project
//!   bind-mounts `/var/www/html`, so an editor in there edits the repository.
//!   A `runtime: node` project does not: its Dockerfile is `COPY . .`, and the
//!   container holds a **snapshot** taken when the image was built. An editor
//!   opened against that snapshot works perfectly, saves without complaint, and
//!   **nothing written in it ever reaches the host** — the whole session is
//!   lost on the next rebuild. That is not a warning, it is a refusal.
//! * **Persistence.** The server unpacks itself into `~/.vscode-server`, which
//!   is a hundred-odd megabytes on the container's writable layer and therefore
//!   gone on every `Rebuild`. A named volume is the whole fix, and this module
//!   writes it as an overlay.
//! * **git.** Optional in the toolchain, so an editor may open onto a working
//!   copy whose history it cannot read. A caveat, not a refusal: editing works.
//!
//! ## Why the mount goes in whether or not anybody attaches
//!
//! Same argument [`crate::debugbridge`] makes for its three: a volume is the
//! part that needs the container recreated, so it goes in once, up front, and
//! attaching afterwards costs nothing. The alternative — add the volume when
//! somebody first presses the button — means the button's first press restarts
//! their application, which is a thing a local environment manager should not
//! do to somebody who asked to open an editor.
//!
//! ## Why an overlay rather than the generator
//!
//! The generator's output is under a byte-for-byte contract with the Bash
//! implementation (`fixtures_differential.rs`), the same reason
//! [`crate::xdebug`], [`crate::phpini`], [`crate::perf`] and
//! [`crate::devserver`] are all overlays. This writes one more `-f`,
//! re-derived on every compose invocation and never stored as state.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Where VS Code's server unpacks itself.
///
/// Under `/root` because nothing this generator writes carries a `USER`
/// directive — the PHP images, `node:X-alpine` and every lang runtime all run
/// as root, so `$HOME` is `/root` in all of them. A `USER` line appearing in a
/// generated Dockerfile would move this, and `editor_claims.rs` fails if one
/// ever does.
pub const SERVER_DIR: &str = "/root/.vscode-server";

/// Where the source lives inside a PHP container.
pub const PHP_WORKDIR: &str = "/var/www/html";

pub fn overlay_path(root: &Path) -> PathBuf {
    root.join("generated").join("docker-compose.editor.yml")
}

/// The named volume that keeps a project's editor server across rebuilds.
pub fn volume_name(service: &str) -> String {
    format!("stackvo-{}-editor-server", service.to_ascii_lowercase())
}

/// The C library the image's userland is built against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Libc {
    Glibc,
    /// Alpine. VS Code publishes a server build for it; JetBrains does not.
    Musl,
}

/// Read from the image reference, which is the only thing that knows.
///
/// One rule rather than a table of runtimes: a table would be a second copy of
/// what the Dockerfiles say, and the copy is the one that goes stale. Every
/// musl base this repository builds on says so in its tag — `node:22-alpine`,
/// `oven/bun:1-alpine` — and every other base it uses is Debian.
pub fn libc_of(image: &str) -> Libc {
    if image.contains("alpine") {
        Libc::Musl
    } else {
        Libc::Glibc
    }
}

/// Why an editor cannot be opened here. Each of these is a refusal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Blocker {
    /// There is no container to attach to.
    NotRunning,
    /// The workdir is not a bind mount, so the container holds a copy of the
    /// source rather than the source. Editing it changes nothing on the host.
    SourceIsASnapshot,
}

/// Worth saying, and not worth refusing over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Caveat {
    /// Alpine. Fine for VS Code, refused by JetBrains — see §2 R-2.
    Musl,
    /// The server directory is not a named volume, so the download repeats
    /// after every rebuild. The overlay provides one; the container predates
    /// it and needs recreating.
    ServerIsNotKept,
}

/// What is true about one project's container, as far as an editor cares.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Readiness {
    pub container: String,
    /// Where the source is inside the container, and therefore what an editor
    /// would be told to open.
    pub workdir: String,
    pub server_dir: String,
    pub libc: Libc,
    pub running: bool,
    /// Observed, not assumed: the workdir is a bind mount from the host.
    pub source_live: bool,
    /// Observed: the server directory is a named volume.
    pub server_kept: bool,
    pub blockers: Vec<Blocker>,
    pub caveats: Vec<Caveat>,
    /// The one field a caller needs. False whenever `blockers` is not empty.
    pub attachable: bool,
}

/// Where a runtime keeps the application inside its container.
pub fn workdir_of(runtime: &str) -> &'static str {
    if runtime == "php" || !is_snapshot_runtime(runtime) {
        PHP_WORKDIR
    } else {
        crate::devserver::CONTAINER_PATH
    }
}

/// Runtimes whose container is built with `COPY . .` and holds a snapshot.
///
/// `node` is here and is the one that can be talked out of it — turning the
/// dev server on lays a bind mount over `/app` ([`crate::devserver`]). The lang
/// runtimes have no such overlay, so for them this is the end of the answer.
fn is_snapshot_runtime(runtime: &str) -> bool {
    runtime == "node" || crate::manifest::LANG_RUNTIMES.contains(&runtime)
}

/// The whole judgement, from facts a caller has already gathered.
///
/// Pure, and separated from the Docker call for the reason every gate in this
/// repository is: the interesting cases are a snapshot container and an
/// alpine image, and neither is worth a running engine to reproduce.
///
/// `mounts` is the container's own mount table — the observed truth. A project
/// whose manifest says one thing and whose running container says another is
/// exactly the case this exists for: turning the dev server on writes an
/// overlay, and the overlay does nothing until the container is recreated.
pub fn readiness(
    container: &str,
    runtime: &str,
    image: &str,
    running: bool,
    mounts: &[crate::engine::Mount],
) -> Readiness {
    let workdir = workdir_of(runtime);
    let libc = libc_of(image);

    let source_live = mounts
        .iter()
        .any(|m| m.destination == workdir && m.kind.as_deref() == Some("bind"));
    let server_kept = mounts
        .iter()
        .any(|m| m.destination == SERVER_DIR && m.kind.as_deref() == Some("volume"));

    let mut blockers = Vec::new();
    if !running {
        blockers.push(Blocker::NotRunning);
    } else if !source_live {
        // Only when it is running, and the `else` is the point: a stopped
        // container has no mount table to read, so "the source is a snapshot"
        // would be reported from an empty list — a refusal invented out of
        // having asked nothing.
        blockers.push(Blocker::SourceIsASnapshot);
    }

    let mut caveats = Vec::new();
    if libc == Libc::Musl {
        caveats.push(Caveat::Musl);
    }
    if running && !server_kept {
        caveats.push(Caveat::ServerIsNotKept);
    }

    Readiness {
        container: container.to_string(),
        workdir: workdir.to_string(),
        server_dir: SERVER_DIR.to_string(),
        libc,
        running,
        source_live,
        server_kept,
        attachable: blockers.is_empty(),
        blockers,
        caveats,
    }
}

// ------------------------------------------------------------- the overlay

/// One project's worth of overlay input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub service: String,
}

/// The overlay: one named volume per project, at the server directory.
pub fn overlay_yaml(entries: &[Entry]) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.service.cmp(&b.service));

    let mut out = String::from(
        "# Generated by StackVo Desktop — do not edit.\n\
         #\n\
         # Re-rendered before every compose command, so edits here are lost.\n\
         #\n\
         # One named volume per project, mounted where an editor server unpacks\n\
         # itself. Without it every Rebuild throws away a hundred megabytes that\n\
         # then download again on the next attach. The mount is in whether or\n\
         # not anybody has opened an editor, because a volume is the part that\n\
         # needs the container recreated — adding it on first use would mean the\n\
         # button restarts the application it was asked to open.\n\
         #\n\
         # NOTE: `stackvo up` from the Bash CLI does not layer this file.\n\
         services:\n",
    );

    for entry in &sorted {
        out.push_str(&format!("  {}:\n", entry.service));
        out.push_str("    volumes:\n");
        out.push_str(&format!(
            "      - \"{}:{SERVER_DIR}\"\n",
            volume_name(&entry.service)
        ));
    }

    // `name:` written out, for the reason `perf.rs` writes it: compose derives
    // a default volume name from the directory it was invoked in, so moving a
    // workspace would hand back an empty volume and a hundred-megabyte download
    // that looks like a network problem.
    out.push_str("volumes:\n");
    for entry in &sorted {
        let name = volume_name(&entry.service);
        out.push_str(&format!("  {name}:\n    name: {name}\n"));
    }

    Some(out)
}

/// Every generated service that could carry an editor.
///
/// The lang runtimes are left out and it is not an oversight: `python`, `go`,
/// `ruby`, `rust`, `bun` and `deno` all build with `COPY . .` and have no
/// equivalent of [`crate::devserver`]'s overlay, so their containers can never
/// hold the source. A volume for an editor that must be refused anyway is a
/// hundred megabytes of nothing.
///
/// `node` keeps its volume even with dev mode off, because dev mode is a switch
/// somebody flips — and the flip already recreates the container. Having the
/// volume there beforehand means that is the only recreate.
fn entries(root: &Path) -> Vec<Entry> {
    let Some(projects) = crate::workspace::projects_root(root) else {
        return Vec::new();
    };
    let Ok(dir) = std::fs::read_dir(&projects) else {
        return Vec::new();
    };

    let compose =
        std::fs::read_to_string(root.join("generated").join("docker-compose.projects.yml"))
            .unwrap_or_default();
    let services = crate::xdebug::generated_services(&compose);

    let mut out = Vec::new();
    for item in dir.flatten() {
        let path = item.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !services.iter().any(|s| s == name) {
            continue;
        }
        let Ok(manifest) = crate::manifest::read_effective(&path, name) else {
            continue;
        };
        if crate::manifest::LANG_RUNTIMES.contains(&manifest.runtime.as_str()) {
            continue;
        }
        out.push(Entry {
            service: name.to_string(),
        });
    }

    out.sort_by(|a, b| a.service.cmp(&b.service));
    out
}

/// Re-render the overlay, and report whether it now exists.
pub fn sync(root: &Path) -> bool {
    let path = overlay_path(root);
    match overlay_yaml(&entries(root)) {
        Some(yaml) => {
            if let Some(parent) = path.parent() {
                if std::fs::create_dir_all(parent).is_err() {
                    return false;
                }
            }
            crate::atomic::write(&path, &yaml).is_ok()
        }
        None => {
            let _ = std::fs::remove_file(&path);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Mount;

    fn mount(destination: &str, kind: &str) -> Mount {
        Mount {
            source: Some("/somewhere".into()),
            destination: destination.into(),
            kind: Some(kind.into()),
        }
    }

    /// A PHP project: the generator bind-mounts the source, so this is the
    /// ordinary case and it is attachable.
    #[test]
    fn a_php_container_with_its_source_mounted_can_carry_an_editor() {
        let r = readiness(
            "stackvo-shop",
            "php",
            "php:8.3-fpm",
            true,
            &[mount(PHP_WORKDIR, "bind"), mount(SERVER_DIR, "volume")],
        );

        assert!(r.attachable);
        assert!(r.blockers.is_empty());
        assert!(r.caveats.is_empty(), "{:?}", r.caveats);
        assert_eq!(r.workdir, PHP_WORKDIR);
        assert_eq!(r.libc, Libc::Glibc);
    }

    /// The one this module exists for.
    ///
    /// A node container with dev mode off holds a copy of the source. An editor
    /// opened against it works — the files are there, saving succeeds, the
    /// language server is happy — and every line written is thrown away by the
    /// next rebuild. Nothing about the session says so, which is why this is a
    /// refusal and not a warning.
    #[test]
    fn a_snapshot_container_is_refused_rather_than_warned_about() {
        let r = readiness(
            "stackvo-shop",
            "node",
            "node:22-alpine",
            true,
            &[mount(SERVER_DIR, "volume")],
        );

        assert!(!r.attachable);
        assert_eq!(r.blockers, vec![Blocker::SourceIsASnapshot]);
        assert_eq!(r.workdir, crate::devserver::CONTAINER_PATH);
    }

    /// The same project with the dev server on: the overlay lays a bind mount
    /// over `/app`, and the observed mount table is what decides.
    #[test]
    fn the_same_node_project_is_attachable_once_the_source_is_live() {
        let r = readiness(
            "stackvo-shop",
            "node",
            "node:22-alpine",
            true,
            &[
                mount(crate::devserver::CONTAINER_PATH, "bind"),
                mount(SERVER_DIR, "volume"),
            ],
        );

        assert!(r.attachable);
        assert!(r.source_live);
        // Alpine is a caveat and never a refusal here: VS Code publishes a musl
        // server. JetBrains does not, and that is R-2's problem to state.
        assert_eq!(r.caveats, vec![Caveat::Musl]);
    }

    /// A stopped container has no mount table, so reading a refusal out of an
    /// empty list would be inventing one.
    #[test]
    fn a_stopped_container_reports_only_that_it_is_stopped() {
        let r = readiness("stackvo-shop", "php", "php:8.3-fpm", false, &[]);

        assert_eq!(r.blockers, vec![Blocker::NotRunning]);
        assert!(
            !r.blockers.contains(&Blocker::SourceIsASnapshot),
            "a container nobody asked was reported as holding a snapshot"
        );
        assert!(r.caveats.is_empty(), "{:?}", r.caveats);
    }

    /// A container built before the overlay existed: attachable, and it will
    /// re-download the server after every rebuild until it is recreated.
    #[test]
    fn a_container_predating_the_volume_is_attachable_and_says_what_it_costs() {
        let r = readiness(
            "stackvo-shop",
            "php",
            "php:8.3-fpm",
            true,
            &[mount(PHP_WORKDIR, "bind")],
        );

        assert!(r.attachable);
        assert!(!r.server_kept);
        assert_eq!(r.caveats, vec![Caveat::ServerIsNotKept]);
    }

    /// A bind mount somewhere else is not the source being live. The logs mount
    /// is on every PHP container and would satisfy a check that only counted
    /// binds.
    #[test]
    fn a_bind_mount_at_the_wrong_path_does_not_count_as_the_source() {
        let r = readiness(
            "stackvo-shop",
            "php",
            "php:8.3-fpm",
            true,
            &[mount("/var/log", "bind")],
        );

        assert!(!r.source_live);
        assert_eq!(r.blockers, vec![Blocker::SourceIsASnapshot]);
    }

    /// And a *volume* at the workdir is not the source either — it is
    /// `perf.rs`'s named volume, which is precisely a directory the host copy
    /// no longer reaches.
    #[test]
    fn a_named_volume_at_the_workdir_is_not_the_source_being_live() {
        let r = readiness(
            "stackvo-shop",
            "php",
            "php:8.3-fpm",
            true,
            &[mount(PHP_WORKDIR, "volume")],
        );

        assert!(!r.source_live);
    }

    #[test]
    fn libc_is_read_from_the_image_rather_than_from_a_table_of_runtimes() {
        assert_eq!(libc_of("node:22-alpine"), Libc::Musl);
        assert_eq!(libc_of("oven/bun:1-alpine"), Libc::Musl);
        assert_eq!(libc_of("php:8.3-fpm"), Libc::Glibc);
        assert_eq!(libc_of("python:3.12-slim"), Libc::Glibc);
        assert_eq!(libc_of("denoland/deno:2.1.4"), Libc::Glibc);
    }

    /// The generator is the one that decides, so the answer is taken from it.
    #[test]
    fn the_node_image_this_reads_is_the_one_the_generator_writes() {
        assert_eq!(
            libc_of(&crate::generator::node_base_image("22")),
            Libc::Musl
        );
    }

    #[test]
    fn the_lang_runtimes_keep_their_source_out_of_the_container() {
        for runtime in crate::manifest::LANG_RUNTIMES {
            assert_eq!(
                workdir_of(runtime),
                crate::devserver::CONTAINER_PATH,
                "{runtime}"
            );
        }
        assert_eq!(workdir_of("php"), PHP_WORKDIR);
    }

    #[test]
    fn the_overlay_names_its_volume_so_moving_the_workspace_cannot_lose_it() {
        let yaml = overlay_yaml(&[Entry {
            service: "shop".into(),
        }])
        .expect("one project renders an overlay");

        assert!(yaml.contains("  shop:\n    volumes:\n"), "{yaml}");
        assert!(
            yaml.contains(&format!("stackvo-shop-editor-server:{SERVER_DIR}")),
            "{yaml}"
        );
        assert!(
            yaml.contains(
                "volumes:\n  stackvo-shop-editor-server:\n    name: stackvo-shop-editor-server\n"
            ),
            "{yaml}"
        );
    }

    /// An empty `volumes:` key is a compose error, and a workspace with no
    /// projects is an ordinary state.
    #[test]
    fn no_projects_renders_no_file_at_all() {
        assert!(overlay_yaml(&[]).is_none());
    }

    #[test]
    fn the_overlay_is_sorted_so_the_file_does_not_churn() {
        let yaml = overlay_yaml(&[
            Entry {
                service: "shop".into(),
            },
            Entry {
                service: "blog".into(),
            },
        ])
        .expect("two projects");

        assert!(
            yaml.find("  blog:").unwrap() < yaml.find("  shop:").unwrap(),
            "{yaml}"
        );
    }
}
