//! Reading the StackVo `.env`, exactly as the Bash loader and the Node parser
//! read it — see `contracts/env.schema.json` → `parsing.rules`.
//!
//! Naive on purpose: first `=` wins, no unquoting, no interpolation. Being
//! cleverer than the Bash loader would mean the two tools disagree about what a
//! line means, which is the drift this contract exists to prevent.

use crate::error::{Error, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// What a secret looks like once it has crossed the IPC boundary.
///
/// One constant rather than a literal at each site, because it is compared as
/// well as written: the front end tells a masked value from a real one by
/// matching this exact string, and a stray sixth bullet in one of four places
/// would be a value the UI treats as somebody's password. `env_reveal` is the
/// one deliberate way the real thing is asked for.
pub const MASK: &str = "••••••••";

#[derive(Debug, Clone, Default)]
pub struct Env {
    vars: BTreeMap<String, String>,
    /// The keys the **file** set, as opposed to the ones [`EMBEDDED`] supplied.
    ///
    /// [`Self::get`] answers from the merged map, which is right for almost
    /// every caller: a default the app knows is a perfectly good answer. It is
    /// wrong for exactly one shape of question — "did somebody *choose* this" —
    /// and `db.rs` asks it, because its resolution order puts `.env` ahead of
    /// what the package declares. See [`Self::stated`].
    ///
    /// `parse` already had to keep this apart for the alias chain, and the
    /// comment there records what it cost to learn: while both spellings could
    /// only come from the file, resolving against the merged map worked, and
    /// the day the current name shipped as an embedded default a checkout
    /// asking for Apache was quietly served nginx. This is the same distinction
    /// kept rather than thrown away at the end of `parse`.
    stated: BTreeSet<String>,
    /// Keys whose `.env` value pointed at the keystore and got no answer.
    ///
    /// Carried rather than logged because the one caller that must not proceed
    /// — the generator — is not the one that loads the file. See
    /// [`Self::unresolved_secrets`].
    unresolved: Vec<String>,
}

/// Values this app knows rather than values the user chose.
///
/// They shipped in `.env` because the shell generator had nowhere else to put
/// them, and that made them stale by design: a workspace created last year
/// still offers last year's PHP versions, because the list lives in a file the
/// app never updates. Here they travel with the binary — new PHP release, new
/// build, new list.
///
/// **Defaults, not constants.** `.env` still wins, so anyone who does want to
/// pin a container name or trim the catalog writes the key and it takes
/// effect. Nothing became unreachable; it stopped having to be copied.
///
/// Some of these *are* choices — the domain suffix, whether TLS is on — and
/// they stay editable in Settings, which writes the key to `.env` when it is
/// changed. What moved is only the default: a fresh workspace no longer ships
/// seven lines restating what the app would have done anyway, and a `.env`
/// line now means somebody decided something rather than that a file was
/// copied.
///
/// These are the ones that **stay**. The service half is next door and is not
/// the same kind of thing — see [`LEGACY_SERVICES`].
pub const SETTINGS: [(&str, &str); 36] = [
    // HOST_UID and HOST_GID are deliberately absent. `template::variables`
    // fills them from getuid()/getgid() when nothing else has, and it does that
    // only for keys that are missing — embedding them pinned one machine's ids
    // into every install, so Grafana would have run as uid 501 on a Linux box
    // where the developer is 1000.
    // Off, and off is the default that matters: a project that stops
    // behind somebody's back and then answers 502 is worse than one that stays
    // up, so this is asked for rather than assumed.
    ("IDLE_SUSPEND_MINUTES", "0"),
    ("SUPPORTED_SERVERS", "nginx,apache,caddy,frankenphp,swoole,roadrunner"),
    ("SUPPORTED_SERVERS_DEFAULT", DEFAULT_SERVER),
    ("SUPPORTED_LANGUAGES", "php,python,go,ruby,rust,nodejs"),
    ("SUPPORTED_LANGUAGES_PHP_VERSIONS", "5.6,7.0,7.1,7.2,7.3,7.4,8.0,8.1,8.2,8.3,8.4,8.5"),
    ("SUPPORTED_LANGUAGES_PHP_DEFAULT", "8.4"),
    ("SUPPORTED_LANGUAGES_PHP_EXTENSIONS", "apcu,bcmath,bz2,calendar,ctype,curl,dba,dom,enchant,ev,event,exif,ffi,fileinfo,filter,ftp,gd,gettext,gmp,hash,iconv,igbinary,imagick,imap,intl,json,ldap,lz4,mbstring,mcrypt,memcache,memcached,mongodb,monolog,mysqli,mysqlnd,odbc,opcache,openswoole,openssl,pcntl,pdo,pdo_dblib,pdo_mysql,pdo_oci,pdo_odbc,pdo_pgsql,pdo_sqlite,pdo_sqlsrv,pgsql,phalcon,phar,posix,pspell,readline,redis,session,shmop,simplexml,soap,sockets,sodium,sqlite3,sqlsrv,swoole,sysvmsg,sysvsem,sysvshm,tokenizer,uv,xdebug,xml,xmlreader,xmlrpc,xmlwriter,xsl,zip,zlib"),
    ("SUPPORTED_LANGUAGES_PHP_EXTENSIONS_DEFAULT", "mbstring,tokenizer,xml,ctype,json,openssl,pdo,pdo_mysql,pdo_pgsql,pdo_sqlite,fileinfo,curl,zip,gd,imagick,intl,sodium,bcmath,redis,opcache,memcached,mongodb,swoole,soap,dom,filter,hash,pcntl,session,xmlreader,xmlwriter,xdebug"),
    ("SUPPORTED_LANGUAGES_PYTHON_VERSIONS", "2.7,3.5,3.6,3.7,3.8,3.9,3.10,3.11,3.12,3.13,3.14"),
    ("SUPPORTED_LANGUAGES_PYTHON_DEFAULT", "3.14"),
    ("SUPPORTED_LANGUAGES_GO_VERSIONS", "1.11,1.12,1.13,1.14,1.15,1.16,1.17,1.18,1.19,1.20,1.21,1.22,1.23"),
    ("SUPPORTED_LANGUAGES_GO_DEFAULT", "1.23"),
    ("SUPPORTED_LANGUAGES_RUBY_VERSIONS", "2.4,2.5,2.6,2.7,3.0,3.1,3.2,3.3"),
    ("SUPPORTED_LANGUAGES_RUBY_DEFAULT", "3.3"),
    ("SUPPORTED_LANGUAGES_RUST_VERSIONS", "1.70,1.72,1.74,1.75,1.76,1.78,1.80,1.81,1.82,1.83,1.84"),
    ("SUPPORTED_LANGUAGES_RUST_DEFAULT", "1.84"),
    ("SUPPORTED_LANGUAGES_NODEJS_VERSIONS", "16,18,20,21,22,23"),
    ("SUPPORTED_LANGUAGES_NODEJS_DEFAULT", "22"),
    // Stack-shaping choices. Editable in Settings; absent from a fresh `.env`
    // because the default is the answer almost everyone keeps.
    ("DEFAULT_TLD_SUFFIX", DEFAULT_TLD_SUFFIX),
    ("SERVER_MAX_BODY_SIZE", "1m"),
    ("SERVER_FASTCGI_TIMEOUT", "60"),
    ("SERVER_CLIENT_BODY_TIMEOUT", "60"),
    ("SERVER_KEEPALIVE_TIMEOUT", "75"),
    ("SERVER_TCP_NODELAY", "on"),
    ("SERVER_GZIP", "off"),
    ("SERVER_GZIP_COMP_LEVEL", "1"),
    ("SERVER_GZIP_TYPES", ""),
    ("SERVER_FASTCGI_CONNECT_TIMEOUT", "60"),
    ("SERVER_FASTCGI_SEND_TIMEOUT", "60"),
    ("SSL_ENABLE", "true"),
    ("REDIRECT_TO_HTTPS", "true"),
    ("DOCKER_DEFAULT_NETWORK", DEFAULT_NETWORK),
    ("PHP_DEFAULT_TOOLS", "composer,nodejs"),
    ("PHP_TOOL_COMPOSER_VERSION", "latest"),
    ("PHP_TOOL_NODEJS_VERSION", "20"),
    (
        "PHP_DEFAULT_APT_PACKAGES",
        "git,wget,unzip,default-mysql-client,postgresql-client,redis-tools,strace,vim,nano,curl,\
         iputils-ping,net-tools,telnet,htop,procps,tar,gzip,bzip2,p7zip-full",
    ),
];

/// The service half. Seventy-eight keys, and not all of them are legacy.
///
/// The `.env` branch of the renderer is gone: services come from the instance
/// table and packages now, and `skeleton/core/templates/services/`
/// left the binary with it. This constant stayed behind, and the sentence that
/// kept it said [`crate::handover`] needs it — a `.env` that predates the market
/// says `SERVICE_MYSQL_ENABLE=true` and nothing else, so without a `VERSION`
/// default beside it there would be no tag to migrate.
///
/// ## That sentence was not true, and the way to find out was to try it
///
/// `npm run legacy:rehearse` empties this constant, runs the suite and puts the
/// tree back. With the whole half gone, `handover_equivalence.rs` passes **13 of
/// 13**: every image, port and volume preserved, every refusal still refusing.
/// The migration never needed a default, because [`crate::handover::plan`] does
/// not read one — where the `.env` states no version it asks the catalogue
/// (`catalogue.recommended`), which is the whole point of services being
/// dynamic. Not one failing test was the migration.
///
/// What was actually holding it up was [`crate::mail`]: `detect` decided which
/// catcher a workspace knows about by asking whether a key was **present**, and
/// on an untouched workspace the only thing making `SERVICE_MAILPIT_ENABLE`
/// present was this table. A panel's behaviour hanging off a constant named for
/// migration. That question was pointed at the catalogue, where the vocabulary
/// already lived, and **seventy-two keys left with it** — every
/// `_ENABLE`, `_VERSION` and `_VERSIONS`, which were the `.env` shadow of a
/// catalogue that had moved into packages. 150 became 78.
///
/// Nothing was lost with them. All twenty-five `_ENABLE` defaults were `"false"`
/// and [`Env::bool`] reads a missing key as false, so as *values* they were the
/// identity; their only job was to be present, and that job is the catalogue's.
///
/// ## What is left, and why it is not inert
///
/// All seventy-eight are the same old `.env` family, and the package manifest
/// took over every one of their jobs: `url` for `_URL`, `ports` for
/// `_HOST_PORT`, `settings` for the passwords and panel options.
/// `package-version.schema.json` says so in as many words — a setting's `key`
/// is "the bare key, without the `SERVICE_<ID>_` prefix the old .env family
/// carried". Every reader is a correctly gated pre-migration branch:
/// `commands.rs` tries `instance_domains` (the table, plus `manifest.url`)
/// first, [`crate::db`] tries the manifest's `connection` block first.
///
/// So they *look* like migration residue. One thing stops them being it, and
/// it is not in this file. [`crate::db`]'s `Value::or_env` resolves
/// `stored → .env → manifest default`, and `.env` is in the middle on purpose:
/// a handover leaves the real password there while the manifest declares a
/// placeholder, so preferring the manifest would hand a dump the wrong password
/// with no readable error. But the `.env` it consults is a merged [`Env`], and
/// [`Env::parse`] lays these defaults **under** the file. On a workspace
/// installed from packages, with no `.env` at all,
/// `SERVICE_MYSQL_ROOT_PASSWORD` still answers `"root"` out of the binary — and
/// by that order it beats what the package declares, which is the reverse of
/// what the package system is for.
///
/// Pinned by `db::tests::stored_beats_env_beats_the_packages_default`, in the
/// state `mail::detect` was in before anybody went looking: described, relied
/// upon, and asserted nowhere.
///
/// A new service arriving as a package must still not gain a key here — gaining
/// one would mean the app had an opinion about a service it does not ship.
///
/// ## What deletes the rest
///
/// The migration cutoff, and for the first time that is all of it. The last
/// non-migration reader closed when [`Env::stated`] arrived: `db.rs`'s middle
/// slot now takes only what the file wrote, so an embedded default cannot reach
/// it and a workspace installed from packages takes its password from the
/// package. Every remaining reader is the migration or a branch that runs only
/// before it.
///
/// The engineering around it is arranged: the keys are one constant rather than
/// lines mixed into 36 others, the split is held by
/// `config::tests::the_two_halves_partition_the_defaults`, every module that
/// reads one is named by `legacy_env_claims.rs`, and every site that *fails*
/// without one is named by the rehearsal — which is the half a grep cannot
/// produce, because a test can lean on a default without naming a key.
///
/// Credentials are **not** absent, and the paragraph here used to say they
/// were. They start at `SERVICE_MYSQL_ROOT_PASSWORD` below, with the comment
/// explaining why placeholders every install shares are not secrets. The claim
/// survived because nothing read it.
pub const LEGACY_SERVICES: [(&str, &str); 78] = [
    ("SERVICE_RABBITMQ_URL", "rabbitmq"),
    ("SERVICE_KIBANA_URL", "kibana"),
    ("SERVICE_GRAFANA_URL", "grafana"),
    ("SERVICE_MAILHOG_URL", "mailhog"),
    ("SERVICE_MAILPIT_URL", "mailpit"),
    ("SERVICE_PHPMYADMIN_URL", "phpmyadmin"),
    ("SERVICE_PHPMYADMIN_ARBITRARY", "1"),
    ("SERVICE_PHPMYADMIN_HOST", "stackvo-mysql"),
    ("SERVICE_PHPMYADMIN_UPLOAD_LIMIT", "300M"),
    ("SERVICE_ADMINER_URL", "adminer"),
    ("SERVICE_ADMINER_DEFAULT_SERVER", "stackvo-mysql"),
    ("SERVICE_ADMINER_DESIGN", "pepa-linha"),
    ("SERVICE_PGADMIN_URL", "pgadmin"),
    ("SERVICE_PGADMIN_SERVER_MODE", "False"),
    ("SERVICE_PGADMIN_MASTER_PASSWORD_REQUIRED", "False"),
    ("SERVICE_KAFBAT_URL", "kafbat"),
    ("SERVICE_KAFBAT_DYNAMIC_CONFIG", "true"),
    ("SERVICE_KAFBAT_CLUSTER_NAME", "stackvo-kafka"),
    ("SERVICE_KAFBAT_BOOTSTRAP_SERVERS", "stackvo-kafka:9092"),
    ("SERVICE_MONGO_EXPRESS_URL", "mongo-express"),
    ("SERVICE_MONGO_EXPRESS_MONGODB_SERVER", "stackvo-mongo"),
    ("SERVICE_MONGO_EXPRESS_BASEURL", "/"),
    ("SERVICE_PHPCACHEADMIN_URL", "phpcacheadmin"),
    ("SERVICE_PHPCACHEADMIN_REDIS_HOST", "stackvo-redis"),
    ("SERVICE_PHPCACHEADMIN_MEMCACHED_HOST", "stackvo-memcached"),
    // MinIO's domain reaches the console rather than the S3 API — the API is
    // addressed by an endpoint URL an SDK already holds, and giving it a second
    // name would be a name that works for a browser and not for a client.
    ("SERVICE_MINIO_URL", "minio"),
    ("SERVICE_MINIO_REGION", "us-east-1"),
    ("SERVICE_MEILISEARCH_URL", "meilisearch"),
    ("SERVICE_TYPESENSE_URL", "typesense"),
    // Per-service defaults. The Services pane edits these, and writes the key
    // to `.env` when one is changed — so a fresh workspace ships no service
    // configuration at all, and a line in that file means a decision.
    //
    // Credentials ARE here, further down, and this comment used to say they
    // were deliberately not. They start at `SERVICE_MYSQL_ROOT_PASSWORD` with
    // the note explaining why placeholders every install shares are not
    // secrets. Nothing read the claim, so nothing caught it.
    ("SERVICE_ADMINER_HOST_PORT", "8082"),
    (
        "SERVICE_ELASTICSEARCH_VERSIONS",
        "9.4.4,9.3.8,8.19.19,8.11.3,7.17.28",
    ),
    ("SERVICE_GRAFANA_ADMIN_USER", "admin"),
    ("SERVICE_KAFBAT_HOST_PORT", "8080"),
    (
        "SERVICE_KIBANA_VERSIONS",
        "9.4.4,9.3.8,8.19.19,8.11.3,7.17.28",
    ),
    ("SERVICE_MARIADB_DATABASE", "stackvo"),
    ("SERVICE_MEILISEARCH_HOST_PORT", "7700"),
    // Two published ports, and both are named: the S3 API is what an SDK
    // connects to and the console is what a browser opens. One key for "the
    // MinIO port" would silently be the wrong one half the time.
    ("SERVICE_MINIO_CONSOLE_HOST_PORT", "9001"),
    ("SERVICE_MINIO_HOST_PORT", "9000"),
    (
        "SERVICE_MINIO_VERSIONS",
        "RELEASE.2025-09-07T16-13-09Z,RELEASE.2025-07-23T15-54-02Z,\
         RELEASE.2025-06-13T11-33-47Z,RELEASE.2025-04-22T22-12-26Z",
    ),
    ("SERVICE_MONGO_EXPRESS_ADMIN_USERNAME", "root"),
    ("SERVICE_MONGO_EXPRESS_BASICAUTH_USERNAME", "admin"),
    ("SERVICE_MONGO_EXPRESS_HOST_PORT", "8083"),
    ("SERVICE_MONGO_EXPRESS_MONGODB_PORT", "27017"),
    ("SERVICE_MONGO_INITDB_ROOT_USERNAME", "root"),
    ("SERVICE_MYSQL_DATABASE", "stackvo"),
    ("SERVICE_PGADMIN_DEFAULT_EMAIL", "admin@stackvo.loc"),
    ("SERVICE_PGADMIN_HOST_PORT", "5050"),
    ("SERVICE_PHPCACHEADMIN_ADMIN_USER", "admin"),
    ("SERVICE_PHPCACHEADMIN_HOST_PORT", "8084"),
    ("SERVICE_PHPCACHEADMIN_MEMCACHED_PORT", "11211"),
    ("SERVICE_PHPCACHEADMIN_REDIS_PORT", "6379"),
    ("SERVICE_PHPMYADMIN_HOST_PORT", "8081"),
    ("SERVICE_PHPMYADMIN_PORT", "3306"),
    ("SERVICE_POSTGRES_DB", "stackvo"),
    ("SERVICE_POSTGRES_USER", "stackvo"),
    ("SERVICE_RABBITMQ_DEFAULT_USER", "admin"),
    ("SERVICE_TYPESENSE_HOST_PORT", "8108"),
    // Not 6379. Valkey speaks Redis's protocol, so the case for having it is
    // usually "move this project off Redis" — which means both running at once,
    // and a shared port makes whichever starts second fail to bind.
    ("SERVICE_VALKEY_HOST_PORT", "6381"),
    // Ports and starting credentials, so a workspace ships no `.env` content at
    // all. These are placeholders every install shares, not secrets: they are
    // in this file, which is public, and were in the committed `.env.example`
    // before that. Changing one writes it to `.env`, where it stays private —
    // and `no_real_credential_is_compiled_into_the_binary` below is what keeps
    // a real one from ever being pasted in here.
    ("SERVICE_POSTGRES_HOST_PORT", "5432"),
    ("SERVICE_KAFKA_HOST_PORT", "9092"),
    ("SERVICE_KAFKA_EXTERNAL_HOST_PORT", "29092"),
    ("SERVICE_MYSQL_ROOT_PASSWORD", "root"),
    ("SERVICE_MARIADB_ROOT_PASSWORD", "root"),
    ("SERVICE_POSTGRES_PASSWORD", "root"),
    ("SERVICE_MONGO_INITDB_ROOT_PASSWORD", "root"),
    ("SERVICE_REDIS_PASSWORD", ""),
    ("SERVICE_RABBITMQ_DEFAULT_PASS", "admin"),
    ("SERVICE_GRAFANA_ADMIN_PASSWORD", "admin"),
    ("SERVICE_PGADMIN_DEFAULT_PASSWORD", "admin"),
    ("SERVICE_MONGO_EXPRESS_ADMIN_PASSWORD", "root"),
    ("SERVICE_MONGO_EXPRESS_BASICAUTH_PASSWORD", "admin"),
    ("SERVICE_PHPCACHEADMIN_ADMIN_PASS", "admin"),
    ("SERVICE_BLACKFIRE_SERVER_ID", ""),
    ("SERVICE_BLACKFIRE_SERVER_TOKEN", ""),
    // MinIO refuses to start on a root password shorter than eight characters,
    // so this one cannot be `root` like the databases above it. The value is
    // MinIO's own documented placeholder, which is the point: it is the string
    // somebody recognises as "still the default".
    ("SERVICE_MINIO_ROOT_USER", "minioadmin"),
    ("SERVICE_MINIO_ROOT_PASSWORD", "minioadmin"),
    ("SERVICE_MEILISEARCH_MASTER_KEY", "stackvo-master-key"),
    ("SERVICE_TYPESENSE_API_KEY", "stackvo-api-key"),
];

/// The domain suffix every service sits under when nothing chooses another.
///
/// One literal, and it used to be eleven. `certs::suffix` derived this value
/// one way — trimmed, lower-cased, an empty string treated as absent — and ten
/// call sites derived it another, with a bare `unwrap_or("stackvo.loc")`. Same
/// key, two answers, and the divergence was measurable rather than theoretical:
///
/// ```text
///   DEFAULT_TLD_SUFFIX=            call sites ""          certs::suffix "stackvo.loc"
///   DEFAULT_TLD_SUFFIX=Shop.LOC    call sites "Shop.LOC"  certs::suffix "shop.loc"
/// ```
///
/// The empty case is the one that bites. `Env::parse` lets a file's empty value
/// win over the embedded default, so a `.env` carrying the key with nothing
/// after it gave every project a domain ending in a bare dot while the
/// certificate was still issued for `stackvo.loc`.
///
/// So there is one derivation now — [`Env::tld_suffix`] — and one literal, here,
/// which `EMBEDDED` and `certs::FALLBACK_SUFFIX` both point at.
pub const DEFAULT_TLD_SUFFIX: &str = "stackvo.loc";

/// The Docker network every generated compose file joins.
///
/// Same story, smaller: five sites, four of them `unwrap_or("stackvo-net")`.
/// See [`Env::docker_network`]. A network name is not case-folded — Docker's
/// names are case-sensitive — so that derivation only trims and rejects empty.
pub const DEFAULT_NETWORK: &str = "stackvo-net";

/// The web server a project that did not choose one runs.
///
/// Written five times as `unwrap_or("nginx")` in the render path and read from
/// the setting in exactly one place — `Catalog.default_server`, which only
/// decides what the new-project wizard shows preselected. The consequence was
/// measurable and is the reason this constant exists: with
/// `SUPPORTED_SERVERS_DEFAULT=caddy` a project opened from the wizard ran
/// Caddy, an adopted project ran nginx, and a manifest with its `server` line
/// deleted ran nginx. One setting, three answers.
///
/// [`Env::default_server`] is the one derivation now, and
/// [`crate::manifest::Manifest::server_or`] is where a project that did not
/// choose gets it.
pub const DEFAULT_SERVER: &str = "nginx";

/// [`Env::tld_suffix`], for a caller holding the raw value rather than an `Env`.
///
/// The generator reaches this one: it resolves its settings into a plain map
/// before rendering, so it has the string and not the reader. Two entry points,
/// one rule — which is the point of the whole change.
pub fn tld_suffix_of(raw: Option<&str>) -> String {
    raw.map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_TLD_SUFFIX.to_string())
}

/// [`Env::docker_network`], for the same caller.
pub fn docker_network_of(raw: Option<&str>) -> String {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_NETWORK)
        .to_string()
}

/// Both halves, in the order [`Env::parse`] lays them down.
///
/// One name because every consumer wants the merged view — the settings form,
/// the credential guard, the "is this a decision or a default" question. The
/// split above is about what *leaves* later, not about what anybody reads now,
/// and giving callers two constants to remember would be a way for one of them
/// to be forgotten on the day the second is deleted.
pub const EMBEDDED: [(&str, &str); 114] = both_halves();

/// Concatenation, at compile time.
///
/// A `const fn` rather than a `LazyLock` or a `Vec` built on demand: this is
/// read on every `Env::parse`, and the two halves are literals. Nothing here
/// is worth a heap allocation or a synchronisation primitive.
const fn both_halves() -> [(&'static str, &'static str); 114] {
    let mut out = [("", ""); 114];
    let mut i = 0;
    while i < SETTINGS.len() {
        out[i] = SETTINGS[i];
        i += 1;
    }
    let mut j = 0;
    while j < LEGACY_SERVICES.len() {
        out[SETTINGS.len() + j] = LEGACY_SERVICES[j];
        j += 1;
    }
    out
}

/// Older spellings, and what they mean now: `(legacy, current)`.
///
/// StackVo renamed these and kept reading both. The names are not
/// interchangeable in one direction — the current name is what the code asks
/// for — so the legacy value is copied forward at parse time rather than
/// checked at every call site.
const ALIASES: [(&str, &str); 6] = [
    ("DEFAULT_SERVER", "SUPPORTED_SERVERS_DEFAULT"),
    ("DEFAULT_PHP_VERSION", "SUPPORTED_LANGUAGES_PHP_DEFAULT"),
    ("SUPPORTED_WEBSERVERS", "SUPPORTED_SERVERS"),
    // Three port keys that never followed the convention the other six do.
    // Renamed rather than left alone because the odd ones out are the reason
    // somebody looks for `SERVICE_POSTGRES_HOST_PORT`, does not find it, and
    // concludes the port cannot be changed. An existing checkout keeps its
    // spelling through this table, so nothing moves ports on an upgrade.
    ("HOST_PORT_POSTGRES", "SERVICE_POSTGRES_HOST_PORT"),
    ("HOST_PORT_KAFKA", "SERVICE_KAFKA_HOST_PORT"),
    (
        "HOST_PORT_KAFKA_EXTERNAL",
        "SERVICE_KAFKA_EXTERNAL_HOST_PORT",
    ),
];

impl Env {
    /// A missing `.env` is not an error.
    ///
    /// The file holds overrides, and having none is the normal state of a
    /// fresh workspace — it is created the first time Settings writes a key.
    /// Failing here instead would make every command in the app depend on a
    /// file whose entire purpose is to be optional.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(".env");
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(Error::io(format!("reading {}", path.display()), e)),
        };
        let mut env = Self::parse(&text);
        env.apply_policy(crate::policy::current());
        // After the policy, so an administrator can push a reference too, and
        // only in `load`: `parse` stays a pure function of its argument, which
        // is what lets a hundred tests build an `Env` from a string without a
        // keychain prompt.
        env.unresolved = crate::secrets::resolve(&mut env.vars);
        Ok(env)
    }

    /// Keys that name a keystore entry the keystore would not produce.
    ///
    /// Empty on the overwhelming majority of machines, because it is empty
    /// unless somebody has moved a password. When it is not empty the value is
    /// **missing**, not blank — see [`crate::secrets::resolve`] for why that
    /// distinction is load-bearing — and `render_generated` refuses rather than
    /// writing a compose file with a hole in it.
    pub fn unresolved_secrets(&self) -> &[String] {
        &self.unresolved
    }

    /// Let an administrator's policy have the last word.
    ///
    /// Precedence is embedded default < `.env` < policy, and the order is the
    /// decision: a setting pushed to a fleet that a stale `.env` silently
    /// overrides is not a policy, it is a suggestion. Applied in [`Self::load`]
    /// only — [`Self::parse`] stays pure so every test that builds an `Env`
    /// from a string keeps getting exactly the string it wrote.
    pub fn apply_policy(&mut self, policy: &crate::policy::Policy) {
        for (key, value) in policy.settings() {
            self.vars.insert(key.clone(), value.clone());
        }
    }

    pub fn parse(text: &str) -> Self {
        let mut from_file: BTreeMap<String, String> = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                from_file.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        // The embedded values go in first so the file can overwrite any of
        // them. Reversing this would make them constants, which is a
        // different promise from the one being made.
        let mut vars: BTreeMap<String, String> = EMBEDDED
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();

        // An older name is honoured only when the file does not also carry the
        // current one, so a checkout that spells a setting the old way still
        // gets what it asked for.
        //
        // This has to look at what the *file* set, not at the merged map.
        // Callers used to resolve these chains themselves with `first_of`,
        // which worked while both names could only come from the file and
        // broke the moment the current name shipped as an embedded default:
        // the first arm then always answered, the alias never fired, and a
        // checkout asking for Apache was quietly served nginx.
        let mut stated: BTreeSet<String> = BTreeSet::new();
        for (legacy, current) in ALIASES {
            if let Some(value) = from_file.get(legacy) {
                if !from_file.contains_key(current) {
                    vars.insert(current.to_string(), value.clone());
                    // Written under its current name, so a caller asking
                    // whether the file chose it gets yes. It did — in the older
                    // spelling, which is the whole point of the chain.
                    stated.insert(current.to_string());
                }
            }
        }

        stated.extend(from_file.keys().cloned());
        vars.extend(from_file);
        Self {
            vars,
            stated,
            unresolved: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }

    /// The domain suffix every service sits under.
    ///
    /// The one derivation, and it exists because there used to be two — see
    /// [`DEFAULT_TLD_SUFFIX`] for the measurement. Lower-cased because a domain
    /// is case-insensitive and the certificate is issued in one case: a `.env`
    /// spelling it `Shop.LOC` otherwise gave the generator one string and
    /// `mkcert` another. Empty means absent, because [`Self::parse`] lets a
    /// file's empty value win over the embedded default, and "the key is there
    /// with nothing after it" is not a choice of suffix.
    pub fn tld_suffix(&self) -> String {
        tld_suffix_of(self.get("DEFAULT_TLD_SUFFIX"))
    }

    /// The Docker network every generated compose file joins.
    ///
    /// Not case-folded, unlike [`Self::tld_suffix`]: Docker network names are
    /// case-sensitive, so lowering one would name a network that does not
    /// exist. Empty is still absent, for the same reason as there.
    pub fn docker_network(&self) -> String {
        docker_network_of(self.get("DOCKER_DEFAULT_NETWORK"))
    }

    /// The web server a project that did not choose one runs.
    ///
    /// See [`DEFAULT_SERVER`]. Not case-folded: a server id is matched against
    /// `Server::parse`'s own list, and lowering one here would only hide a
    /// misspelling that ought to be reported.
    pub fn default_server(&self) -> String {
        self.get("SUPPORTED_SERVERS_DEFAULT")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_SERVER)
            .to_string()
    }

    /// What the **file** says, with no embedded default underneath it.
    ///
    /// [`Self::get`] is right for almost everything: a value the app knows is a
    /// good answer, and that is what [`EMBEDDED`] is for. This is for the one
    /// question where a default is not an answer — *did somebody choose this* —
    /// and [`crate::db`] is the caller that has to ask.
    ///
    /// Its resolution order is `stored → .env → the package's default`, with
    /// `.env` deliberately ahead of the package: a handover leaves the real
    /// password in `.env` while the manifest declares a placeholder, so
    /// preferring the manifest would hand a dump the wrong password with no
    /// readable error. Reading that middle slot with `get` put a third thing in
    /// it — on a workspace installed from packages, with no `.env` at all,
    /// `SERVICE_MYSQL_ROOT_PASSWORD` answered `"root"` out of the binary and
    /// **beat what the package declared**, which is backwards.
    ///
    /// So the middle slot asks this instead, and the embedded service defaults
    /// go back to being what they are: values for a workspace that has a
    /// pre-market `.env`, which is the migration and nothing else.
    pub fn stated(&self, key: &str) -> Option<&str> {
        if !self.stated.contains(key) {
            return None;
        }
        self.vars.get(key).map(|s| s.as_str())
    }

    /// The whole map, unredacted.
    ///
    /// For the template renderer, which is reproducing what Bash does after
    /// exporting `.env` — it needs the real values, including the secrets that
    /// legitimately end up inside a generated service definition. Everything
    /// user-facing uses [`Self::redacted`] instead; this is deliberately not
    /// something a command returns.
    pub fn raw(&self) -> &BTreeMap<String, String> {
        &self.vars
    }

    /// Only lowercase `true` counts, matching the Bash `[ "$value" = "true" ]`
    /// comparisons. `TRUE` and `1` are falsy here because they are falsy there.
    pub fn bool(&self, key: &str) -> bool {
        self.get(key) == Some("true")
    }

    /// Comma-separated list, empty entries dropped.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `.env` key family for a service id: `mongo-express` → `SERVICE_MONGO_EXPRESS_`.
    ///
    /// Note the direction. Going the other way — deriving a compose profile by
    /// lowercasing the env key — is exactly the bug in CONFLICTS.md C-09, where
    /// `SERVICE_MONGO_EXPRESS_ENABLE` yields the profile `mongo_express` while
    /// the template declares `mongo-express`. Service ids come from the
    /// contract catalog, never from reversing this transform.
    pub fn service_prefix(service_id: &str) -> String {
        format!("SERVICE_{}_", service_id.to_uppercase().replace('-', "_"))
    }

    pub fn service_enabled(&self, service_id: &str) -> bool {
        self.bool(&format!("{}ENABLE", Self::service_prefix(service_id)))
    }

    pub fn service_version(&self, service_id: &str) -> Option<&str> {
        self.get(&format!("{}VERSION", Self::service_prefix(service_id)))
    }

    /// The image tags the settings sheet offers for a service, newest first.
    ///
    /// Empty is a legitimate answer and means "no list" rather than "no
    /// versions": the sheet falls back to a plain text field, which is what
    /// every service had before this existed. That matters for the `.env`
    /// override — somebody who writes `SERVICE_MONGO_VERSIONS=` has asked for
    /// the field back, not for an empty dropdown.
    ///
    /// The current value is folded in when the list does not already carry it,
    /// because a value absent from its own options is a control that opens
    /// showing nothing selected. That happens on any workspace pinning a tag
    /// off the list, which is exactly the case the free-text field exists for.
    pub fn service_versions(&self, service_id: &str) -> Vec<String> {
        let mut versions = self.list(&format!("{}VERSIONS", Self::service_prefix(service_id)));
        if let Some(current) = self.service_version(service_id) {
            if !current.is_empty() && !versions.is_empty() && !versions.iter().any(|v| v == current)
            {
                versions.insert(0, current.to_string());
            }
        }
        versions
    }

    pub fn service_url(&self, service_id: &str) -> Option<&str> {
        self.get(&format!("{}URL", Self::service_prefix(service_id)))
    }

    pub fn service_host_port(&self, service_id: &str) -> Option<u16> {
        self.get(&format!("{}HOST_PORT", Self::service_prefix(service_id)))
            .and_then(|v| v.parse().ok())
    }

    /// Every `SERVICE_<ID>_*` value a user might need to connect with, with the
    /// secrets already masked.
    ///
    /// `ENABLE`, `VERSION` and `URL` are dropped: they are the service's own
    /// wiring and are shown elsewhere in the row, so repeating them here would
    /// pad the list with the three entries nobody came for.
    ///
    /// Masked rather than raw for the reason `redacted()` exists — a password
    /// crossing into the webview by default puts it in every screenshot of this
    /// page. `env_reveal` hands over a single value when asked for it.
    pub fn service_credentials(&self, service_id: &str) -> Vec<(String, String, bool)> {
        let prefix = Self::service_prefix(service_id);

        self.vars
            .iter()
            .filter_map(|(key, value)| {
                let field = key.strip_prefix(&prefix)?;
                // VERSIONS joins the excluded three for the same reason VERSION
                // is there: it is not something you connect with. It is the
                // catalog behind the version control, and a comma-joined list
                // of image tags in a credentials block is noise.
                if matches!(field, "ENABLE" | "VERSION" | "VERSIONS" | "URL") || value.is_empty() {
                    return None;
                }

                let secret = Self::is_secret(key);
                let shown = if secret {
                    MASK.to_string()
                } else {
                    value.clone()
                };
                Some((field.to_string(), shown, secret))
            })
            .collect()
    }

    /// Keys whose values must never reach a log, an event or an error message.
    /// Mirrors `contracts/env.schema.json` → `secrets.policy`.
    ///
    /// `KEY` joined the list with Meilisearch and Typesense, and it is the
    /// suffix that shows why the list is a list rather than the word
    /// "password": `SERVICE_MEILISEARCH_MASTER_KEY` and
    /// `SERVICE_TYPESENSE_API_KEY` are credentials in every sense that matters
    /// — they open the whole index — and under the old five suffixes they
    /// would have been printed in full on the Services page, written into
    /// events, and refused by [`crate::secrets::is_movable`], which is the same
    /// list read from the other end.
    pub fn is_secret(key: &str) -> bool {
        ["PASSWORD", "PASS", "TOKEN", "SECRET", "SERVER_ID", "KEY"]
            .iter()
            .any(|suffix| key.ends_with(suffix))
    }

    /// The whole map with secret values replaced. This is what `env_get`
    /// returns — the raw values never cross the IPC boundary by default.
    pub fn redacted(&self) -> BTreeMap<String, String> {
        self.vars
            .iter()
            .map(|(k, v)| {
                let value = if Self::is_secret(k) && !v.is_empty() {
                    MASK.to_string()
                } else {
                    v.clone()
                };
                (k.clone(), value)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Embedded default < `.env` < policy, and the third layer is the one this
    /// checks. Reversed, an administrator's setting would be silently undone by
    /// whatever a workspace's `.env` happened to say — which is not a policy,
    /// it is a suggestion.
    #[test]
    fn a_policy_wins_over_both_the_file_and_the_embedded_default() {
        let mut env = Env::parse("DEFAULT_TLD_SUFFIX=mine.loc\n");
        assert_eq!(env.get("DEFAULT_TLD_SUFFIX"), Some("mine.loc"));
        // Untouched by the file, so still the embedded value.
        assert_eq!(env.get("SERVER_GZIP"), Some("off"));

        env.apply_policy(&crate::policy::Policy::parse(
            r#"{
                "schemaVersion": 1,
                "settings": { "DEFAULT_TLD_SUFFIX": "corp.test", "SERVER_GZIP": "on" }
            }"#,
            std::path::Path::new("/etc/stackvo/policy.json"),
        ));

        assert_eq!(env.get("DEFAULT_TLD_SUFFIX"), Some("corp.test"));
        assert_eq!(env.get("SERVER_GZIP"), Some("on"));
    }

    /// `parse` stays pure — a hundred tests build an `Env` from a string and
    /// have to get exactly the string they wrote, whatever machine they run on.
    #[test]
    fn an_unmanaged_policy_changes_nothing() {
        let mut env = Env::parse("DEFAULT_TLD_SUFFIX=mine.loc\n");
        let before = env.raw().clone();
        env.apply_policy(&crate::policy::Policy::none());
        assert_eq!(env.raw(), &before);
    }

    const SAMPLE: &str = r#"
# comment line
DEFAULT_PHP_VERSION=8.2

SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_ROOT_PASSWORD=hunter2
SERVICE_MONGO_EXPRESS_ENABLE=true
SUPPORTED_SERVERS=nginx,apache, caddy
LOOKS_LIKE_URL=postgres://user:pw@host:5432/db?a=1
SERVICE_REDIS_ENABLE=TRUE
"#;

    #[test]
    fn credentials_mask_secrets_and_drop_the_wiring() {
        let env = Env::parse(
            "SERVICE_MYSQL_ENABLE=true\n\
             SERVICE_MYSQL_VERSION=8.0\n\
             SERVICE_MYSQL_URL=db.stackvo.loc\n\
             SERVICE_MYSQL_ROOT_PASSWORD=hunter2\n\
             SERVICE_MYSQL_DATABASE=stackvo\n\
             SERVICE_MYSQL_EMPTY=\n\
             SERVICE_MONGO_DATABASE=other\n",
        );

        let creds = env.service_credentials("mysql");
        let keys: Vec<&str> = creds.iter().map(|(k, _, _)| k.as_str()).collect();

        // ENABLE/VERSION/URL are the service's wiring, shown elsewhere in the
        // row; an empty value is not a credential; another service's keys are
        // not this service's.
        assert_eq!(keys, vec!["DATABASE", "ROOT_PASSWORD"]);

        let password = creds.iter().find(|(k, _, _)| k == "ROOT_PASSWORD").unwrap();
        assert_eq!(password.1, "••••••••", "the raw secret must not cross");
        assert!(password.2, "and it must be flagged as one");

        let database = creds.iter().find(|(k, _, _)| k == "DATABASE").unwrap();
        assert_eq!(database.1, "stackvo", "a non-secret is shown as it is");
        assert!(!database.2);
    }

    #[test]
    fn splits_on_the_first_equals_only() {
        let env = Env::parse(SAMPLE);
        assert_eq!(
            env.get("LOOKS_LIKE_URL"),
            Some("postgres://user:pw@host:5432/db?a=1")
        );
    }

    /// The settings form binds to these by name and shows their defaults as
    /// the starting value. A rename here without one there produces a control
    /// wired to nothing: it renders, it accepts input, and it saves a key
    /// nothing reads. Nothing about that looks broken on screen, so the names
    /// are pinned on this side.
    /// The bug this pins down is one embedding created.
    ///
    /// Callers resolved `SUPPORTED_SERVERS_DEFAULT` then `DEFAULT_SERVER` in
    /// order, which was right while both could only come from the file. Once
    /// the first shipped as an embedded default it was always present, so the
    /// second was never consulted: a checkout that said Apache got nginx, with
    /// nothing to see in the file it had written.
    #[test]
    fn a_legacy_key_still_beats_the_embedded_default_it_was_renamed_to() {
        let env = Env::parse("DEFAULT_SERVER=apache\nDEFAULT_PHP_VERSION=7.4\n");
        assert_eq!(env.get("SUPPORTED_SERVERS_DEFAULT"), Some("apache"));
        assert_eq!(env.get("SUPPORTED_LANGUAGES_PHP_DEFAULT"), Some("7.4"));

        // The current name wins when the file carries both — the alias is a
        // fallback, not an override.
        let both = Env::parse("DEFAULT_SERVER=apache\nSUPPORTED_SERVERS_DEFAULT=caddy\n");
        assert_eq!(both.get("SUPPORTED_SERVERS_DEFAULT"), Some("caddy"));

        // And with neither spelled out, the embedded default still answers.
        assert_eq!(
            Env::parse("").get("SUPPORTED_SERVERS_DEFAULT"),
            Some("nginx")
        );
    }

    /// The divergence this collapsed, stated as the two cases that produced it.
    ///
    /// Before, `certs::suffix` trimmed, lower-cased and treated an empty value
    /// as absent, while ten call sites did a bare `unwrap_or("stackvo.loc")`.
    /// Measured on the two inputs that separate them:
    ///
    /// ```text
    ///   DEFAULT_TLD_SUFFIX=            call sites ""          certs::suffix "stackvo.loc"
    ///   DEFAULT_TLD_SUFFIX=Shop.LOC    call sites "Shop.LOC"  certs::suffix "shop.loc"
    /// ```
    ///
    /// The empty case is the one that reached a user: `Env::parse` lets a
    /// file's empty value win over the embedded default, so every project got a
    /// domain ending in a bare dot while the certificate was still `stackvo.loc`.
    #[test]
    fn a_blank_or_oddly_cased_suffix_resolves_the_way_the_certificate_does() {
        assert_eq!(
            Env::parse("DEFAULT_TLD_SUFFIX=").tld_suffix(),
            "stackvo.loc"
        );
        assert_eq!(
            Env::parse("DEFAULT_TLD_SUFFIX=   ").tld_suffix(),
            "stackvo.loc"
        );
        assert_eq!(
            Env::parse("DEFAULT_TLD_SUFFIX=Shop.LOC").tld_suffix(),
            "shop.loc"
        );
        assert_eq!(Env::parse("").tld_suffix(), DEFAULT_TLD_SUFFIX);
        assert_eq!(
            Env::parse("DEFAULT_TLD_SUFFIX=dev.test").tld_suffix(),
            "dev.test"
        );
    }

    /// A network name is **not** case-folded, and that difference is the whole
    /// reason these are two functions rather than one: Docker network names are
    /// case-sensitive, so lowering `MyNet` would name a network that is not
    /// there. Empty is still absent, for the reason above.
    #[test]
    fn a_network_name_is_trimmed_but_never_case_folded() {
        assert_eq!(
            Env::parse("DOCKER_DEFAULT_NETWORK=").docker_network(),
            "stackvo-net"
        );
        assert_eq!(
            Env::parse("DOCKER_DEFAULT_NETWORK=MyNet").docker_network(),
            "MyNet"
        );
        assert_eq!(Env::parse("").docker_network(), DEFAULT_NETWORK);
    }

    /// One setting, one answer. It used to be one setting and three: the wizard
    /// read `SUPPORTED_SERVERS_DEFAULT` and the five render sites did not, so
    /// `=caddy` gave a wizard project Caddy, an adopted project nginx, and a
    /// manifest with its `server` line deleted nginx.
    #[test]
    fn the_default_server_is_the_setting_and_not_a_literal() {
        assert_eq!(Env::parse("").default_server(), DEFAULT_SERVER);
        assert_eq!(
            Env::parse("SUPPORTED_SERVERS_DEFAULT=caddy").default_server(),
            "caddy"
        );
        assert_eq!(
            Env::parse("SUPPORTED_SERVERS_DEFAULT=  ").default_server(),
            DEFAULT_SERVER
        );
    }

    #[test]
    fn the_stack_shaping_settings_keep_their_names_and_defaults() {
        let embedded: BTreeMap<&str, &str> = EMBEDDED.iter().copied().collect();
        for (key, expected) in [
            ("DEFAULT_TLD_SUFFIX", "stackvo.loc"),
            ("SSL_ENABLE", "true"),
            ("REDIRECT_TO_HTTPS", "true"),
            ("DOCKER_DEFAULT_NETWORK", "stackvo-net"),
            ("PHP_DEFAULT_TOOLS", "composer,nodejs"),
        ] {
            assert_eq!(embedded.get(key), Some(&expected), "{key}");
        }

        // Editable as chips, so it has to survive the split/join round trip
        // the form does — no stray spaces, no empty entry from a trailing
        // comma.
        let apt = embedded
            .get("PHP_DEFAULT_APT_PACKAGES")
            .expect("apt package defaults ship");
        let items: Vec<&str> = apt.split(',').collect();
        assert!(items.len() > 10, "expected a real package list");
        assert!(
            items.iter().all(|p| !p.is_empty() && p.trim() == *p),
            "a chip would render blank or padded: {items:?}"
        );
    }

    #[test]
    fn only_lowercase_true_is_truthy() {
        let env = Env::parse(SAMPLE);
        assert!(env.bool("SERVICE_MYSQL_ENABLE"));
        // Matches Bash's `[ "$value" = "true" ]` — uppercase is NOT true there.
        assert!(!env.bool("SERVICE_REDIS_ENABLE"));
    }

    #[test]
    fn list_trims_entries() {
        let env = Env::parse(SAMPLE);
        assert_eq!(
            env.list("SUPPORTED_SERVERS"),
            vec!["nginx", "apache", "caddy"]
        );
    }

    #[test]
    fn service_prefix_maps_dash_to_underscore() {
        assert_eq!(
            Env::service_prefix("mongo-express"),
            "SERVICE_MONGO_EXPRESS_"
        );
        let env = Env::parse(SAMPLE);
        assert!(env.service_enabled("mongo-express"));
    }

    #[test]
    fn secrets_are_redacted_but_keys_survive() {
        let env = Env::parse(SAMPLE);
        let out = env.redacted();
        assert_eq!(out["SERVICE_MYSQL_ROOT_PASSWORD"], "••••••••");
        assert_eq!(out["DEFAULT_PHP_VERSION"], "8.2");
    }

    /// An API key is a credential, and the suffix list is how this codebase
    /// says so.
    ///
    /// Search engines authenticate with a key rather than a password, so
    /// Meilisearch and Typesense arrived holding a secret that four separate
    /// mechanisms — the Services page's mask, `redacted()`, the log scrubber
    /// and [`crate::secrets::is_movable`] — would all have waved through. They
    /// share one list, which is the point: this asserts on the list, not on
    /// four behaviours that happen to agree today.
    #[test]
    fn a_key_is_as_much_a_credential_as_a_password() {
        for key in [
            "SERVICE_MEILISEARCH_MASTER_KEY",
            "SERVICE_TYPESENSE_API_KEY",
            "SERVICE_MYSQL_ROOT_PASSWORD",
            "SERVICE_BLACKFIRE_SERVER_TOKEN",
        ] {
            assert!(Env::is_secret(key), "{key} must be treated as a secret");
            assert!(crate::secrets::is_movable(key), "{key} must be movable");
        }

        // And the suffix is a suffix, not a substring: the words appear inside
        // ordinary key names, and matching those would mask a version number.
        for key in [
            "SERVICE_MINIO_ROOT_USER",
            "SERVICE_KAFBAT_CLUSTER_NAME",
            "SERVICE_MEILISEARCH_VERSION",
            "SERVICE_PGADMIN_MASTER_PASSWORD_REQUIRED",
        ] {
            assert!(!Env::is_secret(key), "{key} is not a secret");
        }
    }

    /// The split is a partition, and both numbers are measured rather than
    /// estimated.
    ///
    /// The service half was called "roughly half of 186" for three rounds. It is
    /// 150 of 186 — four fifths — and the difference matters, because the size
    /// of that constant is the size of the deletion this is waiting to make. A
    /// number nobody counts drifts toward whatever the last person guessed.
    ///
    /// The membership rule is the prefix and nothing else. A cleverer rule —
    /// "keys the handover reads", "keys with a `_VERSION` beside them" — would
    /// have to be maintained beside the constant it describes, and the whole
    /// point of the split is that the day it is deleted, nobody has to decide
    /// anything.
    #[test]
    fn the_two_halves_partition_the_defaults() {
        assert_eq!(SETTINGS.len(), 36);
        assert_eq!(LEGACY_SERVICES.len(), 78);
        assert_eq!(EMBEDDED.len(), SETTINGS.len() + LEGACY_SERVICES.len());

        for (key, _) in LEGACY_SERVICES {
            assert!(
                key.starts_with("SERVICE_"),
                "{key} is in the legacy half but is not a service key"
            );
        }
        for (key, _) in SETTINGS {
            assert!(
                !key.starts_with("SERVICE_"),
                "{key} is a service key sitting in the half that stays — it would \
                 survive the deletion `LEGACY_SERVICES` is waiting for"
            );
        }

        // A key in both halves would be a value whose meaning depends on which
        // one `both_halves` laid down last, and the merge is a `BTreeMap` so
        // the loser would vanish without a word.
        let mut seen = std::collections::BTreeSet::new();
        for (key, _) in EMBEDDED {
            assert!(seen.insert(key), "{key} is defined twice");
        }
        assert_eq!(seen.len(), EMBEDDED.len());
    }

    /// A default is an answer; it is not a choice, and one caller needs to tell.
    ///
    /// `get` merges [`EMBEDDED`] under the file and that is right almost
    /// everywhere. `stated` is the narrow question `db.rs` asks — its
    /// resolution order puts `.env` ahead of what a package declares, so a
    /// default arriving in that slot outranks the package, which is backwards.
    /// This is the line between the two, and the alias case is
    /// included because "the file chose it" has to survive the file choosing it
    /// under the older spelling.
    #[test]
    fn stated_is_what_the_file_wrote_and_get_is_that_plus_the_defaults() {
        let env = Env::parse("SERVICE_MYSQL_ROOT_PASSWORD=hunter2\n");
        assert_eq!(env.stated("SERVICE_MYSQL_ROOT_PASSWORD"), Some("hunter2"));
        assert_eq!(env.get("SERVICE_MYSQL_ROOT_PASSWORD"), Some("hunter2"));

        // The embedded default answers `get` and refuses `stated`: nobody chose
        // it, and a dump that used it would be using the binary's opinion over
        // the package's.
        let untouched = Env::parse("");
        assert_eq!(untouched.get("SERVICE_MYSQL_ROOT_PASSWORD"), Some("root"));
        assert_eq!(untouched.stated("SERVICE_MYSQL_ROOT_PASSWORD"), None);

        // A key nobody has ever heard of is absent from both.
        assert_eq!(untouched.stated("SERVICE_NOTHING_AT_ALL"), None);

        // An alias: chosen under the old name, so `stated` answers under the
        // new one. Anything else would make the chain a choice the file made
        // and this function cannot see.
        let aliased = Env::parse("DEFAULT_SERVER=apache\n");
        assert_eq!(aliased.stated("SUPPORTED_SERVERS_DEFAULT"), Some("apache"));

        // And an empty value is still a choice — blanking a line is how
        // somebody asks for the field back (see `service_versions`). Filtering
        // empties is the caller's business, not this function's.
        let blanked = Env::parse("SERVICE_MYSQL_ROOT_PASSWORD=\n");
        assert_eq!(blanked.stated("SERVICE_MYSQL_ROOT_PASSWORD"), Some(""));
    }

    /// `both_halves` concatenates rather than interleaving.
    ///
    /// A `const fn` with two hand-written index loops is exactly the shape that
    /// can be off by one and still compile — the array is the right length
    /// either way, and the hole shows up as an empty key that `Env::parse`
    /// happily inserts under `""`.
    ///
    /// Elementwise rather than by the four corners, and the rehearsal is why.
    /// This used to assert `EMBEDDED[SETTINGS.len()] == LEGACY_SERVICES[0]`,
    /// which reads fine and is a **compile error** the moment the legacy half
    /// is empty: `deny(unconditional_panic)` rejects the constant index, so
    /// deleting the constant that is waiting to go stopped the crate's
    /// tests from building instead of failing one. Comparing the whole array
    /// against the two halves laid end to end checks every entry rather than
    /// four, and survives one half going away — which is the state this test is
    /// eventually meant to be read in.
    #[test]
    fn the_merge_keeps_every_entry_and_invents_none() {
        let mut expected = Vec::with_capacity(EMBEDDED.len());
        expected.extend_from_slice(&SETTINGS);
        expected.extend_from_slice(&LEGACY_SERVICES);
        assert_eq!(EMBEDDED.as_slice(), expected.as_slice());

        for (key, _) in EMBEDDED {
            assert!(!key.is_empty(), "the merge left a hole");
        }
    }

    /// Every service in the catalog can be switched on from a fresh install.
    ///
    /// `service_enable` writes `SERVICE_<NAME>_ENABLE=true`, and the Services
    /// page decides what to render from the catalog — so a service the
    /// contract declares and `EMBEDDED` says nothing about is a row that reads
    /// as "off" because the key is missing rather than because anybody chose.
    /// The version key is the same story one step later: an image tag of the
    /// empty string is `image: "minio/minio:"`, which compose rejects.
    /// Every service that ever lived in `.env` still has its defaults there.
    ///
    /// Scoped to those, and the scope is the point. [`LEGACY_SERVICES`] exists
    /// so a workspace created by an older StackVo keeps rendering and so the
    /// handover has something to read; it is not a catalogue any more. A
    /// service that arrived as a package — Solr, ClickHouse — never had an
    /// `.env` key and must not gain one, because gaining one would mean the
    /// app had an opinion about a service it does not ship.
    ///
    /// The remaining work is the other direction: that constant goes once no
    /// supported workspace still needs migrating. `tests/legacy_env_claims.rs`
    /// carries the list of what has to change on the day, and the version it is
    /// due by.
    #[test]
    fn every_catalog_service_ships_an_enable_and_a_version() {
        let embedded: std::collections::BTreeSet<&str> =
            EMBEDDED.iter().map(|(key, _)| *key).collect();

        for (service, _) in crate::contracts::env_schema().service_catalog() {
            // Package-native: no template, no `.env` key, nothing embedded.
            if !embedded.contains(format!("{}ENABLE", Env::service_prefix(&service)).as_str()) {
                continue;
            }
            let prefix = Env::service_prefix(&service);
            for suffix in ["ENABLE", "VERSION", "VERSIONS"] {
                let key = format!("{prefix}{suffix}");
                assert!(
                    embedded.contains(key.as_str()),
                    "{service} has no {key} default"
                );
            }
        }
    }

    /// The version a service ships on is one the version picker offers.
    ///
    /// Without this the two halves drift the moment either is edited alone: a
    /// bumped default that nobody added to the list opens a combobox on a value
    /// its own menu does not contain, and a trimmed list quietly orphans the
    /// value every fresh workspace starts with. `service_versions` papers over
    /// exactly that at runtime by folding the current value in — which is right
    /// for a user's own pinned tag and wrong as a way for the shipped pair to
    /// disagree, so the invariant is asserted on `EMBEDDED` rather than on what
    /// the reader returns.
    #[test]
    fn every_shipped_version_is_offered_by_its_own_catalog() {
        let embedded: BTreeMap<&str, &str> = EMBEDDED.iter().copied().collect();

        for (service, _) in crate::contracts::env_schema().service_catalog() {
            let prefix = Env::service_prefix(&service);
            // See the test above: a package-native service has no `.env` half.
            let Some(version) = embedded.get(format!("{prefix}VERSION").as_str()).copied() else {
                continue;
            };
            let versions = embedded[format!("{prefix}VERSIONS").as_str()];

            let offered: Vec<&str> = versions.split(',').map(str::trim).collect();
            assert!(
                offered.contains(&version),
                "{service} ships {version} but offers {versions}"
            );
            assert!(
                offered.iter().all(|v| !v.is_empty()),
                "{service} has a blank entry in {versions} — an empty image tag"
            );
        }
    }

    /// A pinned tag that is not on the list still shows as the current value.
    ///
    /// The list is stated rather than inherited. It used to come from
    /// `SERVICE_MONGO_VERSIONS` as an embedded default, so this test was reading
    /// the catalogue that moved into packages — and when those keys
    /// left it failed for a reason that had nothing to do with folding.
    /// A test about what `service_versions` does with a list should say what the
    /// list is.
    #[test]
    fn an_unlisted_version_is_folded_into_the_options() {
        const OFFERED: &str = "SERVICE_MONGO_VERSIONS=8.0,7.0,6.0\n";

        let env = Env::parse(&format!("{OFFERED}SERVICE_MONGO_VERSION=8.0.28\n"));
        let versions = env.service_versions("mongo");

        assert_eq!(versions.first().map(String::as_str), Some("8.0.28"));
        assert!(versions.contains(&"8.0".to_string()), "{versions:?}");
        // Folded in once, not appended to a list that already had it.
        let listed =
            Env::parse(&format!("{OFFERED}SERVICE_MONGO_VERSION=7.0\n")).service_versions("mongo");
        assert_eq!(listed.iter().filter(|v| *v == "7.0").count(), 1);
        assert_eq!(listed.first().map(String::as_str), Some("8.0"));
    }

    /// Emptying the list is how a workspace asks for the plain text field back,
    /// so it must not be answered with a one-entry list built from the current
    /// value — that would make the setting impossible to turn off.
    #[test]
    fn an_emptied_catalog_offers_nothing_rather_than_the_current_value() {
        let env = Env::parse("SERVICE_MONGO_VERSION=8.0\nSERVICE_MONGO_VERSIONS=\n");
        assert!(env.service_versions("mongo").is_empty());
    }

    /// The catalog is not a credential, and must not surface as one.
    #[test]
    fn the_version_catalog_stays_out_of_the_credentials_block() {
        let env = Env::parse("SERVICE_MONGO_ENABLE=true\n");
        let fields: Vec<String> = env
            .service_credentials("mongo")
            .into_iter()
            .map(|(key, _, _)| key)
            .collect();

        assert!(!fields.iter().any(|f| f == "VERSIONS"), "{fields:?}");
        assert!(!fields.iter().any(|f| f == "VERSION"), "{fields:?}");
    }
}
