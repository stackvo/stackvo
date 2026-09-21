# The manifest

`stackvo.json` sits in the project folder and describes the project. It is
the only file you are meant to edit by hand; everything else is rendered
from it. The contract is `contracts/project.schema.json` in the repository;
this page is that contract in plain words.

Two fields are required: `name` and `domain`. Everything else has a default.

## Top level

| Field | Type | Default | What it says |
| --- | --- | --- | --- |
| `name` | string | — | The project's identifier. Must match the folder name. Lower case, letters, digits, dots, dashes, underscores. |
| `domain` | string | — | The address the project opens at. Convention: `<name>.loc`. Needs a hosts line, which the app writes. |
| `runtime` | `php` `node` `python` `go` `ruby` `rust` `bun` `deno` | `php` | What runs the project. Absent means PHP. |
| `server` | `nginx` `apache` `caddy` `frankenphp` `swoole` `roadrunner` | `nginx` | The web server, for PHP. Ignored for other runtimes — Traefik proxies straight to the app's port. |
| `document_root` | string | `public` | The folder the web server publishes, relative to the project. No leading slash. |
| `aliases` | list of strings | `[]` | Extra hostnames the same project answers on. `*.shop.loc` is a wildcard: it goes into the certificate and the router, not the hosts file. |
| `lan_share` | boolean | `false` | Also answer on a name other devices on the network can resolve, through sslip.io. |
| `services` | list of ids | `[]` | The backing services the project needs — `mysql`, `redis`, `mailpit` — by catalogue id. The half of the environment a teammate gets by cloning. |
| `commands` | object | — | Commands the project offers as buttons. See below. |
| `hooks` | object | `{}` | Commands to run after a build, after a start, before a stop. |
| `schedule` | list | `[]` | Named jobs on a timer, each with its own log. |
| `components` | object | — | Other directories of this repository, each with its own runtime and address. |
| `sidecars` | object | — | Containers the project needs that the catalogue does not have. They come and go with the project. |
| `providers` | object | — | Named places the project's data really lives, and how to fetch it or send it back. |

## The runtime block

One block, named after the runtime. `php` for PHP, `node` for Node, and
`python`, `go`, `ruby`, `rust`, `bun` or `deno` for the others. Every field
inside is optional; an absent field takes the ecosystem's default.

=== "php"

    ```json
    "php": {
      "version": "8.4",
      "extensions": ["redis", "intl", "gd"],
      "xdebug": false
    }
    ```

    | Field | What it says |
    | --- | --- |
    | `version` | `5.6` to `8.5`. Default `8.4`. |
    | `extensions` | Compiled into the image. One that cannot be built on the chosen version is flagged before you save. |
    | `xdebug` | Whether the extension is in the image. Toggled from the Xdebug card. |

=== "node"

    ```json
    "node": {
      "version": "22",
      "package_manager": "pnpm",
      "install": "pnpm install",
      "build": "pnpm build",
      "start": "pnpm start",
      "port": 3000
    }
    ```

    | Field | What it says |
    | --- | --- |
    | `version` | `16` to `23`. Default `22`. |
    | `package_manager` | Enables Corepack, so `packageManager` in `package.json` pins a version. |
    | `install`, `build`, `start` | The three commands. `build` may be empty. |
    | `port` | What the app listens on inside the container. It must bind to `0.0.0.0`. |

=== "python, go, ruby, rust, bun, deno"

    ```json
    "python": { "version": "3.12", "install": "pip install -r requirements.txt", "start": "python app.py", "port": 8000 }
    ```

    The same five fields — `version`, `install`, `build`, `start`, `port` —
    with the ecosystem's defaults when absent.

## Commands

```json
"commands": {
  "reindex": {
    "exec": ["php", "artisan", "app:reindex"],
    "about": "Rebuild the search index",
    "interactive": false
  }
}
```

| Field | What it says |
| --- | --- |
| id (the key) | Lower-case letters, digits and dashes. The button's name. |
| `exec` | The program and its arguments, as a list. No shell, so nothing is interpreted. |
| `about` | The one line shown under the button. |
| `interactive` | Whether it needs a terminal. |

Commands run in the project's container and nowhere else.

## Hooks

```json
"hooks": {
  "post-build": [["composer", "install"]],
  "post-start": [["php", "artisan", "migrate", "--force"]],
  "pre-stop": []
}
```

Each is a list of commands, run in the container. There is no `pre-start`,
on purpose: before a start there is no container to run in.

## Schedule

```json
"schedule": [
  { "label": "Nightly report", "cron": "0 2 * * *", "exec": ["php", "artisan", "report:nightly"], "enabled": true }
]
```

Named jobs, each with its own last run and its own log — rather than one
all-or-nothing scheduler process.

## Components

```json
"components": {
  "api":    { "runtime": "go",     "path": "api",    "port": 8080 },
  "web":    { "runtime": "nodejs", "path": "web",    "port": 3000 },
  "worker": { "runtime": "python", "path": "worker" }
}
```

A monorepo as one project. Each component gets its own Dockerfile, compose
service and router, at `<component>.<domain>` unless `domain` says
otherwise. `version`, `install`, `build` and `start` work as in the runtime
block. No component can open a host port.

## Sidecars

```json
"sidecars": {
  "chromium": { "image": "selenium/standalone-chromium:latest", "about": "Browser for Dusk", "env": { "SE_NODE_MAX_SESSIONS": "2" } }
}
```

A container beside the project that the catalogue does not provide. It is
rendered into the project's own compose block, comes up and goes down with
the project, and is not a shared service.

## A full example

```json
{
  "name": "shop",
  "domain": "shop.loc",
  "aliases": ["admin.shop.loc"],
  "runtime": "php",
  "server": "nginx",
  "document_root": "public",
  "php": { "version": "8.4", "extensions": ["redis", "intl", "gd"] },
  "services": ["mysql", "redis", "mailpit"],
  "commands": {
    "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Rebuild the search index" }
  },
  "hooks": { "post-start": [["php", "artisan", "migrate", "--force"]] }
}
```

## Two files beside it

| File | What it carries | Committed? |
| --- | --- | --- |
| `.stackvo/site.json` | Environment variables for the container, directory listing, SSH agent forwarding. | Yes — it travels with the repository. |
| `stackvo.preset.json` | Service **versions** and the shareable settings that live in `.env`. Can never hold a secret. | Yes — see [Sharing and teams](../guide/sharing.md). |
