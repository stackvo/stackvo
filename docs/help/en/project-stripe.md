# Stripe webhooks

Forwards live Stripe events into this project. The CLI connects outward, so nothing here has to be reachable from the internet.

## Controls

| Control | What it does |
| --- | --- |
| Secret or restricted API key | Stored in the OS keystore. The field does not show what was stored. |
| Save | Writes the key. |
| Forget | Clears it. |
| Start | Runs `stripe listen` and forwards events into the container. |
| Stop | Ends the session. |
| Copy | Puts the signing secret on the clipboard. |

## How it differs from a tunnel

A tunnel's address changes on every start, so the webhook registration and its signing secret have to be renewed each time. This has no address, and the signing secret stays the same for the session.

## Worth knowing

- Use a restricted key where you can. This forwards real events into an application you are editing.
- The signing secret is printed per session. If you stopped and started, put the new one into your application.
