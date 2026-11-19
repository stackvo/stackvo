//! Two per-project settings that reach the container through this app's own
//! layers rather than through the manifest.
//!
//! ## Why they share a file
//!
//! They are the same kind of thing: a switch somebody sets on one project, that
//! the generator cannot read from the manifest because
//! `contracts/project.schema.json` is `additionalProperties: false` and the
//! contract is frozen. `.stackvo/` is the established answer — `php.ini`,
//! `xdebug.json`, `devserver.json` and `perf.json` are all there, and all of
//! them travel with the project when a teammate clones it.
//!
//! Two settings, one file, because a second file per switch is how a directory
//! becomes a place people stop looking.
//!
//! ## Where each one lands, and why that is not the same place
//!
//! **Environment variables** go into the compose overlay, as `environment:` on
//! the project's service. They are the container's, not the web server's, and a
//! `.env` inside the project would be the application's own file — which this
//! app has no business writing, because Laravel, Symfony and every framework
//! since already own it.
//!
//! **Directory listing** goes into the *server* configuration, because that is
//! what it is: `autoindex on` for nginx, `file_server browse` for Caddy. It
//! reuses the per-project server-directives mechanism that already exists
//! rather than adding a second path into the same file.
//!
//! ## What this refuses
//!
//! A value with a newline in it. The overlay is YAML and the directive block is
//! nginx configuration, and in both a newline in a value is not an escaping
//! problem to solve — it is a second directive somebody else wrote.
//!
//! ## Host directories, and the gate in front of them
//!
//! A project sometimes needs a directory that is not inside it: a shared
//! library checked out beside it, which the application loads by a relative
//! path such as `../cbglib`. The container only sees `/var/www/html`, so
//! without a mount that path does not exist in there.
//!
//! **This is not in the manifest, and it is not trusted from this file
//! either.** `project.schema.json` refuses host paths on purpose: a repository
//! that could name one could mount the directory above itself. `site.json`
//! travels with a clone just as the manifest does, so a mount written into it
//! is a request rather than a decision. A mount reaches the overlay only when
//! two things agree:
//!
//! * it is in `.stackvo/site.json`, and
//! * the same resolved source, target and mode were saved **from this app** on
//!   this machine, which records it in `<root>/site-mounts.json` — a file no
//!   repository can write.
//!
//! A cloned `site.json` therefore shows its mounts as waiting for approval and
//! renders nothing until somebody saves them in the pane. Editing the source
//! by hand after approval changes what it resolves to, and the old approval no
//! longer matches.
//!
//! What a source may be is narrow: an existing directory **inside the projects
//! root**, after symlinks are resolved, that is neither the root itself nor the
//! project's own directory. `/`, the home directory, `~/.ssh` and the Docker
//! socket are all outside the projects root, so they are refused by the same
//! rule rather than by a list somebody has to keep complete.

use crate::error::{Code, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "site.json";

/// One project's settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Forward the host's SSH agent into the container.
    ///
    /// What it is for: `composer install` on a private repository, and
    /// `git pull` inside the container. Both need a key, and the alternative
    /// people reach for is copying `~/.ssh/id_ed25519` into the image — a
    /// private key baked into a layer that is then pushed somewhere.
    ///
    /// Off by default, and per project rather than global: forwarding an agent
    /// lets anything running in that container **sign with every key in it**
    /// for as long as it is up. That is the right trade while a dependency
    /// install is running and the wrong one as a permanent setting on a
    /// container running somebody else's code.
    #[serde(default)]
    pub ssh_agent: bool,
    /// Environment variables for the project's own container.
    ///
    /// Sorted, because the overlay is rendered from it before every compose
    /// command and a map that reorders would rewrite the file for nothing.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Serve a directory index where there is no index file.
    #[serde(default)]
    pub directory_listing: bool,
    /// Host directories this project asks to have mounted.
    ///
    /// A request, not a decision — see the module documentation. Skipped when
    /// empty so a `site.json` that never used this is written byte for byte as
    /// it was before the field existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mounts: Vec<Mount>,
}

/// One host directory, as the project asks for it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Mount {
    /// Relative to the project's directory (`../cbglib`) or absolute.
    pub source: String,
    /// Where it appears inside the container (`/var/www/cbglib`).
    pub target: String,
    /// Read-only unless the project says otherwise: a shared library is read
    /// by the application, and a container that can write into a sibling's
    /// checkout can break a project that is not the one running.
    #[serde(default = "read_only_by_default")]
    pub read_only: bool,
}

fn read_only_by_default() -> bool {
    true
}

/// The most mounts one project may ask for. A handful is the real use; a
/// limit keeps a hostile file from turning one overlay into thousands of lines.
pub const MAX_MOUNTS: usize = 16;

pub fn config_path(root: &Path, name: &str) -> PathBuf {
    crate::workspace::projects_root(root)
        .unwrap_or_default()
        .join(name)
        .join(crate::phpini::CONFIG_DIR)
        .join(FILE_NAME)
}

pub fn overlay_path(root: &Path) -> PathBuf {
    root.join("generated").join("docker-compose.site.yml")
}

/// Read one project's settings, dropping anything that is not writable back.
pub fn read(root: &Path, name: &str) -> Config {
    std::fs::read_to_string(config_path(root, name))
        .ok()
        .and_then(|text| serde_json::from_str::<Config>(&text).ok())
        .map(|mut config| {
            config
                .env
                .retain(|key, value| checked_key(key).is_ok() && checked_value(value).is_ok());
            // Only the shape is checked here. Whether the source exists, and
            // whether it was approved, is decided when the overlay is rendered
            // — a mount whose directory is missing today is still the
            // project's request and is shown, not silently forgotten.
            config.mounts.retain(|m| checked_mount_text(m).is_ok());
            config.mounts.truncate(MAX_MOUNTS);
            config
        })
        .unwrap_or_default()
}

pub fn write(root: &Path, name: &str, config: &Config) -> Result<()> {
    for (key, value) in &config.env {
        checked_key(key)?;
        checked_value(value)?;
    }
    if config.mounts.len() > MAX_MOUNTS {
        return Err(Error::new(
            Code::InvalidInput,
            format!("a project can mount at most {MAX_MOUNTS} host directories"),
        ));
    }
    for mount in &config.mounts {
        checked_mount_text(mount)?;
    }

    let path = config_path(root, name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|e| Error::new(Code::IoError, format!("serialising the settings: {e}")))?;
    crate::atomic::write(&path, &format!("{text}\n"))
}

/// Is this a name a shell would hand to a process?
///
/// The POSIX rule, and the same one `env_writer` enforces on the workspace
/// `.env`: letters, digits and underscores, not starting with a digit. A key
/// with a space in it is not a variable nothing can read — it is a line compose
/// parses as something else.
pub fn checked_key(key: &str) -> Result<()> {
    let ok = !key.is_empty()
        && key.len() <= 128
        && !key.starts_with(|c: char| c.is_ascii_digit())
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

    if !ok {
        return Err(Error::new(
            Code::InvalidInput,
            format!("\"{key}\" is not a usable environment variable name"),
        )
        .with_hint(crate::hints::ENV_KEY_CHARSET));
    }
    Ok(())
}

/// A value that stays one value.
pub fn checked_value(value: &str) -> Result<()> {
    if value.len() > 4096 {
        return Err(Error::new(
            Code::InvalidInput,
            "that value is too long for an environment variable".to_string(),
        ));
    }
    // A newline in a YAML scalar ends it. Everything after would be read as
    // more configuration, at whatever indentation it happens to have.
    if value.contains(['\n', '\r']) || value.contains('\0') {
        return Err(Error::new(
            Code::InvalidInput,
            "an environment variable's value cannot contain a line break".to_string(),
        )
        .with_hint(crate::hints::ENV_IS_ONE_KEY_PER_LINE));
    }
    Ok(())
}

// --------------------------------------------------------- host directories

/// Container paths a mount may not land on or under.
///
/// `/var/www/html` is the project's own bind mount: a mount beneath it makes
/// Docker create the mount point *in the host checkout*, and one on it replaces
/// the project. The rest are the image's own system directories, where a
/// directory from the host is not a library — it is a different container.
const RESERVED_TARGETS: &[&str] = &[
    "/var/www/html",
    "/bin",
    "/boot",
    "/dev",
    "/etc",
    "/lib",
    "/lib64",
    "/proc",
    "/run",
    "/sbin",
    "/sys",
    "/usr",
    "/var/lib",
    "/var/log",
    "/var/run",
];

fn invalid(message: String) -> Error {
    Error::new(Code::InvalidInput, message)
}

/// Is the target a container path compose will read as one path?
///
/// Absolute, made of plain segments, and drawn from a character set with no
/// colon in it — compose splits a mount on `:`, so a colon in the target is a
/// second field rather than part of a name.
pub fn checked_target(target: &str) -> Result<()> {
    let plain = target.len() <= 256
        && target.starts_with('/')
        && target
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/-_.@+".contains(c));
    let segments_ok = plain
        && target[1..]
            .split('/')
            .all(|s| !s.is_empty() && s != "." && s != "..");

    if !segments_ok {
        return Err(invalid(format!(
            "\"{target}\" is not a usable container path — it must be absolute, like /var/www/cbglib"
        )));
    }
    if let Some(reserved) = RESERVED_TARGETS
        .iter()
        .find(|r| target == **r || target.starts_with(&format!("{r}/")))
    {
        return Err(invalid(format!(
            "\"{target}\" is inside {reserved}, which the container needs for itself"
        )));
    }
    Ok(())
}

/// The checks that need nothing but the text: what `read` and `write` apply.
fn checked_mount_text(mount: &Mount) -> Result<()> {
    let source = &mount.source;
    if source.trim().is_empty() || source.len() > 1024 {
        return Err(invalid("a mount needs a source directory".to_string()));
    }
    // Same reason as `checked_value`: the source ends up inside a quoted YAML
    // scalar, and a line break or a quote there is structure, not a path.
    if source.contains(['\n', '\r', '\0', '"']) {
        return Err(invalid(format!(
            "\"{}\" cannot be used as a source directory",
            source.escape_debug()
        )));
    }
    checked_target(&mount.target)
}

/// Resolve a mount's source on this machine, or say why it cannot be used.
///
/// Returns the canonical host path. The rule is containment, not a list of
/// forbidden places: inside the projects root, after symlinks, and neither the
/// root nor the project itself.
pub fn resolve_source(root: &Path, name: &str, mount: &Mount) -> Result<PathBuf> {
    checked_mount_text(mount)?;

    let projects = crate::workspace::projects_root(root)
        .ok_or_else(|| invalid("no projects directory is set".to_string()))?;
    let projects = std::fs::canonicalize(&projects)
        .map_err(|e| Error::io(format!("resolving {}", projects.display()), e))?;
    let project = projects.join(name);

    let asked = Path::new(&mount.source);
    let candidate = if asked.is_absolute() {
        asked.to_path_buf()
    } else {
        project.join(asked)
    };
    let resolved = std::fs::canonicalize(&candidate).map_err(|_| {
        invalid(format!(
            "\"{}\" does not exist on this machine",
            mount.source
        ))
    })?;

    if !resolved.is_dir() {
        return Err(invalid(format!("\"{}\" is not a directory", mount.source)));
    }
    let project = std::fs::canonicalize(&project).unwrap_or(project);
    if !resolved.starts_with(&projects) || resolved == projects {
        return Err(invalid(format!(
            "\"{}\" is outside the projects directory ({}) — only a directory beside your projects can be mounted",
            mount.source,
            projects.display()
        )));
    }
    if resolved == project || resolved.starts_with(&project) {
        return Err(invalid(format!(
            "\"{}\" is part of this project, which is already mounted at /var/www/html",
            mount.source
        )));
    }

    // Compose splits on `:`. A drive letter is rewritten away by
    // `to_docker_mount`; a colon anywhere else in the path cannot be written
    // in the short syntax at all.
    if spec_source(&resolved).contains(':') {
        return Err(invalid(format!(
            "\"{}\" has a colon in its path, which compose cannot mount",
            resolved.display()
        )));
    }
    Ok(resolved)
}

fn spec_source(resolved: &Path) -> String {
    crate::paths::to_docker_mount(&resolved.display().to_string())
}

/// The compose short-syntax line for a resolved mount, unquoted.
///
/// This exact string is also what an approval records, so changing the source,
/// the target or the mode is a different mount that has to be approved again.
fn spec(resolved: &Path, mount: &Mount) -> String {
    format!(
        "{}:{}{}",
        spec_source(resolved),
        mount.target,
        if mount.read_only { ":ro" } else { "" }
    )
}

pub fn approvals_path(root: &Path) -> PathBuf {
    root.join("site-mounts.json")
}

/// What this machine has agreed to mount, per project.
fn load_approvals(root: &Path) -> BTreeMap<String, Vec<String>> {
    std::fs::read_to_string(approvals_path(root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn store_approvals(root: &Path, approvals: &BTreeMap<String, Vec<String>>) -> Result<()> {
    let path = approvals_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    let text = serde_json::to_string_pretty(approvals)
        .map_err(|e| Error::new(Code::IoError, format!("serialising the approvals: {e}")))?;
    crate::atomic::write(&path, &format!("{text}\n"))
}

/// One mount as the pane shows it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MountRow {
    pub source: String,
    pub target: String,
    pub read_only: bool,
    /// The host directory it resolves to here, when it resolves.
    pub resolved: Option<String>,
    /// Saved from this app on this machine, for exactly this source, target
    /// and mode — which is what puts it in the overlay.
    pub approved: bool,
    /// Why it cannot be mounted here, when it cannot.
    pub error: Option<String>,
}

/// Every mount this project asks for, with whether it will be applied.
pub fn mount_rows(root: &Path, name: &str) -> Vec<MountRow> {
    let approved = load_approvals(root).remove(name).unwrap_or_default();
    read(root, name)
        .mounts
        .into_iter()
        .map(|mount| match resolve_source(root, name, &mount) {
            Ok(resolved) => MountRow {
                approved: approved.contains(&spec(&resolved, &mount)),
                resolved: Some(resolved.display().to_string()),
                error: None,
                source: mount.source,
                target: mount.target,
                read_only: mount.read_only,
            },
            Err(e) => MountRow {
                approved: false,
                resolved: None,
                error: Some(e.message.clone()),
                source: mount.source,
                target: mount.target,
                read_only: mount.read_only,
            },
        })
        .collect()
}

/// Replace a project's mounts and approve every one of them on this machine.
///
/// This is the gate: it is only reachable from the pane, and it refuses the
/// whole list if any one entry does not resolve, so what is approved is always
/// exactly what was shown.
pub fn save_mounts(root: &Path, name: &str, mounts: Vec<Mount>) -> Result<()> {
    if mounts.len() > MAX_MOUNTS {
        return Err(invalid(format!(
            "a project can mount at most {MAX_MOUNTS} host directories"
        )));
    }

    let mut specs = Vec::with_capacity(mounts.len());
    let mut targets = std::collections::BTreeSet::new();
    for mount in &mounts {
        let resolved = resolve_source(root, name, mount)?;
        if !targets.insert(mount.target.clone()) {
            return Err(invalid(format!(
                "{} is the target of two mounts",
                mount.target
            )));
        }
        specs.push(spec(&resolved, mount));
    }

    let mut config = read(root, name);
    config.mounts = mounts;
    write(root, name, &config)?;

    let mut approvals = load_approvals(root);
    if specs.is_empty() {
        approvals.remove(name);
    } else {
        approvals.insert(name.to_string(), specs);
    }
    store_approvals(root, &approvals)
}

/// The mounts that go into the overlay: requested, resolvable and approved.
fn approved_specs(root: &Path, name: &str, mounts: &[Mount], approved: &[String]) -> Vec<String> {
    mounts
        .iter()
        .filter_map(|mount| {
            let resolved = resolve_source(root, name, mount).ok()?;
            let line = spec(&resolved, mount);
            approved.contains(&line).then_some(line)
        })
        .collect()
}

/// The server directives that turn a directory index on.
///
/// Returned per server rather than written once, because the two that have a
/// configuration file spell it differently and the three that do not have one
/// cannot do it at all — Apache is configured by `sed` inside its own
/// Dockerfile and Swoole by an inline script, which is the same boundary
/// `ServerExtras` already documents.
pub fn listing_directives(server: &str) -> Option<&'static str> {
    match server {
        // `autoindex_exact_size off` prints 4.0K rather than 4096, which is
        // what a person reading a directory listing wants.
        "nginx" => Some("autoindex on;\nautoindex_exact_size off;\nautoindex_localtime on;"),
        "caddy" | "frankenphp" => Some("file_server browse"),
        _ => None,
    }
}

// -------------------------------------------------------------- the overlay

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub service: String,
    pub env: BTreeMap<String, String>,
    /// Whether this project asked for the agent.
    pub ssh_agent: bool,
    /// Approved host directories, as compose short-syntax lines.
    pub mounts: Vec<String>,
}

/// Where the host's agent socket is, as the container has to reach it.
///
/// **Not `$SSH_AUTH_SOCK`.** On macOS and Windows the daemon runs in a VM and
/// the host's socket path means nothing inside it; Docker Desktop publishes the
/// agent at a fixed path instead, and mounting that is the documented way. On
/// Linux the daemon is the host, so the real socket is the answer and the fixed
/// path does not exist.
///
/// Returned as `(host source, container target)` because they differ: the
/// target is always the same inside the container, so `SSH_AUTH_SOCK` can be a
/// constant and every project's container looks alike.
///
/// Measured on macOS rather than taken from documentation. The host's own
/// socket was `/var/run/com.apple.launchd.dV6erVP2st/Listeners`, and mounting
/// Docker Desktop's path instead produced a container that could **talk to the
/// agent**: `ssh-add -l` inside it answered *"The agent has no identities"* —
/// which is the agent replying. A socket that had not been forwarded gives
/// *"Could not open a connection to your authentication agent"*, and the two
/// messages are the whole difference between this working and not.
pub fn agent_socket() -> Option<(String, &'static str)> {
    const TARGET: &str = "/run/host-services/ssh-auth.sock";

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        // Docker Desktop's own magic path. It exists whether or not an agent is
        // running on the host, so the check that matters is the one below.
        std::env::var_os("SSH_AUTH_SOCK")
            .filter(|value| !value.is_empty())
            .map(|_| ("/run/host-services/ssh-auth.sock".to_string(), TARGET))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let socket = std::env::var_os("SSH_AUTH_SOCK")?;
        let path = std::path::PathBuf::from(&socket);
        path.exists().then(|| (path.display().to_string(), TARGET))
    }
}

/// Render the overlay, or `None` when no project sets anything.
///
/// The same rule the other three overlays follow: compose rejects a file whose
/// `services` map is empty, so "nothing to add" has to mean "no file".
pub fn overlay_yaml(entries: &[Entry]) -> Option<String> {
    let agent = agent_socket();
    let entries: Vec<&Entry> = entries
        .iter()
        .filter(|e| !e.env.is_empty() || (e.ssh_agent && agent.is_some()) || !e.mounts.is_empty())
        .collect();
    if entries.is_empty() {
        return None;
    }

    let mut out = String::from(
        "# Generated by StackVo Desktop — do not edit.\n\
         #\n\
         # Re-rendered from projects/*/.stackvo/site.json before every compose\n\
         # command, so edits here are lost. Change them in the app instead.\n\
         #\n\
         # NOTE: `stackvo up` from the Bash CLI does not layer this file, and\n\
         # will recreate these containers without these variables.\n\
         services:\n",
    );

    let mut sorted = entries.clone();
    sorted.sort_by(|a, b| a.service.cmp(&b.service));

    for entry in sorted {
        out.push_str(&format!("  {}:\n", entry.service));

        let forwarding = entry.ssh_agent.then_some(agent.as_ref()).flatten();
        if !entry.env.is_empty() || forwarding.is_some() {
            out.push_str("    environment:\n");
        }
        for (key, value) in &entry.env {
            // Double-quoted with the two characters YAML reads inside quotes
            // escaped. A value is one line by the time it gets here —
            // `checked_value` refuses the rest — so this is the whole of the
            // quoting problem rather than the start of one.
            out.push_str(&format!("      {key}: \"{}\"\n", escape(value)));
        }
        if let Some((_, target)) = forwarding {
            out.push_str(&format!("      SSH_AUTH_SOCK: \"{target}\"\n"));
        }
        if forwarding.is_some() || !entry.mounts.is_empty() {
            out.push_str("    volumes:\n");
        }
        if let Some((source, target)) = forwarding {
            // Quoted because a Windows host path holds a colon, which is what
            // compose splits a mount on — the same rule `phpini` follows.
            out.push_str(&format!("      - \"{source}:{target}\"\n"));
        }
        for line in &entry.mounts {
            // Already a resolved, approved short-syntax line; `resolve_source`
            // refused anything with a quote or a stray colon in it.
            out.push_str(&format!("      - \"{line}\"\n"));
        }
    }

    Some(out)
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Every project with variables **and** a compose service to set them on.
///
/// Two sources, laid one over the other. `.stackvo/site.json` is what the
/// project itself declares and travels with a clone; a worktree's record is
/// what *this machine* gave one branch, and it is deliberately not in the
/// checkout — see [`crate::worktree`] for why nothing may be written into a
/// worktree's directory that git would notice.
///
/// The worktree's values win where the two name the same variable. That is the
/// order the specificity implies: a per-branch database is a narrower statement
/// than the project's own default, and the person who created the worktree said
/// so more recently than the person who committed `site.json`.
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

    // Once, not per project: a workspace with forty projects and two worktrees
    // would otherwise read and parse the same file forty times.
    let worktrees = crate::worktree::Table::load(root).unwrap_or_default();
    let mut approvals = load_approvals(root);

    let mut out = Vec::new();
    for item in dir.flatten() {
        let Some(name) = item
            .path()
            .file_name()
            .and_then(|n| n.to_str().map(str::to_string))
        else {
            continue;
        };
        if !services.contains(&name) {
            continue;
        }
        let config = read(root, &name);
        let mut env = config.env;
        if let Some(record) = worktrees.get(&name) {
            env.extend(crate::worktree::env_for(root, record));
        }
        let approved = approvals.remove(&name).unwrap_or_default();
        let mounts = approved_specs(root, &name, &config.mounts, &approved);
        if env.is_empty() && !config.ssh_agent && mounts.is_empty() {
            continue;
        }
        out.push(Entry {
            service: name,
            env,
            ssh_agent: config.ssh_agent,
            mounts,
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

    fn config(pairs: &[(&str, &str)], listing: bool) -> Config {
        Config {
            env: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            directory_listing: listing,
            ssh_agent: false,
            mounts: Vec::new(),
        }
    }

    /// The socket the container mounts is a fixed path on the two
    /// platforms where the daemon is in a VM, and the real one on the platform
    /// where it is not — mounting `$SSH_AUTH_SOCK` on macOS forwards a path
    /// that means nothing inside the VM.
    #[test]
    fn the_agent_is_forwarded_through_the_path_the_platform_has() {
        let entry = Entry {
            service: "shop".into(),
            env: BTreeMap::new(),
            ssh_agent: true,
            mounts: Vec::new(),
        };

        match agent_socket() {
            Some((source, target)) => {
                let yaml = overlay_yaml(&[entry]).expect("an overlay");
                assert!(
                    yaml.contains(&format!("SSH_AUTH_SOCK: \"{target}\"")),
                    "{yaml}"
                );
                assert!(yaml.contains(&format!("- \"{source}:{target}\"")), "{yaml}");
                assert_eq!(target, "/run/host-services/ssh-auth.sock");
                #[cfg(target_os = "macos")]
                assert_eq!(
                    source, "/run/host-services/ssh-auth.sock",
                    "Docker Desktop publishes the agent here; the host path is \
                     meaningless inside the VM"
                );
            }
            // No agent running: asking for one has to produce nothing rather
            // than a mount of a socket that is not there, which is a container
            // that fails to start.
            None => assert!(overlay_yaml(&[entry]).is_none()),
        }
    }

    /// A name compose would read as something other than a variable.
    #[test]
    fn a_key_that_is_not_a_variable_name_is_refused() {
        for good in ["APP_ENV", "_X", "A1", "STRIPE_KEY"] {
            assert!(checked_key(good).is_ok(), "{good}");
        }
        for bad in [
            "", "1ST", "APP ENV", "APP-ENV", "APP.ENV", "APP=ENV", "ünlü",
        ] {
            assert!(checked_key(bad).is_err(), "{bad} was accepted");
        }
    }

    /// A newline in a YAML scalar ends it, and everything after is read as
    /// configuration somebody else wrote.
    #[test]
    fn a_value_cannot_carry_a_second_line() {
        assert!(checked_value("ordinary").is_ok());
        assert!(checked_value("with spaces and: colons").is_ok());
        assert!(checked_value("a\nb").is_err());
        assert!(checked_value("a\r\nb").is_err());
        assert!(checked_value("a\0b").is_err());
    }

    /// The quoting is what stands between a value and the file's structure.
    #[test]
    fn a_value_stays_a_value_in_the_overlay() {
        let yaml = overlay_yaml(&[Entry {
            service: "shop".into(),
            ssh_agent: false,
            mounts: Vec::new(),
            env: config(
                &[
                    ("APP_ENV", "local"),
                    ("QUOTED", r#"say "hi""#),
                    ("WINDOWS", r"C:\tmp"),
                    ("COLON", "a: b"),
                ],
                false,
            )
            .env,
        }])
        .expect("an overlay");

        assert!(yaml.contains("  shop:\n    environment:\n"), "{yaml}");
        assert!(yaml.contains(r#"      APP_ENV: "local""#), "{yaml}");
        assert!(yaml.contains(r#"      QUOTED: "say \"hi\"""#), "{yaml}");
        assert!(yaml.contains(r#"      WINDOWS: "C:\\tmp""#), "{yaml}");
        // A colon inside a quoted scalar is a colon, not a mapping.
        assert!(yaml.contains(r#"      COLON: "a: b""#), "{yaml}");
    }

    #[test]
    fn nothing_to_set_is_no_file_at_all() {
        assert!(overlay_yaml(&[]).is_none());
        assert!(overlay_yaml(&[Entry {
            service: "shop".into(),
            env: BTreeMap::new(),
            ssh_agent: false,
            mounts: Vec::new(),
        }])
        .is_none());
    }

    /// A hand-edited file is read defensively — what is acted on is only ever
    /// what this module would have written.
    #[test]
    fn a_hostile_setting_on_disk_is_dropped_rather_than_rendered() {
        let dir = std::env::temp_dir().join(format!("stackvo-site-{}", std::process::id()));
        let projects = dir.join("projects");
        std::fs::create_dir_all(projects.join("shop").join(".stackvo")).unwrap();
        std::fs::write(dir.join("projects.path"), projects.display().to_string()).unwrap();
        std::fs::write(
            projects.join("shop").join(".stackvo").join(FILE_NAME),
            "{\"env\":{\"GOOD\":\"1\",\"BAD KEY\":\"2\",\"NEWLINE\":\"a\\nb\"},\"directoryListing\":true}",
        )
        .unwrap();

        let config = read(&dir, "shop");
        assert_eq!(config.env.len(), 1, "{:?}", config.env);
        assert_eq!(config.env.get("GOOD").map(String::as_str), Some("1"));
        assert!(config.directory_listing);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// This is a server setting, and the servers that have no configuration file
    /// cannot do it — which is a sentence the pane has to be able to say.
    #[test]
    fn only_the_servers_with_a_config_file_can_list_a_directory() {
        assert!(listing_directives("nginx")
            .unwrap()
            .contains("autoindex on;"));
        assert!(listing_directives("caddy").unwrap().contains("browse"));
        assert!(listing_directives("frankenphp").is_some());
        assert_eq!(listing_directives("apache"), None);
        assert_eq!(listing_directives("swoole"), None);
    }

    /// Round trip: what is written is what comes back.
    #[test]
    fn what_is_written_is_what_is_read() {
        let dir = std::env::temp_dir().join(format!("stackvo-site-rt-{}", std::process::id()));
        let projects = dir.join("projects");
        std::fs::create_dir_all(projects.join("shop")).unwrap();
        std::fs::write(dir.join("projects.path"), projects.display().to_string()).unwrap();

        let written = config(&[("APP_ENV", "local"), ("DEBUG", "true")], true);
        write(&dir, "shop", &written).unwrap();
        assert_eq!(read(&dir, "shop"), written);

        // And refusing happens before anything is written.
        assert!(write(&dir, "shop", &config(&[("BAD KEY", "1")], false)).is_err());
        assert_eq!(read(&dir, "shop"), written, "the good file survived");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ----------------------------------------------------- host directories

    /// A workspace with a project `shop`, a sibling library `lib`, and a
    /// directory outside the projects root. Returned as (root, projects, outside).
    fn mount_workspace(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("stackvo-site-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let projects = dir.join("projects");
        std::fs::create_dir_all(projects.join("shop").join("src")).unwrap();
        std::fs::create_dir_all(projects.join("lib")).unwrap();
        let outside = dir.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(dir.join("projects.path"), projects.display().to_string()).unwrap();
        (dir, projects, outside)
    }

    fn mount(source: &str, target: &str) -> Mount {
        Mount {
            source: source.into(),
            target: target.into(),
            read_only: true,
        }
    }

    /// The default is the safe one: a mount nobody marked writable is read-only.
    #[test]
    fn a_mount_is_read_only_unless_it_says_otherwise() {
        let parsed: Mount =
            serde_json::from_str(r#"{"source":"../lib","target":"/var/www/lib"}"#).unwrap();
        assert!(parsed.read_only);
    }

    /// A `site.json` that never used mounts is written exactly as before.
    #[test]
    fn no_mounts_writes_no_mounts_key() {
        let text = serde_json::to_string(&config(&[], false)).unwrap();
        assert!(!text.contains("mounts"), "{text}");
    }

    #[test]
    fn a_target_must_be_a_plain_absolute_path_outside_the_reserved_ones() {
        for good in [
            "/var/www/cbglib",
            "/opt/lib",
            "/srv/shared-lib_2",
            "/home/app/.lib",
        ] {
            assert!(checked_target(good).is_ok(), "{good}");
        }
        for bad in [
            "",
            "relative/path",
            "/",
            "/var/www/html",
            "/var/www/html/vendor",
            "/etc",
            "/etc/nginx",
            "/usr/local/lib",
            "/proc/self",
            "/var/run/docker.sock",
            "/opt/../etc",
            "/opt/./lib",
            "/opt//lib",
            "/opt/lib/",
            "/opt/a:b",
            "/opt/with space",
            "/opt/\"quoted\"",
            "/opt/line\nbreak",
        ] {
            assert!(checked_target(bad).is_err(), "{bad:?} was accepted");
        }
    }

    /// The containment rule: a sibling is fine, everything that is not inside
    /// the projects root — or is the root, or is the project — is refused.
    #[test]
    fn only_a_directory_beside_the_project_resolves() {
        let (dir, projects, outside) = mount_workspace("resolve");
        let lib = std::fs::canonicalize(projects.join("lib")).unwrap();

        let resolved = resolve_source(&dir, "shop", &mount("../lib", "/var/www/lib")).unwrap();
        assert_eq!(resolved, lib);
        // An absolute path to the same place is the same mount.
        let absolute = mount(&projects.join("lib").display().to_string(), "/var/www/lib");
        assert_eq!(resolve_source(&dir, "shop", &absolute).unwrap(), lib);

        for (source, why) in [
            ("../../outside", "outside the projects root"),
            ("..", "the projects root itself"),
            (".", "the project itself"),
            ("src", "inside the project"),
            ("../missing", "does not exist"),
            ("/", "the filesystem root"),
        ] {
            assert!(
                resolve_source(&dir, "shop", &mount(source, "/var/www/lib")).is_err(),
                "{source} ({why}) was accepted"
            );
        }
        assert!(resolve_source(
            &dir,
            "shop",
            &mount(&outside.display().to_string(), "/var/www/lib")
        )
        .is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A symlink inside the project that points out of the projects root is
    /// judged by where it points, not by where it sits.
    #[cfg(unix)]
    #[test]
    fn a_symlink_cannot_carry_a_mount_out_of_the_projects_root() {
        let (dir, projects, outside) = mount_workspace("symlink");
        std::os::unix::fs::symlink(&outside, projects.join("escape")).unwrap();
        assert!(resolve_source(&dir, "shop", &mount("../escape", "/var/www/lib")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The gate itself: a mount that arrived in `site.json` — by a clone, or a
    /// hand edit — is shown but not rendered until it is saved from the app.
    #[test]
    fn a_mount_is_rendered_only_after_it_is_approved_here() {
        let (dir, projects, _) = mount_workspace("gate");
        let lib = std::fs::canonicalize(projects.join("lib")).unwrap();

        // Written as a clone would bring it: straight into the file.
        let mut requested = config(&[], false);
        requested.mounts = vec![mount("../lib", "/var/www/lib")];
        write(&dir, "shop", &requested).unwrap();

        let rows = mount_rows(&dir, "shop");
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].approved, "a cloned mount is not approved");
        assert_eq!(
            rows[0].resolved.as_deref(),
            Some(lib.display().to_string().as_str())
        );
        let approvals = load_approvals(&dir).remove("shop").unwrap_or_default();
        assert!(approved_specs(&dir, "shop", &requested.mounts, &approvals).is_empty());

        // Saved from the pane: approved, and rendered.
        save_mounts(&dir, "shop", requested.mounts.clone()).unwrap();
        let rows = mount_rows(&dir, "shop");
        assert!(rows[0].approved);
        let approvals = load_approvals(&dir).remove("shop").unwrap_or_default();
        let lines = approved_specs(&dir, "shop", &requested.mounts, &approvals);
        // In Docker's shape, not the host's: on Windows the canonical path is
        // `\\?\C:\…` and the overlay carries `/c/…`.
        assert_eq!(
            lines,
            vec![format!("{}:/var/www/lib:ro", spec_source(&lib))]
        );

        // A hand edit to the mode is a different mount and needs approving again.
        let mut edited = read(&dir, "shop");
        edited.mounts[0].read_only = false;
        write(&dir, "shop", &edited).unwrap();
        assert!(!mount_rows(&dir, "shop")[0].approved);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// One bad entry refuses the whole save, so nothing half-approved exists.
    #[test]
    fn saving_mounts_is_all_or_nothing() {
        let (dir, _, _) = mount_workspace("atomic");
        let result = save_mounts(
            &dir,
            "shop",
            vec![
                mount("../lib", "/var/www/lib"),
                mount("../../outside", "/opt/x"),
            ],
        );
        assert!(result.is_err());
        assert!(read(&dir, "shop").mounts.is_empty());
        assert!(load_approvals(&dir).is_empty());

        // Two mounts on one target are refused too.
        assert!(save_mounts(
            &dir,
            "shop",
            vec![
                mount("../lib", "/var/www/lib"),
                mount("../lib", "/var/www/lib")
            ],
        )
        .is_err());

        // And an empty list clears the approval.
        save_mounts(&dir, "shop", vec![mount("../lib", "/var/www/lib")]).unwrap();
        save_mounts(&dir, "shop", Vec::new()).unwrap();
        assert!(!load_approvals(&dir).contains_key("shop"));
        assert!(read(&dir, "shop").mounts.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The overlay shape: one `volumes:` key, the agent and the mounts under it.
    #[test]
    fn approved_mounts_land_under_the_service_volumes() {
        let yaml = overlay_yaml(&[Entry {
            service: "shop".into(),
            env: BTreeMap::new(),
            ssh_agent: false,
            mounts: vec!["/w/projects/lib:/var/www/lib:ro".into()],
        }])
        .expect("an overlay");
        assert!(
            yaml.contains("  shop:\n    volumes:\n      - \"/w/projects/lib:/var/www/lib:ro\"\n"),
            "{yaml}"
        );
        assert!(!yaml.contains("environment:"), "{yaml}");
        assert_eq!(yaml.matches("volumes:").count(), 1, "{yaml}");
    }
}
