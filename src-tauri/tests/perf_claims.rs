//! The performance number in the product is the one the benchmark produced.
//!
//! I-1's whole design rests on a fact that a single figure hides: `vendor` in a
//! volume buys the **framework boot** and does nothing at all for writes, and
//! `storage/framework` is the one that buys the **writes**. That is why the
//! feature is a list of directories rather than one "make it fast" switch —
//! and the product was stating the average of the two, as prose, in two
//! languages, on the card's header rather than on the rows somebody toggles.
//!
//! The number now lives in `perf::GAINS`. This holds it against
//! `examples/perf_layer_bench.rs`, which is where it was measured and which now
//! records what it printed — the record used to exist only in a Vue file's
//! comment, which is not a place a measurement can be found from the program
//! that produced it.
//!
//! ## Why the source and not a run
//!
//! Running the bench takes minutes, needs Docker, and — the reason that settles
//! it — **measures the machine it runs on**. A test that re-measured would fail
//! on a faster laptop, which is the opposite of what this is for. What can be
//! settled mechanically is whether the two written records agree, and that is
//! the half that goes wrong: somebody improves a sentence and the figure moves.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// The `MEASURED` block out of the bench's own doc comment.
///
/// Read as `path workload multiple`, one row per line, from the fence that
/// opens with `MEASURED` — a marker rather than "the first code block", so
/// another example can be added above it without moving the parse.
fn measured() -> Vec<(String, String, String)> {
    let source = read("examples/perf_layer_bench.rs");
    let mut rows = Vec::new();
    let mut inside = false;

    for line in source.lines() {
        let Some(body) = line.strip_prefix("//!") else {
            continue;
        };
        let body = body.trim();

        if body == "MEASURED" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if body == "```" || body.is_empty() {
            break;
        }

        let fields: Vec<&str> = body.split_whitespace().collect();
        assert_eq!(
            fields.len(),
            3,
            "a MEASURED row is `path workload multiple`, got: {body}"
        );
        rows.push((
            fields[0].to_string(),
            fields[1].to_string(),
            fields[2].to_string(),
        ));
    }

    assert!(
        !rows.is_empty(),
        "no MEASURED block in examples/perf_layer_bench.rs — the number in the \
         product would have nothing behind it"
    );
    rows
}

/// `perf::GAINS`, read out of the source.
///
/// The table is `pub` and this could link the library, but reading it as text
/// is what lets the failure name the line somebody edited — and it is the same
/// shape `architecture_claims.rs` and `readme_claims.rs` use for the same
/// reason.
fn declared() -> Vec<(String, String, String)> {
    let source = read("src/perf.rs");
    let start = source
        .find("pub const GAINS:")
        .expect("perf.rs declares GAINS");
    let body = &source[start..];
    let end = body.find("];").expect("GAINS is terminated");

    let mut rows = Vec::new();
    for line in body[..end].lines() {
        let line = line.trim();
        if !line.starts_with("(\"") {
            continue;
        }
        let fields: Vec<&str> = line
            .trim_start_matches('(')
            .trim_end_matches("),")
            .split(',')
            .map(|f| f.trim().trim_matches('"'))
            .collect();
        assert_eq!(fields.len(), 3, "a GAINS row has three fields: {line}");
        // `38` in the table is `3.8` on the wire — held as an integer because
        // two `f32` literals of one decimal are two roundings, and comparing
        // them asks about IEEE 754 rather than about the benchmark.
        let tenths: u16 = fields[2].parse().expect("the multiple is an integer");
        rows.push((
            fields[0].to_string(),
            fields[1].to_string(),
            format!("{}.{}", tenths / 10, tenths % 10),
        ));
    }
    rows
}

#[test]
fn the_table_in_the_product_is_the_table_the_bench_printed() {
    assert_eq!(
        declared(),
        measured(),
        "perf::GAINS and the MEASURED block in examples/perf_layer_bench.rs \
         disagree. One of them was edited without the other."
    );
}

/// The failure this whole file exists to stop happening again.
///
/// The measurement was in `perf.explain`, in English and in Turkish, as a
/// sentence. A translated string is the worst possible home for a number
/// somebody measured: the translator has no way to know it is a measurement
/// rather than a phrase, and a locale can drift from the benchmark without
/// anything anywhere disagreeing. It is data now, and it renders on the row it
/// belongs to.
#[test]
fn no_locale_restates_the_measurement_as_prose() {
    for locale in ["en", "tr"] {
        let source = read(&format!("../src/i18n/locales/{locale}.js"));
        let Some(start) = source.find("  perf: {") else {
            panic!("{locale}.js has no perf block");
        };
        let block = &source[start..start + source[start..].find("\n  },").unwrap()];

        for (_, _, multiple) in measured() {
            assert!(
                !block.contains(&multiple),
                "{locale}.js states `{multiple}` in the perf strings. The \
                 measurement belongs in perf::GAINS, which renders on the row \
                 that was measured."
            );
            // The comma decimal a Turkish translation would naturally write,
            // which is the same number and would not have matched above.
            let comma = multiple.replace('.', ",");
            assert!(
                !block.contains(&comma),
                "{locale}.js states `{comma}` in the perf strings."
            );
        }
    }
}

/// The help documents carry the same multiples, in both languages.
///
/// A table with the raw seconds beside it is the right home for the number in
/// prose — unlike a locale string, it is a document somebody writes rather than
/// translates blind, and it has room to say which machine produced it. It is
/// still a fifth copy, so it is held like the others.
#[test]
fn both_help_documents_state_the_multiples_the_bench_printed() {
    for locale in ["en", "tr"] {
        let doc = read(&format!("../docs/help/{locale}/project-perf.md"));
        for (path, _, multiple) in measured() {
            // Turkish writes a comma decimal, which is the same number.
            let comma = multiple.replace('.', ",");
            assert!(
                doc.contains(&multiple) || doc.contains(&comma),
                "docs/help/{locale}/project-perf.md does not state {multiple} for {path}"
            );
        }
        // And says whose machine, which is the half the card cannot fit.
        assert!(
            doc.contains("perf_layer_bench.rs"),
            "docs/help/{locale}/project-perf.md does not name what measured it"
        );
    }
}

/// Every directory the feature offers is either measured or silent about it.
///
/// `bootstrap/cache` and `node_modules` have never been through the bench.
/// That is fine and is the point: what must not happen is one of them quietly
/// acquiring a neighbour's figure, which is the same act as the average this
/// replaced.
#[test]
fn nothing_carries_a_figure_that_was_not_measured_for_it() {
    let source = read("src/perf.rs");
    let offered: Vec<String> = source
        .lines()
        .skip_while(|line| !line.contains("pub fn suggestions("))
        .take_while(|line| !line.trim_start().starts_with("out.sort()"))
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("out.push(\"")?;
            rest.split('"').next().map(str::to_string)
        })
        .collect();

    assert!(
        offered.len() >= 4,
        "expected the four suggested directories, read {offered:?}"
    );

    let measured: Vec<String> = measured().into_iter().map(|(path, _, _)| path).collect();
    for path in &measured {
        assert!(
            offered.contains(path),
            "`{path}` is measured and no longer offered — the figure has nowhere to render"
        );
    }
}
