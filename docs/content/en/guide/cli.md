# Command line

Everything the window does has a `stackvo` command. It is not needed to use
the app — the CLI is for scripts, CI steps, and the one command you want to
run in the project you have `cd`-ed into.

<figure markdown>
![The Terminal tab](../screenshots/web/project-detail-terminal.webp){ loading=lazy }
<figcaption>The Terminal tab: a shell inside the project's container.</figcaption>
</figure>

## Putting it on your PATH

`stackvo` and `stackvo-mcp` ship inside the app. **Settings → Tooling → Add**
links them into a directory the app owns and writes one line into your
shell's startup file. **Remove** takes the line back out.

## Everyday commands

```bash
stackvo status                        # projects and services
stackvo up shop / down shop           # start / stop
stackvo restart shop
stackvo logs shop --follow            # live logs
stackvo open shop                     # open in a browser
stackvo doctor                        # what is wrong, and the fix
stackvo tui                           # a full-screen terminal UI
```

## In the project's container

`cd` into a project and type:

```bash
stackvo php -v            # the project's PHP, on a host that has none
stackvo artisan migrate --force
stackvo composer install
stackvo npm run build
stackvo wp plugin list    # also console, rails, bundle, yarn, pnpm
stackvo python -V         # and ruby, go, cargo, bun, deno
stackvo shell             # an interactive shell in the container
stackvo exec <program>    # anything else
```

Everything after the command name is passed on untouched, and the exit code
is passed through — which is what makes `stackvo artisan test` meaningful in
a CI script.

## What scripts can rely on

`--json` on every command, the answer on stdout and the narration on stderr,
exit codes that mean one thing each. The
[reference](../reference/cli.md#the-contract-scripts-can-rely-on) has the
table; the guarantee is that the table you see in a terminal is rendered from
the same value, so the two cannot drift.

## The TUI

`stackvo tui` is the window in a terminal: projects, services, logs,
start and stop — for a machine you reach over SSH, or a preference.
