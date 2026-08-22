# Dumps

Catches `dump()` and `dd()` out of the response and shows them here instead. The formatting is done by Symfony's own dump server, running inside your project's container.

## Controls

| Control | What it does |
| --- | --- |
| Catch dump() and dd() | Turns capture on. It takes effect immediately and does not touch the container. |
| Search | Filters the visible dumps. |
| Copy | Puts the visible ones on the clipboard. |
| Pause | Stops new dumps being added to the list. |
| Clear | Deletes the list and the stored events. |
| Clicking a row | Opens the whole dump. |

## Worth knowing

- Nothing accumulates while capture is off. Turn it on first, then reload the page you are investigating.
- Capture stays on across pages. A dump from a queue worker or a console command is caught too.
- `dd()` ends the request; `dump()` does not. Both show up here.
- If the container is not carrying the bridge, the card says so and offers to recreate it.
