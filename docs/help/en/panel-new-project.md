# New project

Creates a project in one of three ways: from a bare skeleton, from a framework's own installer, or from an existing git repository.

## The three ways

| Starting point | What it does | When |
| --- | --- | --- |
| Empty project | Creates a project from the form's values. No installer runs. | You will bring the code yourself, or you already have a skeleton. |
| Framework template | The framework's own installer runs in a throwaway container, then the result is adopted. | You are starting fresh with Laravel, WordPress, Next.js and so on. |
| Clone from git | Clones the repository and adopts what arrives. | The code already exists. |

## Empty project

You fill in every field yourself.

| Field | What for |
| --- | --- |
| Project name | Lower case; starts with a letter or digit, and may use hyphens, underscores and dots. For example `api.myapp`. |
| Domain | The address the project opens at. Left empty, it is derived from the name. |
| Aliases | Other names the project answers on. They are written into `stackvo.json`, so a colleague who clones gets them too. |
| Runtime | PHP, Node, or another runtime from the catalogue. |
| Version | The version of the runtime you picked. |

### If you pick PHP

| Field | What for |
| --- | --- |
| Web server | What serves the project. |
| Document root | The subfolder the web server publishes. `public` on Laravel, the project root on WordPress. |
| PHP extensions | The extensions compiled into the container. An extension that cannot be installed on the chosen PHP version is marked. |

### If you pick Node or another runtime

| Field | What for |
| --- | --- |
| Package manager | Enables Corepack in the image, which is what lets `packageManager` in `package.json` pin a version. Leaving it unpinned builds the image exactly as before. |
| Install command | The command that installs dependencies. |
| Build command | Optional. It can be left empty. |
| Start command | The command that runs the application. |
| Port | The port the application listens on inside the container. |

Your application has to bind to `0.0.0.0`. Traefik cannot reach a server listening only on `127.0.0.1`, and the address returns 502.

## Framework template

Templates are grouped by runtime: PHP, JavaScript, CMS and e-commerce, Python, Go, Ruby and Rust. The group heading is the runtime the choice implies — picking Nuxt is picking Node.

The process:

1. The framework's own installer runs in a throwaway container. So `composer create-project` or `npx create-next-app` really runs; StackVo does not copy a snapshot.
2. When the installer finishes, the result is adopted.
3. Runtime, web server and document root are detected from what the installer **actually wrote**. Laravel serves from `public/`, WordPress from the project root; that difference is read, not guessed.

This is why the runtime fields are hidden once a template is chosen: they have nothing to say, because the installer gives the answer.

The first run downloads the installer image. Give it a few minutes.

The detected values can be changed afterwards in the project's settings.

## Clone from git

| Field | What for |
| --- | --- |
| Repository URL | An SSH or HTTPS clone URL. Any server, including your own GitLab. |

The clone is done by **the git on your machine**. Your key, your `ssh` configuration and your server permissions come from your own setup; StackVo manages none of it. A URL that works in your terminal works here.

After cloning:

- If the repository has a `stackvo.json`, its settings are used as they are. The team's answer is yours, and the fields in the form are ignored.
- If it does not, the project is configured by detection from the files that arrived.

## What happens after Create

1. The project folder is prepared.
2. `stackvo.json` is written.
3. The configuration is generated: the Dockerfile, the compose files and the routing labels.
4. The image is built and the container is started.
5. The domain is written into the hosts file and included in the certificate.

## Worth knowing

- If the domain falls outside the workspace's suffix, the wildcard certificate does not cover it. The panel says so; regenerate the certificates after creating the project.
- If the suffix is an extension on browsers' HSTS list, such as `.dev`, the address only opens over HTTPS and the warning cannot be clicked through. Turn HTTPS on in Settings first.
- A wildcard alias reaches the certificate and the router, but no hosts file can express a wildcard. Those names do not resolve unless you add them by hand or the Local DNS responder is on.
- Runtimes with no generator are hidden from the list, and the panel names which ones.
- A project's name cannot be changed later. Its domain can.
