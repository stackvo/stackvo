# Extra directives

Lines added to every configuration generated for this server.

## Controls

| Control | What it does |
| --- | --- |
| Directive text | Lines in the server's own syntax. For example: `client_body_timeout 120s;` |

## Variables

`{{ VAR }}` is substituted from `.env`. It takes effect on the next generate.

## Worth knowing

- Comments and blank lines are dropped. A file holding only a note changes nothing.
- What you write is not validated. An invalid directive stops the server from starting; if that happens, look at the logs.
- It only works on servers configured through a file. See the "Where it applies" card above.
