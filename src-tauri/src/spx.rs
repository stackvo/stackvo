//! A sampling profiler that can be left on, and the one the competitors ship.
//!
//! [`crate::profile`] reads Xdebug's cachegrind output and its header explains
//! why it was the route taken: `xdebug.mode=profile` needs no new extension,
//! and the alternatives — SPX, XHProf, Excimer — are not in
//! `contracts/php-extensions.json`. That reasoning was right about the
//! contract and wrong about the conclusion, because it assumed the only way to
//! get an extension into a container is to put it in the manifest.
//!
//! Herd and Lerd both ship **php-spx** and both sell it on the same property:
//! it samples, so it can be left on during a real page load, where Xdebug's
//! profiler costs several times the request. That is not a nicer version of
//! what this app already has; it is the case Xdebug's profiler cannot cover.
//!
//! ## Why this needs no contract change
//!
//! `php-extensions.json` is the data half of the Bash generator's own install
//! matrix, shared with the upstream repository. Adding `spx` to it would claim
//! the Bash CLI knows how to install something it has never heard of — and it
//! could not be honoured anyway: **SPX is not on PECL**. It is built from
//! source, and the contract's `special` install method is documented as
//! "NOT IMPLEMENTED in v1 — readers MUST reject".
//!
//! So the extension never enters the manifest, the Dockerfile or the contract.
//! It is installed the way the debug bridge is installed: into a directory this
//! app owns, mounted into the container, and switched on by an ini in
//! `conf.d`. The image is untouched, and a project that never asks for it pays
//! nothing.
//!
//! ## Built in the image it will run in
//!
//! An extension has to match the exact PHP version, ABI and thread-safety of
//! the binary that loads it. Rather than guess, the build runs in a throwaway
//! container **from the project's own image** — `docker run --rm` — so the
//! compiler, the headers and the target are the same ones php-fpm was built
//! with. The output lands on the host, keyed by PHP version, so two projects on
//! 8.4 share one build and a project on 8.2 gets its own.
//!
//! The build is not run against the *running* container: that would mean
//! `apt-get install` inside somebody's live php-fpm, which lasts until the next
//! recreate and is a side effect nobody asked for.
//!
//! ## Four things measured rather than read
//!
//! Every one of these was established by building and loading SPX in this
//! repository's own project image, and three of them contradict what the
//! documentation implies:
//!
//! 1. **`extension=`, not `zend_extension=`.** SPX is a normal extension.
//!    Loading it as a Zend extension fails with "doesn't appear to be a valid
//!    Zend extension" — which is the error this module was written against on
//!    its first run.
//! 2. **A report is a pair**: `<key>.json` holding the metadata and
//!    `<key>.txt.gz` holding the trace. The JSON is what a list is built from,
//!    so this app does not need SPX's UI to say what was recorded.
//! 3. **The metadata is rich enough to be a list on its own** — wall time, peak
//!    memory, call counts, and whether the run was CLI or a request, with the
//!    URI and method when it was.
//! 4. **`spx_utils_ip_match` accepts `*` and IPv4 CIDR**, and nothing else.
//!    That decides [`ini`]'s whitelist: behind the stack's own proxy the
//!    address SPX sees is the proxy's container address, so the private ranges
//!    are what have to be allowed, and `*` is not needed to do it.

use crate::error::{Code, Error, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Where the built extension and the web UI assets are mounted, read-only.
pub const DIR: &str = "/usr/local/etc/stackvo-spx";

/// Where SPX writes its reports inside the container.
pub const DATA_DIR: &str = "/var/log/spx";

/// Where the ini lands.
///
/// `zzz-` for the reason `crate::xdebug::INI_CONTAINER_PATH` gives: PHP reads
/// `conf.d` alphabetically and the last file wins, and `zz-stackvo.ini` is the
/// *user's* own php.ini. A name sorting before it would let a hand-written
/// `spx.data_dir` send reports somewhere this app cannot read them, which
/// presents as profiling that produced nothing.
pub const INI_CONTAINER_PATH: &str = "/usr/local/etc/php/conf.d/zzz-stackvo-spx.ini";

/// The tag built from. `release/latest` is the branch the project's own README
/// tells people to check out, rather than `master`.
pub const SOURCE_REF: &str = "release/latest";

pub const SOURCE_URL: &str = "https://github.com/NoiseByNorthwest/php-spx.git";

/// The per-project switch, beside the other `.stackvo/` settings.
pub const CONFIG_FILE: &str = "spx.json";

/// Microseconds between samples, by default.
///
/// **Not php-spx's own default, which is `0` — meaning every call.** With that
/// value SPX is a tracing profiler: accurate, and expensive in exactly the way
/// this module exists to avoid. The claim on the pane — the profiler you can
/// leave on — is only true when a period is set, so one is.
///
/// 100 µs is ten thousand samples a second: enough that a function holding a
/// tenth of a 30 ms request shows up with thirty samples behind it, and few
/// enough that the page still feels like the page. It is a setting rather than
/// a constant because the other end of the trade is real — `0` is still the
/// right answer for a fast function you want counted exactly.
pub const SAMPLING_PERIOD_US: u32 = 100;

/// How many functions a hotspot list carries.
///
/// A profile of a Laravel request names thousands. The question being answered
/// is "where did the time go", and it is answered by the first screenful.
pub const HOTSPOTS: usize = 25;

/// Events read from one trace before the rest is skipped.
///
/// A recording of a slow page can be tens of millions of events, and a report
/// nobody asked to be exhaustive should not be able to hold a thread for a
/// minute. The replay is over a prefix, which is a real answer about the start
/// of the request rather than a wrong one about all of it — so it is reported
/// as truncated rather than presented as complete.
pub const MAX_EVENTS: u64 = 2_000_000;

/// Bytes of decompressed trace read at most.
///
/// The file is gzip and its contents are another process's output. A cap here
/// is what stops a 200 MB trace — or a corrupt one that decompresses forever —
/// from being read into this process's memory in full.
pub const MAX_TRACE_BYTES: u64 = 192 * 1024 * 1024;

/// Everything the extension needs, for one PHP version.
///
/// Keyed by PHP version rather than by project: the artefact is a function of
/// the interpreter, so two projects on 8.4 share one build.
pub fn build_dir(root: &Path, php_version: &str) -> PathBuf {
    root.join("generated").join("spx").join(php_version)
}

pub fn extension_path(root: &Path, php_version: &str) -> PathBuf {
    build_dir(root, php_version).join("spx.so")
}

pub fn assets_path(root: &Path, php_version: &str) -> PathBuf {
    build_dir(root, php_version).join("assets")
}

/// Is there a usable build for this PHP version?
///
/// Both halves, because the assets are what the web UI is served from and a
/// build interrupted between the two leaves an extension that loads and a
/// control panel that is a blank page.
pub fn built(root: &Path, php_version: &str) -> bool {
    extension_path(root, php_version).is_file() && assets_path(root, php_version).is_dir()
}

/// Host side of the report directory, per project.
///
/// Under `logs/` rather than in the project, for the reason
/// `crate::profile::host_dir` gives: these accumulate, they are machine-local,
/// and a source tree is not where they belong.
pub fn data_dir(root: &Path, project: &str) -> PathBuf {
    root.join("logs").join("projects").join(project).join("spx")
}

pub fn ini_path(root: &Path, project: &str) -> PathBuf {
    root.join("generated")
        .join("spx")
        .join(format!("{project}.ini"))
}

pub fn overlay_path(root: &Path) -> PathBuf {
    root.join("generated").join("docker-compose.spx.yml")
}

/// Where the key lives.
///
/// In `generated/` and not in `.stackvo/`: the second travels with a clone, and
/// a key committed to a repository is a key that authenticates nobody. It is
/// regenerated when the file is missing, so deleting it rotates it.
pub fn key_path(root: &Path) -> PathBuf {
    root.join("generated").join("spx").join("key")
}

/// The key this workspace uses, creating one if there is none.
pub fn key(root: &Path) -> Result<String> {
    let path = key_path(root);
    if let Ok(text) = std::fs::read_to_string(&path) {
        let existing = text.trim().to_string();
        if !existing.is_empty() {
            return Ok(existing);
        }
    }

    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|e| Error::new(Code::IoError, format!("generating an SPX key: {e}")))?;
    let fresh: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    crate::atomic::write(&path, &format!("{fresh}\n"))?;
    Ok(fresh)
}

// ---------------------------------------------------------------- the build

/// The shell the throwaway container runs.
///
/// A pure function so the thing that actually installs software on somebody's
/// machine is readable and testable rather than a string built at a call site.
///
/// `-e` on the first line and nothing swallowed: a build that fails half way
/// must fail, not leave an unloadable `.so` behind for the next run to trust.
/// The copy of `spx.so` is the **last** step for the same reason —
/// [`built`] treats its presence as "there is a build here".
pub fn build_script(source_ref: &str, source_url: &str) -> String {
    format!(
        "set -e\n\
         export DEBIAN_FRONTEND=noninteractive\n\
         apt-get update\n\
         apt-get install -y --no-install-recommends $PHPIZE_DEPS zlib1g-dev git ca-certificates\n\
         git clone --depth 1 -b {source_ref} {source_url} /tmp/spx\n\
         cd /tmp/spx\n\
         phpize\n\
         ./configure\n\
         make -j\"$(nproc)\"\n\
         rm -rf /out/assets\n\
         cp -r assets/web-ui /out/assets\n\
         cp modules/spx.so /out/spx.so\n\
         php -d extension=/out/spx.so -m | grep -qi '^SPX$'\n"
    )
}

/// `docker run --rm` arguments for the build.
///
/// The project's **own image**, so the ABI cannot disagree with the php-fpm
/// that will load the result. `--rm` because nothing about the build is worth
/// keeping except the two paths it copies out.
pub fn build_args(image: &str, out_dir: &Path, script: &str) -> Vec<String> {
    vec![
        "run".into(),
        "--rm".into(),
        "-v".into(),
        format!("{}:/out", out_dir.display()),
        image.into(),
        "sh".into(),
        "-c".into(),
        script.into(),
    ]
}

/// The image a build for this project runs in.
///
/// Derived rather than inspected: the build is offered before anything has
/// been started, and a project that has never run has no container to read an
/// image name off. `release.rs` builds the same string from the same prefix.
pub fn image_name(project: &str) -> String {
    format!("{}{project}:latest", crate::engine::CONTAINER_PREFIX)
}

// ------------------------------------------------------------------ the ini

/// The ini mounted into a project that has SPX switched on.
///
/// **`extension=`, not `zend_extension=`.** Measured: SPX is a normal
/// extension and loading it the other way fails outright.
///
/// **No profiling defaults are set here, and that is measured rather than
/// assumed.** php-spx has `spx.http_profiling_sampling_period` and its
/// siblings, and they look exactly like a place to put this workspace's
/// settings. They are not: `PHP_RINIT` reads the ini source **only when access
/// was not granted** — when a request carries no key, or the wrong one, and is
/// therefore not being profiled at all. A recording, from this app or from
/// SPX's own panel, always carries a key, so the ini is never consulted for it.
/// Writing the settings here would have produced a pane whose controls appeared
/// to do something and did nothing.
///
/// The whitelist is the private ranges rather than `*`. Behind this stack's own
/// reverse proxy the address in `REMOTE_ADDR` is the proxy's address on the
/// Docker network, not the browser's — so allowing loopback alone would refuse
/// every request that arrives the normal way. `spx_utils_ip_match` accepts an
/// exact address, IPv4 CIDR, or `*`, which is what makes this expressible
/// without opening it to everything. The key is the factor that actually
/// authenticates, and it is 16 random bytes that never reach the repository.
pub fn ini(key: &str) -> String {
    format!(
        "; Generated by StackVo Desktop — do not edit.\n\
         ;\n\
         ; Re-rendered before every compose command; edits here are lost.\n\
         extension={DIR}/spx.so\n\
         spx.data_dir={DATA_DIR}\n\
         spx.http_enabled=1\n\
         spx.http_key=\"{key}\"\n\
         spx.http_ip_whitelist=\"127.0.0.1,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16\"\n\
         spx.http_ui_assets_dir={DIR}/assets\n"
    )
}

// --------------------------------------------------------------- recording
//
// Three ways in, and one of them is not this app's.
//
// **The browser.** SPX's own control panel, opened from [`control_url`], with
// a checkbox that sets a cookie. That is the flow its documentation describes
// and the only one that can profile a page a person is *using* — a click, a
// form, a session. This app cannot set a cookie in somebody's browser and does
// not try to.
//
// **One request.** [`trigger_cookie`] is what makes a single request a
// recording, and the app sends that request itself. No browser is involved, so
// this is the one an assistant or a terminal can use: name a path, get a
// profile. It is php-spx's own documented trigger — the README profiles a page
// with `curl --cookie "SPX_ENABLED=1; SPX_KEY=dev"`.
//
// **One command.** [`trigger_env`] runs an artisan command, a queue worker or a
// test under the profiler. Different door, same reports.
//
// Lerd reaches the first case from its own window by injecting the cookie in
// the web server's configuration. That is not available here and would not be
// worth it if it were: the server config is generated under a byte-for-byte
// contract with the Bash CLI, and reaching into it to set a cookie would put
// this app's UI state inside a file another program owns.

/// The cookie that turns one request into a recording.
///
/// The settings ride along rather than being left to the ini. Both are read and
/// the cookie wins, which matters because **php-spx reads its ini once, when
/// PHP starts**: a recording sent from here does what this project's settings
/// say now, rather than what they said when the container last came up.
pub fn trigger_cookie(key: &str, config: &Config) -> String {
    format!(
        "SPX_ENABLED=1; SPX_KEY={key}; SPX_SAMPLING_PERIOD={}; SPX_BUILTINS={}",
        config.sampling_period,
        u8::from(config.builtins)
    )
}

/// The environment a command runs under to be profiled.
///
/// **`SPX_REPORT=full` is not optional.** php-spx defaults to `full` for a
/// request and to `fp` — a flat profile printed to standard error — for the
/// command line. Without it a CLI run profiles perfectly and writes no report,
/// which presents as a recording that did not happen.
pub fn trigger_env(config: &Config) -> Vec<(String, String)> {
    vec![
        ("SPX_ENABLED".into(), "1".into()),
        ("SPX_AUTO_START".into(), "1".into()),
        ("SPX_REPORT".into(), "full".into()),
        (
            "SPX_SAMPLING_PERIOD".into(),
            config.sampling_period.to_string(),
        ),
        ("SPX_BUILTINS".into(), u8::from(config.builtins).to_string()),
    ]
}

/// `docker exec` argv carrying an environment into the container.
///
/// Separate from [`crate::quickcmd::exec_argv`] rather than a flag on it: these
/// are `-e` pairs for the *container*, and the operation runner's `env` is the
/// environment of the `docker` client, which is a different thing at the same
/// call site.
pub fn record_argv(container: &str, argv: &[String], env: &[(String, String)]) -> Vec<String> {
    let mut out = vec!["exec".to_string()];
    for (name, value) in env {
        out.push("-e".to_string());
        out.push(format!("{name}={value}"));
    }
    out.push(container.to_string());
    out.extend(argv.iter().cloned());
    out
}

/// The URL a recording request is sent to.
///
/// The host is the project's own domain, read from its manifest. What a caller
/// names is the **path**, and it is checked rather than joined: a value opening
/// with `//` is a protocol-relative URL, and accepting one would turn a text
/// field on a pane into a way of making this app fetch somebody else's host
/// with a credential attached.
pub fn request_url(domain: &str, path: &str) -> Result<String> {
    let path = path.trim();
    let path = if path.is_empty() { "/" } else { path };

    let refuse = |why: &str| {
        Err(Error::new(
            Code::InvalidInput,
            format!("\"{path}\" is not a path on this site: {why}"),
        )
        .with_hint(crate::hints::SPX_RECORD_A_PATH))
    };

    if !path.starts_with('/') {
        return refuse("it has to start with /");
    }
    if path.starts_with("//") {
        return refuse("// names another host");
    }
    if path.contains('\\') || path.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return refuse("it has a space or a backslash in it");
    }

    Ok(format!("https://{domain}{path}"))
}

/// How long a recording request is given.
///
/// A profiled request is slower than the same request is normally, and the page
/// worth profiling is the slow one. Two minutes is longer than anything a
/// person would sit through and short enough that a hung request does not hold
/// a connection for the rest of the session.
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Send one recording request, and report what the site answered.
///
/// Three deliberate choices, each of which was a bug the first time it was not
/// made:
///
/// **The workspace's own CA is trusted, and nothing else changes.** The site is
/// `https://` — the proxy redirects `:80` to `:443` — and its certificate is
/// signed by the CA this app generated. Adding that one root is the difference
/// between verifying the certificate and turning verification off, which is
/// what `danger_accept_invalid_certs` would have done for every request this
/// process makes.
///
/// **No proxy.** `reqwest` is built here with `system-proxy` for the updater's
/// sake, and the reader that feature uses takes the host and port of a
/// corporate proxy without its exceptions list — so a request to a `.loc`
/// domain on a managed laptop would be sent to a proxy that has never heard of
/// it. `mail.rs` documents the same trap at length.
///
/// **Redirects are not followed.** A framework that answers `/` with a redirect
/// to `/login` would otherwise produce two recordings for one button, and the
/// second one would be the one the list showed.
pub async fn send(url: &str, cookie: &str) -> Result<u16> {
    let ca = crate::certs::ca_file();
    let pem = std::fs::read(&ca).map_err(|e| {
        Error::io(format!("reading {}", ca.display()), e)
            .with_hint(crate::hints::SPX_NEEDS_THE_LOCAL_CA)
    })?;
    let root = reqwest::Certificate::from_pem(&pem).map_err(|e| {
        Error::new(Code::InvalidInput, format!("reading {}: {e}", ca.display()))
            .with_hint(crate::hints::SPX_NEEDS_THE_LOCAL_CA)
    })?;

    let client = reqwest::Client::builder()
        .no_proxy()
        .add_root_certificate(root)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(REQUEST_TIMEOUT)
        .user_agent(concat!("stackvo-profiler/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| Error::new(Code::IoError, format!("preparing the request: {e}")))?;

    let response = client
        .get(url)
        .header(reqwest::header::COOKIE, cookie)
        .send()
        .await
        .map_err(|e| {
            Error::new(Code::IoError, format!("requesting {url}: {e}"))
                .with_hint(crate::hints::SPX_RECORD_NEEDS_THE_SITE)
        })?;

    Ok(response.status().as_u16())
}

/// The URL that opens SPX's own control panel for a site.
///
/// Its UI is served by the extension itself, from inside the application's own
/// vhost — so there is no port to publish and no second server to run. The key
/// is in the query string because that is the only place SPX reads it from on a
/// first visit; it sets a cookie afterwards.
pub fn control_url(domain: &str, key: &str) -> String {
    format!("https://{domain}/?SPX_KEY={key}&SPX_UI_URI=/")
}

/// Where one recording is read, in SPX's own viewer.
///
/// The flame graph, the call tree and the timeline are that project's work and
/// there is no reason to rebuild them here; what was missing was a way to reach
/// **this** report rather than the list. The shape is the panel's own — its
/// table links rows with `?SPX_UI_URI=/report.html&key=…` — with `SPX_KEY`
/// added so the link works before the panel has ever been opened.
pub fn report_url(domain: &str, key: &str, report: &str) -> String {
    format!("https://{domain}/?SPX_KEY={key}&SPX_UI_URI=/report.html&key={report}")
}

// -------------------------------------------------------------- the overlay

/// One project the overlay names.
pub struct Entry {
    pub service: String,
    /// Host path of the directory holding `spx.so` and `assets/`.
    pub build_host: String,
    /// Host path of the report directory.
    pub data_host: String,
    /// Host path of the generated ini.
    pub ini_host: String,
}

/// Render the overlay, or None when no project wants it.
///
/// None rather than an empty document: compose rejects a file whose `services`
/// map is empty, so "nothing to add" has to mean "no file".
pub fn overlay_yaml(entries: &[Entry]) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    let mut out = String::from(
        "# Generated by StackVo Desktop — do not edit.\n\
         #\n\
         # Mounts php-spx: the extension and its web UI read-only, the report\n\
         # directory writable, and the ini that loads it. Re-rendered before\n\
         # every compose command, so switching a project off removes it.\n\
         #\n\
         # NOTE: `stackvo up` from the Bash CLI does not layer this file.\n\
         services:\n",
    );

    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.service.cmp(&b.service));

    for entry in sorted {
        out.push_str(&format!("  {}:\n", entry.service));
        out.push_str("    volumes:\n");
        // Quoted: a Windows host path contains a colon, which is the separator
        // compose splits these on.
        out.push_str(&format!("      - \"{}:{DIR}:ro\"\n", entry.build_host));
        out.push_str(&format!("      - \"{}:{DATA_DIR}\"\n", entry.data_host));
        out.push_str(&format!(
            "      - \"{}:{INI_CONTAINER_PATH}:ro\"\n",
            entry.ini_host
        ));
    }

    Some(out)
}

// -------------------------------------------------------------- the reports

/// One recorded profile, as SPX's own metadata describes it.
///
/// Read from the `.json` half of the pair rather than from the file name, so
/// the list can say what a run cost without decompressing anything.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// SPX's own key, which is also the file stem of both halves.
    pub key: String,
    /// Unix seconds.
    pub recorded_at: i64,
    /// True for a CLI run — an artisan command, a queue worker, a test.
    pub cli: bool,
    /// The request, when it was one: `GET /api/health`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    /// The command line, when it was a CLI run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// How long the run took, in **microseconds**.
    ///
    /// The field it is read from is called `wall_time_ms`, and it is not
    /// milliseconds. Measured, because the name is convincing and being wrong
    /// about it is wrong by a factor of a thousand while still looking
    /// plausible: a script written to burn exactly 120 ms plus 60 ms produced
    /// `"wall_time_ms": 182837`, and the trace's own cumulative total for the
    /// same run was 182837191 — nanoseconds, a thousand of them per unit of
    /// this field. The pane had been rendering it as "182837 ms" for a run that
    /// took a fifth of a second.
    ///
    /// Kept in SPX's unit rather than converted here: a request that takes
    /// 700 µs is a real thing to profile, and rounding it into whole
    /// milliseconds on the way out throws that away for everything downstream.
    pub wall_time_us: u64,
    pub peak_memory: u64,
    pub call_count: u64,
    /// Bytes both halves take together.
    pub bytes: u64,
}

/// Parse one metadata document.
///
/// Tolerant by design: this is another project's format, read from a file it
/// wrote, and a field that moves should cost a column rather than the list.
pub fn parse_report(text: &str, bytes: u64) -> Option<Report> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let key = value.get("key")?.as_str()?.to_string();

    let string = |name: &str| {
        value
            .get(name)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty() && *v != "n/a")
            .map(str::to_string)
    };
    let number = |name: &str| value.get(name).and_then(|v| v.as_u64()).unwrap_or(0);

    // `cli` is `1` or `0` in the file rather than a boolean.
    let cli = value
        .get("cli")
        .map(|v| v.as_bool() == Some(true) || v.as_u64() == Some(1))
        .unwrap_or(false);

    let request = match (string("http_method"), string("http_request_uri")) {
        (Some(method), Some(uri)) => Some(format!("{method} {uri}")),
        (None, Some(uri)) => Some(uri),
        _ => None,
    };

    Some(Report {
        key,
        recorded_at: value.get("exec_ts").and_then(|v| v.as_i64()).unwrap_or(0),
        cli,
        request,
        command: string("cli_command_line"),
        // Named `_ms` in the file and holding microseconds. See the field.
        wall_time_us: number("wall_time_ms"),
        peak_memory: number("peak_memory_usage"),
        call_count: number("call_count"),
        bytes,
    })
}

/// Every recorded profile for one project, newest first.
pub fn list(root: &Path, project: &str) -> Vec<Report> {
    let dir = data_dir(root, project);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut out: Vec<Report> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };

        // Both halves, because "how much disk is this costing" is the question
        // the number answers and the trace is the larger of the two by orders
        // of magnitude.
        let json_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let trace_bytes = path
            .with_extension("txt.gz")
            .metadata()
            .map(|m| m.len())
            .unwrap_or(0);

        if let Some(report) = parse_report(&text, json_bytes + trace_bytes) {
            out.push(report);
        }
    }

    out.sort_by_key(|report| std::cmp::Reverse(report.recorded_at));
    out
}

/// Refuse anything that is not one of SPX's own report keys.
///
/// A key comes back from a screen and ends up in a path, so it is checked in
/// one place and every caller goes through it rather than each deciding for
/// itself. SPX's keys are alphanumerics, dashes and underscores.
pub fn check_key(key: &str) -> Result<()> {
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(Error::new(
            Code::InvalidInput,
            format!("\"{key}\" is not a report key"),
        ));
    }
    Ok(())
}

/// Delete both halves of one report.
///
/// The key is checked rather than joined: it comes back from a screen and ends
/// up in a path, and SPX's own keys are `[A-Za-z0-9_-]` — so anything else is
/// refused rather than resolved.
pub fn remove(root: &Path, project: &str, key: &str) -> Result<()> {
    check_key(key)?;

    let dir = data_dir(root, project);
    let _ = std::fs::remove_file(dir.join(format!("{key}.json")));
    let _ = std::fs::remove_file(dir.join(format!("{key}.txt.gz")));
    Ok(())
}

/// Remove every report, returning how many and how many bytes.
pub fn clear(root: &Path, project: &str) -> Result<(usize, u64)> {
    let reports = list(root, project);
    let bytes = reports.iter().map(|r| r.bytes).sum();
    let count = reports.len();

    for report in reports {
        remove(root, project, &report.key)?;
    }
    Ok((count, bytes))
}

// ------------------------------------------------------------------ the trace
//
// What the list could not say.
//
// A report row is wall time, peak memory and a call count — enough to pick the
// slow run out of twenty, and nothing at all about **why** it was slow. That
// answer is in the other half of the pair, the `.txt.gz`, and until this it was
// only readable in SPX's own web UI: a browser, a key, and a person.
//
// So it is read here as well. The format was established by recording in this
// repository's own project image and reading the file back, and it is three
// parts:
//
// ```text
// [events]
// 12 1 1043.7118 262144        <- function 12 entered, cumulative metrics
// 12 0 1102.3390 262144        <- and left
// [functions]
// {main}
// App\Http\Kernel::handle
// ```
//
// Two things about it decide the whole implementation. The metric values are
// **cumulative totals**, not per-event costs, so time is attributed by the gap
// between consecutive events — to whatever was on top of the stack across it.
// And the function table is written **last**, after the events that index into
// it, so the replay cannot resolve a name while it runs and does not try to:
// it accumulates by index and puts the names on at the end.

/// One function in a recording.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotspot {
    pub function: String,
    pub calls: u64,
    /// Time in the function's own body, in microseconds.
    pub exclusive_us: f64,
    pub exclusive_percent: f64,
    /// Time in it and everything it called, in microseconds.
    pub inclusive_us: f64,
    pub inclusive_percent: f64,
}

/// Where one recording spent its time.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub key: String,
    /// SPX's own total for the run, in microseconds — the figure the shares are
    /// scaled to, and the same one the list shows beside this recording.
    pub wall_time_us: u64,
    pub call_count: u64,
    /// Distinct functions the trace names.
    pub functions: usize,
    /// Events actually replayed.
    pub events: u64,
    /// The trace was longer than [`MAX_EVENTS`] and the tail was not read.
    ///
    /// Said rather than hidden: the shares below are then about the start of
    /// the run, which is a true answer to a smaller question and a false one to
    /// the question the screen appears to be asking.
    pub truncated: bool,
    pub hotspots: Vec<Hotspot>,
}

/// The running totals a replay builds, by function index.
#[derive(Default)]
struct Totals {
    exclusive: Vec<f64>,
    inclusive: Vec<f64>,
    calls: Vec<u64>,
    /// How many frames of this function are on the stack right now.
    ///
    /// Recursion is why: a function that calls itself is entered twice and left
    /// twice, and adding its inclusive time on both exits counts the inner call
    /// in the outer one. Inclusive time is added only when the outermost frame
    /// leaves.
    depth: Vec<u32>,
}

impl Totals {
    fn reach(&mut self, index: usize) {
        if index >= self.calls.len() {
            let size = index + 1;
            self.exclusive.resize(size, 0.0);
            self.inclusive.resize(size, 0.0);
            self.calls.resize(size, 0);
            self.depth.resize(size, 0);
        }
    }
}

/// Which of a report's metrics is wall time.
///
/// The event lines carry only the metrics that were enabled, in the order the
/// metadata lists them — so the column to read is a property of the recording,
/// not a constant. `wt` is on by default and first, and falling back to column
/// zero when it is absent is better than refusing: a recording made with the
/// metrics changed still has a first metric, and a share of it is still where
/// the time went.
pub fn wall_metric(metrics: &[String]) -> usize {
    metrics.iter().position(|m| m == "wt").unwrap_or(0)
}

/// Replay one trace into per-function totals.
///
/// Written against the text rather than the file so the format — the part of
/// this that is another project's and could move under it — is testable
/// without a container, a build and a recording.
pub fn replay(text: &str, metric: usize, limit: u64) -> (Vec<Hotspot>, u64, bool, f64) {
    /// A guard on the index a line may name. Nothing legitimate approaches it;
    /// a corrupt or hostile file naming `999999999` would otherwise ask this
    /// process for eight gigabytes of zeroes.
    const MAX_FUNCTIONS: usize = 1_000_000;

    #[derive(PartialEq)]
    enum Section {
        Before,
        Events,
        Functions,
    }

    let mut section = Section::Before;
    let mut totals = Totals::default();
    let mut names: Vec<String> = Vec::new();
    let mut stack: Vec<(usize, f64)> = Vec::new();
    let mut previous = 0.0f64;
    let mut events = 0u64;
    let mut truncated = false;

    for line in text.lines() {
        match line.trim_end() {
            "[events]" => {
                section = Section::Events;
                continue;
            }
            "[functions]" => {
                section = Section::Functions;
                continue;
            }
            _ => {}
        }

        match section {
            Section::Before => continue,
            // The names arrive after the events, in index order.
            Section::Functions => names.push(line.trim_end().to_string()),
            Section::Events => {
                if events >= limit {
                    // Kept scanning rather than broken out of: the function
                    // table is below, and stopping here would leave every
                    // hotspot called `#417`.
                    truncated = true;
                    continue;
                }

                let mut parts = line.split_ascii_whitespace();
                let Some(index) = parts.next().and_then(|v| v.parse::<usize>().ok()) else {
                    continue;
                };
                let Some(start) = parts.next().map(|v| v == "1") else {
                    continue;
                };
                let Some(value) = parts.nth(metric).and_then(|v| v.parse::<f64>().ok()) else {
                    continue;
                };
                if index >= MAX_FUNCTIONS || !value.is_finite() {
                    continue;
                }
                totals.reach(index);

                // Cumulative totals, so the cost of anything is the gap since
                // the last event — and it belongs to whatever was running
                // across that gap, which is the frame on top of the stack.
                let step = (value - previous).max(0.0);
                previous = value;
                if let Some(&(top, _)) = stack.last() {
                    totals.exclusive[top] += step;
                }

                if start {
                    totals.calls[index] += 1;
                    totals.depth[index] += 1;
                    stack.push((index, value));
                } else if let Some((left, entered)) = stack.pop() {
                    totals.depth[left] = totals.depth[left].saturating_sub(1);
                    if totals.depth[left] == 0 {
                        totals.inclusive[left] += (value - entered).max(0.0);
                    }
                }
                events += 1;
            }
        }
    }

    // A trace cut short leaves frames open. Their inclusive time is what has
    // elapsed so far rather than nothing, which is what an unclosed `{main}`
    // would otherwise report.
    while let Some((left, entered)) = stack.pop() {
        totals.depth[left] = totals.depth[left].saturating_sub(1);
        if totals.depth[left] == 0 {
            totals.inclusive[left] += (previous - entered).max(0.0);
        }
    }

    let total = previous;
    let share = |value: f64| {
        if total > 0.0 {
            value / total * 100.0
        } else {
            0.0
        }
    };

    let mut hotspots: Vec<Hotspot> = (0..totals.calls.len())
        .filter(|&index| totals.calls[index] > 0 || totals.exclusive[index] > 0.0)
        .map(|index| Hotspot {
            function: names
                .get(index)
                .filter(|name| !name.is_empty())
                .cloned()
                // A trace whose table is shorter than its events is damaged,
                // not unreadable: the index is still an identity.
                .unwrap_or_else(|| format!("#{index}")),
            calls: totals.calls[index],
            exclusive_us: 0.0,
            exclusive_percent: share(totals.exclusive[index]),
            inclusive_us: 0.0,
            inclusive_percent: share(totals.inclusive[index]),
        })
        .collect();

    hotspots.sort_by(|a, b| {
        b.exclusive_percent
            .partial_cmp(&a.exclusive_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.function.cmp(&b.function))
    });

    (hotspots, events, truncated, total)
}

/// Read one recording and say where its time went.
///
/// The absolute times are **derived from the metadata's own total**, not from
/// the raw metric values. The raw unit is nanoseconds and the metadata's is
/// microseconds — measured, and neither is written down anywhere in php-spx —
/// so a share of the run scaled by the number SPX itself puts on that run is
/// both correct and unable to drift away from the figure shown beside it in the
/// list, whatever either unit does next.
pub fn analyse(root: &Path, project: &str, key: &str, limit: usize) -> Result<Analysis> {
    check_key(key)?;

    let dir = data_dir(root, project);
    let metadata = std::fs::read_to_string(dir.join(format!("{key}.json")))
        .map_err(|_| Error::not_found(format!("report {key}")))?;
    let document: serde_json::Value = serde_json::from_str(&metadata)
        .map_err(|e| Error::new(Code::InvalidInput, format!("reading report {key}: {e}")))?;

    let metrics: Vec<String> = document
        .get("enabled_metrics")
        .and_then(|v| v.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let trace = dir.join(format!("{key}.txt.gz"));
    let file = std::fs::File::open(&trace).map_err(|e| {
        Error::io(format!("reading {}", trace.display()), e)
            .with_hint(crate::hints::SPX_TRACE_IS_MISSING)
    })?;

    use std::io::Read as _;
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(std::io::BufReader::new(file))
        .take(MAX_TRACE_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::io(format!("decompressing {}", trace.display()), e))?;

    let hit_the_cap = bytes.len() as u64 >= MAX_TRACE_BYTES;
    // Lossy rather than checked: the cap can land inside a multi-byte
    // character, and one replacement mark in one function name is not a reason
    // to refuse the whole report.
    let text = String::from_utf8_lossy(&bytes);

    let (mut hotspots, events, truncated, _) = replay(&text, wall_metric(&metrics), MAX_EVENTS);
    let functions = hotspots.len();
    hotspots.truncate(limit);

    // Named `_ms` in the file and holding microseconds; see `Report`.
    let wall_time_us = document
        .get("wall_time_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let whole = wall_time_us as f64;
    for hotspot in &mut hotspots {
        hotspot.exclusive_us = hotspot.exclusive_percent / 100.0 * whole;
        hotspot.inclusive_us = hotspot.inclusive_percent / 100.0 * whole;
    }

    Ok(Analysis {
        key: key.to_string(),
        wall_time_us,
        call_count: document
            .get("call_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        functions,
        events,
        truncated: truncated || hit_the_cap,
        hotspots,
    })
}

// ------------------------------------------------------------------- state

/// The per-project switch, beside the other `.stackvo/` settings.
///
/// A file rather than a manifest key, for the reason `xdebug::ModeConfig`
/// gives: the manifest schema is `additionalProperties: false`, and a file
/// under `.stackvo/` travels with a clone.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub enabled: bool,
    /// Microseconds between samples. `0` records every call.
    ///
    /// Defaulted per field rather than by `Default` alone, so a `spx.json`
    /// written before this setting existed reads back as the sampled default
    /// instead of as `0` — which is a different profiler, not a missing value.
    #[serde(default = "default_sampling_period")]
    pub sampling_period: u32,
    /// Profile PHP's own functions too.
    ///
    /// Off by default: it roughly doubles a trace, and the answer is usually a
    /// function in the project rather than `preg_match`. On is what makes the
    /// other case findable.
    #[serde(default)]
    pub builtins: bool,
}

fn default_sampling_period() -> u32 {
    SAMPLING_PERIOD_US
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            sampling_period: SAMPLING_PERIOD_US,
            builtins: false,
        }
    }
}

pub fn config_path(root: &Path, project: &str) -> PathBuf {
    crate::workspace::projects_root(root)
        .unwrap_or_default()
        .join(project)
        .join(crate::phpini::CONFIG_DIR)
        .join(CONFIG_FILE)
}

pub fn read_config(root: &Path, project: &str) -> Config {
    std::fs::read_to_string(config_path(root, project))
        .ok()
        .and_then(|text| serde_json::from_str::<Config>(&text).ok())
        .unwrap_or_default()
}

pub fn write_config(root: &Path, project: &str, config: &Config) -> Result<()> {
    let path = config_path(root, project);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|e| Error::new(Code::IoError, format!("serialising the SPX config: {e}")))?;
    crate::atomic::write(&path, &format!("{text}\n"))
}

// -------------------------------------------------------------------- sync

/// Every project that has SPX switched on **and** a build to mount.
///
/// Both, because an entry naming a directory with no `spx.so` in it mounts an
/// empty directory over `conf.d`'s extension path and every request then fails
/// to start PHP. The switch is a request; the build is what makes it possible.
fn entries(root: &Path) -> Vec<Entry> {
    let generated =
        std::fs::read_to_string(root.join("generated").join("docker-compose.projects.yml"))
            .unwrap_or_default();
    let services = crate::xdebug::generated_services(&generated);

    let Some(projects) = crate::workspace::projects_root(root) else {
        return Vec::new();
    };
    let Ok(dirs) = std::fs::read_dir(&projects) else {
        return Vec::new();
    };
    let Ok(key) = key(root) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for dir in dirs.flatten() {
        let path = dir.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Naming a service the generator did not emit declares one with neither
        // an image nor a build context, and compose then refuses every command
        // against the whole stack.
        if !path.is_dir() || !services.iter().any(|s| s == name) {
            continue;
        }
        let config = read_config(root, name);
        if !config.enabled {
            continue;
        }

        let Ok(manifest) = crate::manifest::read(&path.join("stackvo.json"), name) else {
            continue;
        };
        let Some(php) = manifest.php.as_ref() else {
            continue;
        };
        if !built(root, &php.version) {
            continue;
        }

        // SPX does not create its own output directory and says nothing when it
        // is missing — the same trap `xdebug::ensure_output_dir` documents,
        // found the same way.
        let data = data_dir(root, name);
        if std::fs::create_dir_all(&data).is_err() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o777));
        }

        let ini_host = ini_path(root, name);
        if let Some(parent) = ini_host.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                continue;
            }
        }
        if crate::atomic::write(&ini_host, &ini(&key)).is_err() {
            continue;
        }

        out.push(Entry {
            service: name.to_string(),
            build_host: build_dir(root, &php.version).display().to_string(),
            data_host: data.display().to_string(),
            ini_host: ini_host.display().to_string(),
        });
    }

    out
}

/// Re-render the overlay from the manifests, and report whether it now exists.
///
/// Derived before every compose command rather than stored, for the reason
/// `xdebug::sync` gives at length: an overlay naming a project that has since
/// been deleted declares a service with neither an image nor a build context,
/// and compose then refuses **every** command, including the `down` that would
/// have cleared it.
pub fn sync(root: &Path) -> bool {
    let path = overlay_path(root);

    match overlay_yaml(&entries(root)) {
        Some(yaml) => {
            if let Some(parent) = path.parent() {
                if std::fs::create_dir_all(parent).is_err() {
                    return false;
                }
            }
            match crate::atomic::write(&path, &yaml) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "could not write the SPX overlay");
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

// ------------------------------------------------------------------ status

/// What is true for SPX on one project.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// False for anything that is not a PHP project.
    pub supported: bool,
    /// The switch in `.stackvo/spx.json`.
    pub enabled: bool,
    /// There is an extension built for this project's PHP version.
    pub built: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub php_version: Option<String>,
    /// The image a build would run in, when the project has been built once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Whether the running container actually has the extension mounted.
    ///
    /// Separate from `enabled` for the reason `xdebug::XdebugStatus::active`
    /// is: mounts are fixed when a container is created, so a switch flipped
    /// after it started has not reached it.
    pub active: Option<bool>,
    pub running: bool,
    /// Microseconds between samples; `0` records every call.
    pub sampling_period: u32,
    /// PHP's own functions are profiled too.
    pub builtins: bool,
    /// The site this project answers on, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// The URL that opens SPX's own control panel, when the project has a
    /// domain to reach it on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_url: Option<String>,
    /// What a report key is appended to, to open that recording in SPX's viewer.
    ///
    /// A base rather than a URL per row: the difference between the rows is the
    /// key and nothing else, and every one of these carries the profiler key —
    /// so one field is one thing for a surface that must not return credentials
    /// to strip, instead of one per recording.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_base: Option<String>,
    /// Xdebug is recording at the same time.
    ///
    /// Two profilers hooking one engine is not a supported configuration in
    /// either project, and the symptom is numbers that are wrong rather than
    /// an error — so it is reported rather than prevented, because which one to
    /// turn off is the user's call.
    pub xdebug_conflict: bool,
    pub reports: Vec<Report>,
    pub bytes: u64,
    pub directory: String,
}

/// Read it. Never writes.
pub async fn status(root: &Path, project: &str) -> Result<Status> {
    let dir = crate::workspace::project_dir(root, project)?;
    let file = dir.join("stackvo.json");
    if !file.is_file() {
        return Err(Error::not_found(format!("project {project}")));
    }
    let manifest = crate::manifest::read(&file, project)?;

    let supported = manifest.runtime == "php";
    let php_version = manifest.php.as_ref().map(|p| p.version.clone());
    let reports = list(root, project);

    let details = crate::engine::inspect(project).await.ok();
    let xdebug_mode = crate::xdebug::read_mode(root, project);
    let config = read_config(root, project);

    // Read once. Asking for it twice would create it on the first call and
    // return a different string on the second if the first had failed to write.
    let profiler_key = key(root).ok();

    Ok(Status {
        supported,
        enabled: supported && config.enabled,
        built: php_version.as_deref().is_some_and(|v| built(root, v)),
        image: details.as_ref().and_then(|d| d.image.clone()),
        active: details
            .as_ref()
            .map(|d| d.mounts.iter().any(|m| m.destination == DIR)),
        running: details.as_ref().is_some_and(|d| d.running),
        sampling_period: config.sampling_period,
        builtins: config.builtins,
        domain: manifest.domain.clone(),
        control_url: manifest
            .domain
            .as_deref()
            .and_then(|domain| profiler_key.as_ref().map(|k| control_url(domain, k))),
        view_base: manifest.domain.as_deref().and_then(|domain| {
            profiler_key
                .as_ref()
                .map(|k| report_url(domain, k, "").to_string())
        }),
        xdebug_conflict: supported
            && manifest.php.as_ref().is_some_and(|p| p.xdebug)
            && xdebug_mode.records_to_disk(),
        bytes: reports.iter().map(|r| r.bytes).sum(),
        directory: data_dir(root, project).display().to_string(),
        reports,
        php_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mistake this module was written against on its first run, kept as a
    /// test because the documentation implies the opposite and every other
    /// profiler in this tree is a Zend extension.
    #[test]
    fn it_is_loaded_as_a_normal_extension_not_a_zend_one() {
        let text = ini("abc");
        assert!(text.contains("extension=/usr/local/etc/stackvo-spx/spx.so"));
        assert!(
            !text.contains("zend_extension"),
            "SPX fails to load as a Zend extension: {text}"
        );
    }

    /// Behind this stack's own proxy the address SPX sees is the proxy's, on
    /// the Docker network — so loopback alone would refuse every request that
    /// arrives the normal way, and `*` is not needed to fix it.
    #[test]
    fn the_whitelist_covers_the_docker_network_without_opening_it_to_everything() {
        let text = ini("abc");
        assert!(text.contains("172.16.0.0/12"), "{text}");
        assert!(text.contains("10.0.0.0/8"), "{text}");
        assert!(text.contains("127.0.0.1"), "{text}");
        assert!(
            !text.contains("\"*\""),
            "the whitelist was opened to everything: {text}"
        );
    }

    /// Every prefix SPX's matcher can actually evaluate: it takes an exact
    /// address, IPv4 CIDR with a mask between 1 and 31, or `*`. A `/32` or an
    /// IPv6 range would parse here and be rejected there, silently.
    #[test]
    fn every_whitelist_entry_is_a_shape_spx_can_match() {
        let text = ini("abc");
        let list = text
            .lines()
            .find_map(|line| line.strip_prefix("spx.http_ip_whitelist="))
            .expect("the ini sets a whitelist")
            .trim_matches('"');

        for entry in list.split(',') {
            match entry.split_once('/') {
                None => assert!(
                    entry.parse::<std::net::Ipv4Addr>().is_ok(),
                    "{entry} is not an IPv4 address"
                ),
                Some((address, mask)) => {
                    assert!(
                        address.parse::<std::net::Ipv4Addr>().is_ok(),
                        "{entry} has no IPv4 address"
                    );
                    let bits: u8 = mask.parse().expect("a numeric mask");
                    assert!((1..=31).contains(&bits), "{entry} has an unusable mask");
                }
            }
        }
    }

    /// Derived, because the build is offered before anything has been started
    /// and a project that has never run has no container to read an image off.
    #[test]
    fn the_image_is_the_one_the_generator_tags() {
        assert_eq!(image_name("shop"), "stackvo-shop:latest");
        assert_eq!(image_name("parser.ajans"), "stackvo-parser.ajans:latest");
    }

    #[test]
    fn the_key_is_in_the_ini_and_in_the_url() {
        assert!(ini("s3cret").contains("spx.http_key=\"s3cret\""));
        assert_eq!(
            control_url("shop.loc", "s3cret"),
            "https://shop.loc/?SPX_KEY=s3cret&SPX_UI_URI=/"
        );
    }

    /// The extension is copied **last**, because `built` treats its presence as
    /// "there is a usable build here" — so a build that failed after the assets
    /// were copied must not look finished.
    #[test]
    fn the_build_copies_the_extension_last_and_fails_loudly() {
        let script = build_script(SOURCE_REF, SOURCE_URL);

        assert!(script.starts_with("set -e\n"), "{script}");
        let assets = script
            .find("cp -r assets/web-ui")
            .expect("assets are copied");
        let module = script
            .find("cp modules/spx.so")
            .expect("the module is copied");
        assert!(
            assets < module,
            "the extension was copied before the assets"
        );

        // And the build proves the artefact loads before it is left behind.
        assert!(
            script.contains("php -d extension=/out/spx.so -m"),
            "{script}"
        );
    }

    /// The build runs in a throwaway container from the project's own image:
    /// an extension has to match the ABI of the binary that loads it, and the
    /// running container is not the place to install a compiler.
    #[test]
    fn the_build_is_a_throwaway_container_of_the_projects_own_image() {
        let args = build_args(
            "stackvo-shop:latest",
            Path::new("/w/generated/spx/8.4"),
            "x",
        );

        assert_eq!(args[0], "run");
        assert!(args.contains(&"--rm".to_string()), "{args:?}");
        assert!(
            args.contains(&"/w/generated/spx/8.4:/out".to_string()),
            "{args:?}"
        );
        assert!(
            args.contains(&"stackvo-shop:latest".to_string()),
            "{args:?}"
        );
    }

    /// Read off a report this repository's own image produced, field for
    /// field — the shape is another project's and is not guessed at.
    #[test]
    fn a_real_report_reads_back() {
        let text = r#"{
            "key": "spx-full-20260822_191647-2eb6751e0873-11-1032825502",
            "exec_ts": 1787426207,
            "host_name": "2eb6751e0873",
            "process_pid": 11,
            "process_pwd": "/var/www/html",
            "cli": 1,
            "cli_command_line": "Standard input code",
            "http_request_uri": "n/a",
            "http_method": "n/a",
            "wall_time_ms": 736,
            "peak_memory_usage": 1808984,
            "called_function_count": 1,
            "call_count": 1,
            "enabled_metrics": ["wt", "zm"]
        }"#;

        let report = parse_report(text, 624).expect("it parses");
        assert_eq!(
            report.key,
            "spx-full-20260822_191647-2eb6751e0873-11-1032825502"
        );
        assert_eq!(report.recorded_at, 1787426207);
        assert!(report.cli, "`cli` is 1 rather than true in this format");
        assert_eq!(report.command.as_deref(), Some("Standard input code"));
        // 736 microseconds, not 736 milliseconds — see the field.
        assert_eq!(report.wall_time_us, 736);
        assert_eq!(report.peak_memory, 1808984);
        assert_eq!(report.bytes, 624);

        // `n/a` is SPX's way of saying "not a request", and carrying it into a
        // column would put "n/a n/a" on screen for every CLI run.
        assert_eq!(report.request, None);
    }

    #[test]
    fn a_request_report_names_the_request() {
        let text = r#"{ "key": "k", "exec_ts": 1, "cli": 0,
            "http_method": "GET", "http_request_uri": "/api/health",
            "wall_time_ms": 12, "peak_memory_usage": 2, "call_count": 3 }"#;

        let report = parse_report(text, 0).expect("it parses");
        assert!(!report.cli);
        assert_eq!(report.request.as_deref(), Some("GET /api/health"));
    }

    #[test]
    fn a_document_that_is_not_a_report_is_skipped_rather_than_fatal() {
        assert!(parse_report("not json", 0).is_none());
        assert!(parse_report(r#"{"no":"key"}"#, 0).is_none());
    }

    /// A key comes back from a screen and ends up in a path. SPX's own keys are
    /// alphanumerics, dashes and underscores; anything else is refused rather
    /// than resolved.
    #[test]
    fn a_key_that_is_not_one_never_reaches_the_filesystem() {
        let root = Path::new("/w");
        assert!(remove(root, "shop", "../../etc/passwd").is_err());
        assert!(remove(root, "shop", "a/b").is_err());
        assert!(remove(root, "shop", "").is_err());
        // A real one is accepted, and removing what is not there is not an error.
        assert!(remove(root, "shop", "spx-full-20260822_191647-abc-11-99").is_ok());
    }

    #[test]
    fn the_overlay_mounts_the_build_read_only_and_the_reports_writable() {
        let yaml = overlay_yaml(&[Entry {
            service: "shop".into(),
            build_host: "/w/generated/spx/8.4".into(),
            data_host: "/w/logs/projects/shop/spx".into(),
            ini_host: "/w/generated/spx/shop.ini".into(),
        }])
        .expect("one project renders");

        assert!(yaml.contains("  shop:\n"), "{yaml}");
        assert!(
            yaml.contains(&format!("/w/generated/spx/8.4:{DIR}:ro")),
            "the build must be read-only: {yaml}"
        );
        assert!(
            yaml.contains(&format!("/w/logs/projects/shop/spx:{DATA_DIR}\"")),
            "the reports must be writable: {yaml}"
        );
        assert!(yaml.contains(INI_CONTAINER_PATH), "{yaml}");
    }

    /// Compose rejects a file whose `services` map is empty, so "nothing to
    /// add" has to mean "no file" rather than an empty one.
    #[test]
    fn no_project_renders_no_document() {
        assert!(overlay_yaml(&[]).is_none());
    }

    #[test]
    fn projects_are_rendered_in_a_stable_order() {
        let entry = |name: &str| Entry {
            service: name.into(),
            build_host: "/b".into(),
            data_host: "/d".into(),
            ini_host: "/i".into(),
        };
        let one = overlay_yaml(&[entry("b"), entry("a")]).unwrap();
        let two = overlay_yaml(&[entry("a"), entry("b")]).unwrap();
        assert_eq!(one, two);
    }

    /// Both halves, because an interrupted build leaves an extension that
    /// loads and a control panel that is a blank page.
    #[test]
    fn a_half_finished_build_does_not_count_as_one() {
        let dir = std::env::temp_dir().join(format!("stackvo-spx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(build_dir(&dir, "8.4")).unwrap();

        assert!(!built(&dir, "8.4"));
        std::fs::write(extension_path(&dir, "8.4"), "x").unwrap();
        assert!(!built(&dir, "8.4"), "the assets are missing");
        std::fs::create_dir_all(assets_path(&dir, "8.4")).unwrap();
        assert!(built(&dir, "8.4"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Keyed by PHP version rather than by project: the artefact is a function
    /// of the interpreter, so two projects on 8.4 share one build.
    #[test]
    fn the_build_is_shared_by_php_version() {
        let root = Path::new("/w");
        assert_eq!(build_dir(root, "8.4"), build_dir(root, "8.4"));
        assert_ne!(build_dir(root, "8.4"), build_dir(root, "8.2"));
        assert!(build_dir(root, "8.4").ends_with("generated/spx/8.4"));
    }

    // ------------------------------------------------------------ recording

    /// The host is the project's. The path is the caller's, and a path that
    /// opens with `//` is a URL to somebody else's host — which would make a
    /// text field on a pane into a way of sending this app, with the profiler
    /// key attached, wherever the person typing was told to.
    #[test]
    fn a_path_that_names_another_host_is_refused() {
        assert_eq!(
            request_url("shop.loc", "/checkout").unwrap(),
            "https://shop.loc/checkout"
        );
        assert_eq!(
            request_url("shop.loc", "/api/orders?page=2").unwrap(),
            "https://shop.loc/api/orders?page=2"
        );

        // Empty means the front page, which is what a person leaving the field
        // alone meant.
        assert_eq!(request_url("shop.loc", "  ").unwrap(), "https://shop.loc/");

        for hostile in [
            "//evil.example",
            "https://evil.example",
            "evil.example",
            "/a b",
            "/a\\b",
            "/a\nHost: evil.example",
        ] {
            assert!(
                request_url("shop.loc", hostile).is_err(),
                "{hostile} was accepted"
            );
        }
    }

    /// php-spx's own documented trigger, so a recording this app starts and one
    /// a person starts in the browser go through the same door.
    #[test]
    fn the_cookie_carries_the_key_and_the_settings() {
        let config = Config {
            enabled: true,
            sampling_period: 250,
            builtins: true,
        };
        let cookie = trigger_cookie("s3cret", &config);

        assert!(cookie.contains("SPX_ENABLED=1"), "{cookie}");
        assert!(cookie.contains("SPX_KEY=s3cret"), "{cookie}");
        // Carried rather than left to the ini, because PHP reads its ini once,
        // when it starts: without these the recording would use whatever the
        // settings said when the container last came up.
        assert!(cookie.contains("SPX_SAMPLING_PERIOD=250"), "{cookie}");
        assert!(cookie.contains("SPX_BUILTINS=1"), "{cookie}");
    }

    /// The one that is not optional. php-spx defaults to a flat profile on
    /// standard error for the command line, so a CLI run without this profiles
    /// perfectly and writes no file — which presents as a recording that did
    /// not happen.
    #[test]
    fn a_command_is_told_to_write_a_full_report() {
        let env = trigger_env(&Config::default());
        let get = |name: &str| {
            env.iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
        };

        assert_eq!(get("SPX_REPORT"), Some("full"));
        assert_eq!(get("SPX_ENABLED"), Some("1"));
        assert_eq!(get("SPX_AUTO_START"), Some("1"));
        assert_eq!(
            get("SPX_SAMPLING_PERIOD"),
            Some(SAMPLING_PERIOD_US.to_string().as_str())
        );
    }

    /// `-e` belongs to `docker exec` and has to be in front of the container,
    /// not in front of the program.
    #[test]
    fn the_environment_reaches_the_container_and_not_the_command() {
        let argv = record_argv(
            "stackvo-shop",
            &["php".into(), "artisan".into(), "migrate".into()],
            &[("SPX_ENABLED".into(), "1".into())],
        );

        assert_eq!(
            argv,
            vec![
                "exec",
                "-e",
                "SPX_ENABLED=1",
                "stackvo-shop",
                "php",
                "artisan",
                "migrate"
            ]
        );
    }

    /// The shape SPX's own panel links its rows with, plus the key so the link
    /// works before the panel has ever been opened.
    #[test]
    fn a_report_is_reachable_on_its_own() {
        assert_eq!(
            report_url("shop.loc", "s3cret", "spx-full-1-abc"),
            "https://shop.loc/?SPX_KEY=s3cret&SPX_UI_URI=/report.html&key=spx-full-1-abc"
        );
    }

    /// php-spx's own default period is `0` — every call — which is a tracing
    /// profiler with the cost this module exists to avoid. The pane says "the
    /// profiler you can leave on", and this is what makes that true.
    #[test]
    fn a_recording_is_sampled_unless_somebody_asks_for_every_call() {
        assert_eq!(Config::default().sampling_period, SAMPLING_PERIOD_US);
        assert_ne!(SAMPLING_PERIOD_US, 0);

        // And it reaches the recording through the request, not through the
        // ini — php-spx reads its ini profiling settings only for a request it
        // is NOT profiling. See `ini`.
        assert!(!ini("k").contains("http_profiling"), "{}", ini("k"));
        assert!(trigger_cookie("k", &Config::default())
            .contains(&format!("SPX_SAMPLING_PERIOD={SAMPLING_PERIOD_US}")));
    }

    /// A `spx.json` written before the setting existed. Reading it back as `0`
    /// would silently turn a sampled profiler into a tracing one.
    #[test]
    fn a_config_from_before_the_setting_reads_back_sampled() {
        let config: Config = serde_json::from_str(r#"{"enabled":true}"#).expect("it parses");
        assert!(config.enabled);
        assert_eq!(config.sampling_period, SAMPLING_PERIOD_US);
        assert!(!config.builtins);
    }

    // ---------------------------------------------------------------- traces

    /// One call inside another, in the format a real recording uses: cumulative
    /// totals, and the function names written after the events that index them.
    #[test]
    fn time_is_attributed_to_whatever_was_running_across_the_gap() {
        let trace = "\
[events]
0 1 0.0000 100
1 1 10.0000 100
1 0 30.0000 100
0 0 40.0000 100
[functions]
{main}
App::slow
";
        let (hotspots, events, truncated, total) = replay(trace, 0, MAX_EVENTS);

        assert_eq!(events, 4);
        assert!(!truncated);
        assert_eq!(total, 40.0);

        let of = |name: &str| {
            hotspots
                .iter()
                .find(|spot| spot.function == name)
                .unwrap_or_else(|| panic!("{name} is missing from {hotspots:?}"))
        };

        // 10 before the inner call and 10 after it, out of 40.
        assert_eq!(of("{main}").exclusive_percent, 50.0);
        assert_eq!(of("{main}").inclusive_percent, 100.0);
        // The inner call's own 20.
        assert_eq!(of("App::slow").exclusive_percent, 50.0);
        assert_eq!(of("App::slow").inclusive_percent, 50.0);
        assert_eq!(of("App::slow").calls, 1);
    }

    /// A function that calls itself is entered twice and left twice. Adding its
    /// inclusive time on both exits counts the inner call inside the outer one
    /// and reports a function that held 166% of the run.
    #[test]
    fn a_recursive_call_does_not_count_itself_twice() {
        let trace = "\
[events]
0 1 0.0000
0 1 10.0000
0 0 20.0000
0 0 30.0000
[functions]
rec
";
        let (hotspots, _, _, _) = replay(trace, 0, MAX_EVENTS);

        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].calls, 2);
        assert_eq!(hotspots[0].exclusive_percent, 100.0);
        assert_eq!(
            hotspots[0].inclusive_percent, 100.0,
            "the inner call was counted inside the outer one"
        );
    }

    /// The events carry only the metrics that were enabled, in the order the
    /// metadata lists them — so which column is wall time is a property of the
    /// recording, not a constant.
    #[test]
    fn the_wall_time_column_comes_from_the_recording() {
        assert_eq!(wall_metric(&["wt".into(), "zm".into()]), 0);
        assert_eq!(wall_metric(&["ct".into(), "wt".into()]), 1);
        // A recording with the metrics changed still has a first column, and a
        // share of it still says where the time went.
        assert_eq!(wall_metric(&["ct".into()]), 0);

        let trace = "\
[events]
0 1 999.0000 0.0000
1 1 999.0000 10.0000
1 0 999.0000 30.0000
0 0 999.0000 30.0000
[functions]
{main}
App::slow
";
        // Reading column 1 finds the 30 that the wall clock moved; reading
        // column 0 would find a run that took no time at all.
        let (hotspots, _, _, total) = replay(trace, 1, MAX_EVENTS);
        assert_eq!(total, 30.0);
        assert_eq!(hotspots[0].function, "App::slow");
        assert!(
            (hotspots[0].exclusive_percent - 200.0 / 3.0).abs() < 1e-9,
            "{:?}",
            hotspots[0]
        );
    }

    /// A trace longer than the limit is read as far as the limit and **says
    /// so** — and the function table below the events is still read, or every
    /// hotspot would be called `#0`.
    #[test]
    fn a_long_trace_is_cut_and_admits_it() {
        let trace = "\
[events]
0 1 0.0000
1 1 10.0000
1 0 30.0000
0 0 40.0000
[functions]
{main}
App::slow
";
        let (hotspots, events, truncated, total) = replay(trace, 0, 2);

        assert!(truncated, "the cut was not reported");
        assert_eq!(events, 2);
        assert_eq!(total, 10.0);
        assert_eq!(
            hotspots[0].function, "{main}",
            "the function table was not read: {hotspots:?}"
        );
    }

    /// Another project's file, read defensively: a damaged one costs a row, not
    /// the report.
    #[test]
    fn a_damaged_trace_is_read_as_far_as_it_goes() {
        let (hotspots, events, _, _) = replay("not a trace at all", 0, MAX_EVENTS);
        assert!(hotspots.is_empty());
        assert_eq!(events, 0);

        // An event naming an index the table never lists is still an identity.
        let (hotspots, _, _, _) =
            replay("[events]\n7 1 0.0\n7 0 5.0\n[functions]\n", 0, MAX_EVENTS);
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].function, "#7");
    }

    /// End to end, through gzip and the metadata: the milliseconds are a share
    /// of SPX's own total for the run, so they cannot drift away from the
    /// figure shown beside them in the list.
    #[test]
    fn a_recording_reads_back_from_disk_in_the_metadatas_own_milliseconds() {
        use std::io::Write as _;

        let root = std::env::temp_dir().join(format!("stackvo-spx-read-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = data_dir(&root, "shop");
        std::fs::create_dir_all(&dir).unwrap();

        let key = "spx-full-20260823_101500-abc-1-2";
        std::fs::write(
            dir.join(format!("{key}.json")),
            format!(
                r#"{{"key":"{key}","exec_ts":1,"cli":0,"wall_time_ms":400,
                     "peak_memory_usage":1,"call_count":2,"enabled_metrics":["wt","zm"]}}"#
            ),
        )
        .unwrap();

        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(
            b"[events]\n0 1 0.0000 100\n1 1 10.0000 100\n1 0 30.0000 100\n0 0 40.0000 100\n\
              [functions]\n{main}\nApp::slow\n",
        )
        .unwrap();
        std::fs::write(dir.join(format!("{key}.txt.gz")), gz.finish().unwrap()).unwrap();

        let analysis = analyse(&root, "shop", key, HOTSPOTS).expect("it reads");
        assert_eq!(analysis.wall_time_us, 400);
        assert_eq!(analysis.call_count, 2);
        assert_eq!(analysis.functions, 2);
        assert!(!analysis.truncated);

        let slow = analysis
            .hotspots
            .iter()
            .find(|spot| spot.function == "App::slow")
            .expect("the inner call is there");
        // Half the run's samples, and the run was 400 µs by SPX's own count.
        assert_eq!(slow.exclusive_percent, 50.0);
        assert_eq!(slow.exclusive_us, 200.0);

        // A key that is not one never reaches the filesystem, here either.
        assert!(analyse(&root, "shop", "../../etc/passwd", HOTSPOTS).is_err());
        // And a report that is not there is NotFound rather than a panic.
        assert!(analyse(&root, "shop", "spx-full-nothing", HOTSPOTS).is_err());

        let _ = std::fs::remove_dir_all(&root);
    }

    /// The key never goes where a clone would carry it.
    #[test]
    fn the_key_is_not_in_the_directory_that_travels_with_a_clone() {
        let path = key_path(Path::new("/w"));
        assert!(path.starts_with("/w/generated"), "{}", path.display());
        assert!(
            !path.to_string_lossy().contains(".stackvo"),
            "{}",
            path.display()
        );
    }
}
