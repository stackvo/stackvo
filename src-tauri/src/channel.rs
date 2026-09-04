//! Which update this install is offered, and whether it is offered at all.
//!
//! Release channels, staged rollout and rollback sat behind the endpoint for a
//! reason that turned out to be arithmetic rather than engineering: none of it
//! means anything until there is a place to publish to and a key to sign with.
//! Both now exist, so this is ordinary work.
//!
//! ## What the updater plugin does and does not do
//!
//! `tauri-plugin-updater` fetches a manifest, compares its `version` against
//! the running one, verifies a signature, and installs. That is the whole of
//! it. It has no notion of a channel, no notion of a percentage, and — the one
//! that matters most — **no way to stop**. A release discovered to be broken
//! cannot be recalled, because every running copy will keep asking the same
//! endpoint and getting the same answer.
//!
//! So the three features are not three plugin settings. They are three fields
//! in the manifest and one decision made before the plugin is asked, and that
//! decision is what this module is.
//!
//! ## Rollback is the one that has to work first
//!
//! Channels and rollouts are conveniences. Rollback is the thing that decides
//! how bad a bad release gets, and the updater cannot do it by publishing an
//! older version: the plugin refuses to move backwards, so re-publishing 0.2.0
//! over 0.2.1 changes nothing for anybody who already took 0.2.1.
//!
//! Two fields, because "stop" and "go back" are different intentions:
//!
//! * `paused` — offer nothing. The release stands, and nobody else takes it.
//!   This is the button somebody presses at 2am, and it must not require
//!   building anything.
//! * `supersededBy` — offer this version *instead*, even though it is older.
//!   The explicit downgrade, which the plugin will not do on its own.
//!
//! ## Why the rollout bucket is hashed and not random
//!
//! A percentage implemented with a random number re-rolls on every check: an
//! install lands inside the 10% one hour and outside it the next, so a staged
//! rollout would offer an update, withdraw it, and offer it again. The bucket
//! has to be **stable for this install and this version**, and independent
//! between versions — otherwise the same unlucky 10% is first every time, and
//! a staged rollout stops being a sample.
//!
//! `sha256(install_id : version)`, first eight bytes, modulo 100.

use serde::{Deserialize, Serialize};

/// The stream an install follows.
///
/// Two, and adding a third is a decision rather than a variant: every channel
/// is a release somebody has to actually cut, and a channel nobody publishes to
/// is a setting that silently stops updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Stable,
    Beta,
}

/// The preferences key the chosen channel is stored under.
///
/// In `preferences.json`, beside the editor and the theme, because it is a
/// fact about this installation and not about the stack — and because that is
/// the one file this crate reads **before** the updater plugin is built, which
/// is the only moment the endpoint list can be chosen. See [`configure`].
pub const PREFERENCE: &str = "updateChannel";

/// The tag of the rolling release that serves `beta.json`.
///
/// `releases/latest/download/` is GitHub's own pointer to the newest published
/// non-prerelease, and there is no equivalent for pre-releases. So the beta
/// channel has a pointer of its own: one release, tagged `beta`, marked
/// pre-release so it can never *become* `latest`, holding one file that
/// `release.yml` replaces every time somebody presses Publish. The workflow's
/// `channel` job is the other half of this constant, and
/// `tests/update_channels.rs` holds the two together.
pub const ROLLING_TAG: &str = "beta";

/// What the stable endpoint ends in, and the only shape the beta one can be
/// derived from.
const STABLE_SUFFIX: &str = "/releases/latest/download/latest.json";

impl Channel {
    /// ## How an install ends up on beta
    ///
    /// This enum, [`offer`] and the whole rollout shape were here and tested
    /// before anybody could be on beta, and the reason was a dependency rather
    /// than an omission: nothing had been published, so a setting would have
    /// been the exact failure the note above warns about — somebody ticks
    /// "beta", the endpoint keeps answering `latest.json`, [`offer`] correctly
    /// says `otherChannel`, and they receive nothing at all, with no error.
    ///
    /// v0.2.0 shipped on 3 September 2026 and `release.yml` now publishes the
    /// beta manifest, so the setting exists. Three things keep the failure
    /// impossible rather than merely unlikely:
    ///
    /// * **The endpoint list is chosen per launch, from the preference.** The
    ///   updater plugin walks its endpoints until one answers, so a second
    ///   entry is a *fallback*, never a selector. That is used rather than
    ///   fought: a beta install asks `[beta.json, latest.json]` and a stable
    ///   install asks `[latest.json]` alone. A stable install can never reach
    ///   the beta file, and a beta install whose `beta.json` does not exist yet
    ///   — the state before the first pre-release — walks on to the stable one.
    ///   See [`endpoints`](Self::endpoints) and [`configure`].
    /// * **Beta means "stable, plus betas".** [`accepts`](Self::accepts) lets
    ///   a beta install take a stable manifest, so the fallback above is an
    ///   update and not an `otherChannel` refusal. The reverse stays refused:
    ///   a pre-release must never be offered to somebody who did not ask.
    /// * **`beta.json` names the newest published release, stable or not.** The
    ///   plugin stops at the first endpoint that answers, so a `beta.json`
    ///   still naming 0.3.0-beta.1 after 0.3.0 shipped would hide the stable
    ///   release from every beta install. The workflow refreshes it on every
    ///   publish, with the rule in `tools/beta-manifest.mjs`.
    ///
    /// The name this channel's manifest is published under.
    ///
    /// Stable keeps `latest.json` because that is the name `tauri-action`
    /// writes and the endpoint points at; the beta manifest is a stamped copy
    /// of it under its own name, so one publish produces both and they cannot
    /// drift apart.
    pub fn manifest_name(self) -> &'static str {
        match self {
            Channel::Stable => "latest.json",
            Channel::Beta => "beta.json",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "stable" => Some(Channel::Stable),
            "beta" => Some(Channel::Beta),
            _ => None,
        }
    }

    /// Whether an install on this channel may take a manifest from `published`.
    ///
    /// Beta is a superset of stable, not a sibling: somebody who opted into
    /// pre-releases still wants every stable release, and the endpoint walk in
    /// [`endpoints`](Self::endpoints) relies on it — the stable manifest is
    /// what a beta install reads when no beta has been published yet. Stable
    /// accepts only stable.
    pub fn accepts(self, published: Channel) -> bool {
        !matches!((self, published), (Channel::Stable, Channel::Beta))
    }

    /// This channel's endpoint, derived from the stable one.
    ///
    /// Derived rather than typed, for the reason `updater_endpoint.rs` gives
    /// at length: a constant is a second copy of the repository's name, and
    /// the copy is the one that goes stale. Stable is the endpoint as
    /// configured; beta replaces GitHub's `latest` pointer with the rolling
    /// release [`ROLLING_TAG`] names. `None` when the stable endpoint is not
    /// the GitHub shape this can be derived from — the caller then keeps the
    /// stable list rather than inventing a URL.
    pub fn endpoint(self, stable: &str) -> Option<String> {
        match self {
            Channel::Stable => Some(stable.to_string()),
            Channel::Beta => stable.strip_suffix(STABLE_SUFFIX).map(|base| {
                format!(
                    "{base}/releases/download/{ROLLING_TAG}/{}",
                    self.manifest_name()
                )
            }),
        }
    }

    /// The endpoint list this channel asks, in the order the plugin walks it.
    ///
    /// Stable is always last and always present: the fallback is the whole
    /// safety of the design, and a beta list that could lose it would be a
    /// setting that silently stops updates. A beta endpoint that cannot be
    /// derived leaves the stable list untouched.
    pub fn endpoints(self, stable: &str) -> Vec<String> {
        match self {
            Channel::Stable => vec![stable.to_string()],
            Channel::Beta => match self.endpoint(stable) {
                Some(beta) => vec![beta, stable.to_string()],
                None => vec![stable.to_string()],
            },
        }
    }
}

/// The channel this install chose, from `preferences.json`.
///
/// Through `prefs_get` rather than a second reader, so a corrupt file is
/// moved aside the way the first command would have moved it anyway. Anything
/// unreadable, absent or unknown is stable — the direction in which a wrong
/// answer costs a pre-release nobody asked for rather than an update.
pub fn preferred() -> Channel {
    crate::commands::prefs_get()
        .ok()
        .and_then(|prefs| {
            prefs
                .get(PREFERENCE)
                .and_then(|v| v.as_str())
                .and_then(Channel::parse)
        })
        .unwrap_or_default()
}

/// Rewrite the updater plugin's `endpoints` for `wanted`, in place.
///
/// The plugin reads its endpoint list once, out of the app configuration, when
/// it is built — there is no override on `check()` and no channel placeholder
/// in a URL — so the list has to be chosen before that, and this is the
/// function that chooses it. Pure over the JSON so every branch is testable
/// without a running app; [`configure`] is the two lines that hand it the real
/// configuration.
///
/// Returns the list the plugin will walk. Empty when there was nothing to
/// configure, which is the "updater not set up" case and not an error here.
pub fn apply(updater: &mut serde_json::Value, wanted: Channel) -> Vec<String> {
    let Some(stable) = updater
        .get("endpoints")
        .and_then(|list| list.as_array())
        .and_then(|list| list.first())
        .and_then(|first| first.as_str())
        .map(str::to_string)
    else {
        return Vec::new();
    };

    let list = wanted.endpoints(&stable);
    if wanted == Channel::Beta && list.len() == 1 {
        // Said out loud, once per launch: the person chose beta, and this
        // launch cannot honour it. Stable updates still arrive, which is the
        // one outcome this module refuses to trade away.
        tracing::warn!(
            stable,
            "the beta endpoint cannot be derived from the stable one; this launch asks stable only"
        );
    }
    if let Some(section) = updater.as_object_mut() {
        section.insert("endpoints".into(), serde_json::json!(list));
    }
    list
}

/// Choose this launch's endpoint list from the stored preference.
///
/// Called on the `Context` before `tauri::Builder::run`, which is the only
/// moment the plugin's configuration can still be changed. A change to the
/// preference therefore takes effect at the next launch, and the settings
/// screen says so; until then the check keeps asking the list this launch
/// chose, which in both directions is a safe list — see
/// [`Channel::accepts`].
pub fn configure(config: &mut tauri::Config) {
    let wanted = preferred();
    let Some(updater) = config.plugins.0.get_mut("updater") else {
        return;
    };
    let list = apply(updater, wanted);
    tracing::info!(channel = ?wanted, endpoints = ?list, "update endpoints for this launch");
}

/// The fields this app adds to the updater manifest.
///
/// `tauri-plugin-updater` ignores keys it does not know, so these travel in the
/// same `latest.json` it already reads — one file, one publish, no second
/// endpoint that can be forgotten.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rollout {
    /// The version the manifest offers.
    pub version: String,
    /// Which stream this manifest belongs to.
    pub channel: Channel,
    /// Percentage of installs that may take it, 0–100.
    ///
    /// Absent means 100: a manifest published without thinking about staging is
    /// a manifest offered to everybody, which is what happens today and what
    /// somebody who has not read this file will expect.
    #[serde(default = "everybody")]
    pub percent: u8,
    /// Offer nothing at all, whatever else this manifest says.
    pub paused: bool,
    /// Offer this version instead — including when it is older.
    ///
    /// The explicit downgrade. Empty means "no, take `version`".
    pub superseded_by: Option<String>,
}

fn everybody() -> u8 {
    100
}

/// What this install should be offered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "outcome", content = "detail")]
pub enum Offer {
    /// Take this version.
    Update(String),
    /// Nothing to do: already current, or newer.
    UpToDate,
    /// The publisher stopped this release.
    Paused,
    /// In the channel, but not in this wave yet.
    Waiting { bucket: u8, percent: u8 },
    /// The manifest is for a different stream.
    OtherChannel(Channel),
}

/// Which hundredth this install falls in for this version.
///
/// Public because it is the part a person will want to check by hand when an
/// install did not get an update they expected — `bucket(id, v) < percent` is
/// the whole rule, and being able to compute it is the difference between a
/// staged rollout and a black box.
pub fn bucket(install_id: &str, version: &str) -> u8 {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(install_id.as_bytes());
    // A separator, so `("ab", "c")` and `("a", "bc")` are different installs
    // rather than the same one. `:` cannot appear in a version.
    hasher.update(b":");
    hasher.update(version.as_bytes());
    let digest = hasher.finalize();

    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(head) % 100) as u8
}

/// Is `candidate` a later version than `current`, by semver precedence.
///
/// Not the `semver` crate: the crate is a dependency for a rule that fits in
/// a screen, and the manifest's versions are this app's own. But the rule has
/// to be the whole of semver's ordering and not "three numbers", because the
/// beta channel is built out of pre-release versions and the first version of
/// this split on `-` and compared the pieces as numbers — which made
/// `0.3.0-beta.1` *newer* than `0.3.0`, so a beta install would have declined
/// the stable release that superseded its beta, for ever.
///
/// The order, from the specification: the three numbers first; then a version
/// with no pre-release part beats one with; then the pre-release identifiers
/// one by one, numeric ones as numbers and lower than any word, words as
/// strings, and a shorter list that matches so far is the lower one. Build
/// metadata after `+` is ignored. A number that does not parse sorts as 0,
/// which makes a malformed version *older* — the safe direction, because the
/// outcome is "no update" rather than a downgrade to something unreadable.
fn newer(candidate: &str, current: &str) -> bool {
    precedence_of(candidate, current) == std::cmp::Ordering::Greater
}

/// Semver precedence of `a` against `b`.
fn precedence_of(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let (core_a, pre_a) = split(a);
    let (core_b, pre_b) = split(b);

    for i in 0..core_a.len().max(core_b.len()) {
        let (x, y) = (
            core_a.get(i).copied().unwrap_or(0),
            core_b.get(i).copied().unwrap_or(0),
        );
        if x != y {
            return x.cmp(&y);
        }
    }

    match (pre_a.is_empty(), pre_b.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => {
            for (x, y) in pre_a.iter().zip(pre_b.iter()) {
                let order = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
            pre_a.len().cmp(&pre_b.len())
        }
    }
}

/// `0.3.0-beta.1+build` → `([0, 3, 0], ["beta", "1"])`.
///
/// A leading `v` is tolerated because a tag carries one and a manifest copied
/// from a tag might.
fn split(version: &str) -> (Vec<u64>, Vec<String>) {
    let version = version.trim().trim_start_matches('v');
    let version = version.split('+').next().unwrap_or("");
    let (core, pre) = version.split_once('-').unwrap_or((version, ""));
    let core = core
        .split('.')
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect();
    let pre = if pre.is_empty() {
        Vec::new()
    } else {
        pre.split('.').map(str::to_string).collect()
    };
    (core, pre)
}

/// The whole decision, in the order it has to be made.
///
/// Order is the design. `paused` comes before everything because it is the
/// emergency stop and an emergency stop that can be overridden by a channel
/// mismatch is not one. `supersededBy` comes before the version comparison,
/// because its entire purpose is to defeat that comparison. The rollout bucket
/// comes last, because a wave is only meaningful for an update that would
/// otherwise be offered.
pub fn offer(manifest: &Rollout, wanted: Channel, install_id: &str, current: &str) -> Offer {
    if manifest.paused {
        return Offer::Paused;
    }
    if !wanted.accepts(manifest.channel) {
        return Offer::OtherChannel(manifest.channel);
    }

    // The deliberate downgrade. Checked against the RUNNING version, not
    // against the manifest's: superseding 0.2.1 with 0.2.0 must be a no-op for
    // somebody already on 0.2.0, or every install in the world reinstalls.
    if let Some(target) = manifest.superseded_by.as_deref().filter(|t| !t.is_empty()) {
        if target == current {
            return Offer::UpToDate;
        }
        return Offer::Update(target.to_string());
    }

    if !newer(&manifest.version, current) {
        return Offer::UpToDate;
    }

    let percent = manifest.percent.min(100);
    let bucket = bucket(install_id, &manifest.version);
    if bucket >= percent {
        return Offer::Waiting { bucket, percent };
    }

    Offer::Update(manifest.version.clone())
}

/// This install's identity, for the rollout bucket and nothing else.
///
/// A random string generated once and kept beside the preferences. Not a
/// machine id, not a hardware fingerprint, not anything derived from the user:
/// the only property required is that it is **stable and unique per install**,
/// and anything with more meaning than that is a value that leaves this machine
/// the first time somebody adds telemetry. `privacy_claims.rs` states the rule
/// this follows.
///
/// It never leaves the machine at all — the bucket is computed here and
/// compared here; the endpoint is asked for a file, not told who is asking.
pub fn install_id() -> String {
    let path = match crate::appdir::config() {
        Some(dir) => dir.join("install-id"),
        // No config directory is a machine this app can barely run on. A fixed
        // string means the rollout is not random for it, which is a worse
        // answer than a stable one only if you assume there are many such
        // machines — and an empty id would put every one of them in bucket 0,
        // i.e. always first into every wave.
        None => return "unidentified-install".to_string(),
    };

    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let fresh = fresh_id();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // A write that fails is not an error worth surfacing: the id is recomputed
    // next time, the install lands in a different bucket, and the only
    // consequence is that a staged rollout treats it as a new install. Failing
    // the update check over it would be worse than the thing it protects.
    let _ = std::fs::write(&path, &fresh);
    fresh
}

/// Sixteen bytes of randomness, hex.
///
/// From the OS. Not from a clock: `architecture_claims.rs` bans a clock as an
/// identity, and it banned it because two things built from
/// `SystemTime::now()` in the same microsecond are the same thing — which for
/// a rollout bucket would mean two installs that always update together.
///
/// A failure to read the OS generator falls back to a fixed string rather than
/// to a weaker source. That is a worse *bucket* — every such install lands in
/// the same one — and it is the honest failure: a home-made generator here
/// would be a value that looks random and is not, which is the state this
/// comment exists to prevent.
fn fresh_id() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::fill(&mut bytes).is_err() {
        return "unidentified-install".to_string();
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(version: &str) -> Rollout {
        Rollout {
            version: version.to_string(),
            channel: Channel::Stable,
            percent: 100,
            paused: false,
            superseded_by: None,
        }
    }

    #[test]
    fn an_ordinary_newer_release_is_offered() {
        assert_eq!(
            offer(&manifest("0.2.0"), Channel::Stable, "install", "0.1.0"),
            Offer::Update("0.2.0".into())
        );
    }

    #[test]
    fn the_running_version_and_anything_older_is_not() {
        for current in ["0.2.0", "0.3.0"] {
            assert_eq!(
                offer(&manifest("0.2.0"), Channel::Stable, "install", current),
                Offer::UpToDate,
                "current {current}"
            );
        }
    }

    #[test]
    fn pausing_beats_everything_else_in_the_manifest() {
        // The 2am button. It has to win over a channel match, a rollout wave
        // and a supersede — anything that can override it is a way for a
        // recalled release to keep installing.
        let mut m = manifest("0.2.0");
        m.paused = true;
        m.percent = 100;
        m.superseded_by = Some("0.1.9".into());
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.1.0"),
            Offer::Paused
        );
    }

    #[test]
    fn a_supersede_goes_backwards_which_is_the_whole_point() {
        // The updater plugin will not do this on its own: republishing an older
        // version leaves everybody who already took the bad one on it.
        let mut m = manifest("0.2.1");
        m.superseded_by = Some("0.2.0".into());
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.2.1"),
            Offer::Update("0.2.0".into())
        );
    }

    #[test]
    fn a_supersede_to_the_version_already_running_does_nothing() {
        // Checked against the RUNNING version rather than the manifest's. The
        // other way round, every install on 0.2.0 would reinstall 0.2.0 on
        // every check, forever.
        let mut m = manifest("0.2.1");
        m.superseded_by = Some("0.2.0".into());
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.2.0"),
            Offer::UpToDate
        );
    }

    #[test]
    fn a_manifest_for_another_channel_is_not_offered() {
        let mut m = manifest("0.9.0");
        m.channel = Channel::Beta;
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.1.0"),
            Offer::OtherChannel(Channel::Beta)
        );
    }

    #[test]
    fn a_bucket_is_stable_for_one_install_and_one_version() {
        // The failure a random number would have: an install inside the wave on
        // one check and outside it on the next, so the update appears and
        // disappears.
        for _ in 0..5 {
            assert_eq!(bucket("install-a", "0.2.0"), bucket("install-a", "0.2.0"));
        }
    }

    #[test]
    fn a_bucket_moves_between_versions_so_the_same_installs_are_not_always_first() {
        // Independence between versions is what makes a rollout a sample. If
        // the bucket were a property of the install alone, the same unlucky 1%
        // would take every release first, for ever.
        let ids: Vec<&str> = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let moved = ids
            .iter()
            .filter(|id| bucket(id, "0.2.0") != bucket(id, "0.3.0"))
            .count();
        assert!(
            moved >= 6,
            "only {moved} of 8 installs changed bucket between versions"
        );
    }

    #[test]
    fn the_separator_keeps_two_different_pairs_apart() {
        // Without it, `("ab", "c")` and `("a", "bc")` hash the same bytes and
        // are one install.
        assert_ne!(bucket("ab", "c"), bucket("a", "bc"));
    }

    #[test]
    fn a_wave_offers_roughly_its_percentage() {
        // Not exactly: a hash is not a shuffle, and demanding an exact split
        // would be a test of the digest rather than of the rule. What is
        // checked is that the number is in the right neighbourhood over enough
        // installs to mean something — a rollout that offers 90% at `percent:
        // 10` is the bug worth catching.
        let mut m = manifest("0.2.0");
        m.percent = 10;

        let offered = (0..1000)
            .filter(|i| {
                matches!(
                    offer(&m, Channel::Stable, &format!("install-{i}"), "0.1.0"),
                    Offer::Update(_)
                )
            })
            .count();
        assert!(
            (60..=140).contains(&offered),
            "{offered} of 1000 installs were offered a 10% rollout"
        );
    }

    #[test]
    fn a_zero_percent_wave_offers_nobody_and_a_hundred_offers_everybody() {
        let mut m = manifest("0.2.0");
        for (percent, expect_any) in [(0u8, false), (100, true)] {
            m.percent = percent;
            let offered = (0..200)
                .filter(|i| {
                    matches!(
                        offer(&m, Channel::Stable, &format!("install-{i}"), "0.1.0"),
                        Offer::Update(_)
                    )
                })
                .count();
            assert_eq!(
                offered > 0,
                expect_any,
                "{offered} offered at percent {percent}"
            );
            if percent == 100 {
                assert_eq!(offered, 200, "a full rollout must reach every install");
            }
        }
    }

    #[test]
    fn a_percentage_above_a_hundred_is_a_full_rollout_rather_than_nobody() {
        // `bucket` is 0..=99, so an unclamped 200 would still offer everybody —
        // but `percent` is a u8 read from a file somebody edits, and the
        // clamping is stated rather than emergent.
        let mut m = manifest("0.2.0");
        m.percent = 250;
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.1.0"),
            Offer::Update("0.2.0".into())
        );
    }

    #[test]
    fn a_manifest_with_no_percent_reaches_everybody() {
        // The compatibility case: `latest.json` as it is published today has
        // none of these fields. Deserialising it must mean "offered to all",
        // not "offered to nobody", or adding this module would stop every
        // update in the field.
        let json = r#"{"version":"0.2.0","notes":"","pub_date":"","platforms":{}}"#;
        let m: Rollout = serde_json::from_str(json).expect("an ordinary manifest parses");
        assert_eq!(m.percent, 100);
        assert_eq!(m.channel, Channel::Stable);
        assert!(!m.paused);
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.1.0"),
            Offer::Update("0.2.0".into())
        );
    }

    #[test]
    fn an_unreadable_version_is_treated_as_older_rather_than_newer() {
        // The safe direction: the outcome is "no update", not a move to
        // something nothing could parse.
        let m = manifest("not-a-version");
        assert_eq!(
            offer(&m, Channel::Stable, "install", "0.1.0"),
            Offer::UpToDate
        );
    }

    #[test]
    fn channels_name_different_files_in_one_release() {
        assert_eq!(Channel::Stable.manifest_name(), "latest.json");
        assert_ne!(
            Channel::Beta.manifest_name(),
            Channel::Stable.manifest_name()
        );
        assert_eq!(Channel::parse("BETA"), Some(Channel::Beta));
        assert_eq!(Channel::parse("nightly"), None);
    }

    #[test]
    fn version_comparison_is_numeric_and_not_lexical() {
        // `"0.10.0" > "0.9.0"` is false as strings, and an updater that never
        // offered 0.10 would look like a broken endpoint.
        assert!(newer("0.10.0", "0.9.0"));
        assert!(!newer("0.9.0", "0.10.0"));
        assert!(newer("1.0.0", "0.99.99"));
        assert!(!newer("0.2.0", "0.2.0"));
    }

    #[test]
    fn a_release_is_newer_than_its_own_pre_releases() {
        // The bug the first version of `newer` had: split on `-` and compared
        // as numbers, `0.3.0-beta.1` came out ahead of `0.3.0`, so a beta
        // install would have declined the stable release for ever.
        assert!(newer("0.3.0", "0.3.0-beta.1"));
        assert!(!newer("0.3.0-beta.1", "0.3.0"));
        // And a pre-release is still ahead of the stable before it.
        assert!(newer("0.3.0-beta.1", "0.2.0"));
        assert!(!newer("0.2.0", "0.3.0-beta.1"));
    }

    #[test]
    fn pre_release_identifiers_order_the_way_semver_says() {
        assert!(newer("0.3.0-beta.2", "0.3.0-beta.1"));
        // Numerically, not as strings: "10" > "9".
        assert!(newer("0.3.0-beta.10", "0.3.0-beta.9"));
        // A word beats a number, and words compare as strings: rc > beta.
        assert!(newer("0.3.0-rc.1", "0.3.0-beta.9"));
        assert!(newer("0.3.0-beta", "0.3.0-1"));
        // A longer list that matches so far is the later one.
        assert!(newer("0.3.0-beta.1", "0.3.0-beta"));
        assert!(!newer("0.3.0-beta.1", "0.3.0-beta.1"));
        // Build metadata and a tag's `v` change nothing.
        assert!(!newer("0.3.0+build.7", "v0.3.0"));
    }

    #[test]
    fn beta_accepts_stable_and_stable_does_not_accept_beta() {
        // Beta is a superset. The fallback endpoint depends on it: a beta
        // install whose `beta.json` does not exist yet reads `latest.json`,
        // and a refusal there would be the silent stop this module exists to
        // prevent. The other direction is the promise to everybody else.
        let mut stable = manifest("0.3.0");
        stable.channel = Channel::Stable;
        assert_eq!(
            offer(&stable, Channel::Beta, "install", "0.2.0"),
            Offer::Update("0.3.0".into())
        );

        let mut beta = manifest("0.3.0-beta.1");
        beta.channel = Channel::Beta;
        assert_eq!(
            offer(&beta, Channel::Stable, "install", "0.2.0"),
            Offer::OtherChannel(Channel::Beta)
        );
        assert_eq!(
            offer(&beta, Channel::Beta, "install", "0.2.0"),
            Offer::Update("0.3.0-beta.1".into())
        );
    }

    const STABLE: &str = "https://github.com/o/r/releases/latest/download/latest.json";

    #[test]
    fn the_beta_endpoint_is_derived_from_the_stable_one() {
        assert_eq!(
            Channel::Beta.endpoint(STABLE).as_deref(),
            Some("https://github.com/o/r/releases/download/beta/beta.json")
        );
        assert_eq!(Channel::Stable.endpoint(STABLE).as_deref(), Some(STABLE));
        // A stable endpoint that is not GitHub's `latest` pointer has no beta
        // sibling this can name, and inventing one would be a URL nobody
        // publishes to.
        assert_eq!(
            Channel::Beta.endpoint("https://updates.example.com/latest.json"),
            None
        );
    }

    #[test]
    fn a_beta_install_asks_beta_first_and_stable_always() {
        let beta = Channel::Beta.endpoints(STABLE);
        assert_eq!(beta.len(), 2);
        assert!(beta[0].ends_with("/releases/download/beta/beta.json"));
        assert_eq!(beta[1], STABLE, "stable is the fallback, and it is last");

        assert_eq!(Channel::Stable.endpoints(STABLE), vec![STABLE.to_string()]);
    }

    #[test]
    fn a_stable_install_never_sees_the_beta_endpoint() {
        let mut updater = serde_json::json!({ "endpoints": [STABLE], "pubkey": "k" });
        let list = apply(&mut updater, Channel::Stable);
        assert_eq!(list, vec![STABLE.to_string()]);
        assert_eq!(updater["endpoints"], serde_json::json!([STABLE]));
        assert_eq!(updater["pubkey"], "k", "nothing else in the section moves");
    }

    #[test]
    fn a_beta_install_keeps_stable_as_the_fallback() {
        let mut updater = serde_json::json!({ "endpoints": [STABLE] });
        let list = apply(&mut updater, Channel::Beta);
        assert_eq!(list.len(), 2);
        assert_eq!(list[1], STABLE);
        assert_eq!(updater["endpoints"], serde_json::json!(list));
    }

    #[test]
    fn an_underivable_beta_endpoint_leaves_the_stable_list_alone() {
        // The "silently stops updates" case, refused: a beta preference that
        // cannot be honoured must not cost the stable endpoint.
        let other = "https://updates.example.com/latest.json";
        let mut updater = serde_json::json!({ "endpoints": [other] });
        assert_eq!(apply(&mut updater, Channel::Beta), vec![other.to_string()]);
        assert_eq!(updater["endpoints"], serde_json::json!([other]));
    }

    #[test]
    fn an_updater_section_with_no_endpoints_is_left_as_it_is() {
        let mut updater = serde_json::json!({ "pubkey": "k" });
        assert!(apply(&mut updater, Channel::Beta).is_empty());
        assert_eq!(updater, serde_json::json!({ "pubkey": "k" }));
    }
}
