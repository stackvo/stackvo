# Reverse proxy

Traefik. Every project and management interface is reached through it, and it terminates TLS.

## Controls

| Control | What it does |
| --- | --- |
| Open the dashboard | Opens Traefik's own dashboard in your browser. |

## What the card shows

Published ports: the ports the proxy listens on for the host. Usually 80 and 443.

## Worth knowing

- Projects do not publish their own ports. The proxy reaches them by name over the Docker network.
- If something else holds port 80 or 443, the proxy cannot start. The doctor reports it.
