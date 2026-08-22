# Project details

One project: what it is built from, what it runs, and what it is doing now. The tabs on the right split it by subject; the bar across the top holds the actions that act on the whole project.

## The bar at the top

| Control | What it does |
| --- | --- |
| Status chip | What the engine reports: running, stopped, or not built. Read from Docker, not remembered. |
| Open in browser | Opens `https://<domain>` in your default browser. |
| Open a terminal | Opens your own terminal application with a shell inside the container. |
| Quick commands | The commands the project's framework offers. They run in your terminal. |
| Open in editor | Opens the project folder in the editor set under Settings → Preferences. |
| Open the folder | Reveals the project folder in Finder or Explorer. |
| Start / Stop | Brings the container up or takes it down. Nothing is rebuilt. |
| Rebuild | Regenerates the Dockerfile from `stackvo.json`, builds the image, recreates the container. |
| Restart | Stops and starts the same container. |
| Delete | Removes the container and the project's entry. Your folder on disk is untouched. |
| Refresh | Re-reads everything on the page from the engine. |

## Rebuild versus restart

These are different operations, and confusing them is the most common mistake.

| Action | What it does | When |
| --- | --- | --- |
| Restart | The same container stops and starts. | A process inside has wedged. |
| Rebuild | The Dockerfile is generated, the image is built, the container is recreated. | A PHP version, an extension or anything else in the image changed. |

If you changed a setting and nothing happened, you probably restarted when you needed to rebuild.

## The tabs

| Tab | What is on it |
| --- | --- |
| Indicator | Live CPU, memory, disk and network; where they go and what recent days looked like. |
| Configuration | `stackvo.json`, the services needed, the raw manifest, machine-only values, worktrees and the Dockerfile. |
| Container | The running process: Docker's facts, the ways in from outside, the workers, and a shell. |
| Logs | Container output and the project's log files. |
| Debugging | Xdebug, the profiler, dumps, the query log and the timeline. PHP only. |
| Runtime settings | `php.ini` for PHP, the dev server for Node. |
| Production image | The image that leaves this machine. |

## Worth knowing

- A tab only appears when it applies. A Node project has no `php.ini` and no Xdebug tab.
- A project that has never been built has no container facts to show.
