//! Reloading Octane — and a better answer than `--watch`.
//!
//! ## What Octane costs, once it is working
//!
//! [`crate::generator`] already writes Swoole and RoadRunner entry points and
//! both of them run `octane:start`; `SUPPORTED_SERVERS` counts `frankenphp`
//! too. So a project can be served by Octane here today.
//!
//! What is missing is the consequence. Octane boots the application **once** and
//! keeps it in memory, so an edited file changes nothing — a route added to
//! `routes/web.php` does not exist until the server restarts, and the developer
//! reloads the page, sees a 404, and goes looking for the mistake in their own
//! code.
//!
//! ## Why not `octane:start --watch`
//!
//! Laravel's own answer, and the price of it is **installing Node and chokidar
//! into the image**: a second file watcher, inside a container, polling a bind
//! mount that the host is already watching. On a macOS or Windows bind mount
//! that polling is the expensive kind.
//!
//! This application already watches the host filesystem ([`crate::watcher`]) and
//! already runs commands inside the project's container. So the honest answer
//! is one action —
//!
//! ```text
//! php artisan octane:reload
//! ```
//!
//! — optionally attached to the watcher that is already running. It adds
//! **nothing to the image**, which makes it strictly better than the documented
//! route rather than merely different from it.
//!
//! ## Two decisions, and both of them are restraints
//!
//! | Decision | Why |
//! | --- | --- |
//! | The reload is **debounced**, and by seconds rather than milliseconds | A `composer install` touches thousands of files. One reload per file is a server that never finishes booting — [`DEBOUNCE`] |
//! | It is **off by default**, per project | A reload that arrives while a request is being served kills that request. Somebody has to decide that is a trade they want, and the default cannot decide it for them |
//!
//! ## What counts as a change
//!
//! [`WATCHED`] is Octane's own default watch list, and it is a list rather than
//! "any file under the project" for a reason a bind mount makes obvious: a
//! build writing into `public/build`, a log rotating in `storage`, a
//! `node_modules` install — none of those changed the application, and each of
//! them would otherwise reload a server mid-request.

use std::path::Path;
use std::time::Duration;

/// Long enough that a `composer install` is one reload rather than four
/// thousand.
///
/// Seconds, deliberately, where [`crate::watcher`]'s own debounce is 400ms.
/// That one is answering "did the editor write this file three times", which is
/// a question about one save; this one is answering "has the developer stopped
/// changing things", which is a question about a whole operation.
pub const DEBOUNCE: Duration = Duration::from_secs(2);

/// The paths Octane itself watches, relative to the project root.
///
/// Octane's own `config/octane.php` default, kept rather than invented: a
/// broader list would reload on things that did not change the application, and
/// a narrower one would leave the developer with the exact bug this exists to
/// remove.
pub const WATCHED: &[&str] = &[
    "app",
    "bootstrap",
    "config",
    "database",
    "public",
    "resources",
    "routes",
    "composer.lock",
    ".env",
];

/// Directories under a watched path that never mean the application changed.
///
/// `public` and `resources` are on the list above because that is where views
/// and PHP entry points live — and they are also where a front-end build writes
/// its output, several hundred files at a time. Without this, running Vite is
/// a reload loop.
pub const IGNORED: &[&str] = &["node_modules", "vendor", "build", "hot", ".git"];

/// Did this change touch the application?
///
/// `relative` is the path inside the project. A path outside any watched entry
/// is not a change to the application, and neither is one inside an ignored
/// directory at any depth.
pub fn is_application_change(relative: &Path) -> bool {
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    let Some(first) = parts.first() else {
        return false;
    };
    if !WATCHED.contains(&first.as_str()) {
        return false;
    }
    if parts.iter().any(|p| IGNORED.contains(&p.as_str())) {
        return false;
    }
    // An editor's swap file is not a save. `.swp`, `~` and the dot-prefixed
    // temporary a save-then-rename leaves behind are all the same event seen
    // too early — the rename that follows is the one that matters, and it
    // arrives as a change to the real name.
    let Some(name) = parts.last() else {
        return false;
    };
    !(name.ends_with('~') || name.ends_with(".swp") || name.starts_with('.') && name != ".env")
}

/// Is this project served by something Octane runs?
///
/// The three servers `octane:start` can drive. `nginx`, `apache` and `caddy`
/// serve through PHP-FPM, which reads the file on every request — there is
/// nothing to reload there, and offering the button would be offering a
/// no-op with an explanation nobody would find.
pub fn is_octane(server: &str) -> bool {
    matches!(server, "swoole" | "roadrunner" | "frankenphp")
}

/// The `docker` arguments that reload one project's workers.
///
/// `octane:reload` rather than a restart: it tells the running server to
/// replace its workers, so the socket stays open and nothing outside notices
/// beyond the requests that were in flight.
pub fn reload_args(project: &str) -> Vec<String> {
    [
        "exec",
        "-i",
        &crate::engine::container_name(project),
        "php",
        "artisan",
        "octane:reload",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// The preferences key holding which projects have the automatic reload on.
///
/// A preference rather than a manifest field, and that is the distinction: the
/// manifest travels with the repository and describes what the project *is*,
/// while "reload my workers when I save" is a thing one developer wants on one
/// machine. A colleague who wants their requests to survive an editor save
/// should not inherit this from a `git pull`.
pub const PREF_KEY: &str = "octaneReload";

/// Which projects have it on, out of a preferences document.
pub fn enabled_in(prefs: &serde_json::Value, project: &str) -> bool {
    prefs
        .get(PREF_KEY)
        .and_then(|v| v.get(project))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

// ------------------------------------------------------------- the reload

/// Reload this project's workers, if the developer asked for that.
///
/// Called from the watcher thread, which is not async and must not block: a
/// `docker exec` that took a second would stall every file event behind it. So
/// the check is cheap and synchronous, and the command itself goes onto the
/// async runtime.
///
/// Every outcome is emitted, failures included. A reload that silently did not
/// happen is the same experience as the bug this feature removes.
pub fn reload_if_enabled(app: &tauri::AppHandle, project: String) {
    let prefs = crate::appdir::config()
        .map(|dir| dir.join("preferences.json"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    if !enabled_in(&prefs, &project) {
        return;
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let args = reload_args(&project);
        let mut lines = Vec::new();
        let result = crate::runner::stream("docker", &args, std::path::Path::new("."), |line| {
            lines.push(line.to_string());
        })
        .await;

        let ok = result.as_ref().map(|o| o.success).unwrap_or(false);
        crate::events::emit(
            &handle,
            "octane:reloaded",
            serde_json::json!({
                "project": project,
                "ok": ok,
                "output": lines.join("\n"),
            }),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn only_the_paths_octane_itself_watches_count_as_a_change() {
        for path in [
            "routes/web.php",
            "app/Http/Controllers/HomeController.php",
            "config/app.php",
            "composer.lock",
            ".env",
            "resources/views/home.blade.php",
        ] {
            assert!(is_application_change(&PathBuf::from(path)), "{path}");
        }

        for path in [
            // Not on the list at all.
            "storage/logs/laravel.log",
            "tests/Feature/HomeTest.php",
            "README.md",
            // On the list, but inside something that means a build ran rather
            // than that the application changed.
            "public/build/assets/app-abc123.js",
            "public/hot",
            "resources/node_modules/x/index.js",
            "app/vendor/x.php",
            // An editor mid-save. The rename that follows is the real event.
            "routes/web.php~",
            "routes/.web.php.swp",
            "app/.#Home.php",
        ] {
            assert!(!is_application_change(&PathBuf::from(path)), "{path}");
        }

        // `.env` is dot-prefixed and is the one dotfile that is a real change.
        assert!(is_application_change(&PathBuf::from(".env")));
    }

    /// Only the servers `octane:start` drives. PHP-FPM reads the file on every
    /// request, so a reload button there would be a no-op with a story.
    #[test]
    fn the_button_is_offered_only_where_there_is_something_to_reload() {
        for server in ["swoole", "roadrunner", "frankenphp"] {
            assert!(is_octane(server), "{server}");
        }
        for server in ["nginx", "apache", "caddy", ""] {
            assert!(!is_octane(server), "{server}");
        }
    }

    /// A reload, not a restart — and inside this project's container.
    #[test]
    fn the_reload_replaces_workers_in_the_projects_own_container() {
        assert_eq!(
            reload_args("shop"),
            [
                "exec",
                "-i",
                "stackvo-shop",
                "php",
                "artisan",
                "octane:reload"
            ]
        );
    }

    /// Off unless somebody said otherwise, in any shape the file might be in.
    #[test]
    fn the_automatic_reload_is_off_until_it_is_turned_on() {
        assert!(!enabled_in(&serde_json::json!({}), "shop"));
        assert!(!enabled_in(
            &serde_json::json!({ "octaneReload": {} }),
            "shop"
        ));
        assert!(!enabled_in(
            &serde_json::json!({ "octaneReload": { "shop": false } }),
            "shop"
        ));
        assert!(!enabled_in(
            &serde_json::json!({ "octaneReload": { "other": true } }),
            "shop"
        ));
        assert!(enabled_in(
            &serde_json::json!({ "octaneReload": { "shop": true } }),
            "shop"
        ));
    }
}
