//! Boost and `laravel/mcp`: registering the MCP server that lives *inside* the
//! container.
//!
//! ## This is a break, not a gap
//!
//! `php artisan boost:install` writes a `.mcp.json` into the project and puts
//! **`php artisan boost:mcp`** in it — a command line that assumes a `php` on
//! the host. There is no php on this host and there will not be one:
//! `tooling.rs` records that as a decision (*"a version is a property of a
//! project, not of a directory a shim guessed"*). So Laravel's own installer
//! produces a configuration that cannot start here, and the person who notices
//! sees the failure in their assistant rather than in the tool that wrote it.
//!
//! The repair is not a new rule — [`crate::agents`] already has the three that
//! matter (read, insert one key, write back; never rewrite a file that does not
//! parse; keep a backup) and this module borrows all of them. What is missing
//! is the **command**: not `php artisan boost:mcp` but the passage into the
//! container this application already owns.
//!
//! ## Two servers side by side, and that is not a collision
//!
//! `stackvo-mcp` answers *"why will `shop.loc` not open"* — preflight, hosts,
//! certificate SANs, container logs. Boost answers *"what is in the `users`
//! table"* — schema, route list, tinker, the `artisan` inventory, version-aware
//! documentation search. Neither can answer the other's question, so both are
//! registered and neither replaces the other. [`crate::agents::ENTRY`] is a
//! *machine-wide* registration of this app's own server; everything here is
//! **project-scoped**, in files that live in the project directory.
//!
//! ## What is measured, and what is read from the project's own files
//!
//! Nothing here is guessed from a framework's defaults:
//!
//! | Fact | Where it comes from |
//! | --- | --- |
//! | Is `laravel/boost` / `laravel/mcp` / `laravel/ai` installed | `composer.lock`, already parsed by [`crate::deps`] |
//! | Which MCP servers this project publishes | the project's own `routes/ai.php` — the `Mcp::local()` and `Mcp::web()` lines in it |
//! | What is registered today | the project's own `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json` |
//!
//! A project with no `routes/ai.php` yields no `laravel/mcp` servers rather
//! than a default handle, because a handle this module invented would produce
//! `artisan mcp:start <something>` failing in somebody's assistant.
//!
//! ## The HTTP half needs nothing new
//!
//! `Mcp::web()` registers an **ordinary route in the application**, so it is
//! already served by the project's own web container on the project's own
//! domain over the certificate the browser already trusts. This is the same
//! answer `worker.rs` reached for Reverb — a path on the existing host — except
//! that here there is not even a label to write: the route is inside the app.
//! So this module reports the URL and stops. No new certificate, no hosts
//! entry, no second router.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The three packages this module has something to say about.
///
/// `laravel/ai` is here because it is the reason the other two get installed —
/// and because item 8's leak pattern watches the key it makes people put in
/// `.env`. Reported, never required.
pub const PACKAGES: [&str; 3] = ["laravel/boost", "laravel/mcp", "laravel/ai"];

/// Where the project's own `Mcp::` registrations live.
pub const ROUTES_AI: &str = "routes/ai.php";

/// Which of the three the lock file names, with the version it names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Packages {
    pub boost: Option<String>,
    pub mcp: Option<String>,
    pub ai: Option<String>,
}

impl Packages {
    pub fn any(&self) -> bool {
        self.boost.is_some() || self.mcp.is_some() || self.ai.is_some()
    }
}

/// Read the three out of a dependency set [`crate::deps`] has already parsed.
///
/// Taking the parsed set rather than the file means this cannot disagree with
/// the dependency report about what is installed — the failure mode
/// `read_dependencies` exists to prevent, applied once more.
pub fn packages(deps: &[crate::deps::Dep]) -> Packages {
    let find = |name: &str| {
        deps.iter()
            .find(|d| d.ecosystem == crate::deps::Ecosystem::Packagist && d.name == name)
            .map(|d| d.version.clone())
    };
    Packages {
        boost: find("laravel/boost"),
        mcp: find("laravel/mcp"),
        ai: find("laravel/ai"),
    }
}

// ------------------------------------------------------------------ servers

/// One MCP server this project can serve, and how it is reached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Server {
    /// Boost's own server: `artisan boost:mcp`, over stdio.
    Boost,
    /// A server the project registered with `Mcp::local('<handle>', …)`, run as
    /// `artisan mcp:start <handle>`, over stdio.
    Local { handle: String },
    /// A server the project registered with `Mcp::web('<path>', …)`. Not a
    /// process to start: a route inside the application, already served.
    Web { path: String },
}

impl Server {
    /// A stable id for the UI to send back, and for a config entry's name.
    pub fn id(&self) -> String {
        match self {
            Server::Boost => "boost".to_string(),
            Server::Local { handle } => format!("local:{handle}"),
            Server::Web { path } => format!("web:{path}"),
        }
    }

    pub fn parse_id(id: &str) -> Option<Server> {
        match id {
            "boost" => Some(Server::Boost),
            _ => {
                if let Some(handle) = id.strip_prefix("local:") {
                    (!handle.is_empty()).then(|| Server::Local {
                        handle: handle.to_string(),
                    })
                } else {
                    id.strip_prefix("web:")
                        .filter(|p| !p.is_empty())
                        .map(|p| Server::Web {
                            path: p.to_string(),
                        })
                }
            }
        }
    }

    /// The artisan words that start it, when it is a process at all.
    ///
    /// `None` for [`Server::Web`], which is the whole point of that variant:
    /// nothing starts it, because the application already serves it.
    pub fn artisan(&self) -> Option<Vec<String>> {
        match self {
            Server::Boost => Some(vec!["artisan".into(), "boost:mcp".into()]),
            Server::Local { handle } => {
                Some(vec!["artisan".into(), "mcp:start".into(), handle.clone()])
            }
            Server::Web { .. } => None,
        }
    }

    /// The name this app files the server under when it has to create an entry.
    ///
    /// Only used when nothing is registered yet — an entry that already runs
    /// this server keeps **its own** name, because renaming somebody's server
    /// is a second server in their client, not a repair of the first.
    pub fn default_entry(&self) -> String {
        match self {
            Server::Boost => "laravel-boost".to_string(),
            Server::Local { handle } => format!("laravel-{handle}"),
            Server::Web { .. } => "laravel-mcp".to_string(),
        }
    }
}

/// Every MCP server this project publishes, from the packages it has and the
/// routes file it wrote.
///
/// `routes_ai` is the text of `routes/ai.php` when the project has one. Absent
/// or unreadable yields no `laravel/mcp` servers — see the module comment on
/// why a default handle is not invented.
pub fn servers(packages: &Packages, routes_ai: Option<&str>) -> Vec<Server> {
    let mut out = Vec::new();
    if packages.boost.is_some() {
        out.push(Server::Boost);
    }
    if packages.mcp.is_some() {
        if let Some(text) = routes_ai {
            out.extend(parse_routes(text));
        }
    }
    out
}

/// The `Mcp::local()` and `Mcp::web()` registrations in a `routes/ai.php`.
///
/// A deliberately small reader: it finds `Mcp::local(` or `Mcp::web(` and takes
/// the **first quoted string** after it. That is the argument both forms carry
/// first — a handle for `local`, a URI path for `web` — and reading it out of
/// the project's own file is the difference between reporting what this project
/// registered and reciting what a tutorial usually shows.
///
/// It is not a PHP parser and does not pretend to be. A registration whose
/// first argument is a constant or a variable is **skipped**, not guessed at:
/// there is no string to read, and inventing one would produce a command line
/// that fails in somebody's assistant.
pub fn parse_routes(text: &str) -> Vec<Server> {
    let mut out = Vec::new();

    for (marker, web) in [("Mcp::local(", false), ("Mcp::web(", true)] {
        let mut rest = text;
        while let Some(at) = rest.find(marker) {
            rest = &rest[at + marker.len()..];
            // Only up to the closing paren of this call: a `local(` whose first
            // argument is a class constant must not reach forward into the
            // *next* call's string literal and register a handle twice.
            let call = rest.split(')').next().unwrap_or("");
            if let Some(literal) = first_literal(call) {
                out.push(if web {
                    Server::Web { path: literal }
                } else {
                    Server::Local { handle: literal }
                });
            }
        }
    }

    // Two identical registrations in one file are one server. Sorted so the
    // list does not depend on which marker was scanned first.
    out.sort_by_key(|s| s.id());
    out.dedup();
    out
}

/// The first `'…'` or `"…"` in a fragment, when it starts one before anything
/// else does.
fn first_literal(fragment: &str) -> Option<String> {
    let head = fragment.trim_start();
    let quote = head.chars().next().filter(|c| *c == '\'' || *c == '"')?;
    let body = &head[quote.len_utf8()..];
    let end = body.find(quote)?;
    let literal = &body[..end];
    (!literal.is_empty()).then(|| literal.to_string())
}

// ------------------------------------------------------------- the command

/// The command line that reaches a server **inside the project's container**.
///
/// `docker exec` rather than `stackvo artisan`, and the reason is the working
/// directory. The CLI resolves which project it means from the directory it was
/// started in, and an assistant starts its servers from wherever it happens to
/// be — so the passage that names the container is the one that cannot pick the
/// wrong project. It is also the passage that exists on every machine this runs
/// on: `docker` is already a hard requirement, and the `stackvo` binary is not
/// necessarily on `PATH`.
///
/// `-i` and no `-t`: MCP over stdio is a pipe, and a TTY would put line
/// discipline in the middle of a JSON-RPC stream.
pub fn argv(container: &str, artisan: &[String]) -> Vec<String> {
    let mut out = vec![
        "exec".to_string(),
        "-i".to_string(),
        container.to_string(),
        "php".to_string(),
    ];
    out.extend(artisan.iter().cloned());
    out
}

/// The whole line, `docker` included — what goes into a config file's
/// `command` plus `args`, and what a person can paste into a terminal.
pub fn command_line(container: &str, artisan: &[String]) -> Vec<String> {
    let mut out = vec![DOCKER.to_string()];
    out.extend(argv(container, artisan));
    out
}

/// The program every one of these lines starts with.
///
/// Bare rather than an absolute path: this is what the rest of the application
/// runs too, and an absolute path baked into somebody's editor configuration is
/// a registration that breaks when Docker Desktop moves.
pub const DOCKER: &str = "docker";

// ------------------------------------------------------- the project's files

/// One configuration file that lives **in the project**, not in a home
/// directory.
pub struct File {
    pub id: &'static str,
    /// Relative to the project root, in POSIX spelling.
    pub path: &'static str,
    pub shape: crate::agents::Shape,
    /// Shown in the pane. Product names, so not translated.
    pub label: &'static str,
}

/// The three a project-scoped registration can land in.
///
/// `.mcp.json` is the file `boost:install` writes and the one Claude Code reads
/// per project; the other two are the per-project files their editors read.
/// Everything machine-wide stays in [`crate::agents`] — a server that only
/// exists while this project's container is up does not belong in a file that
/// applies to every directory on the machine.
pub const FILES: &[File] = &[
    File {
        id: "mcp",
        path: ".mcp.json",
        shape: crate::agents::Shape::McpServers,
        label: "Claude Code",
    },
    File {
        id: "cursor",
        path: ".cursor/mcp.json",
        shape: crate::agents::Shape::McpServers,
        label: "Cursor",
    },
    File {
        id: "vscode",
        path: ".vscode/mcp.json",
        shape: crate::agents::Shape::VsCode,
        label: "VS Code",
    },
];

pub fn file(id: &str) -> Option<&'static File> {
    FILES.iter().find(|f| f.id == id)
}

/// What one file says about one server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum State {
    /// The file is not there. Nothing is wrong; nothing is registered either.
    Absent,
    /// The file is there and nothing in it runs this server.
    Unregistered,
    /// Registered, and the command is the one this app would write.
    Container,
    /// Registered, and the command starts a `php` **on this machine**. This is
    /// what `boost:install` writes, and it is the break this module exists for.
    HostPhp,
    /// Registered, and the command is neither of those. Reported and left
    /// alone: somebody has wired this deliberately and this app is not in a
    /// position to say they are wrong.
    Other,
    /// The file does not parse. Never rewritten — [`crate::agents`] carries the
    /// reasoning, and it is the same file, the same rule.
    Unparseable,
}

/// One file's answer about one server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStatus {
    pub id: String,
    pub label: String,
    /// Project-relative, so the pane can show where it would write.
    pub path: String,
    pub state: State,
    /// The entry name the registration is filed under, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    /// The command line as it stands today, joined for display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// The entry in `text` that runs `artisan`, and what its command line is.
///
/// Matched on **what it runs**, not on what it is called: `boost:install` files
/// its server under a name of its own choosing and a person may have renamed
/// it, but an entry whose arguments say `boost:mcp` is that server whatever it
/// is called. Returns the entry's name and its full argv.
pub fn find_entry(
    text: &str,
    shape: crate::agents::Shape,
    artisan: &[String],
) -> Option<(String, Vec<String>)> {
    let verb = artisan.get(1)?;
    let document: serde_json::Value = serde_json::from_str(text).ok()?;
    let servers = document.get(shape.key())?.as_object()?;

    for (name, entry) in servers {
        let command = entry.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let args: Vec<String> = entry
            .get("args")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        if args.iter().any(|a| a == verb) {
            let mut argv = vec![command.to_string()];
            argv.extend(args);
            return Some((name.clone(), argv));
        }
    }
    None
}

/// Does this command line start a PHP on the machine this is running on?
///
/// The basename, so `/usr/bin/php`, `php`, `php8.3` and `/opt/homebrew/bin/php`
/// all answer yes and `docker` answers no. Deliberately narrow: the question is
/// not "is this command wrong", it is "is this the thing `boost:install`
/// writes", and a command this does not recognise is reported as
/// [`State::Other`] rather than diagnosed.
pub fn runs_on_the_host(command: &str) -> bool {
    let base = command
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(command)
        .trim_end_matches(".exe");
    base == "php"
        || base.starts_with("php") && base[3..].chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Read every project file's state for one server. Never writes.
pub fn status(project_dir: &Path, container: &str, server: &Server) -> Vec<FileStatus> {
    let Some(artisan) = server.artisan() else {
        return Vec::new();
    };
    let wanted = command_line(container, &artisan);

    FILES
        .iter()
        .map(|file| {
            let path = project_dir.join(file.path);
            let text = std::fs::read_to_string(&path).ok();

            let (state, entry, command) = match text {
                None => (State::Absent, None, None),
                Some(text) if crate::agents::document(&text).is_err() => {
                    (State::Unparseable, None, None)
                }
                Some(text) => match find_entry(&text, file.shape, &artisan) {
                    None => (State::Unregistered, None, None),
                    Some((name, argv)) => {
                        let state = if argv == wanted {
                            State::Container
                        } else if runs_on_the_host(&argv[0]) {
                            State::HostPhp
                        } else {
                            State::Other
                        };
                        (state, Some(name), Some(argv.join(" ")))
                    }
                },
            };

            FileStatus {
                id: file.id.to_string(),
                label: file.label.to_string(),
                path: file.path.to_string(),
                state,
                entry,
                command,
            }
        })
        .collect()
}

// ------------------------------------------------------------------ report

/// One server, everything this app can say about it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    /// [`Server::id`] — what the pane sends back to register it.
    pub id: String,
    pub server: Server,
    /// The whole line, joined. `None` for a route: nothing starts it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Where a `Mcp::web()` route already answers, on the project's own domain.
    ///
    /// No new certificate, no hosts entry, no second router — the route is
    /// inside the application, so the address the browser already trusts is the
    /// address the server is on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Empty for a route, for the same reason `command` is `None`.
    pub files: Vec<FileStatus>,
}

/// What this project's MCP situation is.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub packages: Packages,
    /// The container a registration would name.
    pub container: String,
    /// Whether the project has a `routes/ai.php` at all.
    ///
    /// Reported separately from the server list so "no servers" can be told
    /// apart from "the file this reads is not there" — the same distinction
    /// [`crate::deps::Report::locks`] draws, and for the same reason.
    pub has_routes: bool,
    pub servers: Vec<ServerStatus>,
}

/// Assemble it. Reads three of the project's own files and nothing else.
pub fn report(
    project_dir: &Path,
    container: &str,
    domain: Option<&str>,
    deps: &[crate::deps::Dep],
) -> Report {
    let packages = packages(deps);
    let routes_path = project_dir.join(ROUTES_AI);
    let routes = std::fs::read_to_string(&routes_path).ok();

    let servers = servers(&packages, routes.as_deref())
        .into_iter()
        .map(|server| {
            let artisan = server.artisan();
            ServerStatus {
                id: server.id(),
                command: artisan
                    .as_ref()
                    .map(|argv| command_line(container, argv).join(" ")),
                files: artisan
                    .as_ref()
                    .map(|_| status(project_dir, container, &server))
                    .unwrap_or_default(),
                url: match (&server, domain) {
                    (Server::Web { path }, Some(domain)) => {
                        Some(format!("https://{domain}/{}", path.trim_start_matches('/')))
                    }
                    _ => None,
                },
                server,
            }
        })
        .collect();

    Report {
        packages,
        container: container.to_string(),
        has_routes: routes_path.is_file(),
        servers,
    }
}

/// `text` with this server registered against the container.
///
/// Pure, and that is where the safety is: [`crate::agents::insert_named`] does
/// the reading, the one-key insert and the write-back, so a project file gets
/// the same three rules a home-directory file gets. An entry that is already
/// there keeps its **own name** — see [`Server::default_entry`].
pub fn register(
    text: &str,
    shape: crate::agents::Shape,
    container: &str,
    server: &Server,
) -> Result<String> {
    let artisan = server.artisan().ok_or_else(|| {
        crate::error::Error::new(
            crate::error::Code::InvalidInput,
            "this server is a route in the application; there is no command to register"
                .to_string(),
        )
    })?;

    let name = find_entry(text, shape, &artisan)
        .map(|(name, _)| name)
        .unwrap_or_else(|| server.default_entry());

    let mut object = serde_json::Map::new();
    if shape == crate::agents::Shape::VsCode {
        object.insert("type".into(), "stdio".into());
    }
    object.insert("command".into(), DOCKER.into());
    let args: Vec<serde_json::Value> = argv(container, &artisan)
        .into_iter()
        .map(serde_json::Value::from)
        .collect();
    object.insert("args".into(), args.into());

    crate::agents::insert_named(text, shape, &name, serde_json::Value::Object(object))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::Shape;

    fn dep(name: &str, version: &str) -> crate::deps::Dep {
        crate::deps::Dep {
            ecosystem: crate::deps::Ecosystem::Packagist,
            name: name.to_string(),
            version: version.to_string(),
            direct: true,
            source: None,
            hashed: true,
        }
    }

    #[test]
    fn the_three_packages_are_read_out_of_the_parsed_lock() {
        let deps = vec![
            dep("laravel/boost", "1.2.0"),
            dep("laravel/framework", "12.0"),
        ];
        let found = packages(&deps);
        assert_eq!(found.boost.as_deref(), Some("1.2.0"));
        assert_eq!(found.mcp, None);
        assert!(found.any());
        assert!(!packages(&[]).any());
    }

    /// The routes file is read, and a registration with nothing to read is
    /// skipped rather than guessed at.
    #[test]
    fn only_registrations_with_a_string_of_their_own_become_servers() {
        let php = r#"<?php
            use Laravel\Mcp\Facades\Mcp;

            Mcp::local('weather', WeatherServer::class);
            Mcp::web('/mcp/orders', OrdersServer::class);
            Mcp::local(Handles::SECRET, HiddenServer::class);
        "#;

        assert_eq!(
            parse_routes(php),
            vec![
                Server::Local {
                    handle: "weather".into()
                },
                Server::Web {
                    path: "/mcp/orders".into()
                },
            ]
        );

        // A call whose first argument is a constant must not reach forward into
        // the next call's literal.
        assert!(parse_routes("Mcp::local(Handles::ONLY, X::class);").is_empty());
        assert!(parse_routes("<?php // nothing here").is_empty());
    }

    /// `laravel/mcp` absent means no servers even when the routes file is full
    /// of them: a command against a package that is not installed is a failure
    /// in somebody's assistant.
    #[test]
    fn servers_follow_the_lock_file_and_not_the_routes_file_alone() {
        let php = "Mcp::local('weather', W::class);";
        assert!(servers(&Packages::default(), Some(php)).is_empty());

        let with_mcp = Packages {
            mcp: Some("0.4.0".into()),
            ..Default::default()
        };
        assert_eq!(
            servers(&with_mcp, Some(php)),
            vec![Server::Local {
                handle: "weather".into()
            }]
        );

        // Boost needs no routes file at all.
        let with_boost = Packages {
            boost: Some("1.0.0".into()),
            ..Default::default()
        };
        assert_eq!(servers(&with_boost, None), vec![Server::Boost]);
    }

    #[test]
    fn ids_round_trip() {
        for server in [
            Server::Boost,
            Server::Local {
                handle: "weather".into(),
            },
            Server::Web {
                path: "/mcp/orders".into(),
            },
        ] {
            assert_eq!(Server::parse_id(&server.id()), Some(server));
        }
        assert_eq!(Server::parse_id("local:"), None);
        assert_eq!(Server::parse_id("nonsense"), None);
    }

    /// The line that replaces `php artisan boost:mcp`.
    #[test]
    fn the_registered_line_goes_through_the_container() {
        let artisan = Server::Boost.artisan().unwrap();
        assert_eq!(
            command_line("stackvo-shop", &artisan),
            [
                "docker",
                "exec",
                "-i",
                "stackvo-shop",
                "php",
                "artisan",
                "boost:mcp"
            ]
        );
        // No `-t`. A TTY in the middle of a JSON-RPC stream is line discipline
        // nobody asked for.
        assert!(!argv("stackvo-shop", &artisan).contains(&"-t".to_string()));
    }

    /// What `boost:install` writes is recognised for what it is.
    #[test]
    fn a_host_php_registration_is_named_as_one() {
        for command in ["php", "/usr/bin/php", "php8.3", "/opt/homebrew/bin/php"] {
            assert!(runs_on_the_host(command), "{command}");
        }
        for command in ["docker", "stackvo", "/usr/local/bin/docker", "phpstorm"] {
            assert!(!runs_on_the_host(command), "{command}");
        }
    }

    /// The entry is found by what it runs, not by what it is called — and the
    /// repair keeps the name it found.
    #[test]
    fn the_repair_keeps_the_entry_name_the_file_already_used() {
        let original = r#"{
  "mcpServers": {
    "somebody-renamed-this": { "command": "php", "args": ["artisan", "boost:mcp"] },
    "unrelated": { "command": "node", "args": ["server.js"] }
  }
}
"#;
        let artisan = Server::Boost.artisan().unwrap();
        let (name, argv) = find_entry(original, Shape::McpServers, &artisan).unwrap();
        assert_eq!(name, "somebody-renamed-this");
        assert_eq!(argv, ["php", "artisan", "boost:mcp"]);

        let out = register(original, Shape::McpServers, "stackvo-shop", &Server::Boost).unwrap();
        let json: serde_json::Value = serde_json::from_str(&out).unwrap();
        let entry = &json["mcpServers"]["somebody-renamed-this"];
        assert_eq!(entry["command"], "docker");
        assert_eq!(entry["args"][0], "exec");
        assert_eq!(entry["args"][2], "stackvo-shop");

        // Everything else in the file survives — the whole reason this borrows
        // `agents::insert_named` instead of rendering a file.
        assert_eq!(json["mcpServers"]["unrelated"]["command"], "node");
        assert!(json["mcpServers"].get("laravel-boost").is_none());
    }

    /// An empty file gains the entry under the default name, and VS Code's
    /// shape gains the transport its format requires.
    #[test]
    fn a_file_with_nothing_in_it_gains_one_entry() {
        let out = register("", Shape::VsCode, "stackvo-shop", &Server::Boost).unwrap();
        let json: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(json["servers"]["laravel-boost"]["type"], "stdio");
        assert_eq!(json["servers"]["laravel-boost"]["command"], "docker");
    }

    /// A route is not a process, and asking to register one says so rather than
    /// writing an entry that starts nothing.
    #[test]
    fn a_web_server_has_no_command_to_register() {
        let web = Server::Web {
            path: "/mcp".into(),
        };
        assert!(web.artisan().is_none());
        assert!(register("{}", Shape::McpServers, "stackvo-shop", &web).is_err());
    }
}
