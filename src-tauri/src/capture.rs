//! The session a request ran under — recorded only while somebody says so.
//!
//! ## The half `explain.rs` refused, and why refusing was right
//!
//! Replaying a recording is already possible for a `GET`, and a `POST` is
//! refused **by name** with the reason: *"only the request line was recorded —
//! not its body, its headers or its session — and a POST replayed without them
//! is a different request, which is usually answered with a redirect or a 419
//! rather than the page"*. That refusal is not a gap in the code. It is what
//! the code knew.
//!
//! K-4's own example sentence is the one that needs the other half: *"this bug
//! only happened in that basket."* A basket is a session. Replaying it means
//! recording the request's **cookies and body** — which means writing somebody's
//! session token and their form input to disk.
//!
//! ## So the feature is the asking, not the storing
//!
//! Every other capability in this application that reaches something the user
//! would want to know about asks first: [`crate::hooks`] against a digest,
//! [`crate::provider`] per direction, [`crate::grant`] with a clock on it. This
//! is the same class and the sharpest instance of it, because what is stored
//! **is the secret itself** — redaction is not available here, since the value
//! is the whole point of keeping it.
//!
//! Four rules fell out of that, and each is a refusal rather than a default:
//!
//! **Off, always, until armed.** There is no setting that leaves it on. The
//! bridge writes nothing without a flag file that this module creates.
//!
//! **Armed for minutes, never indefinitely.** [`MAX_MINUTES`] is an hour, and
//! the window is stored as an absolute expiry rather than a duration — the
//! lesson `stats_store.rs` wrote down about a series reloaded verbatim, applied
//! to a permission: a window that survives a restart by starting its clock
//! again is a window that never closes. An expired one is swept on the next
//! read, so nothing has to remember to.
//!
//! **Disarming deletes.** Turning it off does not merely stop new captures; it
//! removes the ones already taken. A permission that ends leaving its harvest
//! behind is a permission the person believes ended.
//!
//! **The value never leaves this machine and never reaches a report.** It goes
//! into the replay request and nowhere else — not into the audit trail's
//! detail, not into a diagnostics bundle, not onto a screen. `audit` records
//! *that* a window was opened, which is the fact somebody has to be able to
//! date; `leaks.rs` already wrote the rule about the other half.
//!
//! ## Joined by the request line and the clock
//!
//! A recording is `spx`'s and a session is the bridge's, and the two never meet
//! in a file. They are matched the way [`crate::explain`] already matches a
//! query to a request: the same request line, and the nearest moment inside
//! [`JOIN_SECONDS`]. Wider would attach one visitor's basket to another's
//! recording of the same page, which is the one wrong answer this must not
//! give — so the window is narrow and an unmatched recording stays a `GET`-only
//! replay, exactly as it is today.

use crate::error::{Code, Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The longest a capture window may be armed for.
///
/// An hour, and the number is a judgement rather than a constant somebody
/// picked: it is long enough to reproduce a bug through a checkout and short
/// enough that walking away from the machine ends it.
pub const MAX_MINUTES: u32 = 60;

/// How near in time a captured session must be to a recording to be its own.
///
/// Two seconds. A request and the profiler's report of it are written by the
/// same execution, so the true gap is milliseconds; the margin absorbs a slow
/// shutdown handler and nothing else. Wider would let one visitor's session
/// attach to another visitor's recording of the same URL.
pub const JOIN_SECONDS: f64 = 2.0;

/// The file the bridge writes a captured session into.
///
/// Pointed at rather than restated: [`crate::debugbridge`] is what writes it,
/// so it is what names it, and a second literal here is how a reader and a
/// writer come to disagree about a filename inside a container.
pub use crate::debugbridge::{MAX_BODY, SESSIONS_FILE};

// ------------------------------------------------------------- the window

/// Permission to record, with the moment it ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub project: String,
    /// RFC 3339, UTC, fixed width — comparable as text against
    /// [`crate::snapshot::now_rfc3339`], which is how `worktree` expiry already
    /// works and why no date arithmetic happens at read time.
    pub expires: String,
}

impl Window {
    /// Whole minutes left, floored, and never negative.
    ///
    /// Floored rather than rounded: a window with fifty seconds left reading
    /// "1 minute" is a promise the next fifty seconds will not keep.
    pub fn remaining_minutes(&self, now: &str) -> u32 {
        let (Some(end), Some(now)) = (
            crate::audit::seconds_of_rfc3339(&self.expires),
            crate::audit::seconds_of_rfc3339(now),
        ) else {
            return 0;
        };
        ((end - now).max(0) / 60) as u32
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Store {
    #[serde(default)]
    windows: Vec<Window>,
}

/// Where the windows live: the app's own directory, never the project's.
///
/// Never the project's, and that is not a detail. A file under a repository is
/// a file somebody commits, and this one names which projects are currently
/// recording their own sessions.
pub fn path(root: &Path) -> PathBuf {
    root.join("generated").join("debug").join("capture.json")
}

/// The flag the bridge looks for, beside the one that turns the bridge on.
///
/// In the **conf** directory, which is a directory mount — so creating and
/// removing it is seen by a running container immediately, with no recreate.
/// `debugbridge`'s module comment records why that matters and why the ini
/// beside it can never be edited the same way.
pub fn flag_path(root: &Path, project: &str) -> PathBuf {
    crate::debugbridge::conf_dir(root, project).join("capture.flag")
}

/// Where the bridge appends what it captured.
pub fn sessions_path(root: &Path, project: &str) -> PathBuf {
    crate::debugbridge::events_dir(root, project).join(SESSIONS_FILE)
}

fn read_store(root: &Path) -> Store {
    std::fs::read_to_string(path(root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_store(root: &Path, store: &Store) -> Result<()> {
    let file = path(root);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }
    let text = serde_json::to_string_pretty(store)
        .map_err(|e| Error::new(Code::IoError, format!("serialising the windows: {e}")))?;
    std::fs::write(&file, text + "\n")
        .map_err(|e| Error::io(format!("writing {}", file.display()), e))
}

/// Which windows are still open, given the moment.
///
/// Pure, so every branch has a test without a clock. An expired window is not
/// returned **and** is not kept — see [`armed`], which is what actually sweeps.
pub fn open(windows: &[Window], now: &str) -> Vec<Window> {
    windows
        .iter()
        .filter(|w| w.expires.as_str() > now)
        .cloned()
        .collect()
}

/// Is this project recording, and until when?
///
/// Sweeps on the way through: an expired window is removed from disk, its flag
/// is taken away and its captures are deleted. Doing it here rather than on a
/// timer is the same choice `worktree` expiry made — a clock that only runs
/// while the app is open would leave a window open across a night the app spent
/// closed, and the sweep has to happen at the moment somebody asks anyway.
pub fn armed(root: &Path, project: &str) -> Option<Window> {
    let store = read_store(root);
    let now = crate::snapshot::now_rfc3339();
    let live = open(&store.windows, &now);

    if live.len() != store.windows.len() {
        for stale in store.windows.iter().filter(|w| !live.contains(w)) {
            let _ = std::fs::remove_file(flag_path(root, &stale.project));
            let _ = std::fs::remove_file(sessions_path(root, &stale.project));
        }
        let _ = write_store(
            root,
            &Store {
                windows: live.clone(),
            },
        );
    }

    live.into_iter().find(|w| w.project == project)
}

/// Is the bridge that would do the capturing switched on for this project?
///
/// Two flags and not one, deliberately — see [`flag_path`]. This is the outer
/// one: `debugbridge`'s prepend file returns immediately unless it exists, so
/// with the bridge off nothing in it runs, including the half that writes a
/// session.
pub fn bridge_on(root: &Path, project: &str) -> bool {
    crate::debugbridge::sentinel_path(root, project).is_file()
}

/// Open a window, for a stated number of minutes.
///
/// Re-arming replaces rather than extends: a window is a decision with a length
/// somebody just chose, and quietly adding to one they opened an hour ago would
/// be a different decision than the one they made.
///
/// ## Refused while the bridge is off, rather than granted and inert
///
/// The two flags are separate on purpose — *"show me my dumps"* and *"record my
/// session token"* are two permissions — but they are not independent: the
/// inner one is read by a file the outer one is the gate for. Arming without
/// the bridge would write `capture.flag` into a directory nothing is reading,
/// take an audit entry saying credentials are now being written, and capture
/// nothing at all.
///
/// That failure is the one this application refuses everywhere else it appears:
/// `channel.rs` says it in a sentence — *"a channel nobody publishes is a
/// setting that silently stops updates"* — and it would be worse here, because
/// what the person concludes from an empty window is that their `POST` simply
/// cannot be replayed. So it is an error with the reason, and the permission is
/// never taken.
pub fn arm(root: &Path, project: &str, minutes: u32) -> Result<Window> {
    if minutes == 0 || minutes > MAX_MINUTES {
        return Err(Error::new(
            Code::InvalidInput,
            format!("a capture window is 1 to {MAX_MINUTES} minutes"),
        ));
    }

    if !bridge_on(root, project) {
        return Err(Error::new(
            Code::Conflict,
            format!(
                "{project}'s debug bridge is off, and it is what would do the capturing — \
                 arming a window now would grant the permission and record nothing"
            ),
        )
        .with_hint(crate::hints::CAPTURE_NEEDS_THE_BRIDGE));
    }

    let flag = flag_path(root, project);
    if let Some(parent) = flag.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }

    let now = crate::snapshot::now_rfc3339();
    let seconds = crate::audit::seconds_of_rfc3339(&now)
        .ok_or_else(|| Error::new(Code::IoError, "the clock would not read".to_string()))?;
    let window = Window {
        project: project.to_string(),
        expires: crate::audit::rfc3339_of(seconds + (minutes as i64) * 60),
    };

    let mut store = read_store(root);
    store.windows.retain(|w| w.project != project);
    store.windows.push(window.clone());
    write_store(root, &store)?;

    // The flag last, so a store that would not write leaves the bridge
    // capturing nothing — the failure direction that records less rather than
    // more.
    std::fs::write(&flag, "").map_err(|e| Error::io(format!("writing {}", flag.display()), e))?;

    Ok(window)
}

/// What disarming removed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cleared {
    /// Whether a window was open at all — so the caller can tell "I closed it"
    /// from "it had already expired".
    pub was_armed: bool,
    /// How many captured sessions were deleted.
    pub deleted: usize,
}

/// Close it, and delete what it collected.
///
/// The deletion is the point and not a courtesy. A permission that ends leaving
/// its harvest on disk is a permission the person believes ended, and the thing
/// left behind is a session token.
pub fn disarm(root: &Path, project: &str) -> Result<Cleared> {
    let was_armed = armed(root, project).is_some();

    let mut store = read_store(root);
    store.windows.retain(|w| w.project != project);
    write_store(root, &store)?;

    // The flag first: stop new writes before removing what is there, so a
    // request in flight cannot append to a file this is about to delete and
    // leave one line behind.
    let _ = std::fs::remove_file(flag_path(root, project));
    let deleted = clear(root, project);

    Ok(Cleared { was_armed, deleted })
}

/// Delete every captured session for one project, and say how many.
pub fn clear(root: &Path, project: &str) -> usize {
    let file = sessions_path(root, project);
    let count = read(root, project).len();
    let _ = std::fs::remove_file(file);
    count
}

// ------------------------------------------------------------ the sessions

/// One request, as it was actually made.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Seconds since the epoch with fractions — the bridge's `microtime`, the
    /// same clock `debugbridge::Event::at` uses and the same one a recording is
    /// matched against.
    pub at: f64,
    /// `POST /checkout`, in `spx::Report::request`'s spelling so the two can be
    /// compared without either side normalising the other's.
    pub request: String,
    pub method: String,
    /// The `Cookie` header, verbatim. **This is the session token**; see the
    /// module comment on why it is not redacted and what is done instead.
    #[serde(default)]
    pub cookie: Option<String>,
    /// The raw request body, bounded by [`MAX_BODY`].
    #[serde(default)]
    pub body: Option<String>,
    /// The `Content-Type`, without which a body is bytes nobody can send.
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Read what the bridge captured, newest last.
///
/// A line that will not parse is dropped rather than failing the read, on
/// `debugbridge::read_events`' reasoning: this file is written by a container
/// that may be running an older bridge than the app reading it.
pub fn read(root: &Path, project: &str) -> Vec<Session> {
    let Ok(text) = std::fs::read_to_string(sessions_path(root, project)) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str::<Session>(line).ok())
        .collect()
}

/// The session that belongs to one recording, if there is one.
///
/// Pure, and the whole join. Matched on the request line **and** the clock,
/// because either alone is wrong: the line alone would attach one visitor's
/// basket to another visitor's recording of the same page, and the clock alone
/// would attach whatever happened to be next.
///
/// Nearest wins inside the window, so two requests to the same URL a second
/// apart do not swap.
pub fn matching<'a>(
    sessions: &'a [Session],
    request: &str,
    recorded_at: i64,
) -> Option<&'a Session> {
    sessions
        .iter()
        .filter(|s| s.request == request)
        .map(|s| (s, (s.at - recorded_at as f64).abs()))
        .filter(|(_, gap)| *gap <= JOIN_SECONDS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(s, _)| s)
}

/// What a caller may say about a session without saying any of it.
///
/// The shape every screen and every report gets. A cookie is reported as a
/// **count of names**, never the names and never the values: which cookies a
/// site sets is a fact about the site, and the value is the credential itself.
/// A body is reported as a size and a type.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Described {
    pub method: String,
    pub cookies: usize,
    pub body_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

impl Session {
    pub fn describe(&self) -> Described {
        Described {
            method: self.method.clone(),
            cookies: self
                .cookie
                .as_deref()
                .map(|c| c.split(';').filter(|p| !p.trim().is_empty()).count())
                .unwrap_or(0),
            body_bytes: self.body.as_deref().map(str::len).unwrap_or(0),
            content_type: self.content_type.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("stackvo-capture-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("creating the scratch directory");
        dir
    }

    /// Switch the bridge on, as `debugbridge::set_enabled` would.
    ///
    /// Every test below that arms a window needs this first, which is the
    /// dependency being tested in
    /// [`arming_is_refused_while_the_bridge_that_would_capture_is_off`].
    fn bridge(root: &Path, project: &str) {
        let flag = crate::debugbridge::sentinel_path(root, project);
        std::fs::create_dir_all(flag.parent().unwrap()).unwrap();
        std::fs::write(&flag, "").unwrap();
    }

    fn window(project: &str, expires: &str) -> Window {
        Window {
            project: project.into(),
            expires: expires.into(),
        }
    }

    fn session(at: f64, request: &str) -> Session {
        Session {
            at,
            request: request.into(),
            method: request.split(' ').next().unwrap_or("GET").into(),
            cookie: Some("session=abc".into()),
            body: Some("qty=2".into()),
            content_type: Some("application/x-www-form-urlencoded".into()),
        }
    }

    /// Fixed-width UTC compares as text, which is why nothing here does date
    /// arithmetic at read time.
    #[test]
    fn a_window_is_open_until_the_moment_it_names() {
        let windows = [
            window("shop", "2026-08-30T10:00:00Z"),
            window("blog", "2026-08-30T12:00:00Z"),
        ];

        let live = open(&windows, "2026-08-30T11:00:00Z");
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].project, "blog");

        // The boundary is exclusive: a window expiring exactly now is over.
        assert!(open(&windows, "2026-08-30T12:00:00Z").is_empty());
        assert_eq!(open(&windows, "2026-08-30T09:00:00Z").len(), 2);
    }

    /// Floored, and never negative.
    ///
    /// A window with fifty seconds left reading "1 minute" is a promise the
    /// next fifty seconds will not keep.
    #[test]
    fn the_time_left_is_floored_so_the_number_is_never_a_promise() {
        let w = window("shop", "2026-08-30T10:30:00Z");
        assert_eq!(w.remaining_minutes("2026-08-30T10:00:00Z"), 30);
        assert_eq!(w.remaining_minutes("2026-08-30T10:29:10Z"), 0);
        assert_eq!(w.remaining_minutes("2026-08-30T11:00:00Z"), 0);
    }

    /// The whole permission, end to end: nothing until armed, and disarming
    /// deletes.
    ///
    /// The deletion is the point rather than a courtesy — a permission that
    /// ends leaving its harvest on disk is a permission the person believes
    /// ended, and the thing left behind is a session token.
    #[test]
    fn disarming_takes_the_flag_away_and_deletes_what_was_captured() {
        let root = scratch("disarm");
        bridge(&root, "shop");
        assert!(armed(&root, "shop").is_none(), "off until asked for");

        let window = arm(&root, "shop", 30).unwrap();
        assert_eq!(window.project, "shop");
        assert!(flag_path(&root, "shop").is_file(), "the bridge's flag");
        assert!(armed(&root, "shop").is_some());

        // As the bridge would have written it.
        std::fs::create_dir_all(sessions_path(&root, "shop").parent().unwrap()).unwrap();
        std::fs::write(
            sessions_path(&root, "shop"),
            format!(
                "{}\n{}\n",
                serde_json::to_string(&session(1.0, "POST /checkout")).unwrap(),
                serde_json::to_string(&session(2.0, "POST /pay")).unwrap()
            ),
        )
        .unwrap();
        assert_eq!(read(&root, "shop").len(), 2);

        let cleared = disarm(&root, "shop").unwrap();
        assert!(cleared.was_armed);
        assert_eq!(cleared.deleted, 2);
        assert!(!flag_path(&root, "shop").exists());
        assert!(read(&root, "shop").is_empty());
        assert!(armed(&root, "shop").is_none());

        // And disarming again is not an error — it says it found nothing,
        // which is a different fact from having closed one.
        let again = disarm(&root, "shop").unwrap();
        assert!(!again.was_armed);
        assert_eq!(again.deleted, 0);
    }

    /// A window is a length somebody just chose, not one to add to.
    #[test]
    fn re_arming_replaces_rather_than_extends() {
        let root = scratch("rearm");
        bridge(&root, "shop");
        let first = arm(&root, "shop", 60).unwrap();
        let second = arm(&root, "shop", 1).unwrap();

        assert!(second.expires < first.expires, "shorter, not longer");
        assert_eq!(armed(&root, "shop").unwrap().expires, second.expires);

        // And a length outside the bounds is refused rather than clamped: a
        // clamp would grant a window nobody asked for.
        assert!(arm(&root, "shop", 0).is_err());
        assert!(arm(&root, "shop", MAX_MINUTES + 1).is_err());
    }

    /// The expiry sweeps itself, and takes the flag and the captures with it.
    ///
    /// A clock that only ran while the app was open would leave a window open
    /// across a night the app spent closed — the reason `worktree` expiry does
    /// the same thing at read time rather than on a timer.
    #[test]
    fn an_expired_window_removes_its_own_flag_and_captures() {
        let root = scratch("expiry");
        bridge(&root, "shop");
        arm(&root, "shop", 30).unwrap();

        // Backdated by hand, which is the only way to reach the branch without
        // a clock this test would then depend on.
        let stale = format!(
            "{{\"windows\":[{{\"project\":\"shop\",\"expires\":\"2000-01-01T00:00:00Z\"}}]}}"
        );
        std::fs::write(path(&root), stale).unwrap();
        std::fs::create_dir_all(sessions_path(&root, "shop").parent().unwrap()).unwrap();
        std::fs::write(sessions_path(&root, "shop"), "{}\n").unwrap();

        assert!(armed(&root, "shop").is_none());
        assert!(
            !flag_path(&root, "shop").exists(),
            "the bridge stops writing"
        );
        assert!(
            !sessions_path(&root, "shop").exists(),
            "and what it wrote is gone"
        );
    }

    /// A permission that would grant nothing is refused instead of taken.
    ///
    /// The two flags are separate on purpose and not independent: the bridge's
    /// prepend file is the only thing that reads `capture.flag`, so arming
    /// while the bridge is off would write an audit entry saying credentials
    /// are being recorded, record none, and leave the person concluding their
    /// `POST` cannot be replayed.
    #[test]
    fn arming_is_refused_while_the_bridge_that_would_capture_is_off() {
        let root = scratch("nobridge");
        assert!(!bridge_on(&root, "shop"));

        let refused = arm(&root, "shop", 30).unwrap_err();
        assert!(
            refused.message.contains("debug bridge is off"),
            "the reason has to be the one somebody can act on: {}",
            refused.message
        );

        // And nothing was taken: no flag for the bridge to find, and no window
        // for `armed` to report.
        assert!(!flag_path(&root, "shop").exists());
        assert!(armed(&root, "shop").is_none());

        // With the bridge on, the same call is the permission it was.
        bridge(&root, "shop");
        assert!(arm(&root, "shop", 30).is_ok());
        assert!(flag_path(&root, "shop").is_file());
    }

    /// The join, and the wrong answer it exists to avoid.
    ///
    /// The request line alone would attach one visitor's basket to another
    /// visitor's recording of the same page; the clock alone would attach
    /// whatever happened to be next.
    #[test]
    fn a_session_is_matched_by_the_line_and_the_clock_together() {
        let sessions = [
            session(1_000.4, "POST /checkout"),
            session(1_000.9, "POST /checkout"),
            session(1_001.0, "GET /cart"),
        ];

        // Nearest inside the window wins, so two requests to one URL a moment
        // apart do not swap.
        let hit = matching(&sessions, "POST /checkout", 1_001).unwrap();
        assert_eq!(hit.at, 1_000.9);

        // A different line at the same moment is not this recording's session.
        assert!(matching(&sessions, "POST /pay", 1_001).is_none());

        // And the same line far away is somebody else's request.
        assert!(matching(&sessions, "POST /checkout", 2_000).is_none());
    }

    /// What a screen may say about a session without being a second place the
    /// credential exists.
    #[test]
    fn a_session_is_described_by_counts_and_never_quoted() {
        let described = session(1.0, "POST /checkout").describe();

        assert_eq!(described.method, "POST");
        assert_eq!(described.cookies, 1);
        assert_eq!(described.body_bytes, 5);

        let json = serde_json::to_string(&described).unwrap();
        assert!(!json.contains("abc"), "the cookie value must not be here");
        assert!(!json.contains("qty=2"), "nor the body");

        // Two cookies count as two, and an absent one as none rather than one
        // empty name.
        let mut two = session(1.0, "POST /x");
        two.cookie = Some("a=1; b=2;".into());
        assert_eq!(two.describe().cookies, 2);
        two.cookie = None;
        assert_eq!(two.describe().cookies, 0);
    }
}
