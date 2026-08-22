# Workbench

Write a snippet, run it inside this project with the application booted, and read what came back.

For one line at a time the terminal above is better. This is for the twenty lines you keep editing.

## Controls

| Control | What it does |
| --- | --- |
| Run it with | Which runner executes the snippet. |
| Snippet | The code. `⌘/Ctrl + Enter` runs it too. |
| Run | Sends the snippet and shows the output, the exit code and stderr. |
| History | The snippets you have run. Clicking one puts it back in the editor with its runner. |
| Forget | Clears the history. |

## Two kinds of runner

| Kind | What you get |
| --- | --- |
| Application booted | Your models, config and container. |
| Bare | The language on its own. No framework, no database connection. |

Every row says which it is. Picking the wrong one is how you spend ten minutes before noticing your models were never loaded.

## Worth knowing

- Print what you want to see: `dump()`, `echo`, `print`. The value of the last expression is not echoed.
- The exit code decides success, not whether stderr was empty. Plenty of languages write to stderr on a good run.
- A run is time-bounded. If it was stopped at the limit, the card says so.
- The project has to be running, and an empty snippet does not run.
