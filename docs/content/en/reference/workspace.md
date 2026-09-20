# Workspace and files

Everything StackVo writes is in a known place. This page is the list.

## The workspace

One folder, `~/.stackvo` by default. **The folder is the state**: there is no
database.

```text
~/.stackvo/
├── .env                    the stack's settings — written only when a setting is changed
├── generated/              compose files, Dockerfiles, router config — rendered, never edited
├── certs/                  the certificate and its authority
├── commands.json           machine-wide commands, the same shape as a manifest's
└── projects/
    └── shop/               one folder per project
        ├── stackvo.json    the manifest
        ├── .stackvo/       site.json, and what the app writes on request
        └── Dockerfile      rendered
```

`generated/` can be deleted at any time and is rebuilt on demand. A
workspace with no `.env` at all is the normal state, not an unconfigured one:
defaults live in the app and the file appears when you change something.

## Where the rest is kept

| | macOS | Windows | Linux |
| --- | --- | --- | --- |
| Preferences, help cache, the CLI links | `~/Library/Application Support/StackVo/` | `%APPDATA%\StackVo\` | `~/.config/stackvo/` |
| Application log, rotated daily | `~/Library/Logs/StackVo/` | `%LOCALAPPDATA%\StackVo\logs\` | `~/.local/state/stackvo/logs/` |
| Stack state | `~/.stackvo/` | `~/.stackvo/` | `~/.stackvo/` |

`~/.stackvo` is yours to move and safe to delete; nothing the app needs in
order to start is kept there. The CLI links live beside the preferences on
purpose: a PATH entry that vanished when you reset your stack would point at
nothing.

## What the app writes on its own

| File | What for |
| --- | --- |
| `preferences.json` | App settings. |
| `stats-history.json` | CPU and memory readings per container, so a chart is not empty after a restart. |
| `audit.jsonl` | One line per privileged or irreversible act: hosts writes, certificate trust, project deletion, `.env` keys changed, database restores. **Settings → Audit trail** reads it. |
| `crash-<time>-<pid>.txt` | A crash report, shown to you once. |
| `help-cache/` | The help documents fetched from the repository, so the **?** panels work offline. |

## What it writes into your project, only when you ask

| File | Button |
| --- | --- |
| `.stackvo/site.json` | Project settings |
| `.stackvo/context.json` | What an assistant working in the container should know |
| `.devcontainer/` | **Project → Devcontainer → Write files into the project** |
| `stackvo.preset.json` | **Settings → Workspace → Export this stack** |
| `CLAUDE.md`, `AGENTS.md`, `.cursor/rules/…` | **Settings → AI assistants** — only the block between StackVo's markers |

## Environment variables

| Variable | What it does |
| --- | --- |
| `STACKVO_ROOT` | Moves the workspace. |
| `STACKVO_LOG` | Log level, e.g. `stackvo_desktop=debug`. |
| `STACKVO_POLICY_FILE` | Points at a different policy file, so you can test one without root. |
| `DOCKER_HOST` | As usual; the scheme is stripped where needed. |

## Managed machines

An administrator can ship a policy file that sets and locks settings:

| | Path |
| --- | --- |
| macOS | `/Library/Managed Preferences/com.stackvo.desktop.json` |
| Windows | `%ProgramData%\StackVo\policy.json` |
| Linux | `/etc/stackvo/policy.json` |

```json
{
  "schemaVersion": 1,
  "settings": { "DEFAULT_TLD_SUFFIX": "corp.test", "SERVER_TYPE": "nginx" },
  "locked": ["DEFAULT_TLD_SUFFIX"],
  "registryPrefix": "registry.corp.example/proxy"
}
```

`settings` overrides the shipped default and the workspace's `.env`;
`locked` refuses writes to those keys from Settings; `registryPrefix` is put
in front of every generated image reference. It is not a security boundary,
and the app says so: it tells a co-operating application what the
organisation intends. **Settings → Doctor → Is the policy actually holding?** measures whether any of it is
actually holding on this machine.

## Credentials out of `.env`

**Settings → Where credentials are kept** moves a password or token into the
machine's keystore and leaves a reference behind:

```sh
SERVICE_MYSQL_ROOT_PASSWORD=keychain:SERVICE_MYSQL_ROOT_PASSWORD@a1b2c3d4
```

That takes it out of the file that gets backed up and pasted into support
threads. It does not take it off the disk: the real value is still rendered
into the generated compose file, because that is where Compose reads it.
