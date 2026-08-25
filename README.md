# StackVo Desktop

A Docker-based local development environment manager, as a native desktop app.

**Self-contained.** It used to require a clone of
[StackVo](https://github.com/stackvo/stackvo) to read its generator and
templates from. The generator was ported to Rust (the shell was deleted), and
the service templates now ship inside the binary — so a workspace is a folder
this app creates, not one you have to fetch. Point it at an empty directory and
it writes the `.env`, the templates and the project tree itself. An existing
checkout still works and is left exactly as it is.

## Why a desktop app

In StackVo today the dashboard is itself a container: it runs inside the Docker stack it manages,
reaches the engine through a mounted `docker.sock`, is routed by Traefik at `stackvo.loc`, and needs
a hosts-file entry and a self-signed-certificate click-through to open. That one decision is where
most of the friction comes from — the UI can't tell you Docker is down (it needs Docker to run), it
can't stop the stack (that would kill itself), it writes files as root and chowns them back, and it
reads host CPU stats from inside a container, where they're wrong.

StackVo Desktop inverts the relationship. The app runs on the host as a normal user process and
drives Docker directly. Traefik and the project/service containers are unchanged — only the control
plane moves.

|                    | Web UI (today)                           | Desktop                            |
| ------------------ | ---------------------------------------- | ---------------------------------- |
| Runs as            | container, root, `chmod 666 docker.sock` | host process, invoking user        |
| Docker down        | unreachable                              | opens, reports, offers to start it |
| Host metrics       | `/proc` inside a container               | `sysinfo` on the host              |
| Stopping the stack | impossible (kills the UI)                | `compose_down`                     |
| Hosts file         | manual `sudo tee -a /etc/hosts`          | reviewed diff, one elevated write  |
| Windows            | WSL2 only                                | native — no shell, no bash         |
| Install size       | ~600 MB of images                        | ~27 MB, CLI included               |

## Status

**Phase 0 — contracts.** Complete. See [contracts/](contracts/).
**Phase 1 — skeleton + read-only views.** Complete. The app runs, finds a StackVo
checkout, reports the Docker engine, and reads real host metrics.
**Phase 2 — control.** Complete. Start/stop/restart/build projects and services,
enable/disable services, live container logs, and stack-wide `up`/`down` — all
with streamed progress instead of a blocked request.
**Phase 3 — desktop integration.** Complete. Tray, native notifications, a
watcher on `projects/*/stackvo.json`, an elevated hosts-file helper that shows a
diff first, real terminals (container _and_ host), autostart and single-instance.

**Phase 4 — generator port.** Complete. All five web servers, the Node runtime,
`docker-compose.projects.yml` and both Traefik files are ported to Rust and
verified byte-for-byte against the Bash generator. Windows path and named-pipe
handling is written and unit-tested; see the caveat below.

**Phase 5 — releases.** Signed auto-updates are wired: the app checks an
endpoint, verifies the bundle signature against the key compiled into the build,
installs and restarts. `.github/workflows/release.yml` builds and signs for four
targets. **Two things are yours to supply** and are deliberately absent from
this repo — the signing key pair and the endpoint that serves `latest.json`:

```bash
npm run tauri signer generate -- -w ~/.tauri/stackvo.key
```

Put the public half in `src-tauri/tauri.conf.json` → `plugins.updater.pubkey`,
and the private half in the `TAURI_SIGNING_PRIVATE_KEY` repository secret.
Without both, builds produce unsigned artifacts that the updater refuses — the
correct failure, not a bug to route around.

Verification is differential, not by inspection — a generator that produces
"basically the same" output silently changes people's images:

- `tools/make-fixtures.sh` runs the real Bash generator in a throwaway sandbox
  (a copy of `core/` + `.env`, never the user's projects) and freezes its output
  as fixtures, one per server.
- `tests/fixtures_differential.rs` compares the Rust renderer against them.
- `npm run diagnose` runs the same comparison **live** against your own
  projects, and reports which match.

### Taking the generator over

The Rust generator runs _alongside_ the Bash one; it does not replace it.
`generate_with` has three modes:

| Mode     | Behaviour                                                          |
| -------- | ------------------------------------------------------------------ |
| `bash`   | What StackVo does today. The default.                              |
| `verify` | Bash writes; the Rust port renders the same files and is compared. |
| `rust`   | Refuses to write unless the two agree byte-for-byte.               |

Bash runs in every mode. The generator's output is the input to every container
you run, so "probably identical" is not a standard worth shipping — `rust` mode
cannot silently change an image, because a disagreement stops it.

### Windows status

The pure logic — drive-letter to bind-mount conversion (`C:\Users\me` →
`/c/Users/me`), named-pipe detection, `DOCKER_HOST` scheme stripping — lives in
`src-tauri/src/paths.rs` with no `cfg` gates, so its tests run on every
platform. That is deliberate: Windows behaviour verified only on Windows is
Windows behaviour nobody verifies.

What is **not** verified: the handful of `#[cfg(target_os = "windows")]` blocks
in `engine.rs`, `hosts.rs` and `pty.rs`. Cross-compiling from macOS fails in
`tauri-build`, which needs `llvm-rc` to embed the app manifest — a toolchain
gap, not a code error, but it means those blocks have never been compiled. They
need a real Windows machine or a CI runner before anyone claims Windows works.

```bash
npm install
npm run tauri:dev      # run the app

npm test               # everything: vitest + Rust unit, integration, differential
npm run test:js        # front end only
npm run lint           # eslint + prettier
npm run audit          # cargo-deny + npm audit
npm run contracts:check
npm run diagnose       # headless end-to-end check
npm run bundle:budget  # raw asset sizes against tools/bundle-budget.mjs
```

`cargo bench` measures the generator's render path. It is deliberately not in
CI: a hosted runner's variance is wider than the regressions a threshold would
be set to catch, so it is an instrument you run on a quiet machine rather than
a gate. The bundle budget is a gate for the opposite reason — bytes are the
same on every machine.

CI runs all of these on Linux, macOS and Windows, with `cargo clippy -- -D
warnings` and `cargo fmt --check`. The Rust toolchain is pinned in
`src-tauri/rust-toolchain.toml`, so a new stable release cannot turn the build
red without a commit.

### Driving it from a terminal

`stackvo` is a command-line interface over the same core the window drives.

```bash
stackvo path-install          # link it into the app's own directory, and onto PATH
stackvo status
stackvo logs shop --follow
stackvo doctor --json | jq '.ports[] | select(.state != "ok")'
```

Both `stackvo` and `stackvo-mcp` **ship inside the app** — `externalBin` in
`tauri.conf.json`, built by `tools/sidecars.mjs`, landing beside the main binary
in `Contents/MacOS/`. `path-install` is the step that used to be "remember where
you built it": it links them into a directory the app owns and writes one line,
between markers and after a backup, into your shell's startup file. `stackvo
tools` shows the whole state and `stackvo path-remove` takes the line back out.
Settings → Tooling is the same thing with buttons.

From a checkout, `npm run sidecars` builds them; `cargo build --release --bin
stackvo` still works and `tooling.rs` finds either copy.

`--help` splits them by what they do: sixteen that read, twenty-one that change
the stack, one screen, nineteen that run a program in the project's own
container, and two for shell completion. Every one takes `--json`, and the table you see is
rendered _from_ that value rather than from a second query, so the two cannot
come to describe different things.

**Tab completion, in all four shells.** `stackvo path-install` writes it into
the same marked block as the `PATH` line, so one `path-remove` takes both back
out; `stackvo completions zsh` prints the stub on its own for a package
manager. The shell side is four lines and knows nothing — it collects what has
been typed and asks `stackvo complete`, which answers from the same table
`--help` is rendered from. So a command added to the CLI is completable the
moment it exists, and one removed stops being offered; there is no second copy
of the command list in a language no test reads.

It completes commands, flags, and the positionals whose placeholder names a
list this app already keeps — `<project>` from your own projects, `<client>`,
`<target>`, `<tool>`, `[shell]`, and a literal `on|off`. It deliberately offers
**nothing** after a passthrough: `stackvo artisan migrate --<TAB>` must not
suggest `--json`, because the parser stopped at `artisan` and that flag would
reach artisan instead. `examples/completion_probe.rs` drives real bash and zsh
and checks all of it, which is not ceremony — the first version of the bash
stub narrowed `IFS` before expanding the word list, and on the bash macOS ships
that silently collapsed `artisan migrate` into one word. Every Rust test passed
while it did.

**stdout is the answer, stderr is the narration.** A compose build scrolls past
on stderr while `--json` stays clean on stdout, and a failure leaves stdout
empty with a non-zero status. The exit code separates the two failures a script
wants to handle rather than report: `3` is "nothing is set up on this machine",
`4` is "Docker is not running"; `2` is a bad command line and `1` is everything
else.

An unrecognised flag is an **error**, not a shrug — a tool that ignores
`--tial 50` and uses the default has told you it did something it did not do.

**Where packages come from, and how you know.** The index is checked before it
is parsed — a minisign signature over `registry.json`, against keys the machine
already trusts, then a sha256 per manifest, then a sha256 per file. The official
registry key is pinned — its own key pair, never the updater's, so a leak of
either forges one thing and not both — and the published index is signed with
its private half, so the chain runs end to end.

Which signatures get checked is the publisher's decision rather than a setting
you have to find. A signature that is there is checked, and a check that fails
is a refusal — never a quiet fall-through to "unsigned, then". A signature that
is *not* there is accepted only from a source that has never given one: once
this machine has taken a verified index from a source, that source going back
to unsigned is refused, because anyone who can serve a tampered index can also
serve a 404 for its signature. The official catalogue is known to sign, so that
holds on a first refresh too, before this machine has learned anything. `policy.market.requireSignature` still only
tightens — it refuses a missing signature too. An organisation running its own
mirror never waited on any of this: it signs its own index and pins its own key
with `policy.market.additionalKeys`. Keys rotate through a `known-keys.json` signed by a key
already trusted, and a key retired by a build cannot be brought back by any
document. A version its publisher has withdrawn is refused at install, and
`stackvo doctor` lists withdrawn versions this machine already has.

**The commands this project runs.** The built-in catalogue is what most
projects have; a project declares the rest in its own `stackvo.json`:

```json
"commands": {
  "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Rebuild the search index" }
}
```

`stackvo commands` lists both, `stackvo run reindex` runs one, and the project
pane shows them together with the declared ones marked. They run **in the
project's container and nowhere else** — there is no `host` form, which is why
this needed no approval prompt: a container already runs the repository's code.
A step that has to touch your machine is a hook, where it is approved against a
digest first.

**A screen, when one command at a time is not what you want.**

```bash
stackvo tui
```

Every project and service, live, with the cursor on one of them: enter starts
or stops it, `l` shows its last lines, `q` leaves. There is no TUI library
behind it — `ratatui` was measured at 25 new packages for a list, a detail line
and a status bar, so the drawing is the same column arithmetic the tables use
and raw mode is one call each to `libc` and `windows-sys`, both already in the
lock file. `examples/tui_probe.rs` runs it in a real pty and reads the
terminal's settings back afterwards, because the failure mode of getting this
wrong is somebody's shell.

**The project's container, from the working directory.** `cd` into a project and:

```bash
stackvo php -v          # the PHP that project declares, on a host with none
stackvo artisan migrate --force
stackvo composer install
stackvo wp plugin list  # and console, rails, bundle, yarn, pnpm
stackvo python -V       # and ruby, go, cargo, bun, deno
stackvo shell           # an interactive shell in the container
stackvo exec <program>  # anything else
```

**Every runtime this app generates a container for has a row**, not just PHP and
Node — `manifest::LANG_RUNTIMES` names six more and `cli_surface.rs` fails the
build if one of them has no way to be run. The images are built in a single
stage, so `stackvo cargo test` reaches a cargo that is still there. The
framework rows come from `quickcmd`'s catalogue rather than from a README, which
is why `wp` carries `--allow-root` (the container runs as root and wp-cli
refuses without it) and why there is no `drush` row: nothing in this app says
how Drupal is driven, and a row that usually fails is worse than no row.

Running one that is not there exits **127**, the way a shell does, and adds one
line naming what the project actually is — Docker's own message says `python`
is not on `PATH` and cannot know that this is a PHP project.

Which project comes from the working directory — matched against the real
project list rather than a folder name, deepest first, so a worktree wins over
the project it sits inside. `--project` names another from anywhere. Standing in
`app/Http` runs there, in `/var/www/html/app/Http`, so it behaves the way the
command would on the host.

Everything after the command name is passed on untouched, which is what makes
`artisan migrate --force` work — so StackVo's own flags go before it, and
`stackvo --help artisan` (rather than `stackvo artisan --help`, which reaches
artisan) prints this app's usage. The exit code is passed through, so
`stackvo artisan test` means what it says in a CI script.

Like the MCP tool table, every command names the `contracts/ipc.json` command it
implements, and `cli_surface.rs` cross-checks the pair: a command naming
something the contract does not declare fails the build, and so does a command
listed under "Reads" whose contract command is a mutation. Dispatch matches on
an enum rather than on the command's name, so a command in the table with no
implementation does not compile.

### Driving it from an AI assistant

`stackvo-mcp` is an MCP server over the same core the app drives, so an
assistant can answer "why is shop.loc not loading?" from the preflight report,
the hosts file, the certificate's SAN list and a container's last hundred log
lines — without a window open.

```bash
cargo build --release --bin stackvo-mcp
```

Then **Settings → AI assistants**, which lists all eight — Claude Code, Claude
Desktop, Cursor, Windsurf, VS Code, the Gemini CLI, Codex and Zed — says which
of them are on this machine and which already point at the server, and
registers it in one click. `stackvo mcp` prints the same table in a terminal
and `stackvo mcp-install <id>` does the same write.

It reads each client's own configuration file, inserts a single `stackvo` entry
and writes it back, so every other server in that file survives; a copy is kept
beside it as `.stackvo-backup` first. Codex is TOML and is edited with a
format-preserving editor rather than a serialiser, so comments, key order and
quoting come back as they were. A file that is JSON _with comments_ — VS Code's
format — is reported rather than rewritten, with the block to paste, because
stripping the comments to make the edit possible would delete the reader's own
notes:

```json
{ "mcpServers": { "stackvo": { "command": "/path/to/stackvo-mcp" } } }
```

The 34 tools cover the questions an assistant is actually asked. The reads go
wider than the stack's own state: `system` and `container_stats` for a machine
that has run out of memory, `hosts` for a domain that does not resolve,
`log_read` for the application's own exception rather than the container's
stdout, `service_connection` for a connection string, `service_instances` for a
workspace running MySQL 8.0 and 8.4 side by side, `packages` for what could be
installed, `mail_message` for the body of a mail the application sent, and
`snapshots` for what could be restored, `ide_debug` for why a breakpoint is
not being hit — the port, the mapping, and whether anything is listening — and
`profiler` for why one page is slow, which is the sampling profiler rather than
the one that costs several times the request, and `hotspots` for the answer that
question actually wants — the functions one recorded run spent its time in,
read from the trace rather than from SPX's own web UI.

**Read-only by default.** 12 of the 34 tools change things and appear only with
`--allow-writes`: `xdebug_set`, `certificates_reissue`, `project_start`,
`project_stop`, `project_restart`, `service_start`, `service_stop`,
`service_restart`, `snapshot_take`, `stack_up`, `stack_down`, `generate`. Read
that list before passing the flag — it grants an assistant the ability to **stop
the whole stack** and to stop a shared service every project depends on, not
just to toggle Xdebug. Every tool is annotated `readOnlyHint` /
`destructiveHint`, so a client can require confirmation for a tool it has never
seen.

Restoring a snapshot is deliberately **not** a tool. Taking one is: it is the
call to make before asking for a migration, it adds a file and changes nothing.
Putting data back over live rows is a decision for the app's own confirmation,
not for a tool call.

No tool returns a password, and no tool takes an argument that asks for one —
`service_connection` is hard-coded to the unrevealed form, and a test asserts
that no schema on this surface has a `reveal`, `password`, `secret` or `token`
property. The app shows a credential on a click, to the person sitting there;
this surface has no equivalent and is not going to grow one.

The server speaks protocol revisions `2025-06-18`, `2025-03-26` and
`2024-11-05`, and answers `initialize` with the one the client asked for. A
server that answers with a revision the client did not request is entitled to be
hung up on, which reads to the user as "it does not work" with nothing in any
log.

The tool table names, for each tool, the `contracts/ipc.json` command it
implements, and three tests cross-check the two: a tool naming a command that
does not exist fails, a read-only tool backed by a declared `mutation` fails,
and a write-gated tool backed by a mere `query` fails — the gate would be
guarding nothing. Generating the list outright was the obvious move and is the
wrong one: dispatch cannot be generated, so a generated list advertises tools
that fail when called.

**Not exposed:** the rest of the mutating surface. 67 of the 304 commands take
an `AppHandle` because they report progress through Tauri's event system, and a
stdio subprocess has no app to emit into. Decoupling that is a refactor of its
own; pretending otherwise would mean advertising `project_build` and having it
fail. Service control is no longer in that set — `progress::Null` is what let
`instance_start` and its pair off the window, which is why starting Redis from
a chat window works here and did not a release ago.

### Telling the assistant when to use it

Registering the server makes the tools reachable. It does not make them used: an
assistant that has never seen this stack reads the source, guesses at nginx, and
suggests editing a generated file — because nothing told it that
`stackvo_doctor` answers that question in one call.

**Settings → AI rules** writes that into the instructions file the assistant
already reads. Six files, one per file rather than one per product, because
Codex and Zed both read `AGENTS.md`:

| File                                           | Read by                 |
| ---------------------------------------------- | ----------------------- |
| `CLAUDE.md`                                    | Claude Code             |
| `AGENTS.md`                                    | Codex, Zed              |
| `.cursor/rules/stackvo.mdc`                    | Cursor                  |
| `.github/instructions/stackvo.instructions.md` | VS Code, GitHub Copilot |
| `.windsurf/rules/stackvo.md`                   | Windsurf                |
| `GEMINI.md`                                    | Gemini CLI              |

In the project, or in the home directory for the three clients that read a
global file. `stackvo rules` prints the same table in a terminal,
`stackvo rules-install <id>` does the same write, and `--project <name>` aims it
at one project rather than the workspace root. A project's own detail page
carries the same three controls under **AI**, scoped to that project — the
rules are per project, so that is where somebody looks for them first. What it says is what a model gets wrong without being told, in the
order it gets it wrong: which tool answers which question; that everything under
the generated directory is overwritten and the input is what to change; that
`docker compose` by hand takes a name and a port the next generate expects to
own; and that a writing tool can stop the whole stack, so take a snapshot first
and ask before calling one.

Only the region between `<!-- stackvo:rules:begin -->` and
`<!-- stackvo:rules:end -->` is ever written. This is somebody's own `CLAUDE.md`
— a file with no markers is appended to, never replaced, everything else comes
back byte for byte, and a copy is kept beside it as `.stackvo-backup` first. The
front matter Cursor and VS Code need to apply the file at all is written when
the file is created and never again, because a user who narrowed `applyTo` meant
it. A test asserts that every tool the rules name is a tool that exists — rules
that send an assistant at a tool the server would refuse are worse than no rules.

### When something goes wrong

The app writes a rotating log — seven days, then it drops the oldest.
**Settings → Application log** shows where and opens the folder. Password and
token values are masked as the log is written, so it is safe to attach to an
issue, but read it first.

**Settings → Diagnostic bundle** packages the same log with `preflight`,
`doctor`, the engine state and the version into one zip, masked a second time on
the way in. What it holds and what it deliberately leaves out — no `.env`, no
project sources — is listed in [PRIVACY.md](PRIVACY.md), along with everything
this app stores and every host it can reach. There is no telemetry; that
sentence is held up by a test rather than by a promise
(`src-tauri/tests/privacy_claims.rs`).

### Where things are kept

Three directories, and which one you want depends on whose failure you are
looking at. `src-tauri/src/appdir.rs` owns the first two and the reasoning.

|             | macOS                                    | Windows                        | Linux                          |
| ----------- | ---------------------------------------- | ------------------------------ | ------------------------------ |
| App log     | `~/Library/Logs/StackVo/`                | `%LOCALAPPDATA%\StackVo\logs\` | `~/.local/state/stackvo/logs/` |
| Preferences | `~/Library/Application Support/StackVo/` | `%APPDATA%\StackVo\`           | `~/.config/stackvo/`           |
| Stack state | `~/.stackvo/`                            | `~/.stackvo/`                  | `~/.stackvo/`                  |

**The app log and preferences** follow each platform rather than one string
forced onto all three: Apple's log folder, `%LOCALAPPDATA%`, and
`XDG_STATE_HOME` — which is where the XDG specification puts logs, and which
this wrote outside of until it was noticed. Both are named `StackVo` and not
`com.stackvo.desktop`: the bundle identifier is what the OS calls this app —
the `Preferences` plist, the code signature, the privacy prompts — and these
two folders are ours to name and the ones a person is asked to open. Postman,
Termius and Redis Insight all split it the same way.

**Stack state** is `~/.stackvo`, or wherever `STACKVO_ROOT` points: the `.env`,
the templates, the generated compose files, the certificates, and
`logs/projects/<name>/` — which is where a _project's_ web server writes, and is
not the same thing as the app log above. It is the user's to move and safe to
delete; nothing the app needs in order to start is kept there.

Raise the log level with `STACKVO_LOG=stackvo_desktop=debug`.

### Keeping credentials out of `.env`

**Settings → Where credentials are kept** moves a database password, token or
server id into this machine's keystore — Keychain, Credential Manager or the
Secret Service — and leaves a reference in its place:

```sh
SERVICE_MYSQL_ROOT_PASSWORD=keychain:SERVICE_MYSQL_ROOT_PASSWORD@a1b2c3d4
```

That takes it out of the file that gets backed up, synced, and pasted into
support threads. **It does not take it off the disk**: the real value is still
rendered into `generated/docker-compose.dynamic.yml`, because that is where
Compose reads it from. Getting it out of there too changes the generated bytes
and is a v2 change — [decision 0010](docs/durum.md)
has the reasoning and the rest of the rules.

One key at a time and reversible, because `stackvo.sh` reads `.env` directly and
would use the reference string as the password. If you use both tools on one
workspace, leave the credentials where they are; **Settings → Doctor** says so
if you have not.

### Deploying it to more than one machine

There is a fourth location, and it is the only one the person at the keyboard
does not own: a policy file an administrator writes.

|         |                                                         |
| ------- | ------------------------------------------------------- |
| macOS   | `/Library/Managed Preferences/com.stackvo.desktop.json` |
| Windows | `%ProgramData%\StackVo\policy.json`                     |
| Linux   | `/etc/stackvo/policy.json`                              |

```json
{
  "schemaVersion": 1,
  "settings": { "DEFAULT_TLD_SUFFIX": "corp.test", "SERVER_TYPE": "nginx" },
  "locked": ["DEFAULT_TLD_SUFFIX"],
  "registryPrefix": "registry.corp.example/proxy"
}
```

`settings` overrides both the shipped default and the workspace's `.env`.
`locked` refuses a write to those keys from Settings, and only works for keys
`settings` also sets — "do not change this" without saying _to what_ leaves
every machine on whatever it happened to have. `registryPrefix` is prepended to
every image reference in the generated Dockerfiles and compose files, except
one that already names a registry, one already carrying the prefix, and the
`stackvo-*` images that are built here and exist in no registry at all.

A file that does not parse applies nothing and the app starts normally, but the
failure is shown in Settings rather than logged — a policy that quietly does
nothing is one the administrator believes is in force.

`STACKVO_POLICY_FILE` points at a different file, which is how you test one
without root. **This is not a security boundary.** The override exists, the
file is usually within the user's reach, and the layer tells a co-operating app
what your organisation intends — nothing more. See
[decision 0009](docs/durum.md).

`diagnose` is the headless equivalent of the dashboard — it exercises every
read-only command and prints what the UI would show, which makes it a genuine
troubleshooting tool as well as the port's end-to-end check. It deliberately
does not run the mutating commands: those would restart your stack.

The CLI is not being replaced. Both tools read the same `stackvo.json` and `.env`, so a project
created in either works in the other. That compatibility is enforced by a checked-in contract and a
validator, not by convention.

Phase 0 turned up four live bugs in shipped StackVo, found purely by writing the format down:

- Node projects created through the web UI generate as PHP and cannot build ([C-01](contracts/CONFLICTS.md))
- Four of the six advertised runtimes have no generator at all ([C-02](contracts/CONFLICTS.md))
- The default extension set can't build on the default PHP version ([C-06](contracts/CONFLICTS.md))
- `mongo-express` never starts in minimal mode — profile name mismatch ([C-09](contracts/CONFLICTS.md))

## Roadmap

| Phase | Scope                                                                              |
| ----- | ---------------------------------------------------------------------------------- |
| 0 ✅  | Freeze the config contract, extract the extension matrix, derive the IPC surface   |
| 1 ✅  | Tauri + Vue skeleton; port the existing dashboard; read-only views on real metrics |
| 2 ✅  | Container control via `bollard`; streamed build/log progress                       |
| 3 ✅  | Tray, notifications, fs-watcher, hosts helper, PTY, autostart, single-instance     |
| 4 🚧  | Generator port to Rust, native Windows, signed auto-updates                        |

**What is still open is written down, not remembered.**
[`docs/durum.md`](docs/durum.md) is the single work queue: every product gap and
every engineering item this project has diagnosed and not yet closed, with what
was checked to confirm it is still open. The ones no commit can close — the
update endpoint, the signing secrets, the Apple/Windows certificates and the
decisions only the owner can take — are in §5.

## How it is built

- **[ARCHITECTURE.md](ARCHITECTURE.md)** — the map: the four bands of the Rust
  side, the one request flow worth knowing, and what the front end's panes and
  composables are for.
- **[docs/durum.md](docs/durum.md)** — what is left, what is waiting on a
  decision, and the numbered decisions with their consequences. Comments in the
  source say "ADR 0005"; that is §6 of this file. Finished work leaves it — the
  record is `CHANGELOG.md` and the git history.
- **[docs/accessibility.md](docs/accessibility.md)** — the conformance
  statement, in the shape EN 301 549 asks for. Every number in it is reproduced
  by `npm run test:e2e`, and a test fails if the statement and the routes it
  claims come apart.
- **E — IPC surface.** Every command in `contracts/ipc.json` must be registered
  in `src-tauri/src/lib.rs` and wrapped in `src/lib/ipc.js`.
- **F — reachability.** Every wrapper must be called by some view or store, and
  every declared event must actually be emitted.

Without them the contract quietly drifts ahead of the code. It had: by the end
of Phase 3, **22 declared commands had no implementation** — including
`project_create`, so the app could not create a project at all. Suite F then
found **21 wrappers no view called** and **4 events nothing emitted**.

Current state: **no errors**, six warnings — five wrappers no view calls yet,
and one for a checkout with no projects in it.

That number is deliberately not a promise. It was written here as "4 errors, all
of them pre-existing StackVo bugs" and stayed after the four were fixed, because
nothing checks a number in prose. `src-tauri/tests/contract_agreement.rs` is the
part that _is_ checked, and it covers the two edges this suite does not — see
[decision 0006](docs/durum.md).

`tools/measure-env-usage.mjs` is a separate check with the same intent: it
measures which `.env` keys the checkout actually reads and reconciles that
against the `status` labels in the contract. It exists because the first,
hand-run version of that measurement was executed from the wrong directory and
mislabelled twelve keys — a number that looked like evidence and was not
checkable later.

## License

MIT
