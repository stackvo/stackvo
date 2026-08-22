# Docker engine

The state of the engine running the containers.

## What the card shows

| Field | What it means |
| --- | --- |
| State | Running or not. |
| Platform | Docker Desktop, Colima, OrbStack or Docker Engine. |
| Socket | The socket or named pipe the app connects through. |
| Context | The Docker context in use. |
| Version | The engine's version. |
| API version | The API version being spoken. |

## Worth knowing

- With the engine down, the app can do nothing. The card offers a button to start it.
- With more than one Docker installation, the context says which one is being used. If you cannot see containers you expect, look here first.
