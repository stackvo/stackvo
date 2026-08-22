# This machine only

The `stackvo.local.json` file. Values here override `stackvo.json` for this checkout only.

When it is useful: a PHP version you are testing against, or a domain that clashes with something else on this machine.

## Controls

| Control | What it does |
| --- | --- |
| Editor | The file's contents. It is a fragment, not a whole manifest — write only the keys you want to change. |
| Save | Writes the file. |
| Remove | Deletes it; the project goes back to `stackvo.json`. |

## What the card tells you

- **In force** — which fields are currently being read from this file, listed one by one.
- **Ignored** — keys that describe the repository rather than this machine. They are only ever read from `stackvo.json`.
- **git status** — whether the file is being kept out of commits. If git would commit it you get a warning: add `stackvo.local.json` to `.gitignore`, or these settings become the whole team's settings.

## Worth knowing

- If the project is not under git, the git line says nothing. That is not a warning; there is no clone for it to leak into.
- This file is not meant to be committed. A setting the team needs goes in the Manifest card.
