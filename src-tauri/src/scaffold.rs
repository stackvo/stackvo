//! Scaffold a brand-new framework project — the create half of P0-1.
//!
//! The import half shipped first: point the app at an existing folder and
//! detection infers the manifest. This half fills the folder in the first
//! place, by running the framework's own installer in a **throwaway
//! container** — `composer create-project` for Laravel and Symfony, wp-cli
//! for WordPress, `create-next-app` for Next.js, the documented clone-and-tidy
//! for Goravel. Nothing is installed on the host; the container is `--rm` and
//! only the bind-mounted project directory survives it.
//!
//! Scaffolding deliberately ends where adoption begins: once the installer
//! has written the files, the existing `project_adopt` path detects runtime,
//! server and document root from what is actually on disk — the same
//! machinery, whether the code arrived by `git clone` or by this module.
//!
//! On Linux the container would otherwise write root-owned files into the
//! user's checkout, so the invocation carries `--user <uid>:<gid>` on unix.
//! (Docker Desktop on macOS maps ownership anyway; the flag is harmless.)
//!
//! ## What is deliberately not here: Magento
//!
//! Detection knows `magento/product-community-edition` and there is no template
//! to match it, which the coverage test in `tests/scaffold_coverage.rs` requires
//! to be declared rather than merely true. The reason is that Magento Open
//! Source cannot be installed by a command this module could run:
//! `composer create-project magento/project-community-edition` resolves against
//! `repo.magento.com`, which authenticates with a public/private key pair the
//! user creates in their own Adobe Commerce account and writes into an
//! `auth.json`.
//!
//! Every other template here is a button that works with nothing but Docker. A
//! Magento button would be one that always fails until the user has been
//! somewhere else and obtained a credential this app has no place holding — and
//! failing *inside* a throwaway container, where the composer error about a 401
//! from a domain nobody mentioned is the only thing that comes back. Adoption
//! still works: clone or copy a Magento checkout in and detection identifies it,
//! `pub/` and all. Creating one is the ecosystem's job.

use crate::error::{Error, Result};
use serde::Serialize;
use std::path::Path;

/// The three ways a template can fill a directory.
///
/// A container run is the right answer when the ecosystem *has* a scaffolder —
/// `composer create-project` and friends know their own conventions better
/// than a template here ever will, and they stay current without anyone
/// editing this file.
///
/// Six of the frameworks people ask for have no scaffolder at all: Gin, Echo,
/// Flask, FastAPI, Sinatra and Rocket are each "write two files and run the
/// language's own build". Pulling a whole image to write 30 lines would be a
/// download for nothing, so those are written directly — and the dependency
/// install is not skipped, it *moves*: the project's own Dockerfile already
/// runs `pip install -r requirements.txt`, `go build`, `bundle install` or
/// `cargo build --release` for the container's platform, which is where those
/// belong anyway.
///
/// ## Every template installs the framework's latest release
///
/// That is the one promise all three kinds keep, and the third kind exists to
/// keep it. `composer create-project`, `npx create-…@latest`, `pip install
/// django` and `gem install rails` resolve the newest release on their own;
/// `flask>=3.0` and `fastapi>=0.115` are floors pip resolves upward from; a
/// Gemfile's `~> 4.1` tracks the newest 4.x. Go is the ecosystem where a
/// written file *cannot* say "latest": a `require` line names one exact
/// version and minimal version selection keeps it there, so the Gin and Echo
/// templates used to pin the release that was current when the line was
/// typed. They now write a `go.mod` with **no** `require` at all and let
/// `go mod tidy` resolve what `main.go` imports — which is the newest release
/// of each, written into `go.mod` and `go.sum` on the host the way
/// `composer.lock` is. The image that run needs is the one the project's own
/// Dockerfile builds from a minute later, so it is not a download for nothing.
///
/// Each framework's **minimum runtime** is honoured the same way, from what the
/// installer leaves on disk rather than from a table here that would go stale:
/// `detect` reads the `go` directive out of `go.mod` and the `php` constraint
/// out of `composer.json`, and `project_adopt` never puts a project on a
/// runtime below what its own files ask for. Goravel 1.18 asks for Go ≥ 1.25;
/// Laravel 12 asks for PHP ≥ 8.2.
pub enum Installer {
    /// `docker run <image> <args…>` over the bind-mounted project directory.
    Container(&'static str, &'static [&'static str]),
    /// `(relative path, contents)` written into the project directory.
    Files(&'static [(&'static str, &'static str)]),
    /// Files written first, then a container run over them — the Go shape,
    /// where the files name their imports and `go mod tidy` names the
    /// versions.
    FilesThenContainer {
        files: &'static [(&'static str, &'static str)],
        image: &'static str,
        command: &'static [&'static str],
    },
}

/// The image the Go installers run on, and the image a new Go project builds
/// from: `golang:<SUPPORTED_LANGUAGES_GO_DEFAULT>`. The two are held equal by a
/// test below, so the `go mod tidy` that resolves a project's dependencies runs
/// on the toolchain that will compile them, and the pull is shared.
pub const GO_IMAGE: &str = "golang:1.27";
/// Likewise for Node and Python: `node:<SUPPORTED_LANGUAGES_NODEJS_DEFAULT>`
/// and `python:<SUPPORTED_LANGUAGES_PYTHON_DEFAULT>-slim`, each held equal to
/// its setting by the same test. Every installer that runs on a runtime image
/// runs on the one a new project of that runtime builds from.
pub const NODE_IMAGE: &str = "node:24";
pub const PYTHON_IMAGE: &str = "python:3.14-slim";
/// Rust's installer image is the project's: `rust:1` floats on the major tag
/// because `manifest::lang_defaults` says so, and a `cargo install` wants the
/// newest stable anyway.
pub const RUST_IMAGE: &str = "rust:1";

/// Masonite 5's own `masonite new`, with every question answered on the line.
///
/// `pip install` as the non-root user the container runs as falls back to a
/// user install under `$HOME/.local`, whose `bin` is not on the image's PATH —
/// so it is put there first, or the `masonite` it just installed is "not
/// found". `--no-venv` because the project's Dockerfile installs
/// `requirements.txt` for the container's platform; `--no-git` and
/// `--no-serve` because neither a repository nor a server is this command's
/// to start; `--preset=none` because the CSS presets wire Vite assets that a
/// Python container has no Node to build, and a preset is one `craft preset`
/// away later; `--db=sqlite` so the project runs before any service exists.
/// The command writes `.env` and generates `APP_KEY` itself.
const MASONITE_INSTALL: &str = "export PATH=\"$HOME/.local/bin:$PATH\" \
 && pip install --quiet masonite-framework \
 && masonite new . --no-input --no-venv --no-git --no-serve --preset=none --db=sqlite";

/// Loco's `loco new`, which the Rust image has to build first.
///
/// `CARGO_HOME` is the image's `/usr/local/cargo`, root-owned, so a non-root
/// `cargo install` cannot write its registry there; it is moved to `/tmp`
/// along with the `bin` it installs into. `loco new` creates `<path>/<name>`
/// and refuses an existing directory, so it is pointed at `/tmp` and the
/// result is moved into the mounted project directory — dotfiles included,
/// which is what the `/.` spelling of `cp -a` is for. Every wizard question
/// has an answer on the line, and the wizard returns before asking anything
/// when `--db`, `--bg` and `--assets` are all given: SQLite, in-process
/// background workers and server-side assets, which is the SaaS starter.
const LOCO_INSTALL: &str = "export CARGO_HOME=/tmp/cargo PATH=\"/tmp/cargo/bin:$PATH\" \
 && cargo install --quiet loco \
 && loco new --path /tmp --name app --db sqlite --bg background-async --assets serverside \
 && cp -a /tmp/app/. . \
 && rm -rf /tmp/app";

/// The manual install the Goravel documentation gives, verbatim in its steps:
/// clone the skeleton shallow, drop its git metadata, resolve the modules, copy
/// the example environment and generate the application key.
///
/// Three things are removed beyond `.git`: the skeleton's own `Dockerfile` and
/// `docker-compose.yml`, because StackVo writes a project's Dockerfile into the
/// project directory on every generate and would overwrite the first while
/// leaving the second to mislead; and `.github`, whose workflows belong to the
/// skeleton's repository, not to a project that happens to start from it.
/// `.gitignore` stays — the documentation's `rm -rf .git*` takes it too, and a
/// project with no ignore file commits its `.env`.
///
/// `APP_HOST` and `APP_PORT` are rewritten in the copied `.env` because the
/// skeleton binds `127.0.0.1:3000`, which inside a container answers nothing
/// from outside it; `0.0.0.0:8080` is what `manifest::lang_defaults` gives the
/// Go runtime, so the port Traefik proxies to and the port the app listens on
/// agree without anyone editing either. The key is generated through
/// `go run . artisan` rather than the `./artisan` wrapper, which is the same
/// command without depending on an executable bit surviving the mount.
const GORAVEL_INSTALL: &str = "git clone --depth=1 https://github.com/goravel/goravel.git . \
 && rm -rf .git .github Dockerfile docker-compose.yml \
 && go mod tidy \
 && cp .env.example .env \
 && sed -i 's/^APP_HOST=.*/APP_HOST=0.0.0.0/; s/^APP_PORT=.*/APP_PORT=8080/' .env \
 && go run . artisan key:generate";

/// A framework this module knows how to install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Template {
    Laravel,
    Wordpress,
    Symfony,
    Nextjs,
    Nuxt,
    Vue,
    React,
    Svelte,
    Astro,
    // PHP frameworks and CMSes, all of them `composer create-project`.
    Cakephp,
    Yii,
    Codeigniter,
    Laminas,
    Drupal,
    Prestashop,
    Statamic,
    // The lang runtimes, now that they have generators.
    Django,
    Rails,
    Slim,
    Nest,
    Tina,
    Angular,
    Typo3,
    // Goravel is a Go application skeleton the way Laravel is a PHP one, and
    // installs the way its documentation says: clone, tidy, key:generate.
    Goravel,
    // Three more of that shape, one per lang runtime: Masonite (Python),
    // AdonisJS (Node) and Loco (Rust), each by its own scaffolder.
    Masonite,
    Adonisjs,
    Loco,
    // Written directly: these six have no scaffolder of their own.
    Gin,
    Echo,
    Flask,
    Fastapi,
    Sinatra,
    Rocket,
}

impl Template {
    pub const ALL: [Template; 33] = [
        Template::Laravel,
        Template::Wordpress,
        Template::Symfony,
        Template::Nextjs,
        Template::Nuxt,
        Template::Vue,
        Template::React,
        Template::Svelte,
        Template::Astro,
        Template::Cakephp,
        Template::Yii,
        Template::Codeigniter,
        Template::Laminas,
        Template::Drupal,
        Template::Prestashop,
        Template::Statamic,
        Template::Django,
        Template::Rails,
        Template::Slim,
        Template::Nest,
        Template::Tina,
        Template::Angular,
        Template::Typo3,
        Template::Goravel,
        Template::Masonite,
        Template::Adonisjs,
        Template::Loco,
        Template::Gin,
        Template::Echo,
        Template::Flask,
        Template::Fastapi,
        Template::Sinatra,
        Template::Rocket,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Template::Laravel => "laravel",
            Template::Wordpress => "wordpress",
            Template::Symfony => "symfony",
            Template::Nextjs => "nextjs",
            Template::Nuxt => "nuxt",
            Template::Vue => "vue",
            Template::React => "react",
            Template::Svelte => "svelte",
            Template::Astro => "astro",
            Template::Cakephp => "cakephp",
            Template::Yii => "yii",
            Template::Codeigniter => "codeigniter",
            Template::Laminas => "laminas",
            Template::Drupal => "drupal",
            Template::Prestashop => "prestashop",
            Template::Statamic => "statamic",
            Template::Django => "django",
            Template::Rails => "rails",
            Template::Slim => "slim",
            Template::Nest => "nest",
            Template::Tina => "tina",
            Template::Angular => "angular",
            Template::Typo3 => "typo3",
            Template::Goravel => "goravel",
            Template::Masonite => "masonite",
            Template::Adonisjs => "adonisjs",
            Template::Loco => "loco",
            Template::Gin => "gin",
            Template::Echo => "echo",
            Template::Flask => "flask",
            Template::Fastapi => "fastapi",
            Template::Sinatra => "sinatra",
            Template::Rocket => "rocket",
        }
    }

    pub fn parse(s: &str) -> Option<Template> {
        Template::ALL.into_iter().find(|t| t.as_str() == s)
    }

    /// Image and command for the installer container.
    ///
    /// Every command is fully specified so nothing prompts: an installer
    /// waiting for interactive input inside a `-d`-less `docker run` driven
    /// by an operation console is a hang, not a question.
    pub fn installer(self) -> Installer {
        match self {
            Template::Laravel => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "laravel/laravel",
                    ".",
                ],
            ),
            Template::Symfony => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "symfony/skeleton",
                    ".",
                ],
            ),
            Template::Wordpress => Installer::Container("wordpress:cli", &["wp", "core", "download"]),

            // The JavaScript installers. `--no-install` is not a shortcut: a
            // node project's Dockerfile runs `npm install` for the container's
            // own platform, and a host `node_modules` copied into the image is
            // how an arm64 binary ends up in an amd64 container — the very
            // thing NODE_DOCKERIGNORE exists to prevent.
            Template::Nuxt => Installer::Container(
                NODE_IMAGE,
                // `--template` and `--force` are both required: without the
                // first it refuses on a non-interactive terminal, without the
                // second it refuses because the mount point already exists.
                &[
                    "npx",
                    "nuxi@latest",
                    "init",
                    ".",
                    "--template",
                    "minimal",
                    "--packageManager",
                    "npm",
                    "--gitInit",
                    "false",
                    "--no-install",
                    "--force",
                ],
            ),
            Template::Vue => Installer::Container(
                NODE_IMAGE,
                &[
                    "npm",
                    "create",
                    "vite@latest",
                    ".",
                    "--",
                    "--template",
                    "vue-ts",
                ],
            ),
            Template::React => Installer::Container(
                NODE_IMAGE,
                &[
                    "npm",
                    "create",
                    "vite@latest",
                    ".",
                    "--",
                    "--template",
                    "react-ts",
                ],
            ),
            // SvelteKit rather than Vite's svelte template: the router and the
            // dev server are the part a local environment manager can host.
            Template::Svelte => Installer::Container(
                NODE_IMAGE,
                &[
                    "npx",
                    "sv@latest",
                    "create",
                    "--template",
                    "minimal",
                    "--types",
                    "ts",
                    "--no-add-ons",
                    "--no-install",
                    ".",
                ],
            ),
            // `--ignore-platform-reqs` on every composer project except the
            // two that predate it: the *installer* container is not the
            // machine the code will run on. CakePHP asks for ext-intl and the
            // composer image has none, so without it the install fails on a
            // requirement StackVo's own PHP image satisfies. Measured: both
            // CakePHP and CodeIgniter exit 2 without the flag, 0 with it.
            Template::Cakephp => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "cakephp/app",
                    ".",
                ],
            ),
            Template::Yii => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "yiisoft/yii2-app-basic",
                    ".",
                ],
            ),
            Template::Codeigniter => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "codeigniter4/appstarter",
                    ".",
                ],
            ),
            // Zend Framework's maintained successor; the Zend packages are
            // abandoned and `composer create-project zendframework/*` installs
            // code nobody has patched since 2019.
            Template::Laminas => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "laminas/laminas-mvc-skeleton",
                    ".",
                ],
            ),
            Template::Drupal => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "drupal/recommended-project",
                    ".",
                ],
            ),
            Template::Prestashop => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "prestashop/prestashop",
                    ".",
                ],
            ),
            // Statamic is a Laravel application with a flat-file CMS on top,
            // and detection knew the `statamic/cms` marker before there was
            // any way to create one — the catalogue could recognise a Statamic
            // site nobody could scaffold here.
            Template::Statamic => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "statamic/statamic",
                    ".",
                ],
            ),
            // The lang runtimes. Django ships its own scaffolder; Rails needs
            // the *full* ruby image — the slim one has no toolchain, and
            // installing rails there dies compiling websocket-driver's native
            // extension. Measured both ways.
            Template::Django => Installer::Container(
                PYTHON_IMAGE,
                &[
                    "sh",
                    "-c",
                    "pip install --quiet django && django-admin startproject app .",
                ],
            ),
            Template::Rails => Installer::Container(
                "ruby:3.3",
                &[
                    "sh",
                    "-c",
                    "gem install rails --no-document -q && rails new . --skip-bundle --skip-git --force",
                ],
            ),
            Template::Slim => Installer::Container(
                "composer:2",
                &[
                    "create-project",
                    "--prefer-dist",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "slim/slim-skeleton",
                    ".",
                ],
            ),
            Template::Nest => Installer::Container(
                NODE_IMAGE,
                &[
                    "npx",
                    "-y",
                    "@nestjs/cli",
                    "new",
                    ".",
                    "--skip-git",
                    "--skip-install",
                    "--package-manager",
                    "npm",
                ],
            ),
            // `--pkg-manager` is required, not tidiness: without it the
            // installer asks which one to use and an operation console has no
            // way to answer. Measured — it hangs on the prompt.
            Template::Tina => Installer::Container(
                NODE_IMAGE,
                &[
                    "npx",
                    "-y",
                    "create-tina-app@latest",
                    ".",
                    "--template",
                    "basic",
                    "--pkg-manager",
                    "npm",
                ],
            ),
            Template::Angular => Installer::Container(
                NODE_IMAGE,
                &[
                    "npx", "-y", "@angular/cli@latest", "new", "app",
                    "--directory", ".", "--skip-git", "--skip-install", "--defaults",
                ],
            ),
            Template::Typo3 => Installer::Container(
                "composer:2",
                &[
                    "create-project", "--prefer-dist", "--no-interaction",
                    "--ignore-platform-reqs", "typo3/cms-base-distribution", ".",
                ],
            ),

            // Goravel's skeleton is a repository, not a package, so there is
            // no `go install`-style scaffolder to call; the documented install
            // is the sequence in GORAVEL_INSTALL, run on the toolchain the
            // project will build with. The official image has git.
            Template::Goravel => Installer::Container(GO_IMAGE, &["sh", "-c", GORAVEL_INSTALL]),
            Template::Masonite => Installer::Container(PYTHON_IMAGE, &["sh", "-c", MASONITE_INSTALL]),
            // `--kit`, `--db` and `--auth-guard` are the three questions the
            // hypermedia kit asks; each answered, nothing is. Dependencies
            // are installed by the command itself and there is no flag
            // against it — harmless, since NODE_DOCKERIGNORE keeps that tree
            // out of the image and the project's Dockerfile installs again.
            // HOST and PORT need no rewriting: AdonisJS lets an existing
            // process.env win over `.env` (measured in @adonisjs/env's
            // processor), and the node Dockerfile sets both.
            Template::Adonisjs => Installer::Container(
                NODE_IMAGE,
                &[
                    "npm",
                    "create",
                    "adonisjs@latest",
                    ".",
                    "--",
                    "--kit=hypermedia",
                    "--db=sqlite",
                    "--auth-guard=session",
                    "--pkg=npm",
                ],
            ),
            Template::Loco => Installer::Container(RUST_IMAGE, &["sh", "-c", LOCO_INSTALL]),

            // ---- written, not run ------------------------------------------
            //
            // Each is the framework's own hello-world, bound to 0.0.0.0 on the
            // port `manifest::lang_defaults` gives its runtime. The binding is
            // the load-bearing part: a server on 127.0.0.1 inside a container
            // answers nothing from outside it, and Traefik is outside it — the
            // same failure the node dev server taught.
            //
            // The two Go ones are written and then tidied: `go.mod` names no
            // versions, `main.go` names the imports, and `go mod tidy` on the
            // project's own toolchain resolves each to its newest release.
            Template::Gin => Installer::FilesThenContainer {
                files: &[("go.mod", GO_MOD), ("main.go", GIN_MAIN)],
                image: GO_IMAGE,
                command: &["go", "mod", "tidy"],
            },
            Template::Echo => Installer::FilesThenContainer {
                files: &[("go.mod", GO_MOD), ("main.go", ECHO_MAIN)],
                image: GO_IMAGE,
                command: &["go", "mod", "tidy"],
            },
            Template::Flask => Installer::Files(&[
                ("requirements.txt", "flask>=3.0\n"),
                ("main.py", FLASK_MAIN),
            ]),
            Template::Fastapi => Installer::Files(&[
                ("requirements.txt", "fastapi>=0.115\nuvicorn[standard]>=0.32\n"),
                ("main.py", FASTAPI_MAIN),
            ]),
            Template::Sinatra => Installer::Files(&[
                ("Gemfile", SINATRA_GEMFILE),
                ("app.rb", SINATRA_APP),
            ]),
            Template::Rocket => Installer::Files(&[
                ("Cargo.toml", ROCKET_CARGO),
                ("Rocket.toml", ROCKET_CONFIG),
                ("src/main.rs", ROCKET_MAIN),
            ]),
            Template::Astro => Installer::Container(
                NODE_IMAGE,
                &[
                    "npm",
                    "create",
                    "astro@latest",
                    ".",
                    "--",
                    "--template",
                    "minimal",
                    "--no-install",
                    "--no-git",
                    "--skip-houston",
                    "--yes",
                ],
            ),
            Template::Nextjs => Installer::Container(
                NODE_IMAGE,
                // Every choice pinned; `--yes` alone still asks about Turbopack
                // on some versions.
                &[
                    "npx",
                    "--yes",
                    "create-next-app@latest",
                    ".",
                    "--ts",
                    "--eslint",
                    "--app",
                    "--no-tailwind",
                    "--no-src-dir",
                    "--import-alias",
                    "@/*",
                    "--use-npm",
                ],
            ),
        }
    }

    /// The files a template writes before anything runs, if it writes any.
    pub fn files(self) -> Option<&'static [(&'static str, &'static str)]> {
        match self.installer() {
            Installer::Files(files) | Installer::FilesThenContainer { files, .. } => Some(files),
            Installer::Container(..) => None,
        }
    }
}

/// The `docker run` invocation that fills `host_dir` with a new project — or,
/// for a template that writes its files first, the run that follows them.
///
/// `user` is `Some("uid:gid")` on unix so the files land owned by the person
/// who asked for them, not by root.
pub fn run_args(template: Template, host_dir: &str, user: Option<&str>) -> Option<Vec<String>> {
    let (image, command) = match template.installer() {
        Installer::Container(image, command) => (image, command),
        Installer::FilesThenContainer { image, command, .. } => (image, command),
        // A written template runs nothing; the caller writes the files.
        Installer::Files(_) => return None,
    };
    let mount = format!("{}:/app", crate::paths::to_docker_mount(host_dir));

    let mut args: Vec<String> = ["run", "--rm", "-v", &mount, "-w", "/app"]
        .into_iter()
        .map(String::from)
        .collect();

    if let Some(user) = user {
        args.push("--user".into());
        args.push(user.into());
        // A non-root user in a stock image has no writable HOME; installers
        // (composer, npm, wp-cli) all want a cache directory.
        args.push("-e".into());
        args.push("HOME=/tmp".into());
    }

    args.push(image.into());
    args.extend(command.iter().map(|s| s.to_string()));
    Some(args)
}

/// Write a template's files into `dir`, creating parent directories as it
/// goes (`src/main.rs` needs `src/`). Returns what it wrote.
pub fn write_files(template: Template, dir: &Path) -> Result<Vec<String>> {
    let Some(files) = template.files() else {
        return Ok(Vec::new());
    };

    let mut written = Vec::new();
    for (name, contents) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
        }
        std::fs::write(&path, contents)
            .map_err(|e| Error::io(format!("writing {}", path.display()), e))?;
        written.push((*name).to_string());
    }
    Ok(written)
}

/// `uid:gid` of the invoking user, unix only.
pub async fn current_user() -> Option<String> {
    #[cfg(unix)]
    {
        let read = |flag: &str| {
            let flag = flag.to_string();
            async move {
                let out = tokio::process::Command::new("id")
                    .arg(flag)
                    .output()
                    .await
                    .ok()?;
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<u32>()
                    .ok()
            }
        };
        let uid = read("-u").await?;
        let gid = read("-g").await?;
        Some(format!("{uid}:{gid}"))
    }
    #[cfg(not(unix))]
    {
        None
    }
}

// ---- the written templates ------------------------------------------------
//
// Raw strings so the code inside reads as the code it is. Every one binds to
// 0.0.0.0 and to the port `manifest::lang_defaults` gives its runtime, so the
// project the adopter writes and the server the file starts agree without
// anyone editing either.

/// One `go.mod` for both Go hello-worlds, and deliberately without a `require`
/// block: the imports in `main.go` are the dependency list, and the `go mod
/// tidy` that follows the write resolves each to its newest release and pins
/// it here, with its checksum in `go.sum`. A version typed into this constant
/// would be the version new projects got for as long as nobody retyped it —
/// Gin sat at 1.10.0 here while 1.12.0 shipped.
///
/// The `go` directive is the floor the newest Gin and Echo both ask for
/// (1.25, read off their own `go.mod`); `tidy` raises it if a release asks for
/// more. The image that runs it is `GO_IMAGE`, which is newer than either.
const GO_MOD: &str = r#"module app

go 1.25
"#;

const GIN_MAIN: &str = r#"package main

import "github.com/gin-gonic/gin"

func main() {
	r := gin.Default()
	r.GET("/", func(c *gin.Context) {
		c.JSON(200, gin.H{"message": "Gin is running on StackVo"})
	})
	// 0.0.0.0, not localhost: Traefik reaches this from outside the container.
	_ = r.Run("0.0.0.0:8080")
}
"#;

// Echo v5: the module path carries the major, so `/v5` is what "latest" means
// here — `go mod tidy` on `/v4` would resolve the newest v4 forever. The
// handler takes `*echo.Context` (a struct in v5, an interface in v4) and the
// framework no longer carries a logger of its own, so the start error goes
// through `log/slog` the way its own README does.
const ECHO_MAIN: &str = r#"package main

import (
	"log/slog"
	"net/http"

	"github.com/labstack/echo/v5"
)

func main() {
	e := echo.New()
	e.GET("/", func(c *echo.Context) error {
		return c.JSON(http.StatusOK, map[string]string{"message": "Echo is running on StackVo"})
	})
	// 0.0.0.0, not localhost: Traefik reaches this from outside the container.
	if err := e.Start("0.0.0.0:8080"); err != nil {
		slog.Error("failed to start server", "error", err)
	}
}
"#;

const FLASK_MAIN: &str = r#"from flask import Flask

app = Flask(__name__)


@app.route("/")
def index():
    return {"message": "Flask is running on StackVo"}


if __name__ == "__main__":
    # host=0.0.0.0 so Traefik can reach it from outside the container.
    app.run(host="0.0.0.0", port=8000)
"#;

const FASTAPI_MAIN: &str = r#"from fastapi import FastAPI

app = FastAPI()


@app.get("/")
def read_root():
    return {"message": "FastAPI is running on StackVo"}


if __name__ == "__main__":
    # Started through main.py rather than the uvicorn CLI, so the manifest's
    # default start command (`python main.py`) is the one that works.
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
"#;

const SINATRA_GEMFILE: &str = r#"source "https://rubygems.org"

gem "sinatra", "~> 4.1"
gem "puma"
gem "rackup"
"#;

const SINATRA_APP: &str = r#"require "sinatra"

# bind, not localhost — a container's loopback is not reachable from outside.
set :bind, "0.0.0.0"
set :port, 4567

get "/" do
  content_type :json
  '{"message":"Sinatra is running on StackVo"}'
end
"#;

const ROCKET_CARGO: &str = r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
rocket = { version = "0.5", features = ["json"] }
"#;

const ROCKET_CONFIG: &str = r#"[default]
# Rocket binds to 127.0.0.1 unless told otherwise, which inside a container
# means nothing outside it can connect.
address = "0.0.0.0"
port = 8080
"#;

const ROCKET_MAIN: &str = r#"#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Rocket is running on StackVo"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_parses_its_own_name() {
        for t in Template::ALL {
            assert_eq!(Template::parse(t.as_str()), Some(t));
        }
        // Rails became real in the meantime; the unknown case needs a name
        // that is still not one of ours.
        assert_eq!(Template::parse("spring-boot"), None);
        // Each one is a different installer contract, and the count is here so
        // that adding an enum variant without adding it to `ALL` fails rather
        // than producing a template nothing can offer.
        assert_eq!(Template::ALL.len(), 33);
    }

    #[test]
    fn the_runtime_installers_run_on_the_image_a_new_project_builds_from() {
        // `go mod tidy` resolves and checksums modules for the toolchain that
        // runs it; the project's Dockerfile compiles them on
        // `golang:<SUPPORTED_LANGUAGES_GO_DEFAULT>`. One image for both, or the
        // two can disagree about a `go` directive and the pull is paid twice.
        // Node and Python hold the same line for the same reason.
        let setting = |key: &str| {
            crate::config::SETTINGS
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| *v)
                .unwrap_or_else(|| panic!("config::SETTINGS names {key}"))
        };
        assert_eq!(GO_IMAGE, format!("golang:{}", setting("SUPPORTED_LANGUAGES_GO_DEFAULT")));
        assert_eq!(NODE_IMAGE, format!("node:{}", setting("SUPPORTED_LANGUAGES_NODEJS_DEFAULT")));
        assert_eq!(
            PYTHON_IMAGE,
            format!("python:{}-slim", setting("SUPPORTED_LANGUAGES_PYTHON_DEFAULT"))
        );
        assert_eq!(
            RUST_IMAGE,
            format!("rust:{}", crate::manifest::lang_defaults("rust").unwrap().version)
        );
        for (t, image) in [
            (Template::Goravel, GO_IMAGE),
            (Template::Gin, GO_IMAGE),
            (Template::Echo, GO_IMAGE),
            (Template::Masonite, PYTHON_IMAGE),
            (Template::Django, PYTHON_IMAGE),
            (Template::Adonisjs, NODE_IMAGE),
            (Template::Nextjs, NODE_IMAGE),
            (Template::Loco, RUST_IMAGE),
        ] {
            let line = run_args(t, "/x", None).unwrap().join(" ");
            assert!(line.contains(image), "{t:?} runs on {line}");
        }
    }

    #[test]
    fn the_three_skeletons_on_lang_runtimes_answer_every_question_and_land_in_place() {
        // Masonite: user-install bin on PATH first, then the documented
        // command with each wizard question answered.
        let line = run_args(Template::Masonite, "/x", None).unwrap().join(" ");
        for step in [
            "$HOME/.local/bin:$PATH",
            "pip install --quiet masonite-framework",
            "masonite new . --no-input --no-venv --no-git --no-serve --preset=none --db=sqlite",
        ] {
            assert!(line.contains(step), "masonite install lacks `{step}`: {line}");
        }
        assert!(!line.contains("pip install masonite "), "the `masonite` package is the unmaintained 4.x");

        // AdonisJS: the three questions the kit asks, each answered, into `.`.
        let line = run_args(Template::Adonisjs, "/x", None).unwrap().join(" ");
        assert!(line.contains("npm create adonisjs@latest . -- --kit=hypermedia --db=sqlite --auth-guard=session"));

        // Loco: a writable cargo home, the CLI built, the wizard bypassed, and
        // the directory it insists on creating moved into the mount.
        let line = run_args(Template::Loco, "/x", None).unwrap().join(" ");
        for step in [
            "CARGO_HOME=/tmp/cargo",
            "cargo install --quiet loco",
            "loco new --path /tmp --name app --db sqlite --bg background-async --assets serverside",
            "cp -a /tmp/app/. .",
        ] {
            assert!(line.contains(step), "loco install lacks `{step}`: {line}");
        }
    }

    #[test]
    fn go_templates_pin_no_version_and_resolve_the_newest_at_creation() {
        // A `require` line in a written go.mod is a version frozen at the
        // moment somebody typed it; the newest release is what `tidy` finds.
        for t in [Template::Gin, Template::Echo] {
            let files = t.files().expect("the Go hello-worlds are written");
            let go_mod = files
                .iter()
                .find(|(name, _)| *name == "go.mod")
                .map(|(_, body)| *body)
                .unwrap();
            assert!(!go_mod.contains("require"), "{t:?} pins a version: {go_mod}");
            assert!(go_mod.contains("go 1.25"), "{t:?} states the floor Gin and Echo ask for");
            let line = run_args(t, "/x", None).unwrap().join(" ");
            assert!(line.ends_with("go mod tidy"), "{t:?} is not tidied: {line}");
        }
        // Echo's major lives in its module path, so "newest" has to be spelled.
        assert!(ECHO_MAIN.contains("github.com/labstack/echo/v5"));
        assert!(!ECHO_MAIN.contains("echo/v4"));

        // Goravel: the documented sequence, on the toolchain the project will
        // build with, and bound where Traefik can reach it. The skeleton's own
        // Dockerfile goes because StackVo writes one into the same place.
        let line = run_args(Template::Goravel, "/x", None).unwrap().join(" ");
        for step in [
            "git clone --depth=1 https://github.com/goravel/goravel.git .",
            "go mod tidy",
            "cp .env.example .env",
            "APP_HOST=0.0.0.0",
            "APP_PORT=8080",
            "go run . artisan key:generate",
            "rm -rf .git .github Dockerfile docker-compose.yml",
        ] {
            assert!(line.contains(step), "goravel install lacks `{step}`: {line}");
        }
        assert!(!line.contains("rm -rf .git*"), "the ignore file must survive");
    }

    #[test]
    fn the_installer_runs_throwaway_with_only_the_project_dir_surviving() {
        let args = run_args(Template::Laravel, "/Users/x/stackvo/projects/shop", None).unwrap();
        let line = args.join(" ");
        assert!(line.starts_with("run --rm"));
        assert!(line.contains("-v /Users/x/stackvo/projects/shop:/app"));
        assert!(line.contains("composer:2 create-project"));
        assert!(line.contains("--no-interaction"));
        assert!(line.ends_with("laravel/laravel ."));
    }

    #[test]
    fn a_unix_user_gets_ownership_and_a_writable_home() {
        let args = run_args(Template::Nextjs, "/x/projects/web", Some("501:20")).unwrap();
        let line = args.join(" ");
        assert!(line.contains("--user 501:20"));
        assert!(line.contains("-e HOME=/tmp"));
        // The flags come before the image, or docker treats them as the
        // container command.
        assert!(line.find("--user").unwrap() < line.find(NODE_IMAGE).unwrap());
    }

    #[test]
    fn a_written_template_lands_its_files_and_binds_to_all_interfaces() {
        let dir = std::env::temp_dir().join(format!("stackvo-tpl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Rocket is the shape that proves the nesting: src/main.rs needs a
        // directory that does not exist yet.
        let written = write_files(Template::Rocket, &dir).unwrap();
        assert!(written.contains(&"src/main.rs".to_string()));
        assert!(dir.join("src/main.rs").is_file());

        // The one property every written template must hold: bound to
        // 0.0.0.0, never loopback. A container's localhost answers nothing
        // from outside it, and Traefik is outside it.
        for t in [
            Template::Gin,
            Template::Echo,
            Template::Flask,
            Template::Fastapi,
            Template::Sinatra,
            Template::Rocket,
        ] {
            let files = t.files().unwrap_or_else(|| panic!("{t:?} should be written"));
            let body: String = files.iter().map(|(_, c)| *c).collect();
            assert!(body.contains("0.0.0.0"), "{t:?} does not bind to 0.0.0.0");

            // Comments are where these templates *explain* the loopback trap,
            // so scanning the whole file for 127.0.0.1 flags the explanation
            // as the bug. Only code lines count.
            let loopback = body
                .lines()
                .map(str::trim)
                .filter(|l| !l.starts_with('#') && !l.starts_with("//"))
                .any(|l| l.contains("127.0.0.1"));
            assert!(!loopback, "{t:?} binds to loopback in code");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_this_runs_can_prompt() {
        for t in Template::ALL {
            // A written template runs no process, so it cannot ask anything —
            // the guard is about installers.
            let Some(args) = run_args(t, "/x", None) else {
                continue;
            };
            let line = args.join(" ");
            // Interactive installers hang the operation console; each command
            // must either be non-interactive by nature or say so explicitly.
            // Each of these was measured against a real container: with the
            // flag the installer completes, without it the run either asks a
            // question (which an operation console cannot answer) or refuses.
            let non_interactive = line.contains("--no-interaction") // composer
                || line.contains("--yes") // astro
                || line.contains("wp core download") // wp-cli takes none
                || line.contains("--force") // nuxi, after --template
                || line.contains("--no-add-ons") // sv
                || line.contains("--template vue-ts") // vite
                || line.contains("--template react-ts")
                || line.contains("--skip-git --force") // rails
                || line.contains("django-admin startproject") // takes none
                || line.contains("--skip-install") // nest
                || line.contains("--pkg-manager npm") // create-tina-app
                || line.ends_with("go mod tidy") // gin, echo: takes none
                || line.contains("git clone --depth=1") // goravel: a public clone asks nothing
                || line.contains("--no-input") // masonite
                || line.contains("--kit=hypermedia --db=sqlite --auth-guard=session") // adonisjs: its three questions
                || line.contains("--db sqlite --bg background-async --assets serverside"); // loco: its three
            assert!(non_interactive, "{t:?} could prompt: {line}");
        }
    }
}
