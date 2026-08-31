//! Catching debug signals out of a running container, without restarting it.
//!
//! ## What was wrong with the version this replaces
//!
//! [`crate::dumps`] points Symfony's `VarDumper` at Symfony's own
//! `var-dump-server`, started with `docker exec` inside the project container.
//! It works, and three things about it cannot be fixed from where it stands:
//!
//! 1. **Turning it on costs a container.** The two variables arrive through a
//!    compose overlay, and a container's environment is fixed when it is
//!    created — so "start catching dumps" means recreating the container and
//!    waiting for the stack. Herd and Lerd both toggle theirs live; Lerd sells
//!    that explicitly ("no FPM container restart, no worker cascade").
//! 2. **It only sees `dump()`.** Queries, jobs, rendered views and outgoing
//!    HTTP calls are the signals people actually spend the day on, and none of
//!    them reaches `VarDumper`.
//! 3. **The output is text.** Symfony's CLI renderer produces a formatted
//!    block; the app streams it into a `<pre>`. Nothing can be searched,
//!    grouped by request, or linked back to the line that produced it, because
//!    by the time it arrives it is no longer data.
//!
//! ## The mechanism
//!
//! A PHP file, mounted into the container and loaded through
//! `auto_prepend_file`, plus a sentinel file the host creates and removes.
//!
//! `auto_prepend_file` runs **before the application's autoloader**, and that
//! is what makes this work at all: Symfony guards its own helpers with
//! `function_exists('dump')`, so whichever definition arrives first wins, and
//! the prepend file is always first. Lerd's bridge does the same thing for the
//! same reason. Nothing is hooked, patched or observed — the functions are
//! simply declared.
//!
//! It follows that the bridge must declare them **only while capture is on**.
//! With the sentinel absent it declares nothing and returns after a single
//! `is_file`, so Symfony's own `dump()` loads and behaves exactly as it does
//! without this app. That is also why toggling is free: the next request reads
//! the flag again, and a flag is a file in a directory that is already mounted.
//!
//! ## Why a file and not a socket
//!
//! The obvious design is a receiver on the host and a socket from the
//! container, which is what both competitors do — and what the Xdebug overlay
//! already proves is reachable, since it dials `host.docker.internal`. It was
//! rejected on two counts. A host-side listener has to bind somewhere, and the
//! only address every platform agrees a container can reach is not loopback —
//! a dump stream carries request bodies and query bindings, and putting that on
//! a listening port is a bigger decision than the feature is worth. And the
//! events have to survive a page reload to be worth grouping, which means
//! writing them down somewhere regardless.
//!
//! So the bridge appends newline-delimited JSON to a file in a writable mount,
//! and the app tails it. No port, no `host.docker.internal`, no difference
//! between platforms, and "keep the last N events across restarts" — a paid
//! feature elsewhere — is what the design already does.
//!
//! ## What the bridge can see, and the one thing it cannot
//!
//! Two kinds are written from inside the container. A `dump` is a moment: the
//! value, the file and the line. A `request` is the *execution* — one row per
//! page load or per `artisan` command, with the status and how long it took —
//! and it is raised from `register_shutdown_function` rather than from any
//! framework's own "request handled" event. That is not a fallback: PHP itself
//! guarantees the shutdown handler, it runs for a fatal and for an `exit()` as
//! well as for a clean return, and it needs nothing loaded. So the row appears
//! for Laravel, for Symfony, for WordPress and for one hand-written
//! `index.php` alike.
//!
//! **A queued job is not one of them, and the reason was measured.** The
//! obvious design is four listeners on Laravel's own queue events, and there
//! is no moment in this file where they can be attached: it is loaded before
//! the container exists. Watching the autoloader for the queue's classes looks
//! like the way in and is not — Composer registers its own loader with
//! `$prepend = true`, so it lands in *front* of anything registered here, and
//! a handler behind it is never reached for a class Composer can resolve. Run
//! against a real Laravel 12 checkout, `spl_autoload_functions()` comes back
//! `[Composer\Autoload\ClassLoader::loadClass, <this file>]` in that order and
//! the queue's events raise nothing. Jobs therefore arrive from the host, out
//! of the worker's own output — see [`crate::queuelog`], which makes the same
//! argument [`crate::querylog`] makes about the database.
//!
//! ## Verified in a container, not reasoned about
//!
//! Every claim above was run against `php 8.4` in one of this checkout's own
//! project containers before the module was finished:
//!
//! * with the sentinel absent the bridge declares nothing, so `dump()` is
//!   still Symfony's;
//! * with it present, `dump()` captures and returns its argument, and `dd()`
//!   captures and exits 1;
//! * the emitted line parses as the [`Event`] below, with file, line, request
//!   and SAPI filled in;
//! * one `request` line follows every execution, carrying the status the web
//!   SAPI reported (`200` and a deliberate `503` were both read back) and a
//!   duration measured from `REQUEST_TIME_FLOAT`, and it is still written
//!   after `dd()` has called `exit()`;
//! * a `dump()` inside a **queued job** reaches the file — but only once the
//!   worker sidecar is given the same three mounts as the web container, which
//!   is a thing [`crate::worker`] had to be taught;
//! * an `exit()` inside a shutdown function really does skip every handler
//!   registered behind it — two handlers, the first exiting, and the second
//!   printed nothing. That is why the request row is registered at prepend
//!   time rather than anywhere later: first on the stack is the only position
//!   a framework's own shutdown handler cannot take it out of.
//!
//! Two things only showed up by running it. **`php -r` does not process
//! `auto_prepend_file` at all** — `ini_get` reports the setting and the file is
//! never loaded; it costs nothing here, because web requests, `artisan` and
//! queue workers all execute a script, but a `-r` one-liner is not a way to
//! test this. And `spl_autoload_register`'s `$do_throw` argument makes PHP 8
//! print a notice when it is `false`, which a prepend file would put into
//! everybody's response — the single most visible thing this file could ever
//! do.

use crate::error::{Code, Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Where the read-only half is mounted: the bridge and the sentinel.
pub const CONF_DIR: &str = "/usr/local/etc/stackvo";

/// The file the bridge appends a captured session to, beside the events.
///
/// Named here rather than in [`crate::capture`] because the bridge is what
/// writes it: the reader points at this, so the two cannot come to disagree
/// about a filename that only exists inside a container.
pub const SESSIONS_FILE: &str = "sessions.jsonl";

/// What one captured body may weigh before the bridge drops it.
///
/// Dropped rather than truncated, and the PHP says why: half a body is not a
/// request, and replaying one would answer with something that looks like a
/// replay and is not.
pub const MAX_BODY: usize = 64 * 1024;

/// Where the writable half is mounted. Deliberately *not* nested inside
/// [`CONF_DIR`]: a bind mount inside another bind mount is ordering-sensitive
/// on some engines, and there is nothing to gain by finding out which.
pub const EVENTS_DIR: &str = "/usr/local/etc/stackvo-events";

/// The ini in `conf.d`, mounted as a single file.
///
/// `95-` so it lands before the user's `zz-stackvo.ini` and the profiler's
/// `zzz-stackvo-xdebug.ini`. Ordering does not matter for `auto_prepend_file`
/// itself, but a config file that sorts into the middle of somebody else's
/// numbering is a thing to reason about later for no reason.
pub const INI_CONTAINER_PATH: &str = "/usr/local/etc/php/conf.d/95-stackvo-debug.ini";

/// The events file, inside [`EVENTS_DIR`].
pub const EVENTS_FILE: &str = "events.ndjson";

/// Stop growing the file. Reached, it is rotated to `.1` and started again, so
/// the worst case on disk is two of these rather than one that never stops.
pub const MAX_EVENTS_BYTES: u64 = 8 * 1024 * 1024;

pub fn overlay_path(root: &Path) -> PathBuf {
    root.join("generated").join("docker-compose.debug.yml")
}

/// `<root>/generated/debug/<project>` — the per-project home for all of it.
///
/// Per project rather than one shared directory so the bridge can always look
/// at the same absolute path: it has no way to learn which project it is in
/// without an environment variable, and an environment variable would need the
/// container recreating, which is the whole thing this avoids.
pub fn project_dir(root: &Path, project: &str) -> PathBuf {
    root.join("generated").join("debug").join(project)
}

pub fn conf_dir(root: &Path, project: &str) -> PathBuf {
    project_dir(root, project).join("conf")
}

pub fn events_dir(root: &Path, project: &str) -> PathBuf {
    project_dir(root, project).join("events")
}

pub fn sentinel_path(root: &Path, project: &str) -> PathBuf {
    conf_dir(root, project).join("enabled.flag")
}

pub fn events_path(root: &Path, project: &str) -> PathBuf {
    events_dir(root, project).join(EVENTS_FILE)
}

/// One captured signal, as the bridge writes it and the UI reads it.
///
/// Every field is optional except the ones the bridge can always know, because
/// this is parsed from a file a container wrote: a line that is missing
/// something is worth showing without its label, not worth dropping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Seconds since the epoch, with fractions — the bridge's `microtime`.
    pub at: f64,
    /// `dump`, `request` or `job` — see [`KINDS`]. The field was written before
    /// there was a second value for it, so that a second signal would need no
    /// second file and no second reader. Two have arrived and it did not.
    pub kind: String,
    /// The variable's name where `dump($user)` makes one available.
    #[serde(default)]
    pub label: Option<String>,
    /// Relative to the project root where it could be made relative.
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub line: Option<u32>,
    /// `GET /api/health`, or the artisan command for a CLI run.
    #[serde(default)]
    pub request: Option<String>,
    /// `fpm-fcgi`, `cli`, … — what tells a queue worker from a web request.
    #[serde(default)]
    pub sapi: Option<String>,
    /// Milliseconds, where the producer knew how long the thing took.
    ///
    /// A dump is a moment and carries none; a request and a job are stretches
    /// and carry one. Optional rather than zero, because "took no measurable
    /// time" and "nobody measured" are different claims and a screen that
    /// prints `0 ms` for the second one is lying about the first.
    #[serde(default)]
    pub duration: Option<f64>,
    /// How it ended: an HTTP status for a request, `ok` / `failed` for a job.
    ///
    /// A string and not an enum for the same reason [`Event::value`] is
    /// untyped — this is parsed out of a file that an older bridge may have
    /// written, and a status nobody anticipated is worth showing verbatim
    /// rather than dropping the line it arrived on.
    #[serde(default)]
    pub outcome: Option<String>,
    /// The captured value: a tree of typed nodes, bounded by the bridge.
    ///
    /// Untyped here on purpose, and not only to avoid restating the bridge's
    /// shape in two languages. This file is the one thing in the system that
    /// cannot be upgraded in step with what wrote it — a queue worker started
    /// before an update keeps the old bridge loaded for as long as it lives,
    /// and the events it already wrote are on disk regardless. Every version of
    /// the node shape has to survive the trip, including the formatted string
    /// the bridge emitted before it captured trees at all, which the pane still
    /// renders. A stricter type here would turn "an old event" into "a line
    /// that fails to parse", which [`read_events`] silently drops.
    ///
    /// Defaulted for the same reason: a request event carries a small tree and
    /// a job event carries a smaller one, but neither is a *captured value* the
    /// way a dump's is, and a future signal that has nothing to show must not
    /// have to invent a `null` to be readable.
    #[serde(default)]
    pub value: serde_json::Value,
}

/// Every `kind` this build's bridge writes.
///
/// Here rather than only in the PHP because two things read it and neither can
/// see the other: [`crate::timeline`] maps a kind to an axis source, and the
/// pane offers one filter chip per kind. A value that appears in the bridge and
/// in neither of them is an event that arrives and is silently grouped with
/// dumps, which is how the field ended up carrying one value for a year.
pub const KINDS: [&str; 3] = ["dump", "request", "job"];

/// The ini that loads the bridge. Written once and never changed at runtime.
///
/// Mounted as a single file, and that is a constraint worth stating: a file
/// bind mount follows the inode, and this app writes through
/// [`crate::atomic`], which renames a new file over the old one. So an edit
/// here does not reach a running container — which is fine, because it never
/// gets edited. The sentinel does, and it lives in a *directory* mount, where
/// creating and removing files is seen immediately.
pub fn ini() -> String {
    format!(
        "; Generated by StackVo Desktop — do not edit.\n\
         ;\n\
         ; Loads the debug bridge before the application. The bridge does\n\
         ; nothing at all unless {CONF_DIR}/enabled.flag exists, so this costs\n\
         ; one stat per request while capture is off.\n\
         auto_prepend_file={CONF_DIR}/bridge.php\n"
    )
}

/// The bridge itself.
///
/// Written as one string rather than shipped as an asset because it is
/// generated *per project* — the paths are absolute inside the container and
/// identical for every project, but the file has to exist in each project's
/// own mounted directory, and one source of truth for its text beats a copy
/// that can drift.
pub fn bridge_php() -> String {
    format!(
        r#"<?php
// Generated by StackVo Desktop — do not edit; rewritten before every compose
// command.
//
// Loaded through auto_prepend_file, which runs before the application's
// autoloader. That is why dump() and dd() are DEFINED here rather than hooked:
// Symfony guards its own helpers with function_exists(), so whichever arrives
// first wins, and this file is always first.
//
// Nothing below runs unless the sentinel exists. With capture off this file
// declares no functions at all, so Symfony's own dump() loads and behaves
// exactly as it would without StackVo.

if (!@is_file('{CONF_DIR}/enabled.flag')) {{
    return;
}}

if (!function_exists('__stackvo_emit')) {{
    /**
     * Capture a value as a tree, with hard bounds.
     *
     * Not Symfony's cloner: it is not loaded yet and cannot be. The bounds are
     * the point — an Eloquent model graph or a container instance will happily
     * serialise to megabytes, and a debug pane that stalls the request it is
     * observing is worse than no debug pane.
     *
     * A tree rather than the formatted block this used to return. The block was
     * cheaper here and cost the reader everything: a type is not recoverable
     * from text once a string that contains a newline is in it, so the pane
     * could not colour a value, fold a branch, or say "array of 8" without
     * re-parsing prose. What is a type here stays a type all the way to the
     * screen. `n` is the real size and `items` is what survived the bound, so
     * the pane can say what it is not showing instead of pretending it showed
     * everything.
     */
    function __stackvo_capture($value, int $depth = 0): array
    {{
        if ($depth > 4) {{
            return ['t' => 'deep'];
        }}
        if ($value === null) {{
            return ['t' => 'null'];
        }}
        if (is_bool($value)) {{
            return ['t' => 'bool', 'v' => $value];
        }}
        if (is_int($value) || is_float($value)) {{
            // NAN and INF are floats that json_encode refuses, and a line that
            // fails to encode is an event nobody ever sees. They arrive as the
            // text PHP prints for them instead of taking the dump down.
            if (is_float($value) && !is_finite($value)) {{
                return ['t' => 'num', 's' => (string) $value];
            }}
            return ['t' => 'num', 'v' => $value];
        }}
        if (is_string($value)) {{
            $len = strlen($value);
            $node = ['t' => 'str', 'len' => $len];
            $node['v'] = $len > 512 ? substr($value, 0, 512) : $value;
            if ($len > 512) {{
                $node['cut'] = true;
            }}
            return $node;
        }}
        if (is_array($value)) {{
            $node = ['t' => 'arr', 'n' => count($value), 'items' => []];
            $n = 0;
            foreach ($value as $k => $v) {{
                if ($n++ >= 50) {{
                    break;
                }}
                $node['items'][] = ['k' => $k, 'v' => __stackvo_capture($v, $depth + 1)];
            }}
            return $node;
        }}
        if (is_object($value)) {{
            // Closures have nothing worth walking into.
            if ($value instanceof \Closure) {{
                return ['t' => 'fn'];
            }}
            $props = (array) $value;
            $node = ['t' => 'obj', 'class' => get_class($value), 'n' => count($props), 'items' => []];
            $n = 0;
            foreach ($props as $k => $v) {{
                if ($n++ >= 50) {{
                    break;
                }}
                // Private and protected keys arrive NUL-padded from the cast.
                $name = str_replace("\0", '·', (string) $k);
                $node['items'][] = ['k' => $name, 'v' => __stackvo_capture($v, $depth + 1)];
            }}
            return $node;
        }}
        return ['t' => 'other', 'v' => gettype($value)];
    }}

    /** Where the call came from, skipping this file's own frames. */
    function __stackvo_origin(): array
    {{
        $frames = @debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 8) ?: [];
        foreach ($frames as $frame) {{
            $file = $frame['file'] ?? '';
            if ($file !== '' && strpos($file, '{CONF_DIR}') !== 0) {{
                return [$file, $frame['line'] ?? null];
            }}
        }}
        return ['', null];
    }}

    function __stackvo_emit(string $kind, array $payload): void
    {{
        // Checked again rather than once at load: a long-running queue worker
        // outlives many toggles, and a worker that kept writing after the
        // switch was turned off is exactly the "worker cascade" this design
        // exists to avoid.
        if (!@is_file('{CONF_DIR}/enabled.flag')) {{
            return;
        }}

        // Only when the caller did not already answer it. A dump happened at a
        // line and the backtrace is the only way to find which; a job event is
        // raised from inside the framework's dispatcher, where every frame
        // belongs to Illuminate and naming one would point the reader at
        // somebody else's code. Passing `file` explicitly is how a producer
        // says "there is no call site here", and it also saves the walk.
        if (!array_key_exists('file', $payload)) {{
            [$file, $line] = __stackvo_origin();
            $payload['file'] = $file !== '' ? $file : null;
            $payload['line'] = $line;
        }}

        $request = null;
        if (PHP_SAPI === 'cli') {{
            $request = implode(' ', array_slice($_SERVER['argv'] ?? [], 0, 6));
        }} elseif (isset($_SERVER['REQUEST_URI'])) {{
            $request = trim(($_SERVER['REQUEST_METHOD'] ?? '') . ' ' . $_SERVER['REQUEST_URI']);
        }}

        $event = $payload + [
            'at' => microtime(true),
            'kind' => $kind,
            'request' => $request !== '' ? $request : null,
            'sapi' => PHP_SAPI,
        ];

        $line = @json_encode($event, JSON_UNESCAPED_SLASHES | JSON_INVALID_UTF8_SUBSTITUTE);
        if ($line === false) {{
            return;
        }}
        // LOCK_EX because several FPM workers share this file. One line per
        // write and an exclusive lock is enough; nothing here needs ordering
        // beyond what the filesystem already gives.
        @file_put_contents('{EVENTS_DIR}/{EVENTS_FILE}', $line . "\n", FILE_APPEND | LOCK_EX);
    }}
}}

// ----------------------------------------------------- the execution itself

if (!function_exists('__stackvo_finish')) {{
    /**
     * One event per execution, written when PHP is finished with it.
     *
     * Framework-free, and that is the whole reason it is here rather than in a
     * listener. Laravel raises `RequestHandled`, Symfony raises
     * `kernel.terminate`, WordPress raises neither, and this file is loaded
     * before any of the three exists. `register_shutdown_function` is the one
     * hook PHP itself guarantees, and it is called for a fatal and for an
     * `exit()` as well as for a clean return — which are the three endings
     * somebody most wants a row for.
     *
     * The clock is PHP's own `REQUEST_TIME_FLOAT`, stamped before the first
     * line of any of this ran, so the duration covers the autoloader and the
     * framework's boot. Starting the clock here would quietly exclude both,
     * and on a cold opcache those are most of the answer.
     */
    function __stackvo_finish(float $started): void
    {{
        $payload = ['duration' => round((microtime(true) - $started) * 1000, 3)];

        // `http_response_code()` answers `false` under the CLI SAPI rather
        // than a number. Under a web SAPI it answers even after a fatal, which
        // is the case worth having: a 500 nobody logged is exactly the request
        // somebody is looking for.
        $code = PHP_SAPI === 'cli' ? false : @http_response_code();
        if (is_int($code)) {{
            $payload['outcome'] = (string) $code;
        }}

        $payload['value'] = __stackvo_capture([
            'memory' => memory_get_peak_usage(true),
        ]);

        __stackvo_emit('request', $payload);
        __stackvo_session();
    }}

    /**
     * The cookies and body this request actually carried.
     *
     * Written to a file of its own rather than into the event stream, and
     * behind a SECOND flag rather than the one that turns the bridge on. Both
     * are deliberate. A separate file is one thing to delete when the window
     * closes; a separate flag is what makes "show me my dumps" and "record my
     * session token" two different permissions instead of one.
     *
     * Nothing here runs without `capture.flag`, which only ever exists while a
     * window this app opened is open, and which that window's expiry removes.
     */
    function __stackvo_session(): void
    {{
        if (PHP_SAPI === 'cli' || !@is_file('{CONF_DIR}/capture.flag')) {{
            return;
        }}
        if (!isset($_SERVER['REQUEST_URI'])) {{
            return;
        }}

        // `php://input` is empty once a framework has read it, and on a
        // POST with a form encoding PHP has already consumed it into $_POST.
        // Both paths are taken, in that order, because the raw body is the
        // one that replays byte for byte and the rebuilt one is better than
        // nothing.
        $body = @file_get_contents('php://input');
        if (($body === false || $body === '') && !empty($_POST)) {{
            $body = http_build_query($_POST);
        }}
        if (is_string($body) && strlen($body) > {MAX_BODY}) {{
            // Dropped rather than truncated. Half a body is not a request, and
            // sending one would be answering with something that looks like a
            // replay and is not.
            $body = null;
        }}

        $session = [
            'at' => microtime(true),
            'request' => trim(($_SERVER['REQUEST_METHOD'] ?? '') . ' ' . $_SERVER['REQUEST_URI']),
            'method' => $_SERVER['REQUEST_METHOD'] ?? 'GET',
            'cookie' => $_SERVER['HTTP_COOKIE'] ?? null,
            'body' => ($body === false || $body === '') ? null : $body,
            'contentType' => $_SERVER['CONTENT_TYPE'] ?? null,
        ];

        $line = @json_encode($session, JSON_UNESCAPED_SLASHES | JSON_INVALID_UTF8_SUBSTITUTE);
        if ($line === false) {{
            return;
        }}
        @file_put_contents('{EVENTS_DIR}/{SESSIONS_FILE}', $line . "\n", FILE_APPEND | LOCK_EX);
    }}

    // Registered at prepend time, which puts it FIRST on the shutdown stack —
    // before anything the application registers. That ordering is not
    // cosmetic: `exit()` inside a shutdown function skips every handler behind
    // it, and a framework that ends its own request that way would take this
    // one with it.
    register_shutdown_function(
        '__stackvo_finish',
        (float) ($_SERVER['REQUEST_TIME_FLOAT'] ?? microtime(true))
    );
}}

if (!function_exists('dump')) {{
    function dump(...$vars)
    {{
        foreach ($vars as $key => $value) {{
            __stackvo_emit('dump', [
                'label' => is_string($key) ? $key : null,
                'value' => __stackvo_capture($value),
            ]);
        }}
        return count($vars) === 1 ? $vars[array_key_first($vars)] : $vars;
    }}
}}

if (!function_exists('dd')) {{
    function dd(...$vars)
    {{
        // Symfony's own dd() sets this, and it is preserved deliberately: it
        // exists so a dd() left in code cannot be cached as a success, and
        // quietly disagreeing with the framework about what dd() means is a
        // worse surprise than the 500 itself.
        if (!in_array(PHP_SAPI, ['cli', 'phpdbg', 'embed'], true) && !headers_sent()) {{
            header('HTTP/1.1 500 Internal Server Error');
        }}

        foreach ($vars as $key => $value) {{
            __stackvo_emit('dump', [
                'label' => is_string($key) ? $key : null,
                'value' => __stackvo_capture($value),
            ]);
        }}

        exit(1);
    }}
}}
"#
    )
}

/// One project's worth of mounts — the overlay's input, and the worker's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub service: String,
    pub conf_host: String,
    pub events_host: String,
    pub ini_host: String,
}

/// Render the overlay, or None when no project is eligible.
///
/// Three mounts and not one: the configuration is read-only because the
/// container has no business rewriting what this app generated for it, the
/// events directory has to be writable because that is the whole channel, and
/// the ini has to land in `conf.d` by itself — mounting the directory would
/// shadow every other ini in it.
pub fn overlay_yaml(entries: &[Entry]) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    let mut out = String::from(
        "# Generated by StackVo Desktop — do not edit.\n\
         #\n\
         # Mounts the debug bridge. The bridge does nothing until the sentinel\n\
         # file appears in the conf directory, which is why turning capture on\n\
         # and off needs no container at all — only these mounts do.\n\
         #\n\
         # NOTE: `stackvo up` from the Bash CLI does not layer this file.\n\
         services:\n",
    );

    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.service.cmp(&b.service));

    for entry in sorted {
        out.push_str(&format!("  {}:\n", entry.service));
        out.push_str("    volumes:\n");
        // Quoted because a Windows host path contains a colon, which is the
        // separator compose splits these on.
        out.push_str(&format!("      - \"{}:{CONF_DIR}:ro\"\n", entry.conf_host));
        out.push_str(&format!("      - \"{}:{EVENTS_DIR}\"\n", entry.events_host));
        out.push_str(&format!(
            "      - \"{}:{INI_CONTAINER_PATH}:ro\"\n",
            entry.ini_host
        ));
    }

    Some(out)
}

// ------------------------------------------------------------------- I/O

/// Write one project's half of the bridge and answer with its mounts.
///
/// Split out of [`entries`] because a second caller arrived that has nothing
/// to do with compose: a queue worker is a sidecar this app starts with
/// `docker run`, and it needs the same three mounts for the same reason.
/// Without them the bridge is simply absent from the one process where a
/// `dump()` is hardest to catch by any other means — which is what it was,
/// unnoticed, until a job was run against a container with the mounts and one
/// without.
///
/// `None` when the directories or the files could not be written. The caller's
/// honest degradation is "no bridge here", never "this container cannot start".
pub fn prepare(root: &Path, project: &str) -> Option<Entry> {
    let conf = conf_dir(root, project);
    let events = events_dir(root, project);
    std::fs::create_dir_all(&conf).ok()?;
    std::fs::create_dir_all(&events).ok()?;

    // Rewritten every time, like every other generated file: an overlay that
    // is a pure function of the tree cannot be stale.
    let ini_file = conf.join("stackvo-debug.ini");
    crate::atomic::write(&conf.join("bridge.php"), &bridge_php()).ok()?;
    crate::atomic::write(&ini_file, &ini()).ok()?;

    Some(Entry {
        service: project.to_string(),
        conf_host: crate::paths::to_docker_mount(&conf.display().to_string()),
        events_host: crate::paths::to_docker_mount(&events.display().to_string()),
        ini_host: crate::paths::to_docker_mount(&ini_file.display().to_string()),
    })
}

/// Every PHP project with a compose service to mount the bridge on.
///
/// Eligibility is deliberately wide — every PHP project, whether or not anyone
/// has switched capture on. The mounts are what needs a container, so they go
/// in once, for everybody, and the switch afterwards is free. A project that
/// never uses this pays three bind mounts and one `is_file` per request.
///
/// The service check is not belt-and-braces: naming a service the generator did
/// not emit declares one with neither an image nor a build context, and compose
/// then refuses every command against the stack.
fn entries(root: &Path) -> Vec<Entry> {
    let mut out = Vec::new();

    let generated =
        std::fs::read_to_string(root.join("generated").join("docker-compose.projects.yml"))
            .unwrap_or_default();
    let services = crate::xdebug::generated_services(&generated);

    let Some(projects) = crate::workspace::projects_root(root) else {
        return out;
    };
    let Ok(dirs) = std::fs::read_dir(&projects) else {
        return out;
    };

    for dir in dirs.flatten() {
        let path = dir.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !services.iter().any(|s| s == name) {
            continue;
        }
        let manifest = path.join("stackvo.json");
        if !manifest.is_file() {
            continue;
        }
        let Ok(m) = crate::manifest::read(&manifest, name) else {
            continue;
        };
        if m.runtime != "php" {
            continue;
        }

        let Some(entry) = prepare(root, name) else {
            continue;
        };
        out.push(entry);
    }

    out.sort_by(|a, b| a.service.cmp(&b.service));
    out
}

/// Bring every project's bridge up to date with this build, once per run.
///
/// The bridge is a generated file that lives on the host and is mounted into a
/// container as part of a *directory*, so rewriting it reaches a container that
/// is already running — no recreate, no restart, nothing for anybody to know
/// about. But it was only ever rewritten on the way into a compose command, and
/// a stack that nobody has started since the app updated therefore keeps
/// running the bridge it was created with. That is not a rare case; it is the
/// normal one. Somebody who leaves their stack up sees the pane render an old
/// event shape and has no way to find out why, because everything about the
/// app is new and the only stale thing is a file they have never heard of.
///
/// Once per root rather than once per process: the workspace can be pointed
/// somewhere else while the app runs, and the projects in the new one deserve
/// the same treatment. Once rather than per call because this is polled — the
/// pane asks for the overview every second, and rewriting eleven files a second
/// to discover that none of them changed is a strange way to spend a disk.
pub fn refresh(root: &Path) {
    static DONE: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

    let Ok(mut done) = DONE.lock() else {
        return;
    };
    if done.as_deref() == Some(root) {
        return;
    }
    // `entries` rewrites the bridge and the ini for every project it finds
    // eligible, which is the whole of the work; its return value is the
    // overlay's input and nothing here needs it.
    let _ = entries(root);
    *done = Some(root.to_path_buf());
}

/// Re-render the overlay. True when it exists and should be layered.
pub fn sync(root: &Path) -> bool {
    let path = overlay_path(root);

    match overlay_yaml(&entries(root)) {
        Some(yaml) => {
            if let Some(parent) = path.parent() {
                if std::fs::create_dir_all(parent).is_err() {
                    return false;
                }
            }
            // A write failure must not take compose down with it. The honest
            // degradation is "the bridge is not mounted", which `status` then
            // reports, rather than "no container can be started".
            match crate::atomic::write(&path, &yaml) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "could not write the debug overlay");
                    let _ = std::fs::remove_file(&path);
                    false
                }
            }
        }
        None => {
            let _ = std::fs::remove_file(&path);
            false
        }
    }
}

/// What the pane needs to know.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// PHP, and the generator emitted a service for it.
    pub supported: bool,
    /// The sentinel exists — capture is on, with no container involved.
    pub enabled: bool,
    /// The mounts are in the container that is actually running.
    ///
    /// Read from the container, never inferred: `stackvo up` from the Bash CLI
    /// layers three compose files, not eight, and will recreate a container
    /// without them.
    pub mounted: bool,
    pub running: bool,
    /// How many events are waiting in the file.
    pub events: usize,
}

pub async fn status(root: &Path, name: &str) -> Result<Status> {
    let dir = crate::workspace::project_dir(root, name)?;
    let manifest_file = dir.join("stackvo.json");
    if !manifest_file.is_file() {
        return Err(Error::not_found(format!("project {name}")));
    }
    let manifest = crate::manifest::read(&manifest_file, name)?;

    let details = crate::engine::inspect(name).await.ok();
    let mounted = details.as_ref().is_some_and(|d| {
        d.mounts
            .iter()
            .any(|m| m.destination == EVENTS_DIR || m.destination == CONF_DIR)
    });

    Ok(Status {
        supported: manifest.runtime == "php",
        enabled: sentinel_path(root, name).is_file(),
        mounted,
        running: details.as_ref().is_some_and(|d| d.running),
        events: read_events(root, name).len(),
    })
}

/// Turn capture on or off. No container is touched — that is the feature.
pub fn set_enabled(root: &Path, name: &str, on: bool) -> Result<()> {
    crate::workspace::project_dir(root, name)?;
    let flag = sentinel_path(root, name);

    if on {
        if let Some(parent) = flag.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io("creating the debug directory", e))?;
        }

        // Refreshed here and not only before a compose command, because this is
        // the one moment the bridge is known to matter and the only one that
        // does not cost a container. `conf` is mounted as a *directory*, so a
        // rewrite is seen by a container that is already running — which is
        // what makes an updated bridge reach a stack nobody has restarted since
        // the app updated. Without this, turning capture on after an update
        // loads whatever bridge was written when the containers were made, and
        // the pane renders last release's shape until something recreates them.
        let _ = crate::atomic::write(&conf_dir(root, name).join("bridge.php"), &bridge_php());

        std::fs::write(&flag, "")
            .map_err(|e| Error::io(format!("writing {}", flag.display()), e))?;
    } else {
        // See below: the queue's cursor goes with the switch in both
        // directions.
        match std::fs::remove_file(&flag) {
            Ok(()) => {}
            // Already off. The caller asked for a state, not an operation.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::io(format!("removing {}", flag.display()), e)),
        }
    }

    // The worker's log is written whether or not anybody is watching, so the
    // cursor into it is meaningless across a period when nobody was. Removed
    // in both directions: switching on must not replay the hour before it, and
    // switching off must not leave a mark that makes switching on again replay
    // the gap. `queuelog::ingest` re-seeds from the newest line it can see.
    let _ = std::fs::remove_file(crate::queuelog::cursor_path(root, name));

    Ok(())
}

/// Parse the events file, newest last.
///
/// A malformed line is skipped rather than failing the read: the file is
/// appended to by several processes, and a partial write during a crash must
/// not cost the reader every event before it.
pub fn read_events(root: &Path, name: &str) -> Vec<Event> {
    let Ok(text) = std::fs::read_to_string(events_path(root, name)) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<Event>(l).ok())
        .collect()
}

/// Drop everything recorded so far.
///
/// Removed rather than truncated: the file is written by the container, which
/// may be running as a different user, and a host process can always delete an
/// entry from a directory it owns even when it cannot write the file itself.
pub fn clear(root: &Path, name: &str) -> Result<()> {
    crate::workspace::project_dir(root, name)?;
    let path = events_path(root, name);
    // With the cursor gone, the next poll re-seeds from the worker's newest
    // line — so the jobs somebody just dismissed do not come back on it.
    let _ = std::fs::remove_file(crate::queuelog::cursor_path(root, name));
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::new(
            Code::IoError,
            format!("could not clear {}: {e}", path.display()),
        )),
    }
}

/// Keep the file from growing without limit.
///
/// Called on every read rather than on a timer: the only process that can grow
/// it is the container, and the only one that reads it is this app, so the
/// moment a read happens is the moment the size is known to be current.
pub fn rotate_if_large(root: &Path, name: &str) {
    let path = events_path(root, name);
    let Ok(meta) = std::fs::metadata(&path) else {
        return;
    };
    if meta.len() <= MAX_EVENTS_BYTES {
        return;
    }
    let _ = std::fs::rename(&path, path.with_extension("ndjson.1"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bridge_declares_nothing_until_the_sentinel_exists() {
        let php = bridge_php();
        // The early return has to come before every declaration, or a request
        // with capture off pays for parsing and loses Symfony's own dump().
        let guard = php.find("enabled.flag").expect("no sentinel check");
        let first_fn = php.find("function ").expect("no functions at all");
        assert!(
            guard < first_fn,
            "the sentinel is checked after something is already declared"
        );
        assert!(php.contains("return;"));
    }

    /// The whole mechanism rests on this: Symfony guards its helpers with
    /// `function_exists`, so the bridge must guard its own the same way — or
    /// two definitions collide and the request fatals.
    #[test]
    fn the_helpers_are_declared_defensively() {
        let php = bridge_php();
        for name in ["dump", "dd", "__stackvo_emit"] {
            assert!(
                php.contains(&format!("if (!function_exists('{name}'))")),
                "{name} is declared without a guard"
            );
        }
    }

    /// `dd()` sets a 500 in Symfony's own implementation so a forgotten call
    /// cannot be cached as a success. Owning the function is not a licence to
    /// quietly disagree with the framework about what it means.
    #[test]
    fn dd_keeps_the_status_and_the_exit_symfony_gives_it() {
        let php = bridge_php();
        assert!(php.contains("500 Internal Server Error"));
        assert!(php.contains("exit(1);"));
    }

    #[test]
    fn the_overlay_mounts_all_three_and_only_the_events_are_writable() {
        let yaml = overlay_yaml(&[Entry {
            service: "shop".into(),
            conf_host: "/w/generated/debug/shop/conf".into(),
            events_host: "/w/generated/debug/shop/events".into(),
            ini_host: "/w/generated/debug/shop/conf/stackvo-debug.ini".into(),
        }])
        .expect("no overlay");

        assert!(yaml.contains(&format!("/w/generated/debug/shop/conf:{CONF_DIR}:ro")));
        assert!(yaml.contains(&format!("/w/generated/debug/shop/events:{EVENTS_DIR}\"")));
        assert!(yaml.contains(&format!("stackvo-debug.ini:{INI_CONTAINER_PATH}:ro")));
        // The events mount must not be read-only, or the bridge writes nothing
        // and the pane stays empty with everything else looking correct.
        assert!(!yaml.contains(&format!("{EVENTS_DIR}:ro")));
    }

    #[test]
    fn no_eligible_project_means_no_overlay_at_all() {
        assert!(overlay_yaml(&[]).is_none());
    }

    #[test]
    fn the_ini_points_at_the_bridge_the_overlay_mounts() {
        assert!(ini().contains(&format!("auto_prepend_file={CONF_DIR}/bridge.php")));
    }

    #[test]
    fn a_malformed_line_costs_only_itself() {
        let dir = std::env::temp_dir().join(format!("stackvo-bridge-{}", std::process::id()));
        let events = dir.join("generated/debug/shop/events");
        std::fs::create_dir_all(&events).unwrap();
        std::fs::write(
            events.join(EVENTS_FILE),
            "{\"at\":1.0,\"kind\":\"dump\",\"value\":\"one\"}\n\
             not json at all\n\
             \n\
             {\"at\":2.0,\"kind\":\"dump\",\"value\":\"two\"}\n",
        )
        .unwrap();

        let found = read_events(&dir, "shop");
        assert_eq!(found.len(), 2, "a bad line took a good one with it");
        assert_eq!(found[0].value, serde_json::json!("one"));
        assert_eq!(found[1].value, serde_json::json!("two"));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// An event written by an older bridge is still an event.
    ///
    /// The bridge used to render values to a formatted string, and a worker
    /// that was already running when this app updated goes on writing that
    /// shape until it is restarted. Both have to parse, or the pane loses
    /// everything that project dumped before the change.
    #[test]
    fn a_value_from_either_bridge_parses() {
        let old: Event =
            serde_json::from_str(r#"{"at":1.0,"kind":"dump","value":"App\\Models\\User {…}"}"#)
                .expect("the formatted-string shape no longer parses");
        assert!(old.value.is_string());

        let new: Event = serde_json::from_str(
            r#"{"at":1.0,"kind":"dump","value":{"t":"arr","n":2,"items":[]}}"#,
        )
        .expect("the tree shape does not parse");
        assert_eq!(new.value["t"], serde_json::json!("arr"));
    }

    /// The execution row is written by PHP's own shutdown hook and not by a
    /// framework's "request handled" event, so it appears for an application
    /// that has no such event — and after a fatal, which is the ending most
    /// worth a row.
    #[test]
    fn every_execution_ends_in_a_row_of_its_own() {
        let php = bridge_php();
        assert!(php.contains("register_shutdown_function("));
        assert!(php.contains("__stackvo_emit('request'"));
        // The clock is PHP's, stamped before this file was loaded, so the
        // duration covers the autoloader and the framework's boot rather than
        // starting when the bridge did.
        assert!(php.contains("REQUEST_TIME_FLOAT"));
    }

    /// The status is knowable under a web SAPI and is `false` under the CLI.
    /// Printing `0` for a script would be a number that reads as an answer.
    #[test]
    fn the_status_is_only_claimed_where_there_is_one() {
        let php = bridge_php();
        assert!(php.contains("PHP_SAPI === 'cli' ? false : @http_response_code()"));
        assert!(php.contains("is_int($code)"));
    }

    /// Every kind the bridge writes has to be a kind the readers know: the
    /// timeline maps it to an axis source and the pane offers a chip for it.
    /// A value in the PHP and in neither of them is an event that arrives and
    /// is silently filed as a dump.
    #[test]
    fn the_php_writes_no_kind_the_readers_have_not_been_told_about() {
        let php = bridge_php();
        let mut written: Vec<String> = Vec::new();
        for (index, _) in php.match_indices("__stackvo_emit('") {
            let rest = &php[index + "__stackvo_emit('".len()..];
            if let Some(end) = rest.find('\'') {
                written.push(rest[..end].to_string());
            }
        }
        // The declaration itself is `function __stackvo_emit(string $kind` and
        // does not match; what is left is every call site.
        assert!(!written.is_empty(), "no emit call sites found at all");
        for kind in &written {
            assert!(
                KINDS.contains(&kind.as_str()),
                "the bridge writes `{kind}`, which no reader knows"
            );
        }
        assert!(written.iter().any(|k| k == "request"));
        assert!(written.iter().any(|k| k == "dump"));
    }

    /// A row this build has never seen must still parse. The queue's half is
    /// written by the host into the same file, and an event with no captured
    /// value at all is one of the shapes that arrive.
    #[test]
    fn an_event_with_no_value_is_still_an_event() {
        let event: Event = serde_json::from_str(
            r#"{"at":1.0,"kind":"job","label":"App\\Jobs\\Send","outcome":"ok","duration":12.5}"#,
        )
        .expect("a valueless event no longer parses");
        assert_eq!(event.value, serde_json::Value::Null);
        assert_eq!(event.duration, Some(12.5));
        assert_eq!(event.outcome.as_deref(), Some("ok"));
    }

    /// The bounds are the reason this renderer exists rather than a cloner, and
    /// they are what stops a dump of the service container from writing
    /// megabytes into the events file mid-request.
    #[test]
    fn the_capture_keeps_its_bounds_and_reports_what_it_cut() {
        let php = bridge_php();
        assert!(php.contains("$depth > 4"), "the depth bound is gone");
        assert!(php.contains("$n++ >= 50"), "the item bound is gone");
        assert!(php.contains("strlen($value)"), "the string bound is gone");
        // `n` is the real size and `items` is what survived it. Without the
        // count the pane can only show what it was given and has no way to say
        // that anything is missing.
        assert!(php.contains("'n' => count($value)"));
        assert!(php.contains("'n' => count($props)"));
    }
}
