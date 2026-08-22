# On this network

Opens this project on a phone or another computer on the same network.

The name resolves through sslip.io. Nothing is registered, nothing is published, and no traffic leaves the network.

## Controls

| Control | What it does |
| --- | --- |
| Answer on a name other devices can resolve | Turns the shared name on and writes the intent into the manifest. |
| Open | Opens the address in your browser. |
| Copy | Puts the address on the clipboard. |

## How it differs from Share

| | On this network | Share |
| --- | --- | --- |
| Reaches | The local network | The internet |
| Sidecar container | Not needed | Needed |
| The cost | A certificate warning on the visiting device | The address is public |

The certificate is issued by this machine's own authority. A phone does not know that authority, so the visitor gets a warning to accept.

## Worth knowing

- This is a name in the router and the certificate, not a forward to a running container. It is correct while the project is stopped.
- If no address appears, this machine has no network address to build one from.
