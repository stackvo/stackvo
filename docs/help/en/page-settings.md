# Settings

Every setting for the application and the workspace. The list on the right splits them by subject.

## Sections

| Group | What is in it |
| --- | --- |
| Application | Appearance, localisation, preferences, AI assistants, local API. |
| Workspace | Directory and control, domain and network, certificates, credentials. |
| Stack | Web servers, catalogue, project defaults. |
| Help | Doctor, application log, about. |

## Where a setting is stored

| Where | What | When it takes effect |
| --- | --- | --- |
| `preferences.json` | The app's own preferences: editor, theme, language. | Immediately. |
| The workspace's `.env` | Anything about the stack: domain suffix, versions, server settings. | Most need a regenerate. |
| A project's `stackvo.json` | Anything about one project. | From the project page. |

## Worth knowing

- After changing something that writes to `.env` you usually have to regenerate. The card says so.
- An administrator can lock some settings. A locked setting is disabled and names the file that locked it.
