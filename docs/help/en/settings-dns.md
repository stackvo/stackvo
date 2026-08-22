# Local DNS

A responder that answers this workspace's names without editing the hosts file.

It answers for one suffix and refuses everything else. It never forwards, and it has no upstream and no cache. It is not this machine's resolver — only the resolver for the names StackVo generates.

## Controls

| Control | What it does |
| --- | --- |
| Turn the responder on | Starts listening on `127.0.0.1` on the given port. |
| Point the system resolver at it | Makes this machine ask the responder for that suffix. Asks for your password. |
| Test | Asks the responder and the machine separately, and shows all four answers. |

## How it differs from the hosts file

This is what makes wildcards work. A hosts file cannot hold one, so an address like `*.shop.loc` only resolves here.

## The card's warnings

| Warning | What it means |
| --- | --- |
| UDP only | Something else holds the TCP port. Most queries work; a retry over TCP does not. |
| Broken | The machine asks that port and nothing answers there. Turn the responder on, or turn the switch off. |
| Stale | Configuration left from a suffix this workspace no longer uses. Applying again clears it. |

## Worth knowing

- The test asks two different questions: does the responder answer its own probe, and does this machine actually resolve the name. The first can pass while the second fails, and which one failed tells you what to fix.
