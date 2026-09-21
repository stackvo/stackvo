# AI assistants

`stackvo-mcp` is an MCP server over the same core the window drives. An
assistant can answer *"why is shop.loc not loading?"* from the preflight
report, the hosts file, the certificate and the container's last hundred log
lines — with no window open.

<figure markdown>
![The AI assistant card of a project](../screenshots/web/project-detail-agent.webp){ loading=lazy }
<figcaption>What an assistant is told about this project, and what it may do.</figcaption>
</figure>

## Registering it

**Settings → AI assistants** lists the eight clients found on this machine —
Claude Code, Claude Desktop, Cursor, Windsurf, VS Code, Gemini CLI, Codex,
Zed — and registers the server in one click. Each client's own config file is
read, one `stackvo` entry is inserted, and the file is written back with every
other server in it intact. A `.stackvo-backup` copy is kept first.

## The leash: read-only by default

You grant access with switches, and the pane writes back the sentence your
choice amounts to — *"this assistant may restart shop, for the next half
hour"*.

| Setting | Effect |
| --- | --- |
| *(default)* | Reads only. 26 of the 38 tools. |
| **Allow writes** | Adds the 12 mutating tools — start, stop, restart, reissue the certificate, take a snapshot, and `stack_down`, which stops everything. |
| **Bound to a project** | Only that project's four writing tools are offered; the eight no project can bound are not offered at all. |
| **Time limit** | The writing half ends by itself after the time you set. |
| **Tool by tool** | Only the tools you name. |

Read the list before allowing writes: it includes stopping the whole stack and
stopping a shared service every project depends on.

## What it never does

- **No tool returns a password.** A test asserts no schema on this surface
  has a `password`, `secret` or `token` property.
- **Restoring a snapshot is not a tool.** Taking one is — it adds a file.
  Writing over live rows belongs to the app's own confirmation.
- **Every writing call is logged**, refusals included, in **Settings →
  Audit trail** — most entries carrying what would put the act back, so it can be
  reversed in one click.

## Telling the assistant when to use it

**Settings → AI rules** writes a short section into the instruction file the
assistant already reads — `CLAUDE.md`, `AGENTS.md`, `.cursor/rules/`,
`.github/instructions/`, `.windsurf/rules/`, `GEMINI.md`. Only the region
between StackVo's own markers is ever written; the rest of the file comes
back byte for byte.

## A sandbox for the task

Give the assistant a [worktree with an expiry](branches.md) rather than your
machine: its own branch, its own address, its own database, gone when the
time is up.
