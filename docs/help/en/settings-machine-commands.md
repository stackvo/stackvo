# Machine-wide commands

Commands you can run in every project in this workspace.

A project declares its own in `stackvo.json`. This is the layer above: one file at the root of your workspace, adding commands to all of them, without editing a repository somebody else owns.

## The file

Create `commands.json` beside your projects — the path is shown at the top of the pane. It has the same shape a project's `commands` block has:

```json
{
  "commands": {
    "tail": {
      "exec": ["tail", "-f", "storage/logs/laravel.log"],
      "about": "Follow the application log"
    },
    "shell": { "exec": ["bash"], "interactive": true }
  }
}
```

Every command then appears in the quick-command menu of every project, marked with the file it came from.

## Worth knowing

- **`exec` is a list, not a line.** The words are passed to the container one by one; there is no shell, so a pipe, a redirect and `&&` are all literal text. Two commands in one line is two commands.
- **It runs in the project's container and nowhere else.** There is no `host` form. A step that has to run on this machine is a _hook_, which the project declares and which is approved against a digest before it runs.
- **`interactive: true`** opens the terminal you chose in Preferences instead of the in-app console. Use it for anything that asks a question — a REPL, a shell.
- **A project's own command wins** if it uses the same id. Its file is committed and shared; this one is yours, and the pane tells you which file each row came from.
- **An id that is already built in is refused**, and the pane says which. `migrate` means `php artisan migrate` everywhere, and a button whose label does not describe what it runs is worse than one that is missing.
- **The file is the interface.** There is no form here, on purpose: a second way to write the same JSON would disagree with your editor the first time you used one.
