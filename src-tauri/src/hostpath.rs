//! The `PATH` a Dock launch does not have.
//!
//! An app opened from Finder, the Dock or Spotlight inherits launchd's
//! environment, and launchd's `PATH` is `/usr/bin:/bin:/usr/sbin:/sbin`. A
//! terminal's `PATH` is that plus whatever `.zshrc` and `/etc/paths.d` add —
//! and that is where every tool this app shells out to lives. Docker Desktop
//! links `docker` into `/usr/local/bin`; Homebrew puts Colima, Podman and
//! mkcert under `/opt/homebrew/bin`; OrbStack keeps its CLI in
//! `~/.orbstack/bin`. None of those are on the Dock's `PATH`.
//!
//! The symptom in v0.3.2 was the requirements gate reporting "Docker Compose
//! v2" missing on a machine with Compose v5 installed, under an engine row
//! that was green — the engine is reached over its socket, the compose check
//! spawns `docker`, and only one of those needs `PATH`. Every `cargo tauri
//! dev` run inherits a terminal's `PATH`, which is why it never showed up
//! before the `.dmg`.
//!
//! ## Why not run the login shell
//!
//! The usual Tauri answer spawns `$SHELL -ilc 'echo $PATH'` and takes the
//! result. It gives the exact `PATH` the user sees — and it runs their startup
//! files, which may print a banner, start an agent, prompt for a passphrase or
//! block on a slow `nvm` initialisation, with the app's first window waiting
//! behind it. A fixed list of the places tools are actually installed costs
//! nothing, cannot hang, and covers every Docker distribution this app knows
//! how to detect.
//!
//! ## Where the entries go
//!
//! At the **end**. A launch from a terminal already has these directories in
//! the user's own order, and this must not change which `docker` wins there.
//! Only a directory that exists and is not already listed is added, so the
//! terminal case is a no-op. The managed [`crate::tooling::bin_dir`] is in the
//! list too: it is where `tooling` puts the tools it installs, and the app
//! should find them without waiting for the user to restart a shell.
//!
//! Windows is left alone: a GUI process there gets the user's and the
//! machine's `PATH` from the registry, which is the one the installer edited.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Every directory a tool this app runs is known to be installed into.
///
/// Absolute ones first, then the per-user ones under `home`, then the app's
/// own. Order only matters for an entry not already on `PATH`, and among those
/// the system-wide link is the one Docker Desktop itself maintains.
pub fn candidates(home: Option<&Path>, managed: Option<&Path>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = [
        // Docker Desktop's CLI symlinks, MacPorts, and the historical default.
        "/usr/local/bin",
        // Homebrew on Apple silicon: colima, podman, mkcert, git.
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        // MacPorts.
        "/opt/local/bin",
        // Docker Desktop's own copies, for an install whose symlinks were
        // declined or removed.
        "/Applications/Docker.app/Contents/Resources/bin",
        // Snap on Linux.
        "/snap/bin",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();

    if let Some(home) = home {
        for rel in [
            // Docker Desktop's per-user link directory (newer installs).
            ".docker/bin",
            // OrbStack.
            ".orbstack/bin",
            // Rancher Desktop.
            ".rd/bin",
            // pipx, cargo-install style tools, and distro conventions.
            ".local/bin",
            "bin",
        ] {
            out.push(home.join(rel));
        }
    }

    if let Some(managed) = managed {
        out.push(managed.to_path_buf());
    }

    out
}

/// `current` with every entry of `extra` it does not already hold appended, or
/// `None` when there is nothing to add.
///
/// Pure, so it can be tested without touching the process environment: a test
/// that set `PATH` would change what every other test in the binary spawns.
pub fn augmented(current: Option<&OsStr>, extra: &[PathBuf]) -> Option<OsString> {
    let mut entries: Vec<PathBuf> = current
        .map(|p| std::env::split_paths(p).collect())
        .unwrap_or_default();
    let before = entries.len();

    for dir in extra {
        if !entries.iter().any(|e| e == dir) {
            entries.push(dir.clone());
        }
    }

    if entries.len() == before {
        return None;
    }
    std::env::join_paths(entries).ok()
}

/// Put the known tool directories on this process's `PATH`.
///
/// Call it **first** in `run`, before the logger or anything else starts a
/// thread: `setenv` while another thread reads the environment is a data race
/// on every unix libc, and the writer here cannot see who else is reading. The
/// directories added are returned so the caller can log them once a logger
/// exists.
pub fn install() -> Vec<PathBuf> {
    if cfg!(windows) {
        return Vec::new();
    }

    let present: Vec<PathBuf> = candidates(
        dirs::home_dir().as_deref(),
        crate::tooling::bin_dir().as_deref(),
    )
    .into_iter()
    .filter(|dir| dir.is_dir())
    .collect();

    let current = std::env::var_os("PATH");
    let Some(path) = augmented(current.as_deref(), &present) else {
        return Vec::new();
    };

    let had: Vec<PathBuf> = current
        .as_deref()
        .map(|p| std::env::split_paths(p).collect())
        .unwrap_or_default();
    let added: Vec<PathBuf> = present
        .into_iter()
        .filter(|dir| !had.contains(dir))
        .collect();

    std::env::set_var("PATH", path);
    added
}

// Unix only, like the behaviour: on Windows `install` returns before touching
// anything, and these assertions spell paths the way unix does — `:` between
// entries and `/usr/local/bin` as an absolute path, which Windows disagrees with
// on both counts.
#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn a_dock_launch_gains_the_tool_directories_at_the_end() {
        let out = augmented(
            Some(OsStr::new("/usr/bin:/bin:/usr/sbin:/sbin")),
            &[p("/usr/local/bin"), p("/opt/homebrew/bin")],
        )
        .unwrap();
        assert_eq!(
            out,
            OsString::from("/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/bin:/opt/homebrew/bin")
        );
    }

    #[test]
    fn a_terminal_launch_is_left_exactly_as_it_was() {
        // The user's order has /opt/homebrew/bin ahead of /usr/local/bin; a
        // Dock-style append would have put a second copy behind it.
        let out = augmented(
            Some(OsStr::new("/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin")),
            &[p("/usr/local/bin"), p("/opt/homebrew/bin")],
        );
        assert_eq!(out, None);
    }

    #[test]
    fn only_the_missing_entries_are_added_once_each() {
        let out = augmented(
            Some(OsStr::new("/usr/bin:/usr/local/bin")),
            &[
                p("/usr/local/bin"),
                p("/opt/homebrew/bin"),
                p("/opt/homebrew/bin"),
            ],
        )
        .unwrap();
        assert_eq!(
            out,
            OsString::from("/usr/bin:/usr/local/bin:/opt/homebrew/bin")
        );
    }

    #[test]
    fn no_path_at_all_becomes_the_tool_directories() {
        let out = augmented(None, &[p("/usr/local/bin")]).unwrap();
        assert_eq!(out, OsString::from("/usr/local/bin"));
        assert_eq!(augmented(None, &[]), None);
    }

    #[test]
    fn candidates_cover_every_docker_distribution_the_app_detects() {
        let home = p("/Users/someone");
        let managed = p("/Users/someone/Library/Application Support/StackVo/bin");
        let dirs = candidates(Some(&home), Some(&managed));

        // Docker Desktop, both link locations.
        assert!(dirs.contains(&p("/usr/local/bin")));
        assert!(dirs.contains(&p("/Users/someone/.docker/bin")));
        // Homebrew (Colima, Podman, mkcert).
        assert!(dirs.contains(&p("/opt/homebrew/bin")));
        // OrbStack and Rancher Desktop.
        assert!(dirs.contains(&p("/Users/someone/.orbstack/bin")));
        assert!(dirs.contains(&p("/Users/someone/.rd/bin")));
        // The app's own.
        assert_eq!(dirs.last(), Some(&managed));

        // Without a home or a managed directory the system-wide ones remain.
        let bare = candidates(None, None);
        assert!(bare.contains(&p("/usr/local/bin")));
        assert!(bare.iter().all(|d| d.is_absolute()));
    }
}
