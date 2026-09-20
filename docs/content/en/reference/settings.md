# Settings, `.env`

The stack's settings live in `<workspace>/.env`, one `KEY=VALUE` per line.
Every key is typed and defaulted in `contracts/env.schema.json`; this page
lists the ones the app reads today. You rarely edit the file — the
**Settings** page writes it — but knowing the names helps when a teammate's
preset or a policy file names one.

## How the file is read

- Blank lines and lines starting with `#` are ignored.
- Split on the **first** `=`; the rest is the value, trimmed. Values are
  never quoted — a quoted value keeps its quotes and is wrong.
- Booleans are `true` or `false`, lower case. Lists are comma-separated.
- The file is written only when a setting is changed. No `.env` at all is
  the normal state.

## Domain and HTTPS

| Key | Default | What it does |
| --- | --- | --- |
| `DEFAULT_TLD_SUFFIX` | `stackvo.loc` | The suffix every service and admin-UI hostname is built from. **Settings → Domain and network**. |
| `SSL_ENABLE` | `true` | Emit HTTPS routers and the certificate. |
| `REDIRECT_TO_HTTPS` | `true` | Send plain HTTP to HTTPS. |
| `IDLE_SUSPEND_MINUTES` | `0` | Stop a project Traefik has not routed to for this many minutes. `0` is off. |

## Services

Every service is configured by a `SERVICE_<NAME>_*` family, where `<NAME>`
is the catalogue id upper-cased with `-` as `_` — `mongo-express` becomes
`MONGO_EXPRESS`.

| Key | What it does |
| --- | --- |
| `SERVICE_<NAME>_ENABLE` | `true` or `false`. Gates both generation and the compose profile. |
| `SERVICE_<NAME>_VERSION` | The image tag that runs. |
| `SERVICE_<NAME>_VERSIONS` | The tags the settings sheet offers. |
| `SERVICE_<NAME>_HOST_PORT` | The port published on this machine. |
| `SERVICE_<NAME>_ROOT_PASSWORD` and friends | Credentials. Can be a `keychain:` reference — see [Workspace and files](workspace.md). |

## Web server limits

Rendered into the server block of every PHP project. **Settings → Web servers → Request limits**.

| Key | Default | Rendered as |
| --- | --- | --- |
| `SERVER_MAX_BODY_SIZE` | `1m` | `client_max_body_size`; Caddy honours it too |
| `SERVER_CLIENT_BODY_TIMEOUT` | `60` | `client_body_timeout` |
| `SERVER_KEEPALIVE_TIMEOUT` | `75` | `keepalive_timeout` |
| `SERVER_TCP_NODELAY` | `on` | `tcp_nodelay` |
| `SERVER_GZIP` | `off` | `gzip`; Caddy honours it too |
| `SERVER_GZIP_COMP_LEVEL` | `1` | `gzip_comp_level` |
| `SERVER_GZIP_TYPES` | empty | `gzip_types`; empty means nginx's own list |
| `SERVER_FASTCGI_CONNECT_TIMEOUT` | `60` | `fastcgi_connect_timeout` |
| `SERVER_FASTCGI_SEND_TIMEOUT` | `60` | `fastcgi_send_timeout` |
| `SERVER_FASTCGI_TIMEOUT` | `60` | `fastcgi_read_timeout` |

## Catalogue of runtimes

What the new-project panel offers.

| Key | Default |
| --- | --- |
| `SUPPORTED_SERVERS` | `nginx,apache,caddy,frankenphp,swoole,roadrunner` |
| `SUPPORTED_SERVERS_DEFAULT` | `nginx` |
| `SUPPORTED_LANGUAGES_PHP_VERSIONS` | `5.6` … `8.5` |
| `SUPPORTED_LANGUAGES_PHP_DEFAULT` | `8.4` |
| `SUPPORTED_LANGUAGES_PHP_EXTENSIONS` | The pickable extensions — always a subset of `php-extensions.json` |
| `SUPPORTED_LANGUAGES_NODEJS_VERSIONS` | `16,18,20,21,22,23` |
| `SUPPORTED_LANGUAGES_NODEJS_DEFAULT` | `22` |
| `SUPPORTED_LANGUAGES_PYTHON_DEFAULT` | `3.14` |
| `SUPPORTED_LANGUAGES_GO_DEFAULT` | `1.23` |
| `SUPPORTED_LANGUAGES_RUBY_DEFAULT` | `3.3` |
| `SUPPORTED_LANGUAGES_RUST_DEFAULT` | `1.84` |

## The PHP image

| Key | Default | What it does |
| --- | --- | --- |
| `PHP_DEFAULT_TOOLS` | `composer,nodejs` | Tools baked into every PHP image. Also `git`, `wget`, `unzip`. |
| `PHP_DEFAULT_APT_PACKAGES` | a list | System packages baked in — includes `strace`, `vim`, `htop`. |
| `PHP_TOOL_COMPOSER_VERSION` | `latest` | |
| `PHP_TOOL_NODEJS_VERSION` | `20` | The Node bundled inside the PHP image, for asset builds. |

## Docker

| Key | Default | What it does |
| --- | --- | --- |
| `DOCKER_DEFAULT_NETWORK` | `stackvo-net` | The network every container joins. |
| `STACKVO_VERSION` | — | Informational: which StackVo wrote this workspace. |

!!! note "Keys not listed here"
    The schema also carries keys from the retired Bash implementation, marked
    *dead* — nothing reads them. Leave them out of new files; the app ignores
    them in old ones.
