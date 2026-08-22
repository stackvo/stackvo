# Automatic backups

Scheduled database snapshots, kept in the workspace.

## Controls

| Control | What it does |
| --- | --- |
| Snapshot frequency | Never, hourly, daily or weekly. |
| Scheduled snapshots to keep | Scheduled snapshots older than this count are deleted. |

## How the schedule is measured

From the last snapshot, not from the clock. A laptop that was closed for three days owes one snapshot when it opens, not three.

## Worth knowing

- Only running databases are backed up. A stopped service is skipped.
- Snapshots you named yourself are never deleted and do not count towards the limit. The number is for scheduled ones only.
