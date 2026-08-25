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
///
/// ## It asks whether a file **is** a key, not whether it mentions one
///
/// The first version searched every tracked file for the header string, and it
/// passed here and failed on CI — naming this file and `tools/keys.sh` as
/// committed private keys. Both merely *contain* the words, one as the needle
/// it searches for and the other in a comment, and both were untracked when it
/// was written, so `git ls-files` had not yet handed the scanner its own
/// haystack.
///
/// The shape is what distinguishes them, and it is narrow: a minisign secret
/// key is a comment line and a base64 line, and `tauri signer` writes that pair
/// base64-encoded again onto a single line. Nothing with prose in it looks like
/// either. Checking the shape rather than the substring also removes the reason
/// anybody would ever add an exemption list — an exemption list on a check like
/// this is how the real thing eventually gets exempted.
#[test]
fn no_private_signing_key_is_committed() {
    let mut offenders = Vec::new();
    for path in tracked_files() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue; // a binary file is not a key file
        };
        if is_a_private_key(&text) {
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

/// Is this file's whole content a minisign secret key?
///
/// Both spellings of the header: minisign's own, and `rsign`'s, which is what
/// Tauri's generator writes — measured against a real key rather than guessed,
/// because a scanner looking for the wrong word is a scanner that reads
/// everything and finds nothing.
fn is_a_private_key(text: &str) -> bool {
    const HEADERS: [&str; 2] = [
        "minisign encrypted secret key",
        "rsign encrypted secret key",
    ];
    let names_a_key = |line: &str| HEADERS.iter().any(|h| line.contains(h));

    let trimmed = text.trim();
    let lines: Vec<&str> = trimmed.lines().collect();

    // Plain: a comment line, a base64 line, and nothing else worth the name.
    if lines.len() <= 3 && lines.first().is_some_and(|l| names_a_key(l)) {
        return true;
    }

    // What `tauri signer generate` writes: the whole file, base64 again, on one
    // line. This is the form a key actually takes on disk, so it is the form
    // somebody would accidentally commit.
    if lines.len() == 1 {
        if let Some(inner) = decode_base64(trimmed).and_then(|b| String::from_utf8(b).ok()) {
            return inner.lines().next().is_some_and(names_a_key);
        }
    }

    false
}

/// ADR 0015's whole reason for existing, as a rule about this tree.
///
/// It was written while `PINNED` was still empty, which is exactly when a rule
/// like this is worth having: the moment somebody fills `PINNED` is the moment
/// the shortcut is tempting — there is already a key pair, it already works,
/// and reusing it saves an afternoon. What it costs is the property the
/// separation was for. The ceremony has since happened (ADR 0033) and a key is
/// pinned, so this stopped being vacuous without anybody having to remember to
/// come back and turn it on.
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
    for subcommand in ["generate)", "check)", "sign)", "verify)"] {
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

/// A signature takes the published name only after the app has accepted it.
///
/// The gap this closes is the one the ceremony could not see. `sign` used to
/// produce a `.minisig` and say "publish it", and nothing between that sentence
/// and a user's machine ever asked the question that decides it: **is this a
/// signature a shipped build accepts?** The two keys live in one directory and
/// their file names differ by one word; an index signed with the updater's key,
/// or with a key `PINNED` has since rotated away from, signs without complaint,
/// uploads cleanly, and is refused by every installed copy of the app at once —
/// with the publishing side holding no evidence at all.
///
/// So the order is the property, and it is the same one `market::install` uses
/// for a package: verified whole, *then* moved. A half-right artefact already
/// wearing the right name is the failure the far end cannot recover from on its
/// own, and here the far end is everybody.
#[test]
fn the_ceremony_verifies_a_signature_before_it_publishes_it() {
    let script = read("tools/keys.sh");
    let sign = function_body(&script, "sign()");

    let verified = sign
        .find("verifier ")
        .expect("tools/keys.sh sign no longer asks the verifier anything");
    let published = sign
        .find("mv \"$file.sig\"")
        .expect("tools/keys.sh sign no longer moves the signature into place");

    assert!(
        verified < published,
        "tools/keys.sh publishes the signature before checking it. The check is \
         only worth having above the rename: a `.minisig` that exists is one \
         somebody uploads."
    );

    // And what it asks is the app itself, not a second opinion. A `minisign -V`
    // here, or a key comparison in shell, would be a reimplementation of the
    // thing being tested — and the round the two disagree is the round this
    // prints a tick for a file every machine refuses.
    assert!(
        script.contains("--example verify_index"),
        "tools/keys.sh no longer asks the app's own verifier"
    );
    let example = read("src-tauri/examples/verify_index.rs");
    assert!(
        example.contains("Keys::pinned()"),
        "examples/verify_index.rs no longer checks against the set a shipped \
         build trusts, so agreeing with it proves nothing"
    );
}

/// And the verifier it leans on answers with its exit status, not its prose.
///
/// `sign` decides whether to publish from one number. If `verify_index` ever
/// returned 0 for a refusal — an early `return` down a wrong-arguments path, a
/// refactor that prints the error and falls through — the gate above would keep
/// printing its reassuring line while publishing whatever it was handed, and
/// every test in this file would still pass. So this runs the real thing.
///
/// The fixture is `signed_refresh.rs`'s: a real index and a real signature from
/// a throwaway key. Not pinned, which is what makes it useful here — the same
/// bytes are a refusal by default and an acceptance when the key is named,
/// which is the mirror operator's case and the only pair that shows the exit
/// status tracks the answer rather than the weather.
///
/// ## The binary is found, not built, and that is not a preference
///
/// The first version ran `cargo run --example verify_index`, which passes on
/// its own (`cargo test --test key_ceremony`) and **deadlocks the suite**: the
/// outer `cargo test` holds the lock on the build directory for as long as it
/// is running tests, the inner cargo waits for it, and the run that waits is
/// the one holding it. Measured, not reasoned about — the full suite stopped
/// here for good, and the log's last line was this test's name.
///
/// So the already-built example is located beside this test binary instead.
/// `cargo test` builds every target in the package, examples included, so a
/// whole-suite run — the one CI does — has it. A single-target run does not,
/// and that is a spoken skip rather than a silent one.
#[test]
fn the_verifier_the_ceremony_asks_says_no_with_its_exit_status() {
    let Some(verifier) = built_example("verify_index") else {
        eprintln!(
            "SKIPPED: examples/verify_index is not built here. A whole-suite \
             `cargo test` builds it; a single `--test key_ceremony` run does not."
        );
        return;
    };

    let dir = std::env::temp_dir().join(format!("stackvo-verifier-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let index = dir.join("registry.json");
    std::fs::write(&index, FIXTURE_INDEX).unwrap();
    std::fs::write(dir.join("registry.json.minisig"), FIXTURE_SIG).unwrap();

    let run = |extra: &[&str]| {
        let out = std::process::Command::new(&verifier)
            .arg(&index)
            .args(extra)
            .output()
            .expect("the built verify_index example runs");
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    };

    let (code, said) = run(&[]);
    assert_eq!(
        code,
        Some(1),
        "a signature by a key no shipped build trusts was not a failure:\n{said}"
    );

    let (code, said) = run(&["--key", FIXTURE_PUB]);
    assert_eq!(
        code,
        Some(0),
        "the same signature was refused when its key was named — the mirror \
         operator's whole path:\n{said}"
    );

    // And the file it defaults to is the one the app fetches. Getting this wrong
    // would make every check above pass against a signature nobody publishes.
    std::fs::remove_file(dir.join("registry.json.minisig")).unwrap();
    let (code, _) = run(&["--key", FIXTURE_PUB]);
    assert_eq!(
        code,
        Some(1),
        "with no `.minisig` beside it the verifier still said yes, so it is \
         reading some other file"
    );
}

/// The example binary this run built, if this run built it.
///
/// `target/<profile>/examples/<name>`, reached from the test binary's own path
/// (`target/<profile>/deps/<test>-<hash>`) rather than from a guess about the
/// profile — a hard-coded `debug` would quietly stop testing anything the first
/// time somebody ran the suite in release.
fn built_example(name: &str) -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.pop(); // deps
    path.pop(); // the profile directory
    path.push("examples");
    path.push(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    path.is_file().then_some(path)
}

/// `signed_refresh.rs`'s fixture, and deliberately the same bytes.
///
/// Two copies of one signature is the shape this repository keeps calling out,
/// and the alternative here is worse: an integration test cannot import
/// another's constants, and a *second* throwaway key would mean the ceremony
/// and the refresh path were shown to work on different evidence.
const FIXTURE_PUB: &str = "RWRfCCVyviCaGaAQ8mp1OEv6ArG4JzWXnboNGsyxPKXUj+LZcEb6BG5B";
const FIXTURE_INDEX: &str = r#"{
  "schemaVersion": 1,
  "sequence": 7,
  "generatedAt": "2026-08-23T09:00:00Z",
  "packages": []
}
"#;
const FIXTURE_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVSZkNDVnl2aUNhR1FVM3pINDE5M0hwcmZpZUhDaTRHQlBoYU1JUlY4VGdmclBVMnFYRFBRcWo0VDZJdmNuc0pGTmY3dGxsNXBzSHJHTTZmZnc0NG9uME5MMTVOelNKNGdBPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzg3NTEyNDA2CWZpbGU6aW5kZXguanNvbgpFUDlZZ2lkRE1IR2dsY1M4Z2M1ZzNtQWkraEwzRXZjOHpCbUNDb2NWZjJSMUJzbEhkemxlVHljOXFUQXAxeTVJT1ZsWDdXNFZOSVI1VWpha2xxd1hDQT09Cg==";

/// A path is handed on as the caller meant it, from whatever directory they are in.
///
/// The ceremony is `cd` into the packages repository and name the index sitting
/// in front of you, so a relative path is not an edge case — it is the only way
/// anybody will ever type this. And both helpers in the script run *somewhere
/// else*: `tauri()` in the repository root and `verifier()` in `src-tauri`,
/// each because the tool it calls has to be there. A path handed on unchanged
/// is therefore resolved against a directory the person has never seen.
///
/// It was, and the failure was the quiet kind: `keys.sh verify registry.json`
/// from the packages repository answered `reading registry.json: No such file
/// or directory` — a true sentence about the wrong directory, at the one moment
/// somebody is trying to close the chain for the first time. The signing step
/// had the same fault and predates the verifier.
///
/// ## The stub, and why the real tool is not used here
///
/// A `cargo` on `PATH` that only prints its arguments. Running the real one
/// would mean a nested cargo inside `cargo test`, which deadlocks on the build
/// directory lock — measured, and the reason the verifier test beside this one
/// locates a built binary instead. What is under test is which path the script
/// passes along, and a stub answers that exactly.
#[test]
fn a_relative_path_is_resolved_where_the_caller_stands() {
    let Some(bash) = bash_or_skip("the ceremony is a bash script") else {
        return;
    };

    let dir = std::env::temp_dir().join(format!("stackvo-relative-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("registry.json"), "{}\n").unwrap();
    std::fs::write(dir.join("registry.json.minisig"), "not checked here\n").unwrap();

    let stub = dir.join("bin").join("cargo");
    std::fs::write(&stub, "#!/bin/sh\nprintf '%s\\n' \"$@\"\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let out = std::process::Command::new(bash)
        .arg(repo_root().join("tools/keys.sh"))
        .args(["verify", "registry.json"])
        .current_dir(&dir)
        .env(
            "PATH",
            format!(
                "{}:{}",
                dir.join("bin").display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("tools/keys.sh runs");
    let said = String::from_utf8_lossy(&out.stdout).to_string();

    // Canonical on both sides. macOS reaches its temporary directory through
    // `/var`, which is a symlink to `/private/var`, and bash's `$PWD` is the
    // physical path — so a string comparison here fails on a script that is
    // doing exactly the right thing, which is its own kind of wrong test.
    let wanted = std::fs::canonicalize(dir.join("registry.json")).expect("the fixture is there");
    assert!(
        said.lines()
            .filter_map(|line| std::fs::canonicalize(line).ok())
            .any(|passed| passed == wanted),
        "the script handed on a path that does not resolve where the caller \
         stands. It runs the verifier from src-tauri, so `registry.json` there \
         is a different file — or none.\n  wanted: {}\n  passed:\n{said}",
        wanted.display()
    );
}

/// The key on the ceremony machine is the key the build pins.
///
/// The updater key has been asked this since the script existed; the content
/// key had not, and the asymmetry was the whole bug. Getting the updater pair
/// wrong is caught by a release that will not sign. Getting the content pair
/// wrong is caught by **nobody**: the index signs, publishes, and is refused
/// everywhere, and the one place the answer was ever written down is a constant
/// in this repository.
///
/// Run rather than read. A `grep` for the sentence would pass on a script whose
/// comparison is inverted, and this is a check whose failure mode is silence.
#[test]
fn the_ceremony_refuses_a_content_key_this_build_does_not_pin() {
    let Some(bash) = bash_or_skip("the ceremony is a bash script") else {
        return;
    };
    let pinned = key_list("PINNED");
    let Some(pinned) = pinned.first() else {
        return; // nothing pinned is a different state, and `keys.sh check` says so
    };

    // The key it holds *is* the pinned one: the ceremony says so and says
    // nothing wrong. Without this half the test would also pass on a script
    // that refuses every key it is ever shown.
    let good = fake_keydir("pinned", pinned);
    let (code, output) = run_check(&bash, &good);
    assert!(
        output.contains("private half"),
        "the ceremony no longer says whether the content key it holds is the \
         one this build pins:\n{output}"
    );
    assert_eq!(
        code,
        Some(0),
        "the ceremony refused the key this build actually pins:\n{output}"
    );

    // And one letter of it changed is a different key, which is the case that
    // reaches every machine at once.
    let mut wrong = pinned.clone();
    wrong.pop();
    wrong.push(if pinned.ends_with('A') { 'B' } else { 'A' });
    let bad = fake_keydir("wrong", &wrong);
    let (code, output) = run_check(&bash, &bad);
    assert_eq!(
        code,
        Some(1),
        "a content key this build does not pin was reported as fine. An index \
         signed with it is refused by every shipped copy of the app:\n{output}"
    );
}

/// Where `tools/keys.sh check` looks for the keys, filled with a public half
/// that names `key` and a private half that is only ever tested for existence.
fn fake_keydir(name: &str, key: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stackvo-ceremony-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch key directory");

    // `tauri signer generate` base64-encodes the whole `.pub` file, and the
    // script peels it — so the fixture has to be in that shape rather than the
    // plain one, or it would be testing the wrong reader.
    let plain = format!("untrusted comment: minisign public key\n{key}\n");
    std::fs::write(
        dir.join("registry.key.pub"),
        encode_base64(plain.as_bytes()),
    )
    .unwrap();
    // Never a real one, and it does not have to be: the script asks whether a
    // private half is *here*, and answers the key question from the public one.
    std::fs::write(dir.join("registry.key"), "not a key, and not read as one\n").unwrap();
    dir
}

fn run_check(bash: &str, keydir: &Path) -> (Option<i32>, String) {
    let out = std::process::Command::new(bash)
        .arg("tools/keys.sh")
        .arg("check")
        .current_dir(repo_root())
        .env("STACKVO_KEYDIR", keydir)
        .output()
        .expect("tools/keys.sh runs");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code(), text)
}

/// bash, or a spoken skip.
///
/// Windows is the platform where this is absent, and `foreign_import.rs` paid
/// for the lesson that a silent skip is a test that has stopped existing: it
/// measured a tree its own setup had not built and called the reader broken.
fn bash_or_skip(why: &str) -> Option<String> {
    for candidate in ["bash", "/bin/bash", "/usr/bin/bash"] {
        if std::process::Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(candidate.to_string());
        }
    }
    eprintln!("SKIPPED: no bash on this machine — {why}");
    None
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[((n >> (18 - 6 * i)) & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The body of a shell function, from its opening brace to the line that closes
/// it in column one.
fn function_body<'a>(script: &'a str, name: &str) -> &'a str {
    let start = script
        .find(&format!("\n{name} {{"))
        .unwrap_or_else(|| panic!("tools/keys.sh no longer declares {name}"));
    let rest = &script[start + 1..];
    let end = rest.find("\n}\n").expect("the function is closed");
    &rest[..end]
}

/// No sentence in the tree still says the registry key is unpinned.
///
/// The class of failure, not one instance of it: for as long as `PINNED` was
/// empty, half a dozen doc comments explained *why* it was empty and what that
/// meant for the chain — honest, load-bearing prose that the ceremony turned
/// into confident, specific, wrong claims. Two of them survived the round that
/// filled `PINNED` and were still telling readers the chain's first link was
/// open while a key sat six lines above them. §2 C is a row somebody plans work
/// from; the cost of that being stale is the work.
///
/// The README is scanned for the same reason and it had the same fault, one
/// paragraph long and pointed at users: "no official key is pinned yet (the
/// ceremony is an open decision)". That is the sentence somebody decides
/// whether to trust the Market on.
///
/// Checked in both directions, so it is an invariant rather than a one-off
/// clean-up: if the key is ever *removed* — a retirement, a rotation gone
/// wrong — the sentences that now say a key is carried become the wrong ones,
/// and this fails the other way round.
#[test]
fn no_source_file_still_describes_the_other_state_of_pinned() {
    let pinned_is_empty = key_list("PINNED").is_empty();
    let mut offenders = Vec::new();

    for path in tracked_files() {
        // Source, and the one document that makes this claim to users. Not
        // `CHANGELOG.md` or `docs/durum.md` §6, which are records: an entry
        // saying the list was empty when it was empty stays true, and a rule
        // that rewrote history to keep a scan quiet would be the worse trade.
        let source = path.extension().and_then(|e| e.to_str()) == Some("rs");
        let readme = path.ends_with("README.md");
        if !source && !readme {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // This file's own explanations of the rule are what it would trip over.
        if path.ends_with("key_ceremony.rs") {
            continue;
        }
        for (number, line) in text.lines().enumerate() {
            if says_the_wrong_thing(line, pinned_is_empty) {
                offenders.push(format!(
                    "{}:{}: {}",
                    path.strip_prefix(repo_root()).unwrap().display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "PINNED {} a key, and these lines still say the opposite:\n  {}\n\
         A stale comment about the trust chain is worse than none — it reads as \
         a measurement.",
        if pinned_is_empty {
            "carries no"
        } else {
            "carries"
        },
        offenders.join("\n  ")
    );
}

/// Does this line make a present-tense claim about `PINNED` that is now false?
///
/// Past tense is history and stays readable — "for a long time it was empty" is
/// the sentence that explains why the constant is shaped the way it is, and a
/// rule that deleted it would be trading one kind of unreadable for another.
fn says_the_wrong_thing(line: &str, pinned_is_empty: bool) -> bool {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("pinned") {
        return false;
    }
    // `is_empty` is the method, not a claim about the list.
    let prose = lower.replace("is_empty", "").replace("keys.is", "");
    const HISTORY: [&str; 6] = [
        "was ",
        "were ",
        "used to",
        "for a long time",
        "until ",
        "had been",
    ];
    if HISTORY.iter().any(|marker| prose.contains(marker)) {
        return false;
    }
    if pinned_is_empty {
        prose.contains("carries") || prose.contains("the official key is")
    } else {
        prose.contains("empty")
    }
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
