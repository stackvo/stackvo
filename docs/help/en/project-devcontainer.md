# Devcontainer

Writes a `.devcontainer/` folder into this project, so somebody who does not have StackVo can open the repository in VS Code or GitHub Codespaces and get the same environment.

## Controls

| Control | What it does |
| --- | --- |
| Show what would be written | Renders every file and shows it. Nothing is written yet. |
| Write files into the project | Writes them into `<project>/.devcontainer/`. |

## What it carries

- **The same container.** The Dockerfile is the one StackVo builds this project with: the same PHP version, the same extensions, the same web server.
- **The services this project declares**, from the same packages this machine installed — the same image and the same version.
- **Your container names.** `stackvo-mysql-8-4` stays exactly that, because your project's own `.env` names it. Renaming it here would break the application on the machine that cannot see why.
- **The ports this workspace allocated**, so a database client already pointed at them keeps working.

## What it does not carry, and why

- **The domain and its HTTPS.** `shop.loc` works because of a certificate authority installed in this machine's trust stores and a router that is not part of this project. The application is reached on a forwarded port instead.
- **Passwords.** Each one leaves as a name — `DEV_MYSQL_8_4_ROOT_PASSWORD` — that Compose reads from `.devcontainer/.env`. A `.gitignore` is written beside it holding that one line.
- **Per-service tuning that contains a password.** A `my.cnf` is carried; one that holds a secret is not, because a placeholder can be filled in in a compose file and cannot be in a config file.

## Worth knowing

- These files are meant to be committed. That is the point.
- They are rewritten from the manifest every time. Edit `stackvo.json` and write again rather than editing them.
- Dependencies are not installed for you on a PHP project. Nothing in a manifest says the project uses Composer, and a first-open command that fails is worse than one that is absent. Node and the other runtimes do install, because their manifest names the command.
- Node, Python, Go, Ruby, Rust, Bun and Deno projects get the toolchain and are told to stay running. Your editor attaches to that container; the application is not its main process, so you start it yourself.
