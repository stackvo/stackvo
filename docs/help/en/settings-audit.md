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

## What an assistant did

Every writing call an assistant makes through the MCP server is recorded here — including the ones that were **refused**, which is often the more interesting line: an assistant that tried to stop the whole stack and was told it may not is exactly what you want to see when you are deciding what to grant next time.

This is the one place the bar above is widened, and for a reason. Starting a container from this window is not recorded because the person who pressed the button watched it happen. The same act asked for by an assistant is the one nobody saw.

## Putting one back

An assistant's act carries **what would put it back**, worked out before the call ran and stored on the line. That matters: what `stack_down` stopped exists only before it stopped it, so a plan worked out when you press the button would be worked out against a machine that has already changed.

| Act | What Undo does |
| --- | --- |
| Stopped the whole stack | Starts what was running before it — services first, then projects |
| Started or stopped a project or a service | The other one |
| Turned Xdebug on or off | Sets it back to what it was |

Most acts have no Undo, and the line says why in its own words rather than showing a button that would not keep its promise: a **restart** has already gone through the state an undo would return to; **generate** overwrote output that was not kept, so the repair is to change the input; **reissuing a certificate** replaced one that was not kept either; **taking a snapshot** added a file and changed nothing.

An undo is a sequence, not a transaction. If the fourth of six calls fails, the first three stay done and the trail says where it stopped. The record keeps both halves — that the act happened, and that somebody reversed it — because the file is only ever appended to; the Undo line names the line it put back rather than editing it.

## What is deliberately not

Starting or stopping a container from this window is not here, and neither is reading anything. An act that the same button undoes is not something anybody needs an unrotated record of, and a trail that logged everything would be a trail nobody reads.

## Worth knowing

- Each line says when, what act, what it was done to, and how it ended — succeeded, refused before it was tried, or attempted and failed. A cancelled password prompt is recorded: somebody may need to know the machine was asked.
- The newest entry is at the top.
- The list is capped. If there are more entries than shown, the card says how many.
- Values are never recorded — only which key or which service. A trail carrying passwords is one you could not hand to anybody.
- The file is JSON Lines and lives beside the application log, so you can read it with any text editor or `grep` it without a parser.
- If a line was damaged — a process killed mid-write — it is skipped and the card tells you how many. The rest of the record stands.
