# Where it applies

Not every server is configured through a file. This card says which ones the settings above can be written to.

## Support

| Server | Status |
| --- | --- |
| nginx | Request limits and extra directives are written. |
| Caddy | Request limits and extra directives are written. |
| FrankenPHP | Extra directives only; its Caddyfile does not carry the request limits. |
| Apache | Configured inside its own Dockerfile; there is no file to add directives to. |
| Swoole | Configured by an inline script; there is no file to add directives to. |

## Worth knowing

- On an unsupported server, changing the settings does nothing. The card says so up front so it does not fail silently.
- The server is chosen under Project defaults; each project can also pick its own.
