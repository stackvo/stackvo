# Telescope, Horizon and Pulse

All three of these ship a web dashboard. All three open in the `local` environment without authentication. StackVo already serves this project on its own domain, over a certificate your browser trusts.

So `https://shop.loc/horizon` has worked all along. Nothing anywhere said so, which is why nobody clicked it.

A link would have been the cheap half of this card, and not the useful half.

## Each of the three goes quietly empty, and every container stays green

| Dashboard | Why it sits there with nothing in it |
| --- | --- |
| **Horizon** | The queue connection has to be `redis`. And its metrics graphs stay flat until `horizon:snapshot` runs **every five minutes** — nothing else writes a snapshot |
| **Telescope** | `telescope:install` and `migrate` have to have run. And without a daily `telescope:prune`, `telescope_entries` grows for as long as the project is open — the symptom is a full disk, not a slow dashboard |
| **Pulse** | Its storage wants MySQL, MariaDB or PostgreSQL and **refuses SQLite**. With Redis ingest it wants a Redis connection *separate from the queue's*. And `pulse:check` is a long-running process |

That last one is why Pulse appears on the **Workers** card rather than in the schedule. `horizon:snapshot` and `telescope:prune` start, do a thing and exit — that is what a schedule is for. `pulse:check` is a loop that does not return, and a schedule entry for it would start a second copy every time it fired. `pulse:work` joins it only when your `.env` says `PULSE_INGEST=redis`, because with storage ingest there is nothing for it to drain.

## What this card actually knows

**StackVo reads `.env` and `composer.lock`. It does not read `config/*.php`.**

A project that has run `config:cache` has a compiled configuration this app cannot see, and it can say something different from either file. So nothing on this card is a verdict:

* every row **names the key it read** and quotes the value it found;
* the sentence about a cached configuration sits **beside that row**, not at the top of the pane.

That second point is deliberate. A warning at the top of a screen and a row at the bottom are two things you have to join up yourself, and a check that calls something broken without having measured it is the check people learn to ignore.

**Two things are not claimed at all.** Whether Telescope's migrations have run is a question about a database this does not query — it is stated as a precondition and never reported as a state. And whether you are on **Redis Cluster**, which Horizon does not support, cannot be read out of `.env` without guessing what `config/database.php` does with the value. It is a sentence here instead of a row that looks measured.

## The two scheduled commands

Where one is missing, this card offers to add it to **this project's schedule** — the same table the Scheduler card shows, with its own log and last run, stored in `stackvo.json` so it travels with the repository.

It goes in through the schedule's single writer rather than through a verb of its own, so the manifest and the generated schedule cannot come apart. A job that is already there is matched on **the artisan command it runs**, not on its label — so renaming it does not make this offer it a second time.

## The addresses

Each link uses that dashboard's own default path. A project that moved its dashboard moved it in `config/*.php`, which this app does not read, and the line beside the link says so.

## Scout: the service is on, the index is empty

Meilisearch and Typesense are catalogue services, so switching one on is a click in the Market. What nothing said is the **next step**.

An empty Meilisearch returns *nothing* for every search. The application looks broken, and **every container is green** while it happens. That is this whole card's pattern in its purest form: the piece is here, and what was never said is its precondition.

This one gets a sentence and not a button, and the reason is worth stating. The commands are:

```
php artisan scout:import "App\Models\Post"
php artisan scout:sync-index-settings
```

The first takes a **model class name StackVo cannot know**. A button that filled in something it had guessed and ran it is exactly what the command catalogue refuses — the same rule that keeps `migrate:fresh` out of it.

The mechanism you need already exists: a project declares its own commands in the `commands` block of `stackvo.json`, and they appear beside the built-in ones. Put your `scout:import` line there.

The note appears only when **both** halves are true — `laravel/scout` in `composer.lock` *and* a `SCOUT_DRIVER` of `meilisearch` or `typesense`. Either alone would be the wrong sentence: Scout on the `database` driver has no index to fill, and a Meilisearch you are running for something else is none of this card's business.
