# Local API

A read-only HTTP surface that answers questions about this workspace.

## Controls

| Control | What it does |
| --- | --- |
| Start | Opens the listener and issues a token. |
| Stop | Closes it. |
| Try it | Shows an example request. |

## What it does and does not do

It serves the read-only half of the tool table the MCP server uses. It listens on `127.0.0.1` and nowhere else.

Nothing here writes, runs a command, or shows a password.

## The token

The token is shown once and never written to disk. If you lose it, stop and start again for a new one.

## Worth knowing

- It is off until you start it. A listener nobody knows about is a listener nobody turns off.
- Anything on this machine that has the token can use the API.
