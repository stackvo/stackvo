# Suspend idle projects

Stops projects nobody is asking for.

## Controls

| Control | What it does |
| --- | --- |
| Idle minutes | Projects that receive no request for this long are stopped. `0` turns it off. |
| Suspend N now | Stops the projects that are already past the threshold. |

## How it is measured

From the proxy's access log. That is the only honest signal: php-fpm uses no CPU whether it is serving or asleep, so watching CPU would mislead.

A project the log has never mentioned is never suspended.

## Worth knowing

- A suspended project is simply stopped. Start it again from the list, the tray or ⌘K.
- There is no wake-on-request. A request to a stopped project does not start it.
