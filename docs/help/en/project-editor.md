# Editor in the container

Opens VS Code on the container this project is running in. The language server, the extensions, the terminal, `composer` and `artisan` all run inside the image — nothing on this machine has to have PHP on it.

This is the other half of the Xdebug card. That one wires an editor on your machine to a debugger in the container; this one puts the editor in there.

## How it opens

VS Code has no "attach to this container" command line. It opens a running container through an address, and StackVo builds that address from three things it already knows: the container's name, the directory your source is mounted at, and the fact that it *is* mounted.

The address is on the card, whether or not the button can be pressed. If VS Code is on this machine but its `code` command is not, the button still works — the application's own URL handler is used instead. If VS Code is not here at all, paste the address into VS Code's Open Folder dialog on the machine that has it.

Nothing is written down. The address is rebuilt every time the card is read, so a container that was recreated or a project that was renamed cannot leave a stale one behind.

## When it refuses

| What the card says | Why |
| --- | --- |
| The container is not running | There is nothing to attach to. Start the project. |
| The container carries a copy of the source | The editor would work perfectly and lose everything. See below. |

The second one is the reason this card refuses rather than warns. A PHP project bind-mounts your repository into the container, so an editor in there edits your files. A Node or Go project's image is built with `COPY . .`, so the container holds a **copy** taken when the image was built. An editor opened onto that copy shows the files, saves without complaint, and every line is thrown away by the next rebuild — with nothing on screen having said so.

A Node project can be talked out of it: turn the dev server on in the Runtime tab and bring the project up again, and the source is mounted. The other runtimes cannot — they have no equivalent, so for them this is the end of the answer.

The card reads the *container's own* mount table, not the manifest. Turning the dev server on writes a file that does nothing until the container is recreated, and when the two disagree the container is the one that is right.

## Worth knowing

- **Alpine images.** A Node project runs on Alpine, and VS Code publishes a server build for it. This is a note, not a problem. JetBrains does not publish one, which is why PhpStorm is a separate question.
- **The download.** VS Code unpacks a server of about a hundred megabytes inside the container. StackVo keeps it in a named volume so a rebuild does not throw it away. A container created before that volume existed says so on the card and offers to recreate itself.
- **git.** If the toolchain has no git in it, the editor opens onto a working copy whose history it cannot read. Editing still works.

## PhpStorm

PhpStorm cannot attach to a container that is already running — JetBrains has no such connection type. What it has is Dev Containers, and that is usually a *second* container: built from an image or a Dockerfile, with its own copy of the source beside the one you are running.

It does not have to be. A dev container can also be described by a compose file and a service, and when those are **StackVo's own** compose files and this project's service, the container it opens is the one already running. That is the file this card writes.

Press **Write the file** and point PhpStorm at the path shown: *Remote Development → Dev Containers → From Local Project → Specify Path*.

Three things in that file are deliberate, and each of them turns off a default that would be wrong here:

| Setting | Why |
| --- | --- |
| `shutdownAction: none` | The default stops the whole compose project when you close the IDE. |
| `overrideCommand: false` | The default replaces the service's command — in a PHP project that command is what serves your site. |
| `runServices` | Unspecified means every service in every file. StackVo already started the ones this project needs. |

### What it costs, and where it stops

- **Attaching recreates this project's container.** That is JetBrains' own behaviour — its plugin says so in its own settings — not something StackVo chose. Your site comes back with it.
- **The file is not for committing.** It names absolute paths on this machine, so it lives under StackVo's own directory rather than in your repository. The devcontainer on the Release tab is the one meant to be committed, and it answers a different question: how somebody *without* StackVo runs this project.
- **Alpine images have no JetBrains backend.** VS Code publishes a server built for musl; JetBrains does not. A Node project on `node:*-alpine` can be opened by VS Code and not by PhpStorm, and the card says so rather than letting you find out from a failed connection.
