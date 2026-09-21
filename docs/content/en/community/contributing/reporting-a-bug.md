# Reporting a bug

Something behaves differently than it should. A report that can be reproduced
is usually fixed in the next release; one that cannot is usually closed with a
question. This page is about writing the first kind.

!!! danger "Security problems are not bugs"

    A way to escape the workspace, to reach Docker with input you did not
    supply, or to make a secret leave the process is reported
    [privately](reporting-a-vulnerability.md), never as an issue.

## Before you report

### Update

Only the latest release is supported and there is no backport branch.
**Settings → Updates** shows which version you have and installs the new one.
If the bug is gone afterwards, it was already fixed.

### Ask Doctor

**Settings → Doctor** checks the machine: Docker, the socket, ports, the hosts
file, the certificate, disk. Most first-run problems are named there with the
repair beside them. If Doctor names yours, it is a machine problem, and
[Troubleshooting](../../getting-started/troubleshooting.md) is faster than an
issue.

### Search

Look through the [issues](https://github.com/stackvo/stackvo/issues?q=is%3Aissue)
and [discussions](https://github.com/stackvo/stackvo/discussions), closed ones
included. If you find it, add what is new about your case there instead of
opening another.

## Write the report

Open a [bug report](https://github.com/stackvo/stackvo/issues/new/choose). The
template asks for five things, and each has a reason.

### What happened

What you did, what you expected, and what you got instead, in that order. One
bug per report — two bugs in one issue get half an answer each.

A good title is a sentence somebody could search for:

| | |
| --- | --- |
| :material-check: | *Restoring a named snapshot fails with `no such volume` after the project was renamed* |
| :material-close: | *Snapshots broken* |

### Version and platform

**Settings → Updates** shows the version. Name the operating system and the
Docker runtime — Docker Desktop, Colima, OrbStack, Podman, a plain engine —
with its version. Most of what goes wrong here is about the machine rather
than the code, and this is the fastest way to tell the two apart.

### The log

**Settings → Application log → Save a diagnostic bundle** writes the log,
the preflight checks, the Doctor report and any crash reports into one
archive. Passwords and tokens are masked as the log is written, and the
archive is plain text inside — read it before you attach it.

Attach the bundle rather than pasting a line from it. The line before the
error is usually the one that explains it.

### Steps to reproduce

Numbered, from a state the reader can get to. "Create a project from the
Laravel template, enable Redis, restart" is a reproduction; "it happens
sometimes" is a clue. If it needs a particular manifest, paste the
`stackvo.json`.

## What happens next

The maintainer reads every report; most get an answer, and "most" is the
honest word. A report that cannot be reproduced gets a question and, if
nothing comes back, is closed. If you would rather fix it than wait,
[Making a pull request](making-a-pull-request.md) is the shorter path.
