//! The two facts `editor.rs` reads out of files it does not own.
//!
//! Whether a container can carry one at all turns on a pair of claims about
//! the generated Dockerfiles, and both are the kind that stay true until
//! somebody makes a reasonable change somewhere else:
//!
//! * **Nothing runs as a named user.** `editor::SERVER_DIR` is `/root/...`
//!   because `$HOME` is `/root` in every image this generator writes. Adding a
//!   `USER node` line — which is a perfectly ordinary hardening change, and one
//!   the node image documents — moves the editor server somewhere else and the
//!   volume then keeps a directory nothing writes to. Nothing would fail; the
//!   download would simply repeat for ever.
//! * **PHP mounts its source and the snapshot runtimes do not.** That is the
//!   whole of the refusal in `editor.rs`, and it is a property of
//!   `render_compose_service`, three hundred lines away, with no comment there
//!   pointing here.
//!
//! The address adds a third, and it is about where it is allowed to be built. `vscode-remote://attached-container+<hex>/<path>` is derived from
//! two facts — the container's name and the directory the source is mounted at
//! — and a second derivation of it anywhere else is a copy that goes stale in
//! silence: it would keep opening a window, just not onto this container.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn generator() -> String {
    let path = repo_root().join("src-tauri/src/generator.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Rust source with its comments and doc comments removed.
///
/// Line-based, because the alternative is that a comment explaining this very
/// rule counts as a breach of it — which is how two checks in this repository
/// failed on their own prose before.
fn without_comments(text: &str) -> String {
    text.lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `$HOME` is `/root`, in every image, because none of them changes the user.
#[test]
fn no_generated_dockerfile_switches_to_a_named_user() {
    let source = without_comments(&generator());

    assert!(
        !source.contains("USER "),
        "a generated Dockerfile now carries a USER directive. `editor::SERVER_DIR` \
         is {} on the argument that every image runs as root, and a named user \
         moves the editor server into that user's home — where the named volume \
         is not mounted. Nothing fails: the server simply downloads again after \
         every rebuild, for ever.",
        stackvo_desktop_lib::editor::SERVER_DIR
    );
}

/// The server directory is under the home directory it assumes.
#[test]
fn the_server_directory_is_under_the_root_home_it_assumes() {
    assert!(
        stackvo_desktop_lib::editor::SERVER_DIR.starts_with("/root/"),
        "SERVER_DIR is {}, which is not under the home directory the test above \
         is guarding",
        stackvo_desktop_lib::editor::SERVER_DIR
    );
}

/// PHP bind-mounts its source; the snapshot runtimes get no `volumes:` at all.
///
/// This is the fact the refusal rests on, and it is written in
/// `render_compose_service` as two arms of a `match` with nothing naming
/// `editor.rs`.
#[test]
fn the_compose_output_still_mounts_php_source_and_still_does_not_mount_nodes() {
    let source = generator();

    let at = source
        .find("pub fn render_compose_service(")
        .expect("the compose renderer is still called that");
    let body = &source[at..];
    let body = &body[..body
        .find("\n/// One declared container")
        .unwrap_or(body.len())];

    assert!(
        body.contains(&format!(
            "{{projects_root}}/{{name}}:{}",
            stackvo_desktop_lib::editor::PHP_WORKDIR
        )),
        "the PHP service no longer bind-mounts the project at {}. `editor.rs` \
         says a PHP container can carry an editor precisely because it does.",
        stackvo_desktop_lib::editor::PHP_WORKDIR
    );

    let none_arm = body
        .find("        None => {")
        .map(|at| &body[at..])
        .expect("the node/lang arm is still the `None` runtime_server arm");
    let none_arm = &none_arm[..none_arm
        .find("        Some(server)")
        .unwrap_or(none_arm.len())];

    assert!(
        !none_arm.contains("volumes:"),
        "the snapshot runtimes now render a `volumes:` key of their own. If the \
         source is mounted there, `editor.rs` refuses an editor it should be \
         allowing — and it refuses it with a sentence about a snapshot that is \
         no longer true:\n{none_arm}"
    );
}

/// The overlay is layered, and layered before the dev server.
#[test]
fn the_editor_overlay_is_layered_into_every_compose_command() {
    let path = repo_root().join("src-tauri/src/runner.rs");
    let source = std::fs::read_to_string(&path).expect("runner.rs");

    assert!(
        source.contains("for file in compose_file_list(root, true)"),
        "compose_base_args no longer builds its `-f` arguments from the list \
         editor.rs reads. Two lists is the one failure this pair cannot see: \
         the IDE would open a container assembled from different files than the \
         ones StackVo starts."
    );

    // Named rather than called: `overlay_files` layers each overlay through one
    // macro, so the syncs appear as `crate::editor::sync` with no arguments.
    // What is being checked has not changed — that the overlay is in the chain,
    // and where in it.
    let editor = source.find("crate::editor::sync").expect(
        "the compose chain still layers the editor overlay — without it the volume \
         is written to a file nothing passes to compose",
    );
    let devserver = source
        .find("crate::devserver::sync")
        .expect("the dev server overlay is still layered");

    assert!(
        editor < devserver,
        "the editor overlay is layered after the dev server's, which is the one \
         that changes what the container runs. Adding a mount onto a service \
         already switched into another mode is the ordering this chain has \
         avoided everywhere else."
    );
}

/// The address is built in one place, and the front end is not it.
///
/// A pane that assembled `vscode-remote://attached-container+…` from a
/// container name it had on screen would work perfectly on the day it was
/// written. What it would not do is follow `editor.rs` — the leading slash
/// Docker puts on a name, the workdir a node project in dev mode has, the
/// refusal when the source is a snapshot — and the failure is a window that
/// opens onto the wrong thing rather than an error anybody sees.
#[test]
fn only_the_rust_side_builds_the_address() {
    let src = repo_root().join("src");
    let mut offenders = Vec::new();

    fn walk(dir: &Path, offenders: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, offenders);
                continue;
            }
            // `ipc.d.ts` is generated from `contracts/ipc.json` and carries
            // the contract's own prose as doc comments — the scheme appears in
            // it as a *description* of the boundary, which is the one place
            // outside Rust it is supposed to appear.
            if path.file_name().is_some_and(|n| n == "ipc.d.ts") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text.contains("attached-container") || text.contains("vscode-remote://") {
                offenders.push(path.display().to_string());
            }
        }
    }
    walk(&src, &mut offenders);

    assert!(
        offenders.is_empty(),
        "the address is derived on the front end as well as in editor.rs: {offenders:?}"
    );
}

/// The name in the address is the name the engine gives the container.
///
/// `editor.rs` is handed a container name and hexes it, and the only thing
/// that decides what that name *is* is `engine::container_name`. An
/// `attach_authority` that spelled the prefix itself would be a second copy of
/// it — and it would produce `stackvo-stackvo-shop` the first time somebody
/// passed a name that already carried one, which is the exact case
/// `container_name` is idempotent for.
#[test]
fn the_address_takes_its_container_name_from_the_engine() {
    let path = repo_root().join("src-tauri/src/editor.rs");
    let source = std::fs::read_to_string(&path).expect("editor.rs");
    let body = without_comments(&source);

    assert!(
        body.contains("crate::engine::container_name("),
        "editor::status no longer asks the engine what the container is called"
    );

    let at = body
        .find("pub fn attach_authority(")
        .expect("the address is still built by attach_authority");
    let region = &body[at..];
    let region = &region[..region.find("\npub fn ").unwrap_or(region.len())];

    assert!(
        !region.contains("stackvo"),
        "attach_authority spells the container prefix itself. It is handed a \
         name that engine::container_name has already built, and a second copy \
         of the prefix here doubles it on any name that arrives with one:\n{region}"
    );
}

/// The word the refusal turns on is the word the engine writes.
///
/// `editor.rs` decides "the source is really mounted" by comparing a mount's
/// kind to `bind`. That string is produced three hundred lines away in
/// `engine::inspect`, and for as long as it was produced with `format!("{:?}")`
/// it arrived quoted — so the comparison was false for every container, and
/// the pane refused every project with a sentence about a snapshot. It took a
/// running daemon to see it (`examples/editor_attach_probe.rs`); no unit test
/// could, because both sides of the comparison are written by hand in one.
#[test]
fn the_mount_kind_the_refusal_reads_is_the_one_the_engine_writes() {
    assert_eq!(stackvo_desktop_lib::engine::mount_kind("bind"), "bind");

    let path = repo_root().join("src-tauri/src/engine.rs");
    let source = without_comments(&std::fs::read_to_string(&path).expect("engine.rs"));

    let at = source
        .find("mounts: info")
        .expect("inspect still builds the mount table");
    let region = &source[at..];
    let region = &region[..region.find("            .collect(),").unwrap_or(region.len())];

    assert!(
        region.contains("mount_kind"),
        "the mount table no longer goes through engine::mount_kind:\n{region}"
    );
    assert!(
        !region.contains("{t:?}") && !region.contains("{:?}"),
        "the mount kind is being Debug-formatted again. Debug on a string puts \
         the quotes in, and editor.rs then refuses every container there is:\n{region}"
    );
}

/// Handing another tool this workspace's compose files only works because they
/// interpolate nothing.
///
/// `runner.rs` invokes compose with `--env-file <root>/.env`, and PhpStorm's
/// Dev Containers plugin invokes it with its own arguments and none of ours.
/// So the moment a `${...}` appears in generated output, the file StackVo hands
/// PhpStorm resolves to something different from the one StackVo runs — an
/// empty image name, a mount at `/`, a service that will not start — and
/// nothing on this side would notice.
///
/// Read from the differential fixture rather than from a live workspace: that
/// file is what `fixtures_differential.rs` holds the generator's output to, so
/// it is the output, and it exists on a machine that has never run StackVo.
#[test]
fn the_generated_compose_carries_nothing_for_another_tool_to_interpolate() {
    let path = repo_root().join("src-tauri/tests/fixtures/docker-compose.projects.yml");
    let text = std::fs::read_to_string(&path).expect("the differential fixture");

    assert!(
        !text.contains("${"),
        "the generated compose now interpolates a variable. editor.rs hands \
         these files to PhpStorm's Dev Containers, which runs compose without \
         this app's --env-file, so an interpolated value there is a different \
         file from the one StackVo runs:\n{path:?}"
    );
}

/// The devcontainer names the same compose files every other command uses.
///
/// A second list written here would be right on the day it was written and
/// wrong the first time an overlay was added — and the failure is quiet: the
/// IDE would attach to a container assembled from fewer files than the one
/// StackVo starts, so the mounts an overlay adds would simply not be there.
#[test]
fn the_devcontainer_takes_its_compose_list_from_the_runner() {
    let path = repo_root().join("src-tauri/src/editor.rs");
    let source = without_comments(&std::fs::read_to_string(&path).expect("editor.rs"));

    assert!(
        source.contains("crate::runner::compose_file_list("),
        "editor.rs builds its own list of compose files instead of reading the \
         one runner.rs assembles for every compose command this app runs"
    );
}
