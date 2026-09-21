//! Long-running processes a project declares, run by the supervisord that is
//! already in its container.
//!
//! ## The bug this closes
//!
//! An nginx or caddy project's container runs `supervisord`, and the config it
//! runs is **generated** — `render_supervisord_conf` writes it under
//! `generated/projects/<name>/`, the Dockerfile `COPY`s it in, and every
//! rebuild starts from that rendered file. So the obvious way to add a queue
//! worker beside `php-fpm` — `[include]` a file, `supervisorctl update` — works
//! exactly until the next rebuild, when the generated file is written again
//! from what the generator knows, which is nothing about the worker. The two
//! processes disappear, `supervisorctl status` shows the two that came with
//! the image, and nothing says why. Measured on a real project, twice.
//!
//! The Debian `[include] files = conf.d/*.conf` does not rescue this either:
//! the image starts supervisord with `-c /etc/supervisor/conf.d/supervisord.conf`,
//! so the generated file **is** the root config and includes nothing.
//!
//! ## The fix is where the declaration lives, not where the daemon reads it
//!
//! A process the generator does not know about is one it cannot keep. So the
//! declaration goes into `stackvo.json`, beside `hooks`, `schedule` and
//! `commands`, and the renderer writes a `[program:]` block for each one into
//! the same file it already writes. A rebuild then *restores* the worker
//! rather than losing it, and a clone gets it, because the definition travels
//! with the repository the way the rest of the environment does.
//!
//! ```json
//! "processes": {
//!   "scheduler": { "exec": ["php", "artisan", "schedule:work"] },
//!   "queue": {
//!     "exec": ["php", "artisan", "queue:work", "rabbitmq", "--queue=photos", "--tries=3"],
//!     "replicas": 2,
//!     "stopWait": 30
//!   }
//! }
//! ```
//!
//! ## Why not the two mechanisms that already exist
//!
//! [`crate::worker`] runs a **sidecar container** per Laravel worker, from a
//! fixed argv. It cannot take `queue:work rabbitmq --queue=a,b,c`, and it
//! cannot run anything that is not one of its six Laravel kinds. [`crate::cron`]
//! runs a command **on a timer**; a worker crammed into it becomes
//! `--stop-when-empty --max-time=110` every two minutes, which is the shape the
//! project this was built for had been forced into. A long process wants a
//! supervisor, and this container has one. This is the third shape, and the
//! distinctions are the design: a schedule entry starts and exits, a worker
//! sidecar is a fixed Laravel command in its own container, a process here is
//! **any argv, under the container's own daemon, for as long as the container
//! is up.**
//!
//! ## Same container, same rule, no gate
//!
//! A process runs inside the project's own container and nowhere else, which
//! is the line [`crate::hooks`] draws: that container already runs the
//! repository's code, so a repository able to name a command in it has gained
//! nothing it did not already have. There is no host form here and cannot be
//! one — supervisord is in the container.
//!
//! ## An argv array. There is no shell.
//!
//! supervisord's `command=` is one line that the daemon splits with Python's
//! `shlex`, and the argv reaches it through [`command_line`], which quotes
//! every word so that split gives back exactly the array in the file. Nothing
//! between the manifest and `execve` expands, globs or splits a string that
//! came out of a repository. A process that needs a pipeline is a script, and
//! a script is one argv element — the same answer hooks give.
//!
//! ## Two paths in, one config out
//!
//! The rendered block reaches the daemon two ways, and both come from the one
//! renderer so they cannot disagree: a **rebuild** bakes it into the image
//! through the generated file, and `supervisor_apply` pushes the same text
//! into a **running** container and asks the daemon to re-read it, so a worker
//! added to the manifest starts now rather than after the next build.

use crate::hooks::Problem;
use serde::Serialize;

/// The manifest key.
pub const KEY: &str = "processes";

/// Program names the generated config already uses. A declared process with
/// one of these would be a second `[program:php-fpm]` in the same file, and
/// supervisord would refuse the whole config — with the web server in it.
pub const RESERVED: [&str; 3] = ["php-fpm", "nginx", "caddy"];

/// How many a project may declare. Sixteen is more than any real project has
/// and few enough that a mistake — a loop writing the manifest — is caught.
pub const MAX: usize = 16;

pub const MAX_ARGV: usize = 32;
pub const MAX_REPLICAS: u32 = 16;
pub const MAX_STOP_WAIT: u32 = 600;
/// supervisord's own default for `stopwaitsecs`.
pub const DEFAULT_STOP_WAIT: u32 = 10;

/// Where the project is mounted, and therefore the working directory of every
/// process — `php artisan …` resolves against it exactly as it does for a
/// hook or a scheduled job.
pub const WORKDIR: &str = "/var/www/html";

/// Where a declared process writes, one file per process.
///
/// **A file, not `/dev/stdout`, and the difference is the log button.** The
/// image's own two programs write to the container's stdout, which `docker
/// logs` and the Logs tab show — and which supervisord cannot read back:
/// `supervisorctl tail` on a device answers `ERROR (unknown error reading
/// log)`, so the pane could show nothing for them and said so. A queue worker
/// is the process whose last hundred lines somebody actually opens the pane
/// for, so its log is a file supervisord owns and can tail, rotated by
/// supervisord at [`LOG_MAX`] with [`LOG_BACKUPS`] kept, and stderr folded
/// into it so there is one file to read rather than two to guess between.
///
/// Directly under `/var/log` rather than in `/var/log/supervisor/`, because
/// that directory does not exist in the generated image and supervisord
/// refuses the *whole* config — web server included — over a log path whose
/// directory is missing. `/var/log` is there in every image.
pub const LOG_DIR: &str = "/var/log";
pub const LOG_MAX: &str = "10MB";
pub const LOG_BACKUPS: u32 = 3;

/// The log file for a process, or the pattern for its copies.
///
/// With replicas the copies are told apart by `%(process_num)02d`, which
/// supervisord expands per copy, exactly as it does in `process_name`.
pub fn log_path(id: &str, replicas: u32) -> String {
    if replicas > 1 {
        format!("{LOG_DIR}/supervisor-{id}_%(process_num)02d.log")
    } else {
        format!("{LOG_DIR}/supervisor-{id}.log")
    }
}

/// One declared process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    /// The key in the file, and the `[program:<id>]` name in the daemon.
    pub id: String,
    /// Program first, then arguments. Never passed to a shell.
    pub exec: Vec<String>,
    /// A paused process stays in the file and out of the daemon's config.
    /// Pausing by deleting would lose the argv, which is the part that took
    /// effort to write — the same rule a scheduled job follows.
    pub enabled: bool,
    /// How many copies supervisord runs. Above one they are named
    /// `<id>_00`, `<id>_01`, … in one group called `<id>`.
    pub replicas: u32,
    /// Seconds supervisord waits after SIGTERM before it kills. A queue worker
    /// finishing a job wants longer than the default ten.
    pub stop_wait: u32,
}

impl Process {
    /// The command as one line, for a screen.
    ///
    /// Display only, on the same terms as [`crate::cron::Job::display`]:
    /// nothing parses this back.
    pub fn display(&self) -> String {
        self.exec.join(" ")
    }
}

/// Serialise the list as the file spells it: an object keyed by id, with the
/// defaults left out.
///
/// The reader's output is posted back through `project_manifest_write`, so
/// what `Manifest` serialises has to be what `parse` accepts — a list of
/// `{id, …}` objects would be dropped on the first save, which is the bug
/// `providers` closed with the same shape of function.
pub fn as_declared_map<S>(processes: &[Process], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;

    let mut map = serializer.serialize_map(Some(processes.len()))?;
    for process in processes {
        // Built by hand rather than from a struct: the file's spelling puts
        // the defaults away — `enabled` only when false, `replicas` only above
        // one, `stopWait` only when it is not supervisord's own ten — and
        // `skip_serializing_if` can skip a field but cannot skip it by value.
        let mut body = serde_json::Map::new();
        body.insert("exec".into(), serde_json::json!(process.exec));
        if !process.enabled {
            body.insert("enabled".into(), serde_json::Value::Bool(false));
        }
        if process.replicas != 1 {
            body.insert("replicas".into(), serde_json::json!(process.replicas));
        }
        if process.stop_wait != DEFAULT_STOP_WAIT {
            body.insert("stopWait".into(), serde_json::json!(process.stop_wait));
        }
        map.serialize_entry(&process.id, &serde_json::Value::Object(body))?;
    }
    map.end()
}

/// Is this usable as a `[program:]` name and as a key in the file?
///
/// Lower-case, digits, `-` and `_`, starting with a letter or digit, at most
/// forty characters. supervisord addresses a process as `group:name`, so a
/// colon is out; a space would be two words in `supervisorctl`.
pub fn is_valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    id.len() <= 40
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Read a `processes` block, naming everything wrong with it.
///
/// Warnings, never errors, on the same terms as `schedule`: a project with one
/// unreadable process still has a web server to start, and a typo here must
/// not be why a project cannot be opened. The process that could not be read
/// is left out and the reason is reported against its key.
pub fn parse(json: &serde_json::Value) -> (Vec<Process>, Vec<Problem>) {
    let mut processes = Vec::new();
    let mut problems = Vec::new();

    let Some(block) = json.get(KEY) else {
        return (processes, problems);
    };
    let Some(object) = block.as_object() else {
        problems.push(Problem {
            path: KEY.into(),
            message: "`processes` must be an object keyed by process id".into(),
        });
        return (processes, problems);
    };

    for (id, entry) in object {
        let path = format!("{KEY}.{id}");

        if !is_valid_id(id) {
            problems.push(Problem {
                path,
                message: format!(
                    "`{id}` is not a process id: lower-case letters, digits, `-` and `_`, at most 40 characters"
                ),
            });
            continue;
        }
        if RESERVED.contains(&id.as_str()) {
            problems.push(Problem {
                path,
                message: format!(
                    "`{id}` is the name of a process the generated image already runs"
                ),
            });
            continue;
        }
        if processes.len() >= MAX {
            problems.push(Problem {
                path,
                message: format!("a project may declare at most {MAX} processes"),
            });
            continue;
        }

        let Some(fields) = entry.as_object() else {
            problems.push(Problem {
                path,
                message: format!("`{id}` must be an object with an `exec` array"),
            });
            continue;
        };

        // An unknown key is reported rather than ignored. `replica` for
        // `replicas` would otherwise run one copy and say nothing, which is
        // the class of silence the whole manifest reader refuses.
        let mut unknown: Vec<&str> = fields
            .keys()
            .map(String::as_str)
            .filter(|k| !matches!(*k, "exec" | "enabled" | "replicas" | "stopWait"))
            .collect();
        if !unknown.is_empty() {
            unknown.sort_unstable();
            problems.push(Problem {
                path: path.clone(),
                message: format!(
                    "`{id}` has a key this reader does not know: {}",
                    unknown.join(", ")
                ),
            });
        }

        let exec = match fields.get("exec").and_then(|v| v.as_array()) {
            Some(array) => {
                let argv: Vec<String> = array
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                if argv.len() != array.len() {
                    problems.push(Problem {
                        path,
                        message: format!("`{id}`: every element of `exec` must be a string"),
                    });
                    continue;
                }
                argv
            }
            None => {
                problems.push(Problem {
                    path,
                    message: format!("`{id}` has no `exec` array"),
                });
                continue;
            }
        };
        if exec.is_empty() || exec[0].trim().is_empty() {
            problems.push(Problem {
                path,
                message: format!("`{id}`: `exec` names no program"),
            });
            continue;
        }
        if exec.len() > MAX_ARGV {
            problems.push(Problem {
                path,
                message: format!("`{id}`: `exec` has more than {MAX_ARGV} words"),
            });
            continue;
        }
        // A newline in a word is a second line in a config whose grammar is
        // line-oriented — a second directive nobody wrote. Refused rather than
        // escaped, as `site.rs` does for the same reason.
        if exec
            .iter()
            .any(|a| a.contains('\n') || a.contains('\r') || a.contains('\0'))
        {
            problems.push(Problem {
                path,
                message: format!("`{id}`: a word of `exec` contains a line break"),
            });
            continue;
        }

        let enabled = match fields.get("enabled") {
            None => true,
            Some(serde_json::Value::Bool(b)) => *b,
            Some(_) => {
                problems.push(Problem {
                    path,
                    message: format!("`{id}`: `enabled` must be true or false"),
                });
                continue;
            }
        };

        let replicas = match fields.get("replicas") {
            None => 1,
            Some(v) => {
                match v.as_u64() {
                    Some(n) if (1..=u64::from(MAX_REPLICAS)).contains(&n) => n as u32,
                    _ => {
                        problems.push(Problem {
                        path,
                        message: format!("`{id}`: `replicas` must be a whole number from 1 to {MAX_REPLICAS}"),
                    });
                        continue;
                    }
                }
            }
        };

        let stop_wait = match fields.get("stopWait") {
            None => DEFAULT_STOP_WAIT,
            Some(v) => match v.as_u64() {
                Some(n) if (1..=u64::from(MAX_STOP_WAIT)).contains(&n) => n as u32,
                _ => {
                    problems.push(Problem {
                        path,
                        message: format!("`{id}`: `stopWait` must be a whole number of seconds from 1 to {MAX_STOP_WAIT}"),
                    });
                    continue;
                }
            },
        };

        processes.push(Process {
            id: id.clone(),
            exec,
            enabled,
            replicas,
            stop_wait,
        });
    }

    (processes, problems)
}

/// The processes that go into the daemon's config.
pub fn runnable(processes: &[Process]) -> Vec<&Process> {
    processes.iter().filter(|p| p.enabled).collect()
}

/// One word of a `command=` line, quoted so that supervisord's `shlex.split`
/// gives the word back unchanged.
///
/// Bare when it is made of characters no shell grammar reads; otherwise in
/// single quotes, with an embedded quote written as `'\''` — close, escaped
/// quote, reopen — which is the one form POSIX `shlex` accepts. The `%` is
/// doubled afterwards because supervisord runs the whole line through
/// Python's `%` formatting for `%(ENV_X)s`, and a bare `%` there is an error
/// rather than a character.
pub fn shell_word(word: &str) -> String {
    let bare = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    let quoted = if bare {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    };
    quoted.replace('%', "%%")
}

/// The `command=` value for an argv.
pub fn command_line(exec: &[String]) -> String {
    exec.iter()
        .map(|w| shell_word(w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The `[program:]` blocks for every enabled process, ready to append to the
/// generated `supervisord.conf`.
///
/// Each block is preceded by one blank line and ends without one, so the
/// rendered file keeps ending on its last key — which the generator's own
/// tests hold it to. Logs go to a file per process — see [`LOG_DIR`] for why
/// not the container's stdout.
///
/// `stopasgroup` and `killasgroup` because a declared process is often a
/// script or an artisan command that forks — `schedule:work` starts a child
/// per due task — and a stop that reached the parent only would leave the
/// children running under nobody.
pub fn render(processes: &[Process]) -> String {
    let mut out = String::new();
    for process in runnable(processes) {
        out.push_str(&format!(
            "\n[program:{id}]\n\
             command={command}\n\
             directory={WORKDIR}\n\
             autostart=true\n\
             autorestart=true\n\
             stopwaitsecs={stop_wait}\n\
             stopasgroup=true\n\
             killasgroup=true\n",
            id = process.id,
            command = command_line(&process.exec),
            stop_wait = process.stop_wait,
        ));
        if process.replicas > 1 {
            out.push_str(&format!(
                "numprocs={}\nprocess_name=%(program_name)s_%(process_num)02d\n",
                process.replicas
            ));
        }
        out.push_str(&format!(
            "stdout_logfile={log}\n\
             stdout_logfile_maxbytes={LOG_MAX}\n\
             stdout_logfile_backups={LOG_BACKUPS}\n\
             redirect_stderr=true\n",
            log = log_path(&process.id, process.replicas),
        ));
    }
    out
}

/// What the manifest declares and the daemon is not running, and the reverse.
///
/// `groups` is what `supervisorctl status` reported, by group — a process with
/// replicas is one group of several rows, and the group is what the manifest
/// named. `pending` is declared and enabled and absent from the daemon: the
/// manifest changed and nothing applied it, which is the state this whole
/// module exists to make visible. `stale` is in the daemon, not one of the
/// image's own programs, and not declared: added by hand inside the
/// container, or removed from the manifest — either way the next rebuild
/// drops it, and that should be a sentence on screen rather than a surprise.
pub fn drift(processes: &[Process], groups: &[String]) -> (Vec<String>, Vec<String>) {
    let declared: Vec<&str> = runnable(processes).iter().map(|p| p.id.as_str()).collect();

    let pending: Vec<String> = declared
        .iter()
        .filter(|id| !groups.iter().any(|g| g == *id))
        .map(|id| id.to_string())
        .collect();

    let mut stale: Vec<String> = Vec::new();
    for group in groups {
        if RESERVED.contains(&group.as_str()) || declared.contains(&group.as_str()) {
            continue;
        }
        if !stale.contains(group) {
            stale.push(group.clone());
        }
    }

    (pending, stale)
}

/// The `[program:<name>]` block of a supervisord config, as key/value pairs
/// in the order the file has them. `None` when there is no such block.
///
/// What the daemon is *actually* running with, read from the file in the
/// container rather than reconstructed from the manifest — the two agree
/// after a rebuild or an apply and disagree in between, and the detail view
/// exists to show which. The grammar is the daemon's own: `;` and `#`
/// comments, one `key=value` per line, the first `[` after the block ends it.
pub fn program_block(conf: &str, name: &str) -> Option<Vec<(String, String)>> {
    let header = format!("[program:{name}]");
    let mut out = Vec::new();
    let mut inside = false;
    let mut found = false;
    for raw in conf.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            if inside {
                break;
            }
            inside = line == header;
            found |= inside;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            out.push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    found.then_some(out)
}

/// What one process is costing, read from `/proc` inside the container.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    /// Resident set, in kilobytes — `VmRSS`.
    pub rss_kb: u64,
    pub threads: u64,
    /// CPU time consumed over the process's whole life, in seconds.
    pub cpu_seconds: f64,
    /// That time as a percentage of one core over the same span — the number
    /// `ps` prints as `%cpu`. A lifetime average rather than an instant, which
    /// is what one look can honestly give.
    pub cpu_percent: f64,
}

/// Kernel ticks per second, which is what `/proc/<pid>/stat` counts in.
///
/// `USER_HZ` has been 100 on every Linux the kernel ABI has shipped to, and
/// it is the one constant here that is not read from the container: asking
/// `getconf CLK_TCK` would be a fourth `exec` for a number that does not vary.
pub const CLK_TCK: f64 = 100.0;

/// Read one process's cost out of the concatenation of its `/proc/<pid>/status`,
/// `/proc/<pid>/stat` and `/proc/uptime` — one `cat`, three files.
///
/// The three are told apart by shape rather than position: `status` is
/// `Key:\tvalue` lines, `stat` is the one line with a parenthesised command
/// name, and `uptime` is two floats. So the order `cat` was given them in is
/// not load-bearing, and a file that was missing simply leaves its field out.
pub fn resources_from_proc(text: &str) -> Option<Resources> {
    let mut rss_kb = None;
    let mut threads = None;
    let mut ticks: Option<(f64, f64)> = None; // (utime + stime, starttime)
    let mut uptime = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            rss_kb = rest.split_whitespace().next().and_then(|n| n.parse().ok());
        } else if let Some(rest) = line.strip_prefix("Threads:") {
            threads = rest.trim().parse().ok();
        } else if let Some(close) = line.rfind(')') {
            // stat: `<pid> (<comm>) <state> …` — the command name may hold
            // spaces and parentheses, so everything is counted from the LAST
            // closing parenthesis. After it, state is field 3, so utime (14),
            // stime (15) and starttime (22) sit at 11, 12 and 19.
            if line.contains(" (") {
                let fields: Vec<&str> = line[close + 1..].split_whitespace().collect();
                if fields.len() > 19 {
                    let utime: f64 = fields[11].parse().ok()?;
                    let stime: f64 = fields[12].parse().ok()?;
                    let start: f64 = fields[19].parse().ok()?;
                    ticks = Some((utime + stime, start));
                }
            }
        } else if uptime.is_none() {
            let mut floats = line.split_whitespace().map(|f| f.parse::<f64>());
            if let (Some(Ok(up)), Some(Ok(_))) = (floats.next(), floats.next()) {
                uptime = Some(up);
            }
        }
    }

    let (used, start) = ticks?;
    let uptime = uptime?;
    let cpu_seconds = used / CLK_TCK;
    let alive = uptime - start / CLK_TCK;
    let cpu_percent = if alive > 0.0 {
        (cpu_seconds / alive * 1000.0).round() / 10.0
    } else {
        0.0
    };
    Some(Resources {
        rss_kb: rss_kb.unwrap_or(0),
        threads: threads.unwrap_or(0),
        cpu_seconds: (cpu_seconds * 100.0).round() / 100.0,
        cpu_percent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_one_program_block_out_of_the_daemons_file() {
        let conf = "[supervisord]\nnodaemon=true\n\n; the image's own\n[program:php-fpm]\ncommand=/usr/local/sbin/php-fpm -F\nautostart=true\n\n[program:queue]\ncommand=php artisan queue:work\n  directory = /var/www/html\n# a comment\nstopwaitsecs=30\n\n[program:nginx]\ncommand=nginx\n";
        assert_eq!(
            program_block(conf, "queue").unwrap(),
            vec![
                ("command".to_string(), "php artisan queue:work".to_string()),
                ("directory".to_string(), "/var/www/html".to_string()),
                ("stopwaitsecs".to_string(), "30".to_string()),
            ]
        );
        // The last block, with nothing after it to end it.
        assert_eq!(program_block(conf, "nginx").unwrap().len(), 1);
        // Absent is None, and an empty block is Some(empty).
        assert!(program_block(conf, "ghost").is_none());
        assert_eq!(program_block("[program:bare]\n", "bare"), Some(vec![]));
    }

    /// The lines as a php process in a real project container printed them,
    /// one `cat` over three files.
    #[test]
    fn reads_memory_threads_and_cpu_out_of_proc() {
        let text = "Name:\tphp\nState:\tS (sleeping)\nVmRSS:\t   58492 kB\nThreads:\t1\n\
                    10 (php) S 1 10 1 0 -1 4194560 6832 231 5 1 87 27 0 0 20 0 1 0 63607050 104767488 14332 0 0 0 0 0 0 0 0 0 0 0 0 0 0 17 10 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
                    636491.61 7364001.05\n";
        let r = resources_from_proc(text).unwrap();
        assert_eq!(r.rss_kb, 58492);
        assert_eq!(r.threads, 1);
        // (87 + 27) ticks at 100 Hz.
        assert_eq!(r.cpu_seconds, 1.14);
        // Alive for 636491.61 - 636070.50 = 421.11 s → 0.27 % of a core.
        assert_eq!(r.cpu_percent, 0.3);

        // A command name with spaces and a parenthesis in it does not move
        // the fields.
        let odd = text.replace("(php)", "(my (odd) name)");
        assert_eq!(resources_from_proc(&odd).unwrap().cpu_seconds, 1.14);

        // No stat line, no answer — memory alone is not a cost.
        assert!(resources_from_proc("VmRSS:\t10 kB\n636491.61 7364001.05\n").is_none());
    }

    fn parsed(block: serde_json::Value) -> (Vec<Process>, Vec<Problem>) {
        parse(&json!({ "name": "shop", "processes": block }))
    }

    #[test]
    fn absent_means_nothing_declared() {
        let (processes, problems) = parse(&json!({ "name": "shop" }));
        assert!(processes.is_empty());
        assert!(problems.is_empty());
    }

    #[test]
    fn reads_the_defaults_and_the_overrides() {
        let (processes, problems) = parsed(json!({
            "scheduler": { "exec": ["php", "artisan", "schedule:work"] },
            "queue": {
                "exec": ["php", "artisan", "queue:work", "rabbitmq", "--queue=a,b"],
                "replicas": 2,
                "stopWait": 30,
                "enabled": false
            }
        }));
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(processes.len(), 2);

        let scheduler = processes.iter().find(|p| p.id == "scheduler").unwrap();
        assert!(scheduler.enabled, "absent means on");
        assert_eq!(scheduler.replicas, 1);
        assert_eq!(scheduler.stop_wait, DEFAULT_STOP_WAIT);

        let queue = processes.iter().find(|p| p.id == "queue").unwrap();
        assert!(!queue.enabled);
        assert_eq!(queue.replicas, 2);
        assert_eq!(queue.stop_wait, 30);
        assert_eq!(queue.exec[4], "--queue=a,b");
    }

    #[test]
    fn a_bad_entry_is_a_problem_and_the_others_still_read() {
        let (processes, problems) = parsed(json!({
            "ok": { "exec": ["php", "artisan", "horizon"] },
            "php-fpm": { "exec": ["php-fpm"] },
            "Bad Name": { "exec": ["x"] },
            "noexec": { "about": "nothing" },
            "shellish": { "exec": "php artisan queue:work" },
            "empty": { "exec": [] },
            "toomany": { "exec": ["x"], "replicas": 99 },
            "typo": { "exec": ["x"], "replica": 2 },
            "newline": { "exec": ["sh", "-c", "a\nb"] }
        }));

        let ids: Vec<&str> = processes.iter().map(|p| p.id.as_str()).collect();
        // `typo` is read — the unknown key is reported, the process is kept —
        // and everything else with a real fault is left out.
        assert_eq!(ids, vec!["ok", "typo"], "{problems:?}");

        let paths: Vec<&str> = problems.iter().map(|p| p.path.as_str()).collect();
        for expected in [
            "processes.php-fpm",
            "processes.Bad Name",
            "processes.noexec",
            "processes.shellish",
            "processes.empty",
            "processes.toomany",
            "processes.typo",
            "processes.newline",
        ] {
            assert!(
                paths.contains(&expected),
                "{expected} missing from {paths:?}"
            );
        }
        assert!(problems
            .iter()
            .any(|p| p.path == "processes.php-fpm" && p.message.contains("already runs")));
        assert!(problems
            .iter()
            .any(|p| p.path == "processes.typo" && p.message.contains("replica")));
    }

    #[test]
    fn the_block_must_be_an_object() {
        let (processes, problems) = parsed(json!([{ "exec": ["x"] }]));
        assert!(processes.is_empty());
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].path, "processes");
    }

    #[test]
    fn ids_follow_the_program_name_rule() {
        for ok in ["queue", "q1", "0abc", "my-worker", "my_worker"] {
            assert!(is_valid_id(ok), "{ok}");
        }
        for bad in ["", "Queue", "-q", "a:b", "a b", "ä", &"x".repeat(41)] {
            assert!(!is_valid_id(bad), "{bad}");
        }
    }

    /// The whole reason the command is an argv: what the daemon splits has to
    /// be what the file said, word for word.
    #[test]
    fn every_word_survives_the_daemons_split() {
        assert_eq!(shell_word("php"), "php");
        assert_eq!(shell_word("--queue=a,b.c"), "--queue=a,b.c");
        assert_eq!(shell_word("daemon off;"), "'daemon off;'");
        assert_eq!(shell_word("it's"), "'it'\\''s'");
        assert_eq!(shell_word(""), "''");
        assert_eq!(shell_word("$HOME"), "'$HOME'");
        assert_eq!(shell_word("a*b"), "'a*b'");
        // supervisord's own expansion syntax must reach it inert.
        assert_eq!(shell_word("100%"), "100%%");
        assert_eq!(shell_word("%(ENV_HOME)s"), "'%%(ENV_HOME)s'");

        let argv: Vec<String> = ["sh", "-c", "echo 'hi' && exit 1"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(command_line(&argv), "sh -c 'echo '\\''hi'\\'' && exit 1'");
    }

    #[test]
    fn renders_one_block_per_enabled_process_and_ends_on_its_last_key() {
        let (processes, _) = parsed(json!({
            "queue": { "exec": ["php", "artisan", "queue:work"], "replicas": 3, "stopWait": 60 },
            "paused": { "exec": ["php", "artisan", "horizon"], "enabled": false },
            "scheduler": { "exec": ["php", "artisan", "schedule:work"] }
        }));
        let text = render(&processes);

        assert!(text.starts_with("\n[program:"), "{text:?}");
        assert!(text.ends_with("redirect_stderr=true\n"));
        assert!(!text.ends_with("\n\n"));
        // A file supervisord can tail, never the device it cannot.
        assert!(!text.contains("/dev/stdout"), "{text}");
        assert!(
            !text.contains("[program:paused]"),
            "a paused process is out of the config"
        );

        assert!(
            text.contains(
                "[program:queue]\n\
             command=php artisan queue:work\n\
             directory=/var/www/html\n\
             autostart=true\n\
             autorestart=true\n\
             stopwaitsecs=60\n\
             stopasgroup=true\n\
             killasgroup=true\n\
             numprocs=3\n\
             process_name=%(program_name)s_%(process_num)02d\n\
             stdout_logfile=/var/log/supervisor-queue_%(process_num)02d.log\n\
             stdout_logfile_maxbytes=10MB\n\
             stdout_logfile_backups=3\n\
             redirect_stderr=true\n"
            ),
            "{text}"
        );

        // One copy: no numprocs, no process_name, default stop wait.
        assert!(
            text.contains(
                "[program:scheduler]\n\
             command=php artisan schedule:work\n\
             directory=/var/www/html\n\
             autostart=true\n\
             autorestart=true\n\
             stopwaitsecs=10\n\
             stopasgroup=true\n\
             killasgroup=true\n\
             stdout_logfile=/var/log/supervisor-scheduler.log\n"
            ),
            "{text}"
        );
        assert_eq!(text.matches("numprocs=").count(), 1);
    }

    #[test]
    fn nothing_enabled_renders_nothing() {
        let (processes, _) = parsed(json!({
            "paused": { "exec": ["x"], "enabled": false }
        }));
        assert_eq!(render(&processes), "");
        assert_eq!(render(&[]), "");
    }

    #[test]
    fn drift_names_what_is_missing_and_what_is_extra() {
        let (processes, _) = parsed(json!({
            "queue": { "exec": ["x"] },
            "scheduler": { "exec": ["y"] },
            "paused": { "exec": ["z"], "enabled": false }
        }));

        // The daemon runs the image's two, the queue, and something somebody
        // added by hand. Two replica rows share one group.
        let groups: Vec<String> = ["php-fpm", "nginx", "queue", "queue", "by-hand"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (pending, stale) = drift(&processes, &groups);

        assert_eq!(pending, vec!["scheduler"], "paused is not pending");
        assert_eq!(
            stale,
            vec!["by-hand"],
            "the image's own programs are never stale"
        );

        // Nothing declared, nothing extra: quiet.
        let (pending, stale) = drift(&[], &["php-fpm".to_string(), "nginx".to_string()]);
        assert!(pending.is_empty() && stale.is_empty());
    }

    #[test]
    fn serialises_as_the_file_spells_it() {
        let (processes, _) = parsed(json!({
            "queue": { "exec": ["php", "artisan", "queue:work"], "replicas": 2, "stopWait": 30 },
            "scheduler": { "exec": ["php", "artisan", "schedule:work"], "enabled": false }
        }));

        #[derive(Serialize)]
        struct Wrap<'a> {
            #[serde(serialize_with = "as_declared_map")]
            processes: &'a [Process],
        }
        let value = serde_json::to_value(Wrap {
            processes: &processes,
        })
        .unwrap();

        assert_eq!(
            value,
            json!({
                "processes": {
                    "queue": { "exec": ["php", "artisan", "queue:work"], "replicas": 2, "stopWait": 30 },
                    "scheduler": { "exec": ["php", "artisan", "schedule:work"], "enabled": false }
                }
            })
        );

        // And the reader takes its own output back, unchanged.
        let (again, problems) = parse(&value);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(again, processes);
    }
}
