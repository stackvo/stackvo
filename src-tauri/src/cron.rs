//! Scheduled commands per project, with Docker doing the supervising.
//!
//! [`crate::worker`] already runs Laravel's own scheduler as a sidecar, and
//! that is the right answer for a project whose schedule lives in
//! `routes/console.php`. It is the *only* answer it can give, though: one
//! process, all-or-nothing, and nothing on screen about which entry last ran.
//! Every comparable tool ships a table of named jobs instead — a label, a
//! frequency, a last run, a log — and the reason is that "did my task run?" is
//! a question about one task rather than about a process.
//!
//! So this is that table. What it is *not* is a second execution model:
//!
//! ## The job runs in the project's own container
//!
//! Same image, same PHP, same extensions, same bind mount, same network — so
//! `.env` and the database resolve exactly as they do for the site, and a job
//! that works in the terminal works here. [`crate::hooks`] draws the line this
//! sits on and draws it around *where* a command runs rather than around what
//! it is: a command inside the project's own container "has gained nothing it
//! did not already have", because that container already runs the repository's
//! code. That is why a schedule needs no consent gate while a host hook does,
//! and it is also why there is deliberately no host-side job kind here. Adding
//! one would be adding the dangerous half of hooks on a timer, which is worse
//! than hooks: nobody is watching when it fires.
//!
//! ## A job is an argv array. There is no shell.
//!
//! ```json
//! "schedule": [
//!   { "label": "Cache cleanup", "cron": "*/5 * * * *",
//!     "exec": ["php", "artisan", "cache:clear"] }
//! ]
//! ```
//!
//! The same rule as hooks, for the same reason, with one extra edge: the text
//! reaches a shell script this module writes, so "no shell" has to survive a
//! trip through `sh`. It does, and [`tick_sh`] says how — the argv is stored
//! NUL-separated and handed to `env`, which execs its first argument. Nothing
//! ever splits, globs or expands a string that came out of a repository.
//!
//! ## Why a sidecar and not a timer in this app
//!
//! Same answer [`crate::worker`] gives: `--restart unless-stopped` is Docker's
//! own supervisor, and a schedule that only fires while a desktop app happens
//! to be open is a schedule that will be wrong on the morning somebody needed
//! it. The tick loop is POSIX `sh` rather than `cron` or `supercronic` because
//! the image is the *project's*, and this app does not get to add packages to
//! it — the generated Dockerfile is held to byte parity with what it produced
//! before. `sh`, `date`, `sleep` and `env` are in every image the generator can
//! produce, across all eight runtimes.
//!
//! ## Where the schedule lives
//!
//! In `stackvo.json`, beside `hooks` and `commands`, so it travels with the
//! repository the way the rest of a project's environment definition does. A
//! clone gets the schedule; a malformed entry is a manifest warning and the
//! other jobs still run.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Sidecar containers are `stackvo-cron-<project>`.
pub const ID_PREFIX: &str = "cron-";

/// Where the generated schedule is mounted inside the sidecar.
pub const SCHEDULE_DIR: &str = "/stackvo/schedule";

/// Where the sidecar writes what happened. Bound to the host so the app can
/// read a last run without entering the container.
pub const STATE_DIR: &str = "/stackvo/state";

/// One scheduled command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// What the reader called it. Also the name of its log, by way of
    /// [`slug`] — `cache-cleanup.log` is findable and `cron_1780135038297.log`
    /// is not.
    pub label: String,
    /// Five fields, already validated by [`parse_expression`].
    pub cron: String,
    /// Program first, then arguments. Never passed to a shell.
    pub exec: Vec<String>,
    /// A paused job stays in the file and out of the schedule. Pausing by
    /// deleting would lose the command, which is the thing that took effort to
    /// write.
    pub enabled: bool,
}

impl Job {
    /// The command as one line, for a screen.
    ///
    /// Display only, on the same terms as [`crate::hooks::Step::display`]:
    /// nothing parses this back, because an argv that round-tripped through a
    /// string would be one that can be re-split.
    pub fn display(&self) -> String {
        self.exec.join(" ")
    }

    /// The stable name this job's log and last-run file are keyed by.
    pub fn id(&self) -> String {
        slug(&self.label)
    }
}

/// A label as a filename: lower-cased, non-alphanumerics collapsed to `-`.
///
/// Renaming a job starts a new log rather than renaming the old one. That is
/// the honest behaviour for a file keyed by a name — the alternative is a
/// generated id nobody can find on disk, and the log people go looking for is
/// the one named after the job on screen.
pub fn slug(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut dash = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "job".to_string()
    } else {
        out
    }
}

/// Something wrong with a declaration, named rather than raised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub path: String,
    pub message: String,
}

// --------------------------------------------------------- cron expressions

/// The five fields, and what each one may hold.
///
/// Day-of-week accepts 7 as Sunday as well as 0, because `0` and `7` both
/// appear in the wild and rejecting one would reject a line somebody copied
/// from a working crontab.
const FIELDS: [(&str, u32, u32); 5] = [
    ("minute", 0, 59),
    ("hour", 0, 23),
    ("day of month", 1, 31),
    ("month", 1, 12),
    ("day of week", 0, 7),
];

/// Check a five-field expression, naming the first thing wrong with it.
///
/// Validated here and never in the container: the tick script reads its fields
/// rather than defending against them, and that is only safe because nothing
/// unvalidated is ever written for it to read.
///
/// The accepted grammar is the portable subset — `*`, `a`, `a-b`, `*/n`,
/// `a-b/n`, and comma-separated lists of those. Names (`MON`, `JAN`) and the
/// `@daily` macros are deliberately absent: every one of them is a second
/// spelling of something already expressible, and a second spelling is a second
/// thing the matcher has to agree with.
pub fn parse_expression(expression: &str) -> Result<(), String> {
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(format!(
            "a cron expression has five fields, and this has {}: `{expression}`",
            fields.len()
        ));
    }

    for (field, (name, lo, hi)) in fields.iter().zip(FIELDS) {
        for part in field.split(',') {
            check_part(part, name, lo, hi)?;
        }
    }
    Ok(())
}

fn check_part(part: &str, name: &str, lo: u32, hi: u32) -> Result<(), String> {
    let (range, step) = match part.split_once('/') {
        Some((range, step)) => {
            let step: u32 = step
                .parse()
                .map_err(|_| format!("`{step}` is not a step for the {name} field"))?;
            if step == 0 {
                return Err(format!("a step of 0 never comes round: `{part}`"));
            }
            (range, Some(step))
        }
        None => (part, None),
    };

    let bounds = if range == "*" {
        None
    } else if let Some((from, to)) = range.split_once('-') {
        Some((number(from, name, lo, hi)?, number(to, name, lo, hi)?))
    } else {
        let value = number(range, name, lo, hi)?;
        // A bare number with a step is `5/10`, which some crons read as `5-59/10`
        // and others reject. Rejected here rather than guessed at: the writer
        // meant one of them and this way they get to say which.
        if step.is_some() {
            return Err(format!(
                "`{part}` is ambiguous — write the range you mean, as in `{value}-{hi}/{}`",
                step.unwrap_or(1)
            ));
        }
        Some((value, value))
    };

    if let Some((from, to)) = bounds {
        if from > to {
            return Err(format!(
                "`{range}` runs backwards in the {name} field: {from} is after {to}"
            ));
        }
    }
    Ok(())
}

fn number(text: &str, name: &str, lo: u32, hi: u32) -> Result<u32, String> {
    let value: u32 = text
        .parse()
        .map_err(|_| format!("`{text}` is not a number in the {name} field"))?;
    if value < lo || value > hi {
        return Err(format!(
            "{value} is outside the {name} field, which is {lo}–{hi}"
        ));
    }
    Ok(value)
}

// --------------------------------------------------------------- the block

/// Read a `schedule` block, naming everything wrong with it.
///
/// Warnings rather than a failure, on the same terms as [`crate::hooks::parse`]:
/// a project with one unreadable job still has three that run, and refusing to
/// open it would be the wrong trade for a convenience.
pub fn parse(json: &serde_json::Value) -> (Vec<Job>, Vec<Problem>) {
    let mut jobs = Vec::new();
    let mut problems = Vec::new();

    let Some(block) = json.get("schedule") else {
        return (jobs, problems);
    };
    let Some(list) = block.as_array() else {
        problems.push(Problem {
            path: "schedule".into(),
            message: "`schedule` must be an array of jobs".into(),
        });
        return (jobs, problems);
    };

    let mut seen: Vec<String> = Vec::new();

    for (index, entry) in list.iter().enumerate() {
        let path = format!("schedule[{index}]");
        let Some(object) = entry.as_object() else {
            problems.push(Problem {
                path,
                message: "a job must be an object".into(),
            });
            continue;
        };

        let label = match object.get("label").and_then(|v| v.as_str()) {
            Some(label) if !label.trim().is_empty() => label.trim().to_string(),
            _ => {
                problems.push(Problem {
                    path,
                    message: "a job needs a `label`, which is what names its log".into(),
                });
                continue;
            }
        };

        let cron = match object.get("cron").and_then(|v| v.as_str()) {
            Some(cron) => cron.trim().to_string(),
            None => {
                problems.push(Problem {
                    path,
                    message: format!("`{label}` has no `cron` expression"),
                });
                continue;
            }
        };
        if let Err(message) = parse_expression(&cron) {
            problems.push(Problem { path, message });
            continue;
        }

        let exec = match object.get("exec").and_then(|v| v.as_array()) {
            Some(array) => {
                let argv: Vec<String> = array
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                if argv.len() != array.len() || argv.iter().any(|a| a.is_empty()) {
                    problems.push(Problem {
                        path,
                        message: format!(
                            "`{label}`'s `exec` must be an array of non-empty strings"
                        ),
                    });
                    continue;
                }
                argv
            }
            None => {
                problems.push(Problem {
                    path,
                    message: format!("`{label}` has no `exec` array"),
                });
                continue;
            }
        };
        if exec.is_empty() {
            problems.push(Problem {
                path,
                message: format!("`{label}`'s `exec` names no program"),
            });
            continue;
        }

        // Two jobs sharing a slug would share a log and a last run, and the
        // second one would silently overwrite the first's. Named here, where
        // the reader can still fix it, rather than discovered later as a log
        // that seems to hold two jobs' output.
        let id = slug(&label);
        if seen.contains(&id) {
            problems.push(Problem {
                path,
                message: format!(
                    "`{label}` is already the name of another job — two jobs cannot share the log `{id}.log`"
                ),
            });
            continue;
        }
        seen.push(id);

        jobs.push(Job {
            label,
            cron,
            exec,
            // Absent means on. A schedule somebody wrote is one they meant to
            // run, and requiring `"enabled": true` on every entry would make
            // the common case carry the rare one's field.
            enabled: object
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        });
    }

    (jobs, problems)
}

/// Check a whole schedule before any of it is written down.
///
/// [`parse`] names problems and drops the entry, which is right for a file
/// somebody else wrote. A schedule arriving from this app's own form is a
/// different situation: the reader is here, and silently dropping the job they
/// just typed would be the worst of both.
pub fn validate(jobs: &[Job]) -> Result<(), String> {
    let mut seen: Vec<String> = Vec::new();
    for job in jobs {
        if job.label.trim().is_empty() {
            return Err("a job needs a label, which is what names its log".into());
        }
        if job.exec.is_empty() || job.exec.iter().any(|a| a.is_empty()) {
            return Err(format!("`{}` needs a command to run", job.label));
        }
        parse_expression(&job.cron)?;

        let id = slug(&job.label);
        if seen.contains(&id) {
            return Err(format!(
                "two jobs are both called `{}`, and they would share the log `{id}.log`",
                job.label
            ));
        }
        seen.push(id);
    }
    Ok(())
}

// ------------------------------------------------------------- the sidecar

/// The engine-facing id of one project's cron sidecar.
pub fn container_id(project: &str) -> String {
    format!("{ID_PREFIX}{project}")
}

/// `cron-<project>` back into the project name.
pub fn parse_id(id: &str) -> Option<String> {
    id.strip_prefix(ID_PREFIX)
        .filter(|rest| !rest.is_empty())
        .map(str::to_string)
}

/// Does this project have anything to schedule?
///
/// A project with no enabled job gets no sidecar. An idle container that wakes
/// every minute to find an empty directory is a process in the list that means
/// nothing, and the list is where people look to find out what is running.
pub fn runnable(jobs: &[Job]) -> Vec<&Job> {
    jobs.iter().filter(|job| job.enabled).collect()
}

/// One sidecar's live state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerStatus {
    pub project: String,
    pub running: bool,
    /// How often Docker has had to bring it back, on the same terms as
    /// [`crate::worker::WorkerStatus`]: a large number is a crash loop.
    pub restarts: Option<i64>,
    pub container: String,
}

/// The `docker run` invocation for one project's cron sidecar.
///
/// `image` and the bind mount come from the project's own web container, so a
/// job sees exactly what the site sees. The schedule is mounted **read-only**:
/// it is generated from the manifest, and a job that could rewrite its own
/// schedule would be a job whose next run is not the one on screen.
pub fn run_args(
    project: &str,
    image: &str,
    host_root: &str,
    network: &str,
    schedule_host_dir: &str,
    state_host_dir: &str,
) -> Vec<String> {
    let root = crate::paths::to_docker_mount(host_root);
    let root = root.trim_end_matches('/');
    let mount = format!("{root}/projects/{project}:/var/www/html");
    let schedule = format!(
        "{}:{SCHEDULE_DIR}:ro",
        crate::paths::to_docker_mount(schedule_host_dir)
    );
    let state = format!(
        "{}:{STATE_DIR}",
        crate::paths::to_docker_mount(state_host_dir)
    );

    [
        "run",
        "-d",
        "--name",
        &format!("stackvo-{}", container_id(project)),
        "--network",
        network,
        "--restart",
        "unless-stopped",
        "-v",
        &mount,
        "-v",
        &schedule,
        "-v",
        &state,
        "-w",
        "/var/www/html",
        image,
        "sh",
        &format!("{SCHEDULE_DIR}/tick.sh"),
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// ------------------------------------------------------- the generated files

/// The files that make up one project's schedule directory.
///
/// Returned rather than written so the whole rendering is testable without a
/// filesystem, and so the caller can write the directory atomically — a tick
/// that lands halfway through an update would run a job against another job's
/// expression.
pub fn schedule_files(jobs: &[Job]) -> Vec<(String, Vec<u8>)> {
    let mut files = vec![("tick.sh".to_string(), tick_sh().into_bytes())];

    for job in runnable(jobs) {
        let id = job.id();
        files.push((
            format!("jobs/{id}.cron"),
            format!("{}\n", job.cron).into_bytes(),
        ));

        // NUL-separated, because that is the one byte a program name and an
        // argument cannot contain. A newline-separated file would be readable
        // and would also be wrong the first time somebody schedules a command
        // with a newline in an argument.
        let mut argv = Vec::new();
        for part in &job.exec {
            argv.extend_from_slice(part.as_bytes());
            argv.push(0);
        }
        files.push((format!("jobs/{id}.argv"), argv));
    }

    files
}

/// The tick loop, as POSIX `sh`.
///
/// ## Why `env` and not `sh -c`
///
/// The argv comes from a repository. `sh -c "$command"` would re-split it, and
/// every quoting rule in this codebase exists to stop exactly that. `xargs -0`
/// reads the NUL-separated file into an argument list without interpreting it,
/// and `env` execs its first argument as the program — so the bytes written
/// into `.argv` are the bytes the kernel receives, and no shell ever sees them
/// as text.
///
/// The cost is the exit status, and it is paid rather than hidden: `xargs`
/// reports 123 for *any* failing command, so a job that exited 3 and a job
/// whose program does not exist are indistinguishable by number. Measured, not
/// assumed — both come back 123. So [`LastRun`] carries whether the run
/// succeeded rather than a code that would not be the command's, and the log
/// carries the output, which is where the reason actually is.
///
/// ## Why the leading zeros are stripped
///
/// `date +%M` says `08`, and `$(( 08 % 5 ))` is a syntax error in POSIX
/// arithmetic — a leading zero means octal, and there is no digit 8 in octal.
/// So minutes 08 and 09, hours 08 and 09 and days 08 and 09 would each be an
/// error one day a month rather than a failure anybody could reproduce. They
/// are stripped before any arithmetic touches them.
///
/// ## Why the loop sleeps to the top of the minute
///
/// `sleep 60` drifts: each iteration costs the work plus sixty seconds, so a
/// schedule slides forward until a minute is skipped entirely. Sleeping the
/// remainder of the current minute keeps the wake-up aligned with the clock the
/// expression is written against.
pub fn tick_sh() -> String {
    format!(
        r##"#!/bin/sh
# Generated by StackVo. Edits are overwritten when the schedule changes.
#
# The argv of each job is read from a NUL-separated file and handed to `env`,
# which execs it. Nothing here ever passes a job's text to a shell.
#
# `-f` is not decoration. A cron field is very often the single character `*`,
# and an unquoted `*` in a `for` list is expanded by the shell into the names of
# the files in the working directory — so the minute field would be compared
# against `Cargo.lock` and the job would never be due. Globbing is turned back
# on for exactly one line, where a glob is what is wanted.
set -uf

# The mounts, and the one thing that may move them. Production never sets
# these — the sidecar's mounts are fixed by `run_args` — but a test that has to
# start a container to find out whether `run` works is a test that does not get
# written, and this is the whole run path rather than a piece of it.
SCHEDULE=${{STACKVO_CRON_SCHEDULE:-{SCHEDULE_DIR}}}
STATE=${{STACKVO_CRON_STATE:-{STATE_DIR}}}

# `08` is octal to POSIX arithmetic, and there is no digit 8 in octal.
strip0() {{
    v=${{1#0}}
    [ -n "$v" ] || v=0
    printf '%s' "$v"
}}

# Does one cron field match one value? The fields are validated before they are
# written here, so this reads them rather than defends against them.
field_matches() {{
    spec=$1
    value=$2
    lo=$3
    hi=$4

    saved_ifs=$IFS
    IFS=,
    for part in $spec; do
        IFS=$saved_ifs
        step=1
        case $part in
            */*)
                step=${{part#*/}}
                part=${{part%%/*}}
                ;;
        esac
        case $part in
            '*') from=$lo; to=$hi ;;
            *-*) from=${{part%%-*}}; to=${{part#*-}} ;;
            *)   from=$part; to=$part ;;
        esac
        if [ "$value" -ge "$from" ] && [ "$value" -le "$to" ] &&
           [ $(( (value - from) % step )) -eq 0 ]; then
            IFS=$saved_ifs
            return 0
        fi
        IFS=,
    done
    IFS=$saved_ifs
    return 1
}}

# Sunday is both 0 and 7 in a crontab, and an expression may name either.
day_matches() {{
    field_matches "$1" "$2" 0 7 && return 0
    [ "$2" -eq 0 ] && field_matches "$1" 7 0 7 && return 0
    return 1
}}

due() {{
    expression=$1
    # shellcheck disable=SC2086
    set -- $expression
    field_matches "$1" "$MINUTE" 0 59 || return 1
    field_matches "$2" "$HOUR" 0 23 || return 1
    field_matches "$3" "$DOM" 1 31 || return 1
    field_matches "$4" "$MONTH" 1 12 || return 1
    day_matches "$5" "$DOW" || return 1
    return 0
}}

# The status written here is `xargs`'s, not the command's. `xargs` collapses
# every failure into 123 — a command that exited 3 and a program that does not
# exist are the same number — so 0 means it ran and succeeded, and anything else
# means it did not. The reason lives in the log above the line, which is the
# command's own output.
run_job() {{
    id=$1
    [ -n "$id" ] && [ -f "$SCHEDULE/jobs/$id.argv" ] || {{
        echo "no such job: $id" >&2
        return 1
    }}
    started=$(date '+%Y-%m-%d %H:%M:%S')
    {{
        printf '\n=== %s ===\n' "$started"
        xargs -0 env < "$SCHEDULE/jobs/$id.argv" 2>&1
        status=$?
        printf '\n=== exit %s ===\n' "$status"
        printf '%s\t%s\n' "$started" "$status" > "$STATE/$id.last"
    }} >> "$STATE/$id.log" 2>&1
}}

# `tick.sh run <job>` runs one job now, by this same path, so a manual run
# writes the same log and the same last run a tick would. An argument rather
# than an environment variable because of what the two do when they are wrong:
# a mistyped argument falls through to `run_job` with nothing and fails, and a
# mistyped variable falls through to the loop below and hangs whatever was
# waiting for the command to return.
mkdir -p "$STATE"
if [ "${{1:-}}" = "run" ]; then
    run_job "${{2:-}}"
    exit
fi

# Sourced with this set, the script defines its functions and stops. The test
# suite wants that: it exercises the matcher against known times rather than
# against the passage of an hour.
[ "${{STACKVO_CRON_DEFINE_ONLY:-}}" = "1" ] && return 0 2>/dev/null

while :; do
    # Sleep the remainder of this minute rather than a flat 60, which drifts.
    sleep $(( 60 - $(date +%s) % 60 ))

    MINUTE=$(strip0 "$(date +%M)")
    HOUR=$(strip0 "$(date +%H)")
    DOM=$(strip0 "$(date +%d)")
    MONTH=$(strip0 "$(date +%m)")
    DOW=$(date +%w)

    set +f
    set -- "$SCHEDULE"/jobs/*.cron
    set -f

    for file in "$@"; do
        [ -f "$file" ] || continue
        id=$(basename "$file" .cron)
        expression=$(cat "$file")
        if due "$expression"; then
            run_job "$id" &
        fi
    done
done
"##
    )
}

// --------------------------------------------------------------- last runs

/// What the sidecar wrote down about one job's last run.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastRun {
    /// `YYYY-MM-DD HH:MM:SS`, in the container's clock, which is the machine's.
    pub at: String,
    /// Did it succeed? The only question the number can honestly answer.
    pub ok: bool,
    /// What the sidecar wrote down, kept for the reader who wants it.
    ///
    /// **Not the command's exit code.** `xargs` reports 123 for every failing
    /// command, so this is 0 or 123 in practice and a job that exited 3 says
    /// 123 here. [`tick_sh`] explains why that trade was taken.
    pub status: Option<i32>,
}

/// Read one job's last run, or `None` when it has not run yet.
///
/// Absence is a state and not an error: a job created a minute ago has no last
/// run, and the screen should say so rather than show a failure.
pub fn last_run(state_dir: &Path, id: &str) -> Option<LastRun> {
    let text = std::fs::read_to_string(state_dir.join(format!("{id}.last"))).ok()?;
    let line = text.lines().next()?;
    let (at, status) = line.split_once('\t')?;
    let status: Option<i32> = status.trim().parse().ok();
    Some(LastRun {
        at: at.to_string(),
        ok: status == Some(0),
        status,
    })
}

// ------------------------------------------------------------------- I/O

/// Where one project's generated schedule lives on this machine.
///
/// Beside the other generated things (`generated/certs`, `generated/traefik`)
/// rather than inside the project: this directory is derived from the manifest
/// and is rewritten whenever the manifest changes, and a derived file in a
/// repository is a file somebody will eventually commit and then edit.
pub fn schedule_dir(root: &Path, project: &str) -> std::path::PathBuf {
    root.join("generated").join("cron").join(project)
}

/// Where one project's job logs and last runs live.
///
/// Under `logs/`, keyed by project, on the same terms as
/// [`crate::instances`]'s service logs — the place people already look.
pub fn state_dir(root: &Path, project: &str) -> std::path::PathBuf {
    root.join("logs").join("cron").join(project)
}

/// Write the schedule directory, so that a tick never reads a half-written one.
///
/// Rendered into a sibling and moved into place. A tick landing mid-update
/// would otherwise run one job's argv against another job's expression, which
/// is a failure nobody could reproduce because the window is one rename wide.
///
/// The old directory is removed rather than merged: a job that was deleted has
/// to stop firing, and merging would leave its `.cron` file behind to fire
/// forever.
pub fn write_schedule(root: &Path, project: &str, jobs: &[Job]) -> crate::error::Result<()> {
    let target = schedule_dir(root, project);
    let staging = target.with_extension("writing");

    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(staging.join("jobs"))
        .map_err(|e| crate::error::Error::io(format!("creating {}", staging.display()), e))?;

    for (name, contents) in schedule_files(jobs) {
        let path = staging.join(&name);
        std::fs::write(&path, contents)
            .map_err(|e| crate::error::Error::io(format!("writing {}", path.display()), e))?;
    }

    let _ = std::fs::remove_dir_all(&target);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| crate::error::Error::io(format!("creating {}", parent.display()), e))?;
    }
    std::fs::rename(&staging, &target)
        .map_err(|e| crate::error::Error::io(format!("moving {}", target.display()), e))?;

    // The sidecar writes here, and it starts with the directory already
    // present: a bind mount of a path that does not exist is created by the
    // engine as a root-owned directory, and the container is not root.
    let state = state_dir(root, project);
    std::fs::create_dir_all(&state)
        .map_err(|e| crate::error::Error::io(format!("creating {}", state.display()), e))?;

    Ok(())
}

/// One job, as the screen needs it: what it is, and what happened last time.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    pub id: String,
    pub label: String,
    pub cron: String,
    pub exec: Vec<String>,
    /// The argv as one line, for the row. Never parsed back.
    pub command: String,
    pub enabled: bool,
    /// `None` until it has run once, which is a state and not a failure.
    pub last_run: Option<LastRun>,
}

/// Every job in one project, with its last run read from disk.
pub fn statuses(root: &Path, project: &str, jobs: &[Job]) -> Vec<JobStatus> {
    let state = state_dir(root, project);
    jobs.iter()
        .map(|job| {
            let id = job.id();
            JobStatus {
                last_run: last_run(&state, &id),
                id,
                label: job.label.clone(),
                cron: job.cron.clone(),
                exec: job.exec.clone(),
                command: job.display(),
                enabled: job.enabled,
            }
        })
        .collect()
}

/// The tail of one job's log.
///
/// Empty rather than an error when the job has never run: the log is created
/// by the first run, and "nothing yet" is what the screen should say.
pub fn log_tail(root: &Path, project: &str, id: &str, lines: usize) -> String {
    let path = state_dir(root, project).join(format!("{id}.log"));
    let Ok(text) = std::fs::read_to_string(&path) else {
        return String::new();
    };
    let all: Vec<&str> = text.lines().collect();
    let start = all.len().saturating_sub(lines);
    all[start..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_becomes_a_findable_filename() {
        assert_eq!(slug("Cache cleanup"), "cache-cleanup");
        assert_eq!(slug("  Nightly  Prune!  "), "nightly-prune");
        assert_eq!(
            slug("ÖNBELLEK"),
            "nbellek",
            "non-ASCII collapses rather than escapes"
        );
        assert_eq!(
            slug("!!!"),
            "job",
            "a name with nothing in it still names a file"
        );
    }

    /// The grammar the tick script can actually match, and nothing wider.
    #[test]
    fn the_expressions_accepted_are_the_ones_the_matcher_implements() {
        for good in [
            "* * * * *",
            "*/5 * * * *",
            "0 3 * * *",
            "0 0 1 * *",
            "30 2 * * 1",
            "0,15,30,45 * * * *",
            "0 9-17 * * 1-5",
            "0 0-23/2 * * *",
            "0 0 * * 7",
        ] {
            assert!(
                parse_expression(good).is_ok(),
                "{good} should be accepted: {:?}",
                parse_expression(good)
            );
        }

        for (bad, because) in [
            ("* * * *", "four fields is not five"),
            ("* * * * * *", "six fields is not five"),
            ("60 * * * *", "there is no minute 60"),
            ("* 24 * * *", "there is no hour 24"),
            ("* * 0 * *", "there is no day 0 of a month"),
            ("* * * 13 *", "there is no month 13"),
            ("* * * * 8", "there is no day 8 of a week"),
            ("*/0 * * * *", "a step of 0 never comes round"),
            ("MON * * * *", "names are not the accepted spelling"),
            ("@daily", "macros are not the accepted spelling"),
            (
                "17-5 * * * *",
                "a range that runs backwards matches nothing",
            ),
            ("5/10 * * * *", "a bare number with a step is ambiguous"),
        ] {
            assert!(
                parse_expression(bad).is_err(),
                "{bad} should be refused because {because}"
            );
        }
    }

    /// A malformed job is a warning and the rest of the file still runs.
    #[test]
    fn one_unreadable_job_does_not_take_the_others_with_it() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
              "schedule": [
                { "label": "Good", "cron": "*/5 * * * *", "exec": ["php", "artisan", "x"] },
                { "label": "No cron", "exec": ["php"] },
                { "label": "Bad cron", "cron": "* * *", "exec": ["php"] },
                { "label": "No exec", "cron": "* * * * *" },
                { "label": "Empty exec", "cron": "* * * * *", "exec": [] },
                { "label": "Not strings", "cron": "* * * * *", "exec": ["php", 7] },
                { "cron": "* * * * *", "exec": ["php"] },
                { "label": "Also good", "cron": "0 3 * * *", "exec": ["sh", "bin/nightly"] }
              ]
            }"#,
        )
        .unwrap();

        let (jobs, problems) = parse(&json);
        assert_eq!(
            jobs.iter().map(|j| j.label.as_str()).collect::<Vec<_>>(),
            ["Good", "Also good"]
        );
        assert_eq!(problems.len(), 6, "every refusal is named: {problems:?}");
        assert!(problems.iter().all(|p| p.path.starts_with("schedule[")));
    }

    /// Two jobs with one log between them is a log that lies about both.
    #[test]
    fn two_jobs_cannot_share_a_log() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
              "schedule": [
                { "label": "Cache cleanup", "cron": "* * * * *", "exec": ["a"] },
                { "label": "cache  CLEANUP", "cron": "* * * * *", "exec": ["b"] }
              ]
            }"#,
        )
        .unwrap();

        let (jobs, problems) = parse(&json);
        assert_eq!(jobs.len(), 1);
        assert_eq!(problems.len(), 1);
        assert!(
            problems[0].message.contains("cache-cleanup.log"),
            "the message names the collision: {}",
            problems[0].message
        );
    }

    /// Absent means on, and a paused job keeps its command.
    #[test]
    fn a_paused_job_stays_in_the_file_and_out_of_the_schedule() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
              "schedule": [
                { "label": "On", "cron": "* * * * *", "exec": ["a"] },
                { "label": "Off", "cron": "* * * * *", "exec": ["b"], "enabled": false }
              ]
            }"#,
        )
        .unwrap();

        let (jobs, _) = parse(&json);
        assert_eq!(jobs.len(), 2, "both are kept");
        assert_eq!(
            runnable(&jobs)
                .iter()
                .map(|j| j.label.as_str())
                .collect::<Vec<_>>(),
            ["On"],
            "only one is scheduled"
        );

        let files = schedule_files(&jobs);
        let names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"jobs/on.cron"));
        assert!(
            !names.contains(&"jobs/off.cron"),
            "a paused job writes no file"
        );
    }

    /// The argv reaches the container as bytes, not as a line to re-split.
    #[test]
    fn an_argument_with_a_space_in_it_stays_one_argument() {
        let jobs = vec![Job {
            label: "Notify".into(),
            cron: "* * * * *".into(),
            exec: vec!["php".into(), "artisan".into(), "say:hello world".into()],
            enabled: true,
        }];

        let files = schedule_files(&jobs);
        let argv = files
            .iter()
            .find(|(name, _)| name == "jobs/notify.argv")
            .map(|(_, bytes)| bytes.clone())
            .expect("the argv file is written");

        assert_eq!(
            argv,
            b"php\0artisan\0say:hello world\0".to_vec(),
            "one NUL per argument, and none inside one"
        );
        assert_eq!(argv.iter().filter(|b| **b == 0).count(), 3);
    }

    /// The matcher in the container has to agree with the validator in Rust,
    /// and only one of them is written in a language with a test runner.
    ///
    /// So the script itself is run. `due` is exercised against a table of
    /// times rather than against the passage of an hour: the loop is skipped
    /// by `STACKVO_CRON_DEFINE_ONLY`, which leaves the functions defined and the
    /// clock injectable.
    ///
    /// Unix only. The script runs inside a Linux container in production; on
    /// Windows there is no `sh` to run it with, and a test that silently did
    /// nothing there would be worse than one that is honestly absent.
    #[cfg(unix)]
    #[test]
    fn the_tick_script_agrees_with_the_expressions_rust_accepts() {
        let dir = temp("tick");
        let script = dir.join("tick.sh");
        std::fs::write(&script, tick_sh()).expect("the script is written");

        // (expression, minute, hour, day, month, weekday, due?)
        let cases: [(&str, u32, u32, u32, u32, u32, bool); 16] = [
            ("* * * * *", 0, 0, 1, 1, 0, true),
            ("*/5 * * * *", 5, 3, 1, 1, 1, true),
            ("*/5 * * * *", 6, 3, 1, 1, 1, false),
            // The octal trap: `08` and `09` are not numbers in POSIX
            // arithmetic, and a matcher that forgets it fails twice an hour.
            ("*/4 * * * *", 8, 8, 8, 8, 1, true),
            ("*/3 * * * *", 9, 9, 9, 9, 1, true),
            ("0 3 * * *", 0, 3, 1, 1, 1, true),
            ("0 3 * * *", 0, 4, 1, 1, 1, false),
            ("0,15,30,45 * * * *", 30, 12, 1, 1, 1, true),
            ("0,15,30,45 * * * *", 31, 12, 1, 1, 1, false),
            ("0 9-17 * * 1-5", 0, 9, 1, 1, 5, true),
            ("0 9-17 * * 1-5", 0, 18, 1, 1, 5, false),
            ("0 9-17 * * 1-5", 0, 9, 1, 1, 6, false),
            ("0 0-23/2 * * *", 0, 22, 1, 1, 1, true),
            ("0 0-23/2 * * *", 0, 21, 1, 1, 1, false),
            // Sunday is 0 and 7, and an expression may name either.
            ("0 0 * * 7", 0, 0, 1, 1, 0, true),
            ("0 0 * * 0", 0, 0, 1, 1, 0, true),
        ];

        let mut driver = String::from("STACKVO_CRON_DEFINE_ONLY=1 . \"$1\"\n");
        for (expression, minute, hour, dom, month, dow, _) in cases {
            driver.push_str(&format!(
                "MINUTE={minute}; HOUR={hour}; DOM={dom}; MONTH={month}; DOW={dow}\n\
                 if due '{expression}'; then echo due; else echo idle; fi\n"
            ));
        }
        let driver_path = dir.join("driver.sh");
        std::fs::write(&driver_path, driver).expect("the driver is written");

        let output = std::process::Command::new("sh")
            .arg(&driver_path)
            .arg(&script)
            .output()
            .expect("sh runs");
        assert!(
            output.status.success(),
            "the script failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let got: Vec<&str> = std::str::from_utf8(&output.stdout)
            .expect("utf-8")
            .lines()
            .collect();
        assert_eq!(got.len(), cases.len(), "one answer per case: {got:?}");

        for (answer, (expression, minute, hour, dom, month, dow, want)) in got.iter().zip(cases) {
            let due = *answer == "due";
            assert_eq!(
                due,
                want,
                "`{expression}` at {minute:02}:{hour:02} on {dom:02}/{month:02} (weekday {dow}) \
                 should be {}",
                if want { "due" } else { "idle" }
            );
        }

        // Everything the table asserts on is an expression Rust would have
        // written to disk. A case the validator refuses is a case the matcher
        // never sees, and holding the two together is the point.
        for (expression, ..) in cases {
            assert!(
                parse_expression(expression).is_ok(),
                "{expression} is not an expression this app would ever write"
            );
        }
    }

    /// A deleted job has to stop firing, and merging a directory would leave
    /// its `.cron` file behind to fire forever.
    #[test]
    fn rewriting_a_schedule_removes_the_jobs_that_are_gone() {
        let root = temp("write-schedule");
        let job = |label: &str| Job {
            label: label.into(),
            cron: "* * * * *".into(),
            exec: vec!["php".into(), "artisan".into(), label.into()],
            enabled: true,
        };

        write_schedule(&root, "shop", &[job("first"), job("second")]).expect("written");
        let dir = schedule_dir(&root, "shop");
        assert!(dir.join("tick.sh").is_file());
        assert!(dir.join("jobs/first.cron").is_file());
        assert!(dir.join("jobs/second.cron").is_file());

        write_schedule(&root, "shop", &[job("second")]).expect("rewritten");
        assert!(
            !dir.join("jobs/first.cron").exists(),
            "the deleted job would otherwise keep firing"
        );
        assert!(dir.join("jobs/second.cron").is_file());
        assert!(
            !dir.with_extension("writing").exists(),
            "the staging directory does not survive the move"
        );

        // The sidecar's writable mount exists before the sidecar does: the
        // engine would otherwise create it root-owned, and the container is not.
        assert!(state_dir(&root, "shop").is_dir());
    }

    /// "Run now" is the whole run path — argv, log, last run — and it is the
    /// one a reader presses when a job did not fire and they want to know why.
    ///
    /// A mode rather than an environment variable, and this is what that buys:
    /// a name the script does not recognise fails and returns, where an
    /// unrecognised variable would have fallen through to the tick loop and
    /// hung whatever was waiting for the command.
    #[cfg(unix)]
    #[test]
    fn running_a_job_by_hand_writes_the_log_and_the_last_run() {
        let root = temp("run-now");
        let jobs = vec![Job {
            label: "Greet".into(),
            cron: "0 3 * * *".into(),
            exec: vec!["printf".into(), "hello from %s\n".into(), "stackvo".into()],
            enabled: true,
        }];
        write_schedule(&root, "shop", &jobs).expect("written");

        let run = |job: &str| {
            std::process::Command::new("sh")
                .arg(schedule_dir(&root, "shop").join("tick.sh"))
                .arg("run")
                .arg(job)
                .env("STACKVO_CRON_SCHEDULE", schedule_dir(&root, "shop"))
                .env("STACKVO_CRON_STATE", state_dir(&root, "shop"))
                .output()
                .expect("sh runs")
        };

        let ok = run("greet");
        assert!(
            ok.status.success(),
            "{}",
            String::from_utf8_lossy(&ok.stderr)
        );

        let state = state_dir(&root, "shop");
        let log = std::fs::read_to_string(state.join("greet.log")).expect("a log was written");
        assert!(
            log.contains("hello from stackvo"),
            "the output is in the log: {log}"
        );
        assert!(
            log.contains("=== exit 0 ==="),
            "and so is the outcome: {log}"
        );

        let last = last_run(&state, "greet").expect("a last run was written");
        assert!(last.ok);
        assert_eq!(last.at.len(), "2026-08-24 03:00:00".len());

        // A job that is not there fails and returns. It must not hang, and it
        // must not invent a log for a job nobody scheduled.
        let missing = run("nothing-like-this");
        assert!(!missing.status.success());
        assert!(!state.join("nothing-like-this.log").exists());
    }

    #[test]
    fn a_container_id_survives_a_project_name_with_a_dash() {
        assert_eq!(container_id("my-shop"), "cron-my-shop");
        assert_eq!(parse_id("cron-my-shop").as_deref(), Some("my-shop"));
        assert_eq!(parse_id("worker-my-shop-queue"), None);
        assert_eq!(parse_id("cron-"), None);
    }

    /// The schedule is mounted read-only: a job that could rewrite its own
    /// schedule would be one whose next run is not the one on screen.
    #[test]
    fn the_sidecar_mounts_the_schedule_read_only_and_runs_the_tick_script() {
        let args = run_args(
            "shop",
            "stackvo/shop:latest",
            "/Users/dev/.stackvo",
            "stackvo-net",
            "/Users/dev/.stackvo/generated/cron/shop",
            "/Users/dev/.stackvo/logs/cron/shop",
        );
        let line = args.join(" ");

        assert!(line.contains("--name stackvo-cron-shop"));
        assert!(line.contains("--restart unless-stopped"));
        assert!(
            line.contains(&format!("generated/cron/shop:{SCHEDULE_DIR}:ro")),
            "the schedule is read-only: {line}"
        );
        assert!(
            line.contains(&format!("logs/cron/shop:{STATE_DIR}")),
            "the state directory is writable: {line}"
        );
        assert!(line.ends_with(&format!("sh {SCHEDULE_DIR}/tick.sh")));
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-cron-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_job_that_has_not_run_has_no_last_run_rather_than_a_failure() {
        let dir = temp("last-run");
        assert!(last_run(&dir, "cache-cleanup").is_none());

        std::fs::write(dir.join("cache-cleanup.last"), "2026-08-24 03:00:00\t0\n").unwrap();
        let run = last_run(&dir, "cache-cleanup").expect("the file is read");
        assert_eq!(run.at, "2026-08-24 03:00:00");
        assert!(run.ok);
        assert_eq!(run.status, Some(0));

        // 123 is what `xargs` reports for every failing command, which is the
        // only failure this file can ever hold.
        std::fs::write(dir.join("nightly.last"), "2026-08-24 03:00:00\t123\n").unwrap();
        let failed = last_run(&dir, "nightly").expect("the file is read");
        assert!(!failed.ok);
        assert_eq!(failed.status, Some(123));
    }
}
