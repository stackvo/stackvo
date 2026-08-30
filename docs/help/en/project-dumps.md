# Debug signals

Catches what the code did out of the response and shows it here instead. Three things arrive: the values `dump()` and `dd()` were given, one row per **request** the project served, and one row per **queued job** the worker finished with.

## Controls

| Control | What it does |
| --- | --- |
| Catch dump() and dd() | Turns capture on. It takes effect immediately and does not touch the container. |
| Signal | Shows only dumps, only requests or only jobs. |
| Filter by source | Web, CLI or queue — where the code was running. |
| Search | Filters the visible rows, including by status. |
| Copy | Puts the visible ones on the clipboard. |
| Pause | Stops new rows being added to the list. |
| Clear | Deletes the list and the stored events. |
| Clicking a row | Opens the whole dump. |

## The three signals

- **Dumps** carry the value, the file and the line. Clicking the line opens it in your editor.
- **Requests** are one row per execution, with the HTTP status and how long it took. They are written by PHP's own shutdown hook, so a request that ended in a fatal has a row too — and so does an `artisan` command.
- **Jobs** are one row per attempt: the job class, whether it finished or threw, and how long it took. A job with `--tries=3` that always throws produces three rows, because that is what the queue did.

## Worth knowing

- Nothing accumulates while capture is off. Turn it on first, then reload the page you are investigating.
- Capture stays on across pages. A dump from a queue worker or a console command is caught too.
- `dd()` ends the request; `dump()` does not. Both show up here, and the request row after a `dd()` shows the 500 it set.
- Job rows come from the worker this app started. A `queue:work` you are running in your own terminal is not one, and produces no rows.
- Turning capture on shows what happens **next**. The worker's earlier output is not read back in.
- If the container is not carrying the bridge, the card says so and offers to recreate it.
