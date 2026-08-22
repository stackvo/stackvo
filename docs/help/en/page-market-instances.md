# Service instances

The versions this workspace runs. Each has its own data and its own port.

## Controls

| Control | What it does |
| --- | --- |
| Stop / Start | Takes the instance's container down or brings it up. |
| Restart | Stops and starts the same container. |
| Open in browser | Opens the address, for services with a management interface. |
| Make primary | Sets the default instance for that service type. Projects that name no instance use the primary. |
| Settings | Opens the instance's port, credentials and other values. |
| Details | Shows the instance's details. |
| Remove | Deletes the instance. |

## When creating an instance

The card shows the package's own defaults. The part worth changing now is the credentials: an image reads a root password only when it first initialises an empty data directory, so this is the only moment it can be set.

If no free port can be found, the card says so and you pick one yourself.

## Worth knowing

- Two versions of one service can run side by side. Each keeps its own data.
- An instance whose package was uninstalled shows as "package missing". It keeps running but cannot be recreated.
- Removing an instance removes its data. Take a snapshot first if you want to keep it.
