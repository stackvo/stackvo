//! The first link of the chain, closed — and this is the only test that ever
//! gets past it.
//!
//! `signing.rs` proves the verifier: given bytes, a signature and a key, does it
//! say yes to the right one and no to the rest. What nothing proved is the step
//! above it — that `market::refresh`, handed a signed index and a machine that
//! trusts the key, actually **completes**. Every other test of that path is a
//! refusal: no key pinned, key checked before the signature file, index going
//! backwards. A chain with no passing case is a chain whose first success is
//! somebody's first release.
//!
//! ## Why this is its own binary
//!
//! `refresh` reads `policy::current()`, which is a `OnceLock` — set once per
//! process, deliberately, because a policy that could change under a running
//! app is not a policy. A test that points `STACKVO_POLICY_FILE` somewhere
//! therefore has to own its process, and an integration test is a process.
//!
//! ## Where the fixture came from
//!
//! `tools/keys.sh generate` and `tools/keys.sh sign` — the ceremony itself, run
//! against a throwaway key whose private half never left a scratch directory.
//! That provenance is the whole point of the file: it is not a signature this
//! repository made with its own idea of minisign, it is one the **documented
//! procedure** produced. If the ceremony stops working, this stops passing.
//!
//! The key here is *not* the official one, and it stays that way now that there
//! is one. `PINNED` carries the registry's real key and its private
//! half is on one machine, deliberately not in CI — so a test that wanted a
//! signature from it could only get one by putting the key somewhere a test
//! runner can reach, which is the arrangement the ceremony exists to avoid.
//! What a fixture key settles is the code, and the code is the same either way:
//! `verify` does not know which key it was handed.

use stackvo_desktop_lib::market::{self, LocalSource, Trust};

/// The public half, as it appears in a `.pub` file's key line.
const FIXTURE_PUB: &str = "RWRfCCVyviCaGaAQ8mp1OEv6ArG4JzWXnboNGsyxPKXUj+LZcEb6BG5B";

/// `registry.json.minisig`, exactly as `tools/keys.sh sign` wrote it — the
/// base64 envelope `tauri signer` produces, which the app learned to read in
/// the same round this file was written.
const FIXTURE_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVSZkNDVnl2aUNhR1FVM3pINDE5M0hwcmZpZUhDaTRHQlBoYU1JUlY4VGdmclBVMnFYRFBRcWo0VDZJdmNuc0pGTmY3dGxsNXBzSHJHTTZmZnc0NG9uME5MMTVOelNKNGdBPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzg3NTEyNDA2CWZpbGU6aW5kZXguanNvbgpFUDlZZ2lkRE1IR2dsY1M4Z2M1ZzNtQWkraEwzRXZjOHpCbUNDb2NWZjJSMUJzbEhkemxlVHljOXFUQXAxeTVJT1ZsWDdXNFZOSVI1VWpha2xxd1hDQT09Cg==";

const INDEX: &str = r#"{
  "schemaVersion": 1,
  "sequence": 7,
  "generatedAt": "2026-08-23T09:00:00Z",
  "packages": []
}
"#;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("stackvo-signed-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Point the process at a policy that trusts the fixture key.
///
/// `market.additionalKeys` rather than a patched `PINNED`, and that is the
/// honest spelling twice over: it is the field an organisation running its own
/// mirror uses, so this exercises the path somebody is on **today**, and it
/// leaves the shipped key list exactly as a release carries it.
fn trust_the_fixture_key(root: &std::path::Path) {
    let policy = root.join("policy.json");
    std::fs::write(
        &policy,
        format!(r#"{{"schemaVersion": 1, "market": {{"additionalKeys": ["{FIXTURE_PUB}"]}}}}"#),
    )
    .unwrap();
    // Before anything reads the policy. The `OnceLock` means the first reader
    // wins, so this has to happen before `refresh` is called and there is no
    // second chance inside one process.
    unsafe { std::env::set_var(stackvo_desktop_lib::policy::OVERRIDE_VAR, &policy) };
}

fn publish(dir: &std::path::Path, index: &str, signature: Option<&str>) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("registry.json"), index).unwrap();
    match signature {
        Some(sig) => std::fs::write(dir.join("registry.json.minisig"), sig).unwrap(),
        None => {
            let _ = std::fs::remove_file(dir.join("registry.json.minisig"));
        }
    }
}

/// The one that had never been shown to work.
#[test]
fn a_signed_index_from_a_trusted_key_refreshes() {
    let root = scratch("accepted");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, INDEX, Some(FIXTURE_SIG));

    let done = market::refresh(&root, &LocalSource::new(&source), Trust::Signed, None)
        .expect("a signed index from a key this machine trusts");
    assert_eq!(done.registry.sequence, 7);
    // And it says which key, which is what `market_status` puts on the screen.
    assert!(
        done.verified_by.is_some(),
        "a signed refresh did not report the key that verified it"
    );

    // And it was cached, which is what makes the next launch cheap.
    assert!(market::registry_path(&root).is_file());
}

/// One byte, and the whole thing stops. The signature covers the index; an
/// index that changed after it was signed is exactly what the link is for.
#[test]
fn an_index_edited_after_signing_is_refused() {
    let root = scratch("tampered");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(
        &source,
        &INDEX.replace("\"sequence\": 7", "\"sequence\": 8"),
        Some(FIXTURE_SIG),
    );

    let err = market::refresh(&root, &LocalSource::new(&source), Trust::Signed, None)
        .expect_err("an edited index verified");
    assert!(
        err.message.contains("none of the")
            || err.message.contains("signature")
            || err.message.contains("trusts"),
        "the refusal does not say the signature failed: {}",
        err.message
    );
    assert!(
        !market::registry_path(&root).is_file(),
        "a refused index was cached anyway"
    );
}

/// A signature the publisher forgot to upload is a refusal, not a downgrade.
#[test]
fn an_unsigned_index_is_refused_when_a_signature_was_asked_for() {
    let root = scratch("missing");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, INDEX, None);

    assert!(
        market::refresh(&root, &LocalSource::new(&source), Trust::Signed, None).is_err(),
        "a missing signature was accepted"
    );
}

/// The same index, unsigned trust: it refreshes, and nothing pretends a
/// signature was checked.
///
/// Here so the pair reads as one statement. `Trust::Unsigned` is what a
/// directory the user picked gets, and the difference between the two modes has
/// to be visible in a test rather than only in a comment.
#[test]
fn the_unsigned_mode_still_takes_an_index_with_no_signature() {
    let root = scratch("unsigned");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, INDEX, None);

    let done = market::refresh(&root, &LocalSource::new(&source), Trust::Unsigned, None)
        .expect("an unsigned refresh of an unsigned index");
    assert_eq!(done.registry.sequence, 7);
    // Nothing pretends a signature was checked. The pair of assertions is the
    // point: a field that reported "signed" from what the build pins rather
    // than from what happened would be true here and wrong.
    assert_eq!(
        done.verified_by, None,
        "an unsigned refresh claimed a key verified it"
    );
}

/// The order, stated as a refusal only one ordering can produce.
///
/// `refresh` checks the signature before it parses, for the same reason it
/// checks `manifestSha256` before it reads a manifest: a document is parsed by
/// code that trusts its shape, and settling where the bytes came from first is
/// the cheapest way to keep that trust honest. Nothing held it. The comment in
/// `market.rs` said so, the two statements sit four lines apart, and swapping
/// them would have broken no test — a parse error for an untrusted document is
/// a verifier that ran second, and second is not a verifier.
///
/// So the fixture is bytes that are **both** wrong: not an index at all, and
/// not signed by anything. Exactly one ordering can produce a refusal that
/// talks about the key, and that is the one asserted.
#[test]
fn bytes_are_verified_before_they_are_parsed() {
    let root = scratch("ordering");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, "this was never an index\n", Some(FIXTURE_SIG));

    let err = market::refresh(&root, &LocalSource::new(&source), Trust::Signed, None)
        .expect_err("bytes that are not an index refreshed");
    assert!(
        !err.message.contains("unreadable"),
        "the parser reached bytes no trusted key had signed — the signature is \
         checked second: {}",
        err.message
    );
    assert!(
        err.message.contains("none of the"),
        "the refusal is not the signature's: {}",
        err.message
    );
    assert!(
        !market::registry_path(&root).is_file(),
        "a refused index was cached anyway"
    );
}

// ------------------------------------------------- the day the index is signed

/// The mode that made the chain reachable, and the reason it had to exist.
///
/// Everything above this line tests a `Trust` somebody had to ask for. Nothing
/// asked: `market_refresh` passed `Trust::Unsigned` unless an administrator had
/// written `requireSignature`, so on a stock machine the verifier, the pinned
/// key and the whole first link were **code nobody ran**. Publishing a signed
/// index would have changed nothing a user could see.
///
/// `Trust::WhenSigned` takes the fact from the publisher instead of from a
/// setting, and these four are the four things it must do.
#[test]
fn a_publisher_who_starts_signing_is_verified_with_nobody_changing_a_setting() {
    let root = scratch("when-signed");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, INDEX, Some(FIXTURE_SIG));

    let done = market::refresh(
        &root,
        &LocalSource::new(&source),
        Trust::WhenSigned { seen_signed: false },
        None,
    )
    .expect("a signed index under the default mode");
    assert!(
        done.verified_by.is_some(),
        "the signature was published and nothing checked it — which is the \
         state this mode exists to end"
    );
}

/// And the day before, nothing breaks.
#[test]
fn a_publisher_who_has_never_signed_is_still_taken() {
    let root = scratch("when-unsigned");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, INDEX, None);

    let done = market::refresh(
        &root,
        &LocalSource::new(&source),
        Trust::WhenSigned { seen_signed: false },
        None,
    )
    .expect("an unsigned index from a publisher who has never signed");
    assert_eq!(done.registry.sequence, 7);
    assert_eq!(
        done.verified_by, None,
        "an unsigned refresh claimed a key verified it"
    );
}

/// The half that stops the mode being switched off by deleting a file.
///
/// Anyone who can serve a tampered index can serve a 404 for its signature.
/// Without this the whole mode is a suggestion: strip the `.minisig`, and a
/// machine with no memory takes the index unsigned and says so quietly on a
/// screen nobody is looking at.
#[test]
fn a_source_that_has_stopped_signing_is_refused() {
    let root = scratch("stripped");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(&source, INDEX, None);

    let err = market::refresh(
        &root,
        &LocalSource::new(&source),
        Trust::WhenSigned { seen_signed: true },
        None,
    )
    .expect_err("a source that stopped signing was taken anyway");
    assert!(
        err.message
            .contains("signed an index for this machine before"),
        "the refusal reads as a missing file rather than as a signature that \
         disappeared: {}",
        err.message
    );
    assert!(
        !market::registry_path(&root).is_file(),
        "a refused index was cached anyway"
    );
}

/// Absent and wrong are different answers, and only one of them survives.
///
/// The tempting shape of this mode is "try to verify, and carry on if it does
/// not work out" — which would accept a *tampered* index from a publisher who
/// has never signed, because a failed check and a missing file would arrive at
/// the same `else`. A signature that verifies against nothing is the loudest
/// evidence a refresh can produce.
#[test]
fn a_signature_that_fails_is_never_read_as_no_signature() {
    let root = scratch("bad-not-absent");
    trust_the_fixture_key(&root);

    let source = root.join("source");
    publish(
        &source,
        &INDEX.replace("\"sequence\": 7", "\"sequence\": 9"),
        Some(FIXTURE_SIG),
    );

    let err = market::refresh(
        &root,
        &LocalSource::new(&source),
        Trust::WhenSigned { seen_signed: false },
        None,
    )
    .expect_err("an index that failed its own signature was taken as unsigned");
    assert!(
        err.message.contains("none of the"),
        "the refusal is not the signature's: {}",
        err.message
    );
}
