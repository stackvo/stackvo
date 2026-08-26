//! The address, against a container that is actually running.
//!
//! `editor.rs`'s own tests settle the derivation: the hex is held against a
//! literal, the two spellings of a container name produce one address, and the
//! refusals carry their reasons. None of that opens a window, and none of it
//! has ever been shown a real mount table — which is the half this repository
//! has been caught by before: *a thing that does not run looks right.*
//!
//! So this asks a live daemon. For each container named on the command line it
//! prints what `readiness` decides, the address it derives, and — the part
//! worth reading — the JSON that address decodes back to, which is the object
//! VS Code itself parses.
//!
//!   cargo run --example editor_attach_probe -- parser.ajans
//!
//! With no arguments it probes every running `stackvo-` container, which is
//! the honest default: the interesting answers are the ones nobody chose.
//!
//! `STACKVO_ROOT=<dir>` also prints the PhpStorm half — the `devcontainer.json`
//! this app would write for that project — because that file names this
//! workspace's own compose files and there is no way to read it out of a pure
//! function.
use stackvo_desktop_lib::editor::{self, Readiness};
use stackvo_desktop_lib::engine;

/// Read back what VS Code will read, rather than trusting the string's shape.
fn decode(authority: &str) -> String {
    let Some((_, hex)) = authority.split_once('+') else {
        return "<no payload>".into();
    };
    let bytes: Result<Vec<u8>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect();
    match bytes {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => format!("<not hex: {e}>"),
    }
}

fn report(name: &str, runtime: &str, r: &Readiness) {
    println!("\n{name}  (runtime: {runtime})");
    println!("  running       {}", r.running);
    println!("  workdir       {}", r.workdir);
    println!("  source live   {}", r.source_live);
    println!("  server kept   {}", r.server_kept);
    println!("  libc          {:?}", r.libc);
    println!("  blockers      {:?}", r.blockers);
    println!("  caveats       {:?}", r.caveats);
    println!("  ATTACHABLE    {}", r.attachable);
    println!("  folder-uri    {}", r.folder_uri);
    println!("  url handler   {}", r.handler_url);

    let authority = editor::attach_authority(&r.container);
    println!("  decodes to    {}", decode(&authority));
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");

    let names: Vec<String> = std::env::args().skip(1).collect();
    let names = if names.is_empty() {
        let mut found: Vec<String> = runtime
            .block_on(engine::stackvo_containers())
            .map(|map| map.into_keys().collect())
            .unwrap_or_default();
        found.sort();
        found
    } else {
        names
    };

    if names.is_empty() {
        println!("nothing running to ask. Start a project, or name a container.");
        return;
    }

    // The other editor's half, when a workspace was named. Printed rather than
    // written: this is a probe, and writing into somebody's workspace to show
    // them what would be written is the one thing a probe must not do.
    if let Ok(root) = std::env::var("STACKVO_ROOT") {
        let root = std::path::PathBuf::from(root);
        for name in &names {
            let workdir = editor::PHP_WORKDIR;
            println!(
                "\nPhpStorm — {}\n{}",
                editor::jetbrains_path(&root, name).display(),
                editor::jetbrains_json(name, workdir, &compose_list(&root))
            );
        }
    }

    for name in names {
        let container = engine::container_name(&name);
        match runtime.block_on(engine::inspect(&name)) {
            Ok(details) => {
                // The runtime is guessed from the image rather than read from a
                // manifest, because this probe is pointed at whatever is
                // running and not at a workspace: `workdir_of` is what is being
                // measured, so the guess is stated and the answer is printed
                // beside the mount table it was judged against.
                let image = details.image.clone().unwrap_or_default();
                let guessed = if image.contains("node") {
                    "node"
                } else {
                    "php"
                };

                for mount in &details.mounts {
                    println!(
                        "  mount        {:?} {} -> {}",
                        mount.kind.as_deref().unwrap_or("?"),
                        mount.source.as_deref().unwrap_or("-"),
                        mount.destination
                    );
                }

                let r = editor::readiness(
                    &container,
                    guessed,
                    &image,
                    details.running,
                    &details.mounts,
                );
                report(&container, guessed, &r);
            }
            Err(e) => println!("\n{container}\n  could not be inspected: {}", e.message),
        }
    }
}

/// The compose files a devcontainer would name, without re-rendering anything.
fn compose_list(root: &std::path::Path) -> Vec<String> {
    stackvo_desktop_lib::runner::compose_file_list(root, false)
        .iter()
        .map(|p| p.display().to_string())
        .collect()
}
