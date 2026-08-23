# Security

## Reporting a vulnerability

Please report privately, not as a public issue: open a
[security advisory](https://github.com/stackvo/stackvo/security/advisories/new),
or email **backend@cyh.com.tr**.

Include what you did, what happened, and which version and platform. A proof of
concept helps but is not required to file.

You will get an acknowledgement within 72 hours and an assessment within a week.
If a fix is needed, we will agree a disclosure date with you before publishing.

## What this app can reach

StackVo Desktop is a local development tool, so its capabilities are broader
than a typical GUI. Understanding them is the fastest route to a good report:

| Capability                 | Why                                                                         | Boundary                                                                                                                             |
| -------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Docker socket / named pipe | Manages the containers that are the product                                 | Read from `DOCKER_HOST`, the docker context, then well-known sockets. No TLS stack is compiled in — the daemon is local by design    |
| `/etc/hosts`               | Project domains must resolve                                                | Writes only inside a `# >>> stackvo >>>` marker block, after showing the user a diff. Elevation via osascript / pkexec / UAC         |
| Project directories        | Reads and writes `stackvo.json` and generated build inputs                  | Confined to `<workspace>/projects/<name>`; names are validated and the resolved path is checked for containment                      |
| Subprocesses               | Runs the StackVo CLI, `docker compose`, and a PTY                           | Spawned without a shell — arguments are passed as a vector, never interpolated into a command line                                   |
| `.env`                     | Enables services and stores stack configuration                             | Patched line-in-place; keys and values are validated so the format cannot be broken                                                  |
| Service packages           | The catalogue is not compiled in; a service is data fetched from a registry | Verified before it is unpacked, and its rendered compose fragment passes an allowlist before it is assembled — see the section below |

The webview itself is deliberately weak: `capabilities/default.json` grants no
blanket plugin permissions, and everything above happens in typed Rust commands
behind the IPC boundary.

## What it sends, and to whom

Nothing on its own initiative except the update check. There is no telemetry,
no analytics and no crash reporting service; [PRIVACY.md](PRIVACY.md) lists
every host the shipped code can reach, what is stored on disk and for how long,
and `src-tauri/tests/privacy_claims.rs` fails the build when a host appears in
the code that the document does not name.

A build that contacts something not on that list is a vulnerability report, not
a feature request.

## Service packages, and what stands between one and your machine

Until packages existed, this app only ever handed Docker files that were
compiled into its own binary. It now installs **definitions somebody else
wrote** and gives them to Docker, and a compose service is not a passive
description: the right four words in it are root on the host. So the threats are
enumerated rather than assumed, and each one names the thing that answers it.

| #   | Threat                                  | What it would cost               | What answers it                                                                                 |
| --- | --------------------------------------- | -------------------------------- | ----------------------------------------------------------------------------------------------- |
| T-1 | A forged registry (DNS, MITM)           | arbitrary packages installed     | HTTPS only — `http://` is refused before a request — plus the signature                         |
| T-2 | The package repository is taken over    | a bad package for every user     | minisign over the index, pinned keys, rotation and retirement (`signing.rs`, ADR 0021)          |
| T-3 | A malicious compose fragment            | **root on the host**             | the allowlist in `contracts/compose-policy.json`, checked after rendering (`compose_policy.rs`) |
| T-4 | A malicious image                       | execution inside the container   | a pinned digest, and `policy.market.allowedRegistries`                                          |
| T-5 | A template that exfiltrates `.env`      | credentials leave the machine    | the render context is built from the manifest and nothing else (`render.rs`)                    |
| T-6 | Downgrade or replay                     | an old package with a known hole | a monotonic `sequence` in the index; going backwards is refused                                 |
| T-7 | Path traversal (`../`) inside a package | arbitrary file writes            | every relative path is checked at the point of the read or write (`pkg::checked_relative`)      |
| T-8 | A zip bomb or a runaway body            | the disk fills                   | a size cap counted on **bytes received**, never on `Content-Length`, which the sender chooses   |

T-3 is the serious one and is the reason the allowlist is a contract file rather
than a constant: `tools/compose-check.mjs` in the packages repository reads it
before a package is published, and `compose_policy.rs` reads it again here
before a fragment is assembled. Only the second is still standing when a
repository has been taken over or a mirror is lying.

**A workspace may put its own copy of a package file in front of the published
one** (ADR 0031). That copy takes the same path: it is substituted from a
context the manifest defines and it passes the same allowlist. It cannot be a
manifest — overriding one would let a workspace run one image while the
catalogue reported another.

**No package key is pinned in a stock build**, and that is stated rather than
papered over. ADR 0015 gives the registry its own ed25519 pair and the ceremony
that would produce one has not happened; a build with no key **refuses** a
signed refresh instead of quietly accepting an unsigned index. An organisation
running its own mirror signs it and names its key in
`policy.market.additionalKeys`, and gets the whole chain today.

## What is in scope

- Escaping the workspace — reading or writing outside `<workspace>/projects/`
- Reaching Docker, `/etc/hosts` or a subprocess with input the user did not
  supply, for example through a crafted `stackvo.json` or `.env`
- Secrets leaving the process: in an event payload, in the diagnostic log, or in
  a rendered view
- Defeating update signature verification

## What is not

- Anything requiring an attacker who already has your user account. This app
  runs as you and manages your own machine; it is not a privilege boundary
  against yourself.
- Docker being able to do Docker things. Access to the socket is the point.
- Advisories on unmaintained transitive crates from Tauri's Linux stack. These
  are tracked in `src-tauri/deny.toml` with the reasoning; a _vulnerability_ in
  one of them is in scope.

## Supported versions

Only the latest release. This is a pre-1.0 project and there is no backport
branch.
