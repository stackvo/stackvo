# Xdebug

Step debugging for this project.

## Controls

| Control | What it does |
| --- | --- |
| Enabled / Disabled | Turns Xdebug on and off. |

## The first time is different

Turning it on for the first time puts the extension in the image and needs a **rebuild**. Every time after that only restarts the container: the extension stays in the image and costs nothing while it is off.

The second switch being much faster than the first is normal.

## IDE settings

The card lists the values to put into your IDE:

| Field | What for |
| --- | --- |
| Port | The port Xdebug connects on. |
| IDE key | The key that identifies the session. |
| Server name | The `PHP_IDE_CONFIG` value. |
| Path mapping | Which path in the container matches which path on your machine. Without it breakpoints do not bind. |
| Xdebug version | The installed version. |

## Worth knowing

- If the card says the running container is not carrying the Xdebug settings, restart the project.
- `stackvo up` from the command line does not layer this configuration and recreates the container without it.
- Xdebug and the profiler are two modes of one extension. They cannot both be on.

## IDE setup

The three values above are everything an IDE needs, and the path mapping is the one people get wrong — every local-environment tool's documentation names it as the usual reason a breakpoint never hits. This section fills them in.

| Control | What it does |
| --- | --- |
| Write configuration | Adds a `Listen for StackVo: <project>` entry to the project's `.vscode/launch.json`, with the mapping already filled in. |
| Update | Rewrites it after the port or the mapping moved. |
| Remove | Deletes only that entry. |
| Copy block | The configuration to paste, for a file this will not write. |

**VS Code is written; PhpStorm is not.** PhpStorm keeps `.idea/php.xml` and `.idea/workspace.xml` in memory and rewrites them when it exits, so a file edited underneath a running PhpStorm is a file PhpStorm overwrites — and you would be left with a tool claiming it configured something and an IDE that disagrees. So its server entry, with the name and both roots already in it, is offered to paste instead.

A `launch.json` with comments in it — which is what VS Code itself creates — is reported rather than rewritten, because stripping the comments to make the edit possible would delete your own notes.

The path mapping is written **remote to local**, and the local side is `${workspaceFolder}` rather than this machine's path, so the file still works for a colleague who clones the repository.

### Is anything listening?

The other reason a breakpoint never hits is not in any file: the IDE has to be listening on the debug port, and nothing in an IDE says loudly that it is not. The line above the list reads the operating system's own table of listening sockets and names the process holding the port, or says that nothing is.

It is a read, not a connection. Dialling the debug port to see whether anything answers would appear in your IDE as a debug session that immediately dropped.

## When a warning has a button

Three states need three different things done to them, and the pane offers each one where the problem is stated rather than leaving you to find it elsewhere on the page.

| What it says | What it needs | Why |
| --- | --- | --- |
| Not in the image yet | **Regenerate and rebuild** | The extension is compiled in, so the image has to be built before anything can happen. Minutes. |
| In the image, not in the container | **Recreate the container** | A container's environment is fixed when it is created, so restarting is not enough. Seconds. |
| Switched off, still running with it | **Recreate the container** | The same reason, pointing the other way: turning it off does not reach into a container that is already up. |

Nothing is done automatically. A rebuild recreates the container and takes minutes, and a switch that quietly started one would be a surprise you did not ask for — so the pane asks, which is what a warning with a button in it is.

Turning it off does **not** rebuild, and that is deliberate: the extension stays in the image, where it costs nothing while it is off, so turning debugging back on later is a container recreate rather than another build.

The pane re-reads itself when the work finishes. These commands return as soon as the work *starts* — that is what the operation console is for — so a screen that only re-read when the button returned would show you the old container.
