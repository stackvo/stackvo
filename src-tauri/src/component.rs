//! Several runtimes in one repository, as one project.
//!
//! ## The unit every competitor picked, and what it costs
//!
//! **Measured August 2026**, and the date is here because the claim below is
//! about somebody else's product. An undated comparison does not age — it goes
//! quietly wrong, and the reader has no way to tell which.
//!
//! ServBay and FlyEnv both run many languages, and in both of them the unit is
//! a **site**: one directory, one runtime, one hostname. A repository holding
//! `api/` in Go, `web/` in Next.js and `worker/` in Python is three sites there
//! — three entries to create, three to start, three to remember are related —
//! and nothing in the tool knows they are one thing. Nothing in this category
//! treats a monorepo as a single subject.
//!
//! A local binary cannot: a directory has one runtime because the binary that
//! serves it has one. Containers are what make the other answer possible, and
//! this application already had every piece of it — eight runtimes,
//! [`crate::generator`] writing a Dockerfile for each, and Traefik routers
//! generated per hostname. What was missing was a manifest that could say so.
//!
//! ## Third of three, and the distinctions are the whole design
//!
//! | Declaration | What it is | Shared? | Built here? | Routed? |
//! | --- | --- | --- | --- | --- |
//! | `services: ["mysql"]` | A **need**, satisfied from the catalogue | One per machine | No | No |
//! | `sidecars` | **Somebody else's image**, project-scoped | No | No | No |
//! | `components` | **This repository's own code**, in a subdirectory | No | **Yes** | **Yes** |
//!
//! Folding any two of those together would make "how many of these exist" a
//! question with two answers — the reason [`crate::sidecar`] gives for not
//! being a service, applied once more.
//!
//! ## The containment is [`crate::sidecar`]'s, and it carries over exactly
//!
//! **No host port.** A component is reachable from the project's network, and
//! through Traefik when it names a domain. Never a published port: two clones
//! of one repository must not fight over 8080.
//!
//! **The path stays inside the project.** `../` and absolute paths are refused
//! at parse time, so a repository cannot name a build context above itself.
//!
//! **Named from the project, never declared.** `stackvo-<project>-<id>`, the
//! same derivation and the same namespace a sidecar uses — which is why an id
//! used by both is refused rather than silently producing one container for two
//! declarations.
//!
//! **Lives and dies with the project.** The project's own compose profile, so
//! `--profile project-shop` brings the whole repository up and stopping shop
//! stops all of it. That is the "one `up`" half of the item.
//!
//! ## PHP is deliberately not a component runtime
//!
//! A PHP component would need a web server, a document root, a `php.ini`
//! overlay and an FPM/server pair — four things the root project already
//! renders, none of which generalises to N copies inside one compose block, and
//! all of which have differential fixtures frozen over them. Refused **by
//! name**, with that reason, rather than accepted and rendered into something
//! subtly different from a real PHP project.
//!
//! The root project keeps its own runtime and is unchanged: a repository with
//! no `components` renders byte for byte what it always did.

use serde::Serialize;
use std::collections::BTreeMap;

/// Everything one project declared, in the order the file had them.
///
/// The ordered-map shape [`crate::sidecar::Declared`] uses, for the reason
/// written there: a `BTreeMap` alone would alphabetise, and a manifest saved
/// from the editor would come back reordered.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Declared {
    #[serde(serialize_with = "in_file_order")]
    inner: Inner,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Inner {
    by_id: BTreeMap<String, Component>,
    order: Vec<String>,
}

fn in_file_order<S: serde::Serializer>(inner: &Inner, s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut map = s.serialize_map(Some(inner.by_id.len()))?;
    for id in &inner.order {
        if let Some(value) = inner.by_id.get(id) {
            map.serialize_entry(id, value)?;
        }
    }
    map.end()
}

impl Declared {
    pub fn is_empty(&self) -> bool {
        self.inner.by_id.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.by_id.len()
    }

    pub fn get(&self, id: &str) -> Option<&Component> {
        self.inner.by_id.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Component)> {
        self.inner
            .order
            .iter()
            .filter_map(|id| self.inner.by_id.get_key_value(id.as_str()))
    }

    /// Every hostname the components ask for, in declaration order.
    ///
    /// Kept apart from `manifest.aliases` on purpose, even though both end up
    /// in `/etc/hosts` and in the certificate. An alias is *another name for
    /// this project's container*; a component domain is a name for a
    /// **different** container — folding them together would put a component's
    /// hostname into the project's own Traefik rule and route it to the wrong
    /// place.
    pub fn domains(&self) -> Vec<String> {
        self.iter().filter_map(|(_, c)| c.domain.clone()).collect()
    }
}

/// One directory of this repository, with its own runtime.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    /// One of [`crate::manifest::LANG_RUNTIMES`], or `node`.
    pub runtime: String,
    /// Where in the repository it lives, relative to the project directory.
    pub path: String,
    /// The hostname it answers on. Absent means it is reachable from the other
    /// containers and from nothing outside — which is what a queue worker
    /// wants, and forcing a hostname on one would be inventing a URL nobody
    /// asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub version: String,
    /// The dependency step, when the runtime has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<String>,
    /// The compile step, when the runtime has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    pub start: String,
    /// The port it listens on inside its own container. Only ever inside: see
    /// the module comment on why there is no host port here.
    pub port: u16,
}

impl Component {
    /// The build context, relative to the project directory.
    pub fn context(&self, project_dir: &str) -> String {
        format!("{project_dir}/{}", self.path)
    }

    /// As a `LangConfig`, so the Dockerfile renderers this shares with a
    /// single-runtime project need no second entry point.
    ///
    /// Converted rather than stored as one: a component carries a `path` and a
    /// `domain` that a `LangConfig` has no field for, and widening that struct
    /// would put two unrelated shapes in one type.
    pub fn as_lang(&self) -> crate::manifest::LangConfig {
        crate::manifest::LangConfig {
            version: self.version.clone(),
            install: self.install.clone(),
            build: self.build.clone(),
            start: self.start.clone(),
            port: self.port,
        }
    }

    /// As a `NodeConfig`, for `runtime: "node"`.
    ///
    /// `install` defaults to `npm install` because a node image without one
    /// starts with no dependencies, which is not a project anybody meant to
    /// declare — where a `LangConfig` may legitimately have none.
    pub fn as_node(&self) -> crate::manifest::NodeConfig {
        crate::manifest::NodeConfig {
            version: self.version.clone(),
            install: self
                .install
                .clone()
                .unwrap_or_else(|| "npm install".to_string()),
            build: self.build.clone(),
            start: self.start.clone(),
            port: self.port,
            package_manager: None,
        }
    }
}

/// The container's name on this machine.
///
/// [`crate::sidecar::container_name`]'s derivation, deliberately identical:
/// they share one namespace on the machine, so they must share one name-maker
/// or two declarations could produce one container.
pub fn container_name(project: &str, id: &str) -> String {
    crate::sidecar::container_name(project, id)
}

/// The Traefik router name for one component.
///
/// The project's router is `traefik_name(project)`; this is that plus the id,
/// so a component's router can never be the project's own — which would be one
/// rule silently replacing the other.
pub fn router_name(project: &str, id: &str) -> String {
    format!("{}-{id}", project.replace('.', "-"))
}

fn is_id(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 40
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A path that stays inside the project directory.
///
/// The rule `pkg::checked_relative` applies to a package's files, applied to a
/// build context — and it matters more here, because a context is what Docker
/// reads *everything* under. `.` is refused as well as `..`: a component whose
/// directory is the project itself is the project, and declaring it would mean
/// two containers building the same tree with two runtimes.
pub fn is_inside(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('~')
        // A Windows drive letter or a UNC path, neither of which is relative.
        && !path.contains(':')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// Read the `components` block, with every reason it could not be read.
///
/// Reports through [`crate::hooks::Problem`] like every other declaration in
/// the manifest, so a malformed one lands on the manifest as a finding beside
/// the others rather than as a failure out of the generator.
pub fn parse(json: &serde_json::Value) -> (Declared, Vec<crate::hooks::Problem>) {
    use crate::hooks::Problem;

    let mut out = Declared::default();
    let mut problems = Vec::new();

    let Some(block) = json.get("components") else {
        return (out, problems);
    };
    let Some(map) = block.as_object() else {
        problems.push(Problem {
            path: "components".into(),
            message: "`components` must be an object keyed by id".into(),
        });
        return (out, problems);
    };

    for (id, value) in map {
        let at = format!("components.{id}");
        let bad = |message: String| Problem {
            path: at.clone(),
            message,
        };

        if !is_id(id) {
            problems.push(bad(format!(
                "\"{id}\" is not a usable id — lower-case letters, digits and \
                 dashes, up to 40 characters. It becomes part of a container name"
            )));
            continue;
        }

        let Some(object) = value.as_object() else {
            problems.push(bad(
                "a component is an object with `runtime`, `path` and `start`".into(),
            ));
            continue;
        };

        // Named rather than ignored, on `sidecar`'s reasoning: somebody who
        // wrote one has a model to correct, and the correction is a real thing.
        if object.contains_key("ports") {
            problems.push(bad(
                "a component has no host port: it is reachable from this \
                 project's other containers, and from a browser only through \
                 the `domain` it names. A published port would be two clones of \
                 one repository fighting over the same number"
                    .into(),
            ));
            continue;
        }

        let Some(runtime) = object.get("runtime").and_then(|v| v.as_str()) else {
            problems.push(bad("a component needs `runtime`".into()));
            continue;
        };

        // PHP by name, with the reason, rather than a generic "unknown".
        if runtime == "php" {
            problems.push(bad(
                "php is not a component runtime. A PHP part needs a web server, \
                 a document root and a php.ini overlay — which the project's own \
                 runtime already renders, and which does not generalise to \
                 several inside one project. Keep the PHP half as the project's \
                 `runtime` and declare the other languages here"
                    .into(),
            ));
            continue;
        }
        if runtime != "node" && !crate::manifest::LANG_RUNTIMES.contains(&runtime) {
            problems.push(bad(format!(
                "\"{runtime}\" is not a runtime this build knows. One of: node, {}",
                crate::manifest::LANG_RUNTIMES.join(", ")
            )));
            continue;
        }

        let Some(path) = object.get("path").and_then(|v| v.as_str()) else {
            problems.push(bad(
                "a component needs `path` — the directory in this repository it \
                 is built from"
                    .into(),
            ));
            continue;
        };
        if !is_inside(path) {
            problems.push(bad(format!(
                "\"{path}\" is not a directory inside this project. A component \
                 is built from a subdirectory of the repository that declared \
                 it, so `..`, an absolute path and `.` itself are all refused"
            )));
            continue;
        }

        let Some(start) = object
            .get("start")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        else {
            problems.push(bad(
                "a component needs `start` — the command its container runs".into(),
            ));
            continue;
        };

        // Falls back to what the runtime already defaults to, so a component
        // that names only a runtime and a path is a complete declaration. The
        // defaults live in `manifest` and are not copied here.
        //
        // Node needs its own call because it is not in `LANG_RUNTIMES` — it is
        // accepted a few lines above by name, and `lang_defaults` returning
        // `None` for it made a node component the one kind that had to spell
        // `version` out. See `manifest::node_defaults`.
        let defaults = if runtime == "node" {
            Some(crate::manifest::node_defaults())
        } else {
            crate::manifest::lang_defaults(runtime)
        };
        let version = object
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| defaults.as_ref().map(|d| d.version.clone()));
        let Some(version) = version else {
            problems.push(bad(format!(
                "a {runtime} component needs `version` — this build has no \
                 default for it"
            )));
            continue;
        };

        let port = match object.get("port") {
            None => defaults.as_ref().map(|d| d.port).unwrap_or(3000),
            Some(value) => match value.as_u64().filter(|p| *p > 0 && *p <= u16::MAX as u64) {
                Some(p) => p as u16,
                None => {
                    problems.push(bad("`port` must be a number from 1 to 65535".into()));
                    continue;
                }
            },
        };

        let text = |key: &str| {
            object
                .get(key)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };

        let domain = text("domain");
        if let Some(domain) = &domain {
            if !is_hostname(domain) {
                problems.push(bad(format!(
                    "\"{domain}\" is not a hostname. It becomes a routing rule, \
                     an entry in the hosts file and a name on the certificate"
                )));
                continue;
            }
        }

        out.inner.order.push(id.clone());
        out.inner.by_id.insert(
            id.clone(),
            Component {
                runtime: runtime.to_string(),
                path: path.to_string(),
                domain,
                version,
                install: text("install"),
                build: text("build"),
                start: start.to_string(),
                port,
            },
        );
    }

    // Two hostnames answering to two containers is a rule that silently loses
    // to whichever Traefik read last, so it is refused here where the message
    // can name both.
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    let mut clashing = Vec::new();
    for (id, component) in out.iter() {
        if let Some(domain) = &component.domain {
            if let Some(first) = seen.insert(domain.as_str(), id.as_str()) {
                clashing.push((domain.clone(), first.to_string(), id.clone()));
            }
        }
    }
    for (domain, first, second) in clashing {
        out.inner.by_id.remove(&second);
        out.inner.order.retain(|id| id != &second);
        problems.push(Problem {
            path: format!("components.{second}"),
            message: format!(
                "`{domain}` is already the domain of the `{first}` component. \
                 One hostname is one container"
            ),
        });
    }

    (out, problems)
}

/// A hostname, by the same alphabet `manifest` accepts for an alias.
fn is_hostname(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 253
        && !text.starts_with('.')
        && !text.ends_with('.')
        && text.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(text: &str) -> (Declared, Vec<crate::hooks::Problem>) {
        parse(&serde_json::from_str(text).expect("the fixture is JSON"))
    }

    /// The item's own example, as one assertion: one repository, three
    /// runtimes, one project.
    #[test]
    fn three_directories_three_runtimes_one_project() {
        let (declared, problems) = parse_str(
            r#"{"components": {
                 "api":    {"runtime": "go", "path": "api", "domain": "api.shop.loc",
                            "build": "go build -o bin/api ./cmd/api", "start": "./bin/api",
                            "port": 8080},
                 "web":    {"runtime": "node", "path": "web", "domain": "web.shop.loc",
                            "install": "npm ci", "build": "npm run build",
                            "start": "npm start", "port": 3000},
                 "worker": {"runtime": "python", "path": "worker",
                            "start": "python worker.py"}
               }}"#,
        );

        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(declared.len(), 3);

        // In the file's order, not alphabetised — the manifest editor posts
        // this back and a reordered save is a diff nobody made.
        let ids: Vec<&str> = declared.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["api", "web", "worker"]);

        // A worker with no domain is reachable from the other containers and
        // from nothing outside, which is what a queue consumer wants.
        assert_eq!(declared.domains(), ["api.shop.loc", "web.shop.loc"]);
        assert!(declared.get("worker").unwrap().domain.is_none());

        // A version it did not name comes from the runtime's own defaults
        // rather than from a copy kept here.
        assert!(!declared.get("worker").unwrap().version.is_empty());
    }

    /// The containment `sidecar.rs` built, refused by name rather than ignored.
    #[test]
    fn a_component_cannot_ask_for_a_host_port() {
        let (declared, problems) = parse_str(
            r#"{"components": {"api": {"runtime": "go", "path": "api",
                 "start": "./bin/api", "ports": ["8080:8080"]}}}"#,
        );

        assert!(declared.is_empty());
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0].message.contains("two clones"),
            "the reason travels with the refusal: {}",
            problems[0].message
        );
    }

    /// A build context is what Docker reads *everything* under, so the path
    /// stays inside the project.
    #[test]
    fn a_path_that_leaves_the_project_is_refused() {
        for path in [
            "../secrets",
            "/etc",
            "~/x",
            "api/../..",
            "C:/x",
            "a\\b",
            ".",
            "",
        ] {
            assert!(!is_inside(path), "{path:?} must not be a build context");
        }
        for path in ["api", "services/api", "apps/web-ui"] {
            assert!(is_inside(path), "{path:?} is a directory in the project");
        }
    }

    /// PHP by name, with the reason.
    ///
    /// A PHP component would need a web server, a document root and a php.ini
    /// overlay — three things the project's own runtime already renders and
    /// none of which generalises to several inside one project. Accepting it
    /// and rendering something subtly different would be worse than refusing.
    #[test]
    fn php_is_refused_with_the_reason_rather_than_as_an_unknown_runtime() {
        let (_, problems) = parse_str(
            r#"{"components": {"legacy": {"runtime": "php", "path": "legacy",
                 "start": "php -S 0.0.0.0:8000"}}}"#,
        );
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0].message.contains("web server"),
            "{:?}",
            problems[0]
        );

        // And a runtime that is simply not one says which are.
        let (_, unknown) = parse_str(
            r#"{"components": {"x": {"runtime": "cobol", "path": "x", "start": "run"}}}"#,
        );
        assert!(unknown[0].message.contains("node"), "{:?}", unknown[0]);
    }

    /// One hostname is one container.
    ///
    /// Two components on one domain is a Traefik rule that silently loses to
    /// whichever was read last, so it is refused where the message can name
    /// both — and the *second* is dropped, keeping the first, because the file
    /// order is the only tie-break that is not arbitrary.
    #[test]
    fn two_components_cannot_share_a_hostname() {
        let (declared, problems) = parse_str(
            r#"{"components": {
                 "api": {"runtime": "go", "path": "api", "domain": "x.loc", "start": "./a"},
                 "alt": {"runtime": "go", "path": "alt", "domain": "x.loc", "start": "./b"}
               }}"#,
        );

        assert_eq!(declared.len(), 1);
        assert!(declared.get("api").is_some(), "the first one stands");
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("`api`"), "{:?}", problems[0]);
    }

    /// A component and a sidecar share one namespace on the machine, so they
    /// share one name-maker.
    #[test]
    fn a_component_is_named_from_the_project_the_way_a_sidecar_is() {
        assert_eq!(container_name("Shop", "api"), "stackvo-shop-api");
        assert_eq!(
            container_name("Shop", "api"),
            crate::sidecar::container_name("Shop", "api"),
            "one namespace, one derivation — otherwise two declarations make \
             one container"
        );

        // And the router can never be the project's own, which would be one
        // rule silently replacing the other.
        assert_eq!(router_name("parser.ajans", "api"), "parser-ajans-api");
    }

    /// One bad component does not take the others with it.
    ///
    /// The same trade `manifest` makes for sidecars and hooks: a project with
    /// one unreadable declaration still has nine that work, and refusing to
    /// open it would lose the nine to report the tenth.
    #[test]
    fn a_broken_component_is_a_warning_beside_the_ones_that_parsed() {
        let (declared, problems) = parse_str(
            r#"{"components": {
                 "api":  {"runtime": "go", "path": "api", "start": "./bin/api"},
                 "oops": {"runtime": "go", "path": "oops"}
               }}"#,
        );

        assert_eq!(declared.len(), 1);
        assert!(declared.get("api").is_some());
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("`start`"), "{:?}", problems[0]);
    }

    /// A project with no `components` block is untouched, which is what makes
    /// the whole feature additive.
    #[test]
    fn a_project_without_the_block_declares_nothing_and_reports_nothing() {
        let (declared, problems) = parse_str(r#"{"name": "shop", "runtime": "php"}"#);
        assert!(declared.is_empty());
        assert!(problems.is_empty());
        assert!(declared.domains().is_empty());

        // A block of the wrong shape is a problem rather than a silent empty.
        let (_, wrong) = parse_str(r#"{"components": ["api"]}"#);
        assert_eq!(wrong.len(), 1);
    }
}
