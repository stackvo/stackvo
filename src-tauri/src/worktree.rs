//! N — a branch with an environment of its own.
//!
//! `git worktree add` checks a second branch out into a second directory while
//! sharing one repository. This gives that directory everything a project here
//! gets: its own container, its own hostname, its own database and its own
//! environment variables — so `feature-x.shop.loc` and `shop.loc` are two
//! running copies of the same application on two branches, at the same time.
//!
//! **Measured August 2026, against seventeen products.** Nothing else gives a
//! branch its own *database and environment*; the nearest is dde, which gives
//! each worktree a hostname and a TLS certificate and stops there. An earlier
//! version of this line said "the one thing nothing else in this space does",
//! which stopped being true without anybody noticing — an undated claim about
//! the outside world does not become old, it becomes wrong. The lead this
//! measurement records is the two halves dde has not built, not the whole idea.
//!
//! The reason it is natural *here* is that everything it needs
//! already exists: a project is a directory with a manifest, a hostname is a
//! Traefik rule, and a database is a `CREATE DATABASE` on an instance that is
//! already running. There is no new machinery, only a new arrangement of it.
//!
//! ## A worktree is a project. It is not a second kind of thing
//!
//! The directory goes into the project tree beside its parent and is picked up
//! by `list_projects`, `render_generated`, the hosts writer and the certificate
//! exactly as any other project is. Nothing downstream of this module knows the
//! word "worktree", and that is deliberate: a parallel lifecycle for a project
//! that is a project would be a second set of every bug.
//!
//! ## Nothing is written into the checkout that git tracks
//!
//! This is the constraint the whole design turns on. The files in a worktree
//! directory **are the branch's files**. Writing a derived `stackvo.json` there
//! would show up as a modification to whoever is working on that branch, and
//! writing `.stackvo/site.json` would do the same wherever a team commits it.
//!
//! So the two halves go to two places:
//!
//! * **Identity** — the name and the hostname — goes into `stackvo.local.json`,
//!   the machine-local overlay already defined, which exists precisely to be
//!   the file that is never committed. `manifest::local_name_refused` is the one
//!   change this feature needed there: an overlay may restate the directory it
//!   sits in, and nothing else about identity.
//! * **Environment** — the database credentials and anything else — stays in
//!   this module's own table under the app root, outside every checkout, and
//!   reaches the container through the compose overlay `site.rs` already
//!   renders.
//!
//! `stackvo.local.json` is also added to the repository's `.git/info/exclude`
//! when git says it is not ignored yet. That file is git's own local ignore
//! list: it is never committed, it is shared by the main checkout and every
//! linked worktree, and it is the right home for a rule about a file the
//! repository must not carry. The user's `.gitignore` is theirs and is not
//! touched.
//!
//! ## The database is a database, not an instance
//!
//! A branch is not a different engine. Giving each worktree its own MySQL
//! container would cost a gigabyte of memory to hold a copy of one schema, so a
//! worktree gets a database *on the instance that is already running* — created
//! empty, or copied from the workspace's own so the branch starts with the data
//! that was there. See [`crate::db::create_database`] and
//! [`crate::db::copy_database`].
//!
//! ## And a login of its own, which is a second promise
//!
//! "Its own database" and "cannot reach the parent's" are not the same
//! sentence, and for a long time only the first was true here: the branch was
//! handed the *instance's* login, so the parent's data was one `USE shop;`
//! away. That was a small thing while a worktree was somebody's second branch.
//! It is the whole claim once the thing working in there is an assistant that
//! was told to fix a failing test and decided a migration would do it.
//!
//! So the worktree is also given a database account granted on that schema
//! alone — see [`crate::db::create_scoped_user`], which explains why that is
//! arranged on MySQL and MariaDB and refused rather than approximated on the
//! other two. The password lives in [`logins_path`] and not on [`Record`],
//! because `Record` is serialised across the IPC boundary as well as to disk.
//!
//! Where no account could be arranged the branch keeps the shared login and the
//! app **says so** — `worktree_list` reports `isolated` and the pane prints the
//! sentence. An isolation that is claimed and not arranged is worse than one
//! that was never offered: it is the one somebody stops checking.

use crate::error::{Code, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Bumped when a field changes meaning, not when one is added — the rule
/// `instances.rs` states, for the same reason: this file names directories a
/// removal will delete and databases a removal will drop.
pub const SCHEMA_VERSION: u32 = 1;

/// The longest a branch slug may be.
///
/// A DNS label may be 63 characters and the slug is one whole label, so the
/// limit is not about DNS. It is about the *project name*, `<parent>-<slug>`,
/// which becomes a directory, a container name and an image reference, and a
/// 60-character branch name would produce one nobody can read in a list. A
/// truncated slug can collide, and a collision is caught by name — loudly, with
/// the name shown — rather than by making the limit large enough to hope.
const SLUG_LIMIT: usize = 40;

/// `<root>/worktrees.json`.
pub fn path(root: &Path) -> PathBuf {
    root.join("worktrees.json")
}

/// `<root>/worktree-logins.json` — the passwords, and nothing else.
///
/// ## Why a second file rather than a field on the record
///
/// [`Table`] crosses the IPC boundary: `worktree_list` hands its records to a
/// webview. A password on `Record` would be serialised by the same derive that
/// writes the file and shipped to the browser with it, and remembering to strip
/// it at every call site is exactly the arrangement this repository keeps
/// replacing with one that cannot be got wrong. A field that must never reach
/// the webview does not live in a struct that goes there.
///
/// The file is written `0600` where the platform has such a thing, for the same
/// reason `elevate::staging_dir` is: this is the only file in the workspace
/// whose whole content is credentials.
pub fn logins_path(root: &Path) -> PathBuf {
    root.join("worktree-logins.json")
}

/// The login one worktree's container is given, when it has one of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    /// The instance the account lives on, so a teardown knows where to drop it.
    pub instance: String,
    pub user: String,
    pub password: String,
}

/// Every worktree login this machine holds, by worktree name.
pub fn logins(root: &Path) -> BTreeMap<String, Login> {
    std::fs::read_to_string(logins_path(root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn login_of(root: &Path, name: &str) -> Option<Login> {
    logins(root).get(name).cloned()
}

fn save_logins(root: &Path, table: &BTreeMap<String, Login>) -> Result<()> {
    let file = logins_path(root);
    let mut text = serde_json::to_string_pretty(table)
        .map_err(|e| Error::new(Code::IoError, format!("serialising worktree logins: {e}")))?;
    text.push('\n');
    crate::atomic::write(&file, &text)?;
    restrict(&file);
    Ok(())
}

/// `0600`, where the platform has such a thing.
///
/// Best effort and deliberately not fatal: a file this app just wrote that it
/// then cannot chmod is a permissions oddity on that machine, and refusing to
/// create the worktree over it would trade a working feature for a hardening
/// step. Windows has no mode bits and the file inherits the user's profile
/// permissions, which is the same answer the OS gives for every other file
/// here.
fn restrict(file: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = file;
}

/// Remember one worktree's login.
pub fn remember_login(root: &Path, name: &str, login: Login) -> Result<()> {
    let mut table = logins(root);
    table.insert(name.to_string(), login);
    save_logins(root, &table)
}

/// Forget it, and hand it back so the caller can drop the account it names.
pub fn forget_login(root: &Path, name: &str) -> Option<Login> {
    let mut table = logins(root);
    let gone = table.remove(name)?;
    let _ = save_logins(root, &table);
    Some(gone)
}

/// The database one worktree was given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    /// The instance it lives on, as `instances.json` ids it.
    pub instance: String,
    pub name: String,
    /// The database it was copied from, when it was copied rather than created
    /// empty. Kept because it is the only record of where the data came from,
    /// and "is this a copy of production or an empty schema" is the question
    /// somebody asks three weeks later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeded_from: Option<String>,
}

/// One worktree, as this app records it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// The project name, which is also the directory name. The identity.
    pub name: String,
    /// The project this was branched from, by name.
    pub parent: String,
    /// The git branch checked out into it.
    pub branch: String,
    /// The hostname it answers on.
    pub domain: String,
    /// Absolute, and recorded rather than derived: the project tree can be
    /// moved, and a record that pointed at where the directory *would* be would
    /// send `git worktree remove` at a path git has never heard of.
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<Database>,
    /// Extra environment variables for this worktree's container.
    ///
    /// The database credentials are **not** in here: they are computed from the
    /// instance at render time by [`env_for`], so a password that changes in
    /// one place does not leave a stale copy in this file. What is here is what
    /// somebody typed.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// RFC 3339.
    pub created_at: String,
    /// When this worktree stops being wanted. RFC 3339, UTC.
    ///
    /// The field that turns a worktree into a **sandbox**: a branch environment
    /// made for one task, by somebody who is not going to remember it exists.
    /// A person's worktree is theirs until they say otherwise and carries
    /// `None`, which is why this is an option rather than a date far away.
    ///
    /// Nothing acts on it by itself. An app that deleted a directory on a timer
    /// would eventually delete one with a morning's uncommitted work in it, and
    /// no expiry policy is worth that. What the date does is make "this is
    /// finished with" a fact the screen can state and a person can act on in
    /// one click — see [`expired`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// The whole file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub worktrees: Vec<Record>,
}

impl Table {
    /// Read it, or an empty table when there is none.
    pub fn load(root: &Path) -> Result<Self> {
        let file = path(root);
        let text = match std::fs::read_to_string(&file) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    schema_version: SCHEMA_VERSION,
                    worktrees: Vec::new(),
                })
            }
            Err(e) => return Err(Error::io(format!("reading {}", file.display()), e)),
        };

        let table: Self = serde_json::from_str(&text).map_err(|e| {
            Error::new(
                Code::InvalidManifest,
                format!("{} is not readable: {e}", file.display()),
            )
        })?;

        if table.schema_version > SCHEMA_VERSION {
            return Err(Error::new(
                Code::Unsupported,
                format!(
                    "{} is version {} and this app understands {SCHEMA_VERSION} — \
                     it names directories a removal deletes and databases a removal drops, \
                     so a newer file is refused rather than half-read",
                    file.display(),
                    table.schema_version
                ),
            ));
        }
        Ok(table)
    }

    /// Write it, atomically.
    pub fn save(&self, root: &Path) -> Result<()> {
        let file = path(root);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
        }
        let mut text = serde_json::to_string_pretty(&Self {
            schema_version: SCHEMA_VERSION,
            worktrees: self.worktrees.clone(),
        })
        .map_err(|e| Error::new(Code::IoError, format!("serialising worktrees: {e}")))?;
        text.push('\n');
        crate::atomic::write(&file, &text)
    }

    pub fn get(&self, name: &str) -> Option<&Record> {
        self.worktrees.iter().find(|w| w.name == name)
    }

    /// Every worktree of one project, in table order.
    pub fn of_parent<'a>(&'a self, parent: &'a str) -> impl Iterator<Item = &'a Record> {
        self.worktrees.iter().filter(move |w| w.parent == parent)
    }

    /// Add one, refusing a name that is already recorded.
    pub fn insert(&mut self, record: Record) -> Result<()> {
        if self.get(&record.name).is_some() {
            return Err(Error::new(
                Code::AlreadyExists,
                format!("a worktree called \"{}\" is already recorded", record.name),
            ));
        }
        self.worktrees.push(record);
        self.worktrees.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(())
    }

    /// Take one out, and hand it back so the caller can clean up after it.
    pub fn remove(&mut self, name: &str) -> Option<Record> {
        let at = self.worktrees.iter().position(|w| w.name == name)?;
        Some(self.worktrees.remove(at))
    }
}

// ------------------------------------------------------------------- expiry

/// The longest a sandbox may be asked to last.
///
/// Seven days. Not a technical limit — it is the point past which "temporary"
/// has stopped being the word for it, and a worktree meant to live longer is a
/// worktree, which is what the same screen makes when the field is left empty.
pub const MAX_TTL_MINUTES: u32 = 7 * 24 * 60;

/// `now + minutes`, as the same fixed-width UTC string every other timestamp
/// in this app is written in.
///
/// Fixed width and UTC is the whole reason nothing here parses a date:
/// `"2026-08-30T09:00:00Z" < "2026-08-30T09:30:00Z"` is true as *text*, so the
/// comparison in [`expired_at`] is a string comparison and there is no date
/// library, no timezone and no second implementation of anybody's calendar.
/// `audit.rs` states the same property for the same reason.
pub fn expiry_in(minutes: u32) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    crate::audit::rfc3339_of(now + i64::from(minutes.min(MAX_TTL_MINUTES)) * 60)
}

/// Has this worktree's time passed, as of `now`?
///
/// `None` is never expired, and that is the ordinary case: a worktree somebody
/// made for themselves has no date on it.
pub fn expired_at(record: &Record, now: &str) -> bool {
    record
        .expires_at
        .as_deref()
        .is_some_and(|expires| expires <= now)
}

pub fn expired(record: &Record) -> bool {
    expired_at(record, &crate::audit::now_rfc3339())
}

/// How many whole minutes are left, rounded **down**.
///
/// Down, because this number is handed to a grant as `--for`, and rounding a
/// remaining forty seconds up to a minute would be the app granting time the
/// sandbox no longer has. `None` for a worktree with no expiry and `Some(0)`
/// for one whose time has passed — two different answers that a single
/// `Option` would blur into one.
pub fn remaining_minutes_at(record: &Record, now: &str) -> Option<u32> {
    let expires = crate::audit::seconds_of_rfc3339(record.expires_at.as_deref()?)?;
    let now = crate::audit::seconds_of_rfc3339(now)?;
    Some(u32::try_from((expires - now).max(0) / 60).unwrap_or(u32::MAX))
}

pub fn remaining_minutes(record: &Record) -> Option<u32> {
    remaining_minutes_at(record, &crate::audit::now_rfc3339())
}

/// The flags that would grant an assistant this sandbox and nothing else.
///
/// Built here rather than typed on screen so the sentence a person copies is
/// the one [`crate::grant`] enforces — the same reason `agents.rs` renders the
/// registration from a `Grant` instead of assembling the strings itself.
///
/// The two clocks are deliberately the same number and mean different things,
/// which is worth saying once: the sandbox's expiry is when the *environment*
/// is finished with, and `--for` is how long the writing tools last **from each
/// start of the server**. An assistant restarted an hour later gets its writes
/// back and still cannot touch anything but this branch.
pub fn grant_for(record: &Record, minutes: Option<u32>) -> crate::grant::Grant {
    let mut grant = crate::grant::Grant::everything().scoped_to(vec![record.name.clone()]);
    if let Some(minutes) = minutes.filter(|m| *m > 0) {
        grant = grant.lasting(std::time::Duration::from_secs(u64::from(minutes) * 60));
    }
    grant
}

/// Is this project a worktree of another one?
///
/// Read from the table rather than from git, because the question every caller
/// is asking is "did this app make it", not "is the directory a linked
/// checkout". A worktree somebody added by hand and then adopted as a project
/// is a project; this app did not give it a database and must not offer to drop
/// one.
pub fn record_of(root: &Path, name: &str) -> Option<Record> {
    Table::load(root).ok()?.get(name).cloned()
}

/// Forget a worktree without touching git or Docker.
///
/// For `project_delete`, which can reach a worktree by its ordinary Delete
/// button. That path removes the directory and everything Docker held; what it
/// cannot know about is this table and git's own registration, and a record left
/// behind would keep offering a database drop for a project that is gone.
///
/// Returns the record when there was one, so the caller can say what it also
/// cleaned up.
pub fn forget(root: &Path, name: &str) -> Option<Record> {
    let mut table = Table::load(root).ok()?;
    let record = table.remove(name)?;
    if let Err(e) = table.save(root) {
        tracing::warn!(worktree = %name, error = %e.message, "the worktree record was not removed");
        return None;
    }
    // git keeps its own registration under `.git/worktrees/<id>`, and a
    // directory that has gone without `git worktree remove` leaves it behind:
    // the branch stays locked to a path nothing is at, and checking it out
    // anywhere answers "already used by worktree". `prune` is the command git
    // provides for exactly that and is a no-op when there is nothing stale.
    //
    // A worktree is never a worktree's parent — creating one from a linked
    // checkout is refused — so the parent is always an ordinary project
    // directory and there is only one place to look for it.
    if let Some(parent) = crate::workspace::projects_root(root).map(|p| p.join(&record.parent)) {
        prune(&parent);
    }
    Some(record)
}

// ------------------------------------------------------------- derivations
//
// Pure, and tested as such. Every name a worktree has comes from one pair —
// the parent and the branch — the same rule `instances::slug` states, and for
// the same reason: a name stored twice is a name that can disagree with itself.

/// `feature/ABC-123 Login` → `feature-abc-123-login`.
///
/// One DNS label, because it becomes one: the leftmost label of the worktree's
/// hostname, and the tail of its container name. Git allows a branch name to
/// hold slashes, dots, unicode and a great deal else; a hostname allows letters,
/// digits and hyphens.
///
/// Anything outside the set is folded to a hyphen rather than dropped. Dropping
/// is how `feature/a-1` and `feature/a1` arrive at one slug, and two branches
/// with one hostname is the failure this whole feature exists to avoid.
/// Consecutive hyphens then collapse, which is cosmetic and is why it is done
/// after the fold rather than instead of it.
///
/// `None` when nothing usable is left — a branch named `...` or `安全` has no
/// ASCII to build a label from, and inventing one would be inventing a name the
/// user cannot predict.
pub fn slug(branch: &str) -> Option<String> {
    let folded: String = branch
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    let mut out = String::with_capacity(folded.len());
    for c in folded.chars() {
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }

    let out = out.trim_matches('-').to_string();
    // Truncation can land on a hyphen, and a label may not end in one.
    let out: String = out.chars().take(SLUG_LIMIT).collect();
    let out = out.trim_end_matches('-').to_string();

    let usable = !out.is_empty() && out.starts_with(|c: char| c.is_ascii_alphanumeric());
    usable.then_some(out)
}

/// The project name a worktree is filed under: `shop` + `feature-x` →
/// `shop-feature-x`.
pub fn project_name(parent: &str, slug: &str) -> String {
    format!("{parent}-{slug}")
}

/// The label a name contributes to the hostname.
///
/// `shop-feature-x` under `shop` is `feature-x`; a name somebody chose that
/// does not start with the parent's is itself. Deriving the hostname from the
/// *name* rather than from the branch is what keeps the two in step when a user
/// overrides one of them — the alternative is a project called `staging` living
/// at `feature-x.shop.loc`.
pub fn domain_label(parent: &str, name: &str) -> String {
    name.strip_prefix(&format!("{parent}-"))
        .filter(|rest| !rest.is_empty())
        .unwrap_or(name)
        .to_string()
}

/// `feature-x` under `shop.loc` → `feature-x.shop.loc`.
///
/// A subdomain of the parent's, not a sibling. Two reasons, and the second is
/// the one that matters: it reads as what it is — a branch *of* that project —
/// and it stays inside any wildcard certificate or wildcard route the parent
/// already has, so the browser does not meet an interstitial on the first load.
pub fn domain(parent_domain: &str, label: &str) -> String {
    format!("{label}.{parent_domain}")
}

/// The database name a worktree gets: `shop` + `feature-x` → `shop_feature_x`.
///
/// Underscores rather than hyphens. A hyphen in a database name is legal in all
/// three SQL engines and has to be quoted in every single statement that names
/// it — including the ones somebody types into a client later — and the first
/// unquoted one is a syntax error in a migration.
///
/// Truncated from the *left* of the stem when it will not fit, keeping the whole
/// branch part: two worktrees of one project differ in the branch and share the
/// stem, so a limit that cut the branch off would give them the same database.
pub fn database_name(stem: &str, slug: &str) -> String {
    let clean = |s: &str| -> String {
        s.to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    };

    let branch = clean(slug);
    let stem = clean(stem);

    // 63 is the shortest identifier limit of the engines this supports; see
    // `db::is_valid_database_name`.
    let room = 63usize.saturating_sub(branch.len() + 1);
    let stem: String = stem.chars().take(room).collect();
    let stem = stem.trim_end_matches('_').to_string();

    let joined = if stem.is_empty() {
        branch
    } else {
        format!("{stem}_{branch}")
    };

    // A name must begin with a letter — `db::is_valid_database_name` says why —
    // and a branch called `2fa` would otherwise produce one that does not.
    if joined.starts_with(|c: char| c.is_ascii_lowercase()) {
        joined
    } else {
        let prefixed = format!("w_{joined}");
        prefixed.chars().take(63).collect()
    }
}

/// The `stackvo.local.json` a worktree is given.
///
/// Two keys and no more. `name` is what stops W-04 from reporting a project
/// that cannot be reached — the committed manifest is the branch's and says the
/// parent's name — and `domain` is the whole point of the exercise.
///
/// Written as text rather than through `serde_json` on a struct, because this
/// is a file a person opens and edits afterwards: the comment-free JSON they
/// find should be in the order they would have written it.
pub fn local_overlay(name: &str, domain: &str) -> String {
    format!("{{\n  \"name\": \"{name}\",\n  \"domain\": \"{domain}\"\n}}\n")
}

// -------------------------------------------------------------- environment

/// What a worktree's container is given, over and above the project's own.
///
/// Derived on every render rather than stored, so a root password changed in
/// Settings reaches every worktree at the next generate instead of leaving a
/// stale copy in a JSON file nobody would think to look in.
///
/// The database half is absent when the worktree has no database, which is a
/// perfectly ordinary state: a static site on a branch needs a hostname and
/// nothing else.
///
/// `APP_URL` is here because it is the variable that makes a per-branch
/// environment behave like one. The branch's own `.env` names the parent's
/// hostname, and a framework that generates links, redirects and mail from it
/// would send everybody from `feature-x.shop.loc` back to `shop.loc` mid-flow —
/// which is the exact bug this feature would be sold as fixing.
///
/// The container's environment beats the application's `.env` file, which is
/// what makes any of this work: PHP's dotenv libraries load a variable only when
/// the process does not already have one, so a value set here wins without this
/// app ever touching the framework's file (M-5's rule, unchanged).
///
/// Whatever the user typed is laid over the derived values last, so a worktree
/// that wants a different `APP_URL` — or none — can say so.
pub fn env_for(root: &Path, record: &Record) -> BTreeMap<String, String> {
    let mut env: BTreeMap<String, String> = BTreeMap::new();

    env.insert("APP_URL".into(), format!("https://{}", record.domain));
    // What the branch is, for anything that wants to say so on screen. Two
    // variables rather than one composite, because a template that wants only
    // the branch should not have to split a string.
    env.insert("STACKVO_WORKTREE".into(), record.name.clone());
    env.insert("STACKVO_WORKTREE_BRANCH".into(), record.branch.clone());

    if let Some(database) = &record.database {
        if let Ok(connection) = crate::db::connection(root, &database.instance) {
            // The worktree's own account when it has one, and the instance's
            // shared account otherwise. The substitution happens here, in the
            // one function that renders these variables, so a branch either has
            // its own login everywhere or nowhere — there is no third state in
            // which one file says one thing and another says the other.
            let connection = match login_of(root, &record.name) {
                Some(login) if login.instance == database.instance => crate::db::Connection {
                    user: login.user,
                    password: Some(login.password),
                    ..connection
                },
                _ => connection,
            };

            env.insert("DB_CONNECTION".into(), driver_name(connection.kind).into());
            env.insert("DB_HOST".into(), connection.host.clone());
            env.insert("DB_PORT".into(), connection.port.to_string());
            env.insert("DB_DATABASE".into(), database.name.clone());
            env.insert("DB_USERNAME".into(), connection.user.clone());
            if let Some(password) = &connection.password {
                env.insert("DB_PASSWORD".into(), password.clone());
            }
            env.insert(
                "DATABASE_URL".into(),
                database_url(&connection, &database.name),
            );
        } else {
            // The instance was removed under the worktree. Not fatal and not
            // silent: the project still runs at its own hostname, and the pane
            // reports the database as missing.
            tracing::warn!(
                worktree = %record.name,
                instance = %database.instance,
                "the worktree's database instance is not in the table; no credentials were supplied"
            );
        }
    }

    // Sanctum's two, and only where they change something — see [`Auth`].
    // Before the record's own variables, so a value somebody typed still wins.
    let auth = auth_for(root, record);
    if let Some(domain) = auth.session_domain {
        env.insert("SESSION_DOMAIN".into(), domain);
    }
    if let Some(domains) = auth.stateful_domains {
        env.insert("SANCTUM_STATEFUL_DOMAINS".into(), domains);
    }

    for (key, value) in &record.env {
        env.insert(key.clone(), value.clone());
    }

    // The same rules the overlay enforces on a project's own variables. A value
    // with a newline in it would end the YAML scalar it is written into, and a
    // derived value has no more right to do that than a typed one.
    env.retain(|key, value| {
        crate::site::checked_key(key).is_ok() && crate::site::checked_value(value).is_ok()
    });
    env
}

// -------------------------------------------- a session, on a new hostname

/// What a branch's own hostname breaks, and what it takes to un-break it.
///
/// ## The bug this feature produced by working
///
/// Every worktree gets a hostname of its own. Sanctum's SPA mode ties a session
/// to a **list of hostnames** — `sanctum.stateful` and `SESSION_DOMAIN` — and
/// the new host is in neither, so signing in on a branch returns 419 or 401
/// with nothing on screen saying why. The developer blames their own code.
///
/// Passport is blunter. Its signing keys live in `storage/oauth-private.key`,
/// and that file is in `.gitignore` — so a fresh worktree does not have it, and
/// every token request comes back as a stack trace about a missing file.
///
/// ## Two rules, and both are refusals
///
/// | Rule | Why |
/// | --- | --- |
/// | A value the user wrote is never overwritten | This file is the application's, not this app's — the line `env_writer` already holds. In [`env_for`] the record's own variables are laid over these, so anything typed wins |
/// | The parent's key is **never copied** | Copying a signing key into a second environment is the exact opposite of what a worktree's database isolation is for: the branch would be able to mint tokens for the place it branched from |
///
/// ## And two values that are written only when they change something
///
/// Neither is set unconditionally, and that is the measurement rather than a
/// preference:
///
/// * **`SANCTUM_STATEFUL_DOMAINS`** — when the project does not pin one,
///   Sanctum's own default already follows `APP_URL`, which [`env_for`] has
///   just pointed at this branch. Writing a list there would *replace* that
///   default and could only take hostnames away. It is written only where the
///   project pins a list of its own, and then the branch's host is **appended**
///   to it rather than replacing it.
/// * **`SESSION_DOMAIN`** — Laravel's own default is the current host, which is
///   already right. It is written only where the project pins a domain that
///   does not cover the branch's host, which is the case where the cookie would
///   otherwise never be sent.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    /// The version `composer.lock` names, when it names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanctum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passport: Option<String>,
    /// What this worktree's container is given, when anything is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stateful_domains: Option<String>,
    /// Passport is installed and this worktree has no signing key.
    pub passport_keys_missing: bool,
}

/// Where Passport keeps the key a fresh checkout does not have.
pub const PASSPORT_PRIVATE_KEY: &str = "storage/oauth-private.key";

/// Does a pinned cookie domain cover this host?
///
/// `.shop.loc` covers `feature-x.shop.loc`; `shop.loc` covers it too, because
/// that is what a cookie domain means — a domain matches itself and everything
/// under it. `api.shop.loc` covers neither. Written out rather than assumed
/// because the whole value of the check is that it does **not** write a
/// variable where the project's own value already works.
fn covers(pinned: &str, host: &str) -> bool {
    let pinned = pinned.trim().trim_start_matches('.');
    !pinned.is_empty() && (host == pinned || host.ends_with(&format!(".{pinned}")))
}

/// `SESSION_DOMAIN` for this branch, or `None` where nothing needs saying.
pub fn session_domain_for(pinned: Option<&str>, host: &str) -> Option<String> {
    match pinned.map(str::trim).filter(|v| !v.is_empty()) {
        // Laravel's own default is the current host, which is already this
        // branch. Writing the value would change nothing and take a decision
        // away from a file this app does not own.
        None => None,
        Some(pinned) if covers(pinned, host) => None,
        Some(_) => Some(host.to_string()),
    }
}

/// `SANCTUM_STATEFUL_DOMAINS` for this branch, or `None`.
///
/// Appended, never replaced: the pinned list is somebody's, and a branch that
/// took `localhost:3000` out of it would break the SPA that is being developed
/// against it.
pub fn stateful_domains_for(pinned: Option<&str>, host: &str) -> Option<String> {
    let pinned = pinned.map(str::trim).filter(|v| !v.is_empty())?;
    if pinned
        .split(',')
        .any(|entry| entry.trim().eq_ignore_ascii_case(host))
    {
        return None;
    }
    Some(format!("{pinned},{host}"))
}

/// One value out of the first `.env` that has it: the worktree's own, then the
/// project it was branched from.
///
/// In that order because a worktree that has been given its own file has been
/// given it deliberately, and the parent's is what a fresh checkout is actually
/// running against — `.env` is in `.gitignore`, so a new worktree usually has
/// none at all.
fn pinned_env(root: &Path, record: &Record, key: &str) -> Option<String> {
    let parent = crate::workspace::projects_root(root).map(|dir| dir.join(&record.parent));
    [Some(PathBuf::from(&record.path)), parent]
        .into_iter()
        .flatten()
        .find_map(|dir| crate::dashboards::env_value(&dir, key))
        .filter(|value| !value.is_empty())
}

/// What Sanctum and Passport need in this worktree.
///
/// Read from the worktree's own `composer.lock` — the branch's, not the
/// parent's, because a branch that adds or removes a package is exactly the
/// case where the two disagree.
pub fn auth_for(root: &Path, record: &Record) -> Auth {
    let dir = PathBuf::from(&record.path);
    let lock = std::fs::read_to_string(dir.join("composer.lock")).unwrap_or_default();
    let deps = crate::deps::parse_composer_lock(&lock, &Default::default());
    let version = |package: &str| {
        deps.iter()
            .find(|d| d.ecosystem == crate::deps::Ecosystem::Packagist && d.name == package)
            .map(|d| d.version.clone())
    };

    let sanctum = version("laravel/sanctum");
    let passport = version("laravel/passport");

    Auth {
        session_domain: sanctum.as_ref().and_then(|_| {
            session_domain_for(
                pinned_env(root, record, "SESSION_DOMAIN").as_deref(),
                &record.domain,
            )
        }),
        stateful_domains: sanctum.as_ref().and_then(|_| {
            stateful_domains_for(
                pinned_env(root, record, "SANCTUM_STATEFUL_DOMAINS").as_deref(),
                &record.domain,
            )
        }),
        // Asked only where Passport is installed: a missing file in a project
        // that never had Passport is not a finding, it is a project.
        passport_keys_missing: passport.is_some() && !dir.join(PASSPORT_PRIVATE_KEY).is_file(),
        sanctum,
        passport,
    }
}

/// What the framework calls this engine.
///
/// MariaDB is `mysql`, deliberately. Laravel only grew a `mariadb` driver in
/// version 11, and a value the application does not recognise is a connection
/// that does not exist rather than one that is slightly mislabelled — the
/// safer answer is the one every version has understood.
fn driver_name(kind: crate::db::Kind) -> &'static str {
    match kind {
        crate::db::Kind::Mysql | crate::db::Kind::Mariadb => "mysql",
        crate::db::Kind::Postgres => "pgsql",
        crate::db::Kind::Mongo => "mongodb",
    }
}

/// `mysql://user:pass@host:3306/db`, with the password percent-encoded.
///
/// Doctrine and anything else reading `DATABASE_URL` parses this as a URL, and
/// a generated root password containing `@`, `/` or `#` would otherwise end the
/// userinfo early and point the application at a host that does not exist.
fn database_url(connection: &crate::db::Connection, database: &str) -> String {
    let scheme = crate::db::url_scheme(connection.kind);
    let user = percent_encode(&connection.user);
    let host = &connection.host;
    let port = connection.port;

    match &connection.password {
        Some(password) => {
            let password = percent_encode(password);
            format!("{scheme}://{user}:{password}@{host}:{port}/{database}")
        }
        None => format!("{scheme}://{user}@{host}:{port}/{database}"),
    }
}

/// Percent-encoding for a URL's userinfo field.
///
/// An allowlist, not a list of characters to escape: the unreserved set from
/// RFC 3986 passes and everything else is encoded, so a character nobody
/// thought of is escaped rather than let through.
///
/// Public under a second name for one caller: the masking in `commands.rs` has
/// to find the password inside `DATABASE_URL`, and it is in there encoded. A
/// second implementation of this would be a mask that misses.
pub fn url_encoded(value: &str) -> String {
    percent_encode(value)
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

// --------------------------------------------------------------------- git
//
// Everything below asks git a question and reports what it said. The same
// posture `git.rs` takes and for the same reason: the person running this has a
// working git, and a second implementation of "which branches are there" would
// be a worse copy of one that is already correct.
//
// Run synchronously. These are local metadata reads that finish in a few
// milliseconds — `git worktree list` reads one directory — and the one command
// that is not, `worktree add`, goes through `runner::run_operation` so its
// output streams to the console like every other long step.

/// Run git in a directory and hand back stdout, or `None` if it failed.
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    if !crate::git::available() {
        return None;
    }
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        // The same two as a clone, for the same reason: there is no terminal
        // for git to ask a question in. Neither of these commands should ever
        // prompt, and "should never" is how a windowed app hangs.
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;

    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).to_string())
}

/// Is there a repository at this path?
pub fn is_repository(dir: &Path) -> bool {
    git(dir, &["rev-parse", "--is-inside-work-tree"])
        .map(|out| out.trim() == "true")
        .unwrap_or(false)
}

/// Is this directory a *linked* worktree rather than the main checkout?
///
/// Asked of git rather than of the table, because it is a different question:
/// the table says "this app made it", and this says "git considers it a second
/// checkout of a repository somewhere else". Both matter — the first decides
/// whether a database is ours to drop, the second decides whether a worktree
/// may be created from here at all.
pub fn is_linked_worktree(dir: &Path) -> bool {
    let (Some(own), Some(common)) = (
        git(dir, &["rev-parse", "--absolute-git-dir"]),
        git(
            dir,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        ),
    ) else {
        return false;
    };
    let canonical = |value: &str| {
        let path = PathBuf::from(value.trim());
        std::fs::canonicalize(&path).unwrap_or(path)
    };
    canonical(&own) != canonical(&common)
}

/// The branch checked out here, or `None` when the head is detached.
pub fn current_branch(dir: &Path) -> Option<String> {
    let out = git(dir, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    let branch = out.trim().to_string();
    (!branch.is_empty()).then_some(branch)
}

/// Every local branch, in the order git lists them (most recent commit first).
///
/// Sorted by committer date rather than alphabetically because that is the
/// order the answer is wanted in: the branch somebody wants a worktree for is
/// almost always one they touched this week.
pub fn branches(dir: &Path) -> Vec<String> {
    git(
        dir,
        &[
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname:short)",
            "refs/heads",
        ],
    )
    .map(|out| {
        out.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}

/// One entry of `git worktree list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkout {
    pub path: String,
    /// `None` for a detached head, which is a perfectly ordinary worktree and
    /// simply has no branch to report.
    pub branch: Option<String>,
    /// git's own word for "the directory is gone but the registration is not".
    pub prunable: bool,
}

/// What git thinks is checked out where.
///
/// `--porcelain` because the human format is not stable and has been changed
/// twice; the porcelain one is a documented contract of `key value` lines with
/// a blank line between records.
pub fn checkouts(dir: &Path) -> Vec<Checkout> {
    git(dir, &["worktree", "list", "--porcelain"])
        .map(|out| parse_porcelain(&out))
        .unwrap_or_default()
}

/// The parser, separate from the process that produces the text, so the record
/// shapes git documents can be tested without a repository on disk.
fn parse_porcelain(out: &str) -> Vec<Checkout> {
    let mut all = Vec::new();
    let mut current: Option<Checkout> = None;

    for line in out.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            all.extend(current.take());
            continue;
        }
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "worktree" => {
                all.extend(current.take());
                current = Some(Checkout {
                    path: value.to_string(),
                    branch: None,
                    prunable: false,
                });
            }
            "branch" => {
                if let Some(entry) = current.as_mut() {
                    entry.branch = Some(value.trim_start_matches("refs/heads/").to_string());
                }
            }
            "prunable" => {
                if let Some(entry) = current.as_mut() {
                    entry.prunable = true;
                }
            }
            _ => {}
        }
    }
    all.extend(current);
    all
}

/// Does this worktree have changes that `git worktree remove` would refuse to
/// throw away?
///
/// Three answers, like [`crate::git::is_ignored`]. `None` means git could not
/// say, and a removal that treated that as "clean" would be one that decided a
/// question it had not asked.
pub fn is_dirty(dir: &Path) -> Option<bool> {
    let out = git(dir, &["status", "--porcelain"])?;
    Some(!out.trim().is_empty())
}

/// Would git accept this as a branch name?
///
/// Asked of `git check-ref-format`, which is the only complete answer — the
/// rules run to a page and include things like "no component may end in
/// `.lock`". Two forms are refused before git sees them: a leading `-`, which
/// git itself would read as an option long before it validated anything, and
/// `@{`, which `--branch` *resolves* rather than validates, so `@{-1}` would
/// come back as a legal name meaning "the branch before this one".
pub fn is_valid_branch_name(dir: &Path, name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() || name.starts_with('-') || name.contains("@{") {
        return false;
    }
    git(dir, &["check-ref-format", "--branch", name]).is_some()
}

/// The argv for creating the worktree.
///
/// `--` before the path is what stops a path or a branch that begins with `-`
/// from being read as an option — the same rule [`crate::git::clone_args`]
/// states, and the reason it is applied to a path this app built itself is that
/// the projects directory is one the *user* chose.
pub fn add_args(path: &Path, branch: &str, create: bool) -> Vec<String> {
    let mut args = vec!["worktree".to_string(), "add".to_string()];
    if create {
        args.push("-b".to_string());
        args.push(branch.to_string());
    }
    args.push("--".to_string());
    args.push(path.display().to_string());
    if !create {
        args.push(branch.to_string());
    }
    args
}

/// Is this path one git is tracking?
///
/// `--error-unmatch` is what turns "list the files that match" into a yes/no:
/// the exit code is non-zero when nothing matched, which is the question being
/// asked. `None` when git could not answer at all.
fn is_tracked(path: &Path) -> Option<bool> {
    let dir = path.parent()?;
    let name = path.file_name()?.to_str()?;
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(name)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .ok()?;
    Some(out.status.success())
}

/// Remove a worktree through git, so its registration goes with the directory.
///
/// `--force` discards uncommitted work, which is why it is a parameter rather
/// than something this always passes: the screen says what will be lost before
/// anybody can set it.
///
/// ## The overlay is taken out of the way first
///
/// Found by running this rather than reading it. `git worktree remove` refuses
/// while the tree **contains untracked files** — not only modified ones — and
/// `stackvo.local.json` is an untracked file this app put there itself. So a
/// worktree with no user changes at all could not be removed, and the message
/// git gives says "contains modified or untracked files", which sends somebody
/// looking for work they never did.
///
/// [`exclude_local_file`] usually makes it moot, because an excluded file is
/// not untracked as far as `git status` is concerned. Usually is not a
/// guarantee: a repository this app could not write an exclude line to is a
/// repository where it would refuse forever.
///
/// It is removed only when git is not tracking it — a repository that committed
/// the file did so deliberately, and deleting it would be a modification rather
/// than a tidy-up — and it is **put back** if git refuses anyway, so a removal
/// that fails leaves the worktree exactly as capable as it was. Without that,
/// a refused removal would strip the checkout of its name and hostname and turn
/// it into a project the app reports as broken.
pub fn remove(parent_dir: &Path, worktree_path: &Path, force: bool) -> Result<()> {
    if !crate::git::available() {
        return Err(Error::new(Code::NotFound, "git is not installed.")
            .with_hint(crate::hints::INSTALL_GIT_OR_ADOPT));
    }

    let overlay = worktree_path.join(crate::manifest::LOCAL_FILE);
    let saved = match is_tracked(&overlay) {
        Some(false) => std::fs::read_to_string(&overlay).ok(),
        _ => None,
    };
    if saved.is_some() {
        let _ = std::fs::remove_file(&overlay);
    }
    let restore = || {
        if let Some(text) = &saved {
            let _ = std::fs::write(&overlay, text);
        }
    };

    let mut args: Vec<String> = vec!["worktree".into(), "remove".into()];
    if force {
        args.push("--force".into());
    }
    args.push("--".into());
    args.push(worktree_path.display().to_string());

    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(parent_dir)
        .args(&args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| {
            restore();
            Error::io("running git worktree remove", e)
        })?;

    if out.status.success() {
        return Ok(());
    }
    restore();

    let stderr = String::from_utf8_lossy(&out.stderr);
    let reason = stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("git refused to remove the worktree")
        .to_string();

    // git's own word for it, and the one refusal a user can answer.
    let dirty = stderr.contains("contains modified or untracked files");
    let mut error = Error::new(Code::Conflict, reason);
    if dirty {
        error = error.with_hint(crate::hints::WORKTREE_IS_DIRTY);
    }
    Err(error)
}

/// Delete a branch, once the worktree holding it is gone.
///
/// Best effort by design and never fatal: the worktree is already removed by
/// the time this runs, and refusing to finish because a branch has unmerged
/// commits would leave the removal half done. `-D` because `-d` refuses exactly
/// the case somebody asks for this in — a branch whose work was abandoned.
pub fn delete_branch(parent_dir: &Path, branch: &str) -> bool {
    git(parent_dir, &["branch", "-D", "--", branch]).is_some()
}

/// Clear registrations whose directories are gone.
pub fn prune(parent_dir: &Path) -> bool {
    git(parent_dir, &["worktree", "prune"]).is_some()
}

/// Keep `stackvo.local.json` out of every commit in this repository.
///
/// `.git/info/exclude` and not `.gitignore`. The distinction is the whole
/// reason this is acceptable at all: `.gitignore` is a tracked file that the
/// team shares and that this app has no business editing, while
/// `info/exclude` is git's own per-clone ignore list — never committed, never
/// pushed, and read by the main checkout and every linked worktree alike.
///
/// Only when git says the file is not already ignored. A repository whose
/// `.gitignore` already carries the line needs nothing, and appending anyway is
/// how a file accumulates the same rule five times.
///
/// Returns whether a line was written. Never fatal: a repository this could not
/// write to still gets a working worktree, and the pane reports the file as not
/// ignored, which is the same three-state answer `LocalOverridePane` already
/// shows for an ordinary project.
pub fn exclude_local_file(worktree_dir: &Path) -> bool {
    let overlay = worktree_dir.join(crate::manifest::LOCAL_FILE);
    if crate::git::is_ignored(&overlay) == Some(true) {
        return false;
    }

    let Some(common) = git(
        worktree_dir,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    ) else {
        return false;
    };
    let exclude = PathBuf::from(common.trim()).join("info").join("exclude");

    let existing = std::fs::read_to_string(&exclude).unwrap_or_default();
    if existing
        .lines()
        .any(|line| line.trim() == crate::manifest::LOCAL_FILE)
    {
        return false;
    }

    if let Some(parent) = exclude.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }

    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!(
        "\n# StackVo: this machine's settings for this checkout, never committed.\n{}\n",
        crate::manifest::LOCAL_FILE
    ));

    std::fs::write(&exclude, text).is_ok()
}

// -------------------------------------------------- what a worktree would be

// The plan half of this module's own plan-then-apply pair, which used to live
// in `commands.rs` while `create` and `remove` lived here. Nothing in it takes
// an `AppHandle` or a `State` — a path, a name and a request in, a plain plan
// out — so the band rule in `ARCHITECTURE.md` put it on the wrong side of the
// line, and every refusal this module makes was written where no test of this
// module could reach it.

/// What a worktree would be given.
///
/// A plan-then-apply pair, the same shape as `hosts_plan`/`hosts_apply` and
/// `db_move_plan`/`db_move_apply`, and for the sharper version of their reason:
/// this creates a directory, a hostname, a container and a database, and the
/// only moment those can be argued with is before they exist. Every refusal is
/// a sentence rather than a boolean, because "cannot" with no reason is the
/// message people file bugs about.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreePlan {
    pub parent: String,
    pub branch: String,
    /// Whether the branch would be created rather than checked out.
    pub new_branch: bool,
    pub name: String,
    pub path: String,
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<PlannedDatabase>,
    /// When it would expire, when a duration was asked for. Shown before
    /// anything is created, like every other derived value on that screen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
    pub possible: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedDatabase {
    pub instance: String,
    pub service: String,
    pub name: String,
    /// Whether it would be copied from [`Self::source`] rather than created
    /// empty.
    pub seed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Every hostname the workspace already answers on.
///
/// Read off the manifests rather than off the running containers: a project
/// that is stopped still owns its name, and a worktree given a hostname a
/// stopped project holds would take it over the moment both were started —
/// which Traefik reports as nothing at all.
fn claimed_domains(root: &std::path::Path) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let Some(projects) = crate::workspace::projects_root(root) else {
        return out;
    };
    let Ok(entries) = std::fs::read_dir(&projects) else {
        return out;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !path.join(crate::manifest::FILE).is_file() {
            continue;
        }
        let Ok(m) = crate::manifest::read(&path.join(crate::manifest::FILE), name) else {
            continue;
        };
        out.extend(m.domain.iter().map(|d| d.to_ascii_lowercase()));
        out.extend(m.aliases.iter().map(|a| a.to_ascii_lowercase()));
    }
    out
}

/// How a worktree was asked for, once the arguments have been read.
pub struct WorktreeRequest {
    branch: String,
    new_branch: bool,
    name: Option<String>,
    /// `none`, `create` or `copy`.
    database: String,
    instance: Option<String>,
    /// How long this environment is wanted for, in minutes. `None` is "until
    /// somebody says otherwise", which is what a person's own branch is.
    minutes: Option<u32>,
}

impl WorktreeRequest {
    /// Read the options object, defaulting every field.
    ///
    /// One loose `serde_json::Value` rather than six named arguments: the
    /// contract calls it `options` and the shape is a form's, so a field added
    /// next year is a default here rather than a signature change that every
    /// caller has to be edited for.
    pub fn read(branch: String, options: Option<serde_json::Value>) -> Self {
        let options = options.unwrap_or(serde_json::Value::Null);
        let string = |key: &str| {
            options
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        };

        Self {
            branch: branch.trim().to_string(),
            new_branch: options
                .get("newBranch")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            name: string("name"),
            database: string("database").unwrap_or_else(|| "none".to_string()),
            instance: string("instance"),
            // Zero is "no expiry" rather than "expired on arrival": a number
            // field that has been cleared arrives as 0, and an environment that
            // is over before it is built is one nobody could debug.
            minutes: options
                .get("minutes")
                .and_then(|v| v.as_u64())
                .and_then(|m| u32::try_from(m).ok())
                .filter(|m| *m > 0)
                .map(|m| m.min(MAX_TTL_MINUTES)),
        }
    }

    /// How long it was asked for, once clamped.
    pub fn minutes(&self) -> Option<u32> {
        self.minutes
    }
}

/// Work out what creating this worktree would do, and refuse it here if it
/// cannot be done.
pub async fn plan_worktree(
    root: &std::path::Path,
    parent: &str,
    request: &WorktreeRequest,
) -> Result<WorktreePlan> {
    let (dir, manifest) = crate::workspace::project_with_manifest(root, parent)?;

    let mut warnings: Vec<String> = Vec::new();
    let refuse = |plan: WorktreePlan, why: String| WorktreePlan {
        refused: Some(why),
        possible: false,
        ..plan
    };

    // Everything derivable before any refusal, so a refused plan still shows
    // what it *would* have been — a dialog that blanks out when it says no
    // makes the reason harder to act on, not easier.
    let slug = slug(&request.branch);
    let name = match (&request.name, &slug) {
        (Some(given), _) => crate::workspace::canonical_name(given),
        (None, Some(slug)) => project_name(parent, slug),
        (None, None) => String::new(),
    };
    let label = domain_label(parent, &name);
    let parent_domain = manifest.domain.clone().unwrap_or_default();
    let domain = domain(&parent_domain, &label);

    let mut plan = WorktreePlan {
        parent: parent.to_string(),
        branch: request.branch.clone(),
        new_branch: request.new_branch,
        name: name.clone(),
        path: crate::workspace::projects_root(root)
            .map(|p| p.join(&name).display().to_string())
            .unwrap_or_default(),
        domain: domain.clone(),
        database: None,
        // Worked out here, in the preview, so the screen shows the moment
        // rather than the duration: "in 120 minutes" is arithmetic somebody has
        // to do, and "at 14:32" is the answer they were doing it for.
        expires_at: request.minutes.map(expiry_in),
        warnings: Vec::new(),
        refused: None,
        possible: false,
    };

    // ---- the ground it stands on -----------------------------------------
    if !crate::git::available() {
        return Ok(refuse(plan, "git is not installed on this machine.".into()));
    }
    if !is_repository(&dir) {
        return Ok(refuse(plan, format!("{parent} is not a git repository.")));
    }
    if is_linked_worktree(&dir) {
        return Ok(refuse(
            plan,
            format!(
                "{parent} is itself a worktree; create the next one from the project it came from."
            ),
        ));
    }
    if manifest.domain.is_none() {
        return Ok(refuse(
            plan,
            format!("{parent} has no `domain` in its manifest to build a hostname under."),
        ));
    }

    // ---- the branch -------------------------------------------------------
    if request.branch.is_empty() {
        return Ok(refuse(plan, "No branch was named.".into()));
    }
    if !is_valid_branch_name(&dir, &request.branch) {
        return Ok(refuse(
            plan,
            format!(
                "git will not accept \"{}\" as a branch name.",
                request.branch
            ),
        ));
    }

    let checkouts = checkouts(&dir);
    let branch_exists = branches(&dir).contains(&request.branch);
    if request.new_branch && branch_exists {
        return Ok(refuse(
            plan,
            format!("a branch called \"{}\" already exists.", request.branch),
        ));
    }
    if !request.new_branch && !branch_exists {
        return Ok(refuse(
            plan,
            format!(
                "there is no branch called \"{}\"; tick \"create the branch\" to make one.",
                request.branch
            ),
        ));
    }
    if checkouts
        .iter()
        .any(|c| c.branch.as_deref() == Some(request.branch.as_str()))
    {
        return Ok(refuse(
            plan,
            format!(
                "\"{}\" is already checked out in another worktree; git allows a branch in one working tree at a time.",
                request.branch
            ),
        ));
    }

    // ---- the name and the hostname ---------------------------------------
    if name.is_empty() {
        return Ok(refuse(
            plan,
            format!(
                "\"{}\" has no letters or digits to build a name from; give the worktree a name of its own.",
                request.branch
            ),
        ));
    }
    if !crate::workspace::is_safe_name(&name) {
        return Ok(refuse(
            plan,
            format!("\"{name}\" is not a name a project directory can have."),
        ));
    }
    // Through the same gate every other creation path uses, so a name that
    // escapes the project tree is refused here rather than at `create_dir_all`.
    let path = crate::workspace::project_dir(root, &name)?;
    plan.path = path.display().to_string();
    if path.exists() {
        return Ok(refuse(plan, format!("projects/{name} already exists.")));
    }
    if !crate::hosts::is_valid_domain(&domain) {
        return Ok(refuse(
            plan,
            format!("\"{domain}\" is not a hostname; the branch produces a label a resolver would refuse."),
        ));
    }
    if claimed_domains(root).contains(&domain.to_ascii_lowercase()) {
        return Ok(refuse(
            plan,
            format!("another project already answers on {domain}."),
        ));
    }

    // Matched by the parent's own wildcard, which is not a conflict Traefik
    // reports: it has two routers for one name and answers with whichever it
    // ranks higher. Said out loud rather than refused — a wildcard alias is a
    // deliberate arrangement and this hostname is still the more specific rule.
    if manifest
        .aliases
        .iter()
        .any(|alias| alias.strip_prefix("*.") == Some(parent_domain.as_str()))
    {
        warnings.push(format!(
            "{parent} also answers on *.{parent_domain}, so {domain} matches two routes; the exact one wins, but the wildcard will not stop answering."
        ));
    }

    // ---- the database -----------------------------------------------------
    match request.database.as_str() {
        "none" => {}
        mode @ ("create" | "copy") => {
            let instances = crate::db::instances(root).await.unwrap_or_default();
            let chosen = match &request.instance {
                Some(id) => instances.iter().find(|i| &i.id == id),
                // The first database instance in the table, which is the order
                // `instances.json` keeps and therefore the order the Market
                // installed them in.
                None => instances.first(),
            };

            let Some(instance) = chosen else {
                return Ok(refuse(
                    plan,
                    match &request.instance {
                        Some(id) => format!("there is no database instance called \"{id}\"."),
                        None => "no database instance is installed to create one on.".into(),
                    },
                ));
            };
            if !instance.running {
                return Ok(refuse(
                    plan,
                    format!(
                        "{} is not running; a database cannot be created on a stopped engine.",
                        instance.id
                    ),
                ));
            }

            let connection = crate::db::connection(root, &instance.id)?;
            let stem = connection.database.clone().unwrap_or_else(|| parent.into());
            let database = database_name(&stem, &label);

            if !crate::db::is_valid_database_name(&database) {
                return Ok(refuse(
                    plan,
                    format!("\"{database}\" is not a database name this app will create."),
                ));
            }

            let seed = mode == "copy";
            if seed {
                if instance.kind == crate::db::Kind::Mongo {
                    return Ok(refuse(
                        plan,
                        "MongoDB publishes no database name for this workspace, so there is nothing to copy from.".into(),
                    ));
                }
                if connection.database.is_none() {
                    return Ok(refuse(
                        plan,
                        format!("{} has no database configured to copy from.", instance.id),
                    ));
                }
            }
            if instance.kind == crate::db::Kind::Mongo {
                warnings.push(format!(
                    "MongoDB has no CREATE DATABASE; {database} begins existing the first time the branch writes to it."
                ));
            }

            // Asked of the engine rather than assumed: a name left behind by a
            // worktree somebody removed by hand is the case this catches, and
            // creating "on top of" it would hand the branch somebody else's
            // data without saying so.
            if crate::db::databases(root, &instance.id)
                .await
                .unwrap_or_default()
                .iter()
                .any(|existing| existing == &database)
            {
                warnings.push(format!(
                    "{database} already exists on {}; it will be used as it is rather than created.",
                    instance.id
                ));
            }

            plan.database = Some(PlannedDatabase {
                instance: instance.id.clone(),
                service: instance.service.clone(),
                name: database,
                seed,
                source: seed.then(|| connection.database.clone()).flatten(),
            });
        }
        other => {
            return Ok(refuse(
                plan,
                format!("\"{other}\" is not a way to give a worktree a database."),
            ));
        }
    }

    plan.warnings = warnings;
    plan.possible = true;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A refused plan still says what it would have been.
    ///
    /// The comment above the derivation makes the decision out loud — "a dialog
    /// that blanks out when it says no makes the reason harder to act on, not
    /// easier" — and until this commit nothing held it: `plan_worktree` was
    /// private to `commands.rs`, one band above this module, so the refusal
    /// path had no test anywhere.
    ///
    /// A project directory that is not a git repository is the cheapest way in:
    /// no git command runs, and the refusal is reached with every derived field
    /// already filled.
    #[test]
    fn a_refused_plan_still_carries_the_name_and_domain_it_would_have_had() {
        let root = std::env::temp_dir().join(format!(
            "stackvo-worktree-plan-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let project = root.join("projects").join("shop");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join(crate::manifest::FILE),
            r#"{"name":"shop","domain":"shop.loc","runtime":"php"}"#,
        )
        .unwrap();
        crate::workspace::point_at_projects(&root, &root.join("projects")).unwrap();

        let request = WorktreeRequest::read("feature/Login Page".into(), None);
        let plan = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(plan_worktree(&root, "shop", &request))
            .expect("a refusal is an answer, not an error");

        assert!(!plan.possible);
        assert!(plan.refused.is_some(), "no reason was given");

        // The half the decision is about: everything derivable was derived
        // before the refusal, so the dialog still shows the name, the hostname
        // and the path the worktree would have had.
        assert_eq!(plan.name, "shop-feature-login-page");
        assert_eq!(plan.domain, "feature-login-page.shop.loc");
        assert!(
            plan.path.ends_with("shop-feature-login-page"),
            "{}",
            plan.path
        );
        assert_eq!(plan.branch, "feature/Login Page");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_branch_name_becomes_one_dns_label() {
        for (branch, expected) in [
            ("feature/login", "feature-login"),
            ("feature/ABC-123 Login", "feature-abc-123-login"),
            ("release/2.1.0", "release-2-1-0"),
            ("main", "main"),
            ("  spaced  ", "spaced"),
            ("under_score", "under-score"),
            ("dots...and---dashes", "dots-and-dashes"),
            ("-leading", "leading"),
            ("trailing-", "trailing"),
        ] {
            assert_eq!(slug(branch).as_deref(), Some(expected), "{branch}");
        }
    }

    /// Folded, not dropped. Dropping the punctuation makes two branches into
    /// one hostname, and Traefik answers a duplicate `Host()` rule by picking
    /// one and never telling anybody about the other.
    #[test]
    fn two_branches_never_collapse_into_one_slug() {
        assert_ne!(slug("feature/a-1"), slug("feature/a1"));
        assert_ne!(slug("fix/one"), slug("fix-one").map(|s| format!("{s}x")));
    }

    #[test]
    fn a_branch_with_no_usable_characters_has_no_slug() {
        for branch in ["", "   ", "...", "///", "---"] {
            assert_eq!(slug(branch), None, "{branch:?}");
        }
    }

    /// The label is capped, and a cap that landed on a hyphen would produce a
    /// hostname no resolver accepts.
    #[test]
    fn a_long_branch_is_cut_to_a_label_that_is_still_a_label() {
        let long = format!("feature/{}", "a-".repeat(60));
        let slug = slug(&long).expect("a slug");
        assert!(slug.len() <= SLUG_LIMIT, "{slug}");
        assert!(!slug.ends_with('-'), "{slug}");
        assert!(crate::hosts::is_valid_domain(&format!("{slug}.shop.loc")));
    }

    #[test]
    fn the_hostname_is_a_subdomain_of_the_parents() {
        let label = domain_label("shop", "shop-feature-x");
        assert_eq!(label, "feature-x");
        assert_eq!(domain("shop.loc", &label), "feature-x.shop.loc");

        // A name of the user's own keeps its whole self as the label.
        assert_eq!(domain_label("shop", "staging"), "staging");
        assert_eq!(domain_label("shop", "shop"), "shop");
    }

    #[test]
    fn a_database_name_is_one_a_statement_can_carry_unquoted() {
        let name = database_name("stackvo", "feature-x");
        assert_eq!(name, "stackvo_feature_x");
        assert!(crate::db::is_valid_database_name(&name), "{name}");
    }

    /// The stem is what gets cut, never the branch: two worktrees of one
    /// project share the stem and differ only in the branch part, so cutting
    /// the branch would give them one database between them.
    #[test]
    fn a_long_name_keeps_the_branch_and_loses_the_stem() {
        let stem = "a".repeat(80);
        let one = database_name(&stem, "feature-one");
        let two = database_name(&stem, "feature-two");

        assert!(one.len() <= 63, "{} chars", one.len());
        assert_ne!(one, two);
        assert!(one.ends_with("feature_one"), "{one}");
        assert!(crate::db::is_valid_database_name(&one), "{one}");
        assert!(crate::db::is_valid_database_name(&two), "{two}");
    }

    /// A database name must begin with a letter, and branch names beginning
    /// with a digit are ordinary — `2fa`, `3-column-layout`.
    #[test]
    fn a_name_that_would_start_with_a_digit_is_given_a_letter() {
        let name = database_name("", "2fa");
        assert!(crate::db::is_valid_database_name(&name), "{name}");
        assert!(name.starts_with(|c: char| c.is_ascii_lowercase()), "{name}");
    }

    /// A reserved name is refused rather than created — the derivation can
    /// reach one, because a branch may be called `sys`.
    #[test]
    fn the_engines_own_databases_are_never_a_derivation_target() {
        assert!(!crate::db::is_valid_database_name(&database_name(
            "", "sys"
        )));
        assert!(!crate::db::is_valid_database_name(&database_name(
            "", "mysql"
        )));
    }

    #[test]
    fn the_overlay_says_only_which_directory_and_which_hostname() {
        let text = local_overlay("shop-feature-x", "feature-x.shop.loc");
        let value: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

        assert_eq!(value["name"], "shop-feature-x");
        assert_eq!(value["domain"], "feature-x.shop.loc");
        assert_eq!(
            value.as_object().unwrap().len(),
            2,
            "the overlay grew a key: {text}"
        );
    }

    /// The overlay this writes has to be one `manifest` accepts, or creation
    /// produces a project the app then reports as broken.
    #[test]
    fn the_overlay_is_one_the_manifest_reader_accepts() {
        let dir =
            std::env::temp_dir().join(format!("stackvo-worktree-overlay-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // The committed manifest is the branch's, and names the parent.
        std::fs::write(
            dir.join(crate::manifest::FILE),
            "{\n  \"name\": \"shop\",\n  \"domain\": \"shop.loc\",\n  \"runtime\": \"php\",\n  \"php\": {\n    \"version\": \"8.4\"\n  }\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.join(crate::manifest::LOCAL_FILE),
            local_overlay("shop-feature-x", "feature-x.shop.loc"),
        )
        .unwrap();

        let m = crate::manifest::read(&dir.join(crate::manifest::FILE), "shop-feature-x").unwrap();
        assert!(m.valid, "{:?}", m.errors);
        assert_eq!(m.name, "shop-feature-x");
        assert_eq!(m.domain.as_deref(), Some("feature-x.shop.loc"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_argv_puts_the_path_behind_a_double_dash() {
        let args = add_args(Path::new("/tmp/shop-feature-x"), "feature/x", false);
        let dashes = args.iter().position(|a| a == "--").expect("no `--`");
        let path = args
            .iter()
            .position(|a| a == "/tmp/shop-feature-x")
            .expect("no path");
        assert!(dashes < path, "the path can still be read as an option");
        assert_eq!(args[..2], ["worktree", "add"]);
        assert_eq!(args.last().unwrap(), "feature/x");

        // Creating a branch names it before the path, which is where `-b` takes
        // its value; the path is still behind the `--`.
        let created = add_args(Path::new("/tmp/x"), "feature/new", true);
        assert_eq!(created[2], "-b");
        assert_eq!(created[3], "feature/new");
        assert!(
            created.iter().position(|a| a == "--").unwrap() < created.len() - 1,
            "{created:?}"
        );
    }

    #[test]
    fn the_porcelain_listing_is_read_as_records() {
        let sample = "worktree /code/shop\nHEAD abc\nbranch refs/heads/main\n\n\
                      worktree /code/shop-feature-x\nHEAD def\nbranch refs/heads/feature/x\n\n\
                      worktree /code/gone\nHEAD 123\ndetached\nprunable gitdir file points to non-existent location\n";

        // The parser is exercised through the same code path the command uses,
        // by feeding it the text git would have produced.
        let parsed = parse_porcelain(sample);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].branch.as_deref(), Some("main"));
        assert_eq!(parsed[1].path, "/code/shop-feature-x");
        assert_eq!(parsed[1].branch.as_deref(), Some("feature/x"));
        assert_eq!(parsed[2].branch, None, "a detached head has no branch");
        assert!(parsed[2].prunable);
    }

    /// A password with URL punctuation in it would otherwise end the userinfo
    /// early, and the application would connect to a host that does not exist —
    /// or, worse, to one that does.
    #[test]
    fn a_password_reaches_the_url_encoded() {
        let connection = crate::db::Connection {
            instance: "mysql-9-4".into(),
            service: "mysql".into(),
            kind: crate::db::Kind::Mysql,
            host: "stackvo-mysql-9-4".into(),
            port: 3306,
            user: "root".into(),
            password: Some("p@ss/word#1".into()),
            database: Some("stackvo".into()),
        };

        let url = database_url(&connection, "stackvo_feature_x");
        assert_eq!(
            url,
            "mysql://root:p%40ss%2Fword%231@stackvo-mysql-9-4:3306/stackvo_feature_x"
        );
        // The host and the database survive unencoded, which is what makes the
        // URL readable in a log.
        assert!(url.ends_with("/stackvo_feature_x"), "{url}");
    }

    /// `SESSION_DOMAIN` is written only where the project's own value would
    /// stop the cookie reaching the branch.
    #[test]
    fn a_pinned_cookie_domain_that_already_covers_the_branch_is_left_alone() {
        let host = "feature-x.shop.loc";

        // Laravel's own default is the current host. Nothing to say.
        assert_eq!(session_domain_for(None, host), None);
        assert_eq!(session_domain_for(Some("  "), host), None);
        // A parent domain covers everything under it, with or without the dot.
        assert_eq!(session_domain_for(Some(".shop.loc"), host), None);
        assert_eq!(session_domain_for(Some("shop.loc"), host), None);
        assert_eq!(session_domain_for(Some(host), host), None);
        // And one that does not is the case this exists for.
        assert_eq!(
            session_domain_for(Some("api.shop.loc"), host).as_deref(),
            Some(host)
        );
        // `shop.loc` is not covered by `xshop.loc` — the suffix test has to be
        // on a label boundary or every domain covers its own suffixes.
        assert_eq!(
            session_domain_for(Some("xshop.loc"), "shop.loc").as_deref(),
            Some("shop.loc")
        );
    }

    /// The stateful list is appended to, never replaced — and it is not written
    /// at all where the project leaves Sanctum's own default in place, because
    /// that default already follows `APP_URL`.
    #[test]
    fn the_stateful_list_gains_the_branch_and_loses_nothing() {
        let host = "feature-x.shop.loc";

        assert_eq!(stateful_domains_for(None, host), None);
        assert_eq!(stateful_domains_for(Some(""), host), None);
        assert_eq!(
            stateful_domains_for(Some("localhost,localhost:3000,shop.loc"), host).as_deref(),
            Some("localhost,localhost:3000,shop.loc,feature-x.shop.loc")
        );
        // Already there: nothing to write, in either spelling.
        assert_eq!(
            stateful_domains_for(Some("shop.loc,feature-x.shop.loc"), host),
            None
        );
        assert_eq!(
            stateful_domains_for(Some("shop.loc, FEATURE-X.SHOP.LOC"), host),
            None
        );
    }

    #[test]
    fn the_environment_names_the_branchs_own_hostname() {
        let record = Record {
            name: "shop-feature-x".into(),
            parent: "shop".into(),
            branch: "feature/x".into(),
            domain: "feature-x.shop.loc".into(),
            path: "/code/shop-feature-x".into(),
            database: None,
            env: BTreeMap::from([("APP_ENV".to_string(), "branch".to_string())]),
            created_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
        };

        let env = env_for(Path::new("/nonexistent-root"), &record);
        assert_eq!(
            env.get("APP_URL").map(String::as_str),
            Some("https://feature-x.shop.loc")
        );
        assert_eq!(
            env.get("STACKVO_WORKTREE_BRANCH").map(String::as_str),
            Some("feature/x")
        );
        assert_eq!(env.get("APP_ENV").map(String::as_str), Some("branch"));
        // No database means no credentials rather than empty ones.
        assert!(!env.contains_key("DB_DATABASE"), "{env:?}");
    }

    /// What somebody typed wins over what was derived, which is what makes the
    /// derived values a default rather than a decision.
    #[test]
    fn a_typed_variable_overrides_a_derived_one() {
        let record = Record {
            name: "shop-x".into(),
            parent: "shop".into(),
            branch: "x".into(),
            domain: "x.shop.loc".into(),
            path: "/code/shop-x".into(),
            database: None,
            env: BTreeMap::from([("APP_URL".to_string(), "http://localhost:8000".to_string())]),
            created_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
        };

        let env = env_for(Path::new("/nonexistent-root"), &record);
        assert_eq!(
            env.get("APP_URL").map(String::as_str),
            Some("http://localhost:8000")
        );
    }

    /// A value that would end the YAML scalar it is written into never reaches
    /// the overlay, however it got into the record.
    #[test]
    fn a_value_with_a_line_break_is_dropped_before_it_reaches_yaml() {
        let record = Record {
            name: "shop-x".into(),
            parent: "shop".into(),
            branch: "x".into(),
            domain: "x.shop.loc".into(),
            path: "/code/shop-x".into(),
            database: None,
            env: BTreeMap::from([
                ("GOOD".to_string(), "one".to_string()),
                ("BAD".to_string(), "two\n      INJECTED: yes".to_string()),
                ("not a key".to_string(), "three".to_string()),
            ]),
            created_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
        };

        let env = env_for(Path::new("/nonexistent-root"), &record);
        assert!(env.contains_key("GOOD"));
        assert!(!env.contains_key("BAD"), "{env:?}");
        assert!(!env.contains_key("not a key"), "{env:?}");
    }

    fn login_root(what: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "stackvo-worktree-login-{what}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn a_login_is_remembered_and_handed_back_when_it_is_forgotten() {
        let root = login_root("roundtrip");

        assert!(
            login_of(&root, "shop-x").is_none(),
            "none is the empty state"
        );

        remember_login(
            &root,
            "shop-x",
            Login {
                instance: "mysql-8-4".into(),
                user: "shop_x".into(),
                password: "deadbeef".into(),
            },
        )
        .unwrap();
        remember_login(
            &root,
            "shop-y",
            Login {
                instance: "mysql-8-4".into(),
                user: "shop_y".into(),
                password: "feedface".into(),
            },
        )
        .unwrap();

        assert_eq!(login_of(&root, "shop-x").unwrap().user, "shop_x");

        // Handed back, because the caller's next act is to drop the account it
        // names — a forget that returned nothing would leave the account behind
        // with no way to learn its name.
        let gone = forget_login(&root, "shop-x").expect("the login is returned");
        assert_eq!(gone.user, "shop_x");
        assert_eq!(gone.instance, "mysql-8-4");
        assert!(login_of(&root, "shop-x").is_none());
        // And only that one.
        assert_eq!(login_of(&root, "shop-y").unwrap().password, "feedface");
        assert!(
            forget_login(&root, "shop-x").is_none(),
            "twice is not an error"
        );
    }

    /// The reason the passwords are in a second file at all.
    ///
    /// `Table` is serialised twice by the same derive — once to disk and once
    /// across the IPC boundary into a webview. A password on `Record` would
    /// make the second one a leak, and the only defence would be remembering to
    /// strip it at every call site.
    #[test]
    fn the_record_a_webview_receives_carries_no_password() {
        let root = login_root("separation");

        let mut table = Table::default();
        table
            .insert(Record {
                name: "shop-x".into(),
                parent: "shop".into(),
                branch: "x".into(),
                domain: "x.shop.loc".into(),
                path: "/code/shop-x".into(),
                database: Some(Database {
                    instance: "mysql-8-4".into(),
                    name: "shop_x".into(),
                    seeded_from: None,
                }),
                env: BTreeMap::new(),
                created_at: "2026-01-01T00:00:00Z".into(),
                expires_at: None,
            })
            .unwrap();

        remember_login(
            &root,
            "shop-x",
            Login {
                instance: "mysql-8-4".into(),
                user: "shop_x".into(),
                password: "s3cr3t".into(),
            },
        )
        .unwrap();

        let shipped = serde_json::to_string(&table).unwrap();
        assert!(
            !shipped.contains("s3cr3t") && !shipped.contains("password"),
            "the worktree table carries a credential: {shipped}"
        );
        assert_ne!(
            logins_path(&root),
            path(&root),
            "the credentials must not share the file that crosses the boundary"
        );
        // And it really is on disk, in the other file.
        let stored = std::fs::read_to_string(logins_path(&root)).unwrap();
        assert!(stored.contains("s3cr3t"));
    }

    /// The file whose whole content is credentials is not world-readable.
    #[cfg(unix)]
    #[test]
    fn the_login_file_is_readable_only_by_its_owner() {
        use std::os::unix::fs::PermissionsExt;

        let root = login_root("mode");
        remember_login(
            &root,
            "shop-x",
            Login {
                instance: "mysql-8-4".into(),
                user: "shop_x".into(),
                password: "s3cr3t".into(),
            },
        )
        .unwrap();

        let mode = std::fs::metadata(logins_path(&root))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "the credentials file is {mode:o}");
    }

    fn dated(expires_at: Option<&str>) -> Record {
        Record {
            name: "shop-feature-x".into(),
            parent: "shop".into(),
            branch: "feature/x".into(),
            domain: "feature-x.shop.loc".into(),
            path: "/code/shop-feature-x".into(),
            database: None,
            env: BTreeMap::new(),
            created_at: "2026-08-30T09:00:00Z".into(),
            expires_at: expires_at.map(str::to_string),
        }
    }

    /// A person's own branch has no date on it, and never expires.
    #[test]
    fn a_worktree_with_no_expiry_is_never_finished_with() {
        let record = dated(None);
        assert!(!expired_at(&record, "2099-01-01T00:00:00Z"));
        assert_eq!(remaining_minutes_at(&record, "2026-08-30T09:00:00Z"), None);
    }

    /// The comparison is textual, which is only correct because the format is
    /// fixed-width UTC — so this is the test that would catch somebody
    /// "improving" the timestamp into a local one.
    #[test]
    fn an_expiry_passes_at_the_moment_it_names() {
        let record = dated(Some("2026-08-30T11:00:00Z"));

        assert!(!expired_at(&record, "2026-08-30T10:59:59Z"));
        assert!(expired_at(&record, "2026-08-30T11:00:00Z"), "on the second");
        assert!(expired_at(&record, "2026-08-30T11:00:01Z"));
        // Across a month and a year boundary, where a text comparison would
        // break if the fields were not zero-padded.
        assert!(expired_at(
            &dated(Some("2026-09-01T00:00:00Z")),
            "2026-09-01T00:00:00Z"
        ));
        assert!(!expired_at(
            &dated(Some("2027-01-01T00:00:00Z")),
            "2026-12-31T23:59:59Z"
        ));
    }

    #[test]
    fn what_is_left_is_rounded_down_and_never_negative() {
        let record = dated(Some("2026-08-30T11:00:00Z"));

        assert_eq!(
            remaining_minutes_at(&record, "2026-08-30T10:00:00Z"),
            Some(60)
        );
        // Fifty-nine seconds is not a minute of granted time.
        assert_eq!(
            remaining_minutes_at(&record, "2026-08-30T10:59:01Z"),
            Some(0)
        );
        // Past is zero, not a wrapped enormous number — this value becomes a
        // grant's `--for`.
        assert_eq!(
            remaining_minutes_at(&record, "2026-08-30T12:00:00Z"),
            Some(0)
        );
    }

    #[test]
    fn a_ttl_is_clamped_rather_than_taken_at_its_word() {
        let asked = WorktreeRequest::read(
            "feature/x".into(),
            Some(serde_json::json!({ "minutes": 60 * 24 * 365 })),
        );
        assert_eq!(asked.minutes(), Some(MAX_TTL_MINUTES));

        // Cleared fields arrive as zero, and zero is "no expiry" rather than
        // "already over".
        let none = WorktreeRequest::read(
            "feature/x".into(),
            Some(serde_json::json!({ "minutes": 0 })),
        );
        assert_eq!(none.minutes(), None);
    }

    /// The sentence K-1 asks for, as flags: *this assistant may work on this
    /// sandbox, for this long.*
    #[test]
    fn the_registration_for_a_sandbox_names_it_and_nothing_else() {
        let record = dated(Some("2026-08-30T11:00:00Z"));

        assert_eq!(
            grant_for(&record, Some(30)).to_args(),
            vec![
                "--allow-writes".to_string(),
                "--project=shop-feature-x".to_string(),
                "--for=30m".to_string(),
            ]
        );

        // And the scope is what makes it safe: under it the twelve writing
        // tools are the four a project bounds, so this registration cannot
        // stop the stack.
        let grant = grant_for(&record, Some(30));
        let stack_down = crate::mcp::TOOLS
            .iter()
            .find(|t| t.name == "stackvo_stack_down")
            .expect("the tool exists");
        assert!(!grant.opens(stack_down));
    }

    #[test]
    fn the_table_refuses_a_name_it_already_holds() {
        let record = |name: &str| Record {
            name: name.into(),
            parent: "shop".into(),
            branch: "x".into(),
            domain: format!("{name}.shop.loc"),
            path: format!("/code/{name}"),
            database: None,
            env: BTreeMap::new(),
            created_at: "2026-01-01T00:00:00Z".into(),
            expires_at: None,
        };

        let mut table = Table::default();
        table.insert(record("shop-a")).unwrap();
        assert!(table.insert(record("shop-a")).is_err());
        table.insert(record("shop-b")).unwrap();

        assert_eq!(table.of_parent("shop").count(), 2);
        assert_eq!(table.remove("shop-a").unwrap().name, "shop-a");
        assert!(table.get("shop-a").is_none());
    }

    #[test]
    fn a_table_from_a_newer_version_is_refused_rather_than_half_read() {
        let root = std::env::temp_dir().join(format!(
            "stackvo-worktree-table-{}-{}",
            std::process::id(),
            SCHEMA_VERSION
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        // Absent is not an error.
        assert!(Table::load(&root).unwrap().worktrees.is_empty());

        std::fs::write(
            path(&root),
            format!(
                "{{\"schemaVersion\": {}, \"worktrees\": []}}",
                SCHEMA_VERSION + 1
            ),
        )
        .unwrap();
        assert!(Table::load(&root).is_err());

        let _ = std::fs::remove_dir_all(&root);
    }
}
