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

## The third layer: commands a package brought

There is one more source of rows in that menu, and it is not a file you write.

**A package can bring its own commands.** Installing the Redis package can also give you `redis-cli`, the way installing `ddev-redis` gives you `ddev redis-cli` — with one difference: every byte of it was verified before it was read. A signature vouches for the registry, the registry states the manifest's hash, the manifest states the hash of every file beside it, and all of that is re-checked on **every read** rather than only at install.

Those rows are different from the two above in a way the menu tells you about:

| | Runs in | Comes from |
| --- | --- | --- |
| Built-in and project rows | **your project's container** | this app, or the project's `stackvo.json` |
| Machine-wide rows | **your project's container** | your `commands.json` |
| **Package rows** | **that service instance's container** | the installed package |

A package row is tagged with the instance it runs in — `in redis-7-2` — because *"this does not touch your project"* is exactly the kind of thing to say before somebody presses a button.

The containment is the same shape as everywhere else here:

- **Only enabled instances.** A command against a container that is not meant to be running is a button whose failure is `No such container`.
- **The container name is derived, never declared.** A package can no more name somebody else's container than it can name a host port.
- **The id carries the instance** — `redis-7-2:redis-cli` — so two installed versions of one service are two rows rather than one collision, and a project cannot shadow a package's command by declaring the same string.
- **The description is the package author's**, in whatever language they wrote it, and the window marks it as such rather than claiming its own.
