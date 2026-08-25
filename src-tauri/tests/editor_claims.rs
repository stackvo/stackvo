//! The two facts `editor.rs` reads out of files it does not own.
//!
//! §3 R-3 turns on a pair of claims about the generated Dockerfiles, and both
//! are the kind that stay true until somebody makes a reasonable change
//! somewhere else:
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

    let editor = source.find("crate::editor::sync(root)").expect(
        "compose_base_args still layers the editor overlay — without it the volume \
                 is written to a file nothing passes to compose",
    );
    let devserver = source
        .find("crate::devserver::sync(root)")
        .expect("the dev server overlay is still layered");

    assert!(
        editor < devserver,
        "the editor overlay is layered after the dev server's, which is the one \
         that changes what the container runs. Adding a mount onto a service \
         already switched into another mode is the ordering this chain has \
         avoided everywhere else."
    );
}
