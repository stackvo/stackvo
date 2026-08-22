# Startup and shutdown

What happens when the application opens and closes.

## Controls

| Control | What it does |
| --- | --- |
| Start with the machine | StackVo runs on login. |
| When the window is closed | Minimise to the tray instead of quitting, or quit outright. |
| Stop containers on quit | Takes the stack down as the application closes. |

## Worth knowing

- Closing the app does not stop containers on its own. Docker keeps running them unless this setting is on.
- Minimising to the tray keeps the app running, so scheduled backups and idle suspension keep working.
