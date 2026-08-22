# Workers

This project's queue and schedule processes. Without this they would each need a terminal window.

## Controls

| Control | What it does |
| --- | --- |
| Start / Stop | Brings that process up or takes it down. |

Which kinds appear depends on your project's files. A Laravel project with a queue configured gets a queue worker; one with a schedule gets a scheduler.

## What the rows say

| Mark | What it means |
| --- | --- |
| Green | The process is running. |
| Grey | It is not. |
| Restart count | How many times the engine has restarted it. Not shown when it is zero. |

## Worth knowing

- The project has to be running first.
- A worker holds the code it started with. If you changed the code a queue worker runs, stop and start it.
- A restart count climbing on its own is a crash loop wearing a green mark. Look for the reason on the Logs tab.
