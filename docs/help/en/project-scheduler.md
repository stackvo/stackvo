# Scheduled jobs

Named jobs on a timer, each with its own frequency, its own last run and its own log. The Workers pane above runs Laravel's own scheduler as one process; this is the table of individual jobs, and it answers a different question — not "is the scheduler up?" but "did *that* job run?".

## Controls

| Control | What it does |
| --- | --- |
| Start / Stop | Brings the scheduler sidecar up or takes it down. Nothing fires while it is down. |
| New job | Opens the form. Nothing is written until you save. |
| Run now | Runs one job immediately, by the same path a tick would — so its log and last run are written the same way. |
| Pause / Resume | Takes a job out of the schedule without losing the command. |
| Log | The tail of that job's own log. |

## Job types

| Type | What it runs |
| --- | --- |
| Laravel scheduler | `php artisan schedule:run`, once per tick. Use this if your schedule lives in `routes/console.php`. |
| Artisan command | `php artisan` plus whatever you type. |
| Custom command | Whatever you type, run as-is. |

Each word you type becomes one argument. There is no shell, so `&&`, pipes, globs and `$VAR` do not work — and that is deliberate, for the same reason project hooks work that way. A job that needs those is a script: name the script instead, as in `sh scripts/nightly.sh`.

## Frequency

Pick a preset, or choose **Advanced** and write a cron expression yourself. Five fields — minute, hour, day of month, month, day of week — and the portable subset of the syntax: `*`, a number, `a-b`, `*/n`, `a-b/n`, and comma-separated lists of those. Names like `MON` and macros like `@daily` are not accepted; write the numbers.

## What the rows say

| Mark | What it means |
| --- | --- |
| Green clock | The job is in the schedule. |
| Grey pause | The job is paused. It keeps its command and its log. |
| Last run | When it last ran, and whether it worked. Red means it failed. |
| Restart count | How many times the engine has restarted the scheduler. Not shown when it is zero. |

## Worth knowing

- The project has to be running first: a job runs in a sidecar built from the project's own image, so it sees the same PHP, the same extensions and the same `.env` as the site.
- Docker supervises the scheduler with `unless-stopped`, so your jobs keep firing while this app is closed.
- The schedule is stored in `stackvo.json`, so it travels with the repository. A clone gets the same jobs.
- A job's name also names its log — "Cache cleanup" writes `cache-cleanup.log`. Renaming a job starts a new log rather than renaming the old one.
- The failure a job records is a yes or a no rather than a number: every failing command reports the same code, so the reason lives in the log, not in the status.
