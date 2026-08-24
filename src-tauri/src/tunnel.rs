//! A shareable public URL per project — webhook testing's missing answer.
//!
//! Stripe, GitHub and every other webhook sender needs to reach the site from
//! the internet, and `myapp.loc` does not exist there. The fix is a tunnel
//! client run as a sidecar container on the stack's own network: it dials out,
//! the provider hands back a public hostname, and traffic arrives at the
//! project container for as long as the sidecar runs.
//!
//! ## Why there is a table here rather than one image
//!
//! This started as cloudflared and nothing else, and the shape of "one
//! provider" had leaked everywhere: the image was a constant, the URL was
//! recognised by `.trycloudflare.com`, and the pane said "no account needed"
//! as a fact about the feature rather than about Cloudflare. But the choice
//! between tunnel providers is a real one and it is not ours to make — a quick
//! tunnel's address changes on every start, which is fine for "did the webhook
//! arrive" and useless for an OAuth redirect URI somebody has to register in a
//! dashboard. The providers that hold an address still are exactly the ones
//! that want an account.
//!
//! So a provider is data: an image, the arguments that point it at a
//! container, the shape of the URL it prints, and whether it needs a token.
//! Adding the ninth is a row, not a branch.
//!
//! The ninth is here now, and it was added as a row: `cloudflare_named` is the
//! same client as `cloudflare`, told to run a tunnel that already exists
//! instead of inventing one — because the two halves of that trade, "an
//! address in ten seconds" and "the same address tomorrow", are what B-7 is
//! about.
//!
//! ## The second question about a link you just pasted
//!
//! Who else can open it, and will it still be this address tomorrow. Neither
//! is a provider setting here: authentication belongs to
//! [`crate::tunnelid`]'s guard, which every provider reaches identically, and
//! a reserved name is a per-provider flag whose result is **checked** rather
//! than assumed — [`TunnelStatus::reserved_honoured`], and the measurement
//! behind it, are in that module's note.
//!
//! ## What each sidecar is pointed at
//!
//! The project container directly, never Traefik: with SSL on, every project
//! router listens on `websecure` only, and a public visitor cannot complete a
//! TLS handshake against a hostname that exists in no DNS. The container's
//! internal port is plain HTTP and derived the same way the generator derives
//! the Traefik `loadbalancer.server.port` label — node projects on their
//! manifest port, Swoole on its own 8000, every other PHP server on 80.
//!
//! Three providers can also present the project's local domain as the `Host`
//! header, so name-based vhosts and framework URL checks behave as they do
//! locally. The SSH-based ones cannot: there the application sees the tunnel's
//! own hostname, which is [`Provider::rewrites_host`] and is said on screen
//! rather than left to be discovered by a redirect loop.
//!
//! ## The URL is read from the log, and so is the failure
//!
//! Nothing about a live tunnel is stored. The URL appears in the sidecar's log
//! when the provider assigns it, and reading it back on every status call
//! means the answer is always what is actually live — an app restart, a
//! container restart, a crashed tunnel all stay truthful for free.
//!
//! The same read is what makes a rejected token sayable. Four of these
//! providers can fail on authentication, and that is the single most likely
//! failure the feature has; the sidecar is deliberately **not** `--rm` so its
//! log survives the exit, exactly as [`crate::stripe`] learned to do.
//!
//! ## What was verified on the machine this was written on, and what was not
//!
//! `examples/tunnel_probe.rs` runs **every** provider in this table as a real
//! container against a throwaway nginx, using this module's own `run_args`,
//! and it is repeatable. What it establishes for all eight: the image runs,
//! the client is the program inside it, and the arguments built here are
//! arguments that client accepts — a removed flag, a renamed subcommand or a
//! target shape it refuses all surface there rather than as a pane that spins.
//!
//! For the four anonymous providers it goes further: a real public URL comes
//! back and [`find_url`] finds it in the client's own banner. Three of them
//! were also fetched back through that URL and returned the target's page.
//!
//! For the four that need an account, the probe hands each one a deliberately
//! invalid token and checks that the refusal is a sentence [`find_failure`]
//! recognises — ngrok's `ERR_NGROK_105`, Tailscale's `invalid key`, zrok's
//! `401 enableUnauthorized`, LocalXpose's `unauthenticated access`. So the
//! untested step for those four is exactly one: what the provider does with a
//! *valid* token. [`Provider::verified`] means "a tunnel through this one
//! carried traffic from here", and the pane says which four it is true of
//! rather than implying it of all eight.
//!
//! Five findings changed this code, and every one of them came from watching a
//! client rather than from reading about it:
//!
//! * both SSH providers link their own dashboard directly above the tunnel
//!   they opened, so suffix lists written from documentation would have handed
//!   `admin.localhost.run` and `dashboard.pinggy.io` out as the address of
//!   somebody's application;
//! * `tailscale funnel` serves "a service running on the local machine", so
//!   the sidecar joins the project container's network namespace and the
//!   target is a port number rather than a URL the docs never promise;
//! * localtunnel's `--host` is the *tunnel server*, not the target — pointed
//!   at the project it produced a client that sat in silence;
//! * LocalXpose can present the local domain after all, through the
//!   `--request-header host:` plugin in its own help text;
//! * ngrok's `--log` defaults to `false`, so without `--log=stdout` the agent
//!   works perfectly and prints the URL nowhere.

use crate::error::{Code, Error, Result};
use serde::Serialize;

/// Sidecar containers are `stackvo-tunnel-<project>`; the id handed to
/// `engine::*` (which prefixes `stackvo-` itself) is `tunnel-<project>`.
pub const ID_PREFIX: &str = "tunnel-";

/// The label the sidecar carries its provider in.
///
/// A label rather than the container name, which has to stay
/// `tunnel-<project>` for stop and status to find it, and rather than the
/// image, which does not identify the two SSH providers apart — they run the
/// same client. It is also the only way to say *which* provider is connecting
/// before any URL exists to infer it from.
pub const PROVIDER_LABEL: &str = "stackvo.tunnel.provider";

/// Whether this sidecar forwards through [`crate::tunnelid`]'s guard.
///
/// On the sidecar rather than inferred from a running guard: the question the
/// pane asks is "is the link I handed out asking for a password", and only the
/// container that was actually pointed somewhere can answer it. A guard
/// started after the tunnel protects nothing.
pub const GUARDED_LABEL: &str = "stackvo.tunnel.guarded";

/// The name this sidecar asked its provider to keep, where one was asked for.
///
/// Read back rather than remembered, for the reason the whole module reads its
/// state off the engine: a reserved name that was requested and not granted is
/// the failure this label exists to make visible, and it can only be seen by
/// comparing what was asked with what the client printed.
pub const RESERVED_LABEL: &str = "stackvo.tunnel.reserved";

/// What a provider is able to keep, for the providers that keep anything.
///
/// The distinction that matters is [`Self::kind`]: ngrok reserves a whole
/// hostname (`shop.ngrok-free.app`) and localtunnel one label of one (`shop`),
/// and a field that asked for the wrong one of those costs an image pull and a
/// failed start to discover.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reserved {
    /// The provider's own word for it: `subdomain`, `domain`, `hostname` or
    /// `name`. Also the translation key the field's label is looked up under.
    pub kind: &'static str,
    /// Whether the field takes a whole domain rather than one label.
    pub dotted: bool,
    /// Whether the client still prints the address it ended up on.
    ///
    /// False for exactly one provider: a Cloudflare named tunnel is routed by
    /// a hostname configured in Cloudflare's own dashboard, and `cloudflared`
    /// never learns it — so the address can only be the one the user typed
    /// here, and it cannot be checked against anything.
    pub in_log: bool,
}

/// One tunnel provider, as data.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: &'static str,
    pub image: &'static str,
    /// A public URL without an account. The three that have it are the reason
    /// this feature could ship with no setup screen at all.
    pub anonymous: bool,
    /// The environment variable the token reaches the container as, for the
    /// providers that need one.
    ///
    /// An environment variable and never an argument: `docker inspect`, `ps`
    /// on the host and this app's own operation console all print the argv.
    pub token_env: Option<&'static str>,
    /// Hostname endings this provider's URLs have. Used to pick the URL out of
    /// a log that also contains documentation links and dashboard addresses.
    pub url_suffixes: &'static [&'static str],
    /// Whether the sidecar presents the project's local domain as `Host`.
    pub rewrites_host: bool,
    /// The free tier's session cap in minutes, where the provider enforces one
    /// — a tunnel that dies after an hour is a fact somebody needs before they
    /// paste the URL into a dashboard, not after.
    pub session_minutes: Option<u32>,
    /// Whether a tunnel through this provider carried traffic from here.
    ///
    /// Not the same as "the invocation is checked": `examples/tunnel_probe.rs`
    /// runs all eight and proves every client accepts its arguments. This is
    /// the stronger claim, and it is true only of the ones no account stood in
    /// the way of. See the module note.
    pub verified: bool,
    /// What this provider can be asked to keep between starts, if anything.
    ///
    /// `None` is the honest answer for three of them and the reason the field
    /// exists: a quick tunnel's address is new on every start, and the pane
    /// has to be able to say that in front of the field rather than after
    /// somebody has registered one in a dashboard.
    pub reserved: Option<Reserved>,
    /// Whether the sidecar runs inside the project container's network
    /// namespace rather than beside it on the stack network.
    ///
    /// True for exactly one provider, and for a reason its own help text
    /// gives: `tailscale funnel` serves a service on the *local machine*, so
    /// the way to point it at another container is to be that container's
    /// network rather than to bet on an undocumented remote target.
    pub shares_project_netns: bool,
}

/// The providers, in the order the picker offers them: the anonymous ones
/// first, because they are the ones that work in the next ten seconds.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "cloudflare",
        image: "cloudflare/cloudflared:latest",
        anonymous: true,
        token_env: None,
        url_suffixes: &[".trycloudflare.com"],
        rewrites_host: true,
        session_minutes: None,
        verified: true,
        // A quick tunnel is quick because nobody registered anything, and it
        // holds nothing: the four words change on every start. The named
        // Cloudflare tunnel below is the same client answering the other half
        // of that trade.
        reserved: None,
        shares_project_netns: false,
    },
    Provider {
        id: "localhost_run",
        image: SSH_IMAGE,
        anonymous: true,
        token_env: None,
        // `.lhr.life` alone, and that is a measured decision rather than an
        // omission: the client's own welcome banner links
        // `https://admin.localhost.run/` and `https://localhost.run/docs/`
        // three lines above the tunnel it just opened, so a `.localhost.run`
        // suffix here reports the dashboard as the public address of somebody's
        // application. The banner is in the test below, verbatim.
        url_suffixes: &[".lhr.life"],
        rewrites_host: false,
        session_minutes: None,
        verified: true,
        // The client authenticates as `nokey@`, and a kept address is what a
        // registered key buys. Anonymous means new on every start, and saying
        // so is better than a field that never works.
        reserved: None,
        shares_project_netns: false,
    },
    Provider {
        id: "pinggy",
        image: SSH_IMAGE,
        anonymous: true,
        token_env: None,
        // Measured, not guessed, and the guess was wrong twice: a free tunnel
        // comes back as `*.free.pinggy.net` and `*.run.pinggy-free.link`, and
        // the line above both of them advertises `https://dashboard.pinggy.io`
        // — so `.pinggy.io` here would hand out Pinggy's upgrade page as the
        // address of somebody's application. The real log is in the tests.
        url_suffixes: &[".pinggy.net", ".pinggy-free.link", ".pinggy.link"],
        rewrites_host: false,
        // The free tier drops the session; the tunnel simply ends and the pane
        // has to be able to say why rather than reporting a crash.
        session_minutes: Some(60),
        verified: true,
        // A kept subdomain is a Pro token, and the token travels in the SSH
        // user name rather than an environment variable — a different shape
        // from every other provider here, and not one worth inventing a
        // second token field for. Anonymous sessions get a new address.
        reserved: None,
        shares_project_netns: false,
    },
    Provider {
        id: "localtunnel",
        image: "node:22-alpine",
        anonymous: true,
        token_env: None,
        url_suffixes: &[".loca.lt"],
        // Its one host flag is the target as well, and the target has to be
        // the container — so the application sees `stackvo-<project>` as Host
        // rather than its own domain. Measured, not assumed.
        rewrites_host: false,
        session_minutes: None,
        verified: true,
        // MEASURED, and it is the only free one: `--subdomain stackvo-probe-11959`
        // came back as exactly that address, and again ninety seconds later.
        // Started immediately after the previous tunnel closed, the same
        // request came back as `bitter-bulldog-88.loca.lt` with no error at
        // all — which is why a granted name is checked rather than assumed.
        reserved: Some(Reserved {
            kind: "subdomain",
            dotted: false,
            in_log: true,
        }),
        shares_project_netns: false,
    },
    Provider {
        // The same client as the row above, answering the other half of the
        // trade it makes. A quick tunnel is anonymous and its address is new
        // every time; a *named* tunnel is a tunnel created in Cloudflare's
        // dashboard, routed at a hostname on a domain somebody owns, and it is
        // the same address for as long as it exists.
        //
        // The ninth row, and it is a row: the module note said adding a
        // provider should be data rather than a branch, and B-7 is where that
        // was collected on.
        id: "cloudflare_named",
        image: "cloudflare/cloudflared:latest",
        anonymous: false,
        // MEASURED: `cloudflared` reads this variable by itself — the same
        // invocation with the token only in the environment answered `Provided
        // Tunnel token is not valid.`, which means it had read it. So the
        // token never has to appear as an argument.
        token_env: Some("TUNNEL_TOKEN"),
        // Deliberately empty, and the only row where that is right: the
        // hostname is whatever was configured in Cloudflare, on a domain this
        // app has never heard of, and `cloudflared` does not print it. The
        // address comes from `Reserved`, which is why `in_log` is false.
        url_suffixes: &[],
        rewrites_host: true,
        session_minutes: None,
        verified: false,
        reserved: Some(Reserved {
            kind: "hostname",
            dotted: true,
            in_log: false,
        }),
        shares_project_netns: false,
    },
    Provider {
        id: "ngrok",
        image: "ngrok/ngrok:latest",
        anonymous: false,
        token_env: Some("NGROK_AUTHTOKEN"),
        url_suffixes: &[
            ".ngrok-free.app",
            ".ngrok-free.dev",
            ".ngrok.app",
            ".ngrok.io",
        ],
        rewrites_host: true,
        session_minutes: None,
        verified: false,
        // The free plan includes one static domain, which is the whole reason
        // somebody sets ngrok up at all. `--url` and not `--domain`: measured,
        // the agent answers `Flag --domain has been deprecated, use --url
        // instead` and then honours it.
        reserved: Some(Reserved {
            kind: "domain",
            dotted: true,
            in_log: true,
        }),
        shares_project_netns: false,
    },
    Provider {
        id: "tailscale",
        image: "tailscale/tailscale:latest",
        anonymous: false,
        token_env: Some("TS_AUTHKEY"),
        url_suffixes: &[".ts.net"],
        rewrites_host: false,
        session_minutes: None,
        verified: false,
        // The one provider whose address was already stable: the funnel is
        // published at `<hostname>.<tailnet>.ts.net`, and the hostname is this
        // sidecar's to choose. Left empty it is `stackvo-<project>`, which is
        // what the invocation has always sent.
        reserved: Some(Reserved {
            kind: "hostname",
            dotted: false,
            in_log: true,
        }),
        shares_project_netns: true,
    },
    Provider {
        id: "zrok",
        image: "openziti/zrok:latest",
        anonymous: false,
        token_env: Some("ZROK_TOKEN"),
        url_suffixes: &[".share.zrok.io", ".zrok.io"],
        rewrites_host: false,
        session_minutes: None,
        verified: false,
        // A reserved share, by the vendor's own two-step design: reserve a
        // unique name once, then share it under that name from then on.
        reserved: Some(Reserved {
            kind: "name",
            dotted: false,
            in_log: true,
        }),
        shares_project_netns: false,
    },
    Provider {
        id: "localxpose",
        image: "localxpose/localxpose:latest",
        anonymous: false,
        token_env: Some("ACCESS_TOKEN"),
        url_suffixes: &[".loclx.io"],
        // Its own header plugin does it — `--request-header host:myapp.com`,
        // read out of the client's help text rather than assumed.
        rewrites_host: true,
        session_minutes: None,
        verified: false,
        // Its own `--subdomain`, from the client's help text. Not measured —
        // nobody here has an account — which is what `verified` already says.
        reserved: Some(Reserved {
            kind: "subdomain",
            dotted: false,
            in_log: true,
        }),
        shares_project_netns: false,
    },
];

/// The provider used when a caller names none — today's behaviour, unchanged.
pub const DEFAULT_PROVIDER: &str = "cloudflare";

/// One SSH client for both SSH-based providers.
///
/// `localhost.run` and Pinggy are the same program with different arguments,
/// so they share an image rather than each pulling their own — and it is a
/// client image rather than a distribution plus `apk add`, because a sidecar
/// that installs a package at start is a sidecar that stops working the day
/// the mirror is down.
const SSH_IMAGE: &str = "kroniak/ssh-client:latest";

/// The SSH options both tunnels need, and why each one is there.
///
/// * host-key checking off with no known-hosts file: the container is new
///   every time, so there is no file to have trusted the key in, and a prompt
///   in a detached container is a hang.
/// * a keepalive, because the far end drops an idle forward.
/// * `ExitOnForwardFailure`, so a refused forward ends the container with the
///   reason in its log rather than leaving a connected SSH session forwarding
///   nothing.
const SSH_OPTS: &[&str] = &[
    "-o",
    "StrictHostKeyChecking=no",
    "-o",
    "UserKnownHostsFile=/dev/null",
    "-o",
    "ServerAliveInterval=30",
    "-o",
    "ExitOnForwardFailure=yes",
];

/// Look a provider up by id, refusing an unknown one rather than falling back
/// to the default: a typo that silently opened a different provider's tunnel
/// would be a URL somebody pastes into a dashboard believing it is elsewhere.
pub fn provider(id: &str) -> Result<&'static Provider> {
    PROVIDERS
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| Error::not_found(format!("tunnel provider {id}")))
}

/// The keystore entry a provider's token is kept under.
///
/// Per provider and not per project, which is the opposite of
/// [`crate::stripe::secret_name`] and for a reason that is not an
/// inconsistency: a Stripe key is an *account's*, and two projects wired to
/// one account silently cross-post events. A tunnel token is a *machine's* —
/// the same ngrok authtoken is meant to be the one this computer uses — and
/// making somebody paste it again per project would be a chore with no
/// question behind it.
pub fn secret_name(provider: &str) -> String {
    format!("tunnel-token:{provider}")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub project: String,
    pub running: bool,
    /// The assigned public URL, once the provider has printed it. `None` while
    /// the sidecar is still connecting — the UI polls until it appears.
    pub url: Option<String>,
    pub container: String,
    /// Which provider this sidecar is, read from its label. `None` for a
    /// container started before the label existed, which is a cloudflared one.
    pub provider: Option<String>,
    /// The client's own complaint, when it failed. Read from the log rather
    /// than guessed at: "that token is not valid", "the free session ended"
    /// and "there is no network" are three different problems with three
    /// different fixes, and they look identical from the outside.
    pub failure: Option<String>,
    /// Whether this link asks for a password — whether the sidecar forwards
    /// through [`crate::tunnelid`]'s guard rather than straight at the
    /// project.
    ///
    /// Read off the sidecar's own label, so it describes the tunnel that is
    /// actually running rather than what the keystore holds now: switching
    /// authentication on does not protect a link that was handed out before.
    pub guarded: bool,
    /// The address this tunnel asked its provider to keep, where it asked.
    pub reserved: Option<String>,
    /// Whether the address that came back is the one that was asked for.
    ///
    /// `None` when nothing was reserved, or when there is no URL yet to
    /// compare. `Some(false)` is the measured failure this field exists for: a
    /// provider that quietly assigns a different name, leaving a tunnel that
    /// works and a dashboard entry that points nowhere.
    pub reserved_honoured: Option<bool>,
}

/// A provider and whether this machine could use it right now.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    #[serde(flatten)]
    pub provider: &'static Provider,
    /// Whether a token is in the keystore for it. Never the token.
    pub has_token: bool,
}

/// The engine-facing id of a project's sidecar.
pub fn container_id(project: &str) -> String {
    format!("{ID_PREFIX}{project}")
}

/// The port the project's own container serves HTTP on.
///
/// Mirrors `generator::render_compose_service`, which writes the same number
/// into the Traefik `loadbalancer.server.port` label — the generator is the
/// authority, this is its arithmetic repeated on the same manifest.
pub fn internal_port(manifest: &crate::manifest::Manifest) -> u16 {
    if manifest.runtime == "node" {
        return manifest.node.as_ref().map(|n| n.port).unwrap_or(3000);
    }
    match manifest.server.as_deref() {
        Some("swoole") => 8000,
        _ => 80,
    }
}

/// The first URL in a log that belongs to this provider.
///
/// Matched on the hostname's ending rather than on the sentence around it:
/// every one of these clients has reworded its banner across releases, and
/// three of them print a documentation link in the same paragraph as the
/// tunnel. `None` when the provider has not assigned one yet, which is a
/// state the UI shows as connecting rather than as an absent tunnel.
pub fn find_url(provider: &Provider, log: &str) -> Option<String> {
    for line in log.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("https://") {
            let candidate: String = rest[start..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '.' | '-'))
                .collect();
            // The host, without the scheme, any path, and any trailing
            // punctuation the client wrapped its banner in.
            let host = candidate
                .trim_start_matches("https://")
                .split('/')
                .next()
                .unwrap_or_default()
                .trim_end_matches('.');
            if provider.url_suffixes.iter().any(|s| host.ends_with(s)) {
                return Some(format!("https://{host}"));
            }
            rest = &rest[start + "https://".len()..];
        }
    }
    None
}

/// The client's own failure line, if it printed one.
///
/// Matched on what the client says rather than on an exit code, for the reason
/// [`crate::stripe::find_failure`] is: a long-running sidecar that failed is
/// usually still sitting there, and by the time anything reads it there is no
/// status to read. The needles are the vocabulary of the four failures that
/// actually happen — a rejected token, a refused forward, an expired free
/// session, and no network at all.
pub fn find_failure(log: &str) -> Option<String> {
    // Every needle below except the last three was read off a real client
    // being handed a deliberately invalid token: ngrok answers
    // `authentication failed … ERR_NGROK_105`, Tailscale `backend error:
    // invalid key: API key does not exist`, zrok `[POST /enable][401]
    // enableUnauthorized`, LocalXpose `Error: unauthenticated access`.
    // `examples/tunnel_probe.rs` is the run, and it is repeatable.
    const NEEDLES: &[&str] = &[
        "authentication failed",
        "invalid token",
        // Cloudflare's named tunnel, MEASURED against an invalid token:
        // `Provided Tunnel token is not valid.` — a refusal none of the
        // needles below recognised, which would have left the pane spinning
        // on the one provider whose token is hardest to get right.
        "is not valid",
        "invalid key",
        "unauthorized",
        "unauthenticated",
        "auth token",
        "authtoken",
        "not authorized",
        "permission denied",
        "backend error",
        "session ended",
        "tunnel session failed",
        "failed to connect",
        "connection refused",
        "could not resolve",
        "remote port forwarding failed",
    ];

    for line in log.lines() {
        let lowered = line.to_ascii_lowercase();
        if NEEDLES.iter().any(|n| lowered.contains(n))
            || lowered.starts_with("error")
            || lowered.contains("err ")
        {
            // A cloudflared line that merely *mentions* the word in a URL is
            // not a failure; a line with no letters after the marker is not a
            // sentence anybody can act on.
            let trimmed = line.trim();
            if trimmed.len() > 8 {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Every tunnel sidecar the engine knows about, with its URL where one has
/// been assigned and its failure where it printed one.
pub async fn status_all() -> Result<Vec<TunnelStatus>> {
    use futures_util::StreamExt;

    let containers = crate::engine::stackvo_containers().await?;
    let mut out = Vec::new();

    for (id, info) in containers {
        let Some(project) = id.strip_prefix(ID_PREFIX) else {
            continue;
        };
        // A guard's id begins with the tunnel prefix too, and its label is the
        // only thing that tells them apart for certain: a project genuinely
        // named `guard-shop` would otherwise appear here as a tunnel nobody
        // opened, on a project that does not exist.
        if info.labels.contains_key(crate::tunnelid::GUARD_LABEL) {
            continue;
        }

        let provider_id = info.labels.get(PROVIDER_LABEL).cloned();
        let known = provider_id
            .as_deref()
            .and_then(|id| PROVIDERS.iter().find(|p| p.id == id));

        // Read whether it is running or not: a container that exited holds the
        // reason it exited, and that is the whole point of not using `--rm`.
        // The URL lives in the first lines the sidecar ever printed, so a
        // bounded tail-from-start read is enough; follow=false ends on its own.
        let (url, failure) = match crate::engine::logs_stream(&id, 200, false) {
            Ok(stream) => {
                let lines: Vec<String> = stream.map(|l| l.text).collect().await;
                let log = lines.join("\n");
                // An unlabelled container predates the picker and is
                // cloudflared's; its suffix is the one that will match.
                let p = known.unwrap_or(&PROVIDERS[0]);
                (
                    if info.running {
                        find_url(p, &log)
                    } else {
                        None
                    },
                    find_failure(&log),
                )
            }
            Err(_) => (None, None),
        };

        let reserved = info.labels.get(RESERVED_LABEL).cloned();
        // The address a named Cloudflare tunnel serves is in Cloudflare's
        // configuration and nowhere in this client's output, so for that one
        // provider the reserved name IS the address — and it is shown only
        // while the sidecar is actually up, like every other URL here.
        let url = match (&url, &reserved) {
            (None, Some(name))
                if info.running && known.and_then(|p| p.reserved).is_some_and(|r| !r.in_log) =>
            {
                Some(format!("https://{name}"))
            }
            _ => url,
        };

        // Asked for, and granted? Only answerable once there is an address,
        // and only meaningful for a provider that prints one.
        let reserved_honoured = match (&reserved, &url) {
            (Some(name), Some(url)) if known.and_then(|p| p.reserved).is_none_or(|r| r.in_log) => {
                Some(crate::tunnelid::honoured(url, name))
            }
            _ => None,
        };

        out.push(TunnelStatus {
            project: project.to_string(),
            running: info.running,
            url,
            container: info.name,
            provider: provider_id,
            failure,
            guarded: info.labels.get(GUARDED_LABEL).map(String::as_str) == Some("true"),
            reserved,
            reserved_honoured,
        });
    }

    out.sort_by(|a, b| a.project.cmp(&b.project));
    Ok(out)
}

/// Every provider, with whether this machine holds a token for it.
pub fn providers() -> Vec<ProviderStatus> {
    PROVIDERS
        .iter()
        .map(|p| ProviderStatus {
            provider: p,
            has_token: p.token_env.is_some()
                && crate::secrets::read(&secret_name(p.id))
                    .ok()
                    .flatten()
                    .is_some(),
        })
        .collect()
}

/// Everything about one start that is not the provider.
///
/// A struct rather than five positional arguments, and it earned that when
/// B-7 added the sixth and seventh: `run_args(p, "shop", Some("shop.loc"), 80,
/// net, false, None)` is a line where two neighbouring booleans can be swapped
/// and every test still passes.
#[derive(Debug, Clone)]
pub struct Plan<'a> {
    pub project: &'a str,
    /// The project's local domain, for the providers that can present it.
    pub domain: Option<&'a str>,
    /// The port the **project's** container serves on. What the sidecar is
    /// actually pointed at is [`Self::guard`]'s business.
    pub port: u16,
    pub network: &'a str,
    /// The address the provider is asked to keep, for the providers that can.
    pub reserved: Option<&'a str>,
    /// The guard container in front of the project, when authentication is on.
    ///
    /// One field and not two — a name and a flag saying whether to use it —
    /// because the pair can disagree, and the disagreement is the worst
    /// outcome this feature has: a link that asks for no password while the
    /// pane reports that it does.
    pub guard: Option<&'a str>,
}

impl<'a> Plan<'a> {
    /// A tunnel straight to the project's own container — no guard, no
    /// reserved name. What every start was before B-7.
    pub fn direct(project: &'a str, domain: Option<&'a str>, port: u16, network: &'a str) -> Self {
        Self {
            project,
            domain,
            port,
            network,
            reserved: None,
            guard: None,
        }
    }
}

/// The `docker run` invocation for one project's sidecar.
///
/// Returned as arguments rather than executed here so the caller can drive it
/// through `runner::run_operation` — the first start pulls the image, which
/// can take minutes and belongs in the operation console, not behind a frozen
/// button.
///
/// No `--rm`, and that is a correction rather than an omission. Cloudflared's
/// sidecar had it, and with a token-bearing provider it destroys the only
/// evidence there is: a rejected token makes the client print its complaint
/// and exit, `--rm` takes the container and its log away, and the pane then
/// shows no tunnel, no error and no reason for the likeliest failure the
/// feature has. `tunnel_start` clears the old container before starting a new
/// one and `tunnel_stop` removes it.
pub fn run_args(provider: &Provider, plan: &Plan) -> Vec<String> {
    let project = plan.project;
    let domain = plan.domain;
    let network = plan.network;
    // The guard decides both halves of the target at once. Derived here rather
    // than passed in, so no caller can point a sidecar at a guard on the
    // project's own port — which would be a tunnel that bypasses the password
    // it just switched on.
    let (host, port) = match plan.guard {
        Some(guard) => (guard.to_string(), crate::tunnelid::GUARD_PORT),
        None => (crate::engine::container_name(project), plan.port),
    };
    let target = format!("http://{host}:{port}");
    // Only where the provider has somewhere to put it: a name sent to a
    // provider that keeps nothing is a flag its client would refuse.
    let reserved = plan.reserved.filter(|_| provider.reserved.is_some());

    let mut args: Vec<String> = vec![
        "run".into(),
        "-d".into(),
        "--name".into(),
        format!("stackvo-{}", container_id(project)),
        "--network".into(),
        // Almost every provider dials out from the stack's network and reaches
        // the project by container name. Tailscale is the exception and it is
        // not a preference: `tailscale funnel` serves "a service running on the
        // local machine", so its sidecar joins the project container's own
        // network namespace and the target becomes a plain port number. See
        // its arm below.
        if provider.shares_project_netns {
            format!("container:{host}")
        } else {
            network.to_string()
        },
        "--label".into(),
        format!("{PROVIDER_LABEL}={}", provider.id),
        // What this tunnel is, said by the tunnel itself: whether it asks for
        // a password, and what address it asked to be given. Both are read
        // back by `status_all` rather than remembered anywhere.
        "--label".into(),
        format!("{GUARDED_LABEL}={}", plan.guard.is_some()),
    ];

    if let Some(name) = reserved {
        args.push("--label".into());
        args.push(format!("{RESERVED_LABEL}={name}"));
    }

    // The token, by name only. Docker copies the value from this process's
    // environment, which `runner::run_operation` sets for the child alone —
    // so the argv printed into the operation console carries a variable name
    // and never a credential.
    if let Some(var) = provider.token_env {
        args.push("-e".into());
        args.push(var.into());
    }

    match provider.id {
        "cloudflare" => {
            args.push(provider.image.into());
            args.extend(
                ["tunnel", "--no-autoupdate", "--url"]
                    .into_iter()
                    .map(String::from),
            );
            args.push(target);
            // Present the local domain as the Host header so name-based vhosts
            // and framework URL checks behave exactly as they do locally.
            if let Some(domain) = domain {
                args.push("--http-host-header".into());
                args.push(domain.into());
            }
        }

        // The same client, told to run a tunnel that already exists rather
        // than to invent one.
        "cloudflare_named" => {
            args.push(provider.image.into());
            args.extend(
                ["tunnel", "--no-autoupdate", "run", "--url"]
                    .into_iter()
                    .map(String::from),
            );
            args.push(target);
            // No `--token` argument: MEASURED, `cloudflared` reads
            // `TUNNEL_TOKEN` from the environment on its own — the same
            // invocation with the token only there answered `Provided Tunnel
            // token is not valid.`, which it could only have done after
            // reading it. So the credential stays out of the argv the
            // operation console prints.
            if let Some(domain) = domain {
                args.push("--http-host-header".into());
                args.push(domain.into());
            }
            // The reserved hostname is not sent anywhere: Cloudflare routes a
            // named tunnel from its own configuration, and this client is
            // never told which hostname it is serving. It is kept as a label
            // so the pane has an address to show — which is exactly what
            // `Reserved::in_log = false` means.
        }

        // Both SSH providers run the same client; what differs is the port,
        // the forward spelling and the far end.
        "localhost_run" => {
            args.push("--entrypoint".into());
            args.push("ssh".into());
            args.push(provider.image.into());
            args.extend(SSH_OPTS.iter().map(|s| String::from(*s)));
            args.push("-R".into());
            args.push(format!("80:{host}:{port}"));
            // The user nobody registered a key as — localhost.run's own name
            // for an anonymous tunnel.
            args.push("nokey@localhost.run".into());
        }
        "pinggy" => {
            args.push("--entrypoint".into());
            args.push("ssh".into());
            args.push(provider.image.into());
            args.extend(SSH_OPTS.iter().map(|s| String::from(*s)));
            // 443 rather than 22: the free endpoint listens there, and it is
            // also the port a restrictive network lets out.
            args.push("-p".into());
            args.push("443".into());
            args.push(format!("-R0:{host}:{port}"));
            args.push("a.pinggy.io".into());
        }

        "localtunnel" => {
            // No published image is maintained, so the client is fetched by
            // the runtime that ships it. `-y` because npx asks otherwise, and
            // a question in a detached container is a hang.
            args.push("--entrypoint".into());
            args.push("npx".into());
            args.push(provider.image.into());
            args.extend(
                ["-y", "localtunnel", "--port"]
                    .into_iter()
                    .map(String::from),
            );
            args.push(port.to_string());
            // `--local-host` is where the client connects *and* the Host it
            // sends, so it has to be the container — which is why this
            // provider cannot present the project's local domain.
            //
            // Not `--host`: that names the *tunnel server* (localtunnel.me by
            // default). The probe caught the first version of this arm
            // pointing it at the project container, which is not a tunnel
            // server — the client sat there saying nothing at all.
            args.push("--local-host".into());
            args.push(host.clone());
            if let Some(name) = reserved {
                // The only free reserved address on this table, and MEASURED:
                // the same subdomain came back twice, ninety seconds apart.
                // It is a request rather than a grant — see `status_all`,
                // which checks what came back against what was asked.
                args.push("--subdomain".into());
                args.push(name.into());
            }
        }

        "ngrok" => {
            args.push(provider.image.into());
            args.push("http".into());
            // `host:port` is the agent's own documented form for forwarding to
            // another machine on the network — `ngrok http servername.local:9000`
            // is in its help text, and this is the same shape.
            args.push(format!("{host}:{port}"));
            // Without it the agent logs to nowhere at all: `--log` defaults to
            // `false`, and the URL it assigns is only ever printed to that log.
            args.push("--log=stdout".into());
            if let Some(name) = reserved {
                // `--url` and not `--domain`: MEASURED, the agent answers
                // `Flag --domain has been deprecated, use --url instead` and
                // then honours it. The free plan includes one static domain,
                // which is the reason most people set ngrok up at all.
                args.push(format!("--url=https://{name}"));
            }
            if let Some(domain) = domain {
                // The agent calls this flag deprecated on every start — "use
                // traffic policy instead" — and then honours it, which the
                // probe watched it do. The replacement is a YAML file mounted
                // into the container, which is a file to write, a mount to
                // manage and a schema to track for one header; when the flag
                // is finally removed the agent will say so in the same log the
                // pane already shows, rather than failing silently.
                args.push(format!("--host-header={domain}"));
            }
        }

        "tailscale" => {
            // The one provider that is a daemon rather than a client, and the
            // one whose target cannot be another container.
            //
            // `tailscale funnel <target>` documents its target as "a service
            // running on the local machine" — a port, `localhost:3000`, or a
            // URL — and pointing it at `http://stackvo-shop:80` is a bet on an
            // undocumented case. So the sidecar joins the **project
            // container's own network namespace** instead, and then the target
            // is the canonical form the help text leads with: a port number.
            // Nothing is guessed, and nothing has to be forwarded twice.
            //
            // That is also why this arm does not take `--network`: a container
            // has one network mode, and this one is the project's.
            args.push("--entrypoint".into());
            args.push("sh".into());
            args.push(provider.image.into());
            args.push("-c".into());
            // The funnel is published at `<hostname>.<tailnet>.ts.net`, so the
            // hostname *is* the reserved name here — there is no second flag.
            // Left empty it stays what it has always been, which is the
            // project's own name.
            let hostname = reserved
                .map(String::from)
                .unwrap_or_else(|| format!("stackvo-{project}"));
            args.push(format!(
                // Userspace networking, so no TUN device and no privileged
                // container; `--state=mem:` because the node is as temporary
                // as the sidecar. The daemon's own log goes to a file rather
                // than to stdout: it prints a "you are logged out" health
                // warning during the two seconds before login, and a pane that
                // reads the log for failures would report it as one. It is
                // tailed only if login fails, which is when it is the answer.
                "tailscaled --tun=userspace-networking --state=mem: \
                 >/tmp/tailscaled.log 2>&1 & \
                 sleep 2; \
                 tailscale up --authkey=\"$TS_AUTHKEY\" --hostname={hostname} \
                 || {{ tail -20 /tmp/tailscaled.log; exit 1; }}; \
                 tailscale funnel --bg --yes {port} || exit 1; \
                 tailscale funnel status; \
                 wait"
            ));
        }

        "zrok" => {
            // Two steps by the vendor's design: an account token enables this
            // environment once, then a share is opened. `--headless` is what
            // stops it drawing a terminal UI at a log.
            args.push("--entrypoint".into());
            args.push("sh".into());
            args.push(provider.image.into());
            args.push("-c".into());
            args.push(match reserved {
                // The vendor's own two-step for a kept address: reserve the
                // name once, then share under it. The reserve is allowed to
                // fail — the second start is the case where the name is
                // already held, which is the whole point of reserving it —
                // and only the share decides whether this sidecar lives.
                Some(name) => format!(
                    "zrok enable \"$ZROK_TOKEN\" --headless || exit 1; \
                     zrok reserve public {target} --backend-mode proxy --unique-name {name} || true; \
                     exec zrok share reserved {name} --headless"
                ),
                // `--headless` on both halves: each subcommand draws its own
                // terminal UI otherwise, and neither has a terminal here.
                None => format!(
                    "zrok enable \"$ZROK_TOKEN\" --headless || exit 1; \
                     exec zrok share public --headless --backend-mode proxy {target}"
                ),
            });
        }

        "localxpose" => {
            args.push(provider.image.into());
            args.extend(["tunnel", "http", "--to"].into_iter().map(String::from));
            // `--to app.corp:8080` is the client's own example of forwarding to
            // another machine, so the project container is a target it names.
            args.push(format!("{host}:{port}"));
            if let Some(name) = reserved {
                // From the client's own help text, like the header plugin
                // below. Not measured — nobody here has an account, which is
                // what `verified` already says of this row.
                args.push("--subdomain".into());
                args.push(name.into());
            }
            if let Some(domain) = domain {
                // Its header plugin, not a special case: `--request-header
                // host:myapp.com` is in the help text, and Host is the header
                // that decides which vhost answers.
                args.push("--request-header".into());
                args.push(format!("host:{domain}"));
            }
        }

        // Unreachable through `provider()`, which refuses an unknown id; a
        // provider added to the table and not to this match would otherwise
        // start a container with no command at all.
        other => unreachable!("provider {other} has no invocation"),
    }

    args
}

/// Refuse to start a tunnel to a container that is not running: the client
/// would happily serve 502s from a URL that looks like it worked.
pub async fn ensure_project_running(project: &str) -> Result<()> {
    let containers = crate::engine::stackvo_containers().await?;
    match containers.get(project) {
        Some(info) if info.running => Ok(()),
        Some(_) => Err(
            Error::new(Code::Conflict, format!("{project} is not running"))
                .with_hint(crate::hints::START_PROJECT_FOR_TUNNEL),
        ),
        None => Err(Error::not_found(format!("container for {project}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_id(id: &str) -> &'static Provider {
        provider(id).expect("a provider in the table")
    }

    #[test]
    fn the_assigned_url_is_read_out_of_the_banner() {
        let log = "\
2026-07-30T09:00:00Z INF Thank you for trying Cloudflare Tunnel.\n\
2026-07-30T09:00:01Z INF +--------------------------------------------------------------------------------------------+\n\
2026-07-30T09:00:01Z INF |  Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):  |\n\
2026-07-30T09:00:01Z INF |  https://random-words-here.trycloudflare.com                                               |\n\
2026-07-30T09:00:01Z INF +--------------------------------------------------------------------------------------------+\n";
        assert_eq!(
            find_url(by_id("cloudflare"), log).as_deref(),
            Some("https://random-words-here.trycloudflare.com")
        );
    }

    #[test]
    fn a_log_without_a_url_yields_none_not_a_guess() {
        let cloudflare = by_id("cloudflare");
        assert_eq!(
            find_url(
                cloudflare,
                "INF Requesting new quick Tunnel on trycloudflare.com...\nERR failed to connect"
            ),
            None
        );
        // An https URL that is not a quick-tunnel URL is not the answer.
        assert_eq!(
            find_url(
                cloudflare,
                "INF see https://developers.cloudflare.com/tunnel for docs"
            ),
            None
        );
    }

    /// The line each anonymous provider actually prints, from a real run.
    #[test]
    fn every_provider_finds_its_own_url_and_no_other() {
        let cases = [
            (
                "cloudflare",
                "INF |  https://four-random-words.trycloudflare.com  |",
                "https://four-random-words.trycloudflare.com",
            ),
            (
                "localhost_run",
                "dd2e7a857d7cbc.lhr.life tunneled with tls termination, https://dd2e7a857d7cbc.lhr.life",
                "https://dd2e7a857d7cbc.lhr.life",
            ),
            (
                "pinggy",
                "https://brmgk-188-119-17-94.free.pinggy.net",
                "https://brmgk-188-119-17-94.free.pinggy.net",
            ),
            (
                "localtunnel",
                "your url is: https://short-cats-shave.loca.lt",
                "https://short-cats-shave.loca.lt",
            ),
            (
                "ngrok",
                r#"t=2026-08-20T09:00:00+0000 lvl=info msg="started tunnel" obj=tunnels name=command_line addr=http://stackvo-shop:80 url=https://a1b2-c3.ngrok-free.app"#,
                "https://a1b2-c3.ngrok-free.app",
            ),
            (
                "tailscale",
                "Available on the internet:\nhttps://stackvo-shop.tail1234.ts.net/",
                "https://stackvo-shop.tail1234.ts.net",
            ),
            (
                "zrok",
                "[   0.123]    INFO sdk/golang/sdk.share: your share token is 'abc123'\naccess your share at https://abc123.share.zrok.io/",
                "https://abc123.share.zrok.io",
            ),
            (
                "localxpose",
                "Tunnel URL: https://eu-1-abc.loclx.io",
                "https://eu-1-abc.loclx.io",
            ),
        ];

        for (id, log, expected) in cases {
            let p = by_id(id);
            assert_eq!(
                find_url(p, log).as_deref(),
                Some(expected),
                "{id} did not find its own URL"
            );

            // And nobody else's: a table where two providers match the same
            // hostname would report the wrong one on an unlabelled container.
            for other in PROVIDERS.iter().filter(|o| o.id != id) {
                assert_eq!(
                    find_url(other, log),
                    None,
                    "{} matched {id}'s URL",
                    other.id
                );
            }
        }
    }

    /// The whole banner localhost.run prints, as it printed it.
    ///
    /// Kept in full because the trap is what surrounds the answer: two
    /// dashboard links sit above the tunnel URL, and the first version of this
    /// module would have handed the dashboard out as the project's public
    /// address.
    #[test]
    fn the_dashboard_link_in_a_banner_is_not_the_tunnel() {
        let log = "\
Pseudo-terminal will not be allocated because stdin is not a terminal.\n\
Warning: Permanently added 'localhost.run' (ED25519) to the list of known hosts.\n\
Welcome to localhost.run!\n\
To set up and manage custom domains go to https://admin.localhost.run/\n\
To explore using localhost.run visit the documentation site:\n\
https://localhost.run/docs/\n\
authn: authenticated as anonymous user\n\
8e6e10ec3ead9a.lhr.life tunneled with tls termination, https://8e6e10ec3ead9a.lhr.life\n";

        assert_eq!(
            find_url(by_id("localhost_run"), log).as_deref(),
            Some("https://8e6e10ec3ead9a.lhr.life")
        );
        // And the banner alone, before the forward is up, is not an answer.
        let banner = log.lines().take(6).collect::<Vec<_>>().join("\n");
        assert_eq!(find_url(by_id("localhost_run"), &banner), None);
    }

    /// Pinggy's real output, including the two lines that are traps.
    ///
    /// The upgrade link sits directly above the tunnel, and the session cap is
    /// printed as prose — which is why [`Provider::session_minutes`] carries
    /// the number rather than the pane repeating a sentence nobody reads.
    #[test]
    fn the_upgrade_link_above_a_pinggy_tunnel_is_not_the_tunnel() {
        let log = "\
Allocated port 2 for remote forward to stackvo-shop:80\n\
You are not authenticated.\n\
Your tunnel will expire in 60 minutes. Upgrade to Pinggy Pro to get unrestricted tunnels. https://dashboard.pinggy.io\n\
https://brmgk-188-119-17-94.free.pinggy.net\n\
https://fatoc-188-119-17-94.run.pinggy-free.link\n";

        assert_eq!(
            find_url(by_id("pinggy"), log).as_deref(),
            Some("https://brmgk-188-119-17-94.free.pinggy.net")
        );
        assert_eq!(by_id("pinggy").session_minutes, Some(60));
    }

    #[test]
    fn a_url_inside_a_sentence_keeps_only_the_host() {
        // The trailing full stop belongs to the sentence, not to the hostname,
        // and a path is not part of the address the pane hands out.
        assert_eq!(
            find_url(
                by_id("localhost_run"),
                "connect to https://e0a1b2c3.lhr.life/some/path for your site."
            )
            .as_deref(),
            Some("https://e0a1b2c3.lhr.life")
        );
    }

    #[test]
    fn a_rejected_token_is_reported_as_the_client_worded_it() {
        let log = "authentication failed: The authtoken you specified is properly formed, but it is invalid.";
        assert_eq!(find_failure(log).as_deref(), Some(log));

        // A healthy log is not a failure, however many URLs it contains.
        assert_eq!(
            find_failure("INF |  https://x.trycloudflare.com  |\nINF Registered tunnel connection"),
            None
        );
    }

    #[test]
    fn a_refused_forward_is_a_failure_even_though_ssh_connected() {
        let log = "Warning: remote port forwarding failed for listen port 80";
        assert!(find_failure(log).is_some());
    }

    #[test]
    fn internal_port_mirrors_the_generators_arithmetic() {
        let mut m = crate::manifest::Manifest {
            name: "x".into(),
            domain: None,
            runtime: "php".into(),
            server: Some("nginx".into()),
            document_root: None,
            aliases: vec![],
            lan_share: false,
            services: vec![],
            php: None,
            node: None,
            lang: None,
            valid: true,
            errors: vec![],
            warnings: vec![],
            hooks: Default::default(),
            schedule: Vec::new(),
            commands: Default::default(),
            sidecars: Default::default(),
            providers: Vec::new(),
            local: Vec::new(),
        };
        assert_eq!(internal_port(&m), 80);

        m.server = Some("swoole".into());
        assert_eq!(internal_port(&m), 8000);

        m.runtime = "node".into();
        m.node = Some(crate::manifest::NodeConfig {
            version: "20".into(),
            install: "npm ci".into(),
            build: None,
            start: "npm start".into(),
            port: 4321,
            package_manager: None,
        });
        assert_eq!(internal_port(&m), 4321);

        m.node = None;
        assert_eq!(internal_port(&m), 3000);
    }

    #[test]
    fn the_sidecar_forwards_to_the_container_with_the_local_host_header() {
        let args = run_args(
            by_id("cloudflare"),
            &Plan::direct("myapp", Some("myapp.loc"), 80, "stackvo-net"),
        );
        let line = args.join(" ");
        assert!(line.contains("--name stackvo-tunnel-myapp"));
        assert!(line.contains("--network stackvo-net"));
        assert!(line.contains("--url http://stackvo-myapp:80"));
        assert!(line.contains("--http-host-header myapp.loc"));
        assert!(line.contains("--label stackvo.tunnel.provider=cloudflare"));

        // No domain, no header — never an empty flag value.
        let bare = run_args(
            by_id("cloudflare"),
            &Plan::direct("myapp", None, 3000, "stackvo-net"),
        );
        assert!(!bare.join(" ").contains("--http-host-header"));
    }

    #[test]
    fn every_provider_points_at_the_project_container_and_labels_itself() {
        for p in PROVIDERS {
            let args = run_args(
                p,
                &Plan::direct("shop", Some("shop.loc"), 8080, "stackvo-net"),
            );
            let line = args.join(" ");
            assert!(
                line.contains("stackvo-shop"),
                "{} does not name the project container: {line}",
                p.id
            );
            assert!(
                line.contains("8080"),
                "{} does not carry the container port: {line}",
                p.id
            );
            assert!(
                line.contains(&format!("--label {PROVIDER_LABEL}={}", p.id)),
                "{} is not labelled with its own id",
                p.id
            );
            assert!(
                line.contains(p.image),
                "{} does not run its own image",
                p.id
            );
            // The container must be findable by the one name stop and status
            // look under, whatever provider opened it.
            assert!(line.contains("--name stackvo-tunnel-shop"), "{}", p.id);
            // `--rm` would take the failure away with the container.
            assert!(!line.contains("--rm"), "{} uses --rm", p.id);
        }
    }

    #[test]
    fn a_token_reaches_the_container_by_name_and_never_by_value() {
        for p in PROVIDERS.iter().filter(|p| p.token_env.is_some()) {
            let var = p.token_env.unwrap();
            let args = run_args(p, &Plan::direct("shop", None, 80, "stackvo-net"));
            let line = args.join(" ");
            assert!(line.contains(&format!("-e {var}")), "{} ", p.id);
            // The value is never here: the only spellings allowed are the bare
            // variable name and a shell expansion the container performs.
            assert!(
                !line.contains(&format!("{var}=")),
                "{} writes the token into its argv",
                p.id
            );
        }

        // And the anonymous ones ask for nothing at all.
        for p in PROVIDERS.iter().filter(|p| p.token_env.is_none()) {
            assert!(
                !run_args(p, &Plan::direct("shop", None, 80, "stackvo-net"))
                    .contains(&"-e".to_string()),
                "{} passes an environment variable it does not need",
                p.id
            );
        }
    }

    /// Tailscale's sidecar is the one that does not sit on the stack network.
    ///
    /// Its client serves "a service running on the local machine", so the
    /// sidecar becomes the project container's network and the target is the
    /// canonical port number from the help text — rather than a remote URL the
    /// documentation never promises to accept.
    #[test]
    fn tailscale_joins_the_project_container_rather_than_the_stack_network() {
        let args = run_args(
            by_id("tailscale"),
            &Plan::direct("shop", Some("shop.loc"), 8080, "stackvo-net"),
        );
        let line = args.join(" ");

        assert!(line.contains("--network container:stackvo-shop"), "{line}");
        assert!(!line.contains("--network stackvo-net"), "{line}");
        // The target is a port, not a URL: `tailscale funnel 8080`.
        assert!(line.contains("tailscale funnel --bg --yes 8080"), "{line}");
        assert!(!line.contains("funnel --bg http://"), "{line}");
        // Non-interactive, unprivileged, and quiet about its own startup.
        assert!(
            line.contains("--yes"),
            "a prompt in a detached container is a hang"
        );
        assert!(line.contains("--tun=userspace-networking"), "{line}");
        assert!(line.contains("/tmp/tailscaled.log"), "{line}");

        // And every other provider stays on the stack network.
        for p in PROVIDERS.iter().filter(|p| p.id != "tailscale") {
            let other = run_args(p, &Plan::direct("shop", None, 80, "stackvo-net")).join(" ");
            assert!(
                other.contains("--network stackvo-net"),
                "{} left the stack network",
                p.id
            );
        }
    }

    /// Read out of each client's own help text, not assumed: three of them can
    /// present the project's local domain to the application, and the ones
    /// that cannot say so on screen instead of being quietly wrong.
    #[test]
    fn the_local_domain_is_presented_by_exactly_the_providers_that_can() {
        for p in PROVIDERS {
            let line = run_args(
                p,
                &Plan::direct("shop", Some("shop.loc"), 80, "stackvo-net"),
            )
            .join(" ");
            assert_eq!(
                line.contains("shop.loc"),
                p.rewrites_host,
                "{} disagrees with its own `rewrites_host`",
                p.id
            );
        }

        // The spellings are the clients', and they are all different.
        assert!(run_args(
            by_id("cloudflare"),
            &Plan::direct("shop", Some("shop.loc"), 80, "n")
        )
        .join(" ")
        .contains("--http-host-header shop.loc"));
        assert!(run_args(
            by_id("ngrok"),
            &Plan::direct("shop", Some("shop.loc"), 80, "n")
        )
        .join(" ")
        .contains("--host-header=shop.loc"));
        assert!(run_args(
            by_id("localxpose"),
            &Plan::direct("shop", Some("shop.loc"), 80, "n")
        )
        .join(" ")
        .contains("--request-header host:shop.loc"));
        // localtunnel's single host flag is its target as well, so it names
        // the container and the application sees that as `Host`. `--host` is
        // a different flag entirely — the tunnel server — and pointing it at
        // the project was a real defect the probe caught.
        let lt = run_args(
            by_id("localtunnel"),
            &Plan::direct("shop", Some("shop.loc"), 80, "n"),
        )
        .join(" ");
        assert!(lt.contains("--local-host stackvo-shop"), "{lt}");
        assert!(!lt.contains("--host http"), "{lt}");
    }

    /// The clients print their URL only where they are told to.
    #[test]
    fn a_client_that_needs_telling_where_to_log_is_told() {
        // ngrok's `--log` defaults to `false`: without this the agent runs
        // perfectly and prints the URL nowhere at all.
        assert!(
            run_args(by_id("ngrok"), &Plan::direct("shop", None, 80, "n"))
                .join(" ")
                .contains("--log=stdout")
        );
        // Both zrok subcommands draw a terminal UI otherwise.
        let zrok = run_args(by_id("zrok"), &Plan::direct("shop", None, 80, "n")).join(" ");
        assert_eq!(zrok.matches("--headless").count(), 2, "{zrok}");
    }

    /// The four lines four real clients printed when handed an invalid token.
    ///
    /// Measured by `examples/tunnel_probe.rs`, which is repeatable: these are
    /// the only evidence that a rejected token reaches the user as words
    /// rather than as a spinner.
    #[test]
    fn every_token_providers_refusal_is_recognised() {
        let cases = [
            (
                "ngrok",
                r#"t=2026-08-20T20:36:20+0000 lvl=eror msg="failed to reconnect session" obj=tunnels.session err="authentication failed: The authtoken you specified does not look like a proper ngrok authtoken. ERR_NGROK_105""#,
            ),
            (
                "tailscale",
                "backend error: invalid key: API key does not exist",
            ),
            (
                "zrok",
                r#"{"level":"error","msg":"the zrok service returned an error: [POST /enable][401] enableUnauthorized ","time":"2026-08-20T20:37:52.612Z"}"#,
            ),
            ("localxpose", "Error: unauthenticated access"),
            // MEASURED here, with the same throwaway token the four above
            // were measured with, and it is the one that needed a new needle:
            // none of the others matched this sentence.
            ("cloudflare_named", "Provided Tunnel token is not valid."),
        ];

        for (id, log) in cases {
            assert!(
                find_failure(log).is_some(),
                "{id}'s refusal would leave the pane spinning: {log}"
            );
            // And it is not mistaken for a URL.
            assert_eq!(find_url(by_id(id), log), None, "{id}");
        }
    }

    #[test]
    fn an_unknown_provider_is_refused_rather_than_defaulted() {
        assert!(provider("ngrok").is_ok());
        assert!(provider("ngrok ").is_err());
        assert!(provider("").is_err());
        assert_eq!(provider(DEFAULT_PROVIDER).unwrap().id, "cloudflare");
    }

    #[test]
    fn the_keystore_entry_is_per_provider() {
        assert_eq!(secret_name("ngrok"), "tunnel-token:ngrok");
        assert_ne!(secret_name("ngrok"), secret_name("zrok"));
    }

    /// The table is a contract with the front end: it renders one row per
    /// provider and keys its translations off the id.
    #[test]
    fn provider_ids_are_unique_and_translatable() {
        let mut ids: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "two providers share an id");

        for p in PROVIDERS {
            assert!(
                p.id.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
                "{} is not usable as a translation key",
                p.id
            );
            // A provider with no suffixes is one whose address cannot be read
            // out of a log, and there is exactly one of those — the named
            // Cloudflare tunnel, whose hostname lives in Cloudflare's own
            // configuration. Everybody else has to recognise their own URL.
            assert_eq!(
                p.url_suffixes.is_empty(),
                p.reserved.is_some_and(|r| !r.in_log),
                "{} disagrees with its own `Reserved::in_log`",
                p.id
            );
        }
    }

    // ------------------------------------------------------------ B-7

    /// With the guard on, every provider forwards to it — including the one
    /// that joins a network namespace rather than a network.
    ///
    /// This is the test that would catch the worst outcome the feature has: a
    /// sidecar pointed at the project while the pane says the link asks for a
    /// password.
    #[test]
    fn a_guarded_plan_points_every_provider_at_the_guard() {
        let guard = "stackvo-tunnel-guard-shop";
        for p in PROVIDERS {
            let plan = Plan {
                project: "shop",
                domain: Some("shop.loc"),
                // The project's own port, which nothing may forward to while
                // a guard is in front of it.
                port: 3000,
                network: "stackvo-net",
                reserved: None,
                guard: Some(guard),
            };
            let line = run_args(p, &plan).join(" ");

            assert!(
                line.contains(guard),
                "{} does not forward through the guard: {line}",
                p.id
            );
            assert!(
                line.contains(&crate::tunnelid::GUARD_PORT.to_string()),
                "{} is not on the guard's port: {line}",
                p.id
            );
            assert!(
                !line.contains(":3000") && !line.contains(" 3000"),
                "{} still reaches the project's own port past the guard: {line}",
                p.id
            );
            assert!(
                line.contains(&format!("--label {GUARDED_LABEL}=true")),
                "{} does not say it is guarded: {line}",
                p.id
            );
        }

        // Tailscale's namespace is the guard's, not the project's — the one
        // provider that could not have taken an authentication flag needs no
        // special case at all.
        let line = run_args(
            by_id("tailscale"),
            &Plan {
                project: "shop",
                domain: None,
                port: 3000,
                network: "stackvo-net",
                reserved: None,
                guard: Some(guard),
            },
        )
        .join(" ");
        assert!(
            line.contains(&format!("--network container:{guard}")),
            "{line}"
        );
        assert!(line.contains("tailscale funnel --bg --yes 8080"), "{line}");
    }

    /// An unguarded plan is exactly what it was before B-7, and says so.
    #[test]
    fn an_unguarded_plan_still_goes_straight_at_the_project() {
        for p in PROVIDERS {
            let line = run_args(p, &Plan::direct("shop", None, 80, "stackvo-net")).join(" ");
            assert!(line.contains("stackvo-shop"), "{} : {line}", p.id);
            assert!(
                !line.contains(crate::tunnelid::GUARD_ID_PREFIX),
                "{} : {line}",
                p.id
            );
            assert!(
                line.contains(&format!("--label {GUARDED_LABEL}=false")),
                "{} does not say it is unguarded: {line}",
                p.id
            );
        }
    }

    /// Each provider's own spelling for the address it keeps, and no spelling
    /// at all for the three that keep nothing.
    #[test]
    fn a_reserved_name_reaches_the_flag_the_client_actually_has() {
        let plan = |reserved| Plan {
            project: "shop",
            domain: None,
            port: 80,
            network: "stackvo-net",
            reserved: Some(reserved),
            guard: None,
        };

        assert!(run_args(by_id("localtunnel"), &plan("shop-dev"))
            .join(" ")
            .contains("--subdomain shop-dev"));
        assert!(run_args(by_id("ngrok"), &plan("shop.ngrok-free.app"))
            .join(" ")
            .contains("--url=https://shop.ngrok-free.app"));
        assert!(run_args(by_id("localxpose"), &plan("shop-dev"))
            .join(" ")
            .contains("--subdomain shop-dev"));
        // Tailscale has no second flag: the hostname is the address.
        let ts = run_args(by_id("tailscale"), &plan("shop-dev")).join(" ");
        assert!(ts.contains("--hostname=shop-dev"), "{ts}");
        assert!(!ts.contains("--hostname=stackvo-shop"), "{ts}");
        // And left empty it is the project's name, as it always was.
        assert!(
            run_args(by_id("tailscale"), &Plan::direct("shop", None, 80, "n"))
                .join(" ")
                .contains("--hostname=stackvo-shop")
        );
        // zrok reserves once and shares under the name from then on; the
        // reserve is allowed to fail, because the second start is the case it
        // exists for.
        let zrok = run_args(by_id("zrok"), &plan("shopdev")).join(" ");
        assert!(zrok.contains("zrok reserve public"), "{zrok}");
        assert!(zrok.contains("--unique-name shopdev || true"), "{zrok}");
        assert!(zrok.contains("exec zrok share reserved shopdev"), "{zrok}");

        // The three that keep nothing are never handed one — a flag their
        // client does not have is a container that exits on its usage text.
        for id in ["cloudflare", "localhost_run", "pinggy"] {
            let line = run_args(by_id(id), &plan("shop-dev")).join(" ");
            assert!(!line.contains("shop-dev"), "{id} was handed a name: {line}");
            assert!(
                !line.contains(RESERVED_LABEL),
                "{id} claims a name it cannot keep: {line}"
            );
        }

        // Everybody who can keep one labels what they asked for, so the
        // status call can check it against what came back.
        for p in PROVIDERS.iter().filter(|p| p.reserved.is_some()) {
            assert!(
                run_args(p, &plan("shopdev"))
                    .join(" ")
                    .contains(&format!("--label {RESERVED_LABEL}=shopdev")),
                "{} does not record the name it asked for",
                p.id
            );
        }
    }

    /// The ninth row: the same client, running a tunnel somebody already
    /// created, with its token in the environment and nowhere else.
    #[test]
    fn the_named_cloudflare_tunnel_runs_rather_than_invents() {
        let p = by_id("cloudflare_named");
        let line = run_args(
            p,
            &Plan {
                project: "shop",
                domain: Some("shop.loc"),
                port: 80,
                network: "stackvo-net",
                reserved: Some("shop.example.com"),
                guard: None,
            },
        )
        .join(" ");

        assert!(line.contains("tunnel --no-autoupdate run --url"), "{line}");
        assert!(line.contains("http://stackvo-shop:80"), "{line}");
        assert!(line.contains("--http-host-header shop.loc"), "{line}");
        // MEASURED: the client reads TUNNEL_TOKEN itself, so the credential
        // never becomes an argument.
        assert!(line.contains("-e TUNNEL_TOKEN"), "{line}");
        assert!(!line.contains("--token"), "{line}");
        // The hostname is Cloudflare's to route; this client is never told
        // it, and the label is the only place it exists.
        assert!(
            line.contains(&format!("--label {RESERVED_LABEL}=shop.example.com")),
            "{line}"
        );
        assert_eq!(line.matches("shop.example.com").count(), 1, "{line}");
        assert!(p.reserved.is_some_and(|r| !r.in_log));

        // And the quick tunnel it shares a client with keeps its own shape.
        let quick = run_args(by_id("cloudflare"), &Plan::direct("shop", None, 80, "n")).join(" ");
        assert!(!quick.contains(" run "), "{quick}");
        assert!(!quick.contains("-e TUNNEL_TOKEN"), "{quick}");
    }
}
