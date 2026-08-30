//! Fetching this project's data from where it really runs, and sending it back.
//!
//! DDEV ships `ddev pull` and `ddev push` with recipes for Upsun, Acquia,
//! Lagoon and Pantheon; Lando and Herd have their own. It is the largest gap
//! the competitor review found and the only one that is a whole category rather
//! than a feature.
//!
//! ## Everything dangerous about this was already answered next door
//!
//! A provider is a **command a repository declares** that reaches the network
//! **with the developer's credentials**. That is [`crate::hooks`]'s threat
//! model word for word — "somebody clones a repository, opens it here, presses
//! a button, and a list of commands written by whoever wrote that repository
//! runs" — so this module borrows its answers rather than inventing worse ones:
//!
//! * **A step is an argv array. There is no shell.** No `sh -c`, no pipeline,
//!   no interpolation. What a recipe cannot express in an argv it does not get
//!   to express.
//! * **It runs in a container**, never on the machine. `hooks::Kind::Host`
//!   exists because some hooks genuinely have to touch the developer's files;
//!   a provider never does. There is no host variant here and there will not
//!   be one.
//! * **Consent is per project and per command list**, keyed on a digest, so
//!   editing the recipe asks again. Its own file, not `hooks`': agreeing to run
//!   `composer install` in a container is not agreeing to send a database to
//!   production.
//! * **An administrator can forbid it and cannot approve it.** Same asymmetry
//!   as `policy::Hooks`: a file pushed to three hundred laptops has not read a
//!   list of commands.
//!
//! ## The credentials are the asset, and they are not in the repository
//!
//! DDEV mounts the developer's ssh agent into the container that runs the pull.
//! That is a coherent choice for a tool whose recipes are curated, and it is
//! the wrong one here: this application's rule is that a repository-declared
//! container gets no host path, and an ssh agent is a host path that
//! signs things.
//!
//! So a recipe **names** what it needs and never carries it. The values come
//! out of the keystore and arrive as environment variables in the
//! container, for the length of one run. A recipe that wants a key it was not
//! given is refused before anything is spawned, by name, rather than failing
//! inside a container with somebody else's error message.
//!
//! ## The dump is a file at a fixed path, because a pipeline needs a shell
//!
//! `ddev pull` is written as shell: `ssh host mysqldump | gzip`. With no shell
//! there is no pipe, so the contract is a **path** instead: a pull writes
//! `/stackvo/dump.sql`, a push reads it. Fixed and not configurable — a recipe
//! choosing where its output goes is one more thing to get wrong and one more
//! thing pointing somewhere surprising.
//!
//! The directory is a scratch directory this application owns for the length of
//! one run, mounted at `/stackvo`. What comes back out is checked with
//! [`std::fs::symlink_metadata`] and refused unless it is a regular file: a
//! container that can write into a mounted directory can write a **symlink**
//! there, and `dump.sql -> /etc/passwd` would have this application read the
//! host's file and feed it to a database.
//!
//! ## Pull lands in the restore that already has a net
//!
//! A pull does not import anything itself. It produces a file and hands it to
//! `db_restore`, which since the safety net went in takes a copy of what it is
//! about to replace. Pulling staging over the wrong database is recoverable for
//! the same reason restoring the wrong file is.
//!
//! ## Push is the same shape and not the same act
//!
//! DDEV's own documentation warns about `push` in the loudest terms it uses
//! anywhere, and it is right to. The code here is nearly symmetric — the same
//! container, the same secrets, the same argv rule, the file travelling the
//! other way — and everything that is *not* symmetric is deliberate:
//!
//! * a recipe has to declare `push` explicitly. Declaring `pull` grants
//!   nothing;
//! * consent is granted per direction. Agreeing to fetch is not agreeing to
//!   send, and the digest covers the direction so it cannot be reused;
//! * it is audited. `pull` is not — it changes this machine, and the app is
//!   full of things that do.
//!
//! What is deliberately absent is a scheduler. Nothing in this application may
//! push on a timer.

use crate::error::{Code, Error, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Where the scratch directory is mounted inside the container.
pub const MOUNT: &str = "/stackvo";

/// The one file a recipe reads or writes, at [`MOUNT`].
pub const DUMP: &str = "dump.sql";

/// Which way the data goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Pull,
    Push,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Pull => "pull",
            Direction::Push => "push",
        }
    }
}

/// One named place this project's data lives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub name: String,
    /// What a person is meant to recognise: "the staging site", "production".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    pub image: String,
    /// Empty means this provider does not offer that direction at all.
    pub pull: Vec<String>,
    pub push: Vec<String>,
    /// Plain values the recipe wants in the container's environment.
    pub env: BTreeMap<String, String>,
    /// Names of values that come out of the keystore, never from the file.
    pub secrets: Vec<String>,
}

impl Provider {
    pub fn command(&self, direction: Direction) -> &[String] {
        match direction {
            Direction::Pull => &self.pull,
            Direction::Push => &self.push,
        }
    }

    pub fn offers(&self, direction: Direction) -> bool {
        !self.command(direction).is_empty()
    }
}

/// Serialise providers the way the *file* spells them: an object keyed by name,
/// in the order they were declared.
///
/// The struct is a list because a recipe carries its own name and every
/// consumer iterates it. The file is a map, and this is the single place the
/// two shapes meet — the webview round-trips a serialised manifest straight
/// back into `project_manifest_write`, so a `providers` that serialised as a
/// list would parse back as nothing and vanish on the next form save. That is
/// the bug this field exists to close, and it would have been reintroduced one
/// layer further down.
pub fn as_declared_map<S>(
    providers: &[Provider],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;

    /// A provider minus its name, which is the key it is written under. Written
    /// out rather than reusing [`Provider`] so the serialised shape is the
    /// file's shape exactly, with no key the reader has to know to ignore.
    #[derive(Serialize)]
    struct Body<'a> {
        image: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        about: &'a Option<String>,
        #[serde(skip_serializing_if = "<[String]>::is_empty")]
        pull: &'a [String],
        #[serde(skip_serializing_if = "<[String]>::is_empty")]
        push: &'a [String],
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        env: &'a BTreeMap<String, String>,
        #[serde(skip_serializing_if = "<[String]>::is_empty")]
        secrets: &'a [String],
    }

    let mut map = serializer.serialize_map(Some(providers.len()))?;
    for provider in providers {
        map.serialize_entry(
            &provider.name,
            &Body {
                image: &provider.image,
                about: &provider.about,
                pull: &provider.pull,
                push: &provider.push,
                env: &provider.env,
                secrets: &provider.secrets,
            },
        )?;
    }
    map.end()
}

/// Why a recipe was not read, in the reader's terms.
///
/// The same shape `hooks::Problem` has: a manifest is a file somebody wrote by
/// hand, and "invalid" with nothing pointing at a line is a message that gets
/// worked around rather than fixed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub provider: String,
    pub message: String,
}

/// `[a-z0-9-]`, 1..=32, not starting or ending with a dash.
fn is_name(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 32
        && !text.starts_with('-')
        && !text.ends_with('-')
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The shape an environment variable name has, everywhere.
fn is_env_name(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 64
        && text
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
        && text
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Read the `providers` block, and say what was wrong with the rest.
///
/// Never fails: a malformed recipe is dropped and named. The alternative is a
/// project that will not open because one line of one optional block has a typo
/// in it, which is how a feature becomes a thing people delete.
pub fn parse(json: &serde_json::Value) -> (Vec<Provider>, Vec<Problem>) {
    let mut out = Vec::new();
    let mut problems = Vec::new();

    let Some(map) = json.get("providers").and_then(|v| v.as_object()) else {
        return (out, problems);
    };

    for (name, body) in map {
        let mut refuse = |message: &str| {
            problems.push(Problem {
                provider: name.clone(),
                message: message.to_string(),
            });
        };

        if !is_name(name) {
            refuse("a provider name is 1–32 characters of a–z, 0–9 and dashes");
            continue;
        }
        let Some(body) = body.as_object() else {
            refuse("a provider is an object");
            continue;
        };

        let image = body.get("image").and_then(|v| v.as_str()).unwrap_or("");
        if image.trim().is_empty() {
            refuse("no image: a provider names the container its command runs in");
            continue;
        }
        // The characters a reference has. Refused here rather than at `docker
        // run`, where a value with a space in it becomes a second argument.
        if image.contains(char::is_whitespace) {
            refuse("an image reference has no spaces in it");
            continue;
        }

        let argv = |key: &str| -> std::result::Result<Vec<String>, String> {
            match body.get(key) {
                None => Ok(Vec::new()),
                Some(serde_json::Value::Array(items)) => {
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        let Some(text) = item.as_str() else {
                            return Err(format!("every word of `{key}` is a string"));
                        };
                        out.push(text.to_string());
                    }
                    if out.first().is_some_and(|first| first.trim().is_empty()) {
                        return Err(format!("`{key}` starts with an empty word"));
                    }
                    Ok(out)
                }
                // The mistake this catches is the one everybody makes first,
                // because every other tool in this space takes a shell string.
                Some(serde_json::Value::String(_)) => Err(format!(
                    "`{key}` is a list of words, not a command line — there is no shell here, \
                     so `[\"pg_dump\", \"-Fc\"]` rather than `\"pg_dump -Fc\"`"
                )),
                Some(_) => Err(format!("`{key}` is a list of words")),
            }
        };

        let pull = match argv("pull") {
            Ok(argv) => argv,
            Err(message) => {
                refuse(&message);
                continue;
            }
        };
        let push = match argv("push") {
            Ok(argv) => argv,
            Err(message) => {
                refuse(&message);
                continue;
            }
        };
        if pull.is_empty() && push.is_empty() {
            refuse("neither `pull` nor `push`: this provider does nothing");
            continue;
        }

        let mut secrets: Vec<String> = Vec::new();
        if let Some(items) = body.get("secrets") {
            let Some(items) = items.as_array() else {
                refuse("`secrets` is a list of variable names");
                continue;
            };
            let mut bad = false;
            for item in items {
                match item.as_str() {
                    Some(text) if is_env_name(text) => secrets.push(text.to_string()),
                    Some(text) => {
                        refuse(&format!("`{text}` is not an environment variable name"));
                        bad = true;
                        break;
                    }
                    None => {
                        refuse("`secrets` is a list of variable names");
                        bad = true;
                        break;
                    }
                }
            }
            if bad {
                continue;
            }
        }

        let mut env: BTreeMap<String, String> = BTreeMap::new();
        if let Some(map) = body.get("env") {
            let Some(map) = map.as_object() else {
                refuse("`env` is an object of NAME: value");
                continue;
            };
            let mut bad = false;
            for (key, value) in map {
                if !is_env_name(key) {
                    refuse(&format!("`{key}` is not an environment variable name"));
                    bad = true;
                    break;
                }
                let Some(text) = value.as_str() else {
                    refuse(&format!("`{key}` is not a string"));
                    bad = true;
                    break;
                };
                // A keystore reference in `env` would put a secret's *name* in
                // the plain half and read as though it resolved. `secrets` is
                // the half that resolves; this one is copied through.
                if crate::secrets::is_reference(text) {
                    refuse(&format!(
                        "`{key}` looks like a keystore reference — name it under `secrets` instead"
                    ));
                    bad = true;
                    break;
                }
                if secrets.contains(key) {
                    refuse(&format!(
                        "`{key}` is in both `env` and `secrets`, and only one of them can win"
                    ));
                    bad = true;
                    break;
                }
                env.insert(key.clone(), text.to_string());
            }
            if bad {
                continue;
            }
        }

        out.push(Provider {
            name: name.clone(),
            about: body
                .get("about")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            image: image.to_string(),
            pull,
            push,
            env,
            secrets,
        });
    }

    // In the order the file declared them, and deliberately not sorted.
    //
    // Alphabetising here was invisible while this block was read-only. It stops
    // being invisible the moment the manifest is written back from it: a saved
    // file returns with somebody's recipes re-ordered, which is the outcome
    // `sidecar::Declared` and `quickcmd::Declared` each keep an explicit order
    // list to avoid. `serde_json` is built with `preserve_order`, so the map
    // above already iterates the way the file reads.
    (out, problems)
}

// -------------------------------------------------------------------- consent

/// What has been agreed to, per project, per direction.
///
/// Its own file rather than `hooks`', and its own entry per direction. Agreeing
/// to fetch staging's database is not agreeing to send this one to production,
/// and a single record would have made the second free once the first was
/// given.
#[derive(Debug, Clone, Default)]
pub struct Consent {
    granted: BTreeMap<String, String>,
}

fn consent_key(project: &str, name: &str, direction: Direction) -> String {
    format!("{project}/{name}/{}", direction.as_str())
}

impl Consent {
    pub fn allows(&self, project: &str, name: &str, direction: Direction, digest: &str) -> bool {
        self.granted
            .get(&consent_key(project, name, direction))
            .map(String::as_str)
            == Some(digest)
    }

    pub fn grant(&mut self, project: &str, name: &str, direction: Direction, digest: &str) {
        self.granted
            .insert(consent_key(project, name, direction), digest.to_string());
    }

    pub fn revoke(&mut self, project: &str, name: &str, direction: Direction) {
        self.granted.remove(&consent_key(project, name, direction));
    }
}

/// What consent is granted against.
///
/// Every byte that decides what runs: the image, the words of the command, the
/// plain environment and the names of the secrets — and the direction, so a
/// `pull` agreement cannot be spent on a `push`. Editing any of them asks
/// again, which is the whole value of a digest over a boolean.
pub fn digest(provider: &Provider, direction: Direction) -> String {
    let mut text = String::new();
    text.push_str(direction.as_str());
    text.push('\0');
    text.push_str(&provider.image);
    for word in provider.command(direction) {
        text.push('\0');
        text.push_str(word);
    }
    for (key, value) in &provider.env {
        text.push('\0');
        text.push_str(key);
        text.push('=');
        text.push_str(value);
    }
    for name in &provider.secrets {
        text.push('\0');
        text.push_str(name);
    }
    crate::pkg::sha256_hex(text.as_bytes())
}

pub fn consent_path() -> Option<PathBuf> {
    crate::appdir::config().map(|dir| dir.join("provider-consent.json"))
}

pub fn read_consent(path: &Path) -> Consent {
    // No file, an unreadable one and a malformed one all mean "nothing has been
    // agreed to". The failure mode of this parse must be "asks again", never
    // "assumes yes" — `hooks::read_consent`'s reasoning, and the same danger.
    let Ok(text) = std::fs::read_to_string(path) else {
        return Consent::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Consent::default();
    };
    let Some(map) = value.get("granted").and_then(|v| v.as_object()) else {
        return Consent::default();
    };

    let mut consent = Consent::default();
    for (key, digest) in map {
        if let Some(text) = digest.as_str() {
            consent.granted.insert(key.clone(), text.to_string());
        }
    }
    consent
}

pub fn write_consent(path: &Path, consent: &Consent) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io("creating the application directory", e))?;
    }
    let body = serde_json::json!({
        "schemaVersion": 1,
        "granted": consent.granted,
    });
    crate::atomic::write(
        path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&body).map_err(|e| Error::new(
                Code::IoError,
                format!("serialising the provider consent: {e}")
            ))?
        ),
    )
}

/// A project's recipes, planned, with whatever could not be read.
///
/// One shape for the whole card. The problems travel beside the plans rather
/// than replacing them: a project with three good recipes and one typo should
/// show three buttons and one sentence, not an error page.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Providers {
    pub recipes: Vec<Provider>,
    /// Both directions of every recipe, in order.
    pub plans: Vec<Plan>,
    pub problems: Vec<Problem>,
}

// ----------------------------------------------------------------------- plan

/// Why a run will not happen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Blocked {
    /// An administrator turned providers off.
    PolicyOff,
    /// This provider does not offer this direction.
    NotOffered,
    /// Not agreed to on this machine, or the recipe changed since it was.
    NeedsConsent,
    /// The keystore has no value for a secret the recipe names.
    #[serde(rename_all = "camelCase")]
    MissingSecrets { names: Vec<String> },
}

/// What one run would do, before anything is spawned.
///
/// The review-then-apply shape this application uses for every act it cannot
/// take back — `hosts_plan`, `preset::plan`, `handover::plan`, `hooks::plan`.
/// It matters more here than in most of them: the reader is about to hand
/// somebody else's command their production credentials, and "it ran and
/// something happened" is not a thing anybody can review afterwards.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub provider: String,
    pub direction: Direction,
    /// The image and the words, as they would be run.
    pub image: String,
    pub command: Vec<String>,
    /// The plain environment, shown in full. There is nothing secret in it by
    /// construction — a keystore reference here is refused at parse.
    pub env: BTreeMap<String, String>,
    /// The secrets this would resolve, **by name**. Never a value: this
    /// travels to a web view.
    pub secrets: Vec<String>,
    /// The digest a consent screen would grant.
    pub digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<Blocked>,
}

impl Plan {
    pub fn runnable(&self) -> bool {
        self.blocked.is_none()
    }
}

/// Decide what one run would do.
///
/// `have` answers "does the keystore hold this name" and is passed in rather
/// than read here, for the reason `render::dynamic_compose` takes its own: the
/// caller owns the keystore, and this stays testable without one.
pub fn plan(
    project: &str,
    provider: &Provider,
    direction: Direction,
    policy_allows: bool,
    consent: &Consent,
    have: &dyn Fn(&str) -> bool,
) -> Plan {
    let digest = digest(provider, direction);
    let missing: Vec<String> = provider
        .secrets
        .iter()
        .filter(|name| !have(name))
        .cloned()
        .collect();

    // Ordered cheapest-refusal-first, and the order is what the reader sees:
    // being told to fill in three secrets for a direction an administrator has
    // switched off is three pieces of work for nothing.
    let blocked = if !policy_allows {
        Some(Blocked::PolicyOff)
    } else if !provider.offers(direction) {
        Some(Blocked::NotOffered)
    } else if !consent.allows(project, &provider.name, direction, &digest) {
        Some(Blocked::NeedsConsent)
    } else if !missing.is_empty() {
        Some(Blocked::MissingSecrets { names: missing })
    } else {
        None
    };

    Plan {
        provider: provider.name.clone(),
        direction,
        image: provider.image.clone(),
        command: provider.command(direction).to_vec(),
        env: provider.env.clone(),
        secrets: provider.secrets.clone(),
        digest,
        blocked,
    }
}

// ------------------------------------------------------------------- the run

/// `docker run` for one provider, as an argv.
///
/// Built here and tested here, because every dangerous property of this feature
/// is a property of this array:
///
/// * `--rm`, so nothing survives the run;
/// * `--network none` for a **push**… no. See below.
/// * the scratch directory is the only mount, and it is this application's;
/// * secrets arrive as `-e NAME` with no value, which tells Docker to copy the
///   variable from *this process's* environment. The value never appears in an
///   argument, so it is not in `ps`, not in a shell history and not in a crash
///   report.
///
/// The environment for the child is the caller's business; this returns the
/// argv and the names it expects to find set.
pub fn run_args(provider: &Provider, direction: Direction, scratch: &Path) -> Vec<String> {
    let mut args = vec![
        "run".to_string(),
        "--rm".to_string(),
        // No TTY and no stdin. A recipe that stops to ask something would hang
        // an operation console with no way to answer it — the same reason
        // `scaffold` pins every installer non-interactive.
        "-i=false".to_string(),
        "-v".to_string(),
        format!(
            "{}:{MOUNT}",
            crate::paths::to_docker_mount(&scratch.display().to_string())
        ),
        "-w".to_string(),
        MOUNT.to_string(),
    ];

    for (key, value) in &provider.env {
        args.push("-e".to_string());
        args.push(format!("{key}={value}"));
    }
    // Name only. `docker run -e NAME` copies it from this process, so the value
    // is never an argument of anything.
    for name in &provider.secrets {
        args.push("-e".to_string());
        args.push(name.clone());
    }

    args.push(provider.image.clone());
    args.extend(provider.command(direction).iter().cloned());
    args
}

/// The file a pull must have produced, checked before it is believed.
///
/// A container that can write into a mounted directory can write a **symlink**
/// there. `dump.sql -> /etc/passwd` would have this application read the host's
/// own file and hand it to a database — an arbitrary host read, out of a
/// feature whose whole job is to move a file. `symlink_metadata` does not
/// follow, so the check is about the entry rather than about its target.
pub fn produced(scratch: &Path) -> Result<PathBuf> {
    let path = scratch.join(DUMP);
    let meta = std::fs::symlink_metadata(&path).map_err(|_| {
        Error::new(
            Code::NotFound,
            format!("the provider wrote no {MOUNT}/{DUMP}"),
        )
        .with_hint(crate::hints::PROVIDER_WROTE_NOTHING)
    })?;

    if meta.file_type().is_symlink() {
        return Err(Error::new(
            Code::Forbidden,
            format!("{MOUNT}/{DUMP} is a symbolic link, and this reads regular files only"),
        ));
    }
    if !meta.is_file() {
        return Err(Error::new(
            Code::InvalidInput,
            format!("{MOUNT}/{DUMP} is not a file"),
        ));
    }
    if meta.len() == 0 {
        // An empty dump restored over a database is the same act as dropping
        // it, and it is the shape a failed remote command leaves behind.
        return Err(Error::new(
            Code::InvalidInput,
            format!("{MOUNT}/{DUMP} is empty; nothing was fetched"),
        )
        .with_hint(crate::hints::PROVIDER_WROTE_NOTHING));
    }
    Ok(path)
}

/// Names in the recipe that the keystore has nothing for.
pub fn missing_secrets(provider: &Provider, have: &dyn Fn(&str) -> bool) -> Vec<String> {
    provider
        .secrets
        .iter()
        .filter(|name| !have(name))
        .cloned()
        .collect()
}

/// The keystore entry one provider secret lives under.
///
/// Per project and per provider, not per name: two projects both wanting
/// `SSH_KEY` are two different keys, and a shared entry would hand one team's
/// credential to another project's recipe.
pub fn secret_entry(project: &str, provider: &str, name: &str) -> String {
    format!("stackvo/provider/{project}/{provider}/{name}")
}

/// Every secret name across a project's recipes, deduplicated.
pub fn all_secrets(providers: &[Provider]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for provider in providers {
        for name in &provider.secrets {
            seen.insert(name.clone());
        }
    }
    seen.into_iter().collect()
}

// ------------------------------------------------------------------ recipes

/// A starter recipe, shipped in the binary.
///
/// ## Why the catalogue was empty, and why that was the whole feature missing
///
/// The mechanism above is stricter than DDEV's and was finished before this
/// existed: argv only, container only, consent per direction, secrets from the
/// keystore. What it did not have was a single example. DDEV ships
/// `.ddev/providers` with Upsun, Acquia and Lagoon already in every project;
/// here, using the feature meant first learning a file format from source. A
/// mechanism nobody can start using is not a shipped mechanism.
///
/// These are **starting points, not integrations**. Adding one writes it into
/// the project's own `stackvo.json`, where it is a file the user owns and edits
/// — every host name, database name and project id below is a placeholder, and
/// the recipe will not work until somebody replaces them. That is deliberate:
/// a recipe that guessed would be a recipe that reached the wrong server with
/// real credentials.
///
/// Nothing about consent is shortened by a recipe being shipped. It is written
/// into the manifest and then approved through the same digest as one somebody
/// typed, because the two are the same thing the moment the file is saved —
/// and because a recipe the user edited after adding it must ask again.
///
/// ## What could not be shipped, measured rather than assumed
///
/// **SSH plus `mysqldump`, the one that depends on no provider at all, cannot
/// be expressed today.** Not because of the missing shell — `mysqldump` has
/// `--result-file` and needs no pipe — but because of the rule this module
/// states in its own header: a repository-declared container gets no host path,
/// so no agent socket and no key file. `ssh` authenticates with a *file* or an
/// *agent*, and a secret here arrives as an environment variable. There is no
/// argv that turns one into the other. Closing that would mean letting a recipe
/// ask for a secret to be materialised as a file in the scratch directory,
/// which is a decision about this module's threat model rather than a recipe,
/// and it is not made here.
///
/// **Pantheon cannot be expressed either, for two independent reasons.** There
/// is no Terminus image on any registry this checked — DDEV installs the CLI
/// into its own web container rather than running one — and `terminus
/// backup:get` answers with a URL that then has to be downloaded, which is a
/// second command and therefore a shell.
///
/// The Upsun CLI has an image and was run to check it: `ghcr.io/upsun/cli`
/// takes `db:dump --directory --file`, reads `UPSUN_CLI_TOKEN` from the
/// environment, and its entrypoint is the CLI itself so the argv is just the
/// subcommand. It offers no `push`, and that is the CLI's shape rather than a
/// choice here — `db:sql` takes a query, not a file.
///
/// ## The tag is `latest`, and the digest does not cover the bytes
///
/// `ghcr.io/upsun/cli` publishes no version tags. A consent digest covers the
/// image *reference*, so an image that moves under a fixed tag is approved once
/// and can change afterwards. That is true of any recipe naming a moving tag,
/// including one somebody writes; it is said here because this is the one
/// shipped recipe where there was no pinnable alternative to choose.
pub struct Recipe {
    /// The key it is written under, and the name consent is keyed on.
    pub name: &'static str,
    /// One line, for a list.
    pub about: &'static str,
    /// What has to be changed before it will work, in the reader's terms.
    ///
    /// Required rather than optional: every recipe here carries a placeholder,
    /// and a starting point that does not say which words are placeholders is
    /// a command somebody will approve as it stands.
    ///
    /// A [`Edit`] rather than a bare string because the window is bilingual and
    /// this is the sentence that says *what to change* — the same class
    /// `hints.rs` calls the worst one to leave untranslated. The English rides
    /// along for the CLI, the MCP surface and the log; the key is what the
    /// window looks up.
    pub edit: &'static [Edit],
    /// The body, exactly as it sits under `providers.<name>` in the manifest.
    ///
    /// Held as the JSON text rather than as a [`Provider`] so the shipped
    /// recipes go through [`parse`] — the same reader the manifest uses — and a
    /// recipe this repository ships cannot be one the parser refuses.
    pub body: &'static str,
}

/// One thing a person has to change, and the key a locale translates it under.
///
/// The shape `hints.rs` settled on, for the same reason: a bare key would leave
/// the English in this file and the key in another place to typo, and a bare
/// string leaves a Turkish reader an English instruction. Both halves, declared
/// once, in the row they belong to.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Edit {
    /// Looked up as `providerRecipeEdits.<key>` in the locale files.
    pub key: &'static str,
    /// The fallback, and what the log and the MCP surface get.
    pub english: &'static str,
}

pub const RECIPES: [Recipe; 3] = [
    Recipe {
        name: "mysql-remote",
        about: "A MySQL or MariaDB server this machine can reach directly",
        edit: &[
            Edit {
                key: "connection",
                english: "the host, port, user and database name in both commands",
            },
            Edit {
                key: "mysqlPassword",
                english: "MYSQL_PWD, which the client reads from the environment so the \
                          password is never an argument",
            },
        ],
        // `--result-file` rather than a redirect, and `--execute=source …`
        // rather than `< dump.sql`: both are the client's own answer to the
        // absence of a shell, which is why this pair is expressible and the ssh
        // one is not.
        body: r#"{
          "image": "mysql:8.4",
          "about": "The database this project really runs against",
          "pull": [
            "mysqldump",
            "--host=db.example.com",
            "--port=3306",
            "--user=stackvo",
            "--single-transaction",
            "--quick",
            "--no-tablespaces",
            "--result-file=/stackvo/dump.sql",
            "the_database"
          ],
          "push": [
            "mysql",
            "--host=db.example.com",
            "--port=3306",
            "--user=stackvo",
            "--execute=source /stackvo/dump.sql",
            "the_database"
          ],
          "secrets": ["MYSQL_PWD"]
        }"#,
    },
    Recipe {
        name: "postgres-remote",
        about: "A PostgreSQL server this machine can reach directly",
        edit: &[
            Edit {
                key: "connection",
                english: "the host, port, user and database name in both commands",
            },
            Edit {
                key: "postgresPassword",
                english: "PGPASSWORD, which libpq reads from the environment",
            },
        ],
        // `--no-owner --no-privileges` because the roles on the far side are
        // not the roles here, and a restore that fails on a missing role is a
        // pull that appears to have worked. `ON_ERROR_STOP` for the reverse
        // reason: a push must not half-apply.
        body: r#"{
          "image": "postgres:17",
          "about": "The database this project really runs against",
          "pull": [
            "pg_dump",
            "--host=db.example.com",
            "--port=5432",
            "--username=stackvo",
            "--no-owner",
            "--no-privileges",
            "--file=/stackvo/dump.sql",
            "the_database"
          ],
          "push": [
            "psql",
            "--host=db.example.com",
            "--port=5432",
            "--username=stackvo",
            "--dbname=the_database",
            "--set=ON_ERROR_STOP=1",
            "--file=/stackvo/dump.sql"
          ],
          "secrets": ["PGPASSWORD"]
        }"#,
    },
    Recipe {
        name: "upsun",
        about: "An Upsun (Platform.sh) environment, through its own CLI",
        edit: &[
            Edit {
                key: "upsunProject",
                english: "the project id and the environment name",
            },
            Edit {
                key: "upsunToken",
                english: "UPSUN_CLI_TOKEN, an API token from the Upsun console",
            },
        ],
        // No `push`. `upsun db:sql` takes a query rather than a file, so there
        // is no argv that sends this database to that one — and an empty `push`
        // is how a recipe says a direction is not offered.
        body: r#"{
          "image": "ghcr.io/upsun/cli:latest",
          "about": "The Upsun environment this project deploys to",
          "pull": [
            "db:dump",
            "--project=YOUR-PROJECT-ID",
            "--environment=main",
            "--directory=/stackvo",
            "--file=dump.sql",
            "--yes"
          ],
          "secrets": ["UPSUN_CLI_TOKEN"]
        }"#,
    },
];

pub fn recipe(name: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.name == name)
}

/// A shipped recipe, read through the same parser the manifest uses.
///
/// Returns `None` only if the shipped text is malformed, which
/// [`tests::every_shipped_recipe_survives_the_parser_that_reads_the_manifest`]
/// makes a build failure rather than a runtime one.
pub fn as_provider(recipe: &Recipe) -> Option<Provider> {
    let body: serde_json::Value = serde_json::from_str(recipe.body).ok()?;
    let wrapped = serde_json::json!({ "providers": { recipe.name: body } });
    let (mut providers, problems) = parse(&wrapped);
    problems.is_empty().then(|| providers.pop()).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------- recipes

    /// A recipe this repository ships must survive the reader the manifest
    /// uses. Anything else means shipping a file that opens as a `Problem` in
    /// the pane, next to the sentence explaining why the user's own file was
    /// wrong.
    #[test]
    fn every_shipped_recipe_survives_the_parser_that_reads_the_manifest() {
        for shipped in &RECIPES {
            let body: serde_json::Value = serde_json::from_str(shipped.body)
                .unwrap_or_else(|e| panic!("{}: not JSON: {e}", shipped.name));
            let wrapped = serde_json::json!({ "providers": { shipped.name: body } });

            let (providers, problems) = parse(&wrapped);
            assert!(
                problems.is_empty(),
                "{} is refused by the parser: {problems:?}",
                shipped.name
            );
            assert_eq!(providers.len(), 1, "{}", shipped.name);
            assert_eq!(providers[0].name, shipped.name);

            // And the convenience path agrees with the long one, since that is
            // what the command actually calls.
            assert_eq!(
                as_provider(shipped).map(|p| p.name),
                Some(shipped.name.to_string())
            );
        }
    }

    /// Names are keys, and a duplicate would make one of them unreachable
    /// through `recipe()` while both still showed in the list.
    #[test]
    fn every_shipped_recipe_has_its_own_name_and_says_what_to_edit() {
        let mut seen = BTreeSet::new();
        for shipped in &RECIPES {
            assert!(
                seen.insert(shipped.name),
                "two recipes are called {}",
                shipped.name
            );
            assert!(
                is_name(shipped.name),
                "{} is not a usable key",
                shipped.name
            );
            assert_eq!(
                super::recipe(shipped.name).map(|r| r.name),
                Some(shipped.name)
            );

            // Every one of these carries a placeholder host, database or
            // project id. A starting point that does not say which words are
            // placeholders is a command somebody approves as it stands.
            assert!(
                !shipped.edit.is_empty(),
                "{} says nothing about what has to be changed",
                shipped.name
            );
            assert!(!shipped.about.is_empty());
        }
        assert_eq!(super::recipe("no-such-recipe").map(|r| r.name), None);
    }

    /// The rule the module header states, held against the recipes it ships.
    ///
    /// A shipped `sh -c` would be the loudest possible contradiction: the
    /// consent screen shows every word of the command, and a shell string is
    /// one word that means anything.
    #[test]
    fn no_shipped_recipe_reaches_for_a_shell() {
        for shipped in &RECIPES {
            let provider = as_provider(shipped).expect("parses");
            for direction in [Direction::Pull, Direction::Push] {
                let argv = provider.command(direction);
                let Some(program) = argv.first() else {
                    continue;
                };
                assert!(
                    !matches!(program.as_str(), "sh" | "bash" | "zsh" | "dash" | "env"),
                    "{} runs `{program}` — there is no shell here",
                    shipped.name
                );
                // A pipeline or a redirect could only work through one, so
                // their presence means somebody wrote a command line and
                // expected it to be read as one.
                for word in argv {
                    assert!(
                        !word.starts_with('|') && !word.starts_with('>') && !word.starts_with('<'),
                        "{} has `{word}` in its argv, which needs a shell",
                        shipped.name
                    );
                }
            }
        }
    }

    /// Every shipped recipe reads or writes the one file the contract names,
    /// and gets its credential from the keystore rather than from the file.
    ///
    /// The first half is what makes a pull land anywhere: `produced` looks for
    /// `/stackvo/dump.sql` and nothing else, so a recipe writing next to it
    /// fails with "the provider wrote no dump" after a successful-looking run.
    /// The second is the rule the whole module exists for — a password in
    /// `env` would be a credential in a committed file.
    #[test]
    fn every_shipped_recipe_uses_the_agreed_path_and_keeps_its_password_out_of_the_file() {
        let dump = format!("{MOUNT}/{DUMP}");

        for shipped in &RECIPES {
            let provider = as_provider(shipped).expect("parses");
            assert!(
                !provider.secrets.is_empty(),
                "{} names no secret, so where does it authenticate from",
                shipped.name
            );
            assert!(
                provider.env.is_empty(),
                "{} puts values in `env`, which is the committed half",
                shipped.name
            );

            for direction in [Direction::Pull, Direction::Push] {
                let argv = provider.command(direction);
                if argv.is_empty() {
                    continue;
                }
                assert!(
                    argv.iter().any(|word| word.contains(&dump))
                        // The Upsun CLI splits it: `--directory=/stackvo`
                        // plus `--file=dump.sql`.
                        || (argv.iter().any(|w| w.contains(MOUNT))
                            && argv.iter().any(|w| w.contains(DUMP))),
                    "{} ({}) never names {dump}, so a pull would produce nothing \
                     and a push would send nothing",
                    shipped.name,
                    direction.as_str()
                );
            }
        }
    }

    /// `run_args` over a shipped recipe, end to end — the array is where every
    /// dangerous property of this feature lives, so at least one shipped recipe
    /// is checked through it rather than only through the parser.
    #[test]
    fn a_shipped_recipe_builds_the_argv_the_run_rules_require() {
        let provider =
            as_provider(super::recipe("mysql-remote").expect("shipped")).expect("parses");
        let args = run_args(&provider, Direction::Pull, Path::new("/tmp/scratch"));
        let line = args.join(" ");

        assert!(line.starts_with("run --rm"));
        assert!(line.contains("-i=false"), "{line}");
        assert!(line.contains(&format!("/tmp/scratch:{MOUNT}")), "{line}");
        // Name only: `docker run -e NAME` copies the value from this process,
        // so it is not in `ps` and not in a crash report.
        assert!(line.contains("-e MYSQL_PWD "), "{line}");
        assert!(
            !line.contains("MYSQL_PWD="),
            "a value reached the argv: {line}"
        );
        // Exactly one mount, and it is this application's scratch directory.
        assert_eq!(args.iter().filter(|a| *a == "-v").count(), 1);
    }

    fn recipe(body: serde_json::Value) -> (Vec<Provider>, Vec<Problem>) {
        parse(&serde_json::json!({ "providers": { "staging": body } }))
    }

    fn ok(body: serde_json::Value) -> Provider {
        let (out, problems) = recipe(body);
        assert!(problems.is_empty(), "{problems:?}");
        out.into_iter().next().expect("one provider")
    }

    fn refused(body: serde_json::Value) -> String {
        let (out, problems) = recipe(body);
        assert!(out.is_empty(), "it was accepted");
        problems.into_iter().next().expect("a reason").message
    }

    fn full() -> Provider {
        ok(serde_json::json!({
            "about": "the staging site",
            "image": "ghcr.io/example/dbtools:1",
            "pull": ["fetch", "--out", "dump.sql"],
            "push": ["send", "dump.sql"],
            "env": { "REMOTE_HOST": "staging.example.com" },
            "secrets": ["SSH_KEY"],
        }))
    }

    #[test]
    fn a_recipe_reads_as_what_it_says() {
        let p = full();
        assert_eq!(p.name, "staging");
        assert_eq!(p.image, "ghcr.io/example/dbtools:1");
        assert_eq!(p.pull, ["fetch", "--out", "dump.sql"]);
        assert!(p.offers(Direction::Pull) && p.offers(Direction::Push));
        assert_eq!(p.secrets, ["SSH_KEY"]);
    }

    /// The mistake everybody makes first, because every comparable tool takes a
    /// shell string — and the one this feature exists to refuse.
    #[test]
    fn a_command_line_is_not_a_command() {
        let message = refused(serde_json::json!({
            "image": "x", "pull": "pg_dump -Fc | gzip > dump.sql",
        }));
        assert!(message.contains("list of words"), "{message}");
        assert!(message.contains("no shell"), "{message}");
    }

    #[test]
    fn a_provider_that_does_nothing_is_refused_rather_than_listed() {
        let message = refused(serde_json::json!({ "image": "x" }));
        assert!(message.contains("neither"), "{message}");
    }

    #[test]
    fn an_image_is_required_and_is_one_word() {
        assert!(refused(serde_json::json!({ "pull": ["x"] })).contains("no image"));
        assert!(
            refused(serde_json::json!({ "image": "a b", "pull": ["x"] })).contains("no spaces")
        );
    }

    /// A secret named in `env` reads as though it resolved and does not.
    #[test]
    fn a_keystore_reference_in_the_plain_half_is_refused() {
        let message = refused(serde_json::json!({
            "image": "x",
            "pull": ["y"],
            "env": { "SSH_KEY": "keychain:stackvo/whatever" },
        }));
        assert!(message.contains("keystore reference"), "{message}");
        assert!(message.contains("secrets"), "{message}");
    }

    #[test]
    fn a_name_in_both_halves_is_refused_because_only_one_can_win() {
        let message = refused(serde_json::json!({
            "image": "x",
            "pull": ["y"],
            "env": { "TOKEN": "plain" },
            "secrets": ["TOKEN"],
        }));
        assert!(message.contains("both"), "{message}");
    }

    #[test]
    fn one_bad_recipe_does_not_take_the_others_with_it() {
        // A project that will not open because one line of one optional block
        // has a typo is how a feature becomes a thing people delete.
        let (out, problems) = parse(&serde_json::json!({
            "providers": {
                "good": { "image": "x", "pull": ["y"] },
                "bad": { "image": "x", "pull": "y" },
            }
        }));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "good");
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].provider, "bad");
    }

    // ------------------------------------------------------------- consent

    /// Agreeing to fetch is not agreeing to send.
    #[test]
    fn consent_is_granted_per_direction() {
        let p = full();
        let mut consent = Consent::default();
        consent.grant(
            "shop",
            "staging",
            Direction::Pull,
            &digest(&p, Direction::Pull),
        );

        assert!(consent.allows(
            "shop",
            "staging",
            Direction::Pull,
            &digest(&p, Direction::Pull)
        ));
        assert!(
            !consent.allows(
                "shop",
                "staging",
                Direction::Push,
                &digest(&p, Direction::Push)
            ),
            "a pull agreement was spent on a push"
        );
    }

    /// And the digest covers the direction, so the two can never collide even
    /// when the words happen to match.
    #[test]
    fn the_two_directions_never_share_a_digest() {
        let same = ok(serde_json::json!({
            "image": "x", "pull": ["move"], "push": ["move"],
        }));
        assert_ne!(
            digest(&same, Direction::Pull),
            digest(&same, Direction::Push)
        );
    }

    #[test]
    fn editing_any_part_of_the_recipe_asks_again() {
        let base = full();
        let before = digest(&base, Direction::Pull);

        for changed in [
            serde_json::json!({ "image": "ghcr.io/example/dbtools:2", "pull": ["fetch", "--out", "dump.sql"], "push": ["send", "dump.sql"], "env": { "REMOTE_HOST": "staging.example.com" }, "secrets": ["SSH_KEY"] }),
            serde_json::json!({ "image": "ghcr.io/example/dbtools:1", "pull": ["fetch", "--out", "other.sql"], "push": ["send", "dump.sql"], "env": { "REMOTE_HOST": "staging.example.com" }, "secrets": ["SSH_KEY"] }),
            serde_json::json!({ "image": "ghcr.io/example/dbtools:1", "pull": ["fetch", "--out", "dump.sql"], "push": ["send", "dump.sql"], "env": { "REMOTE_HOST": "production.example.com" }, "secrets": ["SSH_KEY"] }),
            serde_json::json!({ "image": "ghcr.io/example/dbtools:1", "pull": ["fetch", "--out", "dump.sql"], "push": ["send", "dump.sql"], "env": { "REMOTE_HOST": "staging.example.com" }, "secrets": ["SSH_KEY", "API_TOKEN"] }),
        ] {
            assert_ne!(
                digest(&ok(changed.clone()), Direction::Pull),
                before,
                "this change did not ask again: {changed}"
            );
        }
    }

    /// `about` is prose for a human and decides nothing, so rewording it must
    /// not throw away an approval somebody already gave.
    #[test]
    fn rewording_the_description_does_not_ask_again() {
        let a = ok(serde_json::json!({ "image": "x", "pull": ["y"], "about": "staging" }));
        let b = ok(serde_json::json!({ "image": "x", "pull": ["y"], "about": "the staging site" }));
        assert_eq!(digest(&a, Direction::Pull), digest(&b, Direction::Pull));
    }

    #[test]
    fn an_unreadable_consent_file_asks_again_rather_than_assuming_yes() {
        let dir = std::env::temp_dir().join(format!("stackvo-provider-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("provider-consent.json");

        for content in ["", "{", "{\"granted\": 3}", "null"] {
            std::fs::write(&path, content).unwrap();
            let consent = read_consent(&path);
            assert!(
                !consent.allows("shop", "staging", Direction::Push, "anything"),
                "{content:?} was read as an approval"
            );
        }
        // And a file that is not there at all.
        std::fs::remove_file(&path).unwrap();
        assert!(!read_consent(&path).allows("shop", "staging", Direction::Push, "anything"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---------------------------------------------------------------- plan

    fn nothing(_: &str) -> bool {
        false
    }
    fn everything(_: &str) -> bool {
        true
    }

    #[test]
    fn a_plan_names_the_secrets_and_never_their_values() {
        let p = full();
        let mut consent = Consent::default();
        consent.grant(
            "shop",
            "staging",
            Direction::Pull,
            &digest(&p, Direction::Pull),
        );

        let plan = plan("shop", &p, Direction::Pull, true, &consent, &everything);
        assert!(plan.runnable());
        assert_eq!(plan.secrets, ["SSH_KEY"]);

        // It travels to a web view, so the shape is what matters: names only.
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("SSH_KEY"));
        assert!(!json.contains("keychain:"), "{json}");
    }

    #[test]
    fn the_cheapest_refusal_comes_first() {
        // Being told to fill in three secrets for a direction an administrator
        // switched off is three pieces of work for nothing.
        let p = full();
        let plan = plan(
            "shop",
            &p,
            Direction::Pull,
            false,
            &Consent::default(),
            &nothing,
        );
        assert_eq!(plan.blocked, Some(Blocked::PolicyOff));
    }

    #[test]
    fn a_direction_a_recipe_does_not_offer_is_named_rather_than_silent() {
        let only_pull = ok(serde_json::json!({ "image": "x", "pull": ["y"] }));
        let plan = plan(
            "shop",
            &only_pull,
            Direction::Push,
            true,
            &Consent::default(),
            &everything,
        );
        assert_eq!(plan.blocked, Some(Blocked::NotOffered));
    }

    #[test]
    fn a_missing_secret_is_named_before_anything_is_spawned() {
        let p = full();
        let mut consent = Consent::default();
        consent.grant(
            "shop",
            "staging",
            Direction::Pull,
            &digest(&p, Direction::Pull),
        );

        let plan = plan("shop", &p, Direction::Pull, true, &consent, &nothing);
        assert_eq!(
            plan.blocked,
            Some(Blocked::MissingSecrets {
                names: vec!["SSH_KEY".into()]
            })
        );
    }

    // ------------------------------------------------------------- the run

    /// Every dangerous property of this feature is a property of this array.
    #[test]
    fn the_secret_value_is_never_an_argument() {
        let p = full();
        let args = run_args(&p, Direction::Pull, Path::new("/w/scratch"));

        // `-e NAME` with no `=` tells Docker to copy it from this process, so
        // the value is not in `ps`, not in a shell history, not in a crash
        // report.
        let index = args.iter().position(|a| a == "SSH_KEY").expect("the name");
        assert_eq!(args[index - 1], "-e");
        assert!(!args.iter().any(|a| a.contains("SSH_KEY=")), "{args:?}");

        // The plain half travels as a value, because it is not a secret.
        assert!(args.iter().any(|a| a == "REMOTE_HOST=staging.example.com"));
    }

    #[test]
    fn the_scratch_directory_is_the_only_mount() {
        let args = run_args(&full(), Direction::Pull, Path::new("/w/scratch"));
        let mounts: Vec<&String> = args
            .iter()
            .enumerate()
            .filter(|(i, _)| *i > 0 && args[i - 1] == "-v")
            .map(|(_, a)| a)
            .collect();
        assert_eq!(mounts, [&format!("/w/scratch:{MOUNT}")]);
        assert!(args.contains(&"--rm".to_string()));
    }

    #[test]
    fn the_command_is_the_last_thing_and_the_image_is_before_it() {
        let args = run_args(&full(), Direction::Push, Path::new("/w/s"));
        let image = args
            .iter()
            .position(|a| a == "ghcr.io/example/dbtools:1")
            .expect("the image");
        assert_eq!(&args[image + 1..], ["send", "dump.sql"]);
    }

    // ------------------------------------------------------ what came back

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stackvo-prod-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The escape this check exists for.
    ///
    /// A container that can write into a mounted directory can write a symlink
    /// there, and `dump.sql -> /etc/passwd` would have this application read the
    /// host's own file and hand it to a database.
    #[cfg(unix)]
    #[test]
    fn a_symlink_where_the_dump_should_be_is_refused() {
        let dir = scratch("symlink");
        std::os::unix::fs::symlink("/etc/passwd", dir.join(DUMP)).unwrap();

        let error = produced(&dir).expect_err("a symlink was believed");
        assert_eq!(error.code, Code::Forbidden);
        assert!(error.message.contains("symbolic link"), "{}", error.message);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_dump_is_refused_because_restoring_it_is_dropping_the_database() {
        let dir = scratch("empty");
        std::fs::write(dir.join(DUMP), "").unwrap();
        let error = produced(&dir).expect_err("an empty dump was believed");
        assert!(error.message.contains("empty"), "{}", error.message);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_written_at_all_says_so() {
        let dir = scratch("absent");
        let error = produced(&dir).expect_err("a missing dump was believed");
        assert_eq!(error.code, Code::NotFound);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_real_dump_comes_back() {
        let dir = scratch("real");
        std::fs::write(dir.join(DUMP), "CREATE TABLE t (id int);\n").unwrap();
        assert_eq!(produced(&dir).unwrap(), dir.join(DUMP));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Two projects wanting `SSH_KEY` are two credentials.
    #[test]
    fn a_secret_is_scoped_to_the_project_and_the_provider() {
        assert_ne!(
            secret_entry("shop", "staging", "SSH_KEY"),
            secret_entry("blog", "staging", "SSH_KEY")
        );
        assert_ne!(
            secret_entry("shop", "staging", "SSH_KEY"),
            secret_entry("shop", "production", "SSH_KEY")
        );
    }
}
