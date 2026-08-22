# Services this project needs

Which database, cache or queue the project needs, and whether it is switched on here.

## Two lists, two different things

| List | Where it comes from | What it means |
| --- | --- | --- |
| Declared | `stackvo.json` | Somebody wrote it down and committed it. Everyone who clones the repository sees the same thing. |
| Suggested | The project's own `.env` | A guess this app made, read from keys like `DB_CONNECTION=pgsql`. Each row says which key it came from. |

A guess is never written on its own. Writing it is a separate button, because the moment you do it goes into a file your colleagues will read as a decision.

## States

| State | What it means |
| --- | --- |
| On here | The service is running. |
| Not on here | The project wants it and this machine does not have it. |
| No template in this version | The name is not recognised. It is not removed from the file, it is just not acted on. |

## Controls

| Control | What it does |
| --- | --- |
| Enable N services | Writes `.env`, regenerates the compose files and starts the services. |
| Write to stackvo.json | Adds the suggestions you ticked to the project's manifest as declarations. This is a change you commit. |

## Worth knowing

- Enabling a service changes this machine. Declaring it changes the repository. They are separate decisions.
- Check before you write: a guess can also come from a stale key in `.env`.
