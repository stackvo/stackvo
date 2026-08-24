//! The supervisord inside a project's own container.
//!
//! StackVo runs its own processes with Docker as the supervisor — see
//! [`crate::worker`] and [`crate::cron`]. This is about the daemon that is
//! already in there: an nginx or caddy project's generated image runs
//! `supervisord` as its command, with `php-fpm` and the web server under it, so
//! every such project has had a supervisord in it the whole time.
//!
//! ## One way in, and why there is only one
//!
//! `docker exec supervisorctl`. supervisord's real API is XML-RPC over a
//! socket, and this module used to speak it — over a TCP port, a Unix socket
//! and an ssh tunnel as well, for daemons on machines this workspace does not
//! own. That was a second product living inside a local development tool: the
//! processes StackVo cares about are in containers it started, on this machine,
//! and reaching them is `docker exec` and nothing else.
//!
//! The cost is real and is named rather than hidden. `supervisorctl` answers in
//! text meant for a person, so [`parse_status`] reads it back into the rows the
//! XML-RPC API would have sent — and what cannot be recovered from that text is
//! absent rather than guessed at: no batching, and a log tail with no offset to
//! page back from.
//!
//! ## No shell, ever
//!
//! [`Target::exec`] takes an argument vector and `docker exec` execs it, so
//! nothing between this app and the process splits, expands or quotes anything.
//! File content goes in on standard input for the same reason. It is the rule
//! [`crate::hooks`] and [`crate::cron`] keep, and it is why there is no
//! "run a command in the container" feature here.
//!
//! ## What is derived rather than reported
//!
//! supervisord says what state a process is in and nothing about how it got
//! there, so a process that has crashed and restarted forty times in the last
//! minute reports `RUNNING` — which is the case somebody opened this to find.
//! [`Watch`] derives the restart count and flapping by watching the pid change
//! between looks; [`Check`] answers the other half, whether the thing inside
//! the process is answering at all.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{Code, Error, Result};

/// How long any one command is allowed to take.
///
/// Per command rather than per poll: a container that has gone away should fail
/// the row it owns and leave the rest of the table alone.
const TIMEOUT: Duration = Duration::from_secs(15);

// ------------------------------------------------------------- the target

/// What running a command in the container produced.
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// One project's supervisord.
///
/// There is nothing to configure and nothing stored: a project already names
/// its container, and the daemon is inside it. That is the whole reason this
/// module is small — it reaches exactly one kind of thing, in exactly one way.
#[derive(Debug, Clone)]
pub struct Target {
    pub project: String,
    pub container: String,
}

/// The supervisord in one project's container.
pub fn for_project(project: &str) -> Target {
    Target {
        project: project.to_string(),
        container: format!("{}{project}", crate::engine::CONTAINER_PREFIX),
    }
}

/// The whole command line, as an argv.
///
/// Its own function so it can be asserted without an engine. The test that used
/// to cover this ran `docker exec` for real and could not fail the way it was
/// written for — `docker exec <container> false` exits non-zero whether or not
/// `-i` was passed, and on a machine with no `docker` at all it failed for a
/// reason that says nothing about this code. What matters here is one flag in
/// one position, which is a string comparison.
fn docker_argv(container: &str, argv: &[String], with_input: bool) -> Vec<String> {
    let mut full = vec!["docker".to_string(), "exec".to_string()];
    // Without `-i` the engine gives the process a closed standard input, and a
    // command reading from it reads nothing — successfully. `tee` exits 0 and
    // writes an empty file, which is the destructive half of that.
    if with_input {
        full.push("-i".into());
    }
    full.push(container.to_string());
    full.extend(argv.iter().cloned());
    full
}

impl Target {
    /// Run a command in the container.
    ///
    /// `argv`, never a command line. `docker exec` takes an argument vector
    /// and execs it, so there is no shell between this app and the process —
    /// which is the same rule [`crate::hooks`] and [`crate::cron`] keep, and
    /// why there is no "run a command in the container" feature here.
    pub async fn exec(&self, argv: &[String]) -> Result<Output> {
        self.exec_with(argv, None).await
    }

    /// The same, with something written to the command's standard input.
    pub async fn exec_with(&self, argv: &[String], input: Option<&str>) -> Result<Output> {
        let full = docker_argv(&self.container, argv, input.is_some());

        let (program, args) = full.split_first().expect("a program");
        let late = || {
            Error::new(
                Code::NetworkError,
                format!("{program} did not answer in time"),
            )
        };
        let mut command = tokio::process::Command::new(program);
        command.args(args);

        let output = if let Some(input) = input {
            use tokio::io::AsyncWriteExt;

            command
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            let mut child = command
                .spawn()
                .map_err(|e| Error::io(format!("running {program}"), e))?;
            let mut pipe = child.stdin.take().expect("stdin was piped");
            let input = input.to_string();

            let work = async move {
                pipe.write_all(input.as_bytes())
                    .await
                    .map_err(|e| Error::io("writing to the command", e))?;
                // Shut down *and dropped* before waiting. Shutting down ends
                // the write half; the descriptor stays open until the handle
                // goes, and a command reading standard input waits for the
                // descriptor, not the half.
                pipe.shutdown()
                    .await
                    .map_err(|e| Error::io("closing the command's input", e))?;
                drop(pipe);
                child
                    .wait_with_output()
                    .await
                    .map_err(|e| Error::io(format!("running {program}"), e))
            };
            tokio::time::timeout(TIMEOUT, work)
                .await
                .map_err(|_| late())??
        } else {
            tokio::time::timeout(TIMEOUT, command.output())
                .await
                .map_err(|_| late())?
                .map_err(|e| Error::io(format!("running {program}"), e))?
        };

        Ok(Output {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            code: output.status.code().unwrap_or(-1),
        })
    }
}

// ------------------------------------------------------------ supervisorctl

/// A `supervisorctl` verb, and what it answers with.
///
/// This is the seam. Callers ask for `supervisor.getAllProcessInfo` — the name
/// supervisord's own XML-RPC API uses — and get the array that API would have
/// sent; what happened underneath is that `supervisorctl status` ran in the
/// container and its output was read back into that shape.
///
/// Keeping supervisord's names rather than inventing new ones is deliberate:
/// they are what the daemon's documentation uses, so a reader tracking down
/// what a verb does has something to search for.
///
/// The mapping is partial, and a method with no `supervisorctl` spelling is
/// refused by name — a caller learns that this cannot be answered rather than
/// getting an empty value that looks like an answer.
impl Target {
    pub async fn call(&self, method: &str, params: &[Value]) -> Result<Value> {
        let arg = |n: usize| -> String {
            params
                .get(n)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let ctl = |verb: Vec<String>| {
            let mut argv = vec!["supervisorctl".to_string()];
            argv.extend(verb);
            argv
        };

        match method {
            "supervisor.getState" => {
                let out = self.exec(&ctl(vec!["status".into()])).await?;
                // Both streams: `supervisorctl` writes its refusal to stdout.
                let said = format!("{}{}", out.stdout, out.stderr);
                if said.contains("refused connection") || said.contains("no such file") {
                    return Err(
                        Error::new(Code::NetworkError, said.trim().to_string()).with_hint(
                            "supervisord opens no RPC socket unless its config asks for one",
                        ),
                    );
                }
                Ok(json!({ "statecode": 1, "statename": "RUNNING" }))
            }

            "supervisor.getSupervisorVersion" => {
                let out = self.exec(&ctl(vec!["version".into()])).await?;
                Ok(Value::String(out.stdout.trim().to_string()))
            }

            "supervisor.getAllProcessInfo" => {
                let out = self.exec(&ctl(vec!["status".into()])).await?;
                Ok(Value::Array(parse_status(&out.stdout)))
            }

            // Answered from the whole table rather than by asking for one row:
            // `supervisorctl status <name>` exits non-zero when that process is
            // merely stopped, and telling "stopped" apart from "no such
            // process" would then mean parsing the same text anyway.
            "supervisor.getProcessInfo" => {
                let wanted = arg(0);
                let out = self.exec(&ctl(vec!["status".into()])).await?;
                parse_status(&out.stdout)
                    .into_iter()
                    .find(|row| full_name(row) == wanted)
                    .ok_or_else(|| Error::new(Code::NotFound, format!("BAD_NAME: {wanted}")))
            }

            "supervisor.startProcess" => self.act(ctl(vec!["start".into(), arg(0)])).await,
            "supervisor.stopProcess" => self.act(ctl(vec!["stop".into(), arg(0)])).await,
            "supervisor.startProcessGroup" => {
                self.act(ctl(vec!["start".into(), format!("{}:*", arg(0))]))
                    .await
            }
            "supervisor.stopProcessGroup" => {
                self.act(ctl(vec!["stop".into(), format!("{}:*", arg(0))]))
                    .await
            }
            "supervisor.startAllProcesses" => {
                self.act(ctl(vec!["start".into(), "all".into()])).await
            }
            "supervisor.stopAllProcesses" => self.act(ctl(vec!["stop".into(), "all".into()])).await,
            "supervisor.signalProcess" => {
                self.act(ctl(vec!["signal".into(), arg(1), arg(0)])).await
            }
            "supervisor.clearProcessLogs" => self.act(ctl(vec!["clear".into(), arg(0)])).await,
            "supervisor.clearAllProcessLogs" => {
                self.act(ctl(vec!["clear".into(), "all".into()])).await
            }

            "supervisor.tailProcessStdoutLog" | "supervisor.tailProcessStderrLog" => {
                let lines = params.get(2).and_then(Value::as_i64).unwrap_or(200);
                let mut verb = vec!["tail".to_string(), format!("-{}", lines.clamp(1, 5000))];
                verb.push(arg(0));
                // `supervisorctl tail` takes the channel as a trailing word.
                // The flag that looks right — `-f` — means *follow*, and a
                // follow never returns.
                if method.ends_with("StderrLog") {
                    verb.push("stderr".to_string());
                }
                let out = self.exec(&ctl(verb)).await?;
                // The RPC shape is `[text, offset, overflow]`, and the two
                // numbers are honestly zero: `supervisorctl tail` has no offset
                // to report.
                Ok(json!([out.stdout, 0, false]))
            }

            other => Err(Error::new(
                Code::Unsupported,
                format!("`{other}` is not something supervisorctl can do"),
            )),
        }
    }

    /// A control verb, whose answer is whether it worked.
    ///
    /// The refusal is in the output, and **not at the start of the line**:
    /// `supervisorctl` names the process first, so a missing one comes back as
    /// `nosuch: ERROR (no such process)`.
    ///
    /// The exit code is checked too but second, because it is the less
    /// informative half: `supervisorctl` exits 1 for a refusal and says which
    /// one on stdout, and a caller told only "exited 1" has to go and look.
    async fn act(&self, argv: Vec<String>) -> Result<Value> {
        let out = self.exec(&argv).await?;
        let text = format!("{}{}", out.stdout, out.stderr);

        if let Some(line) = text.lines().find(|l| l.contains("ERROR")) {
            return Err(Error::new(Code::Conflict, line.trim().to_string()));
        }
        if out.code != 0 {
            let said = text.trim();
            return Err(Error::new(
                Code::Conflict,
                if said.is_empty() {
                    format!("supervisorctl exited {}", out.code)
                } else {
                    said.lines().next().unwrap_or(said).to_string()
                },
            ));
        }
        Ok(Value::Bool(true))
    }

    /// Does one line of `supervisorctl` output report a refusal?
    #[cfg(test)]
    fn is_refusal(line: &str) -> bool {
        line.contains("ERROR")
    }
}

/// `group:name`, or just the name when a process is its own group.
///
/// The form supervisord uses everywhere a process is addressed — every control
/// verb takes it, and it is what a status line prints in its first column.
pub fn full_name(row: &Value) -> String {
    let name = row.get("name").and_then(Value::as_str).unwrap_or_default();
    let group = row.get("group").and_then(Value::as_str).unwrap_or_default();
    if group.is_empty() || group == name {
        name.to_string()
    } else {
        format!("{group}:{name}")
    }
}

/// supervisord's numeric state for a name `supervisorctl` prints.
pub fn state_code(name: &str) -> i64 {
    match name {
        "STOPPED" => 0,
        "STARTING" => 10,
        "RUNNING" => 20,
        "BACKOFF" => 30,
        "STOPPING" => 40,
        "EXITED" => 100,
        "FATAL" => 200,
        _ => 1000,
    }
}

/// The other direction, for a table that arrived over XML-RPC as numbers.
pub fn state_name(code: i64) -> &'static str {
    match code {
        0 => "STOPPED",
        10 => "STARTING",
        20 => "RUNNING",
        30 => "BACKOFF",
        40 => "STOPPING",
        100 => "EXITED",
        200 => "FATAL",
        _ => "UNKNOWN",
    }
}

/// Read one `supervisorctl status` line back into the row XML-RPC would send.
///
/// ```text
/// web:app                  RUNNING   pid 1234, uptime 1:23:45
/// flaky                    FATAL     Exited too quickly (process log may have details)
/// ```
///
/// The second field is the state and is always upper case, which is what tells
/// a status line apart from the banner `supervisorctl` prints when it cannot
/// reach the daemon.
fn parse_status_line(line: &str) -> Option<Value> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let mut fields = line.split_whitespace();
    let full = fields.next()?;
    let statename = fields.next()?;
    if !statename.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }

    let rest = line[full.len()..].trim_start();
    let rest = rest[statename.len()..].trim_start();

    let (group, name) = match full.split_once(':') {
        Some((group, name)) => (group.to_string(), name.to_string()),
        None => (full.to_string(), full.to_string()),
    };

    Some(json!({
        "name": name,
        "group": group,
        "state": state_code(statename),
        "statename": statename,
        "description": rest,
        "pid": parse_pid(rest),
        "start": 0,
        "stop": 0,
        "now": 0,
        // Not derivable from the text, and left absent rather than guessed:
        // `uptime` is computed from `start` and `now` by the caller, and a
        // fabricated pair would produce a number that means nothing.
        "exitstatus": 0,
        "spawnerr": if state_code(statename) == 200 { rest } else { "" },
        "uptimeText": parse_uptime(rest),
    }))
}

/// `pid 1234, uptime 0:00:12` → 1234.
fn parse_pid(text: &str) -> i64 {
    let Some(at) = text.find("pid ") else {
        return 0;
    };
    text[at + 4..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// `pid 1234, uptime 1 day, 2:03:04` → `1 day, 2:03:04`.
fn parse_uptime(text: &str) -> String {
    let Some(at) = text.find("uptime ") else {
        return String::new();
    };
    text[at + 7..].trim().to_string()
}

/// Every status line in `supervisorctl status` output.
pub fn parse_status(stdout: &str) -> Vec<Value> {
    stdout.lines().filter_map(parse_status_line).collect()
}

/// Where the health checks live.
///
/// A file of their own rather than a field on the server record, because a
/// check can be about a project's container too — and that has no stored
/// record to hang one on. Keyed by the same id the commands take, which is what
/// lets one store serve both halves.
pub fn checks_path(root: &std::path::Path) -> PathBuf {
    root.join("supervisor-checks.json")
}

pub fn checks(root: &std::path::Path) -> Result<Vec<Check>> {
    let path = checks_path(root);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    serde_json::from_str(&text).map_err(|e| {
        Error::new(
            Code::InvalidInput,
            format!("{} is not readable: {e}", path.display()),
        )
    })
}

/// The checks for one server, by the id its commands take.
pub fn checks_for(root: &std::path::Path, project: &str) -> Vec<Check> {
    checks(root)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.project == project)
        .collect()
}

pub fn save_checks(root: &std::path::Path, checks: &[Check]) -> Result<()> {
    let text = serde_json::to_string_pretty(checks)
        .map_err(|e| Error::new(Code::IoError, e.to_string()))?;
    crate::atomic::write(&checks_path(root), &format!("{text}\n"))
}

/// Add a check, or replace the one already on that process.
pub fn upsert_check(root: &std::path::Path, check: Check) -> Result<Vec<Check>> {
    check.validate()?;
    let mut all = checks(root)?;
    match all
        .iter_mut()
        .find(|c| c.project == check.project && c.process == check.process)
    {
        Some(existing) => *existing = check,
        None => all.push(check),
    }
    save_checks(root, &all)?;
    Ok(all)
}

pub fn remove_check(root: &std::path::Path, project: &str, process: &str) -> Result<Vec<Check>> {
    let kept: Vec<Check> = checks(root)?
        .into_iter()
        .filter(|c| !(c.project == project && c.process == process))
        .collect();
    save_checks(root, &kept)?;
    Ok(kept)
}

/// Probe every check for one server and hang the answers on the rows.
///
/// Concurrently, because a check is a network round trip and a server with six
/// of them would otherwise add six timeouts to a poll that has to finish before
/// the next one starts.
pub async fn attach_checks(root: &std::path::Path, snapshot: &mut Snapshot) {
    let checks = checks_for(root, &snapshot.project);
    if checks.is_empty() {
        return;
    }

    let probes = checks
        .iter()
        .map(|check| async move { (check.process.clone(), probe(check).await) });
    let answers: Vec<(String, CheckResult)> = futures_util::future::join_all(probes).await;

    for (process, result) in answers {
        if let Some(row) = snapshot
            .processes
            .iter_mut()
            .find(|p| p.full_name == process)
        {
            // Counted here rather than in `summarize`, which runs before the
            // probes: a screen showing "3 running" beside a row that is up and
            // not answering has told the reader the opposite of the point.
            if !result.ok {
                snapshot.summary.failing += 1;
            }
            row.check = Some(result);
        }
    }
}

// --------------------------------------------------------------- alarms

/// Something worth interrupting somebody about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Alarm {
    /// A process gave up. supervisord has stopped retrying it.
    Fatal,
    /// It comes back every time and keeps dying — see [`Watch`].
    Flapping,
    /// It is up and its own health check stopped answering.
    NotAnswering,
}

impl Alarm {
    pub fn as_str(self) -> &'static str {
        match self {
            Alarm::Fatal => "fatal",
            Alarm::Flapping => "flapping",
            Alarm::NotAnswering => "notAnswering",
        }
    }
}

/// One thing that just became true.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Raised {
    pub project: String,
    pub process: String,
    pub kind: Alarm,
    /// What the daemon or the probe said, so the notification can carry a
    /// reason rather than only a name.
    pub detail: String,
}

/// What each process looked like last time, so only *changes* are reported.
///
/// Edge-triggered, and that is the whole design. A process sitting in FATAL is
/// not news every twenty seconds — it was news once, when it got there. Level
/// triggering would produce a notification every poll for as long as nobody
/// fixed it, which trains somebody to dismiss the one that mattered.
#[derive(Default)]
pub struct Alarms {
    seen: std::collections::HashMap<String, (i64, bool, bool)>,
}

impl Alarms {
    /// Compare this look with the last one and say what is new.
    pub fn changed(&mut self, project: &str, snapshot: &Snapshot) -> Vec<Raised> {
        let mut raised = Vec::new();

        for row in &snapshot.processes {
            let key = format!("{project}:{}", row.full_name);
            // A check that has never run is not a check that failed.
            let answering = row.check.as_ref().is_none_or(|c| c.ok);
            let now = (row.state, row.flapping, answering);

            if let Some(&(was_state, was_flapping, was_answering)) = self.seen.get(&key) {
                let mut raise = |kind: Alarm, detail: String| {
                    raised.push(Raised {
                        project: project.to_string(),
                        process: row.full_name.clone(),
                        kind,
                        detail,
                    })
                };

                if row.state == 200 && was_state != 200 {
                    raise(Alarm::Fatal, row.spawn_err.clone());
                }
                if row.flapping && !was_flapping {
                    raise(Alarm::Flapping, format!("{} restarts", row.restarts));
                }
                if !answering && was_answering {
                    let said = row
                        .check
                        .as_ref()
                        .map(|c| c.detail.clone())
                        .unwrap_or_default();
                    raise(Alarm::NotAnswering, said);
                }
            }
            // The first look records and says nothing. A process that was
            // already FATAL when this app opened did not just become FATAL,
            // and announcing it would be announcing the past.
            self.seen.insert(key, now);
        }

        raised
    }

    /// Forget a project, so one that is deleted does not leave its processes
    /// in memory for the rest of the session.
    pub fn forget(&mut self, project: &str) {
        let prefix = format!("{project}:");
        self.seen.retain(|key, _| !key.starts_with(&prefix));
    }
}

// ------------------------------------------------------- the background watch

/// How often a watched server is looked at when nobody is on its page.
///
/// Slower than the screen's own refresh on purpose. This exists to *notice*,
/// not to watch: a page somebody is reading wants to feel live, and a
/// background connection to somebody's production box every four seconds is a
/// cost nobody asked for. Twenty seconds is inside the minute somebody would
/// take to notice on their own.
const WATCH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(20);

/// Watch the running projects, and say when something in one breaks.
///
/// ## Why this is in the backend at all
///
/// The pane polls for itself, and that is enough for a pane. It is not enough
/// for a notification: the whole value of one is that nobody is looking, and a
/// poll that only runs while its tab is open can only ever tell somebody what
/// is already in front of them. So the noticing has to outlive the pane, which
/// means it lives here.
///
/// ## What it does not do
///
/// Nothing is fixed. No process is restarted — this looks and reports. A tool
/// that quietly restarted `php-fpm` would be hiding the event it was built to
/// surface, and doing it while nobody was watching.
///
/// It also looks at **running** projects only, and asks the engine which those
/// are rather than trying every project it knows about: a stopped project has
/// no container to exec into, and a `docker exec` that can only fail is a
/// process started twenty seconds at a time for nothing.
pub fn watch(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(WATCH_INTERVAL).await;
            if let Err(e) = sweep(&app).await {
                // A workspace that has moved, or an engine that is not there.
                // Logged rather than raised: this is a background loop, and
                // there is no caller to hand an error to.
                tracing::debug!("supervisor watch: {}", e.message);
            }
        }
    });
}

async fn sweep(app: &tauri::AppHandle) -> crate::error::Result<()> {
    use tauri::{Emitter, Manager};

    let state = app.state::<crate::commands::AppState>();
    let root = state.root()?;

    let running: Vec<String> = crate::engine::stackvo_containers()
        .await?
        .into_iter()
        .filter(|(_, info)| info.running)
        .map(|(id, _)| id)
        // Workers, scheduled jobs and service instances carry their own
        // prefixes; a project's container is the one whose id is just its name.
        .filter(|id| !id.contains('-') || crate::workspace::project_dir(&root, id).is_ok())
        .collect();

    for project in running {
        let target = for_project(&project);

        // The same watch the pane uses, so a restart seen here counts for the
        // table too — and the two never disagree about how many there were.
        let mut seen = {
            let Ok(mut held) = state.supervisor_watch.lock() else {
                continue;
            };
            std::mem::take(&mut *held)
        };
        let looked = snapshot(&mut seen, &target).await;
        if let Ok(mut held) = state.supervisor_watch.lock() {
            *held = seen;
        }

        // A project without a reachable supervisord is not an alarm. Most of
        // them are not PHP, or run a server that does not use one, and
        // announcing that every twenty seconds would be announcing the shape of
        // somebody's workspace.
        let Ok(mut snapshot) = looked else {
            continue;
        };
        attach_checks(&root, &mut snapshot).await;

        let raised = {
            let Ok(mut alarms) = state.supervisor_alarms.lock() else {
                continue;
            };
            alarms.changed(&project, &snapshot)
        };

        for alarm in raised {
            let _ = app.emit("supervisor:alarm", &alarm);
        }
    }

    Ok(())
}

// ------------------------------------------------- a project's own daemon

/// What happened when we went looking for a supervisord in a container.
///
/// Three failures that look identical from the outside — the table is empty
/// either way — and send somebody to three different places. Separated here so
/// the screen can say which one it is rather than "could not connect".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reach {
    /// It answered.
    Ok,
    /// There is no `supervisorctl` in there at all. The project runs its
    /// server some other way — apache, frankenphp, swoole, or a runtime that
    /// is not PHP — and there is nothing to show.
    NoSupervisord,
    /// `supervisorctl` is there and the daemon would not talk to it. Almost
    /// always one thing: the image was built before StackVo put a socket in
    /// the generated config, so it needs a rebuild.
    NoSocket,
    /// The container is not running.
    Stopped,
}

/// Read `supervisorctl`'s failure, which is the only thing that says which.
///
/// Takes **both streams together**, and that is not tidiness. `supervisorctl`
/// writes its refusal to *stdout* — `unix:///var/run/supervisor.sock no such
/// file`, exit 4 — while the engine's own failures come back on stderr.
/// Measured, after a version of this that read stderr alone reported a project
/// whose daemon would not talk as reachable and then showed an empty table with
/// nothing to explain it.
///
/// The exit code is deliberately not part of this. `supervisorctl status`
/// exits non-zero whenever a process is merely down — a project with one
/// stopped worker exits 3 and has answered perfectly well — so a code here
/// would report a healthy daemon as unreachable on the days somebody stopped
/// something.
///
/// Pure, and tested against the strings the engine and supervisord actually
/// produce — the whole judgement is in text nobody here controls, so it is
/// worth being able to change it without a container.
pub fn classify(output: &str) -> Reach {
    let text = output.to_ascii_lowercase();

    // The engine, before anything in the container ran.
    if text.contains("is not running") || text.contains("no such container") {
        return Reach::Stopped;
    }
    // The engine again: there is no such program in the image. Matched on the
    // exec wording rather than on "no such file", which is also the first half
    // of supervisorctl's own refusal below.
    if text.contains("executable file not found") || text.contains("exec failed") {
        return Reach::NoSupervisord;
    }
    // supervisord's own refusal, and the reason a rebuild fixes it. Both
    // spellings: the socket file is absent before the daemon has ever made
    // one, and present-but-closed once it has.
    if text.contains("refused connection") || text.contains("no such file") {
        return Reach::NoSocket;
    }
    Reach::Ok
}

// --------------------------------------------------------- health checks

/// A probe that answers the question `RUNNING` cannot.
///
/// supervisord reports that a process is *up*. It has no idea whether the
/// thing inside it is answering — a `php-fpm` that has run out of workers, a
/// queue worker wedged on a lock and a web server serving 502 are all
/// `RUNNING`, and that is the state somebody is staring at when they open this.
///
/// ## One check per process, not a list
///
/// A process either answers or it does not. Two checks on one process would be
/// two answers to one question, and the row has one place to put an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    /// Which project's supervisord this is about.
    pub project: String,
    /// `group:name`, as the table shows it.
    pub process: String,
    /// `http` or `tcp`.
    pub kind: String,
    /// A URL for `http`, `host:port` for `tcp`.
    pub target: String,
    /// The status that counts as answering. 200 unless something else is
    /// meant — a health endpoint behind auth answers 401 and is working.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl Check {
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms.unwrap_or(4000).clamp(200, 30_000))
    }

    pub fn validate(&self) -> Result<()> {
        let bad = |m: &str| Err(Error::new(Code::InvalidInput, m.to_string()));
        if self.project.trim().is_empty() {
            return bad("a check needs a project to be about");
        }
        if self.process.trim().is_empty() {
            return bad("a check needs a process to be about");
        }
        match self.kind.as_str() {
            "http" => {
                if !self.target.starts_with("http://") && !self.target.starts_with("https://") {
                    return bad("an HTTP check needs a URL starting with http:// or https://");
                }
            }
            "tcp" => {
                // Split from the right: an IPv6 literal is full of colons and
                // only the last one is the port.
                let ok = self.target.rsplit_once(':').is_some_and(|(host, port)| {
                    !host.is_empty() && port.parse::<u16>().is_ok_and(|p| p > 0)
                });
                if !ok {
                    return bad("a TCP check needs host:port");
                }
            }
            other => {
                return Err(Error::new(
                    Code::InvalidInput,
                    format!("`{other}` is not a kind of check"),
                ))
            }
        }
        Ok(())
    }
}

/// What one probe found.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub ok: bool,
    /// What happened, in the words of whatever refused — a status code, a
    /// connection error. Shown beside the row, so it has to be short enough to
    /// read there.
    pub detail: String,
    pub ms: u64,
}

/// Run one probe.
///
/// Never fails: a probe that could not be made is a failing check, not an
/// error. The caller is drawing a row, and an error here would take the row's
/// other columns with it.
pub async fn probe(check: &Check) -> CheckResult {
    let started = std::time::Instant::now();
    let elapsed = |started: std::time::Instant| started.elapsed().as_millis() as u64;

    match check.kind.as_str() {
        "http" => {
            let want = check.expect_status.unwrap_or(200);
            let client = match reqwest::Client::builder()
                .timeout(check.timeout())
                // `no_proxy` for the reason `mail.rs` gives: the process-wide
                // proxy feature would send a request for a machine on this
                // network to a proxy that has never heard of it.
                .no_proxy()
                // A local development certificate is signed by StackVo's own
                // CA, which this process has no reason to know about. A check
                // that failed on trust would be reporting the certificate,
                // not the service — and the service is the question.
                .danger_accept_invalid_certs(true)
                .build()
            {
                Ok(client) => client,
                Err(e) => {
                    return CheckResult {
                        ok: false,
                        detail: e.to_string(),
                        ms: elapsed(started),
                    }
                }
            };

            match client.get(&check.target).send().await {
                Ok(response) => {
                    let got = response.status().as_u16();
                    CheckResult {
                        ok: got == want,
                        detail: if got == want {
                            format!("HTTP {got}")
                        } else {
                            format!("HTTP {got}, expected {want}")
                        },
                        ms: elapsed(started),
                    }
                }
                Err(e) => CheckResult {
                    ok: false,
                    // reqwest's message carries the whole URL and the chain
                    // beneath it, which is a paragraph in a table cell.
                    detail: short_reason(&e.to_string()),
                    ms: elapsed(started),
                },
            }
        }
        "tcp" => {
            let target = check.target.clone();
            let connect = tokio::net::TcpStream::connect(target);
            match tokio::time::timeout(check.timeout(), connect).await {
                Ok(Ok(_)) => CheckResult {
                    ok: true,
                    detail: "connected".into(),
                    ms: elapsed(started),
                },
                Ok(Err(e)) => CheckResult {
                    ok: false,
                    detail: short_reason(&e.to_string()),
                    ms: elapsed(started),
                },
                Err(_) => CheckResult {
                    ok: false,
                    detail: "timed out".into(),
                    ms: elapsed(started),
                },
            }
        }
        other => CheckResult {
            ok: false,
            detail: format!("`{other}` is not a kind of check"),
            ms: 0,
        },
    }
}

/// The first clause of a nested error message.
///
/// The row this lands in is one line beside a process name. A message that
/// wraps to three pushes the thing it is about off the screen.
fn short_reason(message: &str) -> String {
    let first = message.split(':').next_back().unwrap_or(message).trim();
    let text = if first.len() < 8 { message } else { first };
    text.chars().take(90).collect::<String>().trim().to_string()
}

// -------------------------------------------------------- what we watched

/// How long a restart stays in the window that decides flapping.
const FLAP_MEMORY: Duration = Duration::from_secs(300);
/// A process that restarts this often, this recently, is flapping.
const FLAP_WINDOW: Duration = Duration::from_secs(60);
const FLAP_COUNT: usize = 3;

#[derive(Default)]
struct History {
    last_pid: i64,
    restarts: i64,
    seen_at: Vec<std::time::Instant>,
}

/// What repeated looking tells you that one look cannot.
///
/// supervisord reports a process's *current* state and nothing about how it got
/// there. So a process that has crashed and been restarted forty times in the
/// last minute reports `RUNNING`, and the screen agrees with it — which is the
/// exact case somebody opened this app to find.
///
/// Both numbers here are derived by watching the pid change between polls.
/// Neither is persisted, and that is deliberate: a restart count that survived
/// a restart of this app would answer "how many times since some moment nobody
/// remembers", which is not a question anybody has. This one means "since I
/// started looking".
///
/// ## What it cannot see, and what covers that
///
/// A process that dies faster than the poll interval is undercounted: between
/// two looks it may have restarted five times and shown one new pid. Measured
/// against a daemon running a program that exits a second after it starts, at a
/// four-second cadence — the count moved by one.
///
/// That case is not missed, though, it is just reported by supervisord rather
/// than derived here: a program failing to stay up sits in `BACKOFF`, which is
/// a state the table already shows and a description that already says
/// "Exited too quickly". The two signals cover different halves — `BACKOFF` is
/// "it cannot start", and flapping is "it starts, looks fine, and keeps dying",
/// which is the half nothing else on screen would say.
#[derive(Default)]
pub struct Watch {
    by_process: std::collections::HashMap<String, History>,
}

impl Watch {
    /// Note what this poll saw, and say what it means.
    ///
    /// Called once per snapshot with every row, because the absence of a
    /// process from the table is also information — a process that is gone has
    /// no pid to compare against next time.
    fn observe(&mut self, project: &str, rows: &mut [Process]) {
        let now = std::time::Instant::now();

        for row in rows.iter_mut() {
            let key = format!("{project}:{}", row.full_name);
            let known = self.by_process.contains_key(&key);
            let history = self.by_process.entry(key).or_default();

            // Only a *change* of pid counts, and only against a pid this
            // process has been seen with before. Without the second half, the
            // first poll after opening the app would report every running
            // process as having just restarted.
            if known && row.pid > 0 && row.pid != history.last_pid {
                history.restarts += 1;
                history.seen_at.push(now);
                history
                    .seen_at
                    .retain(|at| now.duration_since(*at) < FLAP_MEMORY);
            }
            history.last_pid = row.pid;

            row.restarts = history.restarts;
            row.flapping = history
                .seen_at
                .iter()
                .filter(|at| now.duration_since(**at) < FLAP_WINDOW)
                .count()
                >= FLAP_COUNT;
        }
    }

    /// Forget a server. Called when one is removed, so its rows do not sit in
    /// memory for the rest of the session.
    pub fn forget(&mut self, project: &str) {
        let prefix = format!("{project}:");
        self.by_process.retain(|key, _| !key.starts_with(&prefix));
    }
}

// ------------------------------------------------------------- the snapshot

/// One process, as the screen needs it.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    /// `group:name` — what every control verb takes.
    pub full_name: String,
    pub name: String,
    pub group: String,
    pub state: i64,
    pub state_name: String,
    /// supervisord's own one-line summary, which is where a FATAL says why.
    pub description: String,
    pub pid: i64,
    /// Seconds, when the transport reported enough to work it out.
    ///
    /// The command family cannot: `supervisorctl` prints an uptime already
    /// formatted and no timestamps to subtract. Absent rather than zero, so a
    /// screen can tell "not running" from "not known".
    pub uptime: Option<i64>,
    /// The uptime as `supervisorctl` printed it, when that is all there is.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub uptime_text: String,
    pub spawn_err: String,
    /// Derived by watching, not reported. See [`Watch`].
    pub restarts: i64,
    pub flapping: bool,
    /// What a probe of this process's own service found, when one is
    /// configured. `None` means nobody asked, which is not the same as passing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<CheckResult>,
}

/// How many processes are in each state worth counting separately.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub fatal: usize,
    /// Starting, stopping, backing off — the states that are on their way
    /// somewhere. Counted together because none of them is a resting place.
    pub other: usize,
    /// How many are flapping. Separate from the states above because a
    /// flapping process is usually counted as running, which is the problem.
    pub flapping: usize,
    /// How many are up and not answering their own health check. Separate for
    /// the same reason flapping is: they are counted as running, and that is
    /// exactly what makes them hard to see.
    #[serde(default)]
    pub failing: usize,
}

/// One look at one server.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub project: String,
    /// supervisord's own state name — `RUNNING`, or `RESTARTING` while it
    /// reloads. Not a process state.
    pub daemon: String,
    pub version: String,
    pub processes: Vec<Process>,
    pub summary: Summary,
}

fn field_str(row: &Value, key: &str) -> String {
    row.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn field_i64(row: &Value, key: &str) -> i64 {
    row.get(key).and_then(Value::as_i64).unwrap_or(0)
}

/// One row of `getAllProcessInfo`, as the screen needs it.
pub fn normalize(row: &Value) -> Process {
    let state = field_i64(row, "state");
    let start = field_i64(row, "start");
    let now = field_i64(row, "now");

    // Only while running, and only when both timestamps are there. supervisord
    // leaves `start` set on a process that has since exited, so the subtraction
    // would otherwise report how long ago it was started as how long it has
    // been up.
    let uptime = (state == 20 && start > 0 && now >= start).then_some(now - start);

    let state_name_text = {
        let reported = field_str(row, "statename");
        if reported.is_empty() {
            state_name(state).to_string()
        } else {
            reported
        }
    };

    Process {
        full_name: full_name(row),
        name: field_str(row, "name"),
        group: field_str(row, "group"),
        state,
        state_name: state_name_text,
        description: field_str(row, "description"),
        pid: field_i64(row, "pid"),
        uptime,
        uptime_text: field_str(row, "uptimeText"),
        spawn_err: field_str(row, "spawnerr"),
        restarts: 0,
        flapping: false,
        check: None,
    }
}

fn summarize(processes: &[Process]) -> Summary {
    let count = |want: i64| processes.iter().filter(|p| p.state == want).count();
    Summary {
        total: processes.len(),
        running: count(20),
        stopped: count(0),
        fatal: count(200),
        other: processes
            .iter()
            .filter(|p| !matches!(p.state, 0 | 20 | 200))
            .count(),
        flapping: processes.iter().filter(|p| p.flapping).count(),
        // Filled in by `attach_checks`, which runs after this: the probes are
        // network round trips and the table is built before they are made.
        failing: 0,
    }
}

/// Everything one server is doing, in one round trip where the transport allows.
///
/// The version and the state come back with the table rather than from a
/// second call, because a screen that shows a process list beside a stale
/// "connected" badge is one that will eventually show both at once and be
/// wrong about one of them.
pub async fn snapshot(watch: &mut Watch, target: &Target) -> Result<Snapshot> {
    // Three calls and not one batch. `supervisorctl` has no multicall, so a
    // batch here would be this loop with a wrapper around it — and the wrapper
    // was the only thing the RPC transports needed it for.
    let state = target.call("supervisor.getState", &[]).await?;
    let version = target
        .call("supervisor.getSupervisorVersion", &[])
        .await
        .ok();
    let table = target.call("supervisor.getAllProcessInfo", &[]).await?;

    let mut processes: Vec<Process> = table
        .as_array()
        .map(|rows| rows.iter().map(normalize).collect())
        .unwrap_or_default();

    // Sorted by the name a control verb takes, so the table has an order that
    // does not change between polls. supervisord returns configuration order,
    // which moves when somebody edits the config.
    processes.sort_by(|a, b| a.full_name.cmp(&b.full_name));

    watch.observe(&target.project, &mut processes);

    Ok(Snapshot {
        project: target.project.clone(),
        daemon: state
            .get("statename")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN")
            .to_string(),
        version: version
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default(),
        summary: summarize(&processes),
        processes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of the command family: text meant for a person, read
    /// back into the rows XML-RPC would have sent.
    #[test]
    fn supervisorctl_output_becomes_the_rows_xml_rpc_would_have_sent() {
        let stdout = "\
web:app                          RUNNING   pid 1234, uptime 1:23:45
web:worker                       RUNNING   pid 1235, uptime 0:00:12
pipeline:queue-worker-00         BACKOFF   Exited too quickly (process log may have details)
lonely                           STOPPED   Not started
flaky                            FATAL     Exited too quickly (process log may have details)
";
        let rows = parse_status(stdout);
        assert_eq!(rows.len(), 5);

        assert_eq!(rows[0]["group"], "web");
        assert_eq!(rows[0]["name"], "app");
        assert_eq!(rows[0]["state"], 20);
        assert_eq!(rows[0]["statename"], "RUNNING");
        assert_eq!(rows[0]["pid"], 1234);
        assert_eq!(rows[0]["uptimeText"], "1:23:45");

        // A process with no group is its own group, the same way supervisord
        // reports it.
        assert_eq!(rows[3]["group"], "lonely");
        assert_eq!(rows[3]["name"], "lonely");
        assert_eq!(rows[3]["state"], 0);
        assert_eq!(rows[3]["pid"], 0);

        assert_eq!(rows[2]["state"], 30);
        assert_eq!(rows[4]["state"], 200);
        assert!(
            rows[4]["spawnerr"]
                .as_str()
                .unwrap()
                .contains("too quickly"),
            "a FATAL row carries why"
        );
    }

    /// The banner `supervisorctl` prints when it cannot reach the daemon is
    /// not a process, and a table with a row called "unix:///var/run/..." in
    /// it is worse than an empty one.
    #[test]
    fn output_that_is_not_a_status_line_produces_no_row() {
        let noise = "\
unix:///var/run/supervisor.sock refused connection
error: <class 'FileNotFoundError'>, [Errno 2] No such file or directory
";
        assert!(parse_status(noise).is_empty());
        assert!(parse_status("").is_empty());
        assert!(parse_status("   \n\n  ").is_empty());
    }

    /// `supervisorctl` puts the process name in front of the word, so a rule
    /// anchored to the start of the line calls every refusal a success.
    #[test]
    fn a_refusal_is_recognised_wherever_supervisorctl_puts_the_word() {
        // Copied from a real daemon.
        assert!(Target::is_refusal("nosuch: ERROR (no such process)"));
        assert!(Target::is_refusal("steady: ERROR (already started)"));
        assert!(Target::is_refusal("web:app: ERROR (not running)"));

        // And the successes are not refusals.
        assert!(!Target::is_refusal("steady: started"));
        assert!(!Target::is_refusal("steady: stopped"));
        assert!(!Target::is_refusal("pipeline:sleeper-00: started"));
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-supervisor-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn row(name: &str, state: i64, pid: i64) -> Value {
        json!({
            "name": name,
            "group": "web",
            "state": state,
            "statename": state_name(state),
            "pid": pid,
            "start": 1_700_000_000i64,
            "now": 1_700_000_060i64,
        })
    }

    /// The whole reason [`Watch`] exists: supervisord reports the state a
    /// process is in, never how many times it has been in it.
    #[test]
    fn a_restart_is_a_pid_that_changed_between_two_looks() {
        let mut watch = Watch::default();
        let mut first = vec![normalize(&row("app", 20, 100))];
        watch.observe("shop", &mut first);

        // Nothing is claimed on the first look. A process that was already
        // running when the app opened has not restarted since.
        assert_eq!(first[0].restarts, 0);
        assert!(!first[0].flapping);

        let mut second = vec![normalize(&row("app", 20, 101))];
        watch.observe("shop", &mut second);
        assert_eq!(second[0].restarts, 1);

        // The same pid again is not another restart.
        let mut third = vec![normalize(&row("app", 20, 101))];
        watch.observe("shop", &mut third);
        assert_eq!(third[0].restarts, 1);

        // And a process with no pid — stopped — is not a restart either.
        let mut fourth = vec![normalize(&row("app", 0, 0))];
        watch.observe("shop", &mut fourth);
        assert_eq!(fourth[0].restarts, 1);
    }

    /// Three restarts inside a minute is the case the state column hides: the
    /// process reports RUNNING every time somebody looks.
    #[test]
    fn a_process_that_keeps_coming_back_is_flapping_even_while_it_reports_running() {
        let mut watch = Watch::default();
        for pid in [10, 11, 12, 13] {
            let mut rows = vec![normalize(&row("app", 20, pid))];
            watch.observe("shop", &mut rows);
            if pid == 13 {
                assert_eq!(rows[0].state_name, "RUNNING", "the state still says fine");
                assert!(rows[0].flapping, "and this is what says it is not");
                assert_eq!(rows[0].restarts, 3);
            } else {
                assert!(!rows[0].flapping, "not yet at pid {pid}");
            }
        }
    }

    /// Two servers can run a process with the same name, and they are not the
    /// same process.
    #[test]
    fn the_history_is_kept_per_project() {
        let mut watch = Watch::default();
        for project in ["shop", "blog"] {
            let mut rows = vec![normalize(&row("app", 20, 10))];
            watch.observe(project, &mut rows);
        }
        let mut moved = vec![normalize(&row("app", 20, 11))];
        watch.observe("shop", &mut moved);
        assert_eq!(moved[0].restarts, 1);

        let mut untouched = vec![normalize(&row("app", 20, 10))];
        watch.observe("blog", &mut untouched);
        assert_eq!(untouched[0].restarts, 0, "the other project is unaffected");

        watch.forget("shop");
        let mut again = vec![normalize(&row("app", 20, 12))];
        watch.observe("shop", &mut again);
        assert_eq!(again[0].restarts, 0, "a forgotten project starts over");
    }

    /// `start` stays set on a process that has exited, so subtracting it would
    /// report how long ago it ran as how long it has been up.
    #[test]
    fn uptime_is_absent_rather_than_wrong_when_a_process_is_not_running() {
        let running = normalize(&row("app", 20, 5));
        assert_eq!(running.uptime, Some(60));

        let exited = normalize(&row("app", 100, 0));
        assert_eq!(exited.uptime, None);

        // And absent, not zero, when the transport never reported timestamps.
        let from_text = normalize(&json!({
            "name": "app", "group": "web", "state": 20, "statename": "RUNNING",
            "pid": 7, "uptimeText": "1:23:45",
        }));
        assert_eq!(from_text.uptime, None);
        assert_eq!(from_text.uptime_text, "1:23:45");
    }

    /// The summary is what the server list shows, so the states it collapses
    /// have to be the ones that mean the same thing to a reader.
    #[test]
    fn the_summary_counts_flapping_separately_from_the_state_it_reports_as() {
        let mut watch = Watch::default();
        let mut rows = vec![
            normalize(&row("a", 20, 1)),
            normalize(&row("b", 0, 0)),
            normalize(&row("c", 200, 0)),
            normalize(&row("d", 30, 0)),
        ];
        watch.observe("shop", &mut rows);
        let summary = summarize(&rows);

        assert_eq!(summary.total, 4);
        assert_eq!(summary.running, 1);
        assert_eq!(summary.stopped, 1);
        assert_eq!(summary.fatal, 1);
        assert_eq!(summary.other, 1, "BACKOFF is on its way somewhere");
        assert_eq!(summary.flapping, 0);

        // A flapping process is still counted as running — that is the point
        // of counting it twice.
        rows[0].flapping = true;
        let summary = summarize(&rows);
        assert_eq!(summary.running, 1);
        assert_eq!(summary.flapping, 1);
    }

    /// Three failures that look the same on screen and send somebody to three
    /// different places, told apart by text nobody here controls.
    #[test]
    fn the_reason_a_container_has_no_process_table_is_read_out_of_the_failure() {
        // Docker, before anything in the container ran.
        assert_eq!(
            classify("Error response from daemon: container abc is not running"),
            Reach::Stopped
        );
        assert_eq!(
            classify("Error: No such container: stackvo-shop"),
            Reach::Stopped
        );

        // Docker again: the image has no supervisorctl in it.
        assert_eq!(
            classify(
                "OCI runtime exec failed: exec failed: unable to start container process: exec: \
                 \"supervisorctl\": executable file not found in $PATH"
            ),
            Reach::NoSupervisord
        );

        // supervisord's own refusal. Both of these were read off a container:
        // the first from a daemon whose config predates the socket, the second
        // from one that is up but not listening.
        //
        // Note where they arrive. `supervisorctl` writes them to **stdout**
        // and exits 4 — a classifier reading stderr alone sees nothing, calls
        // it reachable, and the screen then shows an empty table with no
        // explanation on it.
        assert_eq!(
            classify("unix:///var/run/supervisor.sock no such file"),
            Reach::NoSocket
        );
        assert_eq!(
            classify("unix:///var/run/supervisor.sock refused connection"),
            Reach::NoSocket
        );

        // Nothing said at all is a daemon that answered. So is a daemon
        // reporting a dead process — `supervisorctl status` exits non-zero
        // whenever anything is down, which is why the exit code decides
        // nothing here.
        assert_eq!(classify(""), Reach::Ok);
        assert_eq!(
            classify("php-fpm  FATAL  can't find command '/usr/local/sbin/php-fpm'"),
            Reach::Ok,
            "a daemon reporting a dead process is a daemon that answered"
        );
    }

    /// A check that cannot be made is refused at the form, not discovered as a
    /// row that says "not a kind of check" for ever.
    #[test]
    fn a_check_is_refused_before_it_is_stored() {
        let check = |kind: &str, target: &str| Check {
            project: "shop".into(),
            process: "web:app".into(),
            kind: kind.into(),
            target: target.into(),
            expect_status: None,
            timeout_ms: None,
        };

        assert!(check("http", "https://shop.loc/up").validate().is_ok());
        assert!(
            check("http", "shop.loc/up").validate().is_err(),
            "no scheme"
        );
        assert!(check("tcp", "127.0.0.1:6379").validate().is_ok());
        assert!(check("tcp", "127.0.0.1").validate().is_err(), "no port");
        assert!(check("tcp", "127.0.0.1:0").validate().is_err(), "port zero");
        // An IPv6 literal is full of colons and only the last one is the port.
        assert!(check("tcp", "[::1]:6379").validate().is_ok());
        assert!(check("ping", "127.0.0.1").validate().is_err());

        let mut nameless = check("tcp", "127.0.0.1:80");
        nameless.process = "  ".into();
        assert!(nameless.validate().is_err(), "a check is about a process");
    }

    #[test]
    fn a_check_survives_being_written_and_replaced() {
        let root = temp("checks");
        let check = |process: &str, target: &str| Check {
            project: "shop".into(),
            process: process.into(),
            kind: "http".into(),
            target: target.into(),
            expect_status: None,
            timeout_ms: None,
        };

        assert!(checks(&root).unwrap().is_empty(), "none is a state");

        upsert_check(&root, check("php-fpm", "https://shop.loc/up")).unwrap();
        upsert_check(&root, check("nginx", "https://shop.loc/")).unwrap();
        assert_eq!(checks(&root).unwrap().len(), 2);

        // One check per process: the same process replaces rather than adds.
        upsert_check(&root, check("php-fpm", "https://shop.loc/health")).unwrap();
        let all = checks(&root).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(
            all.iter().find(|c| c.process == "php-fpm").unwrap().target,
            "https://shop.loc/health"
        );

        // Kept apart by project, because two projects run processes with the
        // same names and they are not the same process.
        let mut elsewhere = check("php-fpm", "http://10.0.0.4/up");
        elsewhere.project = "blog".into();
        upsert_check(&root, elsewhere).unwrap();
        assert_eq!(checks_for(&root, "shop").len(), 2);
        assert_eq!(checks_for(&root, "blog").len(), 1);

        remove_check(&root, "shop", "php-fpm").unwrap();
        assert_eq!(checks_for(&root, "shop").len(), 1);
        assert_eq!(
            checks_for(&root, "blog").len(),
            1,
            "the other project is untouched"
        );
    }

    /// A probe that could not be made is a failing check, not an error: the
    /// caller is drawing a row, and an error would take the row's other
    /// columns with it.
    #[tokio::test]
    async fn a_probe_that_cannot_be_made_fails_the_check_rather_than_the_row() {
        let unreachable = Check {
            project: "shop".into(),
            process: "web:app".into(),
            kind: "tcp".into(),
            // Port 1 on loopback: nothing listens, and it refuses at once
            // rather than hanging, so this does not spend the timeout.
            target: "127.0.0.1:1".into(),
            expect_status: None,
            timeout_ms: Some(1000),
        };
        let result = probe(&unreachable).await;
        assert!(!result.ok);
        assert!(!result.detail.is_empty(), "it says what refused");
        assert!(result.detail.len() < 120, "short enough for a table cell");

        let nonsense = Check {
            kind: "ping".into(),
            ..unreachable.clone()
        };
        assert!(!probe(&nonsense).await.ok);
    }

    /// The count exists because these processes are counted as running, which
    /// is exactly what makes them hard to see.
    #[test]
    fn a_process_that_is_up_and_not_answering_is_counted_separately() {
        let mut snapshot = Snapshot {
            project: "shop".into(),
            processes: vec![
                normalize(&row("app", 20, 1)),
                normalize(&row("worker", 20, 2)),
            ],
            ..Default::default()
        };
        snapshot.summary = summarize(&snapshot.processes);
        assert_eq!(snapshot.summary.running, 2);
        assert_eq!(snapshot.summary.failing, 0);

        snapshot.processes[0].check = Some(CheckResult {
            ok: false,
            detail: "HTTP 502, expected 200".into(),
            ms: 12,
        });
        snapshot.summary.failing += 1;

        // Still running. That is the point of counting it twice.
        assert_eq!(snapshot.summary.running, 2);
        assert_eq!(snapshot.summary.failing, 1);
    }

    /// A command that is going to read its standard input has to be given
    /// one. The failure without this is silent and destructive: `tee` exits 0
    /// and writes an empty file.
    ///
    /// Asserted against the argv rather than against a running engine. The
    /// version before this ran `docker exec <container> false` and checked that
    /// it exited non-zero — which it does with or without `-i`, so the test
    /// could not fail the way it was written for, and on a runner with no
    /// `docker` binary it failed for a reason that says nothing about this
    /// code (CI, macos-latest).
    #[test]
    fn a_docker_command_with_input_is_given_a_standard_input() {
        let with = docker_argv("shop", &["tee".into(), "/etc/x.conf".into()], true);
        let without = docker_argv("shop", &["tee".into(), "/etc/x.conf".into()], false);

        assert_eq!(
            with,
            ["docker", "exec", "-i", "shop", "tee", "/etc/x.conf"],
            "a command being written to needs `-i`, and it goes before the container",
        );
        assert_eq!(
            without,
            ["docker", "exec", "shop", "tee", "/etc/x.conf"],
            "and a command with nothing to read must not be given an open input",
        );
    }

    /// The argv is passed through, never re-split.
    ///
    /// `docker exec` execs the vector it is given, so a word with a space in it
    /// is one argument. A shell anywhere in this path would make it two, and
    /// this module exists partly to not have one.
    #[test]
    fn a_word_with_a_space_in_it_stays_one_word() {
        let argv = docker_argv(
            "shop",
            &["supervisorctl".into(), "start php-fpm nginx".into()],
            false,
        );

        assert_eq!(argv.len(), 5, "{argv:?}");
        assert_eq!(argv[4], "start php-fpm nginx");
    }

    fn snap(rows: Vec<Process>) -> Snapshot {
        Snapshot {
            project: "shop".into(),
            processes: rows,
            ..Default::default()
        }
    }

    fn failing(detail: &str) -> Option<CheckResult> {
        Some(CheckResult {
            ok: false,
            detail: detail.into(),
            ms: 5,
        })
    }

    /// A process that was already broken when this app opened did not just
    /// break. Announcing it would be announcing the past.
    #[test]
    fn the_first_look_records_and_says_nothing() {
        let mut alarms = Alarms::default();
        let mut dead = normalize(&row("app", 200, 0));
        dead.flapping = true;
        dead.check = failing("HTTP 502");

        assert!(alarms.changed("shop", &snap(vec![dead])).is_empty());
    }

    /// Edge, not level. A process sitting in FATAL is not news every twenty
    /// seconds — it was news once.
    #[test]
    fn a_state_is_reported_when_it_becomes_true_and_not_while_it_stays_true() {
        let mut alarms = Alarms::default();
        let healthy = || normalize(&row("app", 20, 5));

        alarms.changed("shop", &snap(vec![healthy()]));

        let mut broken = normalize(&row("app", 200, 0));
        broken.spawn_err = "Exited too quickly".into();
        let raised = alarms.changed("shop", &snap(vec![broken.clone()]));
        assert_eq!(raised.len(), 1);
        assert_eq!(raised[0].kind, Alarm::Fatal);
        assert_eq!(raised[0].process, "web:app");
        assert_eq!(raised[0].project, "shop");
        assert_eq!(raised[0].detail, "Exited too quickly", "it carries why");

        // Still FATAL, and silent.
        assert!(alarms
            .changed("shop", &snap(vec![broken.clone()]))
            .is_empty());

        // Recovered, then broken again — which is news a second time.
        alarms.changed("shop", &snap(vec![healthy()]));
        assert_eq!(alarms.changed("shop", &snap(vec![broken])).len(), 1);
    }

    #[test]
    fn flapping_and_a_check_that_stopped_answering_are_reported_too() {
        let mut alarms = Alarms::default();
        alarms.changed("shop", &snap(vec![normalize(&row("app", 20, 5))]));

        let mut wobbling = normalize(&row("app", 20, 6));
        wobbling.flapping = true;
        wobbling.restarts = 4;
        wobbling.check = failing("HTTP 502, expected 200");

        let raised = alarms.changed("shop", &snap(vec![wobbling]));
        let kinds: Vec<Alarm> = raised.iter().map(|r| r.kind).collect();
        assert!(kinds.contains(&Alarm::Flapping));
        assert!(kinds.contains(&Alarm::NotAnswering));
        assert!(
            !kinds.contains(&Alarm::Fatal),
            "it never left RUNNING, which is the point of both"
        );
        assert!(raised.iter().any(|r| r.detail.contains("502")));
    }

    /// A check that has never run is not a check that failed.
    #[test]
    fn a_process_with_no_check_is_not_a_process_that_stopped_answering() {
        let mut alarms = Alarms::default();
        alarms.changed("shop", &snap(vec![normalize(&row("app", 20, 5))]));
        let raised = alarms.changed("shop", &snap(vec![normalize(&row("app", 20, 5))]));
        assert!(raised.is_empty());
    }

    #[test]
    fn forgetting_a_project_leaves_the_others_alone() {
        let mut alarms = Alarms::default();
        alarms.changed("shop", &snap(vec![normalize(&row("app", 20, 5))]));
        alarms.changed("blog", &snap(vec![normalize(&row("app", 20, 5))]));

        alarms.forget("shop");

        // Forgotten means the next look is a first look, which says nothing.
        assert!(alarms
            .changed("shop", &snap(vec![normalize(&row("app", 200, 0))]))
            .is_empty());
        // And the other server still remembers, so its change is reported.
        assert_eq!(
            alarms
                .changed("blog", &snap(vec![normalize(&row("app", 200, 0))]))
                .len(),
            1
        );
    }

    #[test]
    fn the_state_names_and_codes_agree_in_both_directions() {
        for name in [
            "STOPPED", "STARTING", "RUNNING", "BACKOFF", "STOPPING", "EXITED", "FATAL",
        ] {
            assert_eq!(state_name(state_code(name)), name, "{name} round-trips");
        }
        assert_eq!(state_code("SOMETHING_ELSE"), 1000);
        assert_eq!(state_name(1000), "UNKNOWN");
    }
}
