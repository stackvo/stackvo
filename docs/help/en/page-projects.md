# Projects

Every project in this workspace and the state of its container. For the detailed controls of one project, press **Details** at the end of its row.

## The bar at the top

| Control | What it does |
| --- | --- |
| Running chip | How many projects are running, out of how many there are. |
| Plus | Opens the new project panel. |
| Refresh | Reads the list from the engine again. |
| Three dots | Opens the unmanaged code panel. |

## The table

| Column | What it shows |
| --- | --- |
| Favourite | A star. Favourites are gathered at the top of the list. |
| Domain | The project's address. If it does not resolve, the row says so and offers to add the hosts entry. |
| Runtime | PHP, Node or another runtime, and its version. |
| Repo | Git repository information. A repository with no remote is marked as such. |
| Server | The web server serving the project. |
| Configuration | Whether the manifest is valid and the generated files are current. |
| Status | Running, stopped or not built. |

If a project is a branch of another, the row says **branch of {project}**. Without it, one application on two branches looks like two unrelated entries.

## Row actions

| Action | What it does |
| --- | --- |
| Stop / Start | Takes the container down or brings it up. |
| Restart | Stops and starts the same container. |
| Rebuild | Regenerates the Dockerfile, builds the image, recreates the container. |
| Terminal | Opens a shell inside the container. |
| Open in browser | Opens the project's address. |
| Details | Goes to the project detail page. |
| Delete | Removes the container and the project's entry. Your folder on disk is untouched. |
| Three-dot menu | Offers whichever action makes sense now: build, start, stop, apply changes or add the hosts entry. |

The menu only shows the action that applies. On a project that has never been built it says Build, not Start.

## Search and filters

| Control | What it does |
| --- | --- |
| Search | Filters by project name and domain. |
| Status filter | All, running, stopped or not built. |
| Favourites only | Shows what you starred. |
| Clear filters | Removes every filter. |

## New project

The plus button opens a panel. There are three ways in:

- **Empty project** — a bare project from the form's values.
- **A framework template** — Laravel, WordPress, Symfony, Next.js and so on. The framework's own installer runs in a throwaway container and the result is adopted. Runtime, server and document root are read from what the installer actually wrote.
- **Clone from a git repository** — clones an existing repository and adopts it.

Left empty, the domain is derived from the project name.

## Unmanaged code

The panel behind the three-dot menu finds code on this machine that StackVo is not running.

| Source | What it does |
| --- | --- |
| Folders in your projects directory | Lists folders with no `stackvo.json` and turns one into a project with **Adopt**. What it is gets detected from the files in the folder. |
| XAMPP and Laragon sites | Reads those tools' installation directories and lists their sites. |
| Projects with a compose file | Derives a project from an existing `docker-compose.yml`. The `stackvo.json` that would be written, and where each value was read from, are shown first. |

## Worth knowing

- Importing never writes into the other tool's folder. The site is copied into this workspace. With **Move instead of copy**, the original is deleted once the copy is complete and the other tool stops serving that site.
- When nothing recognisable is found during adoption, defaults are used, and the panel says so row by row.
- Importing from compose lists separately any service that has no StackVo equivalent. You have to handle those yourself.
- Deleting a project does not delete your code. Only the container and the entry are removed.
