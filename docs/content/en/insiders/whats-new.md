# What's new

Each release, as the person using the app sees it. The
[changelog](changelog.md) has every change with the reasoning; this page has
the ones that change what you can do.

## 0.3.3 — 24 October 2026

A maintenance release: the fixes below, and the TLS, archive and async
libraries moved past ten published advisories.

**macOS**

- **The Dock icon fills its slot.** macOS 26 drew it small on a grey tile;
  it is now in Apple's icon shape, like its neighbours.
- **Docker Compose is found when the app is opened from Finder, the Dock or
  Spotlight.** Those launches do not see `/usr/local/bin` or
  `/opt/homebrew/bin`, so every `.dmg` install stopped at the requirements
  gate saying "update Docker Desktop" with nothing to update. The app now
  adds those directories itself, and the same fix reaches `git` and
  `mkcert`.

**Services**

- **Start finishes an enable that was cut off.** An enable interrupted
  mid-pull left a service "on" with no container, and Start could only
  answer "container not found". Start now creates what is missing, and an
  enable that fails undoes itself and offers Enable again.

**Updates**

- **A newer release is announced once, at launch**, in a line at the bottom
  of the window that leads to the Updates page. Until now only opening
  Settings asked, so an install left on the dashboard never heard of one.

**Accessibility**

- **The command palette button says its keys.** Its accessible name now
  includes the `⌘K` or `Ctrl+K` printed on it, so voice control understands
  the words you can read.

## 0.3.2 — 12 October 2026

**Projects**

- **Goravel, Masonite, AdonisJS and Loco** in the new-project panel, one
  application skeleton per runtime, installed the way each one's
  documentation says.
- **Every template installs the framework's newest release.** Gin and Echo
  no longer pin a version; the newest lands in `go.mod` and `go.sum` the
  way `composer.lock` does. A PHP version chosen below what the installed
  `composer.json` requires is raised to that floor rather than refused.
- **Go 1.24 to 1.27**, defaulting to 1.27; **Node 24 and 25**, defaulting
  to 24.

**Fixed**

- **Start all, stop all and restart all did nothing.** They run now, and
  containers that fail during a bulk action are reported rather than
  skipped in silence.
- **A transient PECL or NodeSource outage no longer fails the whole
  build.** Each install is retried, and the error names the failing step
  instead of "docker exited with code 1".
- **The Mail page can enable the catcher again**, Mailpit or MailHog,
  whichever is installed.

## 0.3.1 — 6 October 2026

**Services**

- **28 services in the catalogue**, published from the packages repository
  rather than shipped with the app: Qdrant, Neo4j, Memgraph, Nessie,
  OpenSearch and OpenSearch Dashboards, the OpenTelemetry Collector, Loki,
  Tempo, Karapace, Apicurio Registry, Redpanda Connect, Flink, Spark,
  Temporal and its UI, Zitadel, Unleash, Vault, OpenBao, Ollama, MLflow,
  Label Studio, Argilla, GreenMail, Icecast, Postal and Backstage.

**Fixed**

- **"Pull and use" kept refusing a correctly signed catalogue** with "signed,
  but by none of the keys this machine trusts". A refused index is no
  longer remembered, and a machine already stuck recovers on its next
  refresh.

## 0.3.0 — 22 September 2026

**Projects**

- **A project declares the processes its supervisor runs.** A queue worker
  added inside the container used to vanish at the next rebuild. A
  `processes` block in `stackvo.json` — an id, an argv, optionally
  `replicas` — is rendered into the container's supervisor config, so a
  rebuild restores the worker and a clone gets it. The Supervisor pane can
  push the same config into the running container, so the new process
  starts now without a rebuild.
- **A supervised process, in full.** The info button on a Supervisor row
  opens it: who put it there, its config block, resident memory, threads and
  CPU, a log that follows, and stop, start, restart and a signal. The pane
  header carries the daemon's own controls, with a confirmation before the
  two that take the web server down.
- **A folder named like a site adopts as that site.** `api.showtv` becomes
  `api.showtv.loc`, not `api.showtv.stackvo.loc` — a fourth label a wildcard
  certificate could never reach.

**Updates**

- **A beta channel.** A pre-release never reaches a stable install; turn
  betas on under **Settings → About → Also receive beta releases**, and a
  beta install keeps receiving stable releases too.
- **One `SHA256SUMS.txt` on every release**, beside the installers, as the
  release notes had promised since 0.2.0.

**Help**

- **Help documents are fetched, never bundled**, so a corrected sentence
  reaches you without a new installer. Offline, the panel says you are
  offline rather than "not written yet".

**Documentation**

- **A documentation site**, [stackvo.github.io/stackvo](https://stackvo.github.io/stackvo/),
  in English and Turkish: getting started, a guide per task, the reference
  in plain words, and this page.

**Security**

- **rustls 0.23.45**, for CVE-2025-61730, the day the advisory was
  published.
- **The catalogue signing key is rotated**; the published index is re-signed
  with the new one.

## 0.2.0 — 2 September 2026

The first published release: ten installers, unsigned, verified by
`SHA256SUMS` and a minisign signature the updater checks.

**Projects**

- **A monorepo is one project.** A repository with `api/` in Go, `web/` in
  Next.js and `worker/` in Python is one entry, one start, one certificate
  and one set of hostnames.
- **`stackvo.lock`** records the service versions and package digests a
  project was actually built against, so a clone gets the same stack.
- **`git bisect` with the right environment.** Each step runs the commit
  against the PHP and service versions the project declared at the time,
  not the ones on the machine today.
- **A file at the root of the workspace** adds commands to every project in
  it, and a package can bring its own commands — installing Redis gives you
  `redis-cli`.
- **RoadRunner** joins Swoole as a Laravel Octane driver.

**Debugging**

- **Why was this request slow** puts the profile, the query log and the
  timeline around one request on one screen.
- **Replaying the request that failed**, body, headers and session
  included, and a replay can be bound to a database snapshot so a POST is
  safe to press twice.
- **The editor inside the container:** VS Code running in the image, with
  the language server, `composer` and `artisan` in there and no PHP on the
  host.

**Sharing**

- **A shared tunnel can ask for a password** and can keep its address
  between starts, where the provider allows it.

**The machine**

- **Podman is recognised**, rootless first.
- **What can leave this machine** lists which containers can reach the
  internet at all, and which pulls bypassed the organisation's mirror.
- **Docker's cost is measured** rather than argued about, and *It works on
  my machine* is answered by comparing what two machines actually run.
- **All four stack actions ask first**, because start-all and stop-all act
  on every container on the machine.
- **The app tells you it crashed**, once, on the next launch, with a button
  that opens the report.
- **An introduction, once,** after the setup screens, instead of four gates
  and no welcome.

**Fixed**

- Forty-seven English sentences that were printed into a Turkish window.
- A DDEV project configured for Apache was imported as nginx.
- The manifest editor saved the wrong field names and could switch a
  project's document root.

**Windows**

- The test suite runs on Windows, and the two product bugs it found are
  fixed.
