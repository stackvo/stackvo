# Fetch and send data

Named places this project's data really lives — a staging site, production — and how to fetch a copy of the database from there, or send this one back.

A recipe is written in `stackvo.json`, so it travels with the repository and a teammate gets it on clone.

```json
"providers": {
  "staging": {
    "about": "the staging site",
    "image": "ghcr.io/example/dbtools:1",
    "pull": ["fetch-dump", "--out", "dump.sql"],
    "env": { "REMOTE_HOST": "staging.example.com" },
    "secrets": ["SSH_KEY"]
  }
}
```

## Starting points

A project that declares no recipe is offered the three this build ships, on the card itself:

| Recipe | What it reaches | Both directions? |
| --- | --- | --- |
| `mysql-remote` | A MySQL or MariaDB server this machine can reach directly | Yes |
| `postgres-remote` | A PostgreSQL server this machine can reach directly | Yes |
| `upsun` | An Upsun (Platform.sh) environment, through its own CLI | Fetch only — its CLI's `db:sql` takes a query, not a file |

**Adding one approves nothing.** It is written into your `stackvo.json` and then goes through the same approval as a recipe you typed. Every one of them carries a placeholder host, database or project id, so the version you approve is by definition not the version that was added — and the card lists the words you have to replace.

Two recipes people ask for are not here, and the reasons are worth knowing before you go looking:

- **SSH plus `mysqldump`** — a recipe's container gets no path from your machine, so it has no agent socket and no key file, and `ssh` authenticates with neither an environment variable nor a command-line word. The missing shell is not the obstacle; `mysqldump --result-file` needs no pipe.
- **Pantheon** — there is no Terminus image to run, and `terminus backup:get` answers with a URL that then has to be downloaded, which is a second command and therefore a shell.

## The rules, and why each one is there

**A command is a list of words, not a command line.** There is no shell, so no pipes, no redirection and no `$VARIABLE`. Write `["pg_dump", "-Fc"]`, not `"pg_dump -Fc | gzip"`.

**It runs in a container, never on your machine.** A recipe comes from a repository, and a repository is something you clone.

**Passwords and keys are named, never written.** List them under `secrets` and fill in the values on the card. They go into your operating system's keystore, and reach the container as environment variables for the length of one run.

**A pull writes `/stackvo/dump.sql`; a push reads it.** That path is fixed. StackVo mounts a scratch directory of its own there and removes it afterwards.

## Controls

| Control | What it does |
| --- | --- |
| Database | Which of your instances a fetch lands in, or a send reads from. |
| Approve fetching / Approve sending | Agrees to that exact command. Separately for each direction. |
| Fetch now / Send now | Runs it. |
| Copy what this replaces first | Takes a snapshot of the local database before a fetch overwrites it. On by default. |
| Withdraw approval | Asks again next time. |

## Worth knowing

- **Approving a fetch is not approving a send.** They are agreed to separately and nothing about one makes the other cheaper.
- **Editing the recipe asks again.** The approval covers the image, every word of the command, the environment and the names of the secrets. Rewording `about` does not, because it decides nothing.
- **A fetch is recoverable.** It ends in the same restore the Dumps card uses, which copies what it is about to replace.
- **A send is not.** It writes to somewhere that is not this machine. It is recorded in the audit log; a fetch is not, because a fetch only changes this machine.
- Nothing sends on a schedule, and there is no way to make it.
- An administrator can switch either direction off for a whole fleet. They cannot approve one on your behalf.
