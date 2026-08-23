//! Wiring an IDE to the debugger, rather than describing how.
//!
//! Every competitor's step-debugging page is the same page: here is the port,
//! here is the host, here is the path mapping, now open your IDE and type them
//! in. DDEV writes it out for PhpStorm and VS Code, Laradock gives a table,
//! ServBay gives a table with a different port per PHP version, Herd gives a
//! `php.ini` snippet. All five then name the same failure as the common one —
//! **the path mapping** — and none of them fills it in for you.
//!
//! It is the right thing to name. `xdebug.rs` already computes both halves of
//! that mapping: the host directory the project lives in and
//! `/var/www/html` inside the container. They were on the status object and on
//! the screen, as two strings for somebody to copy into a dialog by hand.
//!
//! ## What is written, and what is only shown
//!
//! **VS Code is written.** Its debug configuration is `launch.json` in the
//! project, it is plain data, and the mapping can be expressed portably —
//! `${workspaceFolder}` rather than this machine's path — so the file a
//! colleague clones works on their machine too.
//!
//! **PhpStorm is not.** Its equivalent lives in `.idea/php.xml` and
//! `.idea/workspace.xml`, which the IDE keeps in memory and rewrites on exit;
//! a file edited underneath a running PhpStorm is a file PhpStorm overwrites,
//! and the user is left with a tool that says it configured something and an
//! IDE that disagrees. So this detects the project is a PhpStorm project,
//! computes the same three values, and hands over the XML to paste — which is
//! the competitors' path, aimed at the exact file with the values already
//! filled in.
//!
//! Refusing to write is not a smaller feature than writing badly. `agents.rs`
//! reached the same conclusion about VS Code's own `mcp.json` when it has
//! comments in it, and this module follows its three rules for the file it
//! *does* write: read, replace one entry, write back; never touch a file that
//! does not parse; keep a copy beside it first.
//!
//! ## The listener
//!
//! The other half of "why does my breakpoint not hit" is not in any file. The
//! IDE has to be listening, and nothing in an IDE says loudly that it is not.
//! DDEV is the only one of the five with a tool for this — `ddev utility
//! xdebug-diagnose` — and it is a separate command somebody has to know about.
//!
//! [`listener`] answers it from the OS's own table of listening sockets, the
//! same one `doctor` reads to say who holds port 80. It is a read, not a
//! connection: dialling the DBGp port to see whether anything answers would
//! show up in the IDE as a debug session that immediately dropped, which is
//! noise this app has no business generating on somebody's screen.

use crate::error::{Code, Error, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The suffix of the copy taken before a file is rewritten — the same one
/// `agents.rs` and `rules.rs` use, for the same reason.
pub const BACKUP_SUFFIX: &str = ".stackvo-backup";

/// How a target is set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    /// This app edits the file.
    Written,
    /// The values are computed and shown; the user pastes them.
    Shown,
}

/// One IDE, and where its debug configuration lives in a project.
pub struct Target {
    pub id: &'static str,
    /// Product names, so not translated.
    pub label: &'static str,
    /// Relative to the project directory.
    pub path: &'static str,
    /// The directory whose presence means this IDE is used on this project.
    ///
    /// `.vscode` and `.idea` are both created by the IDE the first time
    /// anything is configured, so their presence is the closest thing to
    /// "this project is opened in that editor" that exists on disk.
    pub marker: &'static str,
    pub method: Method,
}

pub const TARGETS: &[Target] = &[
    Target {
        id: "vscode",
        label: "VS Code",
        path: ".vscode/launch.json",
        marker: ".vscode",
        method: Method::Written,
    },
    Target {
        id: "phpstorm",
        label: "PhpStorm",
        // Named even though it is never written: the reader is going to open
        // it, and a row that says "PhpStorm" and no path sends them looking.
        path: ".idea/php.xml",
        marker: ".idea",
        method: Method::Shown,
    },
];

pub fn target(id: &str) -> Option<&'static Target> {
    TARGETS.iter().find(|t| t.id == id)
}

/// The name this app files its configuration under.
///
/// The project is in it because a workspace with two projects open produces
/// two entries in one `launch.json`, and two called "Listen for Xdebug" is a
/// dropdown nobody can use.
pub fn configuration_name(project: &str) -> String {
    format!("Listen for StackVo: {project}")
}

// ------------------------------------------------------------------ VS Code

/// The debug configuration itself.
///
/// `pathMappings` is the whole point and is written **remote → local**, which
/// is the direction the extension expects and the one people reverse. The local
/// side is `${workspaceFolder}` rather than this machine's absolute path: the
/// file is committed by roughly everybody, and an absolute path in it is a
/// configuration that works for exactly one person.
pub fn vscode_configuration(project: &str, port: u16, container_path: &str) -> Value {
    json!({
        "name": configuration_name(project),
        "type": "php",
        "request": "launch",
        "port": port,
        "pathMappings": { container_path: "${workspaceFolder}" },
    })
}

/// The whole file, as it would be created from nothing.
pub fn vscode_document(project: &str, port: u16, container_path: &str) -> Value {
    json!({
        "version": "0.2.0",
        "configurations": [vscode_configuration(project, port, container_path)],
    })
}

/// Parse a `launch.json`, or say why it cannot be edited.
///
/// VS Code's own files are JSON **with comments** — it creates `launch.json`
/// with three of them — and `serde_json` refuses. That is reported rather than
/// worked around, because the tempting fix strips the comments and hands
/// somebody back a file with their own notes deleted. `agents.rs` made this
/// decision first and this is the same file format in the same editor.
fn document(text: &str) -> Result<Value> {
    if text.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }

    let value: Value = serde_json::from_str(text).map_err(|e| {
        Error::new(
            Code::InvalidInput,
            format!("launch.json is not plain JSON: {e}"),
        )
        .with_hint(crate::hints::LAUNCH_JSON_HAS_COMMENTS)
    })?;

    if !value.is_object() {
        return Err(Error::new(
            Code::InvalidInput,
            "launch.json's top level is not a JSON object".to_string(),
        )
        .with_hint(crate::hints::LAUNCH_JSON_HAS_COMMENTS));
    }
    Ok(value)
}

/// `text` with our configuration inserted or replaced, and nothing else
/// changed.
pub fn insert(text: &str, project: &str, port: u16, container_path: &str) -> Result<String> {
    let mut document = document(text)?;
    let object = document.as_object_mut().expect("checked in `document`");

    // `version` only when the file did not have one. Overwriting it would be
    // this code having an opinion about a field it does not use.
    object
        .entry("version")
        .or_insert_with(|| Value::String("0.2.0".into()));

    let list = object
        .entry("configurations")
        .or_insert_with(|| Value::Array(Vec::new()));
    if list.is_null() {
        *list = Value::Array(Vec::new());
    }
    let Some(list) = list.as_array_mut() else {
        return Err(Error::new(
            Code::Conflict,
            "`configurations` in launch.json is not a list".to_string(),
        )
        .with_hint(crate::hints::LAUNCH_JSON_HAS_COMMENTS));
    };

    let entry = vscode_configuration(project, port, container_path);
    let name = configuration_name(project);
    match list
        .iter()
        .position(|c| c.get("name").and_then(Value::as_str) == Some(name.as_str()))
    {
        // Replaced in place rather than removed and pushed: the order of this
        // list is the order of the IDE's dropdown, and somebody who dragged
        // ours to the top meant it.
        Some(at) => list[at] = entry,
        None => list.push(entry),
    }

    crate::agents::render(&document, text)
}

/// `text` without our configuration. Everything else stays, including an empty
/// `configurations` — removing the key as well would be this code deciding
/// something about a file it does not own.
pub fn remove_from(text: &str, project: &str) -> Result<String> {
    let mut document = document(text)?;
    let name = configuration_name(project);

    if let Some(list) = document
        .as_object_mut()
        .and_then(|o| o.get_mut("configurations"))
        .and_then(Value::as_array_mut)
    {
        list.retain(|c| c.get("name").and_then(Value::as_str) != Some(name.as_str()));
    }

    crate::agents::render(&document, text)
}

/// Is our configuration already in this file, and is it the current one?
pub fn state(text: &str, project: &str, port: u16, container_path: &str) -> (bool, bool) {
    let Ok(document) = serde_json::from_str::<Value>(text) else {
        return (false, false);
    };
    let name = configuration_name(project);
    let Some(found) = document
        .get("configurations")
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .find(|c| c.get("name").and_then(Value::as_str) == Some(name.as_str()))
        })
    else {
        return (false, false);
    };

    (
        true,
        *found == vscode_configuration(project, port, container_path),
    )
}

// ----------------------------------------------------------------- PhpStorm

/// The `PhpServers` entry PhpStorm needs, ready to paste into `.idea/php.xml`.
///
/// Shown rather than written — see the module note. The server *name* is the
/// one that matters and is the one people get wrong: Xdebug sends
/// `PHP_IDE_CONFIG=serverName=…` and PhpStorm matches the mapping by that
/// name, so a server called anything else is a debugger that connects and then
/// cannot find the file.
pub fn phpstorm_snippet(server_name: &str, host_path: &str, container_path: &str) -> String {
    format!(
        "<component name=\"PhpProjectServersManager\">\n  \
         <servers>\n    \
         <server host=\"{server_name}\" id=\"stackvo-{server_name}\" name=\"{server_name}\" \
         use_path_mappings=\"true\">\n      \
         <path_mappings>\n        \
         <mapping local-root=\"{host_path}\" remote-root=\"{container_path}\" />\n      \
         </path_mappings>\n    \
         </server>\n  \
         </servers>\n\
         </component>\n"
    )
}

// -------------------------------------------------------------- the listener

/// What, if anything, is listening on the debug port right now.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listener {
    pub port: u16,
    /// The process name, when the OS's table gave one.
    pub process: Option<String>,
    pub pid: Option<u32>,
    /// The table could not be read at all — no `lsof`, no `ss`, no `netstat`.
    /// Distinct from "nothing is listening", which is an answer.
    pub unknown: bool,
}

/// Who holds the debug port.
///
/// Read from the OS's listening-socket table rather than by connecting to it.
/// A connection to a DBGp port is a debug session as far as the IDE is
/// concerned, and one that opens and closes immediately is an entry in
/// somebody's IDE log that this app had no business creating.
pub async fn listener(port: u16) -> Listener {
    match crate::doctor::listeners().await {
        None => Listener {
            port,
            process: None,
            pid: None,
            unknown: true,
        },
        Some(table) => {
            let found = table.get(&port);
            Listener {
                port,
                process: found.and_then(|l| l.process.clone()),
                pid: found.and_then(|l| l.pid),
                unknown: false,
            }
        }
    }
}

// -------------------------------------------------------------------- status

/// One IDE's state for one project.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetStatus {
    pub id: String,
    pub label: String,
    pub method: Method,
    /// Absolute, so the reader can open it — including when this refuses to.
    pub path: String,
    /// The IDE's own directory is in the project, which is the closest thing
    /// on disk to "this project is opened in that editor".
    pub detected: bool,
    pub exists: bool,
    /// The file is JSON this can edit. False means the buttons are withheld
    /// and the block to paste is offered instead.
    pub parseable: bool,
    /// Our configuration is in the file.
    pub installed: bool,
    /// And it is the one this version would write — false after the port or
    /// the mapping changed underneath it.
    pub current: bool,
    /// What to paste when this will not or cannot write it.
    pub snippet: String,
}

/// Everything the pane needs: the three values, who is listening, and each
/// IDE's state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub project: String,
    pub port: u16,
    pub ide_key: String,
    pub server_name: String,
    /// The host half of the mapping. `None` when the project directory is gone.
    pub host_path: Option<String>,
    pub container_path: String,
    pub listener: Listener,
    pub targets: Vec<TargetStatus>,
}

fn project_dir(root: &Path, project: &str) -> Result<PathBuf> {
    if !crate::workspace::is_safe_name(project) {
        return Err(Error::new(
            Code::InvalidInput,
            format!("\"{project}\" is not a valid project name"),
        ));
    }
    let dir = crate::workspace::require_projects_root(root)?.join(project);
    if !dir.is_dir() {
        return Err(Error::not_found(format!("project {project}")));
    }
    Ok(dir)
}

/// Read every target's state. Never writes.
pub async fn status(root: &Path, project: &str) -> Result<Status> {
    let dir = project_dir(root, project)?;
    let host_path = dir.display().to_string();
    let container_path = crate::xdebug::CONTAINER_PATH.to_string();
    let port = crate::xdebug::PORT;

    // The server name Xdebug will announce. The domain when there is one,
    // because that is what `PHP_IDE_CONFIG` carries and what PhpStorm matches
    // the mapping against; the project name is the fallback and is what a
    // project with no domain gets.
    let server_name = crate::manifest::read(&dir.join("stackvo.json"), project)
        .ok()
        .and_then(|m| m.domain)
        .unwrap_or_else(|| project.to_string());

    let targets = TARGETS
        .iter()
        .map(|target| {
            let path = dir.join(target.path);
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let exists = path.is_file();

            let (installed, current, parseable) = match target.method {
                Method::Written => {
                    let parseable = text.trim().is_empty() || document(&text).is_ok();
                    let (installed, current) = state(&text, project, port, &container_path);
                    (installed, current, parseable)
                }
                // Nothing is read and nothing is written, so there is no state
                // to be stale and no file to fail to parse. Saying "installed:
                // false" here would be a claim about a file this never opened.
                Method::Shown => (false, false, true),
            };

            let snippet = match target.method {
                Method::Written => {
                    serde_json::to_string_pretty(&vscode_document(project, port, &container_path))
                        .unwrap_or_default()
                }
                Method::Shown => phpstorm_snippet(&server_name, &host_path, &container_path),
            };

            TargetStatus {
                id: target.id.to_string(),
                label: target.label.to_string(),
                method: target.method,
                path: path.display().to_string(),
                detected: dir.join(target.marker).is_dir(),
                exists,
                parseable,
                installed,
                current,
                snippet,
            }
        })
        .collect();

    Ok(Status {
        project: project.to_string(),
        port,
        ide_key: crate::xdebug::IDE_KEY.to_string(),
        server_name,
        host_path: Some(host_path),
        container_path,
        listener: listener(port).await,
        targets,
    })
}

// --------------------------------------------------------------------- write

/// Read, edit and write one file, keeping a copy of what was there.
fn rewrite(path: &Path, edit: impl FnOnce(&str) -> Result<String>) -> Result<String> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let updated = edit(&existing)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    if !existing.is_empty() {
        let mut name = path.file_name().unwrap_or_default().to_os_string();
        name.push(BACKUP_SUFFIX);
        let backup = path.with_file_name(name);
        std::fs::write(&backup, &existing)
            .map_err(|e| Error::io(format!("writing {}", backup.display()), e))?;
    }

    crate::atomic::write(path, &updated)?;
    Ok(path.display().to_string())
}

/// Write the debug configuration into one IDE's file. Returns the file written.
pub fn apply(root: &Path, project: &str, id: &str) -> Result<String> {
    let Some(target) = target(id) else {
        return Err(Error::new(Code::InvalidInput, format!("unknown IDE {id}")));
    };
    if target.method != Method::Written {
        return Err(Error::new(
            Code::Unsupported,
            format!(
                "{}'s configuration is shown rather than written",
                target.label
            ),
        )
        .with_hint(crate::hints::PHPSTORM_IS_NOT_WRITTEN));
    }

    let dir = project_dir(root, project)?;
    let container_path = crate::xdebug::CONTAINER_PATH.to_string();
    let port = crate::xdebug::PORT;

    rewrite(&dir.join(target.path), |text| {
        insert(text, project, port, &container_path)
    })
}

/// Take it back out again.
pub fn remove(root: &Path, project: &str, id: &str) -> Result<String> {
    let Some(target) = target(id) else {
        return Err(Error::new(Code::InvalidInput, format!("unknown IDE {id}")));
    };
    let dir = project_dir(root, project)?;
    let path = dir.join(target.path);
    if !path.is_file() {
        return Ok(path.display().to_string());
    }

    rewrite(&path, |text| remove_from(text, project))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTAINER: &str = "/var/www/html";

    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    fn scratch() -> PathBuf {
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("stackvo-ide-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The mapping is the thing every competitor names as the common failure,
    /// so it is the thing asserted first: remote key, local value, and a local
    /// value that is not this machine's path.
    #[test]
    fn the_path_mapping_is_remote_to_local_and_portable() {
        let config = vscode_configuration("shop", 9003, CONTAINER);
        let mappings = config["pathMappings"].as_object().unwrap();

        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[CONTAINER], json!("${workspaceFolder}"));
        assert_eq!(config["port"], json!(9003));
        assert_eq!(config["type"], json!("php"));
    }

    #[test]
    fn a_new_file_is_a_whole_launch_json() {
        let text = insert("", "shop", 9003, CONTAINER).unwrap();
        let back: Value = serde_json::from_str(&text).unwrap();

        assert_eq!(back["version"], json!("0.2.0"));
        assert_eq!(back["configurations"].as_array().unwrap().len(), 1);
        assert_eq!(
            back["configurations"][0]["name"],
            json!("Listen for StackVo: shop")
        );
    }

    /// The promise this module makes about a file it does not own.
    #[test]
    fn another_configuration_in_the_file_survives() {
        let original = json!({
            "version": "0.2.0",
            "inputs": [{ "id": "which", "type": "promptString" }],
            "configurations": [
                { "name": "Launch Chrome", "type": "chrome", "request": "launch" }
            ]
        });
        let text = serde_json::to_string_pretty(&original).unwrap();

        let updated = insert(&text, "shop", 9003, CONTAINER).unwrap();
        let back: Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(
            back["inputs"], original["inputs"],
            "an unknown key was lost"
        );
        assert_eq!(back["configurations"][0]["name"], json!("Launch Chrome"));
        assert_eq!(back["configurations"].as_array().unwrap().len(), 2);

        // And removing ours gives the file back.
        let removed = remove_from(&updated, "shop").unwrap();
        let back: Value = serde_json::from_str(&removed).unwrap();
        assert_eq!(back["configurations"].as_array().unwrap().len(), 1);
        assert_eq!(back["configurations"][0]["name"], json!("Launch Chrome"));
        assert_eq!(back["inputs"], original["inputs"]);
    }

    /// Applied twice is applied once. The first version pushed unconditionally
    /// and a second press produced two identically named entries, which VS Code
    /// shows as two indistinguishable rows in one dropdown.
    #[test]
    fn applying_twice_replaces_rather_than_duplicates() {
        let once = insert("", "shop", 9003, CONTAINER).unwrap();
        let twice = insert(&once, "shop", 9003, CONTAINER).unwrap();

        let back: Value = serde_json::from_str(&twice).unwrap();
        assert_eq!(back["configurations"].as_array().unwrap().len(), 1);
    }

    /// Two projects in one workspace get two entries, because the name carries
    /// the project.
    #[test]
    fn two_projects_do_not_collide() {
        let text = insert("", "shop", 9003, CONTAINER).unwrap();
        let text = insert(&text, "api", 9003, CONTAINER).unwrap();

        let back: Value = serde_json::from_str(&text).unwrap();
        let names: Vec<&str> = back["configurations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            ["Listen for StackVo: shop", "Listen for StackVo: api"]
        );
    }

    /// Our entry is replaced where it stands, because the list is a dropdown
    /// and its order is somebody's preference.
    #[test]
    fn the_position_in_the_dropdown_is_kept() {
        let text = insert("", "shop", 9003, CONTAINER).unwrap();
        let mut document: Value = serde_json::from_str(&text).unwrap();
        document["configurations"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "name": "Launch Chrome", "type": "chrome" }));
        let text = serde_json::to_string_pretty(&document).unwrap();

        let updated = insert(&text, "shop", 9003, CONTAINER).unwrap();
        let back: Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(
            back["configurations"][0]["name"],
            json!("Listen for StackVo: shop")
        );
        assert_eq!(back["configurations"][1]["name"], json!("Launch Chrome"));
    }

    /// VS Code writes `launch.json` with comments in it, so this is the common
    /// case rather than a corner one. Reported, never rewritten — stripping
    /// them to make the edit possible deletes somebody's own notes.
    #[test]
    fn a_file_with_comments_is_refused_rather_than_rewritten() {
        let text = "{\n  // the debugger\n  \"configurations\": []\n}\n";
        let error = insert(text, "shop", 9003, CONTAINER).unwrap_err();

        assert_eq!(error.code, Code::InvalidInput);
        assert_eq!(state(text, "shop", 9003, CONTAINER), (false, false));
    }

    #[test]
    fn state_distinguishes_absent_from_stale() {
        let text = insert("", "shop", 9003, CONTAINER).unwrap();
        assert_eq!(state(&text, "shop", 9003, CONTAINER), (true, true));

        // The port moved underneath it — the case an "Update" button exists for.
        assert_eq!(state(&text, "shop", 9999, CONTAINER), (true, false));
        assert_eq!(state(&text, "other", 9003, CONTAINER), (false, false));
    }

    /// The name PhpStorm matches the mapping by, and both roots, in the one
    /// element somebody has to paste.
    #[test]
    fn the_phpstorm_snippet_carries_the_name_and_both_roots() {
        let xml = phpstorm_snippet("shop.loc", "/w/projects/shop", CONTAINER);

        assert!(xml.contains("name=\"shop.loc\""), "{xml}");
        assert!(xml.contains("local-root=\"/w/projects/shop\""), "{xml}");
        assert!(xml.contains("remote-root=\"/var/www/html\""), "{xml}");
        assert!(xml.contains("use_path_mappings=\"true\""), "{xml}");
    }

    #[test]
    fn a_written_target_round_trips_on_disk() {
        let root = scratch();
        let projects = root.join("projects");
        std::fs::create_dir_all(projects.join("shop")).unwrap();
        crate::workspace::point_at_projects(&root, &projects).unwrap();

        let written = apply(&root, "shop", "vscode").unwrap();
        assert!(written.ends_with("launch.json"));

        let text = std::fs::read_to_string(projects.join("shop/.vscode/launch.json")).unwrap();
        assert_eq!(state(&text, "shop", 9003, CONTAINER), (true, true));

        remove(&root, "shop", "vscode").unwrap();
        let text = std::fs::read_to_string(projects.join("shop/.vscode/launch.json")).unwrap();
        assert_eq!(state(&text, "shop", 9003, CONTAINER), (false, false));

        let _ = std::fs::remove_dir_all(&root);
    }

    /// PhpStorm is offered as a block to paste and refuses to be written, and
    /// the refusal has to name the reason rather than read as a failure.
    #[test]
    fn phpstorm_refuses_to_be_written() {
        let root = scratch();
        let projects = root.join("projects");
        std::fs::create_dir_all(projects.join("shop")).unwrap();
        crate::workspace::point_at_projects(&root, &projects).unwrap();

        let error = apply(&root, "shop", "phpstorm").unwrap_err();
        assert_eq!(error.code, Code::Unsupported);
        assert!(!projects.join("shop/.idea").exists(), "it created .idea");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_unsafe_project_name_never_reaches_the_filesystem() {
        let root = scratch();
        assert!(apply(&root, "../../etc", "vscode").is_err());
        assert!(apply(&root, "no-such-project", "vscode").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The port is Xdebug 3's, and it is read from one place. Xdebug 2 used
    /// 9000, which collides with PHP-FPM, and a configuration written for the
    /// wrong one is the single most common reason a listener never fires.
    #[test]
    fn the_port_is_the_one_the_overlay_dials() {
        assert_eq!(crate::xdebug::PORT, 9003);
        let config = vscode_configuration("shop", crate::xdebug::PORT, CONTAINER);
        assert_eq!(config["port"], json!(9003));
    }

    #[tokio::test]
    async fn the_listener_is_reported_rather_than_dialled() {
        // 9003 is almost certainly free on a test machine, and the assertion
        // that matters is the shape: an answer, not a connection, and "nothing
        // is listening" distinguished from "the table could not be read".
        let found = listener(9003).await;
        assert_eq!(found.port, 9003);
        if !found.unknown {
            assert!(found.process.is_none() || found.pid.is_some());
        }
    }
}
