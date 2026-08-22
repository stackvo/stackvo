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
