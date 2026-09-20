# Glossary

The words these pages use with a fixed meaning. Where the window shows the
word, it is the same word.

| Term | Meaning |
| --- | --- |
| **Workspace** | The one folder StackVo keeps everything in — `~/.stackvo` by default, `STACKVO_ROOT` moves it. Projects, the `.env`, the certificate and the generated files live inside it. The folder is the state; there is no database. |
| **Project** | One folder under `<workspace>/projects/`, with a `stackvo.json` beside the code. Each project is one container, one domain and one runtime version. |
| **Manifest** | `stackvo.json`: the only file you edit by hand. It declares the runtime and its version, the web server, the document root, the services the project needs, its domain and aliases, and its hooks. |
| **Stack** | Everything the workspace runs together: the reverse proxy, the certificate authority, the shared services and every project's container. `stackvo up` and `stackvo down` act on it as a whole. |
| **Service** | A shared container the projects use — a database, a cache, a mail catcher, an admin UI. Services live in the workspace's `.env`, not in a project. |
| **Instance** | One running copy of a service from the catalogue, with its own port and credentials. Two instances of MySQL can be two versions. |
| **Catalogue** | The list of services that can be installed, fetched from a registry as signed packages rather than compiled into the app. **Catalogue** in the sidebar. |
| **Package** | One entry in the catalogue: a service's compose fragment, its image at a pinned digest, and the commands it brings. Verified before it is unpacked. |
| **Generated files** | `<workspace>/generated/`: the compose files, Dockerfiles and server configs rendered from the manifests. Never edited by hand and safe to delete; `stackvo generate` writes them again. |
| **`.env`** | The workspace's settings file: the domain suffix, which services are enabled, ports, server settings. The settings page writes it; most changes need a regenerate. |
| **Domain suffix** | The ending every hostname is built from — `.loc` by default, so a project named `shop` answers at `shop.loc`. `DEFAULT_TLD_SUFFIX` in `.env`. |
| **Hosts file** | The operating system's `/etc/hosts` (or its Windows equivalent). StackVo writes project names into it inside a marked block, after showing you the diff, so the browser can find `shop.loc`. |
| **CA** | The certificate authority StackVo creates once per machine and asks you to trust once. It signs one wildcard certificate that covers every project and service, which is why there is no browser warning. |
| **Doctor** | The settings section that checks the machine — Docker, the socket, ports, the hosts file, the certificate, disk — and names the repair beside each finding. `stackvo doctor` in a terminal. |
| **Snapshot** | A named copy of a project's database, taken before a migration and restored by name. Distinct from a **dump**, which is a file you export and keep. |
| **Worktree** | A git worktree with its own environment: a second checkout of the same project on another branch, with its own container, domain and, if you ask, its own database. *A branch per environment* in the guide. |
| **Tunnel** | A public URL for a project, through one of the providers on the **Share** card, so somebody outside your network can open it. |
| **Hook** | A command the project runs at a moment you choose — after start, before stop — declared in the manifest. |
| **Template** | A framework's own installer run in a throwaway container when you create a project, so a Laravel or Next.js project starts the way its framework intends. |
| **Lock file** | `stackvo.lock`: the service versions and package digests a project was actually built against, so a clone gets the same stack. |
| **Preset** | A workspace's stack settings exported as `stackvo.preset.json`, to be imported on another machine. |
| **Runtime** | The language the project runs on and its version: PHP, Node, Python, Go, Ruby, Rust, Bun or Deno. One per project; a monorepo is one project with several parts. |
| **Profile** | A group of containers that start together. Projects and services are separate profiles, so bringing the stack down takes both. |
| **MCP** | Model Context Protocol. `stackvo-mcp` is a server an AI assistant connects to; its tools are the same commands the window and the CLI use. |
| **TUI** | `stackvo tui`: the window's overview in a terminal, for a machine you reach over SSH. |
