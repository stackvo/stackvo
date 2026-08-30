# Every project

The viewer that shows the dump output of every capturing project in one list.

## The toolbar

| Control | What it does |
| --- | --- |
| Project picker | Which projects to show. Empty means all of them. |
| Signal | Dumps, requests or jobs. |
| Search | Filters the visible rows, including by status. |
| Copy | Puts the visible ones on the clipboard. |
| Pause | Stops new dumps being added. |
| Clear | Deletes the list and the stored events. |
| Help | Explains how capture works. |

## The rows

Each row says which project it came from. A dump names the file and line it happened at, and clicking one opens the whole dump; a request names its status and duration, and a job names its class and how it ended.

## Worth knowing

- Capture is per project and works without touching the container.
- `dd()` ends the request; `dump()` does not. Both appear here.
- Job rows come from the workers this app started, not from a `queue:work` in your own terminal.
- If a project's container is not carrying the bridge, that project gets a warning and the container has to be recreated.
