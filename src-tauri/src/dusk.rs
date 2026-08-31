//! Dusk: a browser in a container, and a certificate it will actually accept.
//!
//! ## The half Sail does not solve either
//!
//! The mechanism was already here. [`crate::imports`] recognises Sail's
//! `selenium` service **by name** when it brings a project across, and the
//! manifest has supported [`crate::sidecar`]s since W-01 — so a chromium
//! container can be expressed today with nothing new invented.
//!
//! What is missing is the other half, and it is the half nobody's `docker-
//! compose.yml` fixes: **Dusk drives a browser, and that browser has to open
//! `https://<domain>`.** The browser is inside a container, and that container
//! does not know this machine's certificate authority — the one [`crate::certs`]
//! owns and installed into *this* machine's stores. A test that falls over on a
//! certificate warning is a test failure that reads as an application bug, and
//! the developer goes looking in their code.
//!
//! So the work is two pieces and **neither works without the other**: the
//! sidecar, and the trust step.
//!
//! ## Three details that are read rather than remembered
//!
//! * On Apple Silicon the image is **`selenium/standalone-chromium`**, not
//!   `standalone-chrome`: Google publishes no arm64 Chrome, and the `chrome`
//!   image has no arm64 manifest. [`image_for`] picks by the architecture this
//!   build is running on.
//! * `.env.dusk.<environment>` is Dusk's own environment file, loaded in place
//!   of `.env` for the duration of a run — which is what makes it the right
//!   place for a driver URL that only means anything while the sidecar is up.
//! * Screenshots, console output and page source are written under
//!   `tests/Browser/`, which is inside the bind mount — so they land on the
//!   host and can be opened, rather than dying with the container.
//!
//! ## The database question is already answered
//!
//! Dusk hits a **real database**, not a transaction that gets rolled back. That
//! is the usual reason people will not run it locally, and this application
//! already has the answer: [`crate::worktree`] gives a branch a database of its
//! own. It is suggested rather than done — a card that moved somebody's test
//! suite onto a different database without being asked would be a card that had
//! decided something for them.
//!
//! ## The honest limit
//!
//! This is not *"we will run your tests"*. It is: make an environment in which
//! `stackvo artisan dusk` **can** run. What the suite does after that is the
//! suite's.
//!
//! Two things are deliberately not done. The project's `DuskTestCase` is not
//! edited — the driver, the window size and the Chrome flags are the
//! repository's code, and this app does not write PHP into somebody's tests.
//! And `/dev/shm` is not resized, because a declared sidecar has no such
//! option; where Chrome runs out of shared memory the answer is a flag in that
//! same `DuskTestCase`, which is why the help document names it instead of a
//! button pretending to fix it.

use serde::Serialize;
use std::path::Path;

/// The id the sidecar is declared under, and therefore half of its container
/// name: `stackvo-<project>-chromium`.
pub const SIDECAR_ID: &str = "chromium";

/// The port Selenium's standalone images listen on, and the path the WebDriver
/// endpoint answers at.
pub const DRIVER_PORT: u16 = 4444;

/// Dusk's own environment file, for the `local` environment.
///
/// The environment is not read from anywhere: `APP_ENV` in a project that has
/// not been deployed is `local`, and a file named for an environment this app
/// guessed would be a file Dusk never loads. Where a project uses another one,
/// the help document says to rename it — which is a sentence, not a wrong file.
pub const ENV_FILE: &str = ".env.dusk.local";

/// Where the CA is put inside the browser's container.
pub const CA_IN_CONTAINER: &str = "/usr/local/share/ca-certificates/stackvo-ca.crt";

/// Chromium reads its certificates from an NSS database, not from the system
/// bundle — which is why trusting the CA is two steps and not one.
pub const NSS_DB: &str = "sql:/home/seluser/.pki/nssdb";

/// The image tag this app pins.
///
/// Pinned rather than tracked, and it is the same rule the package catalogue
/// applies to everything else: an untagged image moves under somebody who
/// pulled it last month. It is written into the project's own `stackvo.json`,
/// so bumping it is an edit to a file the project owns rather than a wait for
/// this application to ship.
pub const TAG: &str = "4.27.0";

/// The Selenium image for the architecture this build runs on.
///
/// **`standalone-chromium` on arm64 and `standalone-chrome` elsewhere.** Google
/// publishes no arm64 build of Chrome, so the `chrome` image has no arm64
/// manifest at all — on an Apple Silicon machine it either refuses to start or
/// runs under emulation, and a browser under emulation is a test suite that
/// times out for a reason nobody will find.
pub fn image_for(arch: &str) -> String {
    let flavour = if arch == "aarch64" || arch == "arm64" {
        "standalone-chromium"
    } else {
        "standalone-chrome"
    };
    format!("selenium/{flavour}:{TAG}")
}

/// The image for *this* machine.
pub fn image() -> String {
    image_for(std::env::consts::ARCH)
}

/// The sidecar this app would add to `stackvo.json`.
///
/// No host port and no host path, because [`crate::sidecar`] allows neither —
/// and neither is wanted: the only thing that has to reach the browser is the
/// project's own container, one hop away on the same network.
pub fn sidecar() -> crate::sidecar::Sidecar {
    crate::sidecar::Sidecar {
        image: image(),
        about: "Chromium with a WebDriver endpoint, for Laravel Dusk.".to_string(),
        command: Vec::new(),
        env: Default::default(),
        volumes: Vec::new(),
    }
}

/// Where Dusk should send its WebDriver commands.
pub fn driver_url(project: &str) -> String {
    format!(
        "http://{}:{DRIVER_PORT}/wd/hub",
        crate::sidecar::container_name(project, SIDECAR_ID)
    )
}

/// The contents of `.env.dusk.local`.
///
/// Two variables and a comment saying where they came from. Nothing else: this
/// file overrides the project's `.env` for the length of a run, and a value
/// this app added on a guess would silently change what the suite tests.
pub fn env_file(project: &str, domain: &str) -> String {
    format!(
        "# Written by StackVo. Dusk loads this in place of .env for a run.\n\
         # The browser is a container on this project's network, so the driver\n\
         # URL is a container name rather than localhost.\n\
         APP_URL=https://{domain}\n\
         DUSK_DRIVER_URL={}\n",
        driver_url(project)
    )
}

// ------------------------------------------------------------- the trust step

/// One command in the trust step, and why it is there.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// Stable key; the UI holds the sentence.
    pub id: &'static str,
    /// argv for `docker`, without the program itself.
    pub args: Vec<String>,
    /// Whether the whole step may fail without the step after it being wrong.
    ///
    /// The NSS step is the one that matters to Chromium and the one most likely
    /// to be missing its tool, so its failure is reported rather than fatal —
    /// and reported as itself, not folded into "trust failed".
    pub optional: bool,
}

/// What has to happen inside the browser's container for `https://<domain>` to
/// load without a warning.
///
/// Three commands, and they are three rather than one because they fail
/// separately and a person needs to know which:
///
/// 1. put the CA in the container;
/// 2. add it to the system bundle, which is what `curl` and the JVM read;
/// 3. add it to the **NSS database**, which is what Chromium reads.
///
/// Run as root (`-u 0`) because the image runs as `seluser` and neither the
/// bundle nor the certificate directory is writable by it. This is `docker
/// exec` against a container this application declared, on this machine — not a
/// privilege it grants anybody else.
///
/// **It has to be re-run when the container is recreated.** That is a property
/// of writing into a container's writable layer, it is not hidden, and the pane
/// says it in the same place as the button.
pub fn trust_steps(project: &str, ca_host_path: &str) -> Vec<Step> {
    let container = crate::sidecar::container_name(project, SIDECAR_ID);
    let argv = |parts: &[&str]| parts.iter().map(|s| s.to_string()).collect::<Vec<String>>();

    vec![
        Step {
            id: "copy",
            args: {
                let mut args = argv(&["cp", ca_host_path]);
                args.push(format!("{container}:{CA_IN_CONTAINER}"));
                args
            },
            optional: false,
        },
        Step {
            id: "bundle",
            args: argv(&["exec", "-u", "0", &container, "update-ca-certificates"]),
            optional: false,
        },
        Step {
            id: "nss",
            // One `sh -c` because the tool is not in every image and the shell
            // is: a step that reports "certutil: not found" is a sentence
            // somebody can act on, and a `docker exec` that cannot start its
            // program is an error about docker instead. `2>&1` for the same
            // reason — the message is the useful half and it goes to stderr.
            args: argv(&[
                "exec",
                "-u",
                "0",
                &container,
                "sh",
                "-c",
                &format!("certutil -d {NSS_DB} -A -t C,, -n StackVo -i {CA_IN_CONTAINER} 2>&1"),
            ]),
            optional: true,
        },
    ]
}

// ----------------------------------------------------------------- the plan

/// What stands, and what would be done.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// The version `composer.lock` names for `laravel/dusk`. `None` and nothing
    /// below applies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed: Option<String>,
    /// The image this machine's architecture wants.
    pub image: String,
    /// `stackvo.json` already declares the sidecar.
    pub declared: bool,
    /// The container is up right now — which the trust step needs, because it
    /// writes into that container.
    pub running: bool,
    /// `.env.dusk.local` is already there. Never overwritten.
    pub env_file_present: bool,
    /// What that file would be given, so it can be read before it is written.
    pub env_file: String,
    /// Whether this project is a worktree with a database of its own — the
    /// answer to "Dusk writes to a real database".
    pub isolated_database: bool,
}

/// Read the plan. Touches nothing.
pub fn plan(
    dir: &Path,
    project: &str,
    domain: &str,
    deps: &[crate::deps::Dep],
    declared: bool,
    running: bool,
    isolated_database: bool,
) -> Plan {
    Plan {
        installed: deps
            .iter()
            .find(|d| d.ecosystem == crate::deps::Ecosystem::Packagist && d.name == "laravel/dusk")
            .map(|d| d.version.clone()),
        image: image(),
        declared,
        running,
        env_file_present: dir.join(ENV_FILE).is_file(),
        env_file: env_file(project, domain),
        isolated_database,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Apple Silicon gets chromium, and the reason is that no arm64 Chrome
    /// exists — not a preference.
    #[test]
    fn the_image_follows_the_architecture() {
        assert_eq!(
            image_for("aarch64"),
            format!("selenium/standalone-chromium:{TAG}")
        );
        assert_eq!(
            image_for("arm64"),
            format!("selenium/standalone-chromium:{TAG}")
        );
        assert_eq!(
            image_for("x86_64"),
            format!("selenium/standalone-chrome:{TAG}")
        );
        // Always a tag. An untagged image moves under whoever pulled it last
        // month, which is the one thing the package rules refuse everywhere
        // else in this repository.
        assert!(image().contains(':'));
        assert!(!image().ends_with(":latest"));
    }

    /// The driver URL names a container, not localhost — `localhost` inside the
    /// project's container is the project's container.
    #[test]
    fn the_driver_url_is_reachable_from_the_project_and_from_nowhere_else() {
        assert_eq!(
            driver_url("shop"),
            "http://stackvo-shop-chromium:4444/wd/hub"
        );

        let file = env_file("shop", "shop.loc");
        assert!(file.contains("APP_URL=https://shop.loc"));
        assert!(file.contains("DUSK_DRIVER_URL=http://stackvo-shop-chromium:4444/wd/hub"));
        // No host port anywhere: a sidecar has none, and two clones of one
        // repository would otherwise fight over 4444. Checked over the
        // assignments rather than the whole file, because the comment above
        // them is what explains why `localhost` would be wrong.
        assert!(!file
            .lines()
            .filter(|line| !line.starts_with('#'))
            .any(|line| line.contains("localhost") || line.contains("127.0.0.1")));
    }

    /// The sidecar this app would write is one the manifest's own rules accept.
    #[test]
    fn the_sidecar_declares_no_host_port_and_no_host_path() {
        let sidecar = sidecar();
        assert!(sidecar.volumes.is_empty());
        assert!(sidecar.command.is_empty());
        assert!(sidecar.image.starts_with("selenium/"));
    }

    /// Three steps, run as root, against this project's own container — and the
    /// NSS one is the optional one because it is the tool most likely to be
    /// absent.
    #[test]
    fn the_trust_step_puts_the_ca_where_chromium_reads_it() {
        let steps = trust_steps("shop", "/Users/x/rootCA.pem");
        assert_eq!(
            steps.iter().map(|s| s.id).collect::<Vec<_>>(),
            ["copy", "bundle", "nss"]
        );

        assert_eq!(
            steps[0].args,
            [
                "cp",
                "/Users/x/rootCA.pem",
                &format!("stackvo-shop-chromium:{CA_IN_CONTAINER}")
            ]
        );
        // Root, because the image runs as `seluser` and neither the bundle nor
        // the certificate directory is writable by it.
        assert!(steps[1].args.windows(2).any(|w| w == ["-u", "0"]));
        assert!(steps[2].args.iter().any(|a| a.contains("certutil")));
        assert!(steps[2].args.iter().any(|a| a.contains(NSS_DB)));

        assert_eq!(
            steps.iter().map(|s| s.optional).collect::<Vec<_>>(),
            [false, false, true]
        );
        // Every step names this project's own container and no other.
        for step in &steps {
            assert!(
                step.args
                    .iter()
                    .any(|a| a.contains("stackvo-shop-chromium")),
                "{step:?}"
            );
        }
    }

    /// A project without Dusk gets a plan that says so, and nothing else on it
    /// pretends to apply.
    #[test]
    fn a_project_without_dusk_is_reported_as_one() {
        let dir = std::env::temp_dir();
        let plan = plan(&dir, "shop", "shop.loc", &[], false, false, false);
        assert!(plan.installed.is_none());
        assert!(!plan.declared);
    }
}
