# Catalogue

Where services come from, and which versions are on this machine.

The Services page shows what is running; this shows what could be.

## The two halves

| Half | What for |
| --- | --- |
| Available | The packages a source publishes, and their versions. This is where you **install**: files land on disk. |
| Service instances | The versions this workspace runs. This is where you **add an instance**: this workspace starts running that version. |

They are different acts. Without the split, "I want to try MySQL 9.4 alongside 8.0" and "replace my database" would be the same button.

## Controls

| Control | What it does |
| --- | --- |
| Search the catalogue | Filters the packages. |
| Source | Where the catalogue is pulled from. The address is kept under Settings → Catalogue. |

## Worth knowing

- StackVo carries no services inside itself. Until a source is given, nothing is available.
- If a source is unsigned, the card says so.
- If this workspace still keeps services in `.env`, the page offers a migration. It does not move data: volumes are adopted, ports are kept, and the old container name lives on as a network alias.
