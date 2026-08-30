# Application log

StackVo's own diagnostic record. Not your projects' server logs — those are on a project's Logs tab.

## Controls

| Control | What it does |
| --- | --- |
| Open the folder | Reveals the log folder in Finder or Explorer. |
| Save a diagnostics bundle | Writes the log, the preflight checks, the doctor report and any crash reports into one archive. |
| Compare with another machine | Opens a bundle somebody sent and lists what is different about this one. |

## "It works on my machine"

That is the oldest complaint in this kind of tool, and the usual answer — *containers solve it* — is not true: the same compose file on two Docker versions is two different things.

Ask the other person for their bundle and open it here. What comes back is only what the two machines **disagree** about, with the rest counted rather than listed:

| Fact | This machine | Theirs |
| --- | --- | --- |
| `engine.version` | 27.1.1 | 25.0.3 |
| `service.redis-7-2` | 7.2 on | 7.2 off |
| `project.shop` | php 8.4 on nginx | php 8.3 on nginx |

A fact only one side states is shown with the other half marked **not stated** — "you have this service and they do not" is usually the most useful line on the page.

The comparison reads one file out of the bundle, `environment.json`: versions, the engine, the services and what each project declares. It holds **no paths** — a home directory differs on every machine and would report two identical setups as different in five places — and **no credentials or `.env` values**, which is the same reason the bundle is safe to send at all. It is compared against what this machine is *right now*, not against a copy of itself from earlier; the question is always what is different now.

"Nothing differs" is a result rather than an empty screen: it means whatever is going wrong is somewhere this cannot see, which is worth knowing before you spend an afternoon on versions.

You can also open the `environment.json` on its own, if that is what somebody pasted to you. A bundle made by a version of StackVo from before this existed says so by name rather than comparing nothing.

## Worth knowing

- Attach the bundle when reporting a problem. The log alone is usually not enough.
- Password and token values are redacted as the log is written.
- The bundle is plain text inside. Have a look before you send it.
- If no writable log location was found on this system, the card says so.
