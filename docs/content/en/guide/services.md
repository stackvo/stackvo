# Services and databases

MySQL, Redis, Elasticsearch and the rest come from a **catalogue**, are
installed as **instances**, and are switched on **per project**. Three acts,
three places.

<figure markdown>
![The catalogue](../screenshots/web/market.webp){ loading=lazy }
<figcaption>The catalogue: services installed as instances, each with its own version and port.</figcaption>
</figure>

## 1. Install from the catalogue

**Catalogue → Available** lists what a source publishes, with versions.
**Install** puts the package on disk. Nothing runs yet.

StackVo carries no services inside itself; until a source is given nothing is
available. The catalogue is signed — the registry with minisign, each package
with a checksum — and an unsigned source is named as such.

## 2. Add an instance

**Catalogue → Service instances → Add** starts that version in this workspace.
Each instance has its own data and its own port.

The moment to set the root password is now: an image reads it only when it
first initialises an empty data directory. After that, the password is what it
is.

Two versions of one service run side by side — **MySQL 8.0 and 8.4** — and
each keeps its own data. **Make primary** sets the default for projects that
name no instance.

## 3. Switch it on for a project

**Project → Services** lists what the project declares and what this machine
runs:

| State | What it means |
| --- | --- |
| **On here** | Running, and the project can reach it. |
| **Not on here** | The project wants it and this machine does not run it. **Enable** starts it. |
| **Suggested** | A guess read from the project's own `.env` — `DB_CONNECTION=pgsql`, say. Never written on its own; **Write to stackvo.json** turns it into a declaration you commit. |

The connection string sits in the same pane. The password appears on a click.

## Two versions at once

Project A on MySQL 8.0, project B on 8.4, both running: services are
instances, not one global copy. Each project connects to the one it asked
for.

## Removing

**Remove** on an instance deletes its data with it. Take a
[snapshot](snapshots.md) first if you want to keep it.
