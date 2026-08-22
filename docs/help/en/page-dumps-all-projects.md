# Every project

The viewer that shows the dump output of every capturing project in one list.

## The toolbar

| Control | What it does |
| --- | --- |
| Project picker | Which projects to show. Empty means all of them. |
| Search | Filters the visible dumps. |
| Copy | Puts the visible ones on the clipboard. |
| Pause | Stops new dumps being added. |
| Clear | Deletes the list and the stored events. |
| Help | Explains how capture works. |

## The rows

Each row says which project it came from and which file and line it happened at. Clicking one opens the whole dump.

## Worth knowing

- Capture is per project and works without touching the container.
- `dd()` ends the request; `dump()` does not. Both appear here.
- If a project's container is not carrying the bridge, that project gets a warning and the container has to be recreated.
