# Why was this request slow

One recorded request, with three instruments around it: the profile, the query log and the axis. Each of those has its own card on this tab and each answers a third of the question. This card answers the question.

## Start with a recording

The card opens on a php-spx recording, because a recording is the only thing here that names a request, says when it started and says how long it took. If nothing has been recorded yet, make one in the **php-spx** card below: switch the extension on, then ask it for the page you are investigating.

Pick a different recording from the picker to look at a different request. If your workspace runs more than one database whose log can be read, pick that too.

## What the evidence says

The findings come first because they are the answer. Two colours:

- **Amber** — something to change. A query shape that ran once per row, a request that spent its time waiting on the database, a single function holding a fifth of the run.
- **Blue** — something the evidence could not cover. The query log was off, the trace was too long to read whole, another recording overlaps this one.

## Where the time went

The bar splits the run into time spent inside a database driver and everything else.

The database half is time inside the driver's **own body** — `PDO`, `mysqli`, `pg_*`, `SQLite3`, the Mongo driver. A framework's query layer (Laravel's `Connection::run`, Doctrine's `executeQuery`) is counted as PHP, because the wait happens underneath it and counting both would count it twice.

If the split says nothing was in the database while the statement list below shows queries, the recording cannot answer the question: php-spx is set not to profile PHP's own functions, so the wait was charged to whichever of your functions called the driver. The card says so, and the switch is in the php-spx card.

## The three lists

Closed by default, because they are the evidence rather than the finding.

| List | What it holds |
| --- | --- |
| Functions | The heaviest functions by time in their own body, with their share of the run and how often they were called. |
| Statements | The repeated shapes first, then every statement the log holds inside this request, stamped by how far into the request it landed. |
| On one axis | Dumps, statements and mail together, in the order they happened. |

## How the join works, and what it cannot do

Everything except the profile is joined to the request **by time**. A recording claims a stretch of wall clock — when it started, plus how long it took — and this card shows what that stretch held.

That is a real limit and it is stated rather than hidden:

- A database log records the statement and the connection and **nothing about which HTTP request caused it**. Nothing here guesses. Anything else your site was doing while the recorded request ran is in these lists too.
- If another recording claims part of the same stretch, the card says so. Everything joined by time is then shared between them.
- Dumps are the exception: the debug bridge writes the request each one happened in, so those really do carry their own attribution.

Attributing a statement to a request for certain would need your application to say so — a header, a comment on the SQL, a collector inside your code. That is the thing this whole feature exists to avoid needing.

## Where the stretch itself comes from

The card says which of two things the window is, because they are not equally trustworthy.

- **Watched.** StackVo sent the request itself — from the button on this card's php-spx neighbour, or from `stackvo spx-record` — so it held the clock on both sides of it. That brackets the run whatever php-spx's own timestamp means.
- **Worked out.** The recording was made somewhere else, usually in php-spx's own control panel in a browser. The window is then its timestamp plus how long the run took, with one second of slack at each end for the rounding — php-spx files a start time as a whole second, so the true start is somewhere inside it.

The worked-out window assumes that timestamp is the **start** of the run. If it turns out to be the moment the file was written, a worked-out window sits one whole duration late. Recording from the button avoids the question entirely.

## Worth knowing

- The lists are capped at 25 rows. The counts in the headings are the real ones, so you can see what is not being shown.
- The query log costs write throughput on every statement. Switch it off in the **Query log** card when you are done.
