//! Whether a container can carry an editor at all, and the address that
//! opens it when it can.
//!
//! [`crate::ide`] wires an IDE on the *host* to a debugger in the container.
//! This is the other half of that idea and a much larger one: the editor itself
//! running **inside** the image — language server, extensions, terminal,
//! `composer` and `artisan` all in there, and no PHP on the machine at all.
//!
//! The address is at the bottom of this file. This is the question that has to
//! be answered before the address is worth having, and it is not one question:
//!
//! * **libc.** VS Code ships a server for musl and JetBrains does not, so
//!   `node:X-alpine` is fine for one and refused by the other. Reported rather
//!   than judged here, and it is the one fact that lands differently on each
//!   of the two editors this file serves: VS Code publishes a musl server and
//!   JetBrains publishes none (decision 0036).
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
    /// Alpine. Fine for VS Code, and the end of the road for JetBrains, which
    /// publishes no musl backend — see [`Jetbrains::musl`] and decision 0036.
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
    /// The address, for `code --folder-uri`. Derived, never stored.
    pub folder_uri: String,
    /// The same address as a URL the OS can hand to VS Code.
    pub handler_url: String,
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
        folder_uri: folder_uri(container, workdir),
        handler_url: handler_url(container, workdir),
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

// ------------------------------------------------------------- the address

/// The scheme VS Code opens a remote folder under.
const REMOTE_SCHEME: &str = "vscode-remote";

/// The authority prefix for a container that is already running.
///
/// Not `dev-container`, which is the other one and means something else: that
/// authority names a `devcontainer.json` and *builds* from it — a second
/// container beside this project's own. This one attaches to the container
/// that is already there, which is the whole of this half.
const ATTACHED: &str = "attached-container";

/// The launcher this module opens. VS Code, and only VS Code.
///
/// Cursor and the other forks carry the same scheme and would very likely
/// work; none of them has been measured here, and an address that opens the
/// wrong thing silently is worse than a button that says "VS Code".
pub const VSCODE: &str = "code";

/// `attached-container+<hex>` — the authority half of the address.
///
/// VS Code has no "attach to this container" command line. What it has is a
/// remote authority, and an attached container's is the hex of the JSON object
/// VS Code writes for itself: `{"containerName":"/stackvo-shop"}`, with the
/// leading slash Docker's own inspect output carries on a name.
///
/// Derived rather than stored, and that is the point: the name comes from
/// [`crate::engine::container_name`] and the path from [`workdir_of`], both
/// already facts of this tree. Nothing is written down, so a renamed project
/// or a recreated container cannot leave a stale address behind.
pub fn attach_authority(container: &str) -> String {
    // Docker reports names with a leading slash and this app passes them
    // without one. Both are the same container, and both must produce the same
    // authority — a second address for one container is a second window.
    let name = container.trim_start_matches('/');
    let json = serde_json::json!({ "containerName": format!("/{name}") }).to_string();

    let mut hex = String::with_capacity(json.len() * 2);
    for byte in json.as_bytes() {
        hex.push_str(&format!("{byte:02x}"));
    }
    format!("{ATTACHED}+{hex}")
}

/// What `code --folder-uri` takes.
pub fn folder_uri(container: &str, workdir: &str) -> String {
    format!(
        "{REMOTE_SCHEME}://{}{}",
        attach_authority(container),
        absolute(workdir)
    )
}

/// The same address as something the OS can open.
///
/// The two are not interchangeable and both are needed. The `--folder-uri`
/// form is an argument to a launcher that may not be installed — the `code`
/// command on `PATH` is a thing somebody has to have run "Install 'code'
/// command" for. VS Code's URL handler is registered by the application
/// itself, so on a machine with the app and without the launcher this is the
/// only one of the two that opens anything.
pub fn handler_url(container: &str, workdir: &str) -> String {
    format!(
        "vscode://{REMOTE_SCHEME}/{}{}",
        attach_authority(container),
        absolute(workdir)
    )
}

/// A path the address can carry: absolute, and with no trailing slash.
fn absolute(workdir: &str) -> String {
    let trimmed = workdir.trim_end_matches('/');
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

/// Whether this machine can open the address at all.
///
/// The same resolver `open_in_editor` uses, so a VS Code installed only as a
/// macOS bundle counts — that is the machine where the launcher is missing and
/// the application is right there.
pub fn installed() -> bool {
    crate::apps::resolve_editor(VSCODE).is_some()
}

// -------------------------------------------------------------- the button

/// Open the editor on the container, or say why not.
///
/// Every refusal here is one the caller could already see in [`Readiness`] —
/// this is the boundary saying it again rather than trusting a screen. A
/// button that is disabled in the UI is not a check.
pub fn open(readiness: &Readiness) -> crate::error::Result<String> {
    use crate::error::{Code, Error};

    match readiness.blockers.first() {
        Some(Blocker::NotRunning) => {
            return Err(Error::new(
                Code::NotFound,
                format!("{} is not running.", readiness.container),
            )
            .with_hint(crate::hints::START_PROJECT_FOR_EDITOR));
        }
        Some(Blocker::SourceIsASnapshot) => {
            return Err(Error::new(
                Code::Unsupported,
                format!(
                    "{} holds a copy of the source at {} rather than the source itself.",
                    readiness.container, readiness.workdir
                ),
            )
            .with_hint(crate::hints::EDITOR_NEEDS_LIVE_SOURCE));
        }
        None => {}
    }

    let Some(launch) = crate::apps::resolve_editor(VSCODE) else {
        return Err(
            Error::new(Code::NotFound, "VS Code was not found on this machine.")
                .with_hint(crate::hints::INSTALL_VS_CODE),
        );
    };

    let spawned = match launch {
        crate::apps::Launch::Command(cmd) => std::process::Command::new(cmd)
            .arg("--folder-uri")
            .arg(&readiness.folder_uri)
            .spawn(),
        // `open -a <bundle> <url>` rather than `--args --folder-uri`: `--args`
        // reaches an application that is being *started*, and VS Code is the
        // kind of application that is already open. The URL handler is read
        // either way.
        crate::apps::Launch::Bundle(bundle) => std::process::Command::new("open")
            .args(["-a", bundle])
            .arg(&readiness.handler_url)
            .spawn(),
    };

    spawned.map_err(|e| Error::io("opening VS Code", e))?;
    Ok(readiness.folder_uri.clone())
}

// ------------------------------------------------------- the other editor

/// The IDE this half is for.
pub const PHPSTORM: &str = "phpstorm";

/// Where StackVo writes the file it hands PhpStorm.
///
/// Under `generated/`, and deliberately **not** in the project.
/// [`crate::devcontainer`] writes into `.devcontainer/` because that file is
/// meant to be committed and describes a machine with no StackVo on it. This
/// one is the opposite in both halves: it names absolute paths under this
/// user's home — the generated compose files — so a teammate who cloned it
/// would get a file that resolves to nothing, and what it describes is the
/// container this machine is already running.
pub fn jetbrains_path(root: &Path, project: &str) -> PathBuf {
    root.join("generated")
        .join("devcontainer")
        .join(project)
        .join("devcontainer.json")
}

/// The file that points PhpStorm at the container that is already there.
///
/// JetBrains has no "attach to a running container" connection type — Gateway
/// offers SSH, WSL, Dev Containers and the cloud plugins, and that is not one
/// of them. What it does have, measured in the IDE rather than read off a page,
/// is Dev Containers with the **compose flavour**: PhpStorm 2026.2 bundles
/// `clouds-docker-gateway`, and the devcontainer schema inside it carries
/// `dockerComposeFile`, `service`, `runServices`, `workspaceFolder`,
/// `shutdownAction` and `overrideCommand`.
///
/// That is what moves the answer. A dev container built from an image or a
/// Dockerfile is a *second* container beside this project's — a second copy of
/// the source, a second database connection, a second port. A dev container
/// that names **StackVo's own compose files and this project's service** is
/// the container that is already running.
///
/// Three of the fields carry the whole decision, and each refuses a default
/// that would have been wrong here:
///
/// * `shutdownAction: "none"` — the compose default is `stopCompose`, so
///   closing the IDE would take the workspace down with it.
/// * `overrideCommand: false` — the default replaces the service's command,
///   which in a PHP project is the thing serving the site. An editor that
///   stops the site to open it is not an editor for a local environment.
/// * `runServices: [service]` — unspecified means every service in every file
///   listed, and StackVo has already started the ones this project needs.
///
/// `customizations.jetbrains.backend` is the plugin's own way of saying which
/// IDE the backend should be, quoted from its change notes: `"jetbrains":
/// {"backend": "IntelliJ"}`. PhpStorm, here, because this is a PHP tool.
pub fn jetbrains_json(project: &str, workdir: &str, compose_files: &[String]) -> String {
    let value = serde_json::json!({
        "name": format!("StackVo: {project}"),
        "dockerComposeFile": compose_files,
        "service": project,
        "runServices": [project],
        "workspaceFolder": workdir,
        "shutdownAction": "none",
        "overrideCommand": false,
        "customizations": { "jetbrains": { "backend": "PhpStorm" } },
    });

    // Trailing newline: the file is read by a human as often as by the IDE.
    format!(
        "{}\n",
        serde_json::to_string_pretty(&value).unwrap_or_default()
    )
}

/// The compose files a devcontainer must name, taken from the one place that
/// knows which overlays are in play.
///
/// [`crate::runner::compose_file_list`] is what every compose command this app
/// runs is built from, so reading the list out of it is the difference between
/// "the same files" and "the same files as of the day this was written".
///
/// `refresh` is the difference between the two callers here, and it is not an
/// optimisation. Writing the file is an act somebody asked for, so it re-renders
/// the overlays first and names what compose would actually layer. *Reading*
/// must not: re-rendering reaches the OS keystore, and a query that reaches the
/// keystore is one the loopback surface may not serve — a rule
/// `websurface_claims.rs` enforces, and which caught this the first time this
/// function was written the other way.
fn compose_files_for(root: &Path, refresh: bool) -> Vec<String> {
    crate::runner::compose_file_list(root, refresh)
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

/// What is true about the PhpStorm half for one project.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Jetbrains {
    /// Is PhpStorm on this machine at all?
    pub installed: bool,
    pub path: String,
    pub exists: bool,
    /// The file on disk is what would be written now. A stale one is worse
    /// than none: it names a compose file list from before an overlay changed.
    pub current: bool,
    /// The compose service the dev container is pointed at.
    pub service: String,
    /// Always true, and it is the cost this half has to state out loud: the
    /// plugin's own advanced setting says "the Main service will always be
    /// recreated", so attaching restarts this project's container.
    pub recreates: bool,
    /// The image is Alpine, and this is where that stops being a note and
    /// becomes a refusal. VS Code publishes a server built against musl;
    /// JetBrains publishes none, so the backend cannot start in there at all.
    /// The file is still written — it is correct, and the image is a thing the
    /// project can change — but the screen must not offer a door that opens
    /// onto nothing.
    pub musl: bool,
}

pub fn jetbrains_status(root: &Path, project: &str, workdir: &str, libc: Libc) -> Jetbrains {
    let path = jetbrains_path(root, project);
    let wanted = jetbrains_json(project, workdir, &compose_files_for(root, false));
    let found = std::fs::read_to_string(&path).ok();

    Jetbrains {
        installed: crate::apps::resolve_editor(PHPSTORM).is_some(),
        path: path.display().to_string(),
        exists: found.is_some(),
        current: found.as_deref() == Some(wanted.as_str()),
        service: project.to_string(),
        recreates: true,
        musl: libc == Libc::Musl,
    }
}

/// Write it, and answer with the path the user has to point PhpStorm at.
pub fn write_jetbrains(root: &Path, project: &str, workdir: &str) -> crate::error::Result<String> {
    let path = jetbrains_path(root, project);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| crate::error::Error::io("creating the devcontainer directory", e))?;
    }
    let json = jetbrains_json(project, workdir, &compose_files_for(root, true));
    crate::atomic::write(&path, &json)?;
    Ok(path.display().to_string())
}

// -------------------------------------------------------------- the reading

/// What is true for one project, container included.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub project: String,
    /// Is there a VS Code on this machine to open the address with?
    pub editor_installed: bool,
    pub readiness: Readiness,
    /// The other editor, which needs a file rather than an address.
    pub jetbrains: Jetbrains,
}

/// Read the container and judge it.
///
/// The image is taken from the running container when there is one and from
/// the manifest when there is not — a stopped node project reported as glibc
/// would be an answer about the wrong image, and the caveat it carries is the
/// one thing about a stopped container that can be said honestly.
pub async fn status(root: &Path, name: &str) -> crate::error::Result<Status> {
    let dir = crate::workspace::require_projects_root(root)?.join(name);
    let manifest_file = dir.join("stackvo.json");
    if !manifest_file.is_file() {
        return Err(crate::error::Error::not_found(format!("project {name}")));
    }
    let manifest = crate::manifest::read(&manifest_file, name)?;

    let details = crate::engine::inspect(name).await.ok();
    let running = details.as_ref().is_some_and(|d| d.running);
    let image = details
        .as_ref()
        .and_then(|d| d.image.clone())
        .unwrap_or_else(|| declared_image(&manifest));
    let mounts = details.map(|d| d.mounts).unwrap_or_default();

    let readiness = readiness(
        &crate::engine::container_name(name),
        &manifest.runtime,
        &image,
        running,
        &mounts,
    );

    Ok(Status {
        project: name.to_string(),
        editor_installed: installed(),
        jetbrains: jetbrains_status(root, name, &readiness.workdir, readiness.libc),
        readiness,
    })
}

/// The image a project *would* run, for when nothing is running.
///
/// Only node has one worth deriving: its base is the one musl image this
/// generator writes, and it is read from the generator rather than repeated
/// here. Everything else falls through to glibc, which is what every other
/// base this repository builds on is.
fn declared_image(manifest: &crate::manifest::Manifest) -> String {
    match manifest.runtime.as_str() {
        "node" => manifest
            .node
            .as_ref()
            .map(|n| crate::generator::node_base_image(&n.version))
            .unwrap_or_default(),
        _ => String::new(),
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
        // server. JetBrains does not, and the PhpStorm half states that where
        // it matters — see `Jetbrains::musl`.
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

    // ---------------------------------------------------------- the address

    /// The one fact this whole feature is: the hex is VS Code's own, computed
    /// here and written out by hand there.
    ///
    /// Held against a literal rather than against `attach_authority`'s own
    /// arithmetic, because a test that hexes the string a second time would
    /// pass on any encoding both halves agreed on — including one VS Code
    /// cannot read. This is the byte sequence VS Code writes into its own
    /// recently-opened list for a container called `stackvo-shop`.
    #[test]
    fn the_authority_is_the_hex_of_the_json_vs_code_writes_for_itself() {
        assert_eq!(
            attach_authority("stackvo-shop"),
            "attached-container+7b22636f6e7461696e65724e616d65223a222f737461636b766f2d73686f70227d"
        );

        // And it decodes back to exactly that object — the leading slash
        // included, which is the half a hand-written string loses.
        let hex = attach_authority("stackvo-shop")
            .split_once('+')
            .map(|(_, hex)| hex.to_string())
            .expect("the authority carries its payload after the +");
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
            .collect();
        assert_eq!(
            String::from_utf8(bytes).expect("utf-8"),
            r#"{"containerName":"/stackvo-shop"}"#
        );
    }

    /// Docker reports names with a leading slash and this app passes them
    /// without one. Two spellings of one container must not be two addresses —
    /// VS Code would open a second window with its own server in it.
    #[test]
    fn a_name_with_and_without_dockers_leading_slash_is_one_address() {
        assert_eq!(
            attach_authority("stackvo-shop"),
            attach_authority("/stackvo-shop")
        );
    }

    /// The name is the one `engine.rs` builds, not a second spelling of it.
    #[test]
    fn the_address_names_the_container_the_engine_would_name() {
        let container = crate::engine::container_name("shop");
        let uri = folder_uri(&container, PHP_WORKDIR);

        assert_eq!(uri, folder_uri("stackvo-shop", PHP_WORKDIR), "{uri}");
        assert!(
            uri.starts_with("vscode-remote://attached-container+"),
            "{uri}"
        );
        assert!(uri.ends_with(PHP_WORKDIR), "{uri}");
    }

    /// Both forms exist because both are needed, and they must agree.
    ///
    /// `--folder-uri` is an argument to a launcher that may not be installed;
    /// the URL handler is registered by the application itself. A machine can
    /// easily have the second and not the first.
    #[test]
    fn the_launcher_argument_and_the_url_handler_carry_the_same_address() {
        let authority = attach_authority("stackvo-shop");

        assert_eq!(
            folder_uri("stackvo-shop", PHP_WORKDIR),
            format!("vscode-remote://{authority}{PHP_WORKDIR}")
        );
        assert_eq!(
            handler_url("stackvo-shop", PHP_WORKDIR),
            format!("vscode://vscode-remote/{authority}{PHP_WORKDIR}")
        );
    }

    /// The path is the container's, so it is absolute and has no trailing
    /// slash — VS Code takes the last segment as the folder's name, and an
    /// empty one is a window titled after nothing.
    #[test]
    fn the_path_is_absolute_and_carries_no_trailing_slash() {
        assert!(folder_uri("stackvo-shop", "var/www/html").ends_with("/var/www/html"));
        assert!(folder_uri("stackvo-shop", "/var/www/html/").ends_with("/var/www/html"));
    }

    /// The address on a readiness is the address for the workdir that same
    /// readiness reported — a node project in dev mode opens `/app`, not the
    /// PHP path.
    #[test]
    fn the_readiness_carries_the_address_for_its_own_workdir() {
        let php = readiness(
            "stackvo-shop",
            "php",
            "php:8.3-fpm",
            true,
            &[mount(PHP_WORKDIR, "bind"), mount(SERVER_DIR, "volume")],
        );
        assert_eq!(php.folder_uri, folder_uri("stackvo-shop", PHP_WORKDIR));

        let node = readiness(
            "stackvo-blog",
            "node",
            "node:22-alpine",
            true,
            &[mount(crate::devserver::CONTAINER_PATH, "bind")],
        );
        assert_eq!(
            node.folder_uri,
            folder_uri("stackvo-blog", crate::devserver::CONTAINER_PATH)
        );
        assert_ne!(node.folder_uri, php.folder_uri);
    }

    /// A refused container still has an address, and that is deliberate: the
    /// string is what somebody pastes into a machine that has VS Code when
    /// this one does not. What it must never be is empty, which would read on
    /// screen as "there is no address for this project".
    #[test]
    fn a_refused_container_still_reports_the_address_it_would_have_used() {
        let r = readiness("stackvo-shop", "php", "php:8.3-fpm", false, &[]);

        assert!(!r.attachable);
        assert!(!r.folder_uri.is_empty());
        assert!(r.folder_uri.contains(&attach_authority("stackvo-shop")));
    }

    /// The refusals reach the caller as refusals, with the reason each carries.
    #[test]
    fn opening_a_container_that_is_not_ready_is_refused_with_the_reason() {
        let stopped = readiness("stackvo-shop", "php", "php:8.3-fpm", false, &[]);
        let err = open(&stopped).expect_err("a stopped container cannot be opened");
        assert_eq!(err.code, crate::error::Code::NotFound);
        assert_eq!(
            err.hint_key,
            Some(crate::hints::START_PROJECT_FOR_EDITOR.key)
        );

        let snapshot = readiness(
            "stackvo-blog",
            "node",
            "node:22-alpine",
            true,
            &[mount(SERVER_DIR, "volume")],
        );
        let err = open(&snapshot).expect_err("a snapshot container cannot be opened");
        assert_eq!(err.code, crate::error::Code::Unsupported);
        assert_eq!(
            err.hint_key,
            Some(crate::hints::EDITOR_NEEDS_LIVE_SOURCE.key)
        );
        // The message names the path, because "the source is a copy" without
        // one leaves the reader looking for which directory is meant.
        assert!(
            err.message.contains(crate::devserver::CONTAINER_PATH),
            "{}",
            err.message
        );
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

    // ----------------------------------------------------- the other editor

    fn phpstorm_file(project: &str, workdir: &str) -> serde_json::Value {
        serde_json::from_str(&jetbrains_json(
            project,
            workdir,
            &[
                "/root/generated/stackvo.yml".to_string(),
                "/root/generated/docker-compose.projects.yml".to_string(),
            ],
        ))
        .expect("what is written is JSON")
    }

    /// The three defaults this file exists to refuse.
    ///
    /// Each of them is silent and each of them is destructive in a different
    /// way, which is why they are asserted together: a file that got two of
    /// three right would look correct in every screenshot.
    #[test]
    fn the_devcontainer_refuses_the_three_defaults_that_would_be_wrong_here() {
        let file = phpstorm_file("shop", PHP_WORKDIR);

        // Closing the IDE would otherwise take the workspace down: the compose
        // flavour's default is `stopCompose`.
        assert_eq!(file["shutdownAction"], "none");

        // The default replaces the service's command — in a PHP project that
        // command is the thing serving the site, so an editor that attached
        // would stop the site it was opened to work on.
        assert_eq!(file["overrideCommand"], false);

        // Unspecified means every service in every file listed, and StackVo has
        // already started the ones this project needs.
        //
        // It carries a second job nothing else states: every generated project
        // service sits behind a compose **profile**, and a profile is only
        // active when something names it — or names a service explicitly.
        // `runServices` is what makes the devcontainer's own `up` name this
        // service, and an empty list here would leave the IDE looking for a
        // service compose does not consider to exist. Measured against the real
        // files: `compose … up -d --no-recreate parser.ajans` answers
        // `Container stackvo-parser.ajans Running`.
        assert_eq!(file["runServices"], serde_json::json!(["shop"]));
    }

    /// The container it opens is this project's own, not a second one.
    #[test]
    fn the_devcontainer_names_stackvos_own_service_and_compose_files() {
        let file = phpstorm_file("shop", PHP_WORKDIR);

        assert_eq!(file["service"], "shop");
        assert_eq!(file["workspaceFolder"], PHP_WORKDIR);
        assert_eq!(
            file["dockerComposeFile"],
            serde_json::json!([
                "/root/generated/stackvo.yml",
                "/root/generated/docker-compose.projects.yml"
            ])
        );
        // No `image`, no `build`: either one is the flavour that builds a
        // second container beside the one already running.
        assert!(file.get("image").is_none(), "{file}");
        assert!(file.get("build").is_none(), "{file}");
    }

    /// Which backend the plugin launches, in the plugin's own spelling.
    #[test]
    fn the_devcontainer_asks_for_the_php_ide_rather_than_the_default_one() {
        let file = phpstorm_file("shop", PHP_WORKDIR);
        assert_eq!(file["customizations"]["jetbrains"]["backend"], "PhpStorm");
    }

    /// The workdir is the readiness's, so a node project in dev mode is opened
    /// at `/app` — the same answer the address gives VS Code.
    #[test]
    fn the_devcontainer_opens_the_workdir_the_readiness_reports() {
        let node = readiness(
            "stackvo-blog",
            "node",
            "node:22-alpine",
            true,
            &[mount(crate::devserver::CONTAINER_PATH, "bind")],
        );
        let file = phpstorm_file("blog", &node.workdir);

        assert_eq!(file["workspaceFolder"], crate::devserver::CONTAINER_PATH);
    }

    /// Alpine is a note for VS Code and the end of the road for JetBrains.
    ///
    /// Two editors, one fact, two different weights — which is exactly why the
    /// libc is reported rather than judged where it is read.
    #[test]
    fn the_php_ide_half_says_when_the_image_has_no_backend_for_it() {
        let php = jetbrains_status(Path::new("/root"), "shop", PHP_WORKDIR, Libc::Glibc);
        assert!(!php.musl);

        let node = jetbrains_status(Path::new("/root"), "blog", "/app", Libc::Musl);
        assert!(node.musl);
        // The file is still described: the image is a thing a project can
        // change, and a path that is correct should not be hidden because
        // today's image cannot walk it.
        assert!(node.path.ends_with("devcontainer.json"));
    }

    /// It is written where StackVo owns the files, and never into the project.
    ///
    /// The list of compose files in it is absolute and under this user's home,
    /// so a committed copy would resolve to nothing on anybody else's machine.
    /// `devcontainer.rs` writes the file that *is* meant to be committed, and
    /// the two must not end up in the same place.
    #[test]
    fn the_file_is_written_under_generated_and_not_into_the_project() {
        let path = jetbrains_path(Path::new("/root"), "shop");

        assert_eq!(
            path,
            Path::new("/root/generated/devcontainer/shop/devcontainer.json")
        );
        assert!(
            !path.to_string_lossy().contains("/projects/"),
            "{}",
            path.display()
        );
        assert!(
            !path.to_string_lossy().contains(crate::devcontainer::DIR),
            "this file has landed in the directory devcontainer.rs writes the \
             committed one into: {}",
            path.display()
        );
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
