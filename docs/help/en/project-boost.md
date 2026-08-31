# The MCP server inside this container

Laravel Boost gives an assistant the questions only your application can answer: what is in the `users` table, which routes exist, what this version of the framework documents, and a `tinker` it can actually run. It installs a small MCP server for that.

Then it registers it — and the line it writes is `php artisan boost:mcp`.

## Why that line cannot work here

It assumes a `php` on your machine. StackVo does not put one there, and that is a decision rather than an omission: **a PHP version is a property of a project, not of a directory a shim guessed.** Your project's PHP, its extensions and its `php.ini` are inside its image, where they are the same on every machine that opens the repository.

So Laravel's own installer produces a configuration that cannot start here. Nothing warns you. The assistant reports a server that will not start, and the reason is in a log you never see.

## What this card writes instead

The passage into the container, which this application already owns:

```
docker exec -i stackvo-<project> php artisan boost:mcp
```

`docker exec` rather than `stackvo artisan` for two reasons. The CLI works out which project it means from the directory it was started in, and an assistant starts its servers from wherever it happens to be — naming the container is the passage that cannot pick the wrong project. And `docker` is already a hard requirement of this application, while the `stackvo` binary is not necessarily on your `PATH`.

`-i` and no `-t`: MCP over stdio is a pipe, and a TTY would put line discipline in the middle of a JSON-RPC stream.

## Two servers, side by side

This does not replace StackVo's own MCP server, and neither replaces the other:

| Server | The question it answers |
| --- | --- |
| **StackVo** (Settings → Assistants) | *"Why will `shop.loc` not open?"* — preflight, hosts, certificate SANs, container logs |
| **Boost** (this card) | *"What is in the `users` table?"* — schema, route list, tinker, the `artisan` inventory, documentation for the version you actually have |

The first is registered once for the machine. This one is registered **per project**, in files that live in the project directory — because a server that only exists while this project's container is up does not belong in a file that applies to every directory on your disk.

## What is read, and what is never guessed

| Fact | Where it comes from |
| --- | --- |
| Whether `laravel/boost`, `laravel/mcp` or `laravel/ai` is installed | `composer.lock` — the same file the dependency card reads, so the two cannot disagree |
| Which servers this project publishes | your own `routes/ai.php` — the `Mcp::local()` and `Mcp::web()` lines in it |
| What is registered today | `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json` in this project |

A registration whose first argument is a constant or a variable is **skipped**, not guessed at. There is no string to read, and a handle invented here would be `artisan mcp:start something` failing in your assistant.

## `Mcp::web()` needs nothing at all

A `Mcp::web()` registration is an **ordinary route inside your application**. It is already served on this project's own domain, over the certificate your browser already trusts. There is no process to start, no certificate to extend, no hosts entry to write and no second router — so that row shows you the URL and offers no button.

## The rules the write follows

The same three the assistant registration in Settings follows, on the same kind of file:

* **Read, replace one entry, write back.** Everything else in the file survives, including keys this code has never heard of.
* **A file that does not parse is not edited.** A `mcp.json` with comments in it, or one you are halfway through editing, is reported and left exactly as it is.
* **The old contents are kept**, beside the file as `.stackvo-backup`.

And one more that belongs to this card: an entry that already runs this server **keeps the name it had**. Renaming it would leave two servers in your client instead of one working one.
