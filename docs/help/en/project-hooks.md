# When this project starts and stops

Commands defined in `stackvo.json`. They run when the project starts, stops or is rebuilt.

## Why consent is asked for

A hook is written by whoever wrote the repository, and pressing Start runs it.

- Steps that run **in the container** need no consent. That container already runs this repository's code.
- Steps that run **on this machine** do. They run on your machine, with your permissions.

The commands are printed in full, one per row, with where each one runs beside it. There is no summary, because a summary would make approving easier than reading.

## Controls

| Control | What it does |
| --- | --- |
| Approve these commands | Approves the exact list on screen. |
| Revoke | Removes the approval. Steps that run on the machine stop running. |

## Worth knowing

- Consent is recorded against those exact commands. If the manifest changes you are asked again. The approval is a receipt for the list, not a vote of confidence in the project.
- An administrator can switch hooks off. The card says so; when host steps are disabled, container steps are unaffected.
