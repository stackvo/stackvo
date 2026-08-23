//! An MCP server over the same core the app drives.
//!
//! Five of the eight competitors ship one — Herd, Lerd, EnvKit, FlyEnv and
//! ServBay — and in 2026 it is the price of entry rather than a differentiator.
//! What is different here is where the tool list comes from.
//!
//! ## The contract is the authority
//!
//! `contracts/ipc.json` already records every command, its arguments, its
//! return type and — the part that matters — its `kind`: `query`, `mutation`,
//! `operation` or `stream`. The table below names, for each tool, the command
//! it implements, and a test cross-checks the two: a tool naming a command that
//! does not exist fails, and a tool claiming to be read-only while its command
//! is declared a mutation fails. That is the same discipline suites E and F
//! apply to the IPC surface, for the same reason — nothing enforces any of it
//! at compile time.
//!
//! Generating the tools outright was the obvious move and is the wrong one:
//! dispatch cannot be generated, so a generated list would advertise tools that
//! then fail when called. A checked hand-written table advertises exactly what
//! it can do.
//!
//! ## What is exposed, and what is not
//!
//! Reads, freely. They are the answer to the question an assistant is usually
//! asked — "why is shop.loc not loading?" — which needs preflight, the hosts
//! file, the certificate's SAN list and a container's last hundred log lines,
//! and needs to change nothing to find out.
//!
//! Writes only behind `--allow-writes`, and only the few that are reversible
//! and cheap. Not a blanket ban on principle: **most mutations are not reachable
//! from here at all.** Thirty-four commands take an `AppHandle` because they
//! report progress through Tauri's event system, and a stdio subprocess has no
//! app to emit into. Decoupling that is a refactor of its own, and pretending
//! otherwise by shelling out to a second copy of the stack would be worse than
//! saying so.

use crate::error::Result;
use serde_json::{json, Value};

/// The protocol revision this server answers with when the client asks for one
/// it does not know, and the newest it implements.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Every revision this server will speak, newest first.
///
/// The spec's rule is that the server echoes the client's requested version
/// when it can support it, and otherwise names its own — a client that asked
/// for `2024-11-05` and got `2025-06-18` back is entitled to hang up. Answering
/// with a constant was therefore not merely out of date; it was a handshake
/// that told older clients to disconnect. Nothing in this surface differs
/// across these three revisions — the tool shape and `tools/call` are
/// unchanged — so supporting them is a matter of saying so.
pub const PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// The revision to answer an `initialize` with.
pub fn negotiate(requested: Option<&str>) -> &'static str {
    requested
        .and_then(|asked| PROTOCOL_VERSIONS.iter().find(|known| **known == asked))
        .copied()
        .unwrap_or(PROTOCOL_VERSION)
}

/// One exposed tool, and the contract command it stands for.
pub struct Tool {
    pub name: &'static str,
    /// The `contracts/ipc.json` command this implements. Cross-checked by test.
    pub command: &'static str,
    /// **Every other contract command this tool's dispatch reaches.**
    ///
    /// A tool is not always one command. `stackvo_project` answers the three
    /// questions that actually explain a site not loading — the certificate,
    /// Xdebug, the PHP limits — rather than leaving them as three more calls,
    /// and that was right and stayed undeclared.
    ///
    /// Undeclared is what made it a hole. [`crate::websurface`] decides what
    /// the loopback surface may serve by asking `exposable` about *the command
    /// a tool names*, and that answer is only the whole answer while a tool
    /// reads nothing else. `stackvo_log_read` broke it: it names `app_logs`,
    /// which lists files and is a `query`, and then returns the tail of one —
    /// which is `app_log_open`, a `mutation`. Container logs were denied that
    /// surface by their `stream` kind while application logs would have walked
    /// straight past it.
    ///
    /// So the reach is written down and the surface intersects over all of it.
    /// Empty for a tool that is exactly its command, which is most of them.
    pub also: &'static [&'static str],
    pub description: &'static str,
    /// True when calling it changes something on disk or in the stack.
    pub writes: bool,
    /// JSON Schema for `arguments`.
    pub schema: fn() -> Value,
}

impl Tool {
    /// Every contract command this tool reaches, its own included.
    pub fn commands(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.command).chain(self.also.iter().copied())
    }
}

fn no_args() -> Value {
    json!({ "type": "object", "properties": {}, "additionalProperties": false })
}

fn project_arg() -> Value {
    json!({
        "type": "object",
        "properties": { "name": { "type": "string", "description": "Project directory name." } },
        "required": ["name"],
        "additionalProperties": false
    })
}

fn logs_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "container": { "type": "string", "description": "Project or service id, without the stackvo- prefix." },
            "tail": { "type": "integer", "description": "How many lines from the end. Default 100.", "minimum": 1, "maximum": 2000 }
        },
        "required": ["container"],
        "additionalProperties": false
    })
}

fn limit_arg() -> Value {
    json!({
        "type": "object",
        "properties": { "limit": { "type": "integer", "minimum": 1, "maximum": 200 } },
        "additionalProperties": false
    })
}

fn stack_up_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": ["minimal", "services", "projects", "all"],
                "description": "Which profiles to bring up. Default minimal (core only)."
            }
        },
        "additionalProperties": false
    })
}

fn generate_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "scope": {
                "type": "string",
                "enum": ["all", "projects", "services"],
                "description": "What to regenerate. Default all."
            }
        },
        "additionalProperties": false
    })
}

fn container_arg() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Project, service or instance id, without the stackvo- prefix." }
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

fn instance_arg() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Instance id as stackvo_service_instances reports it — `mysql-8-4`, not `mysql`." }
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn service_arg() -> Value {
    json!({
        "type": "object",
        "properties": { "service": { "type": "string", "description": "Service or instance id." } },
        "required": ["service"],
        "additionalProperties": false
    })
}

fn id_arg() -> Value {
    json!({
        "type": "object",
        "properties": { "id": { "type": "string" } },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn app_log_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Project directory name." },
            "id": {
                "type": "string",
                "description": "A log id from this tool's own `files` list. Omit to list them."
            },
            "bytes": {
                "type": "integer",
                "description": "How many bytes from the end of that file. Default 65536.",
                "minimum": 1024,
                "maximum": 1048576
            }
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

fn snapshot_take_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "service": { "type": "string", "description": "A database service or instance id, as stackvo_databases reports it." },
            "name": { "type": "string", "description": "What to file the snapshot under. Letters, digits, dash and underscore." }
        },
        "required": ["service", "name"],
        "additionalProperties": false
    })
}

fn xdebug_set_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "enabled": { "type": "boolean" }
        },
        "required": ["name", "enabled"],
        "additionalProperties": false
    })
}

/// Take the profiler's key out of a status document.
///
/// Two fields carry it: `controlUrl` opens SPX's own panel and `viewBase` is
/// what a report key is appended to. A key is a credential however cheap it is,
/// this surface returns none, and [`crate::websurface`] serves whatever a tool
/// returns — so the removal is a function both the arm and its test go through,
/// rather than a few lines in the arm and a re-implementation in the test that
/// can agree with each other while neither matches the code that runs.
///
/// Nothing is lost by removing them: a model cannot open a browser, and where
/// to find the panel is said instead.
pub fn redact_profiler(status: &mut Value) {
    let Some(object) = status.as_object_mut() else {
        return;
    };

    let control = object.remove("controlUrl").is_some();
    let view = object.remove("viewBase").is_some();
    if control || view {
        object.insert(
            "controlPanel".into(),
            json!(
                "Open it from the project's Debug section — the URL carries a key, which this \
                 surface does not return. `stackvo_hotspots` reads a recording without one."
            ),
        );
    }
}

fn hotspots_args() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "key": {
                "type": "string",
                "description": "A report key, as `stackvo_profiler` lists them."
            }
        },
        "required": ["name", "key"],
        "additionalProperties": false
    })
}

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "stackvo_overview",
        command: "preflight",
        also: &["engine_status", "projects_list"],
        description: "The state of the whole stack: the chosen checkout, the Docker engine, \
                      every startup requirement and how many projects and services are running. \
                      Start here — it is the one call that says whether anything else will work.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_doctor",
        command: "doctor",
        also: &[],
        description: "The full diagnosis: every startup requirement, every host port the stack \
                      will claim and who holds it now (the stack itself, another container by \
                      name, or a host process by name and pid), project domains missing from \
                      the hosts file, whether the generated config is older than its inputs, \
                      and how much disk unused images and volumes hold. Use this to explain a \
                      failed start — especially \"address already in use\".",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_projects",
        command: "projects_list",
        also: &[],
        description: "Every managed project: domain, runtime, whether it is built and running, \
                      whether its domain has a hosts entry, and any contract violations in its \
                      stackvo.json.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_project",
        command: "project_get",
        also: &[
            "projects_list",
            "cert_status",
            "xdebug_status",
            "php_ini_status",
            "container_inspect",
        ],
        description: "One project in full: its manifest, its container, its Xdebug state, its \
                      PHP limits as the running container actually reports them, and whether the \
                      HTTPS certificate covers its domain. Use this to explain why a specific \
                      site does not load — or why an upload fails at a limit the user believes \
                      they have already raised.",
        writes: false,
        schema: project_arg,
    },
    Tool {
        name: "stackvo_profiler",
        command: "spx_status",
        also: &["xdebug_status", "projects_list"],
        description: "The sampling profiler (php-spx) for one project: whether it is built for \
                      that PHP version, switched on, actually mounted in the running container, \
                      and every run it has recorded — wall time, peak memory, call count, and \
                      the request or command it was. This is the profiler that can be left on; \
                      Xdebug's costs several times the request and cannot be. Also reports when \
                      both are recording, which is unsupported and shows up as wrong numbers \
                      rather than an error.",
        writes: false,
        schema: project_arg,
    },
    Tool {
        name: "stackvo_hotspots",
        command: "spx_report",
        also: &[],
        description: "Where one recorded profile spent its time: the functions holding it, \
                      ranked, with the share of the run each held in its own body and the share \
                      it held including everything it called, plus how many times each was \
                      called. This is the answer to \"why is this page slow\" — \
                      `stackvo_profiler` lists the recordings and their keys, and this reads \
                      one. A very long trace is replayed up to a limit and says so.",
        writes: false,
        schema: hotspots_args,
    },
    Tool {
        name: "stackvo_ide_debug",
        command: "ide_debug_status",
        also: &[],
        description: "Why a breakpoint is not being hit. The debug port, the IDE key, the server \
                      name and both halves of the path mapping — plus the two things that are not \
                      in any file: whether anything is listening on that port right now and which \
                      process it is, and whether each IDE's configuration in the project is \
                      written, missing or left over from before the values moved. The mapping is \
                      the answer more often than anything else.",
        writes: false,
        schema: project_arg,
    },
    Tool {
        name: "stackvo_services",
        command: "services_list",
        also: &[],
        description: "The shared infrastructure services — databases, caches, search, queues — \
                      with which are enabled, which are running and what they depend on.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_logs",
        command: "container_logs_open",
        also: &["container_inspect"],
        description: "The last lines of a container's log. Reads to the end and returns; it does \
                      not follow.",
        writes: false,
        schema: logs_args,
    },
    Tool {
        name: "stackvo_log_files",
        command: "app_logs_all",
        also: &[],
        description: "Every log file every project writes, newest first, with its size and when \
                      it last changed. The container log carries only stdout — an application's \
                      own exception is in one of these. Read the timestamps to find where the \
                      activity is before opening anything: the file that changed a minute ago is \
                      the one worth reading. Needs no engine, so it still answers with Docker \
                      stopped.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_certificates",
        command: "cert_status",
        also: &[],
        description: "The HTTPS certificate: which domains it covers, which it does not, whether \
                      its CA is trusted and when it expires. A domain missing here is a browser \
                      warning the user cannot otherwise explain.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_databases",
        command: "db_targets",
        also: &[],
        description: "The database services that can be dumped, their database names and whether \
                      they are running.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_mail",
        command: "mail_messages",
        also: &["mail_status"],
        description: "The mail catcher's inbox: what the applications under test have sent.",
        writes: false,
        schema: limit_arg,
    },
    Tool {
        name: "stackvo_mail_message",
        command: "mail_message",
        also: &["mail_status"],
        description: "One captured message in full — headers, the text part and the HTML part. \
                      The inbox listing carries subjects and addresses; this is the body, which \
                      is where a broken reset link or an unrendered Blade variable actually is.",
        writes: false,
        schema: id_arg,
    },
    Tool {
        name: "stackvo_system",
        command: "host_stats",
        also: &["docker_system_resources", "docker_disk_usage"],
        description: "What this machine has left: host CPU, memory, swap, disks and network, \
                      the Docker engine's own totals, and which stack member holds the image and \
                      volume bytes. Sampled twice a third of a second apart, because a single \
                      reading has no CPU delta to report. Use it before blaming the stack for \
                      being slow — a host at 97% memory is the answer.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_container_stats",
        command: "container_stats",
        also: &[],
        description: "One container's live CPU, memory against its limit, and network and block \
                      I/O totals. The per-container half of stackvo_system: which one is eating \
                      the machine.",
        writes: false,
        schema: container_arg,
    },
    Tool {
        name: "stackvo_service_instances",
        command: "instance_list",
        also: &[],
        description: "Every installed service version as a separately controllable instance — \
                      `mysql-8-0` and `mysql-8-4` side by side — with its container name, its \
                      network aliases, the host ports allocated to it, whether it is switched on \
                      and whether its package is actually present on disk. The ids here are what \
                      stackvo_service_start and its pair take.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_service_connection",
        command: "service_connection",
        also: &["instance_list", "services_list"],
        description: "How to reach one service: scheme, host, port, database and user, for the \
                      host and from inside the network. **The password is never included** — \
                      this server has no tool that reads a stored credential back, and that is a \
                      rule rather than an omission.",
        writes: false,
        schema: service_arg,
    },
    Tool {
        name: "stackvo_hosts",
        command: "hosts_overview",
        also: &["hosts_status", "projects_list"],
        description: "Every domain the stack wants, whether the hosts file maps it and to what, \
                      plus StackVo's own entries that no longer serve anything. A domain missing \
                      here does not resolve, which is the most common reason a site that is \
                      built, running and certificated still does not load.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_log_read",
        command: "app_logs",
        also: &["app_log_open", "projects_list"],
        description: "One project's own log files, and — when given an `id` from that list — the \
                      last bytes of one of them. This is the other half of stackvo_log_files: \
                      that one says which file changed a minute ago, this one reads it. \
                      Application exceptions are here, not in the container log.",
        writes: false,
        schema: app_log_args,
    },
    Tool {
        name: "stackvo_snapshots",
        command: "db_snapshots",
        also: &[],
        description: "The database snapshots this workspace holds, newest first, with which \
                      service each came from, its size and when it was taken.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_packages",
        command: "market_catalog",
        also: &["market_status"],
        description: "The service catalogue: every package this workspace's registry knows, each \
                      version it offers, and which of those are already on disk. Answers \"can I \
                      have PostgreSQL 17\" without guessing.",
        writes: false,
        schema: no_args,
    },
    Tool {
        name: "stackvo_xdebug_set",
        command: "xdebug_set",
        also: &[],
        description: "Turn step debugging on or off for one project. The extension is compiled \
                      in, so the project needs regenerating and rebuilding afterwards — the reply \
                      says whether that is outstanding.",
        writes: true,
        schema: xdebug_set_args,
    },
    Tool {
        name: "stackvo_certificates_reissue",
        command: "cert_apply",
        also: &[],
        description: "Reissue the HTTPS certificate for the domains the projects actually have, \
                      and trust the CA if nothing does yet.",
        writes: true,
        schema: no_args,
    },
    Tool {
        name: "stackvo_project_start",
        command: "project_start",
        also: &["projects_list"],
        description: "Start one project's container. Idempotent: starting a running project \
                      succeeds silently.",
        writes: true,
        schema: project_arg,
    },
    Tool {
        name: "stackvo_project_stop",
        command: "project_stop",
        also: &["projects_list"],
        description: "Stop one project's container. Idempotent, like start.",
        writes: true,
        schema: project_arg,
    },
    Tool {
        name: "stackvo_stack_up",
        command: "compose_up",
        also: &[],
        description: "Bring the stack up with docker compose — builds missing images, so the \
                      first run can take minutes. Runs to completion and reports the outcome.",
        writes: true,
        schema: stack_up_args,
    },
    Tool {
        name: "stackvo_stack_down",
        command: "compose_down",
        also: &[],
        description: "Bring the whole stack down: every profile, projects included.",
        writes: true,
        schema: no_args,
    },
    Tool {
        name: "stackvo_generate",
        command: "generate_run",
        also: &[],
        description: "Re-run the generator: derive the compose files, Dockerfiles and configs \
                      from .env and the project manifests. The doctor's 'generated config is \
                      stale' finding is repaired by exactly this.",
        writes: true,
        schema: generate_args,
    },
    Tool {
        name: "stackvo_project_restart",
        command: "project_restart",
        also: &["projects_list"],
        description: "Stop one project's container and start it again. The project's own \
                      lifecycle hooks do not run from here — the same limit stackvo_project_start \
                      and stackvo_project_stop have, and for the same reason.",
        writes: true,
        schema: project_arg,
    },
    Tool {
        name: "stackvo_service_start",
        command: "instance_start",
        also: &[],
        description: "Start one service instance — `redis-7-2`, not `redis`. Idempotent. Takes \
                      an id from stackvo_service_instances; a service that is switched off has no \
                      container to start and must be enabled from the app first.",
        writes: true,
        schema: instance_arg,
    },
    Tool {
        name: "stackvo_service_stop",
        command: "instance_stop",
        also: &[],
        description: "Stop one service instance. Nothing is deleted — the data directory and the \
                      volume stay exactly as they are.",
        writes: true,
        schema: instance_arg,
    },
    Tool {
        name: "stackvo_service_restart",
        command: "instance_restart",
        also: &[],
        description: "Restart one service instance. What a changed configuration usually needs \
                      before it takes effect.",
        writes: true,
        schema: instance_arg,
    },
    Tool {
        name: "stackvo_snapshot_take",
        command: "db_snapshot_take",
        also: &["db_dump"],
        description: "Dump one database into a named snapshot this workspace keeps. Adds nothing \
                      and changes nothing in the database — it is the call to make **before** \
                      asking for a migration to be run. Restoring one is deliberately not exposed \
                      here: putting data back over live rows is a decision for the app's own \
                      confirmation, not for a tool call.",
        writes: true,
        schema: snapshot_take_args,
    },
];

/// The tools this run offers.
pub fn visible(allow_writes: bool) -> impl Iterator<Item = &'static Tool> {
    TOOLS.iter().filter(move |t| allow_writes || !t.writes)
}

// --------------------------------------------------------------- protocol
//
// JSON-RPC 2.0 over stdio, hand-rolled. The surface is three methods; an SDK
// would be a dependency and a version to track for less code than this.

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Tool output. MCP carries text; JSON is what an assistant can actually use.
fn text_result(id: Value, body: &Value, is_error: bool) -> Value {
    let text = serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string());
    ok(
        id,
        json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }),
    )
}

pub fn tools_list(allow_writes: bool) -> Value {
    let tools: Vec<Value> = visible(allow_writes)
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": (tool.schema)(),
                // Advertised so a client can require confirmation without
                // knowing what any individual tool does.
                "annotations": { "readOnlyHint": !tool.writes, "destructiveHint": tool.writes },
            })
        })
        .collect();

    json!({ "tools": tools })
}

/// Handle one request. `None` for a notification, which takes no reply.
pub async fn handle(request: &Value, allow_writes: bool) -> Option<Value> {
    let method = request.get("method")?.as_str()?;
    let id = request.get("id").cloned();

    // A notification has no id and must never be answered — a reply to one is a
    // protocol error that some clients treat as fatal.
    let id = id?;

    Some(match method {
        "initialize" => ok(
            id,
            json!({
                "protocolVersion": negotiate(
                    request
                        .get("params")
                        .and_then(|p| p.get("protocolVersion"))
                        .and_then(|v| v.as_str()),
                ),
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "stackvo", "version": env!("CARGO_PKG_VERSION") },
            }),
        ),
        "ping" => ok(id, json!({})),
        "tools/list" => ok(id, tools_list(allow_writes)),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            match call(name, &args, allow_writes).await {
                Ok(body) => text_result(id, &body, false),
                // Returned as tool output rather than a JSON-RPC error: an
                // assistant can read and act on "Docker is not running", which
                // a transport-level failure would hide from it.
                Err(e) => text_result(
                    id,
                    &json!({ "error": e.message, "code": format!("{:?}", e.code), "hint": e.hint }),
                    true,
                ),
            }
        }
        _ => err(id, -32601, "method not found"),
    })
}

/// Run one tool.
pub async fn call(name: &str, args: &Value, allow_writes: bool) -> Result<Value> {
    use crate::error::{Code, Error};

    let tool = TOOLS
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| Error::new(Code::NotFound, format!("no tool named {name}")))?;

    if tool.writes && !allow_writes {
        return Err(Error::new(
            Code::PermissionDenied,
            format!("{name} changes the stack and this server was started read-only"),
        )
        .with_hint(crate::hints::MCP_NEEDS_ALLOW_WRITES));
    }

    let ws = crate::workspace::resolve();
    let root = ws.require_root()?;
    let string = |key: &str| args.get(key).and_then(|v| v.as_str()).map(str::to_string);

    match name {
        "stackvo_overview" => {
            let preflight = crate::preflight::run().await;
            let engine = crate::engine::status().await;
            let projects = crate::commands::list_projects(&root)
                .await
                .unwrap_or_default();

            Ok(json!({
                "workspace": { "root": ws.root, "version": ws.stackvo_version },
                "engine": engine,
                "preflight": preflight,
                "projects": {
                    "total": projects.len(),
                    "running": projects.iter().filter(|p| p.running).count(),
                    "withProblems": projects.iter().filter(|p| !p.manifest_valid).count(),
                },
            }))
        }

        "stackvo_doctor" => Ok(json!(crate::doctor::run(Some(&root)).await)),

        "stackvo_log_files" => Ok(json!(crate::applog::candidates_all(&root)?)),

        "stackvo_projects" => Ok(json!(crate::commands::list_projects(&root).await?)),

        "stackvo_project" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;

            let projects = crate::commands::list_projects(&root).await?;
            let project = projects
                .into_iter()
                .find(|p| p.name == name)
                .ok_or_else(|| Error::not_found(format!("project {name}")))?;

            // The three questions that actually explain a site not loading,
            // answered together rather than left as three more tool calls.
            let certs = crate::certs::status(&root).await;
            let covered = project
                .domain
                .as_deref()
                .map(|d| crate::certs::covered_by(&certs.covered, d));

            Ok(json!({
                "project": project,
                "xdebug": crate::xdebug::status(&root, &name).await.ok(),
                // Carries `effective` — what PHP in the container actually has
                // — which turns "why is my upload failing at 2M" from a guess
                // into a reading.
                "phpIni": crate::phpini::status(&root, &name).await.ok(),
                "certificateCoversDomain": covered,
                "container": crate::engine::inspect(&name).await.ok(),
            }))
        }

        "stackvo_profiler" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            known_project(&root, &name).await?;

            let mut status = json!(crate::spx::status(&root, &name).await?);
            redact_profiler(&mut status);
            Ok(status)
        }

        "stackvo_hotspots" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            let key =
                string("key").ok_or_else(|| Error::new(Code::InvalidInput, "`key` is required"))?;
            known_project(&root, &name).await?;
            Ok(json!(crate::spx::analyse(
                &root,
                &name,
                &key,
                crate::spx::HOTSPOTS
            )?))
        }

        "stackvo_ide_debug" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            Ok(json!(crate::ide::status(&root, &name).await?))
        }

        "stackvo_services" => Ok(json!(crate::commands::list_services(&root).await?)),

        "stackvo_logs" => {
            use futures_util::StreamExt;

            let container = string("container")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`container` is required"))?;
            let tail = args
                .get("tail")
                .and_then(|v| v.as_u64())
                .unwrap_or(100)
                .clamp(1, 2000) as u32;

            // A container that is not there produces a stream that ends at
            // once, which is indistinguishable from one that has logged
            // nothing. `container_stats` already refuses the same name with a
            // `NotFound`, and two tools over the same id disagreeing about
            // whether it exists is worse than either answer.
            crate::engine::inspect(&container).await?;

            // follow: false, so the stream ends on its own at the last line.
            let stream = crate::engine::logs_stream(&container, tail, false)?;
            let lines: Vec<String> = stream.map(|line| line.text).collect().await;

            Ok(json!({ "container": container, "lines": lines }))
        }

        "stackvo_certificates" => Ok(json!(crate::certs::status(&root).await)),

        "stackvo_databases" => Ok(json!(crate::db::targets(&root).await?)),

        "stackvo_mail" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as u32;
            let status = crate::mail::status(&root).await?;
            let messages = if status.running {
                crate::mail::messages(&root, limit)
                    .await
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            Ok(json!({ "status": status, "messages": messages }))
        }

        "stackvo_mail_message" => {
            let id =
                string("id").ok_or_else(|| Error::new(Code::InvalidInput, "`id` is required"))?;

            // Asked first, because the alternative is what this used to do: a
            // transport error naming an unreachable `http://127.0.0.1:8025`,
            // which is true and tells an assistant nothing it can act on. The
            // catcher being off is an answer, not a fault.
            let status = crate::mail::status(&root).await?;
            if !status.running {
                return Err(Error::new(
                    Code::NotFound,
                    "the mail catcher is not running, so no message can be read",
                ));
            }

            Ok(json!(crate::mail::message(&root, &id).await?))
        }

        "stackvo_system" => {
            // Twice, because `Sampler` reports CPU as the delta between two
            // readings and the first one has nothing to subtract from. A third
            // of a second is the shortest gap that gives a figure worth
            // printing without making an assistant wait on it.
            let mut sampler = crate::stats::Sampler::new();
            let _ = sampler.sample();
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;

            Ok(json!({
                "host": sampler.sample(),
                // Both engine reads are allowed to be absent: this tool's whole
                // point is answering "why is the machine slow", and Docker
                // being down is one of the answers rather than a reason to fail.
                "docker": crate::engine::system_resources().await.ok(),
                "diskByOwner": crate::engine::disk_attribution().await.unwrap_or_default(),
            }))
        }

        "stackvo_container_stats" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            Ok(json!(crate::engine::container_stats(&name).await?))
        }

        "stackvo_service_instances" => {
            let table = crate::instances::Table::load(&root)?;
            let tree = crate::pkg::Tree::open(&crate::market::dir(&root))?;

            let rows: Vec<Value> = table
                .instances
                .iter()
                .map(|instance| {
                    json!({
                        "id": instance.id,
                        "service": instance.service,
                        "version": instance.version,
                        "enabled": instance.enabled,
                        "primary": instance.primary,
                        "container": instance.container(),
                        "aliases": instance.aliases(),
                        "ports": instance.ports,
                        "packagePresent": tree.dir(&instance.service, &instance.version).is_some(),
                    })
                })
                .collect();

            Ok(json!({ "instances": rows }))
        }

        "stackvo_service_connection" => {
            let service = string("service")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`service` is required"))?;

            // `connect::of` answers `None` for two different questions — "that
            // service has no connection string" and "there is no such service"
            // — and a tool that returned null for both would let a misspelt
            // name read as a service that simply cannot be connected to. The
            // first is an answer and stays null; the second is a mistake and
            // has to say so.
            known_service(&root, &service).await?;

            // `reveal: false`, hard-coded rather than taken from the arguments.
            // A `reveal` parameter would be a tool that hands a password to a
            // model on request, which is the one thing this surface does not do.
            Ok(json!({
                "service": service.clone(),
                "connection": crate::connect::of(&root, &service, false).await?,
            }))
        }

        "stackvo_hosts" => {
            let wanted = crate::commands::wanted_domains(&root).await;
            let (_, managed) = crate::hosts::mapped_domains();

            // Only StackVo's own block is reported stale. A line somebody added
            // by hand is theirs, and naming it here would invite an assistant
            // to propose removing it.
            let keep: std::collections::HashSet<String> =
                wanted.iter().map(|d| d.to_ascii_lowercase()).collect();
            let mut stale: Vec<String> =
                managed.into_iter().filter(|d| !keep.contains(d)).collect();
            stale.sort();

            Ok(json!({
                "hostsFile": crate::hosts::hosts_path(),
                "entries": crate::hosts::status_for(&wanted),
                "stale": stale,
            }))
        }

        "stackvo_log_read" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;

            // A project that does not exist has no log files, and so does a
            // project that has simply never written one. Answering both with an
            // empty list is how a typo becomes "this application writes no
            // logs" — the wrong conclusion, reached confidently.
            known_project(&root, &name).await?;

            let files = crate::applog::candidates(&root, &name)?;

            let Some(id) = string("id") else {
                return Ok(json!({ "project": name, "files": files }));
            };

            let bytes = args
                .get("bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(64 * 1024)
                .clamp(1024, 1024 * 1024);

            // `resolve` is the traversal check as well as the lookup: the id is
            // a string a model produced, and `..` in it must not reach the
            // filesystem.
            let path = crate::applog::resolve(&root, &name, &id)?;
            let (text, size) = crate::applog::tail(&path, bytes)?;

            Ok(json!({
                "project": name,
                "file": id,
                "fileBytes": size,
                "text": text,
                "files": files,
            }))
        }

        "stackvo_snapshots" => Ok(json!({ "snapshots": crate::snapshot::list(&root) })),

        "stackvo_packages" => {
            let tree = crate::pkg::Tree::open(&crate::market::dir(&root))?;
            let registry = crate::market::cached(&root)?;

            let packages: Vec<Value> = registry
                .iter()
                .flat_map(|registry| registry.packages.iter())
                .map(|package| {
                    let versions: Vec<Value> = package
                        .versions
                        .iter()
                        .map(|row| {
                            json!({
                                "version": row.version,
                                "installed": tree
                                    .dir(&package.service, &row.version)
                                    .is_some(),
                            })
                        })
                        .collect();

                    json!({
                        "service": package.service,
                        "category": package.category,
                        "capabilities": package.capabilities,
                        "versions": versions,
                    })
                })
                .collect();

            Ok(json!({
                // A workspace whose catalogue has never been fetched has no
                // packages and is not broken, so this says which case it is.
                "catalogueFetched": registry.is_some(),
                "packages": packages,
            }))
        }

        "stackvo_xdebug_set" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            let enabled = args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| Error::new(Code::InvalidInput, "`enabled` is required"))?;

            Ok(json!(crate::xdebug::set(&root, &name, enabled).await?))
        }

        "stackvo_certificates_reissue" => Ok(json!(crate::certs::apply(&root, true).await?)),

        // ---- operations, headless ------------------------------------------
        //
        // These were unreachable from this server while the runner was welded
        // to AppHandle. `progress::Null` drops the progress events (there is
        // no window to receive them); the outcome is the return value, which
        // is what an MCP client wants anyway. `run_operation` awaits the
        // process to completion, so a reply here means the work is done, not
        // merely started.
        "stackvo_project_start" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            // Named before the engine is asked, so a misspelt project reads as
            // "no such project" rather than as "no such container" — which is
            // what a project that exists and has never been built says, and is
            // a different problem with a different fix.
            known_project(&root, &name).await?;
            crate::engine::start_container(&name).await?;
            Ok(json!({ "project": name, "running": true }))
        }

        "stackvo_project_stop" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            known_project(&root, &name).await?;
            crate::engine::stop_container(&name).await?;
            Ok(json!({ "project": name, "running": false }))
        }

        "stackvo_stack_up" => {
            let mode = string("mode").unwrap_or_else(|| "minimal".into());
            let mut cmd_args = crate::runner::compose_base_args(&root);
            cmd_args.extend(crate::runner::profile_args(&mode, &[])?);
            cmd_args.extend([
                "up".into(),
                "-d".into(),
                "--build".into(),
                "--pull=missing".into(),
                "--remove-orphans".into(),
            ]);
            run_headless("up", &mode, "docker", &cmd_args, &root).await?;
            Ok(json!({ "mode": mode, "up": true }))
        }

        "stackvo_stack_down" => {
            let mut cmd_args = crate::runner::compose_base_args(&root);
            cmd_args.extend([
                "--profile".into(),
                "core".into(),
                "--profile".into(),
                "services".into(),
                "--profile".into(),
                "projects".into(),
                "down".into(),
            ]);
            run_headless("down", "stack", "docker", &cmd_args, &root).await?;
            Ok(json!({ "down": true }))
        }

        "stackvo_project_restart" => {
            let name = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;
            known_project(&root, &name).await?;
            crate::engine::stop_container(&name).await?;
            crate::engine::start_container(&name).await?;
            Ok(json!({ "project": name, "running": true, "restarted": true }))
        }

        // ---- service instances ---------------------------------------------
        //
        // `lifecycle` is the same function the window path calls; what differs
        // is the sink. `progress::Null` is why it can be called at all from
        // here — the split that made `AppHandle` stop being a parameter of the
        // work is what put service control on this surface, and it is the gap
        // every rival's MCP server filled a year before this one.
        "stackvo_service_start" | "stackvo_service_stop" | "stackvo_service_restart" => {
            let id =
                string("id").ok_or_else(|| Error::new(Code::InvalidInput, "`id` is required"))?;

            // Looked up before anything is driven, so an unknown id is a clear
            // "no such instance" rather than a compose error about a service
            // that is not in the file.
            let table = crate::instances::Table::load(&root)?;
            let instance = table
                .instances
                .iter()
                .find(|i| i.id == id)
                .ok_or_else(|| Error::not_found(format!("instance {id}")))?;

            if !instance.enabled {
                return Err(Error::new(
                    Code::InvalidInput,
                    format!("{id} is switched off, so it has no container to control"),
                ));
            }

            let phase = match name {
                "stackvo_service_start" => crate::events::START,
                "stackvo_service_stop" => crate::events::STOP,
                _ => crate::events::RESTART,
            };
            crate::commands::lifecycle(&crate::progress::Null, "instance", &id, phase).await?;

            Ok(json!({
                "instance": id,
                "running": name != "stackvo_service_stop",
            }))
        }

        "stackvo_snapshot_take" => {
            let service = string("service")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`service` is required"))?;
            let requested = string("name")
                .ok_or_else(|| Error::new(Code::InvalidInput, "`name` is required"))?;

            // The same two checks the app makes, in the same order: the name is
            // sanitised before it becomes a path, and an existing snapshot is
            // never written over — a backup silently replaced by a newer one is
            // the failure this whole feature exists to prevent.
            let snapshot = crate::snapshot::safe_name(&requested)?;
            let path = crate::snapshot::path_for(&root, &service, &snapshot)?;
            if path.exists() {
                return Err(Error::new(
                    Code::AlreadyExists,
                    format!("a {service} snapshot called `{snapshot}` already exists"),
                )
                .with_hint(crate::hints::SNAPSHOT_NAME_IN_USE));
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
            }

            let bytes = crate::db::dump(&root, &service, &path, |_| {}).await?;
            Ok(json!({
                "service": service,
                "snapshot": snapshot,
                "path": path.display().to_string(),
                "bytes": bytes,
            }))
        }

        "stackvo_generate" => {
            // The Rust writer directly — same renderer the app uses, no shell.
            let scope = string("scope").unwrap_or_else(|| "all".into());
            let report = crate::commands::write_generated(&root, &scope, |_| {})?;
            Ok(report)
        }

        // Unreachable while the table and this match agree, which is what the
        // test below is for.
        _ => Err(Error::new(
            Code::Unsupported,
            format!("{name} is listed but not implemented"),
        )),
    }
}

/// Refuse a project name this workspace does not have.
///
/// The names on this surface come from a model, not from a list somebody
/// clicked, so "no such project" is a case rather than an edge case — and
/// several of the underlying readers answer a missing project with an empty
/// result, which reads as a fact about the project rather than about the name.
async fn known_project(root: &std::path::Path, name: &str) -> Result<()> {
    let projects = crate::commands::list_projects(root).await?;
    if projects.iter().any(|p| p.name == name) {
        return Ok(());
    }
    Err(crate::error::Error::not_found(format!("project {name}")))
}

/// The same for a service or instance id.
///
/// Both spellings are accepted because both are real: `services_list` is the
/// catalogue view somebody reads, `instance_list` is the per-version view the
/// control tools take, and a caller holding either should not have to know
/// which one this happens to consult.
async fn known_service(root: &std::path::Path, id: &str) -> Result<()> {
    if crate::instances::Table::load(root)
        .map(|table| table.instances.iter().any(|i| i.id == id))
        .unwrap_or(false)
    {
        return Ok(());
    }
    if crate::commands::list_services(root)
        .await
        .map(|services| services.iter().any(|s| s.id == id))
        .unwrap_or(false)
    {
        return Ok(());
    }
    Err(crate::error::Error::not_found(format!("service {id}")))
}

/// One headless operation: same runner, same argv the window path builds, no
/// events. The operation id keeps log correlation working.
async fn run_headless(
    prefix: &str,
    subject: &str,
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
) -> Result<()> {
    let operation_id = crate::events::next_operation_id(prefix);
    // The Tauri-free sink, so this whole path names no Tauri type at all —
    // `events::Sink::Headless` did the same job but dragged `AppHandle` into
    // the signature it lives in.
    crate::runner::run_operation(
        &crate::progress::Null,
        crate::runner::Operation {
            operation_id: &operation_id,
            subject,
            // Names are still required by the struct; nothing receives them.
            progress_event: "mcp:progress",
            finished_event: "mcp:done",
            program,
            args,
            cwd,
            env: &[],
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The check that makes the table trustworthy: every tool names a command
    /// the contract actually declares. Without it, a command renamed in
    /// `ipc.json` leaves a tool describing something that no longer exists, and
    /// nothing says so.
    #[test]
    fn every_tool_names_a_real_contract_command() {
        let ipc = crate::contracts::ipc();
        let commands = ipc["commands"]
            .as_object()
            .expect("the contract declares commands");

        for tool in TOOLS {
            assert!(
                commands.contains_key(tool.command),
                "{} names `{}`, which is not in contracts/ipc.json",
                tool.name,
                tool.command
            );
        }
    }

    /// The declared reach has to be real too, or the surface that intersects
    /// over it is intersecting over a typo.
    #[test]
    fn every_command_a_tool_also_reaches_is_a_real_one() {
        let ipc = crate::contracts::ipc();
        let commands = ipc["commands"]
            .as_object()
            .expect("the contract declares commands");

        for tool in TOOLS {
            for command in tool.also {
                assert!(
                    commands.contains_key(*command),
                    "{} declares it reaches `{command}`, which is not in contracts/ipc.json",
                    tool.name
                );
                assert_ne!(
                    *command, tool.command,
                    "{} lists its own command in `also`",
                    tool.name
                );
            }
        }
    }

    /// `also` is only worth having if something is in it. A table that drifted
    /// to all-empty would pass every check above while the loopback surface
    /// went back to intersecting over one command each.
    #[test]
    fn the_declared_reach_is_not_empty_everywhere() {
        assert!(
            TOOLS.iter().any(|t| !t.also.is_empty()),
            "no tool declares reaching a second command, which cannot be true \
             while stackvo_project reads the certificate and the PHP limits"
        );
    }

    /// A read-only tool backed by a mutating command is the failure this whole
    /// arrangement exists to prevent: an assistant told a call was safe, making
    /// it, and changing the user's stack.
    #[test]
    fn read_only_tools_are_backed_by_read_only_commands() {
        let ipc = crate::contracts::ipc();

        for tool in TOOLS.iter().filter(|t| !t.writes) {
            let kind = ipc["commands"][tool.command]["kind"]
                .as_str()
                .unwrap_or("unknown");
            assert!(
                matches!(kind, "query" | "stream"),
                "{} is advertised read-only but `{}` is declared `{kind}`",
                tool.name,
                tool.command
            );
        }
    }

    /// And the converse: a writing tool must not be backed by a query, which
    /// would mean the confirmation gate is guarding nothing.
    #[test]
    fn writing_tools_are_backed_by_mutating_commands() {
        let ipc = crate::contracts::ipc();

        for tool in TOOLS.iter().filter(|t| t.writes) {
            let kind = ipc["commands"][tool.command]["kind"]
                .as_str()
                .unwrap_or("unknown");
            assert!(
                matches!(kind, "mutation" | "operation"),
                "{} is gated behind --allow-writes but `{}` is only a `{kind}`",
                tool.name,
                tool.command
            );
        }
    }

    #[test]
    fn tool_names_are_unique() {
        let mut names: Vec<&str> = TOOLS.iter().map(|t| t.name).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two tools share a name");
    }

    /// The default is read-only. A server that offered its writing tools
    /// unless told not to would be one flag away from an assistant rebuilding
    /// somebody's stack unasked.
    #[test]
    fn writing_tools_are_hidden_unless_asked_for() {
        let listed = tools_list(false);
        let names: Vec<&str> = listed["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();

        assert!(names.contains(&"stackvo_projects"));
        assert!(!names.contains(&"stackvo_xdebug_set"));
        assert!(
            TOOLS.iter().any(|t| t.writes),
            "the gate needs something to gate"
        );

        let with_writes = tools_list(true);
        assert!(with_writes["tools"].as_array().unwrap().len() > names.len());
    }

    /// Every advertised tool is annotated, so a client can decide about a tool
    /// it has never seen.
    #[test]
    fn tools_carry_read_only_annotations() {
        let listed = tools_list(true);
        for entry in listed["tools"].as_array().unwrap() {
            let name = entry["name"].as_str().unwrap();
            let tool = TOOLS.iter().find(|t| t.name == name).unwrap();
            assert_eq!(entry["annotations"]["readOnlyHint"], json!(!tool.writes));
            assert_eq!(entry["annotations"]["destructiveHint"], json!(tool.writes));
        }
    }

    #[tokio::test]
    async fn initialize_reports_the_protocol_and_the_tool_capability() {
        let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" });
        let response = handle(&request, false)
            .await
            .expect("initialize is answered");

        assert_eq!(response["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(response["result"]["serverInfo"]["name"], "stackvo");
        assert!(response["result"]["capabilities"]["tools"].is_object());
    }

    /// Replying to a notification is a protocol error some clients treat as
    /// fatal, and `notifications/initialized` arrives on every session.
    #[tokio::test]
    async fn notifications_are_not_answered() {
        let request = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(handle(&request, false).await.is_none());
    }

    #[tokio::test]
    async fn an_unknown_method_is_a_json_rpc_error_not_a_panic() {
        let request = json!({ "jsonrpc": "2.0", "id": 7, "method": "resources/list" });
        let response = handle(&request, false).await.unwrap();
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["id"], 7);
    }

    /// A refused write comes back as tool output, not a transport error: the
    /// assistant has to be able to read the reason and say it.
    #[tokio::test]
    async fn a_write_refused_in_read_only_mode_explains_itself() {
        let request = json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "stackvo_xdebug_set", "arguments": { "name": "shop", "enabled": true } }
        });

        let response = handle(&request, false).await.unwrap();
        assert_eq!(response["result"]["isError"], json!(true));

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("read-only"), "got: {text}");
        assert!(text.contains("--allow-writes"), "the fix has to be named");
    }

    /// The handshake the constant used to break. A client that asks for the
    /// revision it implements has to be answered with that revision; answering
    /// with a newer one is grounds for it to disconnect, which reads to the
    /// user as "the server does not work" with nothing in any log.
    #[tokio::test]
    async fn initialize_answers_in_the_revision_the_client_asked_for() {
        for asked in PROTOCOL_VERSIONS {
            let request = json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": { "protocolVersion": asked }
            });
            let response = handle(&request, false).await.unwrap();
            assert_eq!(response["result"]["protocolVersion"], json!(asked));
        }
    }

    /// And a revision this does not know falls back to ours rather than
    /// echoing something unimplemented back at the client.
    #[test]
    fn an_unknown_revision_falls_back_to_this_one() {
        assert_eq!(negotiate(Some("1999-01-01")), PROTOCOL_VERSION);
        assert_eq!(negotiate(None), PROTOCOL_VERSION);
        assert_eq!(PROTOCOL_VERSIONS[0], PROTOCOL_VERSION, "newest first");
    }

    /// The profiler's control URL carries its key, and this surface returns no
    /// credentials — however cheap the credential is, and however local the
    /// thing it unlocks. Asserted on the rendered answer rather than on the
    /// struct, because what matters is what leaves the process: the same
    /// dispatch feeds the loopback HTTP surface.
    #[tokio::test]
    async fn the_profiler_tool_does_not_hand_back_its_key() {
        // The real shape, with both fields that carry the key: the panel URL
        // and the base a report key is appended to. `viewBase` was added after
        // this test existed, and a test that re-implemented the redaction would
        // have kept passing while the second field walked straight out.
        let mut status = json!({
            "supported": true, "enabled": true, "built": true,
            "controlUrl": "https://shop.loc/?SPX_KEY=s3cret&SPX_UI_URI=/",
            "viewBase": "https://shop.loc/?SPX_KEY=s3cret&SPX_UI_URI=/report.html&key=",
            "reports": [{ "key": "spx-full-1-abc" }],
        });

        redact_profiler(&mut status);
        let text = serde_json::to_string(&status).unwrap();
        assert!(
            !text.contains("s3cret"),
            "the key survived redaction: {text}"
        );
        assert!(!text.contains("SPX_KEY"), "{text}");
        // And what is left is still useful: the recordings, and where to open
        // the one thing that needed the key.
        assert!(text.contains("spx-full-1-abc"), "{text}");
        assert!(text.contains("controlPanel"), "{text}");

        // A document with neither field is left exactly as it was, rather than
        // gaining a sentence about a panel nobody asked about.
        let mut plain = json!({ "supported": false });
        redact_profiler(&mut plain);
        assert_eq!(plain, json!({ "supported": false }));

        // And the tool is declared read-only, so nothing else about it can put
        // the key back through a write.
        let tool = TOOLS
            .iter()
            .find(|t| t.name == "stackvo_profiler")
            .expect("the profiler tool exists");
        assert!(!tool.writes);
    }

    /// No tool takes a `reveal`, and none may.
    ///
    /// `service_connection` has one in the IPC surface — the app shows a
    /// password on a click, for one service, to the person sitting there. The
    /// same argument on this surface would be a tool that hands a stored
    /// credential to a model on request. The schema is asserted rather than
    /// the call, because the way this comes back is somebody adding the
    /// parameter for symmetry with the command it names.
    #[test]
    fn no_tool_offers_to_reveal_a_credential() {
        for tool in TOOLS {
            let schema = (tool.schema)();
            let properties = schema["properties"].as_object().expect("an object schema");
            for forbidden in ["reveal", "password", "secret", "token"] {
                assert!(
                    !properties.contains_key(forbidden),
                    "{} takes `{forbidden}`",
                    tool.name
                );
            }
        }
    }

    /// Every tool that names a thing takes the id the *listing* tool prints.
    /// An instance tool that took a service name would work for `redis` on a
    /// machine with one Redis and fail on the machine that made instancing
    /// worth building.
    #[test]
    fn instance_tools_ask_for_an_instance_id() {
        for name in [
            "stackvo_service_start",
            "stackvo_service_stop",
            "stackvo_service_restart",
        ] {
            let tool = TOOLS.iter().find(|t| t.name == name).expect(name);
            let schema = (tool.schema)();
            assert_eq!(schema["required"], json!(["id"]), "{name}");
        }
    }

    #[tokio::test]
    async fn an_unknown_tool_is_reported_as_tool_output() {
        let request = json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "stackvo_delete_everything", "arguments": {} }
        });

        let response = handle(&request, true).await.unwrap();
        assert_eq!(response["result"]["isError"], json!(true));
    }
}
