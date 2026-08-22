# Hosts file

Every domain in this workspace resolves by name, so each one needs a line in `/etc/hosts`.

## Controls

| Control | What it does |
| --- | --- |
| Fix all | Adds the missing lines and removes the ones no longer needed. Asks for your password. |

## What the card tells you

| State | What it means |
| --- | --- |
| All resolving | Nothing to do. |
| Missing | Those addresses will not open in a browser. |
| Added by hand | StackVo did not write that line. It is left alone. |
| No longer needed | Lines StackVo wrote that this workspace no longer uses. The same button removes them. |

## Worth knowing

- StackVo only changes lines between its own block markers. The rest of the file is untouched.
- Wildcards cannot go in a hosts file. If you need a wildcard subdomain, use the Local DNS card.
- Changing the file needs administrator rights. Nothing is written until you approve.
