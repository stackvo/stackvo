# What an assistant is told about this project

Two files this app writes into the repository so an assistant working in it knows what it is working in.

## AI rules

A short section written into the instructions file the assistant already reads — `CLAUDE.md`, `AGENTS.md` (Codex and Zed), Cursor's `.cursor/rules/stackvo.mdc`, VS Code's `.github/instructions/stackvo.instructions.md`, `.windsurf/rules/stackvo.md` or `GEMINI.md`.

| Control | What it does |
| --- | --- |
| Write rules | Adds StackVo's block to that file, creating it if it is not there. |
| Update | Replaces a block an older version of the app wrote. |
| Remove | Takes the block out. The rest of the file stays. |

These are the same rules Settings → AI rules writes; here they are aimed at this project rather than at a name picked out of a dropdown. The rules that apply to *every* project on this machine, and registering the MCP server itself, are in Settings.

The rules say which tool answers which question, that everything under the generated directory is overwritten and the input is what to change, that driving Docker by hand takes a name and a port the next generate expects to own, and that a writing tool can stop the whole stack.

### What it is safe to press

Only the region between `<!-- stackvo:rules:begin -->` and `<!-- stackvo:rules:end -->` is ever written. Everything else in the file comes back exactly as it was, a file with no markers is appended to rather than replaced, and a `.stackvo-backup` copy is left beside it first.

## The context file

`.stackvo/context.json` is written for every project on every generate, and there is nothing to switch on: the domain, the runtime, the path inside the container, and the address of each running service as it is reachable *from inside the network*.

Names and addresses only. Passwords are in the project's own `.env` and are deliberately not repeated there — the file lands in a source tree, and a source tree is a thing that gets committed by accident.

For a PHP project the directory is bind-mounted, so the file is live. For a runtime built from source there is no mount, so it reaches the container at the next build.
