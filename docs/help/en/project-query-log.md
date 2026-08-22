# Query log

What the database was actually asked. It works with no agent, no rebuild and no code in your application.

## This is a session, not a feed

You turn recording on, reload the page you are investigating, look, and turn it off. Leaving it on is not a smaller version of this feature but a worse one: the log records every statement unsampled, and every write costs something.

Stopping also clears what was collected, because the log holds the statement text.

## Controls

| Control | What it does |
| --- | --- |
| Database | Which database's log to read. |
| Record queries | Turns recording on and off. |
| Start again | Clears what was collected; recording stays on. |

## Two lists

- **Repeats** — how many times the same shape was asked. This is the finding: a page asking one query three hundred times shows up here.
- **Statements** — every statement recorded, in order. This is the evidence.

## Worth knowing

- Only databases whose log can be read are supported: MySQL, MariaDB, Postgres and Mongo. If your workspace runs none of them, the card says so.
- On Postgres the statements are also written to the server's own log file inside its container. Stopping ends the session here but does not rewrite that file.
- The log holds statement text. Queries containing a password or personal data are recorded too.
