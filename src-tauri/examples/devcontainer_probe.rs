//! Does the exported devcontainer actually parse?
//!
//!   cargo run --example devcontainer_probe
//!
//! `devcontainer.rs` has fourteen unit tests and every one of them asks the
//! same *kind* of question: is this string in that string. They can settle that
//! a password left as a name and that no absolute path survived. They cannot
//! settle the question the export exists for — **whether Docker accepts the
//! file** — because the file is assembled by concatenating a fragment written
//! by somebody else into a document written here, and YAML is whitespace.
//!
//! That is the failure this repository has been caught by before: a thing that
//! is not run looks correct. So this renders **this machine's real projects
//! against this machine's real packages** and hands each result to
//! `docker compose config`, which is the parser that will actually read it.
//!
//! Nothing in the workspace is touched. The export goes to a temp directory,
//! `.env` is filled with a throwaway value for each placeholder — an empty one
//! would only prove Compose tolerates blanks — and everything is removed on the
//! way out.

use stackvo_desktop_lib::generator::ToolchainOptions;
use stackvo_desktop_lib::{devcontainer, instances, manifest, market, pkg, workspace};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let root = workspace::app_root();
    if !root.join("bootstrapped").is_file() {
        eprintln!("no workspace at {} — nothing was checked.", root.display());
        return ExitCode::FAILURE;
    }
    println!("workspace: {}", root.display());

    if !docker_is_up() {
        eprintln!("`docker compose` did not answer; nothing was checked.");
        return ExitCode::FAILURE;
    }

    let Ok(table) = instances::Table::load(&root) else {
        eprintln!("the instance table would not load; nothing was checked.");
        return ExitCode::FAILURE;
    };
    let Ok(tree) = pkg::Tree::open(&market::dir(&root)) else {
        eprintln!("no package tree; nothing was checked.");
        return ExitCode::FAILURE;
    };
    println!(
        "instances: {} enabled",
        table.instances.iter().filter(|i| i.enabled).count()
    );

    let opts = ToolchainOptions {
        tools: vec![],
        apt_packages: vec![],
        composer_version: "latest".into(),
        nodejs_version: "20".into(),
    };

    let mut projects = projects(&root);
    if projects.is_empty() {
        eprintln!("no projects with a manifest; nothing was checked.");
        return ExitCode::FAILURE;
    }

    // The branch that assembles somebody else's compose fragment is the one
    // worth driving, and on the first run of this probe **no project in the
    // workspace declared a service**, so eight green lines had all taken the
    // same empty path. So one case is made up: the first project, with every
    // enabled instance added to what it declares. Named as invented, because a
    // probe that quietly synthesises its own input is a probe reporting on
    // itself.
    let declared: Vec<String> = table
        .instances
        .iter()
        .filter(|i| i.enabled)
        .map(|i| i.service.clone())
        .collect();
    if !declared.is_empty() && !projects.iter().any(|(_, m)| !m.services.is_empty()) {
        let (name, m) = &projects[0];
        let mut invented = m.clone();
        invented.services = declared.clone();
        projects.push((format!("{name} (+{})", declared.join("+")), invented));
    }

    let scratch = std::env::temp_dir().join(format!("stackvo-dcprobe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);

    let mut failures = 0usize;
    for (name, m) in &projects {
        let plan = match devcontainer::plan(m, &table, &tree, &opts) {
            Ok(plan) => plan,
            Err(e) => {
                failures += 1;
                println!("  FAIL  {name}: the plan itself — {}", e.message);
                continue;
            }
        };

        let dir = scratch.join(name);
        if std::fs::create_dir_all(&dir).is_err() {
            failures += 1;
            println!("  FAIL  {name}: could not make a scratch directory");
            continue;
        }
        if let Err(e) = devcontainer::write(&dir, &plan) {
            failures += 1;
            println!("  FAIL  {name}: writing — {}", e.message);
            continue;
        }

        // A real value, not an empty one: an empty `.env` would prove only that
        // Compose tolerates blanks, and the thing under test is whether the
        // placeholder is in a position where a value can arrive at all.
        let env: String = plan
            .secrets
            .iter()
            .map(|name| format!("{name}=probe-only-value\n"))
            .collect();
        let _ = std::fs::write(dir.join(devcontainer::DIR).join(".env"), env);

        match config(&dir.join(devcontainer::DIR)) {
            Ok(services) => println!(
                "  ok    {name}  ({} file(s), {} service(s): {})",
                plan.files.len(),
                services.len(),
                services.join(", ")
            ),
            Err(why) => {
                failures += 1;
                println!("  FAIL  {name}\n{}", indent(&why));
            }
        }
        for note in &plan.skipped {
            println!("        skipped: {note}");
        }
    }

    // `STACKVO_PROBE_KEEP=1` leaves the exports behind. A probe that says
    // "ok" and deletes the evidence is one you have to edit to look at.
    if std::env::var_os("STACKVO_PROBE_KEEP").is_some() {
        println!("\nkept: {}", scratch.display());
    } else {
        let _ = std::fs::remove_dir_all(&scratch);
    }
    println!("\n{} project(s), {failures} failed.", projects.len());
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Every project in the workspace that carries a manifest.
fn projects(root: &Path) -> Vec<(String, manifest::Manifest)> {
    let Some(dir) = workspace::projects_root(root) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !path.join("stackvo.json").is_file() {
            continue;
        }
        if let Ok(m) = manifest::read(&path.join("stackvo.json"), name) {
            out.push((name.to_string(), m));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// `docker compose config`, and the service names it read out of the file.
///
/// `config` and not `up`: what is under test is the document, and starting
/// twenty containers to find out whether a colon was in the right column is a
/// probe nobody would run twice.
fn config(dir: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("docker")
        .args(["compose", "config", "--services"])
        .current_dir(dir)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn docker_is_up() -> bool {
    Command::new("docker")
        .args(["compose", "version"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("        {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Silences an unused-import warning on a platform where nothing above runs.
#[allow(dead_code)]
fn _unused(_: PathBuf) {}
