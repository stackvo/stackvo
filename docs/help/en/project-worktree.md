# Worktrees

Gives a git branch an environment of its own: its own folder, its own address, its own database. Both branches run at the same time, and nothing git would notice is written into your checkout.

## Controls

| Control | What it does |
| --- | --- |
| New worktree | Opens the form. |
| Branch | Which branch gets the environment. A branch already checked out elsewhere cannot be picked. |
| Create branch | Opens a new branch if it does not exist. |
| Name | The new project's name. Left empty, it is derived from the branch. |
| Database | None, new and empty, or a copy of this workspace's. |
| Instance | Which database engine the copy lands in. |
| Create | Prepares the folder, the project and the database you chose. |
| Remove | Deletes the worktree. Deleting the branch and the database are separate switches, both off by default. |

## What the form shows

The name, the address and the database name are shown before you press anything. They come back from the backend, so what is on screen is what will be created.

If something cannot be done, the button stays disabled and the reason sits beside the field that caused it.

## When this project is a worktree

The card shows which project it is a branch of, its branch, its address and its database instead. A worktree of a worktree is not offered.

## Worth knowing

- Copying a database takes as long as the source is big.
- Removing deletes the folder. Commit anything you have not committed first.
