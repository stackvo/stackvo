# Network and TLS

The Docker network the services share, and whether they are served over HTTPS.

## Controls

| Control | What it does |
| --- | --- |
| Docker network | The name of the network every service joins. |
| Serve over HTTPS | Issues and mounts a local certificate for the domain suffix. |
| Redirect HTTP to HTTPS | Plain requests are answered with a redirect rather than the site. |
| Reset | Puts the network name back to its default. |

## Worth knowing

- Changing the network name recreates the containers on the next start.
- Turning HTTPS off does more than disable the certificate: the HTTPS entrypoint is not generated either, while every route still targets it. Nothing resolves until it is turned back on.
- The redirect only makes sense with HTTPS on. Redirecting to a scheme that is off goes nowhere, so the switch stays disabled.
