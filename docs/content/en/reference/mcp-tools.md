# MCP tools

The 38 tools `stackvo-mcp` offers an assistant. 26 only read; 12 change
things and appear only with **Allow writes** — see
[AI assistants](../guide/ai-assistants.md) for the switches. Every tool is
annotated read-only or destructive, so a client can ask before using one it
has not seen.

<figure markdown>
![AI assistants in the settings](../screenshots/web/settings-agents.webp){ loading=lazy }
<figcaption>Which assistants have the MCP server registered, and the leash on writes.</figcaption>
</figure>

## Reads

| Tool | What it answers |
| --- | --- |
| `stackvo_overview` | The state of the whole stack: the workspace, the engine, what is up. |
| `stackvo_doctor` | The full diagnosis: every requirement, every held port, missing hosts lines. |
| `stackvo_system` | What this machine has left: CPU, memory, swap, disks, network. |
| `stackvo_projects` | Every managed project: domain, runtime, built, running. |
| `stackvo_project` | One project in full. |
| `stackvo_container_stats` | One container's live CPU, memory against its limit, network and block I/O. |
| `stackvo_services` | The shared services — databases, caches, search, queues — and their health. |
| `stackvo_service_instances` | Every installed service version as a separately controllable instance. |
| `stackvo_service_connection` | How to reach one service: scheme, host, port, database, user. No password. |
| `stackvo_databases` | The database services that can be dumped, and their databases. |
| `stackvo_snapshots` | The snapshots this workspace holds, newest first. |
| `stackvo_packages` | The service catalogue: every package the registry knows. |
| `stackvo_logs` | The last lines of a container's log. |
| `stackvo_log_files` | Every log file every project writes, newest first. |
| `stackvo_log_read` | One project's log files, and one file's contents. |
| `stackvo_certificates` | The certificate: covered domains, missing ones, trust. |
| `stackvo_hosts` | Every domain the stack wants, and whether the hosts file maps it. |
| `stackvo_mail` | The mail catcher's inbox. |
| `stackvo_mail_message` | One caught message in full. |
| `stackvo_ide_debug` | Why a breakpoint is not being hit: port, IDE key, server name, who is listening. |
| `stackvo_profiler` | The sampling profiler for one project: built, mounted, on. |
| `stackvo_hotspots` | Where one recording spent its time. |
| `stackvo_flame` | One recording as a flame graph. |
| `stackvo_explain_request` | Why one recorded request was slow, the three instruments joined. |
| `stackvo_timeline` | Everything the application reported, on one axis: dumps, requests, jobs. |
| `stackvo_query_log` | What one database was actually asked. |

## Writes

Only with **Allow writes**. Under a project scope, only the first four are
offered.

| Tool | What it does | Scope |
| --- | --- | --- |
| `stackvo_project_start` | Start one project's container. Idempotent. | project |
| `stackvo_project_stop` | Stop one project's container. Idempotent. | project |
| `stackvo_project_restart` | Stop and start one project. | project |
| `stackvo_xdebug_set` | Turn step debugging on or off for one project. | project |
| `stackvo_service_start` | Start one service instance — `redis-7-2`, not `redis`. | stack |
| `stackvo_service_stop` | Stop one instance. Nothing is deleted. | stack |
| `stackvo_service_restart` | Restart one instance. | stack |
| `stackvo_snapshot_take` | Dump one database into a named snapshot. Adds a file, changes nothing. | stack |
| `stackvo_certificates_reissue` | Reissue the certificate for the domains the projects have. | stack |
| `stackvo_generate` | Re-run the generator. | stack |
| `stackvo_stack_up` | Bring the stack up, building missing images. | stack |
| `stackvo_stack_down` | Bring the whole stack down, projects included. | stack |

## Not tools, on purpose

- **Restoring a snapshot.** Writing over live rows belongs to the app's own
  confirmation.
- **Anything that returns a password.** A test asserts no schema on this
  surface has a `password`, `secret` or `token` property.
- **Building a project.** Build progress streams through the app's event
  system, which a stdio server has no way to carry.
