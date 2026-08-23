# Tooling

Puts `stackvo` where your shell can find it, and reports the tools this app runs on the host.

## The commands

`stackvo` runs the stack from a terminal — `stackvo up`, `stackvo artisan migrate`, `stackvo logs`. `stackvo-mcp` is the server the **AI assistants** page registers. An installed StackVo **carries both**, beside its own binary, so the page finds them without anything being built. From a checkout they are built with:

```
npm run sidecars
```

They are linked into one directory the app owns — `~/Library/Application Support/StackVo/bin` on macOS, `%APPDATA%\StackVo\bin` on Windows, `~/.local/share/stackvo/bin` on Linux. Not `~/.stackvo`: that one is the *stack's* state, you can point it somewhere else, and deleting it is a supported way to start over. A `PATH` entry that disappears when you reset your stack points at nothing.

On macOS and Linux the entries are symlinks, so a rebuild is picked up without pressing anything. On Windows they are copies, because a symlink there needs a privilege this app does not ask for — so after an update, press it again.

## Your PATH

| Control | What it does |
| --- | --- |
| Add | Links both commands and writes one line into that shell's startup file. |
| Update | Replaces a line pointing at an older directory. |
| Remove | Takes the line back out. The links stay. |
| Copy line | The line itself, to paste somewhere this app should not edit. |

One file per shell: `.zshrc` for zsh, `.bash_profile` on macOS and `.bashrc` elsewhere for bash, `config.fish` for fish, and the PowerShell profile on Windows. macOS Terminal opens login shells and Linux terminals do not, which is why bash differs by platform.

### What it is safe to press

Only the region between `# stackvo:path:begin` and `# stackvo:path:end` is ever written. Everything else in that file comes back byte for byte, a file with no markers is appended to rather than rewritten, and a `.stackvo-backup` copy is left beside it first. Removing takes out that region and nothing else.

The directory goes **first** on `PATH`. The only names in it are `stackvo`, `stackvo-mcp` and tools you asked this app to manage, so putting it last would mean a managed `mkcert` losing to a half-removed one — which is the state you press this to get out of.

### It applies to the next shell

A startup file is read when a shell starts. The terminal you already have open will not see the change; open a new one, or `source` the file. The page says so while that is true.

## Host tools

Four programs, and they are the four this app itself runs **outside** every container:

| Tool | Without it |
| --- | --- |
| Docker | Nothing works. Every project is a container. |
| Docker Compose | The generated stack is compose files; this runs them. |
| Git | No worktrees, no branch names on the project pages, no cloning into a new project. |
| mkcert | The stack still runs and every browser warns on `.loc`. |

A **yours** badge means the copy found is your own — Homebrew's, the distribution's, Docker Desktop's — and this app will not touch it. **managed** means this app installed it.

Only mkcert has an Install button, and that is not an omission. Docker is an application with an installer and a virtual machine behind it; a bare client dropped on `PATH` would be worse than its absence, because `docker ps` would then fail instead of `docker` being missing. Git ships with every platform, and asking for it on macOS opens a system installer this app should not be racing.

### What the download is checked against

The SHA-256 is compiled into this build of StackVo, one per platform. It is **not** fetched alongside the file — a checksum served beside the thing it describes is not a check, because whoever can replace one can replace the other. Nothing is written until the bytes match, and a mismatch is refused rather than retried.

There is deliberately no Update button. An install that quietly follows upstream is how a pinned checksum stops being a pin; a newer mkcert arrives in a release of this app, after somebody has looked at it.

## What is not here

`composer`, `node`, `npm`, `bun`, `wp`. Other tools of this kind download those onto your host and shim them onto `PATH`. Here they run in the project's container, at the version that project declared — `stackvo composer install`, or the buttons on a project's own page. A copy on the host would be a second answer to "which composer runs", and it would be the wrong one: it knows nothing about the project's PHP.
