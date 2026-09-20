# FAQ

The questions that arrive most often, answered once. If yours is not here, [Discussions](https://github.com/stackvo/stackvo/discussions) is the place to ask it.

## Is it really free?

Yes. Free, MIT-licensed, every feature on every platform. There is no paid
tier, no account and no telemetry, and nothing is going to be gated behind
one. [Sponsoring](../insiders/sponsoring.md) is what keeps it maintained; it
buys nothing that everybody else does not get.

## Is Docker required?

Yes. Docker Desktop, Docker Engine, or an API-compatible runtime — Podman,
Colima, OrbStack. The engine's name is only a label; nothing branches on
which of them answered.

## Is it only for Laravel?

No. Laravel, Symfony and WordPress get templates; a project is any folder
with a runtime — PHP 5.6 to 8.5, Node, Python, Go, Ruby, Rust — and a
monorepo mixing them is one project.

## Why isn't it code-signed?

The app is distributed from GitHub Releases and nowhere else. An Apple
Developer membership and an Authenticode certificate are recurring costs
with an identity attached, and skipping them was the last external
dependency dropped from the chain. In exchange every release publishes
`SHA256SUMS`, and the updater verifies a minisign signature.

## What happens to my existing StackVo (Bash / web UI) setup?

It keeps working. Both read the same `stackvo.json` and `.env`, so a
project created in either works in the other. That compatibility is enforced
by a checked-in contract and a validator, not by convention.

## Can I run two versions of the same service at once?

Yes. Services are installed as instances; MySQL 8.0 and 8.4 run side by
side, and each project connects to the one it asked for.

## What is the state of Windows support?

The pure logic — drive-letter to bind-mount conversion, named-pipe
detection, `DOCKER_HOST` scheme stripping — is tested on every platform, and
Windows is in the CI matrix. What a compiler cannot answer is still
unverified: the hosts-file write through UAC, the named pipe against a real
Docker Desktop, and whether a domain resolves in a browser there.

## Where does my data go?

Nowhere. See [What leaves the machine](privacy.md).

## Can I use it on a server or in CI?

It is not a design goal. The CLI makes headless use technically possible,
but it has not been tested end to end, so it is not called supported.

## What does Docker cost me compared with a native tool?

A first install that includes Docker and its images, an image build on a
project's first start, and the Docker VM's idle memory. In return, every
project's environment is a container: what runs is what a Dockerfile says.
The full table is on [How it compares](comparison.md).
