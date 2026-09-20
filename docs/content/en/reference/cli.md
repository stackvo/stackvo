# Command line

`stackvo <command> [arguments] [flags]`. Every command here is one the
window also has; the CLI is the same core from a terminal. Commands are
grouped the way `stackvo --help` groups them.

## Reads

Nothing changes. Safe to run at any time.

| Command | What it answers |
| --- | --- |
| `status` | Whether anything will work: the workspace, the engine, every startup requirement, how many projects are up. |
| `doctor` | The full diagnosis: requirements, port conflicts by holder, missing hosts entries, stale generated config, disk. |
| `verify <project>` | Whether this machine matches what the repository declares the project needs — and which line does not. |
| `projects` | Every managed project, with its domain and whether it is up. |
| `project <project>` | One project in full: manifest, container, Xdebug, PHP. |
| `services` | The shared services and their health. |
| `logs <id>` | A container's output, for a project or a service. |
| `certs` | The certificate: what it covers, what it misses, when it expires. |
| `db` | The database services, their databases, whether they are running. |
| `mail` | The mail catcher's inbox. |
| `mcp` | Which assistants have `stackvo-mcp` registered. |
| `rules` | Which AI rules files carry StackVo's block. |
| `tools` | Where `stackvo` is installed from and which shell files carry the PATH line. |
| `ide <project>` | The values an IDE needs to step-debug the project, and whether anything is listening. |
| `spx <project>` | The sampling profiler: built, mounted, on. |
| `spx-top <project>` | Where one recording spent its time. |

## Changes the stack

| Command | What it does |
| --- | --- |
| `up` | Bring the stack up. Builds missing images, so a first run takes minutes. |
| `down` | Bring the whole stack down: every profile, projects included. |
| `start <project>` | Start one project's container, then run its post-start hooks. |
| `stop <project>` | Run the pre-stop hooks, then stop the container. |
| `restart <project>` | Stop and start, with the hooks on both ends. |
| `generate` | Re-render the compose files, Dockerfiles and configs from the manifests. |
| `lock <project>` | Write `stackvo.lock`: the service versions and package digests this machine runs. |
| `xdebug <project> on\|off` | Turn step debugging on or off. The first `on` needs a rebuild. |
| `certs-renew` | Reissue the certificate for the domains the projects have. |
| `mcp-install <assistant>` / `mcp-remove` | Register `stackvo-mcp` with one assistant, or take the entry out. `stackvo mcp` lists the ids. |
| `rules-install <file>` / `rules-remove` | Write the AI rules block into one file, or take it out. |
| `path-install` / `path-remove` | Link the commands onto PATH, or take the line back out. |
| `tool-install <tool>` / `tool-remove` | Fetch one host tool — `mkcert` today — checked against a digest compiled into the app. |
| `ide-install <project> <ide>` | Write the debug configuration into one IDE's file in that project. |
| `spx-record <project> <path>` | Profile one request without a browser. |
| `spx-build <project>` | Compile php-spx for the project's PHP version, in a throwaway container. |
| `market-bundle <dir>` | Write the catalogue and every package into one directory, for a machine with no network. |

## Screens

| Command | What it does |
| --- | --- |
| `tui` | The window, in a terminal. |

## In the project's container

Run from inside a project folder. Everything after the command name is
passed on untouched, and the exit code comes back through.

| Command | Runs |
| --- | --- |
| `php`, `composer`, `artisan`, `console` | PHP and its tools |
| `npm`, `yarn`, `pnpm`, `node`, `bun`, `deno` | JavaScript runtimes and package managers |
| `python`, `ruby`, `bundle`, `rails`, `go`, `cargo` | The other runtimes |
| `wp` | WP-CLI |
| `shell` | An interactive shell |
| `exec <program> …` | Anything else. Recorded in the audit trail. |

`stackvo artisan --help` goes to artisan; put `--help` first to see the
app's own.

## Shell completion

`stackvo completions <shell>` prints the stub for bash, zsh, fish or
PowerShell. `path-install` sets it up.

## The contract scripts can rely on

| Rule | Detail |
| --- | --- |
| `--json` | On every command. The table you see is rendered from that value, so the two cannot drift. |
| stdout / stderr | The answer on stdout, the narration on stderr. A failure leaves stdout empty. |
| Unknown flags | An error, never ignored. |

| Exit code | Meaning |
| --- | --- |
| `0` | OK |
| `1` | Failed |
| `2` | Bad command line |
| `3` | No workspace set up on this machine |
| `4` | Docker unreachable |
| `127` | A runtime this project does not have |
