//! Watching the project directory: manifest edits, and folders arriving.
//!
//! StackVo's config flow is manual: edit a manifest, remember to run
//! `stackvo generate`, remember to restart. Forgetting either leaves the
//! containers running against a stale Dockerfile with nothing to indicate it.
//!
//! A host process can just watch the files. This module does not regenerate
//! anything on its own — it emits `manifest:changed` and lets the UI offer the
//! action, because silently rebuilding a container underneath someone who is
//! mid-edit is worse than the problem it solves.
//!
//! ## The second question, added later
//!
//! The project directory **is** what Herd and Valet call a park: point the
//! workspace at it and every child of it is a candidate site. `adoptable`
//! already reads it and says what each folder is — but only when somebody
//! opens the dialog and asks. Clone a repository into the parked folder and
//! the running app said nothing at all, so the answer to "why is my new site
//! not there" was "reopen this list", which is the part a park is supposed to
//! remove.
//!
//! The watcher was already receiving those events and throwing them away:
//! [`project_for`] accepts `<projects>/<name>/stackvo.json` and nothing else,
//! which is right for its own question and drops every `git clone`. So there
//! is a second reader on the same stream, emitting `folder:appeared`. It still
//! decides nothing — the UI refetches the list, which is the only thing that
//! can tell an adoptable folder from one that arrived with its manifest.

use crate::events;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;

/// Holds the live watcher so it can be replaced.
///
/// Dropping a `notify` watcher stops the watch, so the handle has to be kept —
/// and kept somewhere replaceable, or changing the workspace would leave the
/// app watching a directory the user has moved away from. That was the case
/// until this existed: the new workspace only got a watcher on the next launch.
#[derive(Default)]
pub struct Handle(Mutex<Option<notify::RecommendedWatcher>>);

impl Handle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Point the watcher at `root`, replacing whatever it was watching.
    ///
    /// The old watcher is dropped first: two watchers on overlapping trees
    /// would emit every change twice, and the debounce is per-watcher.
    pub fn retarget(&self, app: &AppHandle, root: Option<PathBuf>) {
        let Ok(mut slot) = self.0.lock() else { return };
        *slot = None;

        if let Some(root) = root {
            *slot = start(app.clone(), root).ok();
        }
    }
}

/// Editors write a file several times in a burst (temp file, rename, chmod).
/// Anything inside this window after the first event is the same logical edit.
const DEBOUNCE: Duration = Duration::from_millis(400);

/// Which project a changed path belongs to, if any.
///
/// Accepts only `<projects>/<name>/stackvo.json` — deliberately narrow, so an
/// editor's `stackvo.json~` backup or a file deeper in the tree does not
/// trigger a regenerate prompt.
///
/// Takes the project directory rather than the app root and resolving it here:
/// this is a question about two paths, and asking it that way is what lets it
/// be tested against paths that never exist on disk.
fn project_for(projects: &Path, changed: &Path) -> Option<String> {
    if changed.file_name()? != "stackvo.json" {
        return None;
    }

    let relative = changed.strip_prefix(projects).ok()?;
    let mut parts = relative.components();
    let name = parts.next()?.as_os_str().to_str()?.to_string();

    // Exactly one directory level between projects/ and the file.
    (parts.next().is_some() && parts.next().is_none()).then_some(name)
}

/// The top-level folder names the projects directory holds right now.
///
/// Read once, when the watch starts, so the folders that were already there do
/// not all announce themselves as new the first time anything is written
/// inside one of them.
fn folders_in(projects: &Path) -> HashSet<String> {
    let Ok(entries) = std::fs::read_dir(projects) else {
        return HashSet::new();
    };
    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .filter(|name| !name.starts_with('.'))
        .collect()
}

/// The project a changed path is inside, and the path within it.
///
/// The third reader on this stream, and the shape is a third one again:
/// [`project_for`] wants one exact file, [`folder_for`] wants the first
/// component of anything at all, and this wants **both halves** — because the
/// question it serves is "did the application change", which needs the project
/// to act on and the relative path to decide with.
fn project_and_relative(projects: &Path, changed: &Path) -> Option<(String, PathBuf)> {
    let relative = changed.strip_prefix(projects).ok()?;
    let mut parts = relative.components();
    let name = parts.next()?.as_os_str().to_str()?.to_string();
    if name.starts_with('.') {
        return None;
    }
    let inner: PathBuf = parts.collect();
    (!inner.as_os_str().is_empty()).then_some((name, inner))
}

/// Which top-level folder under `projects/` a changed path is inside.
///
/// Deliberately the opposite shape to [`project_for`]: that one wants one exact
/// file and this one wants the first component of anything at all, because a
/// `git clone` announces itself as several hundred paths deep inside a
/// directory that did not exist a second ago and never as the directory
/// itself.
///
/// Dot-prefixed names are not folders anybody parked. On a real machine the
/// projects directory grows `.DS_Store` and, if somebody made the whole tree a
/// repository, `.git` — which during a fetch produces more events than every
/// real project combined.
fn folder_for(projects: &Path, changed: &Path) -> Option<String> {
    let relative = changed.strip_prefix(projects).ok()?;
    let name = relative.components().next()?.as_os_str().to_str()?;
    (!name.starts_with('.')).then(|| name.to_string())
}

/// Start watching. The returned watcher must be kept alive — dropping it stops
/// the watch, which is why it is parked in Tauri's managed state.
pub fn start(app: AppHandle, root: PathBuf) -> notify::Result<notify::RecommendedWatcher> {
    // Nothing to watch before a project tree is chosen. `NotFound` rather than
    // a silent no-op: `retarget` stores the result, and a watcher that quietly
    // watches nothing has exactly one symptom — edits stop prompting a
    // regenerate, months later, with nothing to point at.
    let Some(projects_dir) = crate::workspace::projects_root(&root) else {
        return Err(notify::Error::io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no project directory has been chosen",
        )));
    };
    let (tx, rx) = mpsc::channel::<Event>();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    watcher.watch(&projects_dir, RecursiveMode::Recursive)?;

    let watched = projects_dir.clone();
    // What was already there when the watch started. A folder is announced
    // when it becomes new to this set and not on a timer: a clone writes for as
    // long as the repository takes and every second of it is a create event,
    // so a time window would either announce the same folder a dozen times or
    // be long enough to miss the next one.
    let mut known = folders_in(&projects_dir);

    std::thread::spawn(move || {
        let mut last: Vec<(String, Instant)> = Vec::new();
        // The Octane reload's own debounce, kept apart from the one above. That
        // one is answering "did the editor write this file three times"; this
        // one is answering "has the developer stopped changing things", and a
        // `composer install` makes the difference four thousand events wide.
        let mut reloaded: Vec<(String, Instant)> = Vec::new();

        for event in rx {
            // A folder that goes away is forgotten, so cloning over the same
            // name again announces it again. Without this the second clone is
            // silent for the rest of the session, which is the one case where
            // "it appeared" is most obviously true.
            if matches!(event.kind, EventKind::Remove(_)) {
                for path in &event.paths {
                    if let Some(folder) = folder_for(&watched, path) {
                        if !watched.join(&folder).exists() {
                            known.remove(&folder);
                        }
                    }
                }
                continue;
            }

            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                continue;
            }

            for path in &event.paths {
                if let Some(folder) = folder_for(&watched, path) {
                    // Directories only. A loose file dropped beside the
                    // projects is not a site, and `adoptable` would not offer
                    // it either.
                    if watched.join(&folder).is_dir() && known.insert(folder.clone()) {
                        events::emit(
                            &app,
                            "folder:appeared",
                            serde_json::json!({
                                "folder": folder,
                                "path": watched.join(&folder).display().to_string(),
                            }),
                        );
                    }
                }

                // Octane holds the application in memory, so an edited file
                // changes nothing until the workers are replaced. Done here
                // rather than in the window, because a reload that only
                // happened while a pane was open would be a feature that works
                // when somebody is watching it.
                if let Some((project, relative)) = project_and_relative(&watched, path) {
                    if crate::octane::is_application_change(&relative) {
                        let now = Instant::now();
                        reloaded
                            .retain(|(_, at)| now.duration_since(*at) < crate::octane::DEBOUNCE);
                        if !reloaded.iter().any(|(p, _)| p == &project) {
                            reloaded.push((project.clone(), now));
                            crate::octane::reload_if_enabled(&app, project);
                        }
                    }
                }

                let Some(project) = project_for(&watched, path) else {
                    continue;
                };

                let now = Instant::now();
                last.retain(|(_, at)| now.duration_since(*at) < DEBOUNCE);
                if last.iter().any(|(p, _)| p == &project) {
                    continue;
                }
                last.push((project.clone(), now));

                events::emit(
                    &app,
                    "manifest:changed",
                    serde_json::json!({
                        "project": project,
                        "path": path.display().to_string(),
                    }),
                );
            }
        }
    });

    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECTS: &str = "/w/stackvo/projects";

    #[test]
    fn a_clone_is_recognised_from_any_path_inside_it() {
        // What `git clone` actually produces. None of these is the directory
        // itself, and the folder has to be named from every one of them.
        for deep in [
            "/w/stackvo/projects/shop/.git/objects/pack/tmp_pack_a1",
            "/w/stackvo/projects/shop/composer.json",
            "/w/stackvo/projects/shop/vendor/laravel/framework/README.md",
        ] {
            assert_eq!(
                folder_for(Path::new(PROJECTS), Path::new(deep)),
                Some("shop".to_string()),
                "{deep}"
            );
        }
    }

    #[test]
    fn dotfiles_and_outsiders_are_not_folders() {
        for path in [
            // `.DS_Store` and a repository over the whole tree: the second one
            // writes more paths during one fetch than every project combined.
            "/w/stackvo/projects/.DS_Store",
            "/w/stackvo/projects/.git/index",
            "/elsewhere/shop/composer.json",
        ] {
            assert_eq!(
                folder_for(Path::new(PROJECTS), Path::new(path)),
                None,
                "{path}"
            );
        }
    }

    #[test]
    fn the_projects_directory_itself_is_not_a_folder_in_it() {
        assert_eq!(folder_for(Path::new(PROJECTS), Path::new(PROJECTS)), None);
    }

    #[test]
    fn what_is_already_there_is_known_before_the_first_event() {
        // The whole point of seeding: without it, the first write inside any
        // existing project announces that project as new.
        let dir = std::env::temp_dir().join(format!("stackvo-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("shop")).unwrap();
        std::fs::create_dir_all(dir.join(".hidden")).unwrap();
        std::fs::write(dir.join("loose.txt"), "not a project").unwrap();

        let known = folders_in(&dir);
        assert!(known.contains("shop"));
        assert!(
            !known.contains(".hidden"),
            "dotfiles are not parked folders"
        );
        assert!(!known.contains("loose.txt"), "a file is not a folder");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The assumption everything above rests on, driven against the real thing.
    ///
    /// `folder_for` names a folder from a path *inside* it, which is only
    /// useful if the platform's watcher reports those paths at all. On macOS
    /// that is FSEvents, and a directory created after the watch began is
    /// exactly the case where a coalescing backend could report the parent and
    /// nothing else — in which case the whole feature would be silent on the
    /// one platform this is developed on, and every test above would still
    /// pass. So this creates a directory the watcher has never seen, writes a
    /// file inside it, and asserts a path under that name arrives.
    #[test]
    fn the_platform_reports_paths_inside_a_directory_it_has_never_seen() {
        use notify::Watcher as _;

        let dir = std::env::temp_dir().join(format!("stackvo-appear-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // macOS hands out `/var/...` and reports `/private/var/...`; comparing
        // the two would fail on the strip_prefix and not on the question.
        let dir = std::fs::canonicalize(&dir).unwrap();

        let (tx, rx) = mpsc::channel::<Event>();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .unwrap();
        watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

        std::fs::create_dir(dir.join("shop")).unwrap();
        std::fs::write(dir.join("shop/composer.json"), "{}").unwrap();

        // Generous, and it returns the moment the answer arrives. A backend
        // with a delivery latency is still a backend that delivers.
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut seen = false;
        while !seen && Instant::now() < deadline {
            let left = deadline.saturating_duration_since(Instant::now());
            let Ok(event) = rx.recv_timeout(left) else {
                break;
            };
            seen = event
                .paths
                .iter()
                .any(|path| folder_for(&dir, path).as_deref() == Some("shop"));
        }

        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            seen,
            "no event named the new folder; `folder:appeared` would never fire on this platform"
        );
    }

    #[test]
    fn a_missing_directory_is_no_folders_rather_than_a_panic() {
        // The workspace can be pointed at a path that is not there yet.
        assert!(folders_in(Path::new("/nowhere/at/all")).is_empty());
    }

    #[test]
    fn matches_a_project_manifest() {
        assert_eq!(
            project_for(
                Path::new(PROJECTS),
                Path::new("/w/stackvo/projects/shop/stackvo.json")
            ),
            Some("shop".to_string())
        );
    }

    #[test]
    fn ignores_editor_backups_and_other_files() {
        for path in [
            "/w/stackvo/projects/shop/stackvo.json~",
            "/w/stackvo/projects/shop/composer.json",
            "/w/stackvo/projects/shop/.stackvo/Dockerfile",
        ] {
            assert_eq!(
                project_for(Path::new(PROJECTS), Path::new(path)),
                None,
                "{path}"
            );
        }
    }

    #[test]
    fn ignores_manifests_outside_the_one_level_layout() {
        // Too shallow, and too deep — neither is a project manifest.
        assert_eq!(
            project_for(
                Path::new(PROJECTS),
                Path::new("/w/stackvo/projects/stackvo.json")
            ),
            None
        );
        assert_eq!(
            project_for(
                Path::new(PROJECTS),
                Path::new("/w/stackvo/projects/a/b/stackvo.json")
            ),
            None
        );
    }

    #[test]
    fn ignores_paths_outside_the_projects_directory() {
        assert_eq!(
            project_for(
                Path::new(PROJECTS),
                Path::new("/w/stackvo/core/stackvo.json")
            ),
            None
        );
    }
}
