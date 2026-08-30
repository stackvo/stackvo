//! Queue and scheduler workers per project, with Docker doing the healing.
//!
//! A Laravel app in production runs three long processes beside the web
//! server — `queue:work`, the scheduler, Horizon — and locally they usually
//! run in a forgotten terminal tab, which is why "my job never ran" is a
//! support staple. Here each worker is a **sidecar container built from the
//! project's own image**: same PHP, same extensions, same bind mount, same
//! network, so `.env` and the database resolve exactly as they do for the web
//! container.
//!
//! Self-healing is deliberately not reimplemented: the sidecar runs with
//! `--restart unless-stopped`, which is Docker's own supervisor. A worker
//! that dies is restarted by the engine whether or not this app is open; the
//! restart count is read back and shown, so a crash-looping worker is a
//! number on screen rather than a mystery.
//!
//! Detection is file-based and honest: `artisan` at the project root offers
//! the queue and scheduler workers; `laravel/horizon` in `composer.json`
//! offers Horizon. No artisan, no offer — a Node project gets an empty list,
//! not an error.

use serde::Serialize;
use std::path::Path;

/// Sidecar containers are `stackvo-worker-<project>-<kind>`.
pub const ID_PREFIX: &str = "worker-";

/// One worker kind this project could run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Queue,
    Scheduler,
    Horizon,
    /// Laravel Reverb — and the one worker that is **reachable**.
    ///
    /// Herd Pro sells Reverb as a service and EnvKit advertises "proxies
    /// WebSocket while keeping trusted HTTPS routing", which is the half that
    /// matters and the half that is not obvious. Reverb is not an image and
    /// never was: it is `php artisan reverb:start` inside the application, so
    /// it belongs here — a sidecar from the project's own image — and not in
    /// the service catalogue.
    ///
    /// What makes it different from the three above is that a browser has to
    /// open a socket to it. **A published host port does not work**, and the
    /// reason is not a preference: this app serves projects over HTTPS, and a
    /// page on `https://` may not open `ws://localhost:8080` — every browser
    /// blocks it as mixed content. So it is routed instead, and routed on the
    /// project's **own domain under Reverb's own path prefixes** rather than at
    /// a hostname of its own. That choice costs nothing and buys everything:
    /// no certificate to extend, no hosts entry to write, no wildcard to
    /// require, and `wss://shop.loc/app/<key>` is same-origin with a
    /// certificate the browser already trusts.
    Reverb,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Queue, Kind::Scheduler, Kind::Horizon, Kind::Reverb];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Queue => "queue",
            Kind::Scheduler => "scheduler",
            Kind::Horizon => "horizon",
            Kind::Reverb => "reverb",
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.as_str() == s)
    }

    /// The process the sidecar runs.
    ///
    /// `queue:work` restarts hourly on purpose: a worker holds the code it
    /// booted with, and an hour is the ceiling on serving yesterday's code
    /// after an edit. `schedule:work` is Laravel's own foreground scheduler —
    /// no host cron entry, nothing to clean up.
    pub fn command(self) -> &'static [&'static str] {
        match self {
            Kind::Queue => &[
                "php",
                "artisan",
                "queue:work",
                "--sleep=3",
                "--tries=3",
                "--max-time=3600",
            ],
            Kind::Scheduler => &["php", "artisan", "schedule:work"],
            Kind::Horizon => &["php", "artisan", "horizon"],
            // `--host=0.0.0.0` because Reverb's own default binds loopback,
            // which inside a container means nothing outside it can connect —
            // including Traefik, one hop away on the same network.
            Kind::Reverb => &[
                "php",
                "artisan",
                "reverb:start",
                "--host=0.0.0.0",
                "--port=8080",
            ],
        }
    }

    /// The port Reverb listens on inside its container.
    ///
    /// Laravel's own default, kept rather than chosen: the value appears in
    /// every Reverb tutorial and in `config/reverb.php`, and a different one
    /// here would be a number somebody has to discover.
    /// Held against the `--port=8080` above by a test, because the two being
    /// one number is the whole reason the route reaches the process.
    pub const REVERB_PORT: u16 = 8080;

    /// The URL path prefixes Reverb answers on.
    ///
    /// Fixed by the Pusher protocol rather than by Laravel: `/app/{key}` is the
    /// websocket and `/apps/{id}/…` is the HTTP API events endpoint. Both are
    /// what an nginx in front of Reverb is told to proxy in Laravel's own
    /// deployment notes, so routing them is not an invention.
    pub const REVERB_PATHS: [&'static str; 2] = ["/app", "/apps"];

    /// Does this worker have to be reachable from a browser?
    pub fn routed(self) -> bool {
        matches!(self, Kind::Reverb)
    }
}

/// The engine-facing id of one worker sidecar.
pub fn container_id(project: &str, kind: Kind) -> String {
    format!("{ID_PREFIX}{project}-{}", kind.as_str())
}

/// `worker-<project>-<kind>` back into its parts. The kind is matched as a
/// suffix because project names may themselves contain `-`.
pub fn parse_id(id: &str) -> Option<(String, Kind)> {
    let rest = id.strip_prefix(ID_PREFIX)?;
    for kind in Kind::ALL {
        if let Some(project) = rest.strip_suffix(&format!("-{}", kind.as_str())) {
            if !project.is_empty() {
                return Some((project.to_string(), kind));
            }
        }
    }
    None
}

/// Which workers this project can offer, from its files alone.
pub fn available(root: &Path, project: &str) -> Vec<Kind> {
    let Some(projects) = crate::workspace::projects_root(root) else {
        return Vec::new();
    };
    let dir = projects.join(project);
    if !dir.join("artisan").is_file() {
        return Vec::new();
    }

    let mut out = vec![Kind::Queue, Kind::Scheduler];

    // One read for both, and `require` only. A package in `require-dev` is a
    // tool for the test suite, not a process this app should offer to run
    // beside the site — and `laravel/horizon` under `require-dev` is a real
    // thing people do while they are evaluating it.
    let composer = std::fs::read_to_string(dir.join("composer.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let requires = |package: &str| {
        composer
            .as_ref()
            .is_some_and(|json| json.get("require").and_then(|r| r.get(package)).is_some())
    };

    if requires("laravel/horizon") {
        out.push(Kind::Horizon);
    }
    if requires("laravel/reverb") {
        out.push(Kind::Reverb);
    }
    out
}

/// One worker's live state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerStatus {
    pub project: String,
    pub kind: Kind,
    pub running: bool,
    /// How often Docker has had to bring it back — the self-heal made
    /// visible. A large number is a crash loop, not a success story.
    pub restarts: Option<i64>,
    pub container: String,
}

/// Every worker sidecar the engine knows about.
pub async fn status_all() -> crate::error::Result<Vec<WorkerStatus>> {
    let containers = crate::engine::stackvo_containers().await?;
    let mut out = Vec::new();

    for (id, info) in containers {
        let Some((project, kind)) = parse_id(&id) else {
            continue;
        };
        // The restart count lives in inspect, not the list; workers are few.
        let restarts = crate::engine::inspect(&id)
            .await
            .ok()
            .map(|d| d.restart_count);

        out.push(WorkerStatus {
            project,
            kind,
            running: info.running,
            restarts,
            container: info.name,
        });
    }

    out.sort_by(|a, b| (&a.project, a.kind.as_str()).cmp(&(&b.project, b.kind.as_str())));
    Ok(out)
}

/// The `docker run` invocation for one worker sidecar.
///
/// `image` and the bind mount come from the project's own web container, so
/// the worker sees exactly what the site sees. `--restart unless-stopped` is
/// the whole self-heal story; `stop` (which removes) is the only way it stays
/// down.
///
/// `debug` is the debug bridge's three mounts, or `None` where they could not
/// be written. They are here because "same PHP, same extensions, same bind
/// mount" was one mount short of true: the bridge reaches the web container
/// through a compose overlay, and a sidecar is not a compose service, so a
/// `dump()` inside a queued job was written by nothing and read by nobody.
/// The ini is a *file* mount into `conf.d` — mounting the directory would
/// shadow every other ini the image carries, including the one the profiler
/// writes.
pub fn run_args(
    project: &str,
    kind: Kind,
    image: &str,
    host_root: &str,
    network: &str,
    domain: &str,
    debug: Option<&crate::debugbridge::Entry>,
) -> Vec<String> {
    let mount = format!(
        "{}/projects/{project}:/var/www/html",
        crate::paths::to_docker_mount(host_root).trim_end_matches('/')
    );

    let mut args: Vec<String> = [
        "run",
        "-d",
        "--name",
        &format!("stackvo-{}", container_id(project, kind)),
        "--network",
        network,
        "--restart",
        "unless-stopped",
        "-v",
        &mount,
        "-w",
        "/var/www/html",
        image,
    ]
    .into_iter()
    .map(String::from)
    .collect();

    if let Some(debug) = debug {
        // In front of the image, like every other flag: `docker run` reads the
        // image as the end of its own arguments and everything after it as the
        // container's command line.
        let image = args.pop().expect("the image was pushed above");
        for mount in [
            format!("{}:{}:ro", debug.conf_host, crate::debugbridge::CONF_DIR),
            format!("{}:{}", debug.events_host, crate::debugbridge::EVENTS_DIR),
            format!(
                "{}:{}:ro",
                debug.ini_host,
                crate::debugbridge::INI_CONTAINER_PATH
            ),
        ] {
            args.push("-v".to_string());
            args.push(mount);
        }
        args.push(image);
    }

    if kind.routed() {
        // The image is the last thing docker reads before the container's own
        // command line, so the labels go in front of it — taken off and put
        // back rather than spliced, which reads as what it is.
        let image = args.pop().expect("the image was pushed above");
        for label in route_labels(project, kind, domain) {
            args.push("-l".to_string());
            args.push(label);
        }
        args.push(image);
    }

    args.extend(kind.command().iter().map(|s| s.to_string()));
    args
}

/// The Traefik labels that put a routed worker on the project's own domain.
///
/// **A path prefix on the existing host, not a hostname of its own.** A new
/// hostname would need the certificate extended, a hosts entry written and — on
/// a project without a `*.` alias — a wildcard that is not there; a path costs
/// none of that and lands the socket same-origin with the page that opens it.
///
/// The priority is **set rather than inherited**. Traefik's default orders
/// routers by rule length, and this rule is longer than the project's bare
/// `Host()` so it would usually win — "usually" being the problem. A number
/// says it.
fn route_labels(project: &str, kind: Kind, domain: &str) -> Vec<String> {
    let name = format!(
        "{}-{}",
        crate::generator::traefik_name(project),
        kind.as_str()
    );
    let paths = Kind::REVERB_PATHS
        .map(|path| format!("PathPrefix(`{path}`)"))
        .join(" || ");

    vec![
        "traefik.enable=true".to_string(),
        format!("traefik.http.routers.{name}.rule=Host(`{domain}`) && ({paths})"),
        format!("traefik.http.routers.{name}.entrypoints=websecure"),
        format!("traefik.http.routers.{name}.tls=true"),
        // Above the project's own router, which matches the same host with no
        // path clause at all.
        format!("traefik.http.routers.{name}.priority=100"),
        format!(
            "traefik.http.services.{name}.loadbalancer.server.port={}",
            Kind::REVERB_PORT
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_even_when_the_project_name_has_dashes_and_dots() {
        for (project, kind) in [
            ("myapp", Kind::Queue),
            ("parser.ajans", Kind::Scheduler),
            ("api-v2.shop", Kind::Horizon),
        ] {
            let id = container_id(project, kind);
            assert_eq!(parse_id(&id), Some((project.to_string(), kind)));
        }
        assert_eq!(parse_id("worker-"), None);
        assert_eq!(parse_id("worker--queue"), None);
        assert_eq!(parse_id("tunnel-myapp"), None);
        // A project that merely ends in a kind word is not a worker.
        assert_eq!(parse_id("worker-queue"), None);
    }

    #[test]
    fn detection_offers_nothing_without_artisan_and_horizon_only_when_required() {
        let dir = std::env::temp_dir().join(format!("stackvo-worker-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let project = dir.join("projects/app");
        std::fs::create_dir_all(&project).unwrap();
        crate::workspace::point_at_projects(&dir, &dir.join("projects")).unwrap();

        assert!(available(&dir, "app").is_empty());

        std::fs::write(project.join("artisan"), "#!/usr/bin/env php").unwrap();
        assert_eq!(available(&dir, "app"), vec![Kind::Queue, Kind::Scheduler]);

        std::fs::write(
            project.join("composer.json"),
            r#"{ "require": { "php": "^8.2", "laravel/horizon": "^5.0" } }"#,
        )
        .unwrap();
        assert_eq!(
            available(&dir, "app"),
            vec![Kind::Queue, Kind::Scheduler, Kind::Horizon]
        );

        // Reverb is found the same way and only the same way.
        std::fs::write(
            project.join("composer.json"),
            r#"{ "require": { "php": "^8.2", "laravel/reverb": "^1.0" } }"#,
        )
        .unwrap();
        assert_eq!(
            available(&dir, "app"),
            vec![Kind::Queue, Kind::Scheduler, Kind::Reverb]
        );

        // `require-dev` is a tool for the test suite, not a process to run
        // beside the site — and putting Horizon there while evaluating it is a
        // real thing people do.
        std::fs::write(
            project.join("composer.json"),
            r#"{ "require": { "php": "^8.2" },
                 "require-dev": { "laravel/horizon": "^5.0", "laravel/reverb": "^1.0" } }"#,
        )
        .unwrap();
        assert_eq!(available(&dir, "app"), vec![Kind::Queue, Kind::Scheduler]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Reverb is the only worker a browser has to reach, and the whole of that
    /// is the labels: a published host port cannot work, because this app
    /// serves over HTTPS and no browser will open `ws://` from an `https://`
    /// page.
    #[test]
    fn a_reverb_worker_is_routed_on_the_projects_own_domain() {
        let args = run_args(
            "shop",
            Kind::Reverb,
            "stackvo-shop:latest",
            "/Users/x/stackvo",
            "stackvo-net",
            "shop.loc",
            None,
        );
        let line = args.join(" ");

        // The project's own host, under Reverb's own path prefixes — so the
        // certificate that already exists covers it and the socket is
        // same-origin with the page.
        assert!(
            line.contains("rule=Host(`shop.loc`) && (PathPrefix(`/app`) || PathPrefix(`/apps`))"),
            "{line}"
        );
        assert!(line.contains("entrypoints=websecure"), "{line}");
        assert!(line.contains(".tls=true"), "{line}");
        // Above the project's own router, which matches the same host with no
        // path clause. Default priority is by rule length and would *usually*
        // do it; usually is what this replaces.
        assert!(line.contains(".priority=100"), "{line}");
        assert!(
            line.contains(&format!("loadbalancer.server.port={}", Kind::REVERB_PORT)),
            "{line}"
        );

        // No published host port — that is the thing that does not work.
        assert!(!line.contains(" -p "), "{line}");

        // The labels come before the image; everything after it is the
        // container's own command line.
        let image_at = args
            .iter()
            .position(|a| a == "stackvo-shop:latest")
            .unwrap();
        let last_label = args.iter().rposition(|a| a == "-l").unwrap();
        assert!(last_label < image_at, "{args:?}");
        assert!(line.ends_with("php artisan reverb:start --host=0.0.0.0 --port=8080"));
    }

    /// The port in the command line and the port in the label are one number.
    ///
    /// They are written in two places — an argument to `reverb:start` and a
    /// Traefik service port — and if they come apart the route reaches a port
    /// nothing is listening on. The symptom is a socket that connects to
    /// Traefik and then closes, which reads as a Reverb problem.
    #[test]
    fn the_routed_port_and_the_listening_port_are_the_same() {
        let command = Kind::Reverb.command().join(" ");
        assert!(
            command.contains(&format!("--port={}", Kind::REVERB_PORT)),
            "{command}"
        );
        // And it binds every interface: Reverb's own default is loopback, which
        // inside a container means Traefik one hop away cannot reach it.
        assert!(command.contains("--host=0.0.0.0"), "{command}");
    }

    /// Only Reverb is routed, and the three that are not must stay that way.
    #[test]
    fn the_unroutable_workers_gain_no_labels() {
        for kind in [Kind::Queue, Kind::Scheduler, Kind::Horizon] {
            assert!(!kind.routed(), "{kind:?}");
            let args = run_args(
                "shop",
                kind,
                "stackvo-shop:latest",
                "/Users/x/stackvo",
                "stackvo-net",
                "shop.loc",
                None,
            );
            assert!(
                !args.iter().any(|a| a == "-l"),
                "`{}` gained a route it has nothing to serve on",
                kind.as_str()
            );
        }
        assert!(Kind::Reverb.routed());
    }

    /// "Same PHP, same extensions, same bind mount" was one mount short of
    /// true. The debug bridge reaches the web container through a compose
    /// overlay, and a sidecar is not a compose service — so a `dump()` inside
    /// a queued job was written by nothing and read by nobody, which is the
    /// one place it is hardest to catch by any other means.
    #[test]
    fn a_worker_carries_the_debug_bridge_the_web_container_gets() {
        let debug = crate::debugbridge::Entry {
            service: "myapp".into(),
            conf_host: "/w/generated/debug/myapp/conf".into(),
            events_host: "/w/generated/debug/myapp/events".into(),
            ini_host: "/w/generated/debug/myapp/conf/stackvo-debug.ini".into(),
        };
        let args = run_args(
            "myapp",
            Kind::Queue,
            "stackvo-myapp:latest",
            "/Users/x/stackvo",
            "stackvo-net",
            "myapp.loc",
            Some(&debug),
        );
        let line = args.join(" ");

        assert!(line.contains(&format!(
            "/w/generated/debug/myapp/conf:{}:ro",
            crate::debugbridge::CONF_DIR
        )));
        // Writable, or the bridge has nowhere to write and the pane stays
        // empty with everything else looking correct.
        assert!(line.contains(&format!(
            "/w/generated/debug/myapp/events:{}",
            crate::debugbridge::EVENTS_DIR
        )));
        assert!(!line.contains(&format!("{}:ro", crate::debugbridge::EVENTS_DIR)));
        // A FILE mount into conf.d. Mounting the directory would shadow every
        // other ini the image carries, the profiler's included.
        assert!(line.contains(&format!(
            "stackvo-debug.ini:{}:ro",
            crate::debugbridge::INI_CONTAINER_PATH
        )));

        // Every mount is an argument to `docker run`, not to the worker: past
        // the image, everything is the container's own command line.
        let image_at = args
            .iter()
            .position(|a| a == "stackvo-myapp:latest")
            .expect("no image");
        assert!(
            args.iter().rposition(|a| a == "-v").unwrap() < image_at,
            "{args:?}"
        );
        assert!(line.ends_with("php artisan queue:work --sleep=3 --tries=3 --max-time=3600"));
    }

    /// A bridge that could not be written must not take the worker with it.
    /// The honest degradation is a worker with no bridge, never a queue that
    /// will not start.
    #[test]
    fn a_worker_starts_without_a_bridge_when_there_is_none() {
        let args = run_args(
            "myapp",
            Kind::Queue,
            "stackvo-myapp:latest",
            "/Users/x/stackvo",
            "stackvo-net",
            "myapp.loc",
            None,
        );
        assert_eq!(
            args.iter().filter(|a| *a == "-v").count(),
            1,
            "the project mount is the only one left: {args:?}"
        );
    }

    #[test]
    fn the_sidecar_runs_the_projects_image_with_docker_as_supervisor() {
        let args = run_args(
            "myapp",
            Kind::Queue,
            "stackvo-myapp:latest",
            "/Users/x/stackvo",
            "stackvo-net",
            "myapp.loc",
            None,
        );
        let line = args.join(" ");
        assert!(line.contains("--name stackvo-worker-myapp-queue"));
        assert!(line.contains("--restart unless-stopped"));
        assert!(line.contains("-v /Users/x/stackvo/projects/myapp:/var/www/html"));
        assert!(line.contains("stackvo-myapp:latest php artisan queue:work"));
        assert!(line.ends_with("--max-time=3600"));

        let horizon = run_args(
            "myapp",
            Kind::Horizon,
            "stackvo-myapp:latest",
            "/Users/x/stackvo",
            "stackvo-net",
            "myapp.loc",
            None,
        );
        assert!(horizon.join(" ").ends_with("php artisan horizon"));
    }
}
