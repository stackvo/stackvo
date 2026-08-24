//! Who may open the tunnel, and what its address is called (B-7).
//!
//! [`crate::tunnel`] answers "can this project be reached from the internet".
//! The second question everybody asks about a link they just pasted into a
//! group chat is not answered there at all, and it is two questions:
//!
//! * **who else can open it** — a quick tunnel is a public, unauthenticated
//!   door into an application running on somebody's laptop, and the Share pane
//!   could only warn about that;
//! * **will it still be this address tomorrow** — an OAuth redirect URI, a
//!   Stripe webhook endpoint and a QR code on a slide are all registered once
//!   and used later, and a name that changes on every start is useless for all
//!   three.
//!
//! ## The guard is ours, and that is the whole design
//!
//! Four of the eight providers can do basic authentication themselves, three
//! cannot, and the four that can spell it four different ways — one of them
//! through a YAML policy file rather than a flag. Wiring each of those would
//! have been four branches, four things to keep up with, and a feature that
//! silently is not there on the provider somebody happened to pick.
//!
//! So authentication is not asked of the provider. When it is on, the sidecar
//! is pointed at **an nginx container of ours** on the stack network, and that
//! container is what forwards to the project. Every provider reaches it the
//! same way they reached the project container, the check is the same check
//! whichever one is running, and it is checkable here rather than in eight
//! vendors' documentation — `examples/tunnel_guard_probe.rs` opens it and
//! measures three answers: no credentials → 401, wrong credentials → 401,
//! right credentials → the application's own page.
//!
//! Tailscale is not an exception this time either. Its sidecar joins a
//! container's network namespace rather than the stack network, and with the
//! guard on, the namespace it joins is the guard's — so `funnel <port>` serves
//! the guarded port and the one provider that could not have taken a flag
//! needs no special case at all.
//!
//! ## The credential is never written to the workspace
//!
//! It lives in the OS keystore, one entry per project — a tunnel credential is
//! this project's, not this machine's, which is the opposite of
//! [`crate::tunnel::secret_name`] and deliberately so: the token identifies the
//! *account* that opens tunnels, and this identifies *the door* one tunnel
//! leads to. Two projects sharing one password means a link handed out for one
//! opens the other.
//!
//! It reaches the guard as an environment variable holding the ready-made
//! `Basic` credential, and the nginx configuration is written **inside** the
//! container from that variable. Nothing on the host holds it except the
//! keystore: no file in `generated/`, no mount, and no argument — `docker
//! inspect`, `ps` and this app's own operation console all print the argv.
//!
//! ## The password is shown, unlike every other secret in this app
//!
//! [`crate::stripe`]'s key and a tunnel token are write-only on purpose: they
//! are *entered* from somewhere else and never needed again. This one is the
//! other kind. It is generated here and it has to be typed into a browser on
//! another device, or pasted next to the link — a credential nobody can read
//! is a credential nobody can use, so [`reveal`] exists and says so.
//!
//! ## A reserved name is a request, not a promise
//!
//! Measured, and this is the finding that shaped the reporting: localtunnel
//! with `--subdomain stackvo-probe-11959` handed back exactly that address, and
//! ninety seconds later handed back the same one again — free, with no account,
//! which is the whole feature working. But started *immediately* after the
//! previous tunnel closed, the same request came back as
//! `bitter-bulldog-88.loca.lt`, with no error and no warning: the server still
//! held the name, so it quietly assigned another.
//!
//! A pane that showed the requested name would have been wrong in exactly that
//! case, and wrong in the way that costs the most — the address is registered
//! in a dashboard and it is not the address the tunnel is on. So the reserved
//! name is stored, sent, and then **compared against what the client actually
//! printed**; [`crate::tunnel::TunnelStatus::reserved_honoured`] is that
//! comparison and the pane says when the answer is no.

use crate::error::{Code, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The engine-facing id prefix of a guard. `engine::*` prefixes `stackvo-`
/// itself, so the container is `stackvo-tunnel-guard-<project>`.
pub const GUARD_ID_PREFIX: &str = "tunnel-guard-";

/// The label a guard carries its project in.
///
/// The label and not the name is what [`crate::tunnel::status_all`] tells a
/// guard from a tunnel by: a guard's id starts with the tunnel prefix, so a
/// project genuinely called `guard-shop` would otherwise appear in the Share
/// pane as a tunnel nobody opened.
pub const GUARD_LABEL: &str = "stackvo.tunnel.guard";

/// nginx, for the reason [`crate::landing`] gives: this app already pulls it
/// for the landing page, so turning authentication on downloads nothing.
pub const GUARD_IMAGE: &str = "nginx:alpine";

/// The port the guard listens on.
///
/// Not 80: nothing publishes it and the number is only ever seen by the
/// sidecar, but a container that answers on the same port as the project it
/// fronts is one nobody can tell apart in a log.
pub const GUARD_PORT: u16 = 8080;

/// The variable the ready-made `Basic` credential reaches the guard in.
pub const AUTH_ENV: &str = "STACKVO_TUNNEL_AUTH";

/// What the browser prompt is titled. Named after the app rather than the
/// project: the prompt appears on somebody else's phone, and "shop" alone
/// tells them nothing about what is asking.
pub const REALM: &str = "StackVo tunnel";

/// The shape [`names_path`] is written in.
const SCHEMA_VERSION: u64 = 1;

/// The guard's engine-facing id for one project.
pub fn guard_id(project: &str) -> String {
    format!("{GUARD_ID_PREFIX}{project}")
}

/// The keystore entry this project's tunnel credential is kept under.
///
/// Per project, and that is the opposite of [`crate::tunnel::secret_name`] for
/// a reason that is not an inconsistency: a provider token says *who is
/// allowed to open tunnels from this machine* and belongs to the machine; this
/// says *who is allowed through this project's door*. One password shared by
/// every project would mean the link handed to a designer for one site is a
/// working password for every other site on the laptop.
pub fn secret_name(project: &str) -> String {
    format!("tunnel-auth:{project}")
}

/// The credential a visitor has to present.
///
/// Serialised in full — password included — and that is the one place in this
/// app where that is right: it is generated here rather than entered, and it
/// exists to be read out to somebody. See the module note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub user: String,
    pub password: String,
}

/// The alphabet a generated password is drawn from.
///
/// Missing on purpose: `0`, `O`, `1`, `l`, `I`. This password is read off one
/// screen and typed into another device — usually a phone, usually by somebody
/// who did not choose it — and the two minutes lost to a zero that was an O is
/// the whole reason the character class is smaller than it could be.
const ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// How many characters a generated password has.
///
/// Twenty out of an alphabet of 56 is a little over 116 bits. The tunnel is
/// public and its address is guessable in a way `myapp.loc` never was — a
/// short password behind a public URL is a lock on an unlocked door.
const PASSWORD_LEN: usize = 20;

/// The default user name, when the caller does not choose one.
const DEFAULT_USER: &str = "stackvo";

impl Credentials {
    /// The form the keystore holds: `user:password`, which is also exactly
    /// what basic authentication encodes.
    pub fn stored(&self) -> String {
        format!("{}:{}", self.user, self.password)
    }

    /// `user:password` back out of the keystore.
    ///
    /// Split on the **first** colon, because a colon is legal in a password
    /// and illegal in a user name — the same rule the HTTP scheme itself uses,
    /// and getting it backwards would lock somebody out of their own tunnel
    /// with no message.
    pub fn parse(stored: &str) -> Option<Self> {
        let (user, password) = stored.split_once(':')?;
        if user.is_empty() || password.is_empty() {
            return None;
        }
        Some(Self {
            user: user.to_string(),
            password: password.to_string(),
        })
    }

    /// The value of the `Authorization` header a visitor must send, without
    /// the `Basic ` prefix.
    pub fn header_value(&self) -> String {
        base64(self.stored().as_bytes())
    }

    /// Refuse what basic authentication cannot express.
    ///
    /// A colon in the user name is not a strict credential — it is a
    /// *different* credential, because the header is split at the first one.
    /// Control characters and non-ASCII are refused for the same reason: the
    /// header is latin-1 on the wire, and a password with an `ş` in it is a
    /// password that works in one browser and not in the next.
    pub fn validate(&self) -> Result<()> {
        let bad = |what: &str, why: &str| {
            Err(Error::new(
                Code::InvalidInput,
                format!("the tunnel {what} {why}"),
            ))
        };

        if self.user.trim().is_empty() {
            return bad("user name", "is empty");
        }
        if self.password.is_empty() {
            return bad("password", "is empty");
        }
        if self.user.contains(':') {
            return bad("user name", "cannot contain a colon");
        }
        for (what, value) in [("user name", &self.user), ("password", &self.password)] {
            if !value.chars().all(|c| c.is_ascii_graphic()) {
                return bad(
                    what,
                    "must be printable ASCII — a browser sends this header as latin-1",
                );
            }
        }
        Ok(())
    }
}

/// A fresh credential, with the user name the caller asked for or the default.
///
/// The password is generated rather than asked for. A field somebody has to
/// fill in is a field that gets `test123`, and this one is behind an address
/// that is on the public internet the moment it is handed out.
pub fn generate(user: Option<&str>) -> Credentials {
    let mut bytes = [0u8; PASSWORD_LEN];
    // A failure here is a system without a random source, which is not a
    // condition this app can carry on through by inventing one: the fallback
    // is refusing to have a password rather than having a predictable one.
    let password = if getrandom::fill(&mut bytes).is_ok() {
        bytes
            .iter()
            // Modulo bias across 56 symbols in 256 values is under 2%, and the
            // alternative — rejection sampling — buys nothing against an
            // attacker who has to make 10^34 guesses either way.
            .map(|b| ALPHABET[*b as usize % ALPHABET.len()] as char)
            .collect()
    } else {
        String::new()
    };

    Credentials {
        user: user
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .unwrap_or(DEFAULT_USER)
            .to_string(),
        password,
    }
}

/// This project's credential, or `None` when the tunnel is open to everyone.
///
/// "A credential is stored" **is** "authentication is on". There is no second
/// switch, because a switch that can disagree with the keystore is a switch
/// that eventually says protected about a tunnel that is not.
pub fn read(project: &str) -> Result<Option<Credentials>> {
    Ok(crate::secrets::read(&secret_name(project))?
        .as_deref()
        .and_then(Credentials::parse))
}

/// Store a credential, or clear it with `None`.
///
/// Returns what is now in force, so a caller never has to guess whether the
/// keystore took it.
pub fn set(project: &str, credentials: Option<Credentials>) -> Result<Option<Credentials>> {
    let entry = secret_name(project);
    match credentials {
        Some(credentials) => {
            credentials.validate()?;
            if credentials.password.is_empty() {
                return Err(Error::new(
                    Code::IoError,
                    "no random source is available to generate a password with",
                ));
            }
            crate::secrets::write(&entry, &credentials.stored())?;
            Ok(Some(credentials))
        }
        None => {
            crate::secrets::delete(&entry)?;
            Ok(None)
        }
    }
}

/// What a pane needs to know before it can offer either half of B-7.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    /// The user name a visitor is asked for, or `None` when the tunnel is open
    /// to anyone with the link. Never the password — [`reveal`] is that.
    pub auth_user: Option<String>,
    /// Whether this machine has a keystore to hold a credential in at all.
    ///
    /// Said rather than discovered: on a Linux box with no secret service the
    /// answer is no, and a switch that fails when it is pressed is worse than
    /// one that explains itself before.
    pub keystore: bool,
    /// The address each provider has been asked to keep, by provider id.
    pub reserved: BTreeMap<String, String>,
}

/// This project's identity, as a pane asks for it.
pub fn identity(project: &str) -> Result<Identity> {
    Ok(Identity {
        auth_user: read(project)?.map(|c| c.user),
        keystore: crate::secrets::available(),
        reserved: names(project),
    })
}

/// The credential, in full, for the one screen that has to show it.
///
/// A separate function from [`read`] with the same body on purpose: the name
/// is what makes a reader stop, and the audit trail of "who calls the thing
/// that returns a password" is worth a line of duplication.
pub fn reveal(project: &str) -> Result<Option<Credentials>> {
    read(project)
}

// ------------------------------------------------------------------ the guard

/// The nginx configuration the guard runs, with the credential left as a
/// placeholder for [`guard_args`] to turn into a shell expansion.
///
/// Kept apart from the shell script so it can be read as what it is — and so
/// the test below can assert on the configuration rather than on quoting.
pub fn guard_conf(target_host: &str, target_port: u16) -> String {
    format!(
        r#"map $http_upgrade $stackvo_upgrade {{
    default upgrade;
    ""      close;
}}

server {{
    listen {GUARD_PORT};
    server_name _;

    # A tunnel carries file uploads as often as webhooks, and nginx's own
    # default is 1M — small enough that the first thing anybody tries through
    # a shared link is the thing that fails.
    client_max_body_size 512m;

    # With "always", or nginx omits the challenge from its own 401 below and
    # the browser shows an error page instead of asking for a password.
    add_header WWW-Authenticate "Basic realm=\"{REALM}\"" always;

    location / {{
        # A comparison rather than a map, and that is a measurement rather
        # than a style: a map keyed on the credential needs a hash bucket
        # bigger than the credential, and nginx refused to start with
        # "could not build map_hash, you should increase map_hash_bucket_size"
        # on a password no longer than one somebody might actually choose.
        # This has no length to exceed.
        if ($http_authorization != "Basic __STACKVO_AUTH__") {{
            return 401;
        }}

        proxy_pass http://{target_host}:{target_port};
        proxy_http_version 1.1;

        # Passed through rather than rewritten: whether the application sees
        # its own domain or the tunnel's is the provider's business, and a
        # guard that quietly changed the answer would make Provider's
        # rewrites_host field a lie on half the table.
        proxy_set_header Host $http_host;

        # The credential stops here. It is the guard's, not the application's,
        # and an app that logs its request headers should not be logging a
        # password that opens its own front door.
        proxy_set_header Authorization "";

        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header X-Forwarded-Host $http_host;

        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $stackvo_upgrade;

        # Long enough for a websocket and for a slow first request against a
        # framework that is still booting.
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
    }}
}}
"#
    )
}

/// The `docker run` invocation for one project's guard.
///
/// The configuration is written **inside** the container from the environment
/// variable, rather than mounted from a file this app wrote: a file would put
/// the password in the workspace, and a workspace is a directory people copy,
/// back up and occasionally commit.
pub fn guard_args(
    project: &str,
    target_host: &str,
    target_port: u16,
    network: &str,
) -> Vec<String> {
    // nginx's own variables have to survive the shell, and the credential has
    // to be expanded by it. Escaping every `$` first and putting the one
    // expansion back afterwards is the order that cannot get them the wrong
    // way round.
    // MEASURED, and it cost a guard that would not start: an unquoted heredoc
    // performs command substitution, so a backtick anywhere in this
    // configuration — including in a comment — becomes `sh: always: not
    // found`. Both metacharacters are escaped, and the one expansion is put
    // back afterwards.
    let conf = guard_conf(target_host, target_port)
        .replace('\\', "\\\\")
        .replace('$', "\\$")
        .replace('`', "\\`")
        .replace("__STACKVO_AUTH__", &format!("${AUTH_ENV}"));

    // Assembled rather than formatted: an nginx configuration is mostly
    // braces, and every one of them would have to be doubled to survive a
    // format string. The heredoc is deliberately **unquoted**, which is what
    // lets the one expansion above happen inside the container.
    let mut script = String::new();
    script.push_str("set -e\n");
    script.push_str(&format!(
        "if [ -z \"${AUTH_ENV}\" ]; then echo 'no tunnel credential was passed'; exit 1; fi\n"
    ));
    script.push_str("cat >/etc/nginx/conf.d/default.conf <<STACKVO_CONF_END\n");
    script.push_str(&conf);
    script.push_str("STACKVO_CONF_END\n");
    script.push_str("exec nginx -g 'daemon off;'\n");

    [
        "run",
        "-d",
        "--name",
        &format!("stackvo-{}", guard_id(project)),
        "--network",
        network,
        "--label",
        &format!("{GUARD_LABEL}={project}"),
        // By name. The value comes from the child's environment, exactly the
        // way a provider token does, so the operation console prints a
        // variable name and never a password.
        "-e",
        AUTH_ENV,
        "--entrypoint",
        "sh",
        GUARD_IMAGE,
        "-c",
        &script,
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// ------------------------------------------------------------------ base64

/// Standard base64, the sixteen lines RFC 4648 describes.
///
/// Written here rather than taken as a dependency for the reason
/// [`crate::qr`] gives about its own encoder: this is a closed specification
/// with published test vectors, so it cannot acquire new cases later and the
/// usual argument for a maintained dependency does not apply. The vectors are
/// in the tests, verbatim from the RFC.
pub fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let bits = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        out.push(TABLE[(bits >> 18 & 0x3f) as usize] as char);
        out.push(TABLE[(bits >> 12 & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(bits >> 6 & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(bits & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

// ------------------------------------------------------------------ names

/// Where the reserved names live.
///
/// Beside the app's own state and **not** in the project's manifest, which was
/// the first design and is wrong: a reserved name belongs to an account on one
/// provider — an ngrok domain, a tailnet host, a zrok share — so a manifest
/// carrying one would travel to a colleague's checkout as a name their account
/// does not hold and their tunnel would silently not get.
pub fn names_path() -> Option<PathBuf> {
    crate::appdir::config().map(|dir| dir.join("tunnel-names.json"))
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Stored {
    #[serde(rename = "schemaVersion")]
    schema_version: u64,
    /// project -> provider -> the name that provider was asked to keep.
    #[serde(default)]
    projects: BTreeMap<String, BTreeMap<String, String>>,
}

fn load(path: &Path) -> Stored {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Stored {
            schema_version: SCHEMA_VERSION,
            projects: BTreeMap::new(),
        };
    };
    match serde_json::from_str::<Stored>(&text) {
        // An unknown version is not readable by definition — the field is
        // there to say so rather than to be guessed past.
        Ok(stored) if stored.schema_version == SCHEMA_VERSION => stored,
        _ => Stored {
            schema_version: SCHEMA_VERSION,
            projects: BTreeMap::new(),
        },
    }
}

/// Every reserved name this project has, by provider id.
pub fn names(project: &str) -> BTreeMap<String, String> {
    let Some(path) = names_path() else {
        return BTreeMap::new();
    };
    load(&path)
        .projects
        .get(project)
        .cloned()
        .unwrap_or_default()
}

/// The name one provider was asked to keep for this project.
pub fn name_of(project: &str, provider: &str) -> Option<String> {
    names(project).get(provider).cloned()
}

/// Store a reserved name, or forget it with `None`.
pub fn set_name(project: &str, provider: &str, name: Option<&str>) -> Result<()> {
    let path = names_path().ok_or_else(|| {
        Error::new(
            Code::IoError,
            "this system has no configuration directory to remember a tunnel name in",
        )
    })?;

    let mut stored = load(&path);
    let entry = stored.projects.entry(project.to_string()).or_default();
    match name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(name) => {
            entry.insert(provider.to_string(), name.to_string());
        }
        None => {
            entry.remove(provider);
        }
    }
    // An empty map for a project is the same as no project, and a file that
    // accumulates one key per project ever opened is one nobody can read.
    stored.projects.retain(|_, names| !names.is_empty());
    stored.schema_version = SCHEMA_VERSION;

    let text = serde_json::to_string_pretty(&stored)
        .map_err(|e| Error::new(Code::IoError, format!("could not write tunnel names: {e}")))?;
    crate::atomic::write(&path, &text)
}

/// Refuse a name a provider could not accept, before a pull rather than after.
///
/// `dotted` says whether this provider's field is a whole domain
/// (`shop.ngrok-free.app`) or one label of one (`shop`). Both are checked
/// against the hostname rules, because everything on this list ends up in a
/// DNS name and a rejected one costs an image pull, a start and a log read to
/// discover.
pub fn validate_name(name: &str, dotted: bool) -> Result<()> {
    let refuse = |why: &str| {
        Err(Error::new(
            Code::InvalidInput,
            format!("\"{name}\" cannot be a tunnel name: {why}"),
        ))
    };

    if name.is_empty() || name.len() > 253 {
        return refuse("it is empty or longer than a hostname may be");
    }
    if !dotted && name.contains('.') {
        return refuse("this provider takes one label, not a whole domain");
    }
    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return refuse("a label is empty or longer than 63 characters");
        }
        if label.starts_with('-') || label.ends_with('-') {
            return refuse("a label may not start or end with a hyphen");
        }
        if !label
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return refuse("only lowercase letters, digits and hyphens are allowed");
        }
    }
    Ok(())
}

/// Whether the address a client actually printed is the one that was asked
/// for.
///
/// The measurement this exists for is in the module note: localtunnel asked
/// for `stackvo-probe-11959` and got it, then asked again while the server
/// still held the name and got `bitter-bulldog-88.loca.lt` — with no error
/// anywhere. Whoever registered the first address in a dashboard would have a
/// tunnel that is up, a pane that says so, and webhooks arriving nowhere.
pub fn honoured(url: &str, reserved: &str) -> bool {
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or_default();
    // Either the whole hostname was reserved, or its first label was.
    host == reserved || host.split('.').next() == Some(reserved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rfc_4648_vectors() {
        // Section 10, verbatim. The one case that is not obvious is the last
        // chunk's padding, which is where every hand-written encoder is wrong.
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), expected, "{input:?}");
        }
    }

    /// The header value a browser sends, against the encoding of the example
    /// in RFC 7617 itself.
    #[test]
    fn the_header_is_what_a_browser_would_send() {
        let credentials = Credentials {
            user: "Aladdin".into(),
            password: "open sesame".into(),
        };
        assert_eq!(credentials.header_value(), "QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
    }

    #[test]
    fn a_password_may_hold_a_colon_and_a_user_name_may_not() {
        let credentials = Credentials::parse("stackvo:a:b:c").expect("a credential");
        assert_eq!(credentials.user, "stackvo");
        assert_eq!(credentials.password, "a:b:c");
        assert_eq!(credentials.stored(), "stackvo:a:b:c");

        assert!(Credentials {
            user: "sta:ckvo".into(),
            password: "x".into()
        }
        .validate()
        .is_err());
        // And half a credential is not one.
        assert!(Credentials::parse("stackvo").is_none());
        assert!(Credentials::parse("stackvo:").is_none());
        assert!(Credentials::parse(":secret").is_none());
    }

    #[test]
    fn a_credential_a_browser_cannot_send_is_refused_here() {
        for bad in ["par ola", "şifre", "pass\tword"] {
            assert!(
                Credentials {
                    user: "stackvo".into(),
                    password: bad.into()
                }
                .validate()
                .is_err(),
                "{bad:?} was accepted"
            );
        }
        assert!(Credentials {
            user: "stackvo".into(),
            password: "Kf3-xQ_9".into()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn a_generated_password_is_long_unambiguous_and_not_the_same_twice() {
        let first = generate(None);
        let second = generate(Some("  "));
        assert_eq!(first.user, DEFAULT_USER);
        assert_eq!(second.user, DEFAULT_USER, "blank is not a user name");
        assert_eq!(first.password.len(), PASSWORD_LEN);
        assert_ne!(first.password, second.password);
        assert!(first.validate().is_ok());

        // The characters that cost somebody two minutes on a phone keyboard.
        for ch in ['0', 'O', '1', 'l', 'I'] {
            assert!(
                !first.password.contains(ch),
                "{ch} is in {}",
                first.password
            );
        }
        assert_eq!(generate(Some("owner")).user, "owner");
    }

    #[test]
    fn the_guard_asks_for_a_credential_and_proxies_only_when_it_gets_one() {
        let conf = guard_conf("stackvo-shop", 80);
        assert!(conf.contains("listen 8080;"), "{conf}");
        assert!(
            conf.contains(r#"if ($http_authorization != "Basic __STACKVO_AUTH__")"#),
            "{conf}"
        );
        assert!(conf.contains("return 401;"), "{conf}");
        assert!(
            conf.contains("proxy_pass http://stackvo-shop:80;"),
            "{conf}"
        );
        // Without `always` nginx drops the challenge from its own 401 and the
        // browser shows an error page instead of a password prompt.
        assert!(
            conf.contains("WWW-Authenticate") && conf.contains("always;"),
            "{conf}"
        );
        // The credential is the guard's business and not the application's.
        assert!(
            conf.contains(r#"proxy_set_header Authorization "";"#),
            "{conf}"
        );
        // A websocket has to survive the hop.
        assert!(
            conf.contains("proxy_set_header Upgrade $http_upgrade;"),
            "{conf}"
        );
    }

    #[test]
    fn the_password_never_reaches_the_argument_list() {
        let args = guard_args("shop", "stackvo-shop", 80, "stackvo-net");
        let line = args.join(" ");

        assert!(line.contains("--name stackvo-tunnel-guard-shop"), "{line}");
        assert!(line.contains("--network stackvo-net"), "{line}");
        assert!(
            line.contains(&format!("--label {GUARD_LABEL}=shop")),
            "{line}"
        );
        // By name only, exactly as a provider token travels.
        assert!(line.contains(&format!("-e {AUTH_ENV}")), "{line}");
        assert!(!line.contains(&format!("{AUTH_ENV}=")), "{line}");
        // The one expansion is the container's, and every nginx variable
        // survives the shell rather than being emptied by it.
        assert!(line.contains(&format!("Basic ${AUTH_ENV}")), "{line}");
        assert!(line.contains("\\$http_authorization"), "{line}");
        assert!(line.contains("\\$proxy_add_x_forwarded_for"), "{line}");
        assert!(!line.contains(" $http_authorization"), "{line}");
        // A guard with no credential is a guard that lets everybody through,
        // so it refuses to start instead.
        assert!(line.contains("no tunnel credential was passed"), "{line}");
        assert!(line.contains("exec nginx -g 'daemon off;'"), "{line}");
    }

    #[test]
    fn the_keystore_entry_is_per_project() {
        assert_eq!(secret_name("shop"), "tunnel-auth:shop");
        assert_ne!(secret_name("shop"), secret_name("blog"));
        // And it is not the provider token's entry, which is per machine.
        assert_ne!(secret_name("ngrok"), crate::tunnel::secret_name("ngrok"));
    }

    #[test]
    fn a_name_that_could_not_be_a_hostname_is_refused_before_the_pull() {
        assert!(validate_name("shop-dev", false).is_ok());
        assert!(validate_name("shop.ngrok-free.app", true).is_ok());

        // One label where a whole domain was given.
        assert!(validate_name("shop.ngrok-free.app", false).is_err());
        for bad in [
            "",
            "-shop",
            "shop-",
            "SHOP",
            "sh op",
            "shop_dev",
            "shop..dev",
        ] {
            assert!(validate_name(bad, true).is_err(), "{bad:?} was accepted");
        }
    }

    /// The measured case: a provider that quietly hands back a different name.
    #[test]
    fn an_address_that_is_not_the_reserved_one_is_not_honoured() {
        assert!(honoured(
            "https://stackvo-probe-11959.loca.lt",
            "stackvo-probe-11959"
        ));
        assert!(!honoured(
            "https://bitter-bulldog-88.loca.lt",
            "stackvo-probe-11959"
        ));
        // ngrok reserves the whole hostname rather than one label.
        assert!(honoured(
            "https://shop.ngrok-free.app",
            "shop.ngrok-free.app"
        ));
        // A name that is merely *contained* in the address is not the address:
        // `shop-2.loca.lt` is not `shop`.
        assert!(!honoured("https://shop-2.loca.lt", "shop"));
    }

    #[test]
    fn names_round_trip_through_the_file_and_an_unreadable_one_is_empty() {
        let dir = std::env::temp_dir().join(format!("stackvo-tunnelid-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a temp dir");
        let path = dir.join("tunnel-names.json");

        // Nothing there yet is not a failure; it is a machine that has never
        // reserved a name.
        assert!(load(&path).projects.is_empty());

        let stored = Stored {
            schema_version: SCHEMA_VERSION,
            projects: BTreeMap::from([(
                "shop".to_string(),
                BTreeMap::from([("ngrok".to_string(), "shop.ngrok-free.app".to_string())]),
            )]),
        };
        std::fs::write(&path, serde_json::to_string(&stored).unwrap()).unwrap();
        assert_eq!(load(&path).projects["shop"]["ngrok"], "shop.ngrok-free.app");

        // A shape from a later version is not read as if it were this one.
        std::fs::write(&path, r#"{"schemaVersion":99,"projects":{"shop":{}}}"#).unwrap();
        assert!(load(&path).projects.is_empty());

        // And nonsense is a fresh start rather than a failure to open a pane.
        std::fs::write(&path, "{ not json").unwrap();
        assert!(load(&path).projects.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
