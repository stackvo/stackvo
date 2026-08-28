# Audit trail

A record of the things this app did to your machine that cannot be taken back.

Not the application log. That one answers "why did it fail?" and deletes itself after seven days. This one answers "what was done, and when?" and is never rotated — which is what lets it still answer three weeks later.

## What is in it

Only acts that change something outside this app and that pressing the button again will not undo:

| Kind | Example |
| --- | --- |
| Elevated writes | A line added to the hosts file, a resolver file written |
| System stores | Adding or removing the local certificate authority's trust |
| Destruction | Deleting a project, restoring a database over what was there |
| Configuration | Writing `.env`, which reconfigures every container in the stack |
| Credentials | Moving a password into the OS keystore, or taking it back out |
| Other applications' files | Registering the MCP server with an assistant, writing an IDE debug configuration, adding StackVo to your shell's `PATH` |

## What is deliberately not

Starting or stopping a container is not here, and neither is reading anything. An act that the same button undoes is not something anybody needs an unrotated record of, and a trail that logged everything would be a trail nobody reads.

## Worth knowing

- Each line says when, what act, what it was done to, and how it ended — succeeded, refused before it was tried, or attempted and failed. A cancelled password prompt is recorded: somebody may need to know the machine was asked.
- The newest entry is at the top.
- The list is capped. If there are more entries than shown, the card says how many.
- Values are never recorded — only which key or which service. A trail carrying passwords is one you could not hand to anybody.
- The file is JSON Lines and lives beside the application log, so you can read it with any text editor or `grep` it without a parser.
- If a line was damaged — a process killed mid-write — it is skipped and the card tells you how many. The rest of the record stands.
