//! Two keys, one procedure, and neither of them in the repository.
//!
//! §3 #2 was never an engineering problem — it is a ceremony nobody had
//! performed. What *was* an engineering problem, and is what this file guards,
//! is that the ceremony had no written form: the updater key had a sentence in
//! a workflow comment, the content key (ADR 0015) had none at all, and the two
//! were reached by different tools. `tools/keys.sh` is the procedure now, and
//! the rules below are the ones a script cannot hold on its own because they
//! are about the tree rather than about a run.
//!
//! ## Why one procedure is the load-bearing part
//!
//! ADR 0015 pays a real price for separating the two keys: two secrets, two
//! places to leak from. What buys that back is that a leak of either is *one*
//! forgery rather than both — a fake installer or a fake package, never a fake
//! installer carrying fake packages. And the sentence ADR 0015 adds is the one
//! this file exists for: **the trade is only worth it while the procedure is
//! shared.** Same tool, same storage, same access list, same rotation. Two
//! procedures means one of them goes unmaintained, and the unmaintained one is
//! the one nobody notices has stopped being followed.
//!
//! ## What cannot be checked here
//!
//! Whether the private halves exist, who holds them, and whether a repository
//! secret is set. None of those are properties of this tree, and a test that
//! pretended to settle them would be the more dangerous kind of green.
//! `tools/keys.sh check` answers what a machine with the keys on it can answer;
//! this answers what a checkout can.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// The `pub const PINNED`/`RETIRED` lists, as key strings.
///
/// Read out of the source rather than linked against, because the interesting
/// case is a key somebody *added* — and a test that imported the constant would
/// be comparing the build to itself.
fn key_list(name: &str) -> Vec<String> {
    let source = read("src-tauri/src/signing.rs");
    let marker = format!("pub const {name}: &[&str] = &[");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("signing.rs no longer declares {name}"))
        + marker.len();
    let body = &source[start..];
    let end = body.find("];").expect("the list is closed");

    body[..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .filter(|k| !k.is_empty())
        .collect()
}

/// The updater's key, reduced to the same shape a pinned key is written in.
///
/// `tauri.conf.json` carries the whole `.pub` file, base64-encoded; `PINNED`
/// carries the key line inside it. Comparing the two without peeling the
/// envelope would be comparing a box to what is in it, and would report
/// "different" for one key stored two ways — which is the exact failure this
/// is meant to catch.
fn updater_key_line() -> Option<String> {
    let conf: serde_json::Value =
        serde_json::from_str(&read("src-tauri/tauri.conf.json")).expect("tauri.conf.json parses");
    let blob = conf["plugins"]["updater"]["pubkey"].as_str()?;
    if blob.is_empty() {
        return None;
    }
    let decoded = decode_base64(blob)?;
    let text = String::from_utf8(decoded).ok()?;
    text.lines().nth(1).map(|line| line.trim().to_string())
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for byte in text.bytes() {
        if byte.is_ascii_whitespace() || byte == b'=' {
            continue;
        }
        let value = ALPHABET.iter().position(|c| *c == byte)? as u32;
        acc = (acc << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Every file in the tree that a commit could carry.
fn tracked_files() -> Vec<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files runs");
    let files: Vec<PathBuf> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| repo_root().join(s))
        .collect();
    assert!(
        files.len() > 100,
        "git listed {} files — the scan has stopped working, and a scan that \
         reads nothing agrees with anything",
        files.len()
    );
    files
}

/// The one that would be unrecoverable.
///
/// A private key in a commit is public from the moment it is pushed and stays
/// in the history after it is deleted; the only real answer is to rotate, and
/// for the updater key rotating means every machine already running StackVo can
/// never be updated again. Cheap to check, and the cost of not checking is not
/// proportional to anything.
#[test]
fn no_private_signing_key_is_committed() {
    // Both spellings: minisign's own header, and Tauri's, which writes its own
    // wording into the file it generates.
    const HEADERS: [&str; 2] = ["minisign encrypted secret key", "untrusted comment: rsign"];

    let mut offenders = Vec::new();
    for path in tracked_files() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue; // a binary file is not a key file
        };
        // The key material is base64 too, so the header is looked for in both
        // envelopes — a `tauri signer` key file committed as its own base64
        // blob would otherwise read as an opaque string.
        let decoded = decode_base64(text.trim())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();
        if HEADERS
            .iter()
            .any(|h| text.contains(h) || decoded.contains(h))
        {
            offenders.push(
                path.strip_prefix(repo_root())
                    .unwrap()
                    .display()
                    .to_string(),
            );
        }
    }

    assert!(
        offenders.is_empty(),
        "a PRIVATE signing key is committed: {offenders:?}\nIt is public now. \
         Rotate it — deleting the file does not remove it from the history."
    );
}

/// ADR 0015's whole reason for existing, as a rule about this tree.
///
/// Vacuous today, because `PINNED` is empty and the ceremony has not happened —
/// and that is exactly when it is worth writing. The moment somebody fills
/// `PINNED` is the moment the shortcut is tempting: there is already a key
/// pair, it already works, and reusing it saves an afternoon. What it costs is
/// the property the separation was for.
#[test]
fn the_updater_and_the_registry_are_not_the_same_key() {
    let Some(updater) = updater_key_line() else {
        return; // no updater key is its own failure, and `updater_endpoint.rs` has it
    };
    assert!(
        !key_list("PINNED").contains(&updater),
        "the updater key and the registry key are the same key. One leak would \
         then forge a signed installer AND signed packages, which is the pair \
         ADR 0015 separates them to keep apart."
    );
}

/// A key cannot be trusted and refused at once.
///
/// `RETIRED` exists because rotation moves keys *in* at run time: a leaked key
/// that is merely absent can reintroduce itself with a `known-keys.json` it
/// signed. A key in both lists is a leak the build believes it has answered —
/// `verify` skips retired keys, so the effect is silent rather than wrong,
/// which is worse: the retirement looks done.
#[test]
fn no_key_is_both_pinned_and_retired() {
    let pinned = key_list("PINNED");
    let retired = key_list("RETIRED");
    let both: Vec<&String> = pinned.iter().filter(|k| retired.contains(k)).collect();
    assert!(
        both.is_empty(),
        "these keys are pinned and retired at once: {both:?}"
    );
}

/// The procedure is a file, and the things that point at it point at something.
///
/// The failure this guards is quiet: somebody renames a subcommand, and the
/// next person to perform the ceremony is reading instructions for a script
/// that no longer answers to them — during the one operation where improvising
/// is worst.
#[test]
fn the_ceremony_script_offers_what_everything_else_names() {
    let script = read("tools/keys.sh");
    for subcommand in ["generate)", "check)", "sign)"] {
        assert!(
            script.contains(subcommand),
            "tools/keys.sh no longer answers to `{}`",
            subcommand.trim_end_matches(')')
        );
    }

    // Both halves, in one place. A script that generated only the updater key
    // would be the second procedure ADR 0015 warns about, wearing the first
    // one's name.
    assert!(
        script.contains("for kind in updater registry"),
        "tools/keys.sh no longer generates both keys — ADR 0015's shared \
         procedure is what makes two keys worth having"
    );

    // The output has to be the name `market::refresh` fetches. `tauri signer`
    // writes `.sig`; the app asks for `.minisig`, and a rename left to a person
    // under time pressure is a rename that gets skipped.
    assert!(
        script.contains(".minisig"),
        "tools/keys.sh no longer produces the file name the app fetches"
    );
}

/// The private key never reaches a place the shell records.
///
/// `tauri signer generate -p <password>` and `sign -p <password>` both exist and
/// both are wrong here: an argument is in the process table while it runs and in
/// shell history afterwards. The interactive prompt is the point, and it is the
/// kind of thing a later "make it scriptable" change removes without noticing.
#[test]
fn the_ceremony_never_puts_a_password_on_a_command_line() {
    let script = read("tools/keys.sh");
    for line in script.lines() {
        let line = line.trim();
        if line.starts_with('#') || !line.contains("tauri signer") {
            continue;
        }
        assert!(
            !line.contains(" -p ") && !line.contains("--password"),
            "tools/keys.sh passes a password as an argument, which puts it in \
             the process table and in shell history: {line}"
        );
    }
}
