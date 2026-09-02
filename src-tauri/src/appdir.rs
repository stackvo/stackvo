//! The folders this app owns inside the OS's own directories.
//!
//! Two of them — where preferences are written, and where the log is written —
//! and until now each was named where it was used, with the bundle identifier
//! spelled out by hand in both. That is how two directories that are supposed
//! to carry the same name stop carrying the same name.
//!
//! ## Why not the bundle identifier
//!
//! The identifier is what the *operating system* calls this app: the
//! `Preferences` plist, the code signature, the privacy prompts. macOS gives no
//! choice there. These two folders are different — they are ours to name, and
//! they are the ones a person is asked to open when something goes wrong.
//!
//! Three comparable apps on the machine this was written on, checked rather
//! than recalled:
//!
//! | app           | identifier                      | Application Support | Logs           |
//! | ------------- | ------------------------------- | ------------------- | -------------- |
//! | Postman       | `com.postmanlabs.mac`           | `Postman`           | —              |
//! | Termius       | `com.termius-dmg.mac`           | `Termius`           | —              |
//! | Redis Insight | `org.RedisLabs.RedisInsight-V2` | `RedisInsight`      | `RedisInsight` |
//!
//! All three put the identifier in the plist, which they must, and the readable
//! name in the folders they chose, which they need not have. None does the
//! reverse.
//!
//! ## Why not `~/.stackvo`
//!
//! That directory is the *stack's* state: the user picks what it points at,
//! `STACKVO_ROOT` can move it, and deleting it is a supported way to start
//! over. A log that moves with the root fragments across roots and disappears
//! along with the workspace whose failure it was recording — and the preference
//! for which editor to open is not a fact about a stack.
//!
//! Redis Insight, the closest analogue here — a desktop shell in front of a
//! service it manages — splits it the same way, and both halves are in use:
//! `~/Library/Logs/RedisInsight/main.log` for the app, `~/.redis-insight/logs/`
//! for the backend it drives.

use std::path::PathBuf;

/// The name this app takes in the OS's directories.
///
/// Title case where the platform's own folders are title case, lower case on
/// Linux where every XDG directory is.
pub fn name() -> &'static str {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        "StackVo"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "stackvo"
    }
}

/// The variable that moves the config directory somewhere a test can delete.
///
/// The same seam `STACKVO_ROOT` is for the app root and `STACKVO_POLICY_FILE`
/// is for the policy file, and it was the one of the three that did not exist:
/// `tests/driver/launch.js` set it, said in its own comment that it moved the
/// settings, and the driven application read `~/.config/stackvo` anyway. A
/// suite that believes it is isolated and is not is worse than one that never
/// claimed to be — it writes the developer's real preferences, and it reads
/// whatever they last chose.
pub const CONFIG_OVERRIDE_VAR: &str = "STACKVO_CONFIG_DIR";

/// Where `preferences.json` lives.
///
/// macOS: `~/Library/Application Support/StackVo`, Windows: `%APPDATA%\StackVo`,
/// Linux: `~/.config/stackvo`. [`CONFIG_OVERRIDE_VAR`] wins over all three.
///
/// The override is taken verbatim rather than canonicalised, unlike
/// `workspace::app_root`: that path reaches generated compose files, where
/// Docker resolves it against its own working directory, and this one is only
/// ever opened by this process.
pub fn config() -> Option<PathBuf> {
    if let Ok(from_env) = std::env::var(CONFIG_OVERRIDE_VAR) {
        if !from_env.trim().is_empty() {
            return Some(PathBuf::from(from_env));
        }
    }
    Some(dirs::config_dir()?.join(name()))
}

/// Where the rotating log files live.
///
/// Deliberately not `config()`: those are things the user chose and would want
/// to keep, and a log is neither. On macOS that distinction needs the
/// platform's own convention, because `config_dir()` and `data_local_dir()` are
/// the same folder there (`~/Library/Application Support`) — measured, not
/// assumed. Apple puts logs in `~/Library/Logs`, which is also where Console.app
/// looks for them.
pub fn logs() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(dirs::home_dir()?.join("Library").join("Logs").join(name()))
    }
    #[cfg(target_os = "windows")]
    {
        Some(dirs::data_local_dir()?.join(name()).join("logs"))
    }
    // Linux and the BSDs. `XDG_STATE_HOME` (`~/.local/state`) rather than
    // `XDG_DATA_HOME`, which is where this used to write: the specification adds
    // state for data that should persist but is not worth backing up, and names
    // log files as the example. `state_dir` answers None off Linux, so the old
    // location stays as the fallback rather than losing the log entirely.
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Some(
            dirs::state_dir()
                .or_else(dirs::data_local_dir)?
                .join(name())
                .join("logs"),
        )
    }
}

/// Where the commands this app puts on `PATH` live.
///
/// Under the OS data directory rather than under `~/.stackvo`, for the reason
/// the module comment gives about the log: `~/.stackvo` is the *stack's* state,
/// the user chooses where it points and deleting it is a supported way to start
/// over. A `PATH` entry that vanishes when somebody resets their stack is a
/// `PATH` entry pointing at nothing — and `agents.rs` will by then have written
/// the linked path into six assistants' configuration files.
///
/// macOS: `~/Library/Application Support/StackVo/bin`, Windows:
/// `%APPDATA%\StackVo\bin`, Linux: `~/.local/share/stackvo/bin`.
pub fn bin() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join(name()).join("bin"))
}

/// The folder name every one of these used before the rename.
///
/// Read-only, and only by the migration below and by `workspace`'s reader for a
/// pre-split `workspace.txt`. It is a historical fact about installs that
/// already exist, so it must never be re-derived from the current identifier —
/// changing the identifier again would silently stop finding what the old one
/// left behind.
pub const LEGACY_DIR: &str = "dev.stackvo.desktop";

/// Bring `preferences.json` across from the folder the old identifier named.
///
/// Only the one file. `workspace.txt` is deliberately left where it is: it
/// belongs to a layout two migrations ago and `workspace::migrate_single_root`
/// is still the thing that reads and retires it.
///
/// Best effort, and silent about a failure beyond the log: the worst outcome is
/// a user whose theme is back to default, which is not a reason to refuse to
/// start. Never overwrites — a preferences file at the new path means this
/// already ran, or the user wrote one, and either way it wins.
pub fn migrate_config() {
    // Not when the directory was overridden. The old location is a fact about
    // this machine's real config directory, and moving a file OUT of it into a
    // temporary directory a test deletes afterwards would lose the preferences
    // of whoever ran the test.
    if std::env::var_os(CONFIG_OVERRIDE_VAR).is_some_and(|v| !v.is_empty()) {
        return;
    }

    let (Some(new_dir), Some(old_dir)) = (config(), dirs::config_dir().map(|d| d.join(LEGACY_DIR)))
    else {
        return;
    };
    move_preferences(&new_dir, &old_dir);
}

/// The move itself, with both directories passed in rather than read.
///
/// Split out for the same reason `workspace::migrate` is: reading them from the
/// OS config directory inside this function would make it testable only by
/// writing to the real one.
fn move_preferences(new_dir: &std::path::Path, old_dir: &std::path::Path) {
    if new_dir == old_dir {
        return;
    }

    let (new, old) = (
        new_dir.join("preferences.json"),
        old_dir.join("preferences.json"),
    );
    if new.exists() || !old.is_file() {
        return;
    }

    if std::fs::create_dir_all(new_dir).is_err() {
        return;
    }

    // Rename first: it is atomic within a filesystem, and both paths are under
    // the same config directory. The copy is for the case where they are not —
    // a home directory assembled out of mounts is unusual, not impossible.
    let moved = std::fs::rename(&old, &new).is_ok()
        || (std::fs::copy(&old, &new).is_ok() && {
            let _ = std::fs::remove_file(&old);
            true
        });

    if moved {
        tracing::info!(from = %old.display(), to = %new.display(), "moved preferences to the renamed config directory");
    } else {
        tracing::warn!(from = %old.display(), to = %new.display(), "could not move preferences; starting with defaults");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `CONFIG_OVERRIDE_VAR` is process-wide, and `cargo test` runs these in
    /// threads. Every test below either sets it or reads a `config()` the
    /// setting would change, so they take turns — the alternative is a suite
    /// that passes alone and fails in company, which is the kind of failure
    /// that gets re-run rather than read.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Take the lock, ignoring a previous test having panicked while holding it.
    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Set the override for the duration of `body`, and put it back afterwards.
    fn with_override<T>(value: Option<&str>, body: impl FnOnce() -> T) -> T {
        let previous = std::env::var_os(CONFIG_OVERRIDE_VAR);
        // SAFETY: the whole module's tests are serialised on `SERIAL`, and the
        // previous value is restored before this returns.
        unsafe {
            match value {
                Some(v) => std::env::set_var(CONFIG_OVERRIDE_VAR, v),
                None => std::env::remove_var(CONFIG_OVERRIDE_VAR),
            }
        }
        let out = body();
        unsafe {
            match previous {
                Some(v) => std::env::set_var(CONFIG_OVERRIDE_VAR, v),
                None => std::env::remove_var(CONFIG_OVERRIDE_VAR),
            }
        }
        out
    }

    /// The seam `tests/driver/launch.js` was already using before it existed.
    ///
    /// It set `STACKVO_CONFIG_DIR`, documented that the settings moved with it,
    /// and nothing read the variable — so the driven application wrote the
    /// preferences of whoever was running CI's image and read back whatever was
    /// there. The suite could not decide what `closeBehaviour` was, which is the
    /// preference that decides whether the window can be closed at all.
    #[test]
    fn the_config_directory_can_be_moved_by_the_environment() {
        let _guard = serial();
        let moved = with_override(Some("/tmp/stackvo-config-test"), config).expect("a directory");
        assert_eq!(moved, PathBuf::from("/tmp/stackvo-config-test"));
    }

    /// An empty value is not a location.
    ///
    /// `env -u` is not what a shell script reaches for; `VAR=` is, and a run
    /// that meant "leave it alone" must not end up writing preferences to the
    /// process's working directory.
    #[test]
    fn an_empty_override_is_ignored() {
        let _guard = serial();
        let (empty, blank, absent) = (
            with_override(Some(""), config),
            with_override(Some("   "), config),
            with_override(None, config),
        );
        assert_eq!(empty, absent);
        assert_eq!(blank, absent);
    }

    /// The name is ours to choose, so the one thing worth asserting is that it
    /// is not the identifier — which is what this rename existed to undo.
    #[test]
    fn the_folder_name_is_not_the_bundle_identifier() {
        assert!(!name().contains('.'), "{} looks like an identifier", name());
        assert_ne!(name(), LEGACY_DIR);
    }

    /// Both folders carry the same name, which is the drift this module exists
    /// to prevent.
    #[test]
    fn config_and_logs_agree_on_the_name() {
        let _guard = serial();
        // Explicitly without the override: this asserts the *derived* name, and
        // a developer who happens to have `STACKVO_CONFIG_DIR` set in their
        // shell would otherwise watch it fail for a reason that is not a fault.
        with_override(None, || {
            let config = config().expect("a config directory");
            let logs = logs().expect("a log directory");
            assert!(config.ends_with(name()));
            assert!(
                logs.components().any(|c| c.as_os_str() == name()),
                "{} does not carry {}",
                logs.display(),
                name()
            );
        });
    }

    /// Logs sitting next to `preferences.json` would be swept up by anyone
    /// clearing "settings" and kept by anyone backing them up. On macOS this
    /// only holds because of the `~/Library/Logs` branch — `config_dir()` and
    /// `data_local_dir()` are the same directory there.
    #[test]
    fn the_log_directory_is_not_the_config_directory() {
        let logs = logs().expect("a log directory");
        let config = dirs::config_dir().expect("a config directory");
        assert!(
            !logs.starts_with(&config),
            "logs at {} sit inside the config directory {}",
            logs.display(),
            config.display()
        );
    }

    /// The other half of the question this module keeps being asked: both
    /// folders belong to the application, not to the stack, so neither may land
    /// under the app root — which `STACKVO_ROOT` can move and the user can
    /// delete.
    #[test]
    fn neither_directory_is_inside_the_app_root() {
        let _guard = serial();
        with_override(None, || {
            let app_root = crate::workspace::app_root();
            for dir in [config().unwrap(), logs().unwrap()] {
                assert!(
                    !dir.starts_with(&app_root),
                    "{} sits inside the app root {}",
                    dir.display(),
                    app_root.display()
                );
            }
        });
    }

    /// XDG puts logs in `XDG_STATE_HOME`, and this wrote to `XDG_DATA_HOME`
    /// until it was noticed. The fallback branch is what keeps the assertion
    /// honest on a machine where `state_dir` answers None.
    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn linux_logs_go_under_the_state_directory() {
        let logs = logs().expect("a log directory");
        match dirs::state_dir() {
            Some(state) => assert!(
                logs.starts_with(&state),
                "logs at {} are not under {}",
                logs.display(),
                state.display()
            ),
            None => assert!(logs.starts_with(dirs::data_local_dir().unwrap())),
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "stackvo-appdir-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn preferences_are_carried_over_and_the_old_copy_is_gone() {
        let base = scratch("carry");
        let (old, new) = (base.join(LEGACY_DIR), base.join(name()));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("preferences.json"), r#"{"theme":"dark"}"#).unwrap();

        move_preferences(&new, &old);

        assert_eq!(
            std::fs::read_to_string(new.join("preferences.json")).unwrap(),
            r#"{"theme":"dark"}"#
        );
        assert!(
            !old.join("preferences.json").exists(),
            "leaving it behind means the next launch has two files to disagree about"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A preferences file already at the new path is the user's, and a
    /// migration that overwrites it is a migration that loses settings.
    #[test]
    fn an_existing_file_at_the_new_path_wins() {
        let base = scratch("nooverwrite");
        let (old, new) = (base.join(LEGACY_DIR), base.join(name()));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join("preferences.json"), r#"{"theme":"light"}"#).unwrap();
        std::fs::write(new.join("preferences.json"), r#"{"theme":"dark"}"#).unwrap();

        move_preferences(&new, &old);

        assert_eq!(
            std::fs::read_to_string(new.join("preferences.json")).unwrap(),
            r#"{"theme":"dark"}"#
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A fresh install has nothing to carry, and must not be left with an empty
    /// directory implying it does.
    #[test]
    fn nothing_to_migrate_creates_nothing() {
        let base = scratch("fresh");
        let (old, new) = (base.join(LEGACY_DIR), base.join(name()));

        move_preferences(&new, &old);

        assert!(
            !new.exists(),
            "{} should not have been created",
            new.display()
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
