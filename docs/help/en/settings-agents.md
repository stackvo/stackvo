# AI assistants

Registers the StackVo MCP server with the assistants on this machine.

## Controls

| Control | What it does |
| --- | --- |
| Install | Writes the `stackvo` entry into that client's configuration file. |
| Remove | Deletes only that entry. |
| Allow writes | Lets the assistant change the stack, not just read it. |

## What it changes

An assistant with this server can answer "why does shop.loc not open?" from the preflight report, the hosts file, the certificate and the container state. It looks instead of guessing.

## Worth knowing

- The write is file-level: the app reads the file, inserts one key and writes it back. Your other servers and any key it does not know survive.
- A `.stackvo-backup` copy is left beside the file first.
- Allowing writes lets an assistant stop and change your stack. Without it, the assistant can only read.
