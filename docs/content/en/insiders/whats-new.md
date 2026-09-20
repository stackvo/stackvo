# What's new

Each release, as the person using the app sees it. The
[changelog](changelog.md) has every change with the reasoning; this page has
the ones that change what you can do.

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
