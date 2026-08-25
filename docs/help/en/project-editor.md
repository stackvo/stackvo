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
