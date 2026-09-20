# Keyboard shortcuts

The window is meant to be used from the keyboard as much as from the mouse.
The list is short because most of the app is a page with buttons, and a
button has a name you can type.

## The command palette

++cmd+k++ on a Mac, ++ctrl+k++ elsewhere. The button in the top bar shows
the keys for your machine. The palette opens over any page and takes:

| Type | Gets you |
| --- | --- |
| A page name | That page: Projects, Catalogue, Logs, Dumps, Mail, Settings |
| A project name | The project, and the verbs beside it: start, stop, restart, build, open in the browser |
| A stack action | Start, stop, restart or rebuild everything |
| A setting's name | That card on the settings page |

| Key | In the palette |
| --- | --- |
| ++up++ ++down++ | Move through the matches |
| ++enter++ | Run the highlighted one |
| ++esc++ | Close |

## Everywhere

| Key | Does |
| --- | --- |
| ++esc++ | Closes the open side panel — a help card, a settings sheet, a dialog |
| ++cmd+w++ / ++alt+f4++ | Closes the window; the containers keep running, and the tray keeps the app |

## In the terminal and the console

The **Terminal** tab is a real terminal: ++ctrl+c++, ++ctrl+d++, ++tab++
completion, history with ++up++ all work as in any other. The console's
`stackvo` command has shell completion for bash, zsh, fish and PowerShell —
see [the command line](cli.md#shell-completion).

## Not a shortcut, but faster

- The tray menu starts and stops the stack without the window.
- `stackvo <verb> <project>` does anything the window does, from any
  terminal.
