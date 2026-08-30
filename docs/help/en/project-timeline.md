# Request timeline

What the code thought it had, what it asked the database for, and what it sent — on one axis, for one page load.

## How a row is placed

| Row | How it is placed |
| --- | --- |
| Dumps and requests | Grouped by request. Both know which request they happened in. |
| Queries, mail and jobs | Placed by time only, outside the groups. |

Queries are not grouped because no database log records which HTTP request produced a statement. Guessing from what sits either side would be silently wrong the first time two requests overlap. A job is the same case for a different reason: it was dispatched by a request that finished before the job started.

Two of the rows are **stretches** rather than instants — a request and a job each cover a span — and each is drawn at its end, which is the moment its duration and its outcome became knowable.

## Controls

| Control | What it does |
| --- | --- |
| Database | Whose queries go on the axis. |
| Refresh | Reads the timeline again. |

## Worth knowing

- If the query log is not recording, only dumps appear here. Turn it on in the card above, reload the page you are investigating, then refresh this one.
- Dump capture has to be on as well, or there is nothing to group. It is the same switch that brings requests and jobs onto the axis.
