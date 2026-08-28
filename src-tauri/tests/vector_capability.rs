//! Local AI, answered as a package question.
//!
//! The question was whether Ollama, Qdrant and pgvector should be catalogue
//! services. The answer: **pgvector only, and not as a service.**
//!
//! ## Why the other two are out
//!
//! Ollama pulls four to eight gigabytes of model on first run and wants a GPU
//! it may not find; Qdrant is a third database beside the four already here.
//! Neither resembles anything in this catalogue, and "Laradock's 130 services"
//! is already written down as a fight not to have. Somebody who wants either
//! writes a `sidecars` block — exactly what that mechanism is for, which is
//! part of why the answer can be no without being a refusal.
//!
//! ## Ollama: the softest half of that reason is now the hardest
//!
//! "Wants a GPU it may not find" was written as a risk. It is not a risk on the
//! platform most of these users are on, it is a **certainty**: Docker Desktop
//! on macOS cannot pass the Apple GPU into a container at all — Apple's
//! virtualisation framework exposes no GPU API for it — so a containerised
//! Ollama on an Apple Silicon Mac is CPU-only and runs **3–5× slower** than the
//! native application it would be replacing. That is not a gap a newer image
//! closes; it held from M1 through the M5 line and the answer from Ollama's own
//! side is "on a Mac, run it natively".
//!
//! So the package that people ask for would be measurably worse than doing
//! nothing, with no way to fix it from here. Re-measured August 2026, against a
//! competitive review that listed Ollama in Laradock, ServBay and FlyEnv — all
//! three of which either run it natively or accept the loss.
//!
//! ## Why pgvector is not a service
//!
//! It is PostgreSQL. Same wire protocol, same port, same volume layout, same
//! client. A second service id would put two rows in the catalogue for one
//! database and make "how many databases are running" a question with two
//! answers. What actually differs is one **capability**.
//!
//! ## What this repository had to do about it: nothing
//!
//! That is the finding, and it is what carrying no service definitions buys.
//! The app ships no
//! service definitions, so a new service is a *package*, and a package this app
//! can already express costs no code here. This file is the proof of that
//! sentence rather than a restatement of it: the fixture is a real postgres
//! package with two versions, and if the capability path could not tell them
//! apart the answer would have been wrong.

use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/packages")
}

fn manifest(version: &str) -> serde_json::Value {
    let path = fixtures()
        .join("databases/postgres/versions")
        .join(version)
        .join("manifest.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn capabilities(version: &str) -> Vec<String> {
    manifest(version)["capabilities"]
        .as_array()
        .expect("a version manifest declares capabilities")
        .iter()
        .filter_map(|c| c.as_str().map(String::from))
        .collect()
}

/// One service, two versions.
///
/// The whole answer, as a shape on disk. If `16-pgvector` ever became its own
/// `service`, this fails — and it would fail for the right reason, because that
/// is the decision changing rather than a detail moving.
#[test]
fn pgvector_is_a_version_of_postgres_and_not_a_service_of_its_own() {
    for version in ["16", "16-pgvector"] {
        assert_eq!(
            manifest(version)["service"],
            "postgres",
            "version `{version}` no longer belongs to the postgres package. \
             the answer was pgvector as a VERSION: same protocol, same \
             port, same volume, same client — a second service id would give \
             the catalogue two rows for one database."
        );
    }

    let package: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixtures().join("databases/postgres/package.json"))
            .expect("the postgres package identity is readable"),
    )
    .expect("package.json parses");
    assert_eq!(package["service"], "postgres");

    // And it is not the recommended one. Somebody installing PostgreSQL should
    // get PostgreSQL; the vector build is what you pick when you need it, and
    // an image with an extra extension is not a better default.
    assert_eq!(
        package["recommendedVersion"], "16",
        "the pgvector build has become the recommended version. It carries an \
         extension most projects never load, and a default should be the plain \
         thing."
    );
}

/// The capability is the difference, and it is the only difference that matters.
///
/// `commands.rs` resolves an `instanceRef` by loading each instance's manifest
/// and asking whether its `capabilities` contain the wanted one. That is the
/// mechanism this decision leans on entirely: a project asking for `vector`
/// must match one of these two and not the other.
#[test]
fn the_vector_capability_is_what_separates_them() {
    let plain = capabilities("16");
    let vector = capabilities("16-pgvector");

    assert!(
        !plain.contains(&"vector".to_string()),
        "plain postgres claims the `vector` capability. It has no pgvector \
         extension installed, so a project matched to it would fail at the \
         first `CREATE EXTENSION vector` — a runtime failure standing in for a \
         catalogue answer."
    );
    assert!(
        vector.contains(&"vector".to_string()),
        "the pgvector build does not claim `vector`, which makes it \
         indistinguishable from plain postgres to the one mechanism that is \
         supposed to tell them apart"
    );

    // Everything a caller reaches postgres *as* is still true of both, which is
    // what makes them versions rather than different things.
    for shared in ["sql", "postgres-protocol"] {
        assert!(
            plain.contains(&shared.to_string()) && vector.contains(&shared.to_string()),
            "`{shared}` is not claimed by both versions. A version that stopped \
             being usable as ordinary postgres would be a different service \
             wearing the same id."
        );
    }
}

/// The image is pinned, and it is the one that carries the extension.
///
/// A moving tag is forbidden in a manifest. `pgvector/pgvector:pg16`
/// is the upstream build of postgres 16 with the extension compiled in — the
/// point of the version existing at all, and the one thing a reader cannot
/// verify from the version string.
#[test]
fn the_pgvector_version_runs_the_image_that_actually_has_pgvector() {
    let image = &manifest("16-pgvector")["image"];
    assert_eq!(image["repository"], "pgvector/pgvector");
    assert_eq!(image["tag"], "pg16");

    let plain = &manifest("16")["image"];
    assert_eq!(plain["repository"], "postgres");
    assert_eq!(plain["tag"], "16");

    for version in ["16", "16-pgvector"] {
        let tag = manifest(version)["image"]["tag"]
            .as_str()
            .expect("a version manifest names its image tag")
            .to_string();
        assert!(
            !["latest", "stable", "edge", "main", "master"].contains(&tag.as_str()),
            "version `{version}` runs a moving tag: an image that can \
             change under somebody who pulled it last month is not a version."
        );
    }
}

/// The package validates against the contract, like every other package.
///
/// The fixture is only worth what it proves, and it proves nothing if it is a
/// shape the app would refuse. Checked against the same schema
/// `tools/validate-contracts.mjs` uses, through the fields that schema requires
/// rather than by re-implementing it.
#[test]
fn the_fixture_carries_every_field_a_version_manifest_must() {
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("src-tauri has a parent")
                .join("contracts/package-version.schema.json"),
        )
        .expect("the package-version schema is readable"),
    )
    .expect("the schema parses");

    let required: Vec<String> = schema["required"]
        .as_array()
        .expect("the schema names its required fields")
        .iter()
        .filter_map(|r| r.as_str().map(String::from))
        .collect();
    assert!(
        !required.is_empty(),
        "the package-version schema requires nothing, so this test checks nothing"
    );

    for version in ["16", "16-pgvector"] {
        let m = manifest(version);
        for field in &required {
            assert!(
                m.get(field).is_some(),
                "postgres {version} has no `{field}`, which \
                 contracts/package-version.schema.json requires"
            );
        }
    }
}

/// A claim about the outside world has to carry the date it was measured.
///
/// This file already does it — "Re-measured August 2026, against a competitive
/// review that listed Ollama in Laradock, ServBay and FlyEnv" — and the reason
/// is in that sentence's shape. A dated claim that ages becomes *old*, which a
/// reader can weigh. An undated one becomes *wrong*, silently, and three did:
///
///   * `worktree.rs` said branch environments were "the one thing … that
///     nothing else in this space does" after dde shipped per-worktree
///     hostnames and certificates;
///   * `imports.rs` said "**Two** of them" while `ALL` carried seven;
///   * `mcp.rs` said "five of the eight competitors" against a survey of
///     seventeen products, at least seven of which ship one.
///
/// None of the three was checkable, because none said when it was true. This
/// requires the date rather than the truth — the measurement is review's job,
/// and the date is what makes review possible.
///
/// ## The ten below are declared debt, not exemptions
///
/// Writing this test found ten more module headers making a claim about
/// somebody else's product, where the review that prompted it had found three.
/// They are listed rather than dated because dating one means *measuring* it,
/// and a date written without a measurement is the same defect wearing a
/// timestamp. The list is here so the number is visible and shrinking it is a
/// task; a module that leaves it must carry a real date, and a new module has
/// to be dated or added deliberately — which is the point.
///
/// This is `published_urls.rs`'s arrangement: a declared list with a reason
/// attached, rather than a check narrow enough to always pass.
const UNDATED: [&str; 10] = [
    "agents.rs",
    "db.rs",
    "debugbridge.rs",
    "devcontainer.rs",
    "ide.rs",
    "phpini.rs",
    "provider.rs",
    "querylog.rs",
    "release.rs",
    "spx.rs",
];

#[test]
fn every_competitive_claim_names_when_it_was_measured() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    // Words that only appear when a module is comparing itself to something
    // outside this repository.
    const COMPARATIVE: [&str; 4] = [
        "competitor",
        "competitive review",
        "rivals",
        "in this space",
    ];

    let mut undated = Vec::new();
    for entry in std::fs::read_dir(&root).expect("src/ is readable") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();

        // The module header only — a claim buried in a function body is not the
        // one a reader takes on trust.
        let header: String = text
            .lines()
            .take_while(|l| l.starts_with("//!") || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let name = path
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .to_string();

        if !COMPARATIVE.iter().any(|w| header.contains(w)) {
            assert!(
                !UNDATED.contains(&name.as_str()),
                "{name} is on the undated list and no longer makes a comparative claim —                  take it off the list rather than leaving a name nobody can act on"
            );
            continue;
        }
        if UNDATED.contains(&name.as_str()) {
            continue;
        }
        // "Measured August 2026", "Re-measured August 2026", "measured in 2026".
        let dated = header.contains("easured") && header.contains("202");
        if !dated {
            undated.push(name);
        }
    }

    assert!(
        undated.is_empty(),
        "these compare this product to others and do not say when they last looked: {undated:?}\n\
         Write the date beside the claim, the way this file does. An undated claim about \
         somebody else's product does not age — it goes quietly wrong. If it genuinely \
         cannot be measured now, add it to UNDATED above, where it is at least counted."
    );
}
