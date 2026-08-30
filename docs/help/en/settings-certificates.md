# HTTPS certificate

One wildcard certificate covering the dashboard, every service and every project.

## Controls

| Control | What it does |
| --- | --- |
| Reissue the certificate | Regenerates it with the current list of domains. |
| Trust the CA (in a terminal) | Opens your terminal and runs the command that trusts the authority. |

## What the card shows

| Item | What it means |
| --- | --- |
| Current / needs reissue | Whether the certificate's coverage matches your domains. |
| CA trusted / untrusted | Whether this machine trusts the authority that issued it. |
| Expiry | How long the certificate is valid for. |
| Covered | The domains the certificate covers. |
| Not covered | Domains it does not. Those give a browser warning. |

## Where the CA is trusted

"Trusted" was one word and there is more than one store.

| Store | Who uses it |
| --- | --- |
| This machine's trust store | Safari, Chrome, Edge, `curl`, and anything else that asks the operating system |
| Firefox's own store | Firefox only — it does not use the system store, it carries its own per profile |

That second row is the one that produces the afternoon lost to it. mkcert installs into Firefox's store **only if `certutil` is on the machine**; without it, mkcert prints a warning and carries on. The install looks like it worked, the system store is green, and Firefox refuses every page.

The card names each store and, where the answer is no, what to do about it — install `nss` (which provides `certutil`) and run the trust step again. A store shown as neither trusted nor untrusted means the browser is not installed here, which is not a problem anybody has to fix.

## Why not real certificates from Let's Encrypt?

Because they cannot be issued for these names, and the reasons are worth stating rather than being discovered:

- A public certificate authority validates that **you control a name in public DNS**. `shop.loc` is not in public DNS and never will be — there is nothing for an authority to check.
- The HTTP-01 challenge needs port 80 on this machine reachable from the internet. A laptop behind a router is not.
- The DNS-01 challenge avoids that, and needs a real domain plus an API token for the DNS provider that holds it — which is a real setup, and not one a local development environment can assume.

What a public certificate would actually buy here is that **other devices** — a phone on the same network, a colleague's laptop — trust it without installing your CA. If that is the need, share the project through a tunnel: the provider terminates TLS with its own public certificate, which every device already trusts.

## Worth knowing

- Issuing a certificate needs `mkcert`. Without it the card says so and nothing can be reissued.
- macOS only lets trust settings be changed interactively; a windowed application cannot do it alone. That is why the button opens your terminal.
- After trusting, quit and reopen the browser completely. An open browser keeps using the old trust list.
