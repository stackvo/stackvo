# Container

What Docker reports about this project's container. Nothing here is a setting; every field is read from the engine.

## Fields

| Field | What it means |
| --- | --- |
| Name | The container's name, `stackvo-<project>`. This is what `docker exec` and `docker logs` want. |
| Uptime | How long the current run has lasted. Resets on restart. |
| Restart policy | What the engine does when the process inside exits. |
| DNS record | Whether the domain resolves on this machine. With no record the browser cannot reach the project. |
| State | Docker's own word: running, exited, created. |
| Created | When the container was made. That is the last rebuild, not the last start. |
| Container ID | The short hash. |
| Image | The image the container was made from. |
| Restart count | How many times the engine has restarted it. |
| Image size | What the image takes on disk. |
| Gateway | The container's address on the stack network. |
| Port mappings | Ports published to the host. |

## Controls

| Control | What it does |
| --- | --- |
| Copy | Puts the name, ID or image on the clipboard. |

## Worth knowing

- The card is empty until the project has been built. Build it from the bar at the top.
- Most projects publish no ports. The router reaches them over the network by name, which is why a project with no port mappings still answers in the browser.
- A restart count climbing on its own is a crash loop. Look for the reason on the Logs tab.
