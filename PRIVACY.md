# Privacy

## The short version

**There is no telemetry, and there is no plan to add any.** Nothing about how
you use this app is counted, sampled or sent anywhere. There is no account, no
sign-in, no crash reporting service and no server behind the app: it talks to
your own Docker daemon and your own filesystem, and the only thing it contacts
on its own initiative is the update endpoint — described below, and only from
the Settings screen.

One thing is worth naming here rather than only in the table: opening a **help
panel** fetches that card's document from the repository, so the text is current
rather than as old as your build. The request names the topic you opened, which
is a fact about you, and it is listed below with everything else. It is cached
after the first fetch, so the panel works offline and a machine that never
reaches the network still has the copy the app shipped with.

This document exists because "we don't collect anything" is not a fact until
somebody writes down what "anything" was measured against. The readiness review
made the point in the other direction: a tool that quietly collects nothing and
a tool that quietly collects something look identical from the outside, so
silence is not a privacy property.

### If that ever changes

Any future telemetry would be **opt-in, off by default, and described here
before it ships**, with the payload listed field by field on the screen that
offers it. An update that starts sending something without that is a defect, not
a decision — report it as one (see [SECURITY.md](SECURITY.md)).

---

## What is stored, and where

Everything here is a plain file on your own machine. Nothing is encrypted at
rest and nothing needs a password to read, because none of it leaves the
account it belongs to.

| What                                                                                                                                                                                                                                                                                                                                                                                                               | Where                                                                                                                                    | How long                                                                                                                                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `preferences.json` — app settings                                                                                                                                                                                                                                                                                                                                                                                  | macOS `~/Library/Application Support/StackVo`, Windows `%APPDATA%\StackVo`, Linux `~/.config/stackvo`                                    | Until you delete it                                                                                                                                                                                                                                                                                                                                                            |
| `preferences.corrupt-<UTC>.json`                                                                                                                                                                                                                                                                                                                                                                                   | Beside it                                                                                                                                | Until you delete it — a file that failed to parse, kept rather than overwritten so the settings it held can be recovered                                                                                                                                                                                                                                                       |
| `stats-history.json` — CPU and memory readings per container, so a chart is not empty after a restart                                                                                                                                                                                                                                                                                                              | Beside `preferences.json`                                                                                                                | **2 hours** of samples. Older readings are discarded when the file is read, not merely hidden — a longer gap would draw as a flat line rather than as absence                                                                                                                                                                                                                  |
| Application log, rotated daily                                                                                                                                                                                                                                                                                                                                                                                     | macOS `~/Library/Logs/StackVo`, Windows `%LOCALAPPDATA%\StackVo\logs`, Linux `~/.local/state/stackvo/logs`                               | **7 files**, oldest deleted automatically                                                                                                                                                                                                                                                                                                                                      |
| `crash-<UTC>-<pid>.txt`                                                                                                                                                                                                                                                                                                                                                                                            | With the logs                                                                                                                            | **10 reports**, oldest deleted automatically. The app tells you one was written — a line on the next launch, once — because a crash you never hear about is one you cannot report. Nothing is sent: the line offers the diagnostic bundle, which is yours to save and yours to send                                                                                            |
| `.crashes-seen` — the name of the newest crash report you have been shown                                                                                                                                                                                                                                                                                                                                          | With the logs                                                                                                                            | Until you delete it. One line, so the notice above appears once rather than every launch                                                                                                                                                                                                                                                                                       |
| `audit.jsonl` — one line per privileged or irreversible act: host-file writes, certificate trust, project deletion, `.env` keys changed, database restores, image bundles loaded, an assistant registered, AI rules written, an IDE's debug configuration written                                                                                                                                                  | With the logs                                                                                                                            | **Never deleted by the app.** That is the point of it — a record rotated away after a week cannot answer a question asked after two. Keys and names only, never values                                                                                                                                                                                                         |
| Diagnostic bundle (`.zip`)                                                                                                                                                                                                                                                                                                                                                                                         | Only where you choose to save it, from the system save dialog                                                                            | Yours — the app never keeps a copy                                                                                                                                                                                                                                                                                                                                             |
| Stack configuration and projects                                                                                                                                                                                                                                                                                                                                                                                   | Your workspace: `.env`, `stackvo.json`, generated compose and server files, project sources, container logs under `logs/projects/<name>` | Until you delete them                                                                                                                                                                                                                                                                                                                                                          |
| Files this app writes **into your project**, each only when you ask for it on screen: `.stackvo/context.json` (what an assistant working in the container should know — names and addresses, never credentials), a marked block in `CLAUDE.md` / `AGENTS.md` / `.cursor/rules/` / `.github/instructions/` / `.windsurf/rules/` / `GEMINI.md`, and a `Listen for StackVo: <project>` entry in `.vscode/launch.json` | Your project directory — so they are committed if you commit them                                                                        | Until you remove them, which the same screens do                                                                                                                                                                                                                                                                                                                               |
| Profiler recordings — Xdebug's cachegrind files under `logs/projects/<name>/xdebug`, php-spx's report pairs under `logs/projects/<name>/spx`                                                                                                                                                                                                                                                                       | Your workspace                                                                                                                           | Until you delete them, which both panes do. **A profile is a picture of your code**: it holds function names, file paths and, for a recorded request, the URI. It holds no arguments and no values. The pane and the `hotspots` tool read the function names back out of that file to say where the time went, which is the same picture presented rather than a new one taken |
| `generated/spx/key` — the key php-spx's control panel asks for                                                                                                                                                                                                                                                                                                                                                     | Your workspace, outside any directory a clone carries                                                                                    | Until you delete it, which rotates it. Local to this machine and this workspace; it authenticates a panel served on your own site's address                                                                                                                                                                                                                                    |

Deleting all four of the app's own locations is supported and loses nothing but
settings and history. The workspace is separate on purpose: it is the stack, not
the app.

The row above it is the one worth reading twice, because those files are **not
in any of the app's own locations**: they are in your repository, they are the
kind of file that gets committed by accident, and they are written by a button
rather than by a background task. None of them carries a credential — the agent
context file has no field one could go in, and the debug configuration is a port
and a path mapping — and every screen that writes one also removes it. What is
written stays between markers this app owns, so the rest of a file you have been
keeping for a year comes back byte for byte.

**One file is read and never written**, and it is worth naming here because it
is the only one somebody other than you may have put there: a policy file at
`/Library/Managed Preferences/com.stackvo.desktop.json` (macOS),
`%ProgramData%\StackVo\policy.json` (Windows) or `/etc/stackvo/policy.json`
(Linux). If your machine is managed, that file can set app settings, prevent
you from changing them, and **redirect every image pull to a registry your
organisation runs** rather than Docker Hub. **Settings** shows you when one is
in force and which file it is. Nothing is sent to it or about it; it is an
input, not a destination.

### What the log contains

Command lines the app runs, the output of `docker compose` and of the StackVo
CLI, the paths involved, and errors. That is enough to reconstruct what you were
doing with the app on a given day, which is exactly why the files stay on your
machine and rotate away after a week.

Values of keys the app knows to be secrets — `*_PASSWORD`, `*_SECRET`,
`*_TOKEN`, `*_KEY` and their family, as `config::Env::is_secret` defines it —
are **masked before the line is written**. The mask is applied a second time
when a diagnostic bundle is built, because a bundle made today can carry lines
written by an older build whose masking rule was narrower.

### What a diagnostic bundle contains

`about.txt` (version, platform), `preflight.json`, `doctor.json`,
`engine.json`, the rotating log files and any crash reports — each log capped at
1 MiB, with `truncated` naming the ones that were cut. **It does not include
`.env`, and it does not include project sources.** It is plain text and JSON so
that you can read it before you send it; the file listing is shown in the app
rather than a bare "saved" confirmation, for the same reason.

Where a bundle goes afterwards is entirely your decision — the app writes the
file and nothing else.

---

## What leaves this machine

### On the app's own initiative

| Host                                                                   | When                                                                                                                                                               | What is sent                                                                                                                                                                                                   |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `raw.githubusercontent.com` — the update endpoint in `tauri.conf.json` | When the **Settings** screen is opened, and when you press _Check for updates_. Not at launch, and not at all when the build has no update public key compiled in. | An ordinary HTTPS GET for a static `latest.json`. The app adds no identifier, no version parameter and no cookie. As with any HTTP request, whoever serves the file sees your IP address and the request time. |
| `127.0.0.1` — the mail catcher                                         | While the mail screen is open                                                                                                                                      | Loopback only. This traffic never reaches a network interface, and the HTTP client for it is built with `no_proxy()` precisely so a corporate proxy setting cannot pull it off the machine.                    |

That is the complete list. Everything else below happens because you asked for
it.

### Because you asked for it

| What you do                     | Where it goes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Create a project from a Git URL | The remote **you** typed, over the credentials your own `git` is configured with                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| Start the stack                 | Whatever registry the images name — Docker Hub by default, or the one a policy file on a managed machine names instead. Your Docker daemon does this, with its own configuration                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| Open a share tunnel             | The provider **you** pick in the Share pane, and only that one: `cloudflared` to `trycloudflare.com`, an SSH client to `localhost.run` (whose banner also links `admin.localhost.run`) or to `a.pinggy.io`, `localtunnel` to `loca.lt`, or — with a token you stored — `ngrok`, Tailscale Funnel, `zrok` or LocalXpose, whose own clients also reach their dashboards (`dashboard.pinggy.io` is advertised in Pinggy's banner). The sidecar dials out; no port on your machine is opened. **The URL is public**: anyone who has it reaches the site on your machine, without a password, until you stop the tunnel                                                                                                                                                                                                                    |
| Follow a link in the app        | Your browser opens it. The app's own links point at `stackvo.github.io`, `github.com`, `docs.docker.com`, and — from the support and share menus — `bsky.app`, `buymeacoffee.com`, `discord.gg`, `fosstodon.org`, `reddit.com`, `twitter.com`, `www.linkedin.com`, `www.youtube.com`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Fetch the service catalogue     | The address **you** chose, or the mirror `policy.market.registryUrl` names on a managed machine. The app suggests the `github.com/stackvo/stackvo-service-packages` repository, which it translates to `raw.githubusercontent.com` before fetching, and nothing fetches anything until you press the button — a first run with no network shows the catalogue gate and stays there. Plain `http://` is refused. Two files are asked for from that one address — `registry.json` and `registry.json.minisig`, the signature over it — and a source that has no signature is asked once and taken as it is. The requests carry an `If-None-Match` for the copy already here and no identifier of any kind; the system proxy is used, unlike the mail client above, because on a managed network the mirror is only reachable through it |
| Install a service package       | The same address. Nothing else is contacted — the image itself is pulled later, by your Docker daemon, from whatever registry the package names                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Open a help panel               | `raw.githubusercontent.com`, for the help document of the card you opened it on. The request carries the topic name (`project-tunnel`, `settings-dns`) and your interface language, and no identifier of any kind. **It says which help page you opened**, which is a fact about what you were stuck on — so it is here rather than buried. One request per topic per run: the answer is cached under the app's own directory, the panel works offline from that cache, and a fresh install with no network reads the copy the app shipped with. A failed request is never reported; it silently reads what is already on disk                                                                                                                                                                                                        |
| Build the sampling profiler     | The container's own package mirror and `github.com`. Building php-spx runs inside a throwaway container of your project's image and does three things with a network: `apt-get` fetches a compiler and `zlib` from whatever mirror that image is configured with, `git` clones `github.com/NoiseByNorthwest/php-spx`, and nothing is uploaded. It happens once per PHP version, only when you press the button, and the container is removed afterwards                                                                                                                                                                                                                                                                                                                                                                               |
| Record a profile of one page    | **Your own project, on your own machine.** The app makes one HTTPS request to the address in that project's manifest — the same address your browser uses — with php-spx's trigger in a cookie. The host is never taken from what you type: a path naming another host is refused, redirects are not followed, and the system proxy is deliberately not used, so the request cannot leave the machine. Nothing is uploaded                                                                                                                                                                                                                                                                                                                                                                                                            |
| Install a host tool             | `github.com`, and only when you press Install on the Tooling page. One HTTPS GET for one release asset of one program — today `mkcert`, from `github.com/FiloSottile/mkcert` — carrying no identifier of any kind. The address is fixed in the build rather than typed, its SHA-256 is compiled in beside it, and bytes that do not match are discarded rather than installed. Nothing is uploaded, and nothing is fetched until the button is pressed                                                                                                                                                                                                                                                                                                                                                                                |
| Capture sessions for replay | **Nowhere — it does not leave this machine.** It is listed here because of what it writes rather than where it sends it: while a capture window is armed, this project's request **cookies and form bodies** are written to `generated/debug/<project>/sessions.jsonl` so that a POST can be replayed. A session cookie is the credential itself, so there is no redacted version of it — instead the permission is off until you press it, lasts minutes and never indefinitely, ends by itself even if the app was closed the whole time, and **deletes what it captured when you stop**. No screen and no report ever shows a captured value: they are reported as a count of cookie names and a size of body. They are not in the audit trail (which records the window being opened, and for how long) and they are **not in a diagnostics bundle**. Nothing is captured until the window is armed |
| Check dependencies for advisories | `api.osv.dev`, and only when you press _Check for advisories_ on a project's Dependencies card. **What is sent is the names and versions of that project's dependencies** — read out of its `composer.lock` and `package-lock.json` — as one POST per thousand packages. That is a real disclosure and it is written here in those words rather than described as "checking for updates": the list says which libraries you use and at which versions. Nothing else goes with it — no identifier, no project name, no path, no file contents. The rest of the card works with no network at all and always has: what the lock file already says about plain-`http://` sources, missing integrity hashes and which index each package came from is read on this machine and never leaves it |
| Send a diagnostic bundle        | Wherever you send it                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |

### While Docker builds an image

The generated Dockerfiles fetch packages during the build, from
`deb.nodesource.com` (Node.js), `dl.cloudsmith.io` (Caddy) and `rubygems.org`
(Ruby), plus whatever the base images pull from their own package mirrors. This
is Docker's network activity, on the machine, driven by files the app writes
into your workspace — you can read every one of them before building.

---

## What this app is _not_ protecting you from

Naming these is more useful than a reassurance:

- **Database passwords are on the disk in plain text, in two files.** `.env`
  holds them, and so does `generated/docker-compose.dynamic.yml`, which is what
  Compose actually reads and which is rendered from `.env` on every generate.
  File permissions are the only boundary. On a managed machine `.env` is also
  whatever your backup and sync tools do with it.

  **Settings → Where credentials are kept** moves a credential into this
  machine's keystore — Keychain, Credential Manager, or the Secret Service —
  and leaves a `keychain:` reference in `.env` in its place. That takes it out
  of the file that gets backed up, synced and pasted into support threads. **It
  does not take it off the disk**: the real value is still rendered into
  `generated/docker-compose.dynamic.yml`, because that is where Compose reads it
  from. Getting it out of there too changes the generated bytes and is tracked
  as a v2 change — see `src-tauri/src/secrets.rs`, which carries the reasoning.

  The `stackvo.sh` command-line tool cannot read a moved credential and would
  use the reference string as the password. If you use both tools on one
  workspace, leave the credentials in `.env`.

- **Docker access is total.** The app manages containers, so it can read and
  write anything those containers mount — including your project sources.
- **A tunnel is a publication.** See above; it is the one action in the app that
  makes something on your machine reachable from the internet.
- **The app runs as you.** It is not a boundary against someone who already has
  your user account.

---

## How this document is kept true

`src-tauri/tests/privacy_claims.rs` scans the shipped code — the Rust
production regions, the front end, and the updater endpoint in
`tauri.conf.json` — for every host it can reach, and **fails the build if one of
them is not named in this file**. A dependency or a feature that starts talking
to somewhere new cannot land quietly; the build stops until this page says so.

What the test cannot settle is a claim about intent — "no telemetry" is not a
string a parser can find. What it can settle is the surface that claim is made
about, and that is what it holds.
