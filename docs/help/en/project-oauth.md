# OAuth callback

The redirect URI to paste into an identity provider's console.

The redirect is sent to the browser; the provider never fetches this address itself. So the local address works for the flow. The only thing that varies is whether a provider will accept the string when you register it.

## Controls

| Control | What it does |
| --- | --- |
| Callback path | The route in your application, for example `/auth/callback`. It is normalised and echoed back. |
| Copy | Puts the local or the public address on the clipboard. |

## The two addresses

| Address | When it works |
| --- | --- |
| Local | `https://<project>.loc/auth/callback`. Always, for the flow on this machine. |
| Public | The same path on a running tunnel. Only while a tunnel is running. |

## Which providers accept which

The card splits them in two and gives each one's rule:

- Providers that accept any URL for a private app take the local address.
- Providers that verify the domain, or require a publicly resolvable one, need the tunnel.

## Worth knowing

- Do not register the address of an anonymous tunnel. It changes on every start, so the registration is stale the next day.
- If a provider insists on a public address, use a tunnel provider that keeps its address.
