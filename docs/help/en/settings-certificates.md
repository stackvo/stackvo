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

## Worth knowing

- Issuing a certificate needs `mkcert`. Without it the card says so and nothing can be reissued.
- macOS only lets trust settings be changed interactively; a windowed application cannot do it alone. That is why the button opens your terminal.
- After trusting, quit and reopen the browser completely. An open browser keeps using the old trust list.
