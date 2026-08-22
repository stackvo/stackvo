# Containers

Brings the whole stack up or takes it down. This affects everything in the workspace, not one project.

## Controls

| Control | What it does |
| --- | --- |
| Bring up | Regenerates the configuration and creates the containers. |
| Take down | Stops and removes the stack's containers. |

## Worth knowing

- Bringing up works at the compose level: the files are regenerated and the containers recreated. For one project, do it from that project's page.
- Taking down does not delete your data. Database volumes stay where they are.
- The output appears in the operation console at the bottom.
