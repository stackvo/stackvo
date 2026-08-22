# Export this stack

Writes which services are enabled, and their versions, to a small JSON file.

## Controls

| Control | What it does |
| --- | --- |
| Preset name | The name written into the file. |
| Save to a file | Writes the JSON file. |

The card shows the file's contents before you save it.

## Worth knowing

- There are no passwords in the file. The format has nowhere to put them.
- It is safe to commit. That is the point: a team running the same stack.
- The file carries "which services and which versions" and nothing else. Not your ports, not your data.
