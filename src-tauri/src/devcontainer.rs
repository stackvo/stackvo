//! Exporting a project as a devcontainer (A-7).
//!
//! ## The report said this was the generator's little sibling. It is not.
//!
//! The competitor row reads "DDEV lists GitHub Codespaces among its supported
//! platforms; the generator already renders compose, and
//! `.devcontainer/devcontainer.json` is a small sibling of that". The first
//! half is true and the second was written without reading
//! [`crate::generator::render_compose_service`]. What that function produces is
//! bound to *this machine* in five separate ways, and every one of them is
//! load-bearing:
//!
//! * `volumes: <projects_root>/<name>:/var/www/html` and
//!   `<host_root>/logs/...` — absolute paths under the user's home;
//! * `context: ./projects/<name>` — relative to `generated/`, a directory the
//!   application owns and no repository contains;
//! * `networks: stackvo-net` — a network created by a different compose file;
//! * Traefik labels, in a file that has no Traefik in it. The router lives in
//!   the workspace stack, and the certificate that makes `shop.loc` load is
//!   issued by a CA installed in this machine's trust stores;
//! * the backing services are in **another compose file entirely**, rendered
//!   with values pulled from the OS keystore.
//!
//! Copying that file into a repository produces something that cannot start on
//! any other machine. So this is not the generator's output relabelled; it is a
//! **second rendering of the same manifest**, and the whole design question is
//! which facts survive the trip and which are properties of this laptop.
//!
//! ## Written on request, never on generate
//!
//! [`crate::agentctx`] writes into the project directory on every generation,
//! and says in the file that `.stackvo/` is not meant to be committed. This is
//! the opposite: `.devcontainer/` exists **to** be committed — it is how a
//! teammate with no StackVo gets the same PHP version, the same extensions and
//! the same database. A file that turns up in somebody's `git status` because
//! they pressed Start is a file they learn to `git checkout`, so nothing here
//! runs from generation. The user asks, sees what would be written, and then it
//! is written.
//!
//! ## What survives, and how each one is kept honest
//!
//! **The Dockerfile is the generator's own.** [`crate::generator`] renders it,
//! not this module — the whole value of the export is that the container is the
//! one StackVo builds, and a second renderer would drift from it in the week
//! nobody is looking. It needs no adjustment either: the PHP image never copies
//! the source, because in StackVo the source arrives through a bind mount, and
//! in a devcontainer it arrives through the workspace mount. The same shape is
//! right for both reasons.
//!
//! The runtimes are the exception, and it is the same asymmetry
//! [`crate::release`] found from the other side. A node or Python Dockerfile
//! does `COPY . .`, installs and sets a `CMD` — it is a *snapshot of the
//! application*, and a dev container must not be one: the source it should be
//! working on is the mounted one, and a container whose main process is the
//! application is a terminal that dies when the application does. So those get
//! a toolchain-only image and `sleep infinity`, with the install moved to
//! `postCreateCommand` where it runs against the mount.
//!
//! **The services are their own packages' fragments**, rendered through
//! [`crate::render::substitute`] — the same strict substituter, where an
//! unknown name is a refusal rather than an empty string. Not a table of image
//! names written here: ADR 0011 is that this application carries no service
//! definitions, and "except in the exporter" is how that decision would have
//! been lost. It also could not have been done by hand and be correct — the
//! mapping from StackVo's `settings.ROOT_PASSWORD` to MySQL's
//! `MYSQL_ROOT_PASSWORD` exists only in that package's template, and a compose
//! file that starts `mysql` with no root password set does not start at all.
//!
//! **Six variables are answered differently**, and they are exactly the ones
//! that name this machine:
//!
//! | variable | here | why |
//! | --- | --- | --- |
//! | `file.*` | `./configs/<slug>/<name>` | the host path is under `~/.stackvo` |
//! | `instance.logs` | a named volume | same |
//! | `settings.*` (secret) | `${DEV_…}` | ADR 0010, and this file is committed |
//! | `network` | `default` | `stackvo-net` belongs to another compose file |
//! | `instance.domain` | `localhost` | there is no Traefik and no certificate |
//! | `port.*` | the host port this machine allocated | see below |
//!
//! The ports are the one that looks like it should have been the container
//! port. It is the host allocation on purpose: it is the number already in the
//! author's database client and their notes, and the export is read by that
//! author first. Codespaces forwards whatever is published either way.
//!
//! **Nothing else is invented.** The container names are kept exactly —
//! `stackvo-mysql-8-4` — which looks odd in a repository that has nothing to do
//! with StackVo and is the only correct answer: the project's own `.env` says
//! `DB_HOST=stackvo-mysql-8-4`, and renaming the service here breaks the
//! application on the machine that has no way to know why.
//!
//! ## Secrets leave as names
//!
//! A secret setting renders as `${DEV_<SLUG>_<KEY>}`, which Compose expands
//! from a `.env` beside the file, and the names are collected into a
//! `.env.example`. `DEV_` is chosen rather than anything shorter because
//! [`crate::template::PREFIXES`] lists the eight prefixes the workspace
//! renderer substitutes — a placeholder starting with `STACKVO_` would have
//! been eaten on the way out, silently, leaving an empty password in a file
//! headed for a repository.
//!
//! A `.gitignore` is written beside them holding one line, `.env`. The whole
//! point of the placeholder is that the value does not travel, and the file the
//! reader is about to create is the file that would undo it.
//!
//! A rendered **config** file is checked for the same placeholder before it is
//! offered, and dropped with a note if it has one: `${…}` is expanded by
//! Compose in a compose file and is a literal five characters in a `my.cnf`.
//! That is a construction guarantee rather than a hope that no package ever
//! puts a secret in a config template.

use crate::error::{Code, Error, Result};
use crate::instances::{Instance, Table};
use crate::manifest::Manifest;
use crate::pkg::Catalogue;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// The directory this writes into the project.
pub const DIR: &str = ".devcontainer";

/// One file, at a path relative to [`DIR`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub path: String,
    pub contents: String,
}

/// What an export would write, before any of it is written.
///
/// The same plan-then-apply split [`crate::handover`] uses, and for a weaker
/// version of the same reason: this one writes into somebody's repository, and
/// what lands in a commit should be readable before it is there.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub project: String,
    pub files: Vec<File>,
    /// The `${DEV_…}` names the reader has to fill in, in `.env`.
    pub secrets: Vec<String>,
    /// Services the project declared that could not be carried, with why.
    pub skipped: Vec<String>,
    /// True things a reader would otherwise discover by failing.
    pub notes: Vec<String>,
}

/// The prefix that keeps a placeholder out of the workspace renderer's reach.
const PLACEHOLDER: &str = "DEV_";

/// `mysql-8-4` + `ROOT_PASSWORD` → `DEV_MYSQL_8_4_ROOT_PASSWORD`.
fn placeholder(slug: &str, key: &str) -> String {
    let mut out = String::from(PLACEHOLDER);
    for ch in slug.chars().chain(std::iter::once('_')).chain(key.chars()) {
        out.push(if ch.is_ascii_alphanumeric() {
            ch.to_ascii_uppercase()
        } else {
            '_'
        });
    }
    out
}

/// The variables one instance's fragment may read, answered for a machine that
/// is not this one.
///
/// Deliberately shaped like [`crate::render`]'s own context and deliberately
/// not calling it: that one resolves the keystore and builds absolute paths,
/// which are the two things that must not happen here. Every entry it produces
/// is produced here too, so a fragment that renders there renders here.
fn context(
    instance: &Instance,
    manifest: &crate::pkg::Manifest,
    secrets: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();

    vars.insert("image".into(), manifest.image.reference());
    // Compose's implicit network, which every service in the file already
    // shares. Naming it is what lets the fragment's `aliases:` block stay.
    vars.insert("network".into(), "default".into());
    vars.insert("instance.container".into(), instance.container());
    vars.insert("instance.slug".into(), instance.id.clone());
    vars.insert(
        "instance.aliases".into(),
        format!(
            "[{}]",
            instance
                .aliases()
                .iter()
                .map(|a| format!("\"{a}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    // A volume rather than a bind mount. The host directory is under
    // `~/.stackvo`, and a devcontainer that bind-mounted a path from the
    // author's laptop would fail to start on every other machine.
    vars.insert("instance.logs".into(), format!("{}-logs", instance.id));
    if let Some(url) = &manifest.url {
        // There is no Traefik here, so the name a browser would use is the
        // forwarded port. `localhost` is the honest stand-in; the alternative
        // is a `.loc` name that resolves nowhere on the machine reading this.
        vars.insert(
            "instance.domain".into(),
            instance.domain(&url.subdomain, "localhost"),
        );
    }

    for port in &manifest.ports {
        let chosen = instance
            .ports
            .get(&port.name)
            .copied()
            .unwrap_or(port.preferred);
        vars.insert(format!("port.{}", port.name), chosen.to_string());
    }

    for volume in &manifest.volumes {
        vars.insert(
            format!("volume.{}", volume.name),
            format!("{}-{}", instance.id, volume.name),
        );
    }

    for file in &manifest.files {
        let name = leaf(&file.template);
        vars.insert(
            format!("file.{}", file.name),
            format!("./configs/{}/{name}", instance.id),
        );
    }

    for companion in &manifest.companions {
        vars.insert(
            format!("companion.{}.host", companion.name),
            format!("stackvo-{}-{}", instance.id, companion.name),
        );
        for port in &companion.ports {
            vars.insert(
                format!("companion.{}.port.{}", companion.name, port.name),
                port.container.to_string(),
            );
        }
    }

    for setting in &manifest.settings {
        // A secret leaves as its own name. Everything else leaves as the value
        // this workspace is using, because tuning is a property of the project
        // and a password is a property of the machine.
        if setting.is_secret() {
            let name = placeholder(&instance.id, &setting.key);
            if !secrets.contains(&name) {
                secrets.push(name.clone());
            }
            vars.insert(format!("settings.{}", setting.key), format!("${{{name}}}"));
            continue;
        }
        let value = instance
            .settings
            .get(&setting.key)
            .cloned()
            .or_else(|| setting.default_text())
            .unwrap_or_default();
        vars.insert(format!("settings.{}", setting.key), value);
    }

    vars
}

/// `files/my.cnf` → `my.cnf`.
fn leaf(template: &str) -> &str {
    template.rsplit('/').next().unwrap_or(template)
}

/// Drop the routing labels. There is no router here.
///
/// A fragment written for this workspace carries `traefik.*` labels, and they
/// are not merely inert in a file with no Traefik in it — they name a host
/// (`phpmyadmin.localhost`) that resolves nowhere, and on a team that *does*
/// run Traefik they would quietly start binding a route nobody asked for.
///
/// Found by the probe and not by the fourteen unit tests above, and the reason
/// is worth keeping: those tests read a fragment written here to exercise them,
/// and the real package's fragment has labels. `import_probe` states the same
/// trap from the other side — a parser that reads a fixture its own author
/// wrote agrees with its author. The test for this one uses a fragment copied
/// from the shipped package rather than one invented for it.
///
/// `labels:` goes too when nothing is left under it: a mapping key with no
/// children is null, and Compose refuses the file rather than ignoring it.
fn without_routing(body: &str) -> String {
    let mut out = String::new();
    let mut held: Vec<String> = Vec::new();
    let mut kept = 0usize;
    let mut in_labels = false;

    let flush = |out: &mut String, held: &mut Vec<String>, kept: &mut usize| {
        if *kept > 0 {
            for line in held.iter() {
                out.push_str(line);
                out.push('\n');
            }
        }
        held.clear();
        *kept = 0;
    };

    for line in body.lines() {
        let indented = line.starts_with(' ') || line.starts_with('\t');

        if in_labels && (!indented && !line.trim().is_empty()) {
            in_labels = false;
            flush(&mut out, &mut held, &mut kept);
        }

        if !in_labels && line.trim_end() == "labels:" && !indented {
            in_labels = true;
            held.push(line.to_string());
            continue;
        }

        if in_labels {
            let item = line.trim().trim_start_matches("- ").trim_matches('"');
            if item.starts_with("traefik.") {
                continue;
            }
            if !line.trim().is_empty() {
                kept += 1;
            }
            held.push(line.to_string());
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    flush(&mut out, &mut held, &mut kept);
    out
}

/// Indent a rendered fragment under its service key.
fn indented(key: &str, body: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  {key}:");
    for line in body.lines() {
        if line.trim().is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "    {line}");
        }
    }
    out
}

/// The image a container somebody *develops inside* should be built from.
///
/// PHP is the generator's own file, unchanged: it installs the interpreter, the
/// extensions the manifest names and the web server, and copies no application
/// code — because in StackVo the source arrives through a bind mount. That is
/// exactly a development environment.
///
/// The runtimes are the opposite and it is not a flaw in them: their Dockerfile
/// does `COPY . .`, runs the install and the build, and ends in a `CMD` that
/// starts the application. It is a snapshot, which is the right thing for the
/// thing StackVo runs and the wrong thing to open a terminal in — the source
/// would be a stale copy of the one on screen, and the container would exit
/// whenever the application did. So they get the toolchain and nothing else.
fn dockerfile(manifest: &Manifest, opts: &crate::generator::ToolchainOptions) -> Result<String> {
    if manifest.runtime == "php" {
        return crate::generator::render_from_manifest(manifest, opts, false)
            .map_err(|e| Error::new(Code::InvalidManifest, e));
    }

    let mut out = String::from("# Written by StackVo — the development environment for this\n");
    let _ = writeln!(
        out,
        "# project, as a devcontainer. Project: {}",
        manifest.name
    );
    out.push_str(
        "#\n# The toolchain only. Your source is mounted, not copied, and the\n\
         # dependencies are installed by `postCreateCommand` against that mount.\n\n",
    );

    if manifest.runtime == "node" {
        let node = manifest.node.as_ref().ok_or_else(|| {
            Error::new(Code::InvalidManifest, "runtime is node with no node block")
        })?;
        let _ = writeln!(out, "FROM node:{}-alpine\n", node.version);
        if let Some(pm) = node.package_manager.as_deref() {
            let _ = writeln!(
                out,
                "# The package manager the manifest pins.\nRUN corepack enable {pm}\n"
            );
        }
    } else {
        let lang = manifest.lang.as_ref().ok_or_else(|| {
            Error::new(
                Code::InvalidManifest,
                format!("runtime is {} with no config block", manifest.runtime),
            )
        })?;
        let image = crate::generator::lang_base_image(&manifest.runtime, &lang.version)
            .ok_or_else(|| {
                Error::new(
                    Code::InvalidManifest,
                    format!("{} is not a runtime this can export", manifest.runtime),
                )
            })?;
        let _ = writeln!(out, "FROM {image}\n");
    }

    let _ = writeln!(
        out,
        "WORKDIR {}",
        crate::release::app_path(&manifest.runtime)
    );
    Ok(out)
}

/// The application's own service, and the one place a path is chosen.
fn app_service(manifest: &Manifest, depends: &[String]) -> String {
    let app = crate::release::app_path(&manifest.runtime);
    let mut out =
        String::from("  app:\n    build:\n      context: .\n      dockerfile: Dockerfile\n");

    // `..` from `.devcontainer/` is the repository. Read-write and unfiltered:
    // this is the tree being worked on, not a build context.
    let _ = writeln!(out, "    volumes:\n      - \"..:{app}\"");
    out.push_str("    networks:\n      - default\n");

    if !depends.is_empty() {
        out.push_str("    depends_on:\n");
        for id in depends {
            let _ = writeln!(out, "      - {id}");
        }
    }

    match manifest.runtime.as_str() {
        "php" => {
            // Nothing. The image's own command is the web server, and it is the
            // same one this machine runs.
        }
        "node" => {
            if let Some(node) = &manifest.node {
                let _ = writeln!(
                    out,
                    "    environment:\n      HOST: \"0.0.0.0\"\n      PORT: \"{}\"",
                    node.port
                );
            }
            out.push_str(SLEEP);
        }
        _ => {
            if let Some(lang) = &manifest.lang {
                let _ = writeln!(
                    out,
                    "    environment:\n      HOST: \"0.0.0.0\"\n      PORT: \"{}\"",
                    lang.port
                );
            }
            out.push_str(SLEEP);
        }
    }

    out
}

/// Why a development container has to be told to stay up.
///
/// Every line carries its own four spaces. A `\` continuation in a Rust string
/// eats the *leading whitespace* of the line after it as well as the newline,
/// which put `command:` in column zero — a top-level compose key, and a file
/// Docker refuses. Found by `examples/devcontainer_probe.rs` on its first run,
/// against a document fourteen string-comparing unit tests had called correct.
const SLEEP: &str = concat!(
    "    # The editor attaches to this container; it is not the application.\n",
    "    # Start that yourself once the dependencies are in — see the\n",
    "    # `start` line in stackvo.json.\n",
    "    command: [\"sleep\", \"infinity\"]\n",
);

/// The port the application answers on inside its container.
fn app_port(manifest: &Manifest) -> u16 {
    match manifest.runtime.as_str() {
        "php" => {
            if manifest.server.as_deref() == Some("swoole") {
                8000
            } else {
                80
            }
        }
        "node" => manifest.node.as_ref().map(|n| n.port).unwrap_or(3000),
        _ => manifest.lang.as_ref().map(|l| l.port).unwrap_or(8000),
    }
}

/// The install this project's manifest already names, if it names one.
fn post_create(manifest: &Manifest) -> Option<String> {
    match manifest.runtime.as_str() {
        // Deliberately absent for PHP. `composer install` is the obvious guess
        // and it is a guess: nothing in a manifest says the project uses
        // Composer, and a `postCreateCommand` that fails is a container that
        // reports itself broken on first open. StackVo does not run it on
        // adoption either, for the same reason.
        "php" => None,
        "node" => manifest.node.as_ref().map(|n| n.install.clone()),
        _ => manifest.lang.as_ref().and_then(|l| l.install.clone()),
    }
    .filter(|command| !command.trim().is_empty())
}

/// `devcontainer.json`, as strict JSON.
///
/// The specification permits comments and every reader in practice accepts
/// them. They are still not used: this file is parsed by editors, CI actions
/// and whatever a team has written around it, and a comment is a bet on all of
/// them. What a comment would have said is in the compose file beside it, where
/// YAML makes the same note free.
fn devcontainer_json(manifest: &Manifest) -> String {
    let mut fields: Vec<String> = vec![
        format!("  \"name\": {}", json_string(&manifest.name)),
        "  \"dockerComposeFile\": \"docker-compose.yml\"".into(),
        "  \"service\": \"app\"".into(),
        format!(
            "  \"workspaceFolder\": {}",
            json_string(crate::release::app_path(&manifest.runtime))
        ),
        format!("  \"forwardPorts\": [{}]", app_port(manifest)),
    ];
    if let Some(command) = post_create(manifest) {
        fields.push(format!(
            "  \"postCreateCommand\": {}",
            json_string(&command)
        ));
    }
    format!("{{\n{}\n}}\n", fields.join(",\n"))
}

/// A JSON string literal. `serde_json` for one scalar would be the same bytes.
fn json_string(value: &str) -> String {
    serde_json::Value::String(value.to_string()).to_string()
}

/// Everything an export would write, without writing any of it.
pub fn plan(
    manifest: &Manifest,
    table: &Table,
    catalogue: &dyn Catalogue,
    opts: &crate::generator::ToolchainOptions,
) -> Result<Plan> {
    let mut out = Plan {
        project: manifest.name.clone(),
        ..Default::default()
    };

    out.files.push(File {
        path: "Dockerfile".into(),
        contents: dockerfile(manifest, opts)?,
    });

    // The files that Dockerfile copies. It names them without a directory, so
    // they sit beside it and the build context is `.devcontainer/` itself.
    for (name, contents) in crate::generator::render_project_config_files(manifest) {
        out.files.push(File {
            path: name.to_string(),
            contents,
        });
    }

    let mut blocks = String::new();
    let mut candidates: Vec<String> = Vec::new();
    let mut depends: Vec<String> = Vec::new();

    for service in &manifest.services {
        // The primary, or whichever single version is switched on. A project
        // naming `mysql` on a machine running 8.0 and 8.4 gets the one this
        // workspace treats as the answer to a bare `stackvo-mysql`, which is
        // also the one its `.env` resolves to.
        let Some(instance) = table
            .primary_of(service)
            .or_else(|| table.of_service(service).find(|i| i.enabled))
        else {
            out.skipped.push(format!(
                "{service}: nothing installed for it in this workspace"
            ));
            continue;
        };
        if !instance.enabled {
            out.skipped
                .push(format!("{service}: installed but switched off"));
            continue;
        }

        let who = format!("{}@{}", instance.service, instance.version);
        let Some(package) = catalogue.manifest(&instance.service, &instance.version) else {
            out.skipped
                .push(format!("{who}: its package is not on this machine"));
            continue;
        };

        let vars = context(instance, &package, &mut out.secrets);
        let fragment = crate::render::shipped(catalogue, instance, &package.compose.file, &who)?;
        let body = crate::render::substitute(&fragment, &vars, &who)?;
        // The same gate the workspace renderer runs, for the same reason one
        // level along: this is a downloaded fragment on its way into somebody's
        // repository, where it will be read as something the team wrote.
        crate::compose_policy::check(
            &who,
            &body,
            &crate::render::permitted(&vars, &package.image.reference()),
        )?;
        let mut body = without_routing(&body);
        if let Some(health) = &package.health {
            body.push_str(&crate::render::healthcheck(health));
        }
        blocks.push_str(&indented(&instance.id, &body));
        blocks.push('\n');
        depends.push(instance.id.clone());

        candidates.push(format!("{}-logs", instance.id));
        for volume in &package.volumes {
            candidates.push(format!("{}-{}", instance.id, volume.name));
        }

        for file in &package.files {
            let name = leaf(&file.template);
            let template = crate::render::shipped(catalogue, instance, &file.template, &who)?;
            let contents = crate::render::substitute(&template, &vars, &who)?;
            // A placeholder is expanded by Compose in a compose file and is
            // five literal characters in a `my.cnf`. Refused rather than
            // written wrong — and refused by construction, so it holds for a
            // package nobody here has read.
            if contents.contains(&format!("${{{PLACEHOLDER}")) {
                out.skipped.push(format!(
                    "{who}: {name} holds a secret, and a config file is not a place a \
                     placeholder can be filled in"
                ));
                continue;
            }
            out.files.push(File {
                path: format!("configs/{}/{name}", instance.id),
                contents,
            });
        }

        // A companion is the instance's own — two Kafkas get two Zookeepers —
        // and it is not optional: a broker whose companion is missing is a file
        // that starts and never connects.
        for companion in &package.companions {
            let id = format!("{}-{}", instance.id, companion.name);
            let mut side = vars.clone();
            side.insert("companion.image".into(), companion.image.reference());
            side.insert(
                "companion.instance.container".into(),
                format!("stackvo-{id}"),
            );
            side.insert("companion.instance.slug".into(), id.clone());
            side.insert(
                "companion.instance.aliases".into(),
                format!("[\"stackvo-{id}\"]"),
            );
            side.insert("companion.instance.logs".into(), format!("{id}-logs"));
            for port in &companion.ports {
                side.insert(
                    format!("companion.port.{}", port.name),
                    port.preferred.to_string(),
                );
            }
            for volume in &companion.volumes {
                let name = format!("{id}-{}", volume.name);
                side.insert(format!("companion.volume.{}", volume.name), name.clone());
                candidates.push(name);
            }
            candidates.push(format!("{id}-logs"));

            let fragment =
                crate::render::shipped(catalogue, instance, &companion.compose.file, &who)?;
            let body = crate::render::substitute(&fragment, &side, &who)?;
            crate::compose_policy::check(
                &who,
                &body,
                &crate::render::permitted(&side, &companion.image.reference()),
            )?;
            let mut body = without_routing(&body);
            if let Some(health) = &companion.health {
                body.push_str(&crate::render::healthcheck(health));
            }
            blocks.push_str(&indented(&id, &body));
            blocks.push('\n');
        }
    }

    let mut compose = String::from(COMPOSE_HEADER);
    compose.push_str("services:\n\n");
    compose.push_str(&app_service(manifest, &depends));
    compose.push('\n');
    compose.push_str(&blocks);

    // Declared only when something refers to them. A candidate is derived from
    // what a package *could* mount; whether its fragment does is the fragment's
    // business, and an unused top-level volume is a volume Compose creates on
    // every machine that opens this repository.
    let mut volumes: Vec<&String> = candidates
        .iter()
        .filter(|name| compose.contains(name.as_str()))
        .collect();
    volumes.sort();
    volumes.dedup();
    if !volumes.is_empty() {
        compose.push_str("volumes:\n");
        for name in volumes {
            let _ = writeln!(compose, "  {name}:");
        }
    }

    out.files.push(File {
        path: "docker-compose.yml".into(),
        contents: compose,
    });
    out.files.push(File {
        path: "devcontainer.json".into(),
        contents: devcontainer_json(manifest),
    });

    if !out.secrets.is_empty() {
        out.secrets.sort();
        out.secrets.dedup();
        let mut env = String::from(
            "# Copy this to `.env` and fill it in. Compose reads that file from\n\
             # beside the compose file; `.gitignore` keeps it out of the repository.\n\n",
        );
        for name in &out.secrets {
            let _ = writeln!(env, "{name}=");
        }
        out.files.push(File {
            path: ".env.example".into(),
            contents: env,
        });
        out.files.push(File {
            path: ".gitignore".into(),
            contents: "# The passwords this directory deliberately does not carry.\n.env\n"
                .to_string(),
        });
    }

    out.notes.push(
        "The domain and its certificate stay behind: they are this machine's, issued by a CA \
         in its trust stores and routed by a Traefik that is not in this file. The application \
         is reached on the forwarded port instead."
            .into(),
    );
    if !out.secrets.is_empty() {
        out.notes.push(format!(
            "{} password(s) leave as names rather than values. Fill them in in `.env`.",
            out.secrets.len()
        ));
    }
    if manifest.runtime == "php" {
        out.notes.push(
            "Dependencies are not installed for you. Nothing in a manifest says this project \
             uses Composer, and a first-open command that fails is worse than one that is \
             absent."
                .into(),
        );
    }
    out.notes.push(
        "Published service ports are the ones this workspace allocated, so the database client \
         already pointed at them keeps working."
            .into(),
    );

    Ok(out)
}

const COMPOSE_HEADER: &str = "\
# Written by StackVo. Regenerate it rather than editing it.
#
# The container names are kept exactly as this workspace uses them
# (`stackvo-mysql-8-4`), which looks odd in a repository that has nothing to do
# with StackVo and is the only answer that works: this project's own `.env`
# names those hosts, and renaming them here breaks the application on the
# machine that has no way to know why.
#
# Ports published here are the ones the author's workspace allocated. In a
# Codespace they are forwarded; on a laptop already running this stack they
# will collide, and the fix is to change them here.

";

/// Write the plan into `<project>/.devcontainer/`.
///
/// Every file, every time. There is no merge: these are rendered outputs (ADR
/// 0002), and a hand-edit that survived a regeneration would be a difference
/// between this file and the manifest that nothing reports.
pub fn write(project_dir: &Path, plan: &Plan) -> Result<Vec<String>> {
    if !project_dir.is_dir() {
        return Err(Error::not_found(format!(
            "directory {}",
            project_dir.display()
        )));
    }

    let root = project_dir.join(DIR);
    let mut written = Vec::with_capacity(plan.files.len());
    for file in &plan.files {
        let path = root.join(&file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
        }
        crate::atomic::write(&path, &file.contents)?;
        written.push(format!("{DIR}/{}", file.path));
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::PackageRef;
    use crate::pkg;
    use std::collections::BTreeMap;

    /// The mysql package's own fragment, near enough that what it exercises is
    /// what the real one exercises: an image, a container name, a secret and a
    /// non-secret setting, a data volume, a config-file mount, a log mount, a
    /// published port and the network block with aliases.
    const FRAGMENT: &str = "\
image: \"{{ image }}\"
container_name: \"{{ instance.container }}\"
environment:
  MYSQL_DATABASE: \"{{ settings.DATABASE }}\"
  MYSQL_ROOT_PASSWORD: \"{{ settings.ROOT_PASSWORD }}\"
volumes:
  - \"{{ volume.data }}:/var/lib/mysql\"
  - \"{{ file.my_cnf }}:/etc/my.cnf:ro\"
  - \"{{ instance.logs }}:/var/log/mysql\"
ports:
  - \"{{ port.main }}:3306\"
networks:
  {{ network }}:
    aliases: {{ instance.aliases }}
labels:
  - \"traefik.enable=true\"
  - \"traefik.http.routers.{{ instance.slug }}.rule=Host(`{{ instance.domain }}`)\"
  - \"traefik.http.services.{{ instance.slug }}.loadbalancer.server.port=80\"
";

    const CNF: &str = "port = {{ port.main }}\n";

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-dc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn plant(root: &Path, cnf: &str) {
        let dir = root.join("packages/databases/mysql/versions/8.4");
        std::fs::create_dir_all(dir.join("files")).unwrap();
        std::fs::write(dir.join("compose.yml.tpl"), FRAGMENT).unwrap();
        std::fs::write(dir.join("files/my.cnf.tpl"), cnf).unwrap();
        let manifest = format!(
            r#"{{"apiVersion": "{}", "service": "mysql", "version": "8.4",
                "image": {{"repository": "mysql", "tag": "8.4"}},
                "instancing": {{"multiple": true}},
                "ports": [{{"name": "main", "container": 3306, "preferred": 3306, "primary": true}}],
                "volumes": [{{"name": "data", "container": "/var/lib/mysql"}}],
                "files": [{{"name": "my_cnf", "template": "files/my.cnf.tpl",
                            "target": "/etc/my.cnf", "sha256": "{}"}}],
                "settings": [{{"key": "DATABASE", "type": "string", "default": "stackvo"}},
                             {{"key": "ROOT_PASSWORD", "type": "secret", "default": "root"}}],
                "url": {{"subdomain": "mysql", "port": "main"}},
                "compose": {{"file": "compose.yml.tpl", "sha256": "{}"}},
                "support": {{"status": "supported"}}}}"#,
            pkg::API_VERSION,
            pkg::sha256_hex(cnf.as_bytes()),
            pkg::sha256_hex(FRAGMENT.as_bytes())
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(
            root.join("packages/databases/mysql/package.json"),
            format!(
                r#"{{"apiVersion": "{}", "service": "mysql", "category": "databases",
                    "name": {{"en": "MySQL"}}, "recommendedVersion": "8.4"}}"#,
                pkg::API_VERSION
            ),
        )
        .unwrap();
    }

    fn table() -> Table {
        let mut table = Table::default();
        table
            .insert(Instance {
                id: "mysql-8-4".into(),
                service: "mysql".into(),
                version: "8.4".into(),
                package: PackageRef {
                    source: "official".into(),
                    sha256: "0".repeat(64),
                    installed_at: "2026-08-23T09:00:00Z".into(),
                },
                enabled: true,
                primary: true,
                // The number this workspace actually allocated, which is the
                // one the author's database client is pointed at.
                ports: BTreeMap::from([("main".to_string(), 33061u16)]),
                volumes: BTreeMap::new(),
                settings: BTreeMap::new(),
                // Never read here. If it were, the value would be in a file
                // headed for a repository.
                secret_refs: BTreeMap::from([(
                    "ROOT_PASSWORD".to_string(),
                    "keychain:stackvo/mysql-8-4/ROOT_PASSWORD".to_string(),
                )]),
            })
            .unwrap();
        table
    }

    fn php(services: &[&str]) -> Manifest {
        crate::manifest::normalize_spec(
            &serde_json::json!({
                "name": "shop",
                "domain": "shop.loc",
                "runtime": "php",
                "server": "nginx",
                "document_root": "public",
                "services": services,
                "php": { "version": "8.4", "extensions": ["pdo_mysql"] },
            }),
            "shop",
        )
    }

    fn node() -> Manifest {
        crate::manifest::normalize_spec(
            &serde_json::json!({
                "name": "site",
                "domain": "site.loc",
                "runtime": "node",
                "node": { "version": "22", "install": "npm ci", "start": "npm run dev", "port": 3000 },
            }),
            "site",
        )
    }

    fn opts() -> crate::generator::ToolchainOptions {
        crate::generator::ToolchainOptions {
            tools: vec![],
            apt_packages: vec![],
            composer_version: "latest".into(),
            nodejs_version: "20".into(),
        }
    }

    fn file<'a>(plan: &'a Plan, path: &str) -> &'a str {
        &plan
            .files
            .iter()
            .find(|f| f.path == path)
            .unwrap_or_else(|| panic!("no {path} in {:?}", plan.files.iter().map(|f| &f.path)))
            .contents
    }

    fn planned(dir: &Path, manifest: &Manifest) -> Plan {
        let tree = pkg::Tree::open(dir).unwrap();
        plan(manifest, &table(), &tree, &opts()).unwrap()
    }

    /// The one that makes the placeholder work, and the one somebody could
    /// break from another file entirely.
    ///
    /// `template::PREFIXES` is the list the workspace renderer substitutes. A
    /// placeholder whose name matched it would be replaced on the way out —
    /// with an empty string, silently — leaving a compose file in a repository
    /// with no password in it and no error anywhere.
    #[test]
    fn the_placeholder_prefix_is_one_the_workspace_renderer_leaves_alone() {
        assert!(!crate::template::is_substituted(&placeholder(
            "mysql-8-4",
            "ROOT_PASSWORD"
        )));
        // And the reason it is not a shorter prefix.
        for prefix in crate::template::PREFIXES {
            assert!(
                !PLACEHOLDER.starts_with(prefix),
                "{PLACEHOLDER} collides with {prefix}"
            );
        }
    }

    #[test]
    fn a_secret_leaves_as_a_name_and_the_value_never_leaves_at_all() {
        let dir = scratch("secret");
        plant(&dir, CNF);
        let plan = planned(&dir, &php(&["mysql"]));

        let compose = file(&plan, "docker-compose.yml");
        assert!(
            compose.contains("${DEV_MYSQL_8_4_ROOT_PASSWORD}"),
            "{compose}"
        );
        // The manifest's own default is a value too, and it is the one that
        // would arrive if the secret branch were ever skipped.
        assert!(!compose.contains("MYSQL_ROOT_PASSWORD: \"root\""));

        assert_eq!(plan.secrets, vec!["DEV_MYSQL_8_4_ROOT_PASSWORD"]);
        assert!(file(&plan, ".env.example").contains("DEV_MYSQL_8_4_ROOT_PASSWORD="));
        // The file the reader is about to create is the file that would undo
        // the whole arrangement.
        assert_eq!(file(&plan, ".gitignore").lines().last(), Some(".env"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_non_secret_setting_leaves_as_its_value() {
        // Tuning is a property of the project and travels; a password is a
        // property of the machine and does not.
        let dir = scratch("setting");
        plant(&dir, CNF);
        let plan = planned(&dir, &php(&["mysql"]));
        assert!(file(&plan, "docker-compose.yml").contains("MYSQL_DATABASE: \"stackvo\""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_in_the_output_names_this_machine() {
        let dir = scratch("portable");
        plant(&dir, CNF);
        let plan = planned(&dir, &php(&["mysql"]));

        for f in &plan.files {
            for needle in [
                // The five couplings in the workspace's own compose.
                "stackvo-net",
                "traefik",
                "/generated/",
                "logs/services",
                "/Users/",
                "C:\\",
            ] {
                assert!(
                    !f.contents.contains(needle),
                    "{} carries `{needle}`:\n{}",
                    f.path,
                    f.contents
                );
            }
            for line in f.contents.lines() {
                let trimmed = line.trim_start_matches([' ', '-', '"']);
                assert!(
                    !trimmed.starts_with('/') && !trimmed.starts_with('~'),
                    "{} mounts an absolute path: {line}",
                    f.path
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_container_names_are_kept_so_the_projects_own_env_still_resolves() {
        // `DB_HOST=stackvo-mysql-8-4` is in the project's .env, and renaming
        // the service here breaks the application on the machine that has no
        // way to know why.
        let dir = scratch("names");
        plant(&dir, CNF);
        let compose = file(&planned(&dir, &php(&["mysql"])), "docker-compose.yml").to_string();
        assert!(compose.contains("container_name: \"stackvo-mysql-8-4\""));
        assert!(compose.contains("\"stackvo-mysql-8-4\", \"stackvo-mysql\""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_published_port_is_the_one_this_workspace_allocated() {
        let dir = scratch("ports");
        plant(&dir, CNF);
        let compose = file(&planned(&dir, &php(&["mysql"])), "docker-compose.yml").to_string();
        assert!(compose.contains("\"33061:3306\""), "{compose}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_config_file_travels_and_a_secret_one_is_refused() {
        let dir = scratch("configs");
        plant(&dir, CNF);
        let plan = planned(&dir, &php(&["mysql"]));
        assert_eq!(
            file(&plan, "configs/mysql-8-4/my.cnf.tpl"),
            "port = 33061\n"
        );
        assert!(plan.skipped.is_empty());
        let _ = std::fs::remove_dir_all(&dir);

        // The same package, with a template that reads the secret. `${…}` is
        // expanded by Compose in a compose file and is five literal characters
        // in a `my.cnf`, so it is refused rather than written wrong.
        let dir = scratch("configs-secret");
        plant(&dir, "password = {{ settings.ROOT_PASSWORD }}\n");
        let plan = planned(&dir, &php(&["mysql"]));
        assert!(!plan.files.iter().any(|f| f.path.starts_with("configs/")));
        assert!(
            plan.skipped.iter().any(|s| s.contains("my.cnf.tpl")),
            "{:?}",
            plan.skipped
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The one the fourteen tests above missed, and why.
    ///
    /// They read a fragment written here; the shipped packages carry
    /// `traefik.*` labels, so `nothing_in_the_output_names_this_machine` was
    /// asserting the absence of something its own fixture never had. The
    /// fixture now carries the labels, copied from the phpmyadmin package.
    #[test]
    fn the_routing_labels_do_not_travel() {
        let dir = scratch("routing");
        plant(&dir, CNF);
        let compose = file(&planned(&dir, &php(&["mysql"])), "docker-compose.yml").to_string();
        assert!(!compose.contains("traefik"), "{compose}");
        // And the key they were under goes with them: a mapping key with no
        // children is null, and Compose refuses the file rather than ignoring
        // it.
        assert!(!compose.contains("labels:"), "{compose}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_label_that_is_not_routing_keeps_its_key() {
        let body =
            "image: \"x\"\nlabels:\n  - \"traefik.enable=true\"\n  - \"com.example.owner=ali\"\n";
        let out = without_routing(body);
        assert!(out.contains("labels:"), "{out}");
        assert!(out.contains("com.example.owner=ali"));
        assert!(!out.contains("traefik"));
    }

    #[test]
    fn what_follows_a_labels_block_is_not_swallowed_with_it() {
        // The filter buffers the block to find out whether to emit it, so the
        // first line at column zero after it has to end the buffering — or a
        // dropped `labels:` would take the rest of the fragment with it.
        let body = "labels:\n  - \"traefik.enable=true\"\nports:\n  - \"80:80\"\n";
        let out = without_routing(body);
        assert!(!out.contains("labels"), "{out}");
        assert!(out.contains("ports:"), "{out}");
        assert!(out.contains("\"80:80\""), "{out}");
    }

    #[test]
    fn a_volume_is_declared_only_when_something_mounts_it() {
        let dir = scratch("volumes");
        plant(&dir, CNF);
        let compose = file(&planned(&dir, &php(&["mysql"])), "docker-compose.yml").to_string();
        assert!(compose.contains("\n  mysql-8-4-data:\n"), "{compose}");
        assert!(compose.contains("\n  mysql-8-4-logs:\n"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_declared_service_this_machine_does_not_have_is_named_rather_than_dropped() {
        let dir = scratch("missing");
        plant(&dir, CNF);
        let plan = planned(&dir, &php(&["mysql", "redis"]));
        assert!(
            plan.skipped.iter().any(|s| s.starts_with("redis:")),
            "{:?}",
            plan.skipped
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_php_dockerfile_is_the_generators_own_and_not_a_second_one() {
        // The whole value of the export is that the container is the one
        // StackVo builds. A renderer of its own would drift from it in the week
        // nobody is looking.
        let dir = scratch("dockerfile");
        plant(&dir, CNF);
        let manifest = php(&[]);
        let plan = planned(&dir, &manifest);
        assert_eq!(
            file(&plan, "Dockerfile"),
            crate::generator::render_from_manifest(&manifest, &opts(), false).unwrap()
        );
        // And the files that Dockerfile copies travel beside it.
        assert!(file(&plan, "Dockerfile").contains("COPY nginx.conf"));
        assert!(plan.files.iter().any(|f| f.path == "nginx.conf"));
        assert!(plan.files.iter().any(|f| f.path == "supervisord.conf"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_runtime_gets_the_toolchain_and_not_the_application() {
        let dir = scratch("node");
        plant(&dir, CNF);
        let plan = planned(&dir, &node());

        let dockerfile = file(&plan, "Dockerfile");
        assert!(dockerfile.contains("FROM node:22-alpine"));
        // The three things that make the dev image a snapshot rather than an
        // environment, and the reason a runtime cannot reuse it.
        assert!(!dockerfile.contains("COPY . ."), "{dockerfile}");
        assert!(!dockerfile.contains("CMD"), "{dockerfile}");
        assert!(!dockerfile.contains("RUN npm ci"), "{dockerfile}");

        // The install moves to where it runs against the mount.
        assert!(file(&plan, "devcontainer.json").contains("\"postCreateCommand\": \"npm ci\""));
        // And the container has to be told to stay up, because the editor
        // attaches to it and the application is not its main process.
        assert!(file(&plan, "docker-compose.yml").contains("sleep"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn php_is_not_told_to_install_anything() {
        // Nothing in a manifest says the project uses Composer, and a
        // first-open command that fails is a container that reports itself
        // broken before anybody has typed anything.
        let dir = scratch("php-postcreate");
        plant(&dir, CNF);
        let json = file(&planned(&dir, &php(&[])), "devcontainer.json").to_string();
        assert!(!json.contains("postCreateCommand"), "{json}");
        assert!(json.contains("\"workspaceFolder\": \"/var/www/html\""));
        assert!(json.contains("\"forwardPorts\": [80]"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_devcontainer_json_is_strict_json() {
        // The specification permits comments and every reader in practice
        // accepts them; this file is still parsed by editors, CI actions and
        // whatever a team wrote around it, and a comment is a bet on all of
        // them.
        let dir = scratch("json");
        plant(&dir, CNF);
        let text = file(&planned(&dir, &php(&["mysql"])), "devcontainer.json").to_string();
        let value: serde_json::Value = serde_json::from_str(&text).expect("strict JSON");
        assert_eq!(value["service"], "app");
        assert_eq!(value["dockerComposeFile"], "docker-compose.yml");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_app_waits_for_what_it_declared() {
        let dir = scratch("depends");
        plant(&dir, CNF);
        let compose = file(&planned(&dir, &php(&["mysql"])), "docker-compose.yml").to_string();
        let app = compose.split("\n  mysql-8-4:").next().unwrap();
        assert!(app.contains("depends_on:"), "{app}");
        assert!(app.contains("- mysql-8-4"), "{app}");
        assert!(app.contains("\"..:/var/www/html\""), "{app}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
