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
| Wanted for | Empty for a branch of your own; a duration makes it a sandbox with an expiry. |
| Create | Prepares the folder, the project and the database you chose. |
| Remove | Deletes the worktree. Deleting the branch and the database are separate switches, both off by default. |

## What the form shows

The name, the address and the database name are shown before you press anything. They come back from the backend, so what is on screen is what will be created.

If something cannot be done, the button stays disabled and the reason sits beside the field that caused it.

## When this project is a worktree

The card shows which project it is a branch of, its branch, its address and its database instead. A worktree of a worktree is not offered.

## Making one for an assistant

Leave **Wanted for** empty and you get a worktree in the ordinary sense: a branch of yours, yours until you remove it.

Choose a duration and you get a *sandbox* — an environment built for one task, by somebody who is not going to remember it exists. Three things then follow, and together they are what makes handing the branch to an AI assistant a different act from handing it your machine:

1. **Its own everything.** Directory, hostname, environment variables, and a copy of the database if you asked for one. Nothing it does reaches the project it was branched from.
2. **Its own database login**, on MySQL and MariaDB — see below. The copy is the worst it can reach.
3. **A registration that says so.** The card shows the exact flags to register the MCP server with: the assistant gets this branch and nothing else, for as long as the sandbox has. Under that scope the twelve writing tools become the four a project can bound, and stopping the whole stack is not one of them.

The work still comes out: a sandbox's output is the **branch**, and removing the environment does not delete it. The database is scaffolding.

Nothing is deleted on a timer, and nothing ever will be — an app that removed a directory by the clock would eventually remove one with a morning's uncommitted work in it. What the expiry does is let the list say the time has passed, so removing it stays one click and a decision.

## Whether the branch can reach the parent's data

"Its own database" and "cannot reach the other one" are two different promises, and only the first is free. A branch's database lives on the same engine as everything else, so what decides the second is **which login the branch is given**.

| What the card says | What it means |
| --- | --- |
| Database login — its own | The branch has an account granted on its own schema and nothing else. It cannot read, or even list, the project it was branched from. |
| Database login — shared with the instance | The branch uses the engine's own account, so it can reach every database on that instance, including the parent's. |

A login of its own is arranged on **MySQL and MariaDB**, where a grant on one schema also covers the tables the application creates later. PostgreSQL is not done: its grants have to be applied to the objects inside the database after the data is copied, plus default privileges for what comes later, and half of that produces a branch whose application cannot read its own tables. MongoDB publishes no database name to scope to. On those two the card says the login is shared, which is the truth rather than a gap left quiet.

It matters most when the thing working on the branch is not you. An assistant told to fix a failing test on a branch can run a migration, drop a table or truncate one; with a login of its own, the worst it can reach is the copy it was given.

## Worth knowing

- Copying a database takes as long as the source is big.
- Removing deletes the folder. Commit anything you have not committed first.
- Removing also drops the branch's own database account, whether or not you asked for the database itself to go: keeping the data does not mean keeping an account that can reach it.
- If the engine refuses to create the account — a database user that cannot `GRANT`, usually — the worktree is still created and the card says the login is shared. Nothing pretends an isolation it did not arrange.
