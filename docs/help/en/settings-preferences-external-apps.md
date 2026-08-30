# External applications

Which application opens a terminal, an editor or a browser.

## Controls

| Control                 | What it does                                                        |
| ----------------------- | ------------------------------------------------------------------- |
| Terminal                | The application the "Open a terminal" buttons use.                  |
| Editor                  | The application the "Open in editor" buttons use.                   |
| Browser                 | The browser addresses open in.                                      |
| Database client command | Used by the "Other…" entry in the open-in-client menu on a service. |

## Worth knowing

- Only applications installed on this machine are listed. One that is not installed is shown disabled.
- Every list ends with **Other…**, for an application that is not listed. Choosing it opens a box for the command that starts it, and the box is the only thing that runs — detection stays the default and is never replaced.
- The command is a launcher and its flags, nothing more. What is being opened — the folder, the address, the connection string — is added as the last argument.
- **This is not a shell.** `$HOME`, `&&`, a pipe and a redirect are all literal text. Quote a path that contains spaces; a backslash is a backslash, so `"C:\Program Files\Sublime Text\subl.exe"` works as written.
- A terminal needs its own flag for the command it should run, because every terminal takes one differently. Put it in the box: `alacritty -e sh -c`, `wezterm start --`, `wt.exe cmd.exe /K`.
- **Other…** is never chosen for you. If the application you named is missing or the box is empty, the buttons fall back to a detected application, the same way they do when a chosen one has been uninstalled.
- These affect what this app opens. They do not change your operating system's defaults.
