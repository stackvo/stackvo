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
| Ask for a password | Puts basic authentication in front of the link. StackVo generates the password and can show it again — unlike a token, it has to be read out to whoever opens the link. |
| The address | The name this provider should keep between starts, where it can keep one. |

## Choosing a provider

| Kind | Address | Account |
| --- | --- | --- |
| Anonymous quick tunnel | Changes on every start | Not needed |
| Provider that keeps an address | Stays the same | Needed |

A changing address is fine for "did the webhook arrive". An address you register once in somebody's console needs a stable one.

## A password in front of the link

Switching it on stores a credential for this project and, from the **next start**, puts a small nginx container between the tunnel and the project — so it works the same whichever provider carries the tunnel. A tunnel that is already running was opened without it, and the pane says so.

The password is kept in the OS keystore, never in the workspace, and it is the one secret this app will show you again: it has to be typed into a browser on somebody else's device.

While it is on, the `Authorization` header belongs to the guard and does not reach the application.

## Keeping the same address

Some providers can be asked to keep an address between starts; three of them cannot and say so instead of offering a field. The name is a **request**: if the provider still holds it from a moment ago it quietly assigns a different one, and the pane says when what came back is not what was asked for.

A named Cloudflare tunnel is the exception that needs the field filled in before it will start — Cloudflare routes it from its own dashboard and the client never prints the address, so the one you type here is the one shown.

## Worth knowing

- The tunnel forwards to the container. If the project is stopped, the address looks alive and returns 502.
- Without a password, the address is public while it runs. Anyone who has it reaches your local project.
- The first start downloads the provider's image, so it takes longer than the ones after it.
