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

## AI rules

Registering the server makes the tools reachable. It does not make them used — an assistant that has never seen this stack reads the source, guesses at nginx and suggests editing a generated file, because nothing told it that one tool call answers the question.

**AI rules** writes a short section into the instructions file the assistant already reads: `CLAUDE.md`, `AGENTS.md` (Codex and Zed), `.cursor/rules/stackvo.mdc`, `.github/instructions/stackvo.instructions.md` (VS Code and Copilot), `.windsurf/rules/stackvo.md`, or `GEMINI.md`.

| Control | What it does |
| --- | --- |
| Write workspace rules into | Which project the rules go into. The workspace root is for an assistant opened on the whole stack. |
| Write rules | Adds StackVo's block to that file, creating it if it is not there. |
| Update | Replaces a block an older version of the app wrote. |
| Remove | Takes the block out. The rest of the file stays. |

**In the project** travels with the repository, so a colleague who clones it gets the same guidance. **On this machine** applies to every session of that assistant, including projects that have nothing to do with StackVo — only some assistants read a global file, so only those are listed there.

### What it is safe to press

Only the region between `<!-- stackvo:rules:begin -->` and `<!-- stackvo:rules:end -->` is ever written. Everything else in the file comes back exactly as it was, a file with no markers is appended to rather than replaced, and a `.stackvo-backup` copy is left beside it first. The front matter Cursor and VS Code need is written when the file is created and never again, so narrowing it later sticks.

### What the rules say

Which tool answers which question; that everything under the generated directory is overwritten and the input is what to change; that driving Docker by hand takes a name and a port the next generate expects to own; and that a writing tool can stop the whole stack, so take a snapshot before a migration and ask before calling one.
