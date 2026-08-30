//! Does a project actually come up, when Docker is actually running?
//!
//! Every other test in this repository answers a question about the tree: what
//! the generator renders, which arguments compose would be given, whether two
//! catalogues agree. None of them starts a container, and that gap has a shape
//! — a Dockerfile that stopped building, a compose file that stopped layering,
//! a supervisord configuration that starts nginx without php-fpm are all
//! invisible to a suite that never runs one. The first person to find out is
//! whoever presses the button.
//!
//! So this one does the whole thing: lays out a workspace, writes a project,
//! renders the generated files with the same [`write_generated`] the app and
//! the CLI both call, runs `docker compose up`, and asks the site for a page.
//!
//! ## Why it is off unless asked for
//!
//! It builds a PHP image from scratch — apt, extensions, Composer, Node — and
//! costs minutes rather than milliseconds. That belongs in a nightly, not in
//! front of a pull request, which is why it is opt-in through
//! `STACKVO_DOCKER_SMOKE=1` and skips loudly otherwise. A test that quietly
//! passed by doing nothing would be the same silence this file exists to end.
//!
//! ## Why it refuses to run beside a live stack
//!
//! The generated compose project is called `stackvo`, its containers are
//! `stackvo-*` and its network is `stackvo-net` — none of those are derived
//! from the workspace, so a second stack is not a second stack, it is the same
//! one. `docker compose up --remove-orphans` against somebody's running
//! installation would recreate their Traefik and remove the containers this
//! project does not know about, which is the developer's own machine.
//!
//! It therefore checks first and refuses, rather than skipping: a machine with
//! a stack on it is not a machine where this test has nothing to say, it is one
//! where running it would do damage. The refusal is the reason this test could
//! not be run on the machine it was written on, and it is why the check exists
//! rather than being left as a note.
//!
//! ## What each half proves
//!
//! * **The container half** — `docker exec … curl http://127.0.0.1/` — is the
//!   roadmap's own question: the image built, supervisord started nginx *and*
//!   php-fpm, the bind mount arrived, and `document_root` points at the
//!   directory the page is in. It needs no host port and no certificate.
//! * **The routing half** — an HTTPS request for the project's domain — is
//!   Traefik: the label the generator wrote became a router, and the router
//!   reaches the container. It needs a certificate, so the test makes a
//!   throwaway self-signed one; the CA is never installed anywhere and the
//!   client is told to accept it. `mkcert` is what a real installation uses and
//!   is not on a runner.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The switch. Anything other than `1` and this test says so and returns.
const ENABLE: &str = "STACKVO_DOCKER_SMOKE";

/// What it prints when it does nothing.
///
/// A constant rather than a sentence inside the `eprintln!`, because the
/// nightly job greps for it: a green job has to mean this test RAN, and the
/// only way it can tell is by looking for this line and failing when it is
/// there. Two copies of a sentence in two languages is exactly the drift
/// `workflow_parity.rs` exists to catch, so it holds this one too.
const SKIPPED: &str = "skipping: this test starts containers";

/// The project this test creates. Deliberately not a name anybody would use
/// for real work, because the teardown removes it.
const PROJECT: &str = "stackvo-smoke-probe";

/// How long the site is given to answer after compose reports the stack up.
///
/// A container is `running` the moment its entrypoint execs, which is before
/// supervisord has started nginx and well before php-fpm has a socket. Ninety
/// seconds is far beyond what that takes and far short of a hung job.
const READY_SECONDS: u64 = 90;

fn docker(args: &[&str]) -> std::process::Output {
    Command::new("docker")
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("could not run `docker {}`: {e}", args.join(" ")))
}

/// Every container Docker believes belongs to the `stackvo` compose project.
///
/// By label rather than by name prefix: the label is what compose itself keys
/// on when it decides what to recreate and what to remove as an orphan, so it
/// is the same question compose would ask.
fn live_stack() -> Vec<String> {
    let out = docker(&[
        "ps",
        "-a",
        "--filter",
        "label=com.docker.compose.project=stackvo",
        "--format",
        "{{.Names}}",
    ]);
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn compose_args(root: &Path) -> Vec<String> {
    // The app's own argument builder, not a second spelling of it. A test that
    // assembled its own `-f` list would go on passing after the real one
    // started layering a file this does not know about.
    stackvo_desktop_lib::runner::compose_base_args(root)
}

fn compose(root: &Path, extra: &[&str]) -> std::process::Output {
    let mut args = compose_args(root);
    args.extend(extra.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    docker(&refs)
}

/// A certificate for `*.loc`, valid for a day, trusted by nobody.
///
/// Traefik carries a default certificate of its own and would serve without
/// this, but the generated dynamic configuration names these two files, and a
/// TLS store pointing at files that are not there is a different thing being
/// tested than the one this file claims to test.
fn self_signed(dir: &Path) {
    std::fs::create_dir_all(dir).expect("could not make the certificate directory");
    let out = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-days",
            "1",
            "-subj",
            "/CN=*.loc",
            "-addext",
            "subjectAltName=DNS:*.loc,DNS:*.stackvo.loc",
            "-keyout",
        ])
        .arg(dir.join("stackvo-wildcard.key"))
        .arg("-out")
        .arg(dir.join("stackvo-wildcard.crt"))
        .output()
        .expect("openssl is required for the routing half — it is on every runner and on macOS");
    assert!(
        out.status.success(),
        "openssl could not make a certificate: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Lay out a workspace with one PHP project in it, and render the generated
/// files exactly as the app does.
///
/// `tag` keeps two tests in this binary out of each other's directory: cargo
/// runs them on threads of one process, so the process id alone is the same
/// name twice.
fn workspace(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("stackvo-smoke-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let project = root.join("projects").join(PROJECT);
    std::fs::create_dir_all(project.join("public")).expect("could not make the project directory");
    std::fs::create_dir_all(root.join("generated")).unwrap();
    std::fs::create_dir_all(root.join("logs")).unwrap();

    std::fs::write(
        project.join("stackvo.json"),
        format!(
            r#"{{
  "name": "{PROJECT}",
  "domain": "{PROJECT}.loc",
  "runtime": "php",
  "server": "nginx",
  "document_root": "public",
  "php": {{ "version": "8.4", "extensions": ["mbstring"] }}
}}
"#
        ),
    )
    .expect("could not write the manifest");

    // The page, and the assertion. `PHP_SAPI` rather than a literal string, so
    // a container serving the file as static text — nginx up, php-fpm not —
    // fails instead of passing with the source on screen.
    std::fs::write(
        project.join("public").join("index.php"),
        "<?php echo 'stackvo smoke ', PHP_SAPI;\n",
    )
    .expect("could not write the page");

    // The pointer file rather than `STACKVO_PROJECTS`: it is what an installed
    // workspace actually carries, and `point_at_projects` says in its own doc
    // comment that it is the only way the pointer is ever created. An
    // environment variable would also be process-global, which is a poor thing
    // for a test to reach for.
    stackvo_desktop_lib::workspace::point_at_projects(&root, &root.join("projects"))
        .expect("could not record where the projects are");

    let report = stackvo_desktop_lib::generator::write_generated(&root, "all", |_| {})
        .expect("the generator could not render this workspace");
    assert!(
        report
            .get("written")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0,
        "the generator wrote nothing: {report}"
    );

    root
}

/// What the page says, asked from inside the container.
fn from_inside() -> String {
    let out = docker(&[
        "exec",
        &format!("stackvo-{PROJECT}"),
        "curl",
        "-sf",
        "http://127.0.0.1/",
    ]);
    if !out.status.success() {
        return String::new();
    }
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[tokio::test]
async fn a_project_comes_up_and_serves_its_document_root() {
    if std::env::var(ENABLE).ok().as_deref() != Some("1") {
        eprintln!("{SKIPPED} — set {ENABLE}=1 to run it");
        return;
    }

    let live = live_stack();
    assert!(
        live.is_empty(),
        "refusing to run: a `stackvo` compose project already exists on this machine \
         ({}). This test uses the same project name, the same container names and the \
         same network, so compose would adopt that stack and `--remove-orphans` would \
         remove containers it does not know about. Take the stack down first.",
        live.join(", ")
    );

    let root = workspace("run");
    self_signed(&stackvo_desktop_lib::certs::cert_dir(&root));

    // External in every generated file, so it has to exist before compose runs.
    // Ignored if it is already there — created by an earlier run of this test,
    // which the refusal above has already established is not a live stack.
    let _ = docker(&["network", "create", "stackvo-net"]);

    let up = compose(
        &root,
        &[
            "--profile",
            "core",
            "--profile",
            "projects",
            "up",
            "-d",
            "--build",
        ],
    );

    // Torn down whatever happens below, including a panic in an assertion —
    // so the result is captured rather than asserted here.
    let outcome = if up.status.success() {
        serve_check().await
    } else {
        Err(format!(
            "compose could not bring the stack up:\n{}",
            String::from_utf8_lossy(&up.stderr)
        ))
    };

    let _ = compose(
        &root,
        &["--profile", "core", "--profile", "projects", "down", "-v"],
    );
    let _ = docker(&["network", "rm", "stackvo-net"]);
    let _ = std::fs::remove_dir_all(&root);

    if let Err(why) = outcome {
        panic!("{why}");
    }
}

/// Both halves, once the stack is up. Errors rather than panics, so the caller
/// can tear the stack down before failing.
async fn serve_check() -> Result<(), String> {
    // ---- the container half
    let mut body = String::new();
    for _ in 0..READY_SECONDS {
        body = from_inside();
        if !body.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    if !body.contains("stackvo smoke") {
        let logs = docker(&["logs", "--tail", "40", &format!("stackvo-{PROJECT}")]);
        return Err(format!(
            "the project container never served its document root in {READY_SECONDS}s.\n\
             last answer: {body:?}\n\
             container log:\n{}\n{}",
            String::from_utf8_lossy(&logs.stdout),
            String::from_utf8_lossy(&logs.stderr)
        ));
    }
    // php-fpm, not nginx handing back the source. `fpm-fcgi` is what the SAPI
    // is called when the file went through the pool.
    if !body.contains("fpm-fcgi") {
        return Err(format!("the page was served without php-fpm: {body:?}"));
    }

    // ---- the routing half
    let client = reqwest::Client::builder()
        // The certificate is the throwaway one this test just made; trusting it
        // anywhere would be a worse thing to do than not checking it.
        .danger_accept_invalid_certs(true)
        // What `curl --resolve` does: ask for the project's own domain without
        // needing a hosts entry, which is the one part of a real installation
        // that needs a password.
        .resolve(
            &format!("{PROJECT}.loc"),
            "127.0.0.1:443".parse().expect("a literal socket address"),
        )
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("could not build the client: {e}"))?;

    let url = format!("https://{PROJECT}.loc/");
    let mut last = String::new();
    for _ in 0..30 {
        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                if status.is_success() && text.contains("stackvo smoke") {
                    return Ok(());
                }
                last = format!("{status}: {text}");
            }
            Err(e) => last = e.to_string(),
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    let logs = docker(&["logs", "--tail", "40", "stackvo-traefik"]);
    Err(format!(
        "the container serves the page but Traefik does not route it.\n\
         last answer: {last}\n\
         traefik log:\n{}\n{}",
        String::from_utf8_lossy(&logs.stdout),
        String::from_utf8_lossy(&logs.stderr)
    ))
}

/// The half of the smoke test that needs no engine, run by the ordinary suite.
///
/// Everything above only happens at night, and a nightly is a slow place to
/// find out that the *setup* stopped working — a renamed generated file, a
/// manifest field the schema no longer accepts, a service that stopped being
/// emitted. None of that needs Docker to notice, so it is noticed here: if this
/// fails, the nightly was going to fail for a reason that has nothing to do
/// with containers.
#[test]
fn the_workspace_this_smoke_builds_renders_a_stack_to_start() {
    let root = workspace("render");

    let compose = std::fs::read_to_string(root.join("generated/docker-compose.projects.yml"))
        .expect("no projects compose file was rendered");

    // A service, from the manifest — the thing `up` would start.
    assert!(
        compose.contains(&format!("container_name: \"stackvo-{PROJECT}\"")),
        "no container for the project:\n{compose}"
    );
    // And the label the routing half of the smoke test asks Traefik about. A
    // rule for a different host is the one failure that looks like a network
    // problem at two in the morning.
    assert!(
        compose.contains(&format!("Host(`{PROJECT}.loc`)")),
        "the project is not routed at the domain the smoke test asks for:\n{compose}"
    );

    // The build context compose resolves, which is relative to the directory of
    // the first `-f` file rather than to the workspace root — so this asserts a
    // real path rather than the string in the file.
    assert!(
        root.join("generated/projects")
            .join(PROJECT)
            .join("Dockerfile")
            .is_file(),
        "the build context has no Dockerfile in it"
    );

    // The page the smoke test asks for has to be under `document_root`, or the
    // container would come up perfectly and answer 404.
    assert!(root
        .join("projects")
        .join(PROJECT)
        .join("public/index.php")
        .is_file());

    // Every file the runner will hand compose exists. This is the list that
    // grows: an overlay added to `compose_file_list` and not rendered here is a
    // `no such file` from compose, at night.
    for file in compose_args(&root)
        .windows(2)
        .filter(|pair| pair[0] == "-f")
        .map(|pair| PathBuf::from(&pair[1]))
    {
        assert!(
            file.is_file(),
            "compose would be handed {} and it is not there",
            file.display()
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}
