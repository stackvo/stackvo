//! What one assistant may do, to which project, and for how long.
//!
//! ## The problem this closes
//!
//! `--allow-writes` was one switch over twelve tools, and the twelve are not
//! one class of thing. `stackvo_xdebug_set` toggles a setting on one project.
//! `stackvo_stack_down` stops every container this machine runs, projects
//! included. Handing an assistant the second in order to give it the first is
//! the whole of the problem, and README's own warning said so in a paragraph
//! rather than in code: *"read that list before passing the flag"*.
//!
//! A person asked what they actually want to allow says a sentence with three
//! parts in it — **"this assistant may restart `shop`, for the next half
//! hour"** — and the flag could express none of them.
//!
//! ## Three limits, and why each is here
//!
//! * **Which tools.** `--allow=project_restart,project_start` opens two tools
//!   rather than twelve. Named, not counted: a grant that says "the safe ones"
//!   is a grant whose meaning changes the next time a tool is added.
//! * **Which project.** `--project=shop` bounds every tool that names a
//!   project — reads included, see below.
//! * **For how long.** `--for=30m` from the moment the server started. An
//!   assistant's session outlives the task it was given, and a grant that
//!   cannot end is a grant that is still open tomorrow.
//!
//! ## The rule with teeth: a scope removes what it cannot bound
//!
//! `--project=shop` cannot make `stackvo_stack_down` safe, because that tool
//! takes no project and stops everything there is. So a project scope does not
//! *narrow* such a tool — it **removes** it. Twelve write tools become four
//! (`xdebug_set`, `project_start`, `project_stop`, `project_restart`), and the
//! eight that a project cannot bound are not offered at all.
//!
//! That is the difference between a scope and a label. A surface that accepted
//! `--project=shop` and still served `stack_down` would be reporting a limit it
//! was not applying, which is worse than having no limit: the person who set it
//! stops watching.
//!
//! ## Reads are bounded too — and exactly how far
//!
//! The tempting rule is "the scope bounds writes; reads are harmless". It is
//! wrong here. `stackvo_explain_request` returns another project's request
//! traces, its queries and what its application dumped; `stackvo_log_read`
//! returns its log files. Somebody who has said "this assistant works on
//! `shop`" has not said "and may read everything the other eleven projects
//! did".
//!
//! So the scope applies to every tool that **names** a project, in both
//! directions, and `stackvo_projects` and `stackvo_overview` report the
//! projects in scope rather than all of them — a list that names what may not
//! be touched is an invitation to try.
//!
//! **It is not information isolation, and must not be described as it.** The
//! machine-wide instruments still answer, because they are about the machine
//! and its shared services rather than about a project: the doctor, the hosts
//! table, the certificate's domain list, the mail catcher, one database
//! service's query log, one container's log by id. Some of those carry another
//! project's name or content, and bounding them would leave a scoped assistant
//! unable to diagnose the project it *was* given — which is the whole reason
//! this surface exists. What the scope buys is that no per-project instrument
//! answers for a project it was not given, and that the writing tools shrink
//! to four. That is a bound on reach, not a wall around data.
//!
//! Two tools deliberately stay open under a scope for the same reason:
//! `stackvo_logs` and `stackvo_container_stats` take a *container* id, which
//! may be a project, a service or an instance. Scoping an argument that is
//! usually not a project would refuse `stackvo_logs redis-7-2` for no reason
//! anybody could act on.
//!
//! ## What this is not
//!
//! Not a security boundary against the assistant's own process. Anything that
//! can launch `stackvo-mcp` can launch it again without the flags — the grant
//! binds *this run*, which is the run the client's configuration file starts.
//! That is the same honesty [`crate::websurface`] applies to loopback: this
//! bounds the accident and the overreach, not the attacker who already has the
//! machine.

use crate::mcp::Tool;
use std::time::{Duration, Instant};

/// The prefix every tool name carries.
///
/// Accepted on the command line and not required: the README documents the
/// writing tools as `project_restart` and `stack_down`, and a flag that only
/// took `stackvo_project_restart` would be a second spelling of a list a person
/// has already read.
pub const PREFIX: &str = "stackvo_";

/// One run's authority.
#[derive(Debug, Clone)]
pub struct Grant {
    /// The write tools this grant opens, by full name. Empty is read-only.
    writes: Vec<&'static str>,
    /// The projects this grant is bounded to. Empty means every project.
    projects: Vec<String>,
    /// How long the grant lasts, from [`Grant::began`].
    lifetime: Option<Duration>,
    began: Instant,
}

/// Why a grant could not be read off the command line.
///
/// Every one of these is a refusal to start rather than a warning and a
/// degraded run. A mistyped `--allow-writes` starts a read-only server today
/// and the person who typed it finds out when the assistant says it cannot;
/// a mistyped `--project=shpo` would be worse — a server that quietly grants
/// nothing while its configuration file says otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bad {
    UnknownFlag(String),
    UnknownTool(String),
    /// A read tool named in `--allow`. Reads need no grant, and accepting the
    /// name would teach a wrong model of what the flag does.
    NotAWriteTool(String),
    Empty(&'static str),
    Duration(String),
}

impl std::fmt::Display for Bad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bad::UnknownFlag(flag) => write!(
                f,
                "unknown option `{flag}` — this server takes --allow-writes, \
                 --allow=<tool,...>, --project=<name,...> and --for=<30m|2h|90s>"
            ),
            Bad::UnknownTool(name) => {
                write!(
                    f,
                    "--allow names `{name}`, which is not a tool on this server"
                )
            }
            Bad::NotAWriteTool(name) => write!(
                f,
                "--allow names `{name}`, which only reads — reads need no grant and are \
                 always served"
            ),
            Bad::Empty(flag) => write!(f, "`{flag}` was given nothing to allow"),
            Bad::Duration(text) => write!(
                f,
                "--for={text} is not a duration — write it as 90s, 30m or 2h"
            ),
        }
    }
}

/// Why one call was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The tool writes and this grant does not open it.
    NotGranted {
        tool: &'static str,
        /// True when the grant opens nothing at all — a different sentence to
        /// say, and a different thing to do about it.
        read_only: bool,
    },
    /// The grant's time is up.
    Expired { lifetime: Duration },
    /// The call named a project this grant is not bounded to.
    ///
    /// Carries the scope as well as the name asked for, because "not allowed"
    /// leaves an assistant guessing at what is, and it will guess by trying.
    OutOfScope { project: String, scope: String },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NotGranted {
                tool,
                read_only: true,
            } => write!(
                f,
                "{tool} changes the stack and this server was started read-only"
            ),
            Refusal::NotGranted {
                tool,
                read_only: false,
            } => write!(
                f,
                "{tool} changes the stack and is not one of the tools this server was granted"
            ),
            Refusal::Expired { lifetime } => write!(
                f,
                "this server's grant lasted {} and has run out",
                human(*lifetime)
            ),
            Refusal::OutOfScope { project, scope } => write!(
                f,
                "this server is scoped to {scope} and was asked about {project}"
            ),
        }
    }
}

impl Refusal {
    /// The suggestion that goes with it, translated by the same catalogue the
    /// window uses.
    pub fn hint(&self) -> crate::hints::Hint {
        match self {
            Refusal::NotGranted {
                read_only: true, ..
            } => crate::hints::MCP_NEEDS_ALLOW_WRITES,
            Refusal::NotGranted {
                read_only: false, ..
            } => crate::hints::MCP_OUTSIDE_GRANT,
            Refusal::Expired { .. } => crate::hints::MCP_GRANT_EXPIRED,
            Refusal::OutOfScope { .. } => crate::hints::MCP_PROJECT_OUT_OF_SCOPE,
        }
    }
}

/// A duration as the flag that would produce it.
fn human(d: Duration) -> String {
    let seconds = d.as_secs();
    if seconds % 3600 == 0 {
        format!("{}h", seconds / 3600)
    } else if seconds % 60 == 0 {
        format!("{}m", seconds / 60)
    } else {
        format!("{seconds}s")
    }
}

/// `90s`, `30m`, `2h` — and nothing without a unit.
///
/// A bare `--for=30` reads as thirty minutes to one person and thirty seconds
/// to the next, and both readings are defensible. Refusing it costs one
/// character and removes the question.
pub fn duration(text: &str) -> Option<Duration> {
    let (number, unit) = text.split_at(text.len().checked_sub(1)?);
    let count: u64 = number.parse().ok()?;
    if count == 0 {
        return None;
    }
    let seconds = match unit {
        "s" => count,
        "m" => count.checked_mul(60)?,
        "h" => count.checked_mul(3600)?,
        _ => return None,
    };
    Some(Duration::from_secs(seconds))
}

/// Every write tool this server has.
pub fn all_writes() -> Vec<&'static str> {
    crate::mcp::TOOLS
        .iter()
        .filter(|t| t.writes)
        .map(|t| t.name)
        .collect()
}

/// The tool named, however it was spelled.
fn tool_named(name: &str) -> Option<&'static Tool> {
    let full = if name.starts_with(PREFIX) {
        name.to_string()
    } else {
        format!("{PREFIX}{name}")
    };
    crate::mcp::TOOLS.iter().find(|t| t.name == full)
}

impl Grant {
    /// A server that only answers questions. The default, and what a client
    /// that passes no arguments gets.
    pub fn read_only() -> Self {
        Self {
            writes: Vec::new(),
            projects: Vec::new(),
            lifetime: None,
            began: Instant::now(),
        }
    }

    /// Every write tool, every project, no expiry — what `--allow-writes`
    /// meant before this module existed, and still means.
    pub fn everything() -> Self {
        Self {
            writes: all_writes(),
            ..Self::read_only()
        }
    }

    /// Bound to these projects. Empty leaves it unbounded.
    pub fn scoped_to(mut self, projects: Vec<String>) -> Self {
        self.projects = projects;
        self
    }

    /// Ends this long after the run began.
    pub fn lasting(mut self, lifetime: Duration) -> Self {
        self.lifetime = Some(lifetime);
        self
    }

    /// Only these write tools.
    pub fn allowing(mut self, writes: Vec<&'static str>) -> Self {
        self.writes = writes;
        self
    }

    pub fn began(&self) -> Instant {
        self.began
    }

    pub fn projects(&self) -> &[String] {
        &self.projects
    }

    pub fn lifetime(&self) -> Option<Duration> {
        self.lifetime
    }

    /// True when this grant opens nothing.
    pub fn is_read_only(&self) -> bool {
        self.writes.is_empty()
    }

    /// True when the grant names the projects it may act on.
    pub fn is_scoped(&self) -> bool {
        !self.projects.is_empty()
    }

    pub fn expired_at(&self, now: Instant) -> bool {
        self.lifetime
            .is_some_and(|lifetime| now.duration_since(self.began) >= lifetime)
    }

    pub fn expired(&self) -> bool {
        self.expired_at(Instant::now())
    }

    /// Whether this grant opens one tool, at this moment.
    ///
    /// Reads are always open — the whole surface is designed so that reading is
    /// the thing an assistant is for. What is decided here is the twelve.
    pub fn opens_at(&self, tool: &Tool, now: Instant) -> bool {
        if !tool.writes {
            return true;
        }
        if self.expired_at(now) {
            return false;
        }
        // The rule with teeth: under a project scope, a write tool that names
        // no project is not narrowed, it is gone.
        if self.is_scoped() && tool.project_arg.is_none() {
            return false;
        }
        self.writes.contains(&tool.name)
    }

    pub fn opens(&self, tool: &Tool) -> bool {
        self.opens_at(tool, Instant::now())
    }

    /// Whether this grant may see or touch one project.
    pub fn covers_project(&self, name: &str) -> bool {
        self.projects.is_empty() || self.projects.iter().any(|p| p == name)
    }

    /// The one funnel every call passes through.
    pub fn admit_at(
        &self,
        tool: &Tool,
        args: &serde_json::Value,
        now: Instant,
    ) -> Result<(), Refusal> {
        if tool.writes && self.expired_at(now) {
            return Err(Refusal::Expired {
                // `expired_at` is only true when there is one.
                lifetime: self.lifetime.unwrap_or_default(),
            });
        }

        if !self.opens_at(tool, now) {
            return Err(Refusal::NotGranted {
                tool: tool.name,
                read_only: self.is_read_only(),
            });
        }

        // Reads and writes alike: a tool that names a project may only name one
        // this grant covers. A tool called without its project argument is left
        // to the dispatch arm, which is where "`name` is required" is said.
        if let Some(key) = tool.project_arg {
            if let Some(project) = args.get(key).and_then(|v| v.as_str()) {
                if !self.covers_project(project) {
                    return Err(Refusal::OutOfScope {
                        project: project.to_string(),
                        scope: self.projects.join(", "),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn admit(&self, tool: &Tool, args: &serde_json::Value) -> Result<(), Refusal> {
        self.admit_at(tool, args, Instant::now())
    }

    /// The flags that would produce this grant.
    ///
    /// What [`crate::agents`] writes into a client's configuration file, and
    /// therefore the reason this is a function rather than four lines built at
    /// the call site: the registration a person reads in `claude.json` and the
    /// grant this module enforces have to be the same sentence.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if !self.writes.is_empty() {
            // Everything, spelled the way the README documents it. A grant that
            // opens all twelve and renders `--allow=` with twelve names would
            // be correct and unreadable.
            if self.writes.len() == all_writes().len() {
                args.push("--allow-writes".to_string());
            } else {
                let short: Vec<&str> = self
                    .writes
                    .iter()
                    .map(|name| name.trim_start_matches(PREFIX))
                    .collect();
                args.push(format!("--allow={}", short.join(",")));
            }
        }

        if !self.projects.is_empty() {
            args.push(format!("--project={}", self.projects.join(",")));
        }

        if let Some(lifetime) = self.lifetime {
            args.push(format!("--for={}", human(lifetime)));
        }

        args
    }

    /// One line for the banner, in the terms the flags were given in.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();

        if self.writes.is_empty() {
            parts.push("read-only; pass --allow-writes to enable the rest".to_string());
        } else if self.writes.len() == all_writes().len() && !self.is_scoped() {
            parts.push("reads and writes".to_string());
        } else {
            let short: Vec<&str> = self
                .writes
                .iter()
                .map(|name| name.trim_start_matches(PREFIX))
                .collect();
            parts.push(format!("reads, and writes: {}", short.join(", ")));
        }

        if self.is_scoped() {
            parts.push(format!("only {}", self.projects.join(", ")));
        }

        if let Some(lifetime) = self.lifetime {
            parts.push(format!("for {}", human(lifetime)));
        }

        parts.join("; ")
    }

    /// Read a grant off the arguments this process was started with.
    pub fn parse<I>(args: I) -> Result<Self, Bad>
    where
        I: IntoIterator<Item = String>,
    {
        let mut grant = Grant::read_only();
        let mut allowed: Vec<&'static str> = Vec::new();
        let mut everything = false;

        for arg in args {
            if arg == "--allow-writes" {
                everything = true;
            } else if let Some(list) = arg.strip_prefix("--allow=") {
                let names = split(list);
                if names.is_empty() {
                    return Err(Bad::Empty("--allow"));
                }
                for name in names {
                    let tool = tool_named(&name).ok_or_else(|| Bad::UnknownTool(name.clone()))?;
                    if !tool.writes {
                        return Err(Bad::NotAWriteTool(name));
                    }
                    if !allowed.contains(&tool.name) {
                        allowed.push(tool.name);
                    }
                }
            } else if let Some(list) = arg.strip_prefix("--project=") {
                let names = split(list);
                if names.is_empty() {
                    return Err(Bad::Empty("--project"));
                }
                for name in names {
                    if !grant.projects.contains(&name) {
                        grant.projects.push(name);
                    }
                }
            } else if let Some(text) = arg.strip_prefix("--for=") {
                grant.lifetime =
                    Some(duration(text).ok_or_else(|| Bad::Duration(text.to_string()))?);
            } else {
                return Err(Bad::UnknownFlag(arg));
            }
        }

        // `--allow-writes` with `--allow=` is not a contradiction to resolve by
        // precedence: the union is what somebody who wrote both meant, and the
        // union of everything with anything is everything.
        grant.writes = if everything { all_writes() } else { allowed };

        Ok(grant)
    }
}

/// A comma list, with the empties dropped — `a,,b` is two names, not three.
fn split(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool(name: &str) -> &'static Tool {
        crate::mcp::TOOLS
            .iter()
            .find(|t| t.name == name)
            .expect(name)
    }

    fn parse(args: &[&str]) -> Result<Grant, Bad> {
        Grant::parse(args.iter().map(|a| a.to_string()))
    }

    #[test]
    fn no_arguments_is_a_server_that_only_answers_questions() {
        let grant = parse(&[]).unwrap();
        assert!(grant.is_read_only());
        assert!(grant.opens(tool("stackvo_doctor")));
        assert!(!grant.opens(tool("stackvo_stack_down")));
    }

    #[test]
    fn allow_writes_still_means_all_twelve() {
        let grant = parse(&["--allow-writes"]).unwrap();
        for t in crate::mcp::TOOLS {
            assert!(grant.opens(t), "{} is closed by --allow-writes", t.name);
        }
        assert_eq!(grant.to_args(), vec!["--allow-writes".to_string()]);
    }

    /// The sentence S-2 asked for: two tools rather than twelve.
    #[test]
    fn a_named_grant_opens_only_what_it_names() {
        let grant = parse(&["--allow=project_restart,project_start"]).unwrap();
        assert!(grant.opens(tool("stackvo_project_restart")));
        assert!(grant.opens(tool("stackvo_project_start")));
        assert!(!grant.opens(tool("stackvo_project_stop")));
        assert!(!grant.opens(tool("stackvo_stack_down")));
        // Reads never needed the grant and still do not.
        assert!(grant.opens(tool("stackvo_doctor")));
    }

    #[test]
    fn the_prefix_is_optional_because_the_readme_omits_it() {
        let short = parse(&["--allow=stack_down"]).unwrap();
        let long = parse(&["--allow=stackvo_stack_down"]).unwrap();
        assert!(short.opens(tool("stackvo_stack_down")));
        assert_eq!(short.to_args(), long.to_args());
    }

    /// The rule with teeth. A scope does not narrow `stack_down`; it removes it.
    #[test]
    fn a_project_scope_removes_every_write_tool_no_project_can_bound() {
        let grant = parse(&["--allow-writes", "--project=shop"]).unwrap();

        let open: Vec<&str> = crate::mcp::TOOLS
            .iter()
            .filter(|t| t.writes && grant.opens(t))
            .map(|t| t.name)
            .collect();

        assert_eq!(
            open,
            vec![
                "stackvo_xdebug_set",
                "stackvo_project_start",
                "stackvo_project_stop",
                "stackvo_project_restart",
            ],
            "a scoped grant offers exactly the write tools a project bounds"
        );
        assert!(!grant.opens(tool("stackvo_stack_down")));
        assert!(!grant.opens(tool("stackvo_service_stop")));
        assert!(!grant.opens(tool("stackvo_snapshot_take")));
    }

    #[test]
    fn a_scope_refuses_another_projects_name_on_a_write() {
        let grant = parse(&["--allow-writes", "--project=shop"]).unwrap();
        assert!(grant
            .admit(tool("stackvo_project_restart"), &json!({ "name": "shop" }))
            .is_ok());
        assert_eq!(
            grant.admit(tool("stackvo_project_restart"), &json!({ "name": "blog" })),
            Err(Refusal::OutOfScope {
                project: "blog".into(),
                scope: "shop".into()
            })
        );
    }

    /// The decision written down in this module's header: reads are bounded
    /// too, because another project's request traces are another project's.
    #[test]
    fn a_scope_refuses_another_projects_name_on_a_read() {
        let grant = parse(&["--project=shop"]).unwrap();
        assert!(grant
            .admit(
                tool("stackvo_explain_request"),
                &json!({ "project": "shop", "key": "k" })
            )
            .is_ok());
        assert_eq!(
            grant.admit(
                tool("stackvo_explain_request"),
                &json!({ "project": "blog", "key": "k" })
            ),
            Err(Refusal::OutOfScope {
                project: "blog".into(),
                scope: "shop".into()
            })
        );
    }

    /// A container id is not a project name, and refusing `redis-7-2` because
    /// it is not `shop` would be a limit nobody could act on.
    #[test]
    fn a_scope_leaves_the_container_tools_alone() {
        let grant = parse(&["--project=shop"]).unwrap();
        assert!(grant
            .admit(tool("stackvo_logs"), &json!({ "container": "redis-7-2" }))
            .is_ok());
        assert!(grant
            .admit(
                tool("stackvo_container_stats"),
                &json!({ "name": "redis-7-2" })
            )
            .is_ok());
    }

    #[test]
    fn a_grant_ends_when_its_time_is_up() {
        let grant = parse(&["--allow-writes", "--for=30m"]).unwrap();
        let restart = tool("stackvo_project_restart");
        let doctor = tool("stackvo_doctor");
        let started = grant.began();

        assert!(grant.opens_at(restart, started + Duration::from_secs(29 * 60)));
        assert!(!grant.opens_at(restart, started + Duration::from_secs(31 * 60)));
        assert_eq!(
            grant.admit_at(
                restart,
                &json!({ "name": "shop" }),
                started + Duration::from_secs(1801)
            ),
            Err(Refusal::Expired {
                lifetime: Duration::from_secs(1800)
            })
        );

        // Expiry takes the writes away and leaves the server useful.
        assert!(grant.opens_at(doctor, started + Duration::from_secs(31 * 60)));
        assert!(grant
            .admit_at(doctor, &json!({}), started + Duration::from_secs(31 * 60))
            .is_ok());
    }

    #[test]
    fn durations_need_a_unit() {
        assert_eq!(duration("90s"), Some(Duration::from_secs(90)));
        assert_eq!(duration("30m"), Some(Duration::from_secs(1800)));
        assert_eq!(duration("2h"), Some(Duration::from_secs(7200)));
        assert_eq!(duration("30"), None);
        assert_eq!(duration("0m"), None);
        assert_eq!(duration(""), None);
        assert!(matches!(parse(&["--for=30"]), Err(Bad::Duration(_))));
    }

    /// A mistyped flag stops the server rather than starting a different one.
    #[test]
    fn a_flag_this_server_does_not_know_is_refused_by_name() {
        assert_eq!(
            parse(&["--allow-write"]).err(),
            Some(Bad::UnknownFlag("--allow-write".into()))
        );
        assert!(matches!(
            parse(&["--allow=no_such_tool"]),
            Err(Bad::UnknownTool(_))
        ));
        assert!(matches!(
            parse(&["--allow=doctor"]),
            Err(Bad::NotAWriteTool(_))
        ));
        assert!(matches!(parse(&["--allow="]), Err(Bad::Empty("--allow"))));
        assert!(matches!(
            parse(&["--project="]),
            Err(Bad::Empty("--project"))
        ));
    }

    /// What a person reads in their client's configuration file has to be the
    /// grant this module enforces, so the flags round-trip.
    #[test]
    fn the_flags_a_grant_renders_parse_back_into_the_same_grant() {
        for args in [
            vec!["--allow-writes"],
            vec!["--allow=project_restart"],
            vec!["--allow-writes", "--project=shop", "--for=30m"],
            vec![
                "--allow=xdebug_set,project_start",
                "--project=shop,blog",
                "--for=2h",
            ],
        ] {
            let grant = parse(&args).unwrap();
            let rendered = grant.to_args();
            let again = Grant::parse(rendered.clone()).unwrap();

            assert_eq!(again.to_args(), rendered, "{args:?} does not round-trip");
            assert_eq!(again.projects(), grant.projects());
            assert_eq!(again.lifetime(), grant.lifetime());
            for t in crate::mcp::TOOLS {
                assert_eq!(again.opens(t), grant.opens(t), "{} differs", t.name);
            }
        }
    }

    #[test]
    fn the_banner_says_all_three_limits() {
        let grant = parse(&["--allow=project_restart", "--project=shop", "--for=30m"]).unwrap();
        let line = grant.describe();
        assert!(line.contains("project_restart"), "{line}");
        assert!(line.contains("shop"), "{line}");
        assert!(line.contains("30m"), "{line}");
    }

    /// Every write tool a project can bound declares the argument that names
    /// it — otherwise a scope would silently drop a tool it could have kept.
    #[test]
    fn the_project_argument_a_tool_declares_is_one_its_schema_has() {
        for t in crate::mcp::TOOLS {
            let Some(key) = t.project_arg else { continue };
            let schema = (t.schema)();
            assert!(
                schema["properties"].get(key).is_some(),
                "{} declares `{key}` as its project argument and its schema has no such property",
                t.name
            );
        }
    }
}
