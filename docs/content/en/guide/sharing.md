# Sharing and teams

Four ways out of your machine, for four different needs.

<figure markdown>
![The Release tab of a project](../screenshots/web/project-detail-release.webp){ loading=lazy }
<figcaption>Sharing: the public tunnel, the LAN address, the production image and the devcontainer.</figcaption>
</figure>

## A public URL

**Project → Share → Get a public URL.** A tunnel client runs as a sidecar
container and dials out; no port is opened on this machine. For webhook
senders and other outside services that cannot reach a `.loc` domain.

Nine providers: Cloudflare (anonymous and named), ngrok, Tailscale, zrok,
Pinggy, localtunnel, localhost.run, LocalXpose.

| Kind | Address | Account |
| --- | --- | --- |
| Anonymous quick tunnel | Changes on every start | Not needed |
| A provider that keeps an address | Stays the same | Needed; the token goes into the OS keystore |

**Ask for a password** puts basic authentication in front of the link.
**Stop** takes the sidecar down and the address stops working immediately.

## A phone on the same Wi-Fi

**Project → On this network** — a name other devices can resolve, nothing
leaving the network, one certificate warning to accept on the phone. See
[Domains and HTTPS](domains-https.md).

## The same stack for a colleague

`stackvo.json` says *which* services. It cannot say which **versions**,
because those live in `.env` — the one file nobody commits. That half is a
**preset**: `stackvo.preset.json`, beside the manifest, in the repository.

```json
{
  "services": { "mysql": { "enabled": true, "version": "8.4" },
                "redis": { "enabled": true, "version": "7.2" } },
  "settings": { "DEFAULT_TLD_SUFFIX": "loc" }
}
```

Export one from **Settings → Workspace**. A colleague clones, opens the
project, and the **Services** card says a preset is there and **shows the
plan first** — a file that arrived with somebody else's clone must not rewrite
a stack because a page was opened. A preset can never carry a secret; the
schema has nowhere to put one.

## Someone without StackVo

**Project → Devcontainer → Write files into the project** puts a `.devcontainer/` into the
project, so a colleague can open the repository in VS Code or GitHub
Codespaces and get the same container: the same PHP version, extensions and
web server, the same services at the same versions. Passwords leave as
names read from a gitignored `.devcontainer/.env`. The files are meant to be
committed, and are rewritten from the manifest every time.

## The image you ship

**Project → Production image** builds a shippable image from the one the
project already runs — same PHP, same extensions, same web server, with your
code inside and Xdebug out. **Check** verifies it, **Push** sends it to a
registry, **Deployment recipe** gives you a compose file that runs it. Only a
verified image is pushed, and only to a tag carrying a registry name.
