# Request timeline

What the code thought it had, what it asked the database for, and what it sent — on one axis, for one page load.

## Two kinds of row

| Row | How it is placed |
| --- | --- |
| Dumps | Grouped by request. A dump knows which request it happened in. |
| Queries and mail | Placed by time only, outside the groups. |

Queries are not grouped because no database log records which HTTP request produced a statement. Guessing from what sits either side would be silently wrong the first time two requests overlap.

## Controls

| Control | What it does |
| --- | --- |
| Database | Whose queries go on the axis. |
| Refresh | Reads the timeline again. |

## Worth knowing

- If the query log is not recording, only dumps appear here. Turn it on in the card above, reload the page you are investigating, then refresh this one.
- Dump capture has to be on as well, or there is nothing to group.
