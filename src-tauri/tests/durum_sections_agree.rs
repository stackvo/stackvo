//! §4 of `docs/durum.md` is a view of §3, and the two had drifted.
//!
//! §3 is the table of remaining engineering debt, one row per item, with a
//! status. §4 is the suggested order — a short list of the same items, said
//! again in a sentence each. Two places stating one fact, which is the shape
//! `tools/linux/Dockerfile` already names in its own opening comment:
//!
//! > when two places state one fact, the second one is the one that goes stale.
//!
//! It did. §4 said of **#35** that the Windows branch "does not even compile
//! here (`aws-lc-sys`'s Windows SDK)", while §3's row for #35 — marked 🟢 —
//! recorded that `cargo-xwin` downloads Microsoft's SDK, points clang at it and
//! removes exactly that obstacle, and that `tools/linux/run.sh --windows` is
//! how it is run. Anyone planning that work from §4 would have started at a
//! blocker that had been taken away, which is worse than an absent note: it
//! reads as a measurement.
//!
//! ## What this checks, and what it honestly cannot
//!
//! Checkable and checked: every `#N` §4 names is a row §3 has; §4 does not
//! carry an item §3 has marked done; and the specific claim that went stale is
//! held against the tree — the tool §3 credits is installed by the image and
//! wired into the script, so a §4 bullet cannot call it missing while it is
//! there.
//!
//! **Not** checkable: whether two paragraphs of Turkish prose agree. A test
//! that pretended to settle that would be a worse lie than the stale sentence
//! it replaced — the same line `platform_matrix_claims.rs` draws around the
//! four counts it refuses to judge.

use std::collections::BTreeMap;
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

fn durum() -> String {
    read("docs/durum.md")
}

/// The text between one `## N.` heading and the next.
fn section(document: &str, heading: &str) -> String {
    let start = document
        .find(heading)
        .unwrap_or_else(|| panic!("docs/durum.md has no `{heading}`"));
    let rest = &document[start + heading.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    rest[..end].to_string()
}

/// §3's rows: item number → status marker.
///
/// The table's own shape — `| 35 | … | 🟢 | … |` — read positionally rather
/// than by regex, because the fourth column is prose that contains pipes in
/// code spans and would end any pattern that tried to span it.
fn debt_rows(document: &str) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    for line in section(document, "## 3. Mühendislik borcu").lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let mut cells = line.split('|').skip(1);
        let Some(number) = cells.next().map(str::trim) else {
            continue;
        };
        let Ok(number) = number.parse::<u32>() else {
            continue; // the header row and the separator
        };
        let Some(status) = cells.nth(1).map(str::trim) else {
            continue;
        };
        out.insert(number, status.to_string());
    }
    assert!(
        out.len() >= 5,
        "read {} rows out of §3; the table's shape changed",
        out.len()
    );
    out
}

/// §4's bullets: item number → the sentence about it.
fn order_bullets(document: &str) -> BTreeMap<u32, String> {
    let mut out = BTreeMap::new();
    let mut current: Option<u32> = None;

    for line in section(document, "## 4. Önerilen sıra").lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("* **#") {
            let number: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            current = number.parse().ok();
            if let Some(n) = current {
                out.insert(n, trimmed.to_string());
            }
            continue;
        }
        // A bullet wraps; its continuation is indented and belongs to it.
        if let Some(n) = current {
            if line.starts_with("  ") && !trimmed.is_empty() {
                out.entry(n).and_modify(|text| {
                    text.push(' ');
                    text.push_str(trimmed);
                });
            } else if trimmed.is_empty() || trimmed.starts_with("* ") {
                current = None;
            }
        }
    }
    out
}

#[test]
fn every_item_in_the_order_is_an_item_in_the_table() {
    let document = durum();
    let rows = debt_rows(&document);
    for number in order_bullets(&document).keys() {
        assert!(
            rows.contains_key(number),
            "§4 lists #{number} and §3 has no row for it"
        );
    }
}

#[test]
fn nothing_the_table_calls_finished_is_still_in_the_order() {
    // §4 is what is left. An item §3 has closed outright has nothing left in
    // it, and leaving it in the order is the same staleness one status further
    // along.
    let document = durum();
    let rows = debt_rows(&document);
    for (number, bullet) in order_bullets(&document) {
        let status = &rows[&number];
        assert!(
            !status.contains('✅'),
            "§3 marks #{number} ✅ and §4 still asks for it: {bullet}"
        );
    }
}

/// The sentence that actually went stale, held against the tree.
///
/// §4 called `aws-lc-sys` a blocker that stopped the Windows branch compiling.
/// §3 credits `cargo-xwin` with removing it and names the script that runs it.
/// Both halves of that credit are files, so the claim is checkable: while the
/// image installs the tool and the script wires the mode, no bullet may say the
/// branch cannot be built here.
#[test]
fn the_windows_branch_can_be_built_here_and_the_document_may_not_say_otherwise() {
    let dockerfile = read("tools/linux/Dockerfile");
    let script = read("tools/linux/run.sh");

    assert!(
        dockerfile.contains("cargo install cargo-xwin"),
        "the image no longer installs cargo-xwin — §3 #35's claim has lost its footing"
    );
    assert!(
        script.contains("--windows") && script.contains("cargo xwin check"),
        "tools/linux/run.sh no longer offers the Windows mode §3 #35 names"
    );

    let bullet = order_bullets(&durum())
        .remove(&35)
        .expect("§4 still tracks #35");
    assert!(
        !bullet.contains("derlenemiyor"),
        "§4 says the Windows branch does not compile, and `tools/linux/run.sh --windows` \
         is in this tree: {bullet}"
    );
}

/// What is actually left of #35, so the next reader starts in the right place.
///
/// This gate has now turned over twice, and both turns are the same shape: it
/// pins the distinction the document is currently at risk of losing, and that
/// distinction moves as the work does.
///
/// It first asked whether §4 still said the Windows branch could not be
/// *compiled* here — `cargo-xwin` had removed that obstacle and the sentence
/// had not caught up. Then it asked whether the document still separated
/// type-checking from **running**, because passing a type check is not running
/// a suite and the two are easy to blur in a summary.
///
/// Both are settled: the branch compiles here and the suite runs on CI. The
/// distinction that can be lost now is the next one along — **running is not
/// passing.** The first real Windows run produced nineteen failures, two of
/// them product bugs; a §4 that says "Windows runs now" and stops has told a
/// reader the work is done.
#[test]
fn the_order_still_says_which_half_of_35_is_left() {
    let bullet = order_bullets(&durum())
        .remove(&35)
        .expect("§4 still tracks #35");
    assert!(
        bullet.contains("yeşil"),
        "§4's #35 no longer distinguishes the suite RUNNING from the suite \
         PASSING, which is the half that is left: {bullet}"
    );
}
