# Project settings

Settings this app applies to the project's container. They are kept in `.stackvo/site.json`, so they travel with the repository when a colleague clones it.

## Environment variables

Given to the container. They are not written into your application's `.env` — that file belongs to the framework.

| Control | What it does |
| --- | --- |
| Name / Value | One variable. |
| Add variable | Adds a row. |
| Remove | Deletes the row. |
| Save | Writes `.stackvo/site.json`. |

Changes take effect when the container is recreated.

## Show a directory listing

Serves a browsable list where there is no index file. Useful for a downloads folder or a build output.

This is a web server directive. Apache and Swoole have no configuration file for it, so on such a project you get the reason instead of a switch.

## Forward my SSH agent

Lets `composer install` and `git pull` reach private repositories from inside the container. No key is copied into the image.

The cost: anything running in that container can sign with your keys for as long as it is up. If no SSH agent is running on this machine there is nothing to forward, and the switch stays off.

## Host directories

Makes a directory beside the project visible inside the container. The use: a shared library checked out next to the project, which the application loads by a relative path such as `../cbglib`. The container only sees `/var/www/html`, so without a mount that path does not exist in there.

| Control | What it does |
| --- | --- |
| Source | The directory on this machine — relative to the project (`../cbglib`) or absolute. |
| Path in the container | Where it appears, for example `/var/www/cbglib`. |
| Read-only | On by default. The container can read the directory but not change it. |
| Save and approve | Writes `.stackvo/site.json` and approves every row on this machine. |

What can be mounted:

- The source must be an existing directory **inside your projects folder**, after symlinks are followed. It cannot be the projects folder itself or this project.
- The target must be an absolute path. It cannot be `/var/www/html` or anything under it, and it cannot be a system directory such as `/etc`, `/usr` or `/var/log`.

Why there is an approval: `.stackvo/site.json` travels with the repository, so a mount in it could have been written by anyone who can push. A mount is applied only after it has been saved from this pane on this machine. The approval is kept in `site-mounts.json` in the StackVo folder, where no repository can write. A mount that came with a clone is listed with a warning and is not applied. Changing its source, target or mode by hand later also needs a new approval.

Changes take effect when the container is recreated. The Bash CLI (`stackvo up`) does not apply these mounts.
