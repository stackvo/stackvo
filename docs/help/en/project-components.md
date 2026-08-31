# The rest of this repository

A repository with `api/` in Go, `web/` in Next.js and `worker/` in Python is **one project** here: one entry, one start, one certificate, one set of hostnames.

Every other tool in this category makes it three. Their unit is a *site* — one directory, one runtime, one hostname — so a monorepo becomes three entries you have to remember are related. A local binary cannot do otherwise: a directory has one runtime because the binary serving it has one.

## Declaring one

```json
{
  "name": "shop",
  "runtime": "php",
  "domain": "shop.loc",
  "components": {
    "api": {
      "runtime": "go",
      "path": "api",
      "domain": "api.shop.loc",
      "build": "go build -o bin/api ./cmd/api",
      "start": "./bin/api",
      "port": 8080
    },
    "worker": {
      "runtime": "python",
      "path": "worker",
      "start": "python worker.py"
    }
  }
}
```

`runtime` is `node` or one of the six language runtimes. `path` is the directory it is built from. `start` is the command its container runs. Everything else has a default.

**A component with no `domain` is not misconfigured.** It is reachable from the project's other containers and from nowhere outside — which is exactly what a queue worker wants, and forcing a hostname on one would invent a URL nobody asked for.

## Three declarations, three different things

| In `stackvo.json` | What it is | Shared? | Built here? | Routed? |
| --- | --- | --- | --- | --- |
| `services: ["mysql"]` | A **need**, satisfied from the catalogue | One per machine | No | No |
| `sidecars` | **Somebody else's image** | No | No | No |
| `components` | **This repository's own code** | No | Yes | Yes |

## What it does for you

- A **Dockerfile** and a `.dockerignore` written into each component's directory, from the same renderers a single-runtime project uses.
- A **compose service** in the project's own block, sharing the project's profile — so `stackvo up shop` brings all of it up, and stopping the project stops all of it.
- A **Traefik router** for each component that names a domain, plus a `/etc/hosts` entry and a name on the certificate. There is nothing extra to run.

## What is refused, and why

- **No host port.** A component is reachable from the project's containers and, through its hostname, from a browser. It never binds a port on your machine — that is what stops two clones of one repository fighting over 8080. A `ports` key is refused by name, on the manifest, with that reason.
- **The path stays inside the project.** `..`, an absolute path and `.` itself are all refused. A build context is what Docker reads *everything* under.
- **PHP is not a component runtime.** A PHP part needs a web server, a document root and a `php.ini` overlay — three things the project's own `runtime` already renders, and none of which generalises to several inside one project. Keep the PHP half as the project's runtime and declare the other languages here.
- **One hostname is one container.** Two components on the same domain is a routing rule that silently loses to whichever was read last, so the second is refused and the first stands.

## Worth knowing

- The container is `stackvo-<project>-<id>` — derived, never declared, so two clones of a repository are two containers rather than one collision. It shares its namespace with sidecars, so an id used by both is reported on the manifest.
- Components use the other containers' names to talk to each other: `stackvo-shop-api:8080`, not `localhost`.
- One broken component is a warning beside the ones that parsed. A project with nine working parts should not fail to open because of the tenth.
