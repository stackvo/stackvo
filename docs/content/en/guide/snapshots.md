# Snapshots and backups

A snapshot is a named copy of a database. Take one before a migration,
restore it by name if the migration goes wrong.

<figure markdown>
![Scheduled jobs and snapshots](../screenshots/web/project-detail-jobs.webp){ loading=lazy }
<figcaption>Scheduled jobs: named jobs on a timer, snapshots among them.</figcaption>
</figure>

## Take one

**Project → Snapshots** → give it a name → **Take**. It adds a file and
changes nothing else.

## Restore one

Same pane. Restore asks for confirmation, because it writes over live rows.
That is also why restoring is deliberately *not* offered to
[AI assistants](ai-assistants.md): taking a snapshot is, restoring is not.

## A schedule

**Settings → Preferences → Automatic backups** makes it automatic: hourly, daily or weekly, keeping
the last N.

The schedule is measured **from the last snapshot, not from the clock**. A
laptop that was closed for three days owes one snapshot when it opens, not
three.

- Only running databases are backed up; a stopped service is skipped.
- Snapshots you named yourself are never deleted by the schedule and do not
  count towards the limit.

## Before removing an instance

**Remove** on a service instance deletes its data. Take a snapshot first.
