# Share

A temporary public URL that forwards to this project. It is for webhook senders and other outside services that cannot reach a `.loc` domain.

The tunnel client runs as a sidecar container and dials out. No port is opened on this machine.

## Controls

| Control | What it does |
| --- | --- |
| Provider | Which service carries the tunnel. Each row says whether its token is stored. |
| Get a public URL | Starts the sidecar and shows the address. |
| Stop | Takes the sidecar down; the address stops working immediately. |
| Copy | Puts the address on the clipboard. |
| Token | Stores a provider's account token in the OS keystore. It is never shown again. |

## Choosing a provider

| Kind | Address | Account |
| --- | --- | --- |
| Anonymous quick tunnel | Changes on every start | Not needed |
| Provider that keeps an address | Stays the same | Needed |

A changing address is fine for "did the webhook arrive". An address you register once in somebody's console needs a stable one.

## Worth knowing

- The tunnel forwards to the container. If the project is stopped, the address looks alive and returns 502.
- The address is public while it runs. Anyone who has it reaches your local project.
- The first start downloads the provider's image, so it takes longer than the ones after it.
