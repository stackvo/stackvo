# AI assistants

Registers the StackVo MCP server with the assistants on this machine.

## Controls

| Control | What it does |
| --- | --- |
| Install | Writes the `stackvo` entry into that client's configuration file. |
| Remove | Deletes only that entry. |
| Allow writes | Lets the assistant change the stack, not just read it. |
| Only these projects | Bounds the registration to the projects you name. |
| Writing lasts | Ends the writing half that long after each start of the server. |

## What it changes

An assistant with this server can answer "why does shop.loc not open?" from the preflight report, the hosts file, the certificate and the container state. It looks instead of guessing.

## Worth knowing

- The write is file-level: the app reads the file, inserts one key and writes it back. Your other servers and any key it does not know survive.
- A `.stackvo-backup` copy is left beside the file first.
- Allowing writes lets an assistant stop and change your stack. Without it, the assistant can only read.
- **Naming a project is the setting that makes the switch safe to use.** With no project named, allowing writes hands over all twelve writing tools, `stack_down` among them — that is one call away from every container on this machine being stopped. Name a project and the twelve become the four a project can bound: `xdebug_set`, `project_start`, `project_stop`, `project_restart`. The other eight are not offered at all, because no project scope can make "stop everything" mean less than it says.
- The scope bounds reading too, and this far: no tool that names a project answers for one outside the scope, so another project's manifest, request traces, profile and log files are closed to it, and the project lists it sees hold only what it is scoped to. It is **not** information isolation — the machine-wide answers still work, because they are about the machine rather than about one project: the doctor, the hosts table, the mail catcher, a database service's query log, a container's log by id. Bounding those would leave the assistant unable to diagnose the project you did give it.
- **Writing lasts** ends the writing tools by itself. Reads keep working; a client that asks for the tool list again after that point is told the truth, and one that does not is refused by name when it calls. Restart the server — closing and reopening the assistant does it — to grant them again.
- The flags shown under the controls are exactly what gets written into the file. That line is the record of what this assistant was allowed to do, and it is what you will read six months from now when somebody asks.

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
