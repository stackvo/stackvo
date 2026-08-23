//! The commands this app puts on your `PATH`, and the host tools it needs.
//!
//! Every rival ships a Tooling page and they all ship the same one: a list of
//! `composer`, `node`, `bun`, `wp-cli`, each with an Install button that fetches
//! a binary into the product's data directory and a shim onto `PATH`. Yerd's is
//! the clearest of them. **That page is deliberately not what this module is.**
//!
//! ## Why the tools themselves are not downloaded
//!
//! A project here declares a PHP version, a Node version and a set of
//! extensions, and runs in an image built from them. `composer` is *in* that
//! image; `stackvo composer install` runs the one the project declared, and
//! [`crate::quickcmd`] offers the same set as buttons. Downloading a second
//! `composer` onto the host would produce two answers to "which one runs", and
//! the host's copy would be the one that is wrong — it knows nothing about the
//! project's PHP. `cli.rs`'s A-3 note says the same thing about `php`: the
//! version is a property of a project, not of a directory a shim guesses at.
//!
//! So the catalogue below holds **host** tools only: the four programs this app
//! itself shells out to. They are not interchangeable with a container's copy,
//! because they run *outside* every container — that is what makes them the
//! host's business and not the image's.
//!
//! ## What was actually missing
//!
//! The half of that page with no answer here at all was the `PATH` half.
//! `stackvo` and `stackvo-mcp` are real programs this repository builds, the
//! README documents them, [`crate::agents`] registers `stackvo-mcp` with six
//! assistants — and nothing anywhere put either of them where a shell would
//! find it. The instruction was "build it and remember the path".
//!
//! ## The three pieces
//!
//! 1. **A directory this app owns** — [`bin_dir`], under the OS's data
//!    directory rather than under `~/.stackvo`. `appdir` gives the reasoning:
//!    `~/.stackvo` is the *stack's* state, the user may point it elsewhere, and
//!    deleting it is a supported way to start over. A `PATH` entry that
//!    disappears when somebody resets their stack is a `PATH` entry pointing at
//!    nothing.
//! 2. **Links into it** — [`link`]. Symlinks on unix, copies on Windows, where
//!    a symlink needs a privilege this app does not ask for.
//! 3. **One line in one shell's startup file** — [`path_apply`], written the
//!    way [`crate::rules`] writes: between markers, after a backup, leaving
//!    every other byte alone. It is the user's `.zshrc`.
//!
//! ## Why the digest is compiled in
//!
//! [`install`] fetches `mkcert` — the one tool in the catalogue that is a
//! single static binary its author publishes, and the one whose absence
//! degrades this app (no trusted HTTPS). It is checked against a SHA-256 that
//! is **in this source file**, not against a checksum file fetched from the
//! same host as the binary. A digest served beside the thing it describes is
//! not a check: whoever can replace one can replace the other. Pinning it here
//! means the bytes that install are the bytes that were reviewed, and a release
//! that wants a newer mkcert edits this table and is reviewed again.
//!
//! That is also why there is no "update" verb. An idempotent install that
//! silently follows upstream is how a pinned digest stops being a pin.

use crate::error::{Code, Error, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The markers around the line we write. `#` is a comment in every shell here,
/// PowerShell included, so one pair covers the table.
pub const BEGIN: &str = "# stackvo:path:begin";
pub const END: &str = "# stackvo:path:end";

/// The copy taken before a startup file is rewritten — the same suffix
/// `rules.rs` and `ide.rs` use, for the same reason.
pub const BACKUP_SUFFIX: &str = ".stackvo-backup";

/// A startup file is prose somebody wrote. Anything larger than this is not one
/// and is not something to read into memory and write back.
const MOST_RC_BYTES: u64 = 1024 * 1024;

// ------------------------------------------------------------- the directory

/// Where the commands this app puts on `PATH` live.
///
/// One directory rather than dropping links into `/usr/local/bin`: that one is
/// shared, needs a privilege on a fresh macOS, and uninstalling would mean
/// deciding which of its entries were ours.
pub fn bin_dir() -> Option<PathBuf> {
    crate::appdir::bin()
}

/// Is [`bin_dir`] on the `PATH` this process was started with?
///
/// The honest reading of "did it take effect". A block written into `.zshrc`
/// changes the *next* shell, and a user who has just pressed the button and is
/// looking at a terminal opened an hour ago needs to be told that rather than
/// left to conclude the button did nothing.
pub fn on_path() -> bool {
    let Some(dir) = bin_dir() else {
        return false;
    };
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|entry| entry == dir)
}

// ------------------------------------------------------------------- shells

/// A shell, its startup file, and how it spells "prepend to `PATH`".
pub struct Shell {
    pub id: &'static str,
    pub label: &'static str,
    /// The startup file, relative to the home directory.
    ///
    /// One file per shell, not two. bash reads `.bashrc` for interactive
    /// non-login shells and `.bash_profile` for login shells, and on macOS
    /// Terminal.app opens login shells while every Linux terminal opens the
    /// other — so the file that gets read differs by platform, and writing both
    /// would put the entry on `PATH` twice for anyone who sources one from the
    /// other, which is the usual arrangement.
    rc: &'static str,
    /// macOS reads a different file for bash, and only for bash.
    rc_macos: Option<&'static str>,
}

/// The four shells a person on macOS, Linux or Windows is actually in.
///
/// No `csh`, no `nu`, no `elvish`. Each row is a file this app writes into
/// somebody's home directory, and a row nobody uses is a file nobody notices
/// has been edited.
pub const SHELLS: &[Shell] = &[
    Shell {
        id: "zsh",
        label: "zsh",
        rc: ".zshrc",
        rc_macos: None,
    },
    Shell {
        id: "bash",
        label: "bash",
        rc: ".bashrc",
        rc_macos: Some(".bash_profile"),
    },
    Shell {
        id: "fish",
        label: "fish",
        rc: ".config/fish/config.fish",
        rc_macos: None,
    },
    Shell {
        id: "powershell",
        label: "PowerShell",
        rc: "Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
        rc_macos: None,
    },
];

/// The shell with this id.
pub fn shell(id: &str) -> Option<&'static Shell> {
    SHELLS.iter().find(|s| s.id == id)
}

/// The startup file for one shell, under one home directory.
///
/// `home` and `macos` are arguments rather than reads so the whole table can be
/// tested on one machine — the bash row differs by platform and a platform is
/// exactly the thing a test cannot change.
pub fn rc_path(id: &str, home: &Path, macos: bool) -> Option<PathBuf> {
    let shell = shell(id)?;
    let relative = match (macos, shell.rc_macos) {
        (true, Some(mac)) => mac,
        _ => shell.rc,
    };
    Some(home.join(relative))
}

/// Which shell the caller is in, from `SHELL` — or PowerShell on Windows,
/// where that variable does not exist.
///
/// Only a default for the interface to preselect. Getting it wrong costs a
/// click; guessing silently and writing to the wrong file would cost a file.
pub fn current_shell() -> Option<&'static str> {
    if let Some(value) = std::env::var_os("SHELL") {
        let value = value.to_string_lossy().to_ascii_lowercase();
        // Matched on the file name, because `/opt/homebrew/bin/fish` and
        // `/usr/local/bin/fish` are the same shell and neither is `/bin/fish`.
        let name = value.rsplit('/').next().unwrap_or_default().to_string();
        return SHELLS.iter().map(|s| s.id).find(|id| name.contains(*id));
    }
    if cfg!(windows) {
        return Some("powershell");
    }
    None
}

// ------------------------------------------------------------------ quoting

/// A path's *contents*, escaped for the inside of a POSIX double-quoted string.
///
/// The quotes are the caller's, because the caller puts `$PATH` inside them too
/// and that one must survive unescaped.
///
/// The default install directory on macOS is `~/Library/Application
/// Support/StackVo/bin` — it has a space in it before anybody does anything
/// unusual, so quoting is not an edge case here, it is the normal case. The
/// four characters below are the ones a double-quoted string still interprets.
fn quote_posix(path: &Path) -> String {
    let mut out = String::new();
    for c in path.to_string_lossy().chars() {
        if matches!(c, '"' | '\\' | '$' | '`') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The same, for fish — which has no backtick substitution and treats a
/// backslash inside double quotes as an escape only before `"`, `\` and `$`.
fn quote_fish(path: &Path) -> String {
    let mut out = String::from("\"");
    for c in path.to_string_lossy().chars() {
        if matches!(c, '"' | '\\' | '$') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// PowerShell single quotes: no interpolation at all, and the only escape is a
/// doubled quote.
fn quote_pwsh(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "''"))
}

/// The one line that puts [`bin_dir`] first on `PATH`, as that shell spells it.
///
/// **First**, not last, and that is a decision rather than a habit: the only
/// names in this directory are `stackvo`, `stackvo-mcp` and tools this app was
/// asked to manage. Appending would mean a managed `mkcert` losing to a
/// half-removed Homebrew one, which is the failure somebody presses this button
/// to get out of.
pub fn line(id: &str, bin: &Path) -> Option<String> {
    Some(match id {
        // `$PATH` inside the quotes, not after them. `export PATH="/a b":$PATH`
        // is valid and wrong: the unquoted expansion is word-split, so a `PATH`
        // that already contains a space reaches `export` as several arguments.
        // fish below needs no such care — it does not word-split a variable.
        "zsh" | "bash" => format!("export PATH=\"{}:$PATH\"", quote_posix(bin)),
        "fish" => format!("set -gx PATH {} $PATH", quote_fish(bin)),
        "powershell" => format!(
            "$env:Path = {} + [IO.Path]::PathSeparator + $env:Path",
            quote_pwsh(bin)
        ),
        _ => return None,
    })
}

/// The whole marked region, as it is written: the `PATH` entry, then the
/// tab-completion stub.
///
/// **One region, not two.** The alternative was a separate completion file per
/// shell — `~/.zsh/completions/_stackvo` and its three siblings — and it is the
/// wrong trade here for the reason [`SHELLS`] already gives about startup
/// files: every extra path is another file in somebody's home directory that
/// this app edits, another place to look when it goes wrong, and another thing
/// `path-remove` has to find. The stub is four lines; it fits where the `PATH`
/// line already lives, and one `path-remove` takes both back out.
///
/// The completion is **appended after** the `PATH` line, because it names the
/// binary and the binary has to be findable when the line runs. A completion
/// registered for a command that is not on `PATH` is not an error in any of
/// these shells — it simply never fires — so the order is the difference
/// between working and quietly not.
pub fn block(id: &str, bin: &Path) -> Option<String> {
    let mut body = line(id, bin)?;
    if let Some(stub) = crate::completions::stub(id, "stackvo") {
        body.push('\n');
        body.push_str(stub.trim_end());
    }
    Some(format!("{BEGIN}\n{body}\n{END}\n"))
}

// ------------------------------------------------------- the marked region
//
// Pure text in, pure text out — the same shape `rules::merge` and
// `rules::strip` have, and for the same reason: these are the functions that
// decide what happens to somebody's `.zshrc`, so they are the ones that must be
// testable without one.

/// The byte range of our region, markers included.
fn region(text: &str) -> Option<(usize, usize)> {
    let start = text.find(BEGIN)?;
    let end = text[start..].find(END)? + start + END.len();
    Some((start, end))
}

/// Put the block into `text`, replacing an existing one.
///
/// Appended when there is no region yet, never prepended: a startup file is
/// read top to bottom and the lines above ours are the ones the user put there
/// deliberately. A `PATH` entry that jumps ahead of them changes what their own
/// file does.
pub fn merge(text: &str, block: &str) -> String {
    if let Some((start, end)) = region(text) {
        let mut out = String::with_capacity(text.len() + block.len());
        out.push_str(&text[..start]);
        out.push_str(block.trim_end());
        out.push_str(&text[end..]);
        return out;
    }

    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(block);
    out
}

/// Take the region back out. Everything else stays byte for byte.
pub fn strip(text: &str) -> String {
    let Some((start, end)) = region(text) else {
        return text.to_string();
    };
    let mut out = String::with_capacity(text.len());
    out.push_str(text[..start].trim_end_matches([' ', '\t', '\n']));
    let rest = text[end..].trim_start_matches('\n');
    if !out.is_empty() && !rest.is_empty() {
        out.push_str("\n\n");
    } else if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(rest);
    out
}

/// `(installed, current)` — whether the file carries our region at all, and
/// whether that region is the line this build would write.
///
/// The second half is what distinguishes "already done" from "done by a build
/// whose data directory was somewhere else", which is a real state: a user who
/// moves between a checkout and an installed app has both.
pub fn state(text: &str, block: &str) -> (bool, bool) {
    let Some((start, end)) = region(text) else {
        return (false, false);
    };
    (true, text[start..end].trim() == block.trim())
}

// ------------------------------------------------------------- the binaries

/// The programs this app puts on `PATH`, and what each is for.
pub const OWN: [(&str, &str); 2] = [
    ("stackvo", "The stack from a terminal."),
    ("stackvo-mcp", "The MCP server assistants talk to."),
];

/// The file name a program has on this platform.
fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Where a binary this repository builds can be found, if anywhere.
///
/// The same search [`crate::agents::binary`] does and in the same order —
/// beside this executable, then the other build profile in this checkout — and
/// deliberately **not** `PATH`. `PATH` is where this module is trying to *put*
/// it: finding the link we made and linking it to itself is a loop, and on a
/// second run it would report the copy in `bin_dir` as the source.
pub fn shipped(name: &str) -> Option<PathBuf> {
    let name = exe_name(name);
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;

    let sibling = dir.join(&name);
    if sibling.is_file() {
        return Some(sibling);
    }

    for profile in ["release", "debug"] {
        if let Some(target) = dir.parent() {
            let other = target.join(profile).join(&name);
            if other.is_file() {
                return Some(other);
            }
        }
    }
    None
}

/// The managed copy of a program, when this app installed one.
///
/// The lookup every caller that shells out to a host tool should use: it
/// answers with our copy when there is one and with the bare name otherwise, so
/// a managed `mkcert` is found by the app itself and not only by the user's
/// shell. `certs::helper` is the caller that matters — the app has no reason to
/// wait for a terminal restart to see a tool it just installed.
pub fn resolve(program: &str) -> PathBuf {
    resolve_in(bin_dir().as_deref(), program)
}

/// [`resolve`], against a directory given rather than found.
///
/// Split out because the test for it could not otherwise be written. It read
/// the **real** application-support directory and asserted the fallback branch
/// with the comment "nothing is installed in a test run" — which stops being
/// true the first time the person developing this app installs a tool with it,
/// and then a full suite fails on one machine and passes on every other. That
/// is the same flaw as a test waiting on the real keychain (§3 #37), one turn
/// quieter: it does not hang, it just accuses the wrong change.
fn resolve_in(dir: Option<&Path>, program: &str) -> PathBuf {
    let name = exe_name(program);
    match dir.map(|dir| dir.join(&name)) {
        Some(path) if path.is_file() => path,
        _ => PathBuf::from(program),
    }
}

// ------------------------------------------------------------ the catalogue

/// A host tool: a program that runs outside every container, which is what
/// makes it this app's business rather than an image's.
pub struct Tool {
    pub id: &'static str,
    pub label: &'static str,
    /// The program name, as `PATH` spells it.
    pub program: &'static str,
    /// The arguments that make it print its version.
    pub version_args: &'static [&'static str],
    /// What breaks without it. Shown on the row, because "install this" without
    /// "or else" is a chore rather than a decision.
    pub why: &'static str,
    /// The pinned download, for the one tool that has one.
    pub download: Option<&'static Download>,
}

/// Where a managed tool comes from, and what its bytes must hash to.
pub struct Download {
    /// The upstream release this build pins.
    pub version: &'static str,
    /// `https://…/{asset}` — the asset name carries the platform.
    pub url_prefix: &'static str,
    /// `(os, arch, asset, sha256)`, with `os`/`arch` spelled as
    /// `std::env::consts` spells them.
    pub assets: &'static [(&'static str, &'static str, &'static str, &'static str)],
    /// The publisher, named on screen. Somebody about to let this app fetch an
    /// executable is entitled to know whose.
    pub publisher: &'static str,
}

/// mkcert v1.4.4, from the author's own GitHub release.
///
/// The digests were taken from those assets and are pinned here rather than
/// fetched; see the module comment for why that is not the same as trusting
/// GitHub twice. `linux-arm` (32-bit) is deliberately absent: nothing in
/// `release.yml` builds this app for it, so it is a platform that cannot get
/// here.
pub static MKCERT: Download = Download {
    version: "1.4.4",
    url_prefix: "https://github.com/FiloSottile/mkcert/releases/download/v1.4.4/",
    publisher: "Filippo Valsorda (FiloSottile/mkcert)",
    assets: &[
        (
            "macos",
            "x86_64",
            "mkcert-v1.4.4-darwin-amd64",
            "a32dfab51f1845d51e810db8e47dcf0e6b51ae3422426514bf5a2b8302e97d4e",
        ),
        (
            "macos",
            "aarch64",
            "mkcert-v1.4.4-darwin-arm64",
            "c8af0df44bce04359794dad8ea28d750437411d632748049d08644ffb66a60c6",
        ),
        (
            "linux",
            "x86_64",
            "mkcert-v1.4.4-linux-amd64",
            "6d31c65b03972c6dc4a14ab429f2928300518b26503f58723e532d1b0a3bbb52",
        ),
        (
            "linux",
            "aarch64",
            "mkcert-v1.4.4-linux-arm64",
            "b98f2cc69fd9147fe4d405d859c57504571adec0d3611c3eefd04107c7ac00d0",
        ),
        (
            "windows",
            "x86_64",
            "mkcert-v1.4.4-windows-amd64.exe",
            "d2660b50a9ed59eada480750561c96abc2ed4c9a38c6a24d93e30e0977631398",
        ),
        (
            "windows",
            "aarch64",
            "mkcert-v1.4.4-windows-arm64.exe",
            "793747256c562622d40127c8080df26add2fb44c50906ce9db63b42a5280582e",
        ),
    ],
};

/// The host tools, and only the host tools.
///
/// Four rows, one per program this app runs on the host. `composer`, `node`,
/// `npm` and `wp` are not here and will not be: they run in the project's
/// container, at the version the project declared.
pub const TOOLS: &[Tool] = &[
    Tool {
        id: "docker",
        label: "Docker",
        program: "docker",
        version_args: &["--version"],
        why: "Every project runs in it. Nothing else here works without it.",
        // Docker Desktop, OrbStack and Colima are applications with installers,
        // virtual machines and a menu bar item. A binary dropped on PATH would
        // be a client with no engine behind it — worse than the honest absence,
        // because `docker ps` would then fail instead of `docker` being missing.
        download: None,
    },
    Tool {
        id: "compose",
        label: "Docker Compose",
        program: "docker",
        version_args: &["compose", "version", "--short"],
        why: "The generated stack is compose files; this is what runs them.",
        // A plugin of the engine above, installed with it.
        download: None,
    },
    Tool {
        id: "git",
        label: "Git",
        program: "git",
        version_args: &["--version"],
        why: "Worktrees, branch names on the project pages, and cloning a repository into a new project.",
        // Every platform ships it or has a one-command install, and on macOS
        // asking for it triggers the Command Line Tools installer — a system
        // dialogue this app should not be racing.
        download: None,
    },
    Tool {
        id: "mkcert",
        label: "mkcert",
        program: "mkcert",
        version_args: &["-version"],
        why: "Trusted HTTPS for .loc domains. Without it the stack still runs and every browser warns.",
        download: Some(&MKCERT),
    },
];

/// The tool with this id.
pub fn tool(id: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|t| t.id == id)
}

/// The asset for one platform, or `None` where the publisher has no build.
pub fn asset(
    download: &'static Download,
    os: &str,
    arch: &str,
) -> Option<(&'static str, &'static str)> {
    download
        .assets
        .iter()
        .find(|(o, a, _, _)| *o == os && *a == arch)
        .map(|(_, _, asset, sha)| (*asset, *sha))
}

// -------------------------------------------------------------------- status

/// Where a program was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    /// In [`bin_dir`] — this app put it there.
    Managed,
    /// Somewhere else on `PATH`. The user's, and left alone.
    System,
    /// Nowhere.
    Missing,
}

/// One host tool, as the pane shows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub id: &'static str,
    pub label: &'static str,
    pub program: &'static str,
    pub why: &'static str,
    pub source: Source,
    /// What it printed when asked for its version, when it could be run.
    pub version: Option<String>,
    /// The file that answered, when one did.
    pub path: Option<String>,
    /// The version this build would install, for the one tool that has one.
    pub offers: Option<String>,
    /// Who publishes that download.
    pub publisher: Option<&'static str>,
    /// False when the publisher has no build for this platform — which is a
    /// different sentence from "you have not installed it".
    pub available_here: bool,
}

/// One of this repository's own binaries.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnStatus {
    pub id: &'static str,
    pub about: &'static str,
    /// Where the build of it is, if this app can find one to link.
    pub built: Option<String>,
    /// Where the link in [`bin_dir`] points, if it exists.
    pub linked: Option<String>,
}

/// One shell's startup file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellStatus {
    pub id: &'static str,
    pub label: &'static str,
    pub path: Option<String>,
    /// Does the file exist at all?
    pub exists: bool,
    /// Does it carry our region?
    pub installed: bool,
    /// Is that region the line this build would write?
    pub current: bool,
    /// The line itself, so a reader can put it somewhere else by hand.
    pub line: Option<String>,
}

/// Everything the Tooling pane reads in one call.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub bin_dir: Option<String>,
    pub on_path: bool,
    pub current_shell: Option<&'static str>,
    pub own: Vec<OwnStatus>,
    pub shells: Vec<ShellStatus>,
    pub tools: Vec<ToolStatus>,
}

/// First line of stdout, or `None` when the program cannot be run at all.
///
/// mkcert prints its version to stdout and Docker prints its to stdout; a
/// program that writes to stderr instead would be reported as present with no
/// version, which is the honest answer and not a failure.
async fn probe(program: &Path, args: &[&str]) -> Option<String> {
    let output = tokio::process::Command::new(program)
        .args(args)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
}

/// The first entry on `PATH` holding a file with this name, skipping our own
/// directory.
///
/// Skipping it is what makes [`Source::Managed`] and [`Source::System`] two
/// different answers rather than one answer twice.
fn on_system_path(name: &str) -> Option<PathBuf> {
    let ours = bin_dir();
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .filter(|dir| Some(dir.as_path()) != ours.as_deref())
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Where one tool is, and which copy it is.
fn locate(program: &str) -> (Source, Option<PathBuf>) {
    let name = exe_name(program);
    if let Some(dir) = bin_dir() {
        let managed = dir.join(&name);
        if managed.is_file() {
            return (Source::Managed, Some(managed));
        }
    }
    match on_system_path(&name) {
        Some(path) => (Source::System, Some(path)),
        // Not `Missing` yet on Windows, where `PATH` alone under-reports: the
        // shell also consults the registry's App Paths. Running the bare name
        // is the fallback, and a version coming back is proof enough.
        None => (Source::Missing, None),
    }
}

/// Read the whole state: the directory, the links, the startup files, the tools.
pub async fn status() -> Status {
    let bin = bin_dir();
    let home = dirs::home_dir();
    let macos = cfg!(target_os = "macos");

    let own = OWN
        .iter()
        .map(|(id, about)| OwnStatus {
            id,
            about,
            built: shipped(id).map(|p| p.display().to_string()),
            linked: bin
                .as_ref()
                .map(|dir| dir.join(exe_name(id)))
                .filter(|p| p.exists())
                .map(|p| p.display().to_string()),
        })
        .collect();

    let shells = SHELLS
        .iter()
        .map(|s| {
            let path = home.as_deref().and_then(|h| rc_path(s.id, h, macos));
            let text = path
                .as_deref()
                .filter(|p| p.is_file())
                .and_then(|p| std::fs::read_to_string(p).ok());
            let block = bin.as_deref().and_then(|dir| block(s.id, dir));
            let (installed, current) = match (&text, &block) {
                (Some(text), Some(block)) => state(text, block),
                _ => (false, false),
            };
            ShellStatus {
                id: s.id,
                label: s.label,
                path: path.as_ref().map(|p| p.display().to_string()),
                exists: text.is_some(),
                installed,
                current,
                line: bin.as_deref().and_then(|dir| line(s.id, dir)),
            }
        })
        .collect();

    let mut tools = Vec::with_capacity(TOOLS.len());
    for spec in TOOLS {
        let (source, path) = locate(spec.program);
        let runnable = path.clone().unwrap_or_else(|| PathBuf::from(spec.program));
        let version = probe(&runnable, spec.version_args).await;
        // A program that would not run is not present, whatever `PATH` says: a
        // dangling symlink and a half-deleted install both leave a file behind.
        let source = if version.is_none() && source == Source::System {
            Source::Missing
        } else {
            source
        };
        let here = spec
            .download
            .and_then(|d| asset(d, std::env::consts::OS, std::env::consts::ARCH));
        tools.push(ToolStatus {
            id: spec.id,
            label: spec.label,
            program: spec.program,
            why: spec.why,
            source,
            version,
            path: path.map(|p| p.display().to_string()),
            offers: spec.download.map(|d| d.version.to_string()),
            publisher: spec.download.map(|d| d.publisher),
            available_here: spec.download.is_none() || here.is_some(),
        });
    }

    Status {
        bin_dir: bin.map(|p| p.display().to_string()),
        on_path: on_path(),
        current_shell: current_shell(),
        own,
        shells,
        tools,
    }
}

// -------------------------------------------------------------------- writes

/// The directory, created.
fn ensure_bin_dir() -> Result<PathBuf> {
    let dir = bin_dir().ok_or_else(|| {
        Error::new(
            Code::IoError,
            "this platform has no data directory to install into",
        )
    })?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("creating {}", dir.display()), e))?;
    Ok(dir)
}

/// Point `bin_dir` at the binaries this build produced.
///
/// Symlinks on unix so an updated build is picked up without pressing anything
/// again; a copy on Windows, where creating a symlink needs Developer Mode or
/// an administrator and this app asks for neither. The copy is the reason
/// Windows users have to press it again after an update, which the pane says.
pub fn link() -> Result<Vec<String>> {
    let dir = ensure_bin_dir()?;
    let mut written = Vec::new();

    for (name, _) in OWN {
        let Some(source) = shipped(name) else {
            continue;
        };
        let target = dir.join(exe_name(name));

        // Replaced rather than skipped: the source moves when somebody
        // switches between a checkout and an installed app, and a link left
        // pointing at a deleted `target/release` is the failure this whole
        // module exists to stop.
        if target.exists() || target.symlink_metadata().is_ok() {
            std::fs::remove_file(&target)
                .map_err(|e| Error::io(format!("replacing {}", target.display()), e))?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&source, &target)
            .map_err(|e| Error::io(format!("linking {}", target.display()), e))?;
        #[cfg(not(unix))]
        std::fs::copy(&source, &target)
            .map(|_| ())
            .map_err(|e| Error::io(format!("copying to {}", target.display()), e))?;

        written.push(target.display().to_string());
    }

    if written.is_empty() {
        return Err(Error::new(
            Code::NotFound,
            "neither `stackvo` nor `stackvo-mcp` was found next to this application",
        )
        .with_hint(crate::hints::CLI_NOT_BUILT));
    }
    Ok(written)
}

/// Take the links back out. The directory stays — it may hold a managed tool.
pub fn unlink() -> Result<Vec<String>> {
    let Some(dir) = bin_dir() else {
        return Ok(Vec::new());
    };
    let mut removed = Vec::new();
    for (name, _) in OWN {
        let target = dir.join(exe_name(name));
        if target.symlink_metadata().is_ok() {
            std::fs::remove_file(&target)
                .map_err(|e| Error::io(format!("removing {}", target.display()), e))?;
            removed.push(target.display().to_string());
        }
    }
    Ok(removed)
}

/// Read, edit and write one startup file, keeping a copy of what was there.
///
/// `rules::rewrite`'s shape, with one addition it does not need: a size limit.
/// A rules file is one this app may create, and a `.zshrc` is one it never
/// creates from nothing on a machine that has been used — reading an arbitrary
/// file called `.bashrc` into memory because it was in the way is not something
/// to do unbounded.
fn rewrite(path: &Path, edit: impl FnOnce(&str) -> String) -> Result<String> {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > MOST_RC_BYTES {
            return Err(Error::new(
                Code::Forbidden,
                format!(
                    "{} is larger than {MOST_RC_BYTES} bytes and was left alone",
                    path.display()
                ),
            )
            .with_hint(crate::hints::PATH_ENTRY_BY_HAND));
        }
    }

    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let updated = edit(&existing);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }

    // Only when there was something to lose — an empty `.stackvo-backup` reads
    // as a startup file this app ate.
    if !existing.is_empty() {
        let mut name = path.file_name().unwrap_or_default().to_os_string();
        name.push(BACKUP_SUFFIX);
        let backup = path.with_file_name(name);
        std::fs::write(&backup, &existing)
            .map_err(|e| Error::io(format!("writing {}", backup.display()), e))?;
    }

    crate::atomic::write(path, &updated)?;
    Ok(path.display().to_string())
}

/// The startup file for one shell, on this machine.
fn rc_for(id: &str) -> Result<PathBuf> {
    if shell(id).is_none() {
        return Err(Error::new(
            Code::InvalidInput,
            format!("unknown shell {id}"),
        ));
    }
    let home = dirs::home_dir()
        .ok_or_else(|| Error::new(Code::IoError, "this account has no home directory"))?;
    rc_path(id, &home, cfg!(target_os = "macos"))
        .ok_or_else(|| Error::new(Code::InvalidInput, format!("unknown shell {id}")))
}

/// Link the binaries and write the `PATH` line into one shell's startup file.
///
/// Both halves, because they are one intention. A `PATH` entry pointing at an
/// empty directory is not a smaller version of this feature; it is the version
/// where the next `stackvo` still says "command not found" and the user
/// concludes the button is broken.
pub fn path_apply(id: &str) -> Result<String> {
    let path = rc_for(id)?;
    let dir = ensure_bin_dir()?;
    // Best effort: a checkout with no build of the CLI is a real state, and
    // refusing to put the directory on `PATH` because of it would mean pressing
    // this again after every `cargo build`.
    let _ = link();

    let block = block(id, &dir)
        .ok_or_else(|| Error::new(Code::InvalidInput, format!("unknown shell {id}")))?;
    rewrite(&path, |text| merge(text, &block))
}

/// Take the line back out. The file stays, minus our region.
///
/// The links stay too. Somebody who removes the `PATH` entry has said where
/// they want `stackvo` looked up from, not that they want it deleted — and
/// `agents.rs` may have registered the linked path with six assistants.
pub fn path_remove(id: &str) -> Result<String> {
    let path = rc_for(id)?;
    if !path.is_file() {
        return Ok(path.display().to_string());
    }
    rewrite(&path, strip)
}

// ------------------------------------------------------------- the download

/// A tool is a program, not an archive. Anything this size is not one.
const MOST_TOOL_BYTES: u64 = 64 * 1024 * 1024;

/// Fetch, verify against the compiled-in digest, and install one tool.
///
/// The order is the point: nothing is written into `bin_dir` until the digest
/// matches. A partial write that is later found to be wrong is a program on
/// somebody's `PATH` that nobody chose.
pub async fn install(id: &str) -> Result<String> {
    let spec =
        tool(id).ok_or_else(|| Error::new(Code::InvalidInput, format!("unknown tool {id}")))?;
    let download = spec.download.ok_or_else(|| {
        Error::new(
            Code::Unsupported,
            format!("{} is not a tool this app installs", spec.label),
        )
        .with_hint(crate::hints::TOOL_IS_NOT_MANAGED)
    })?;

    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    let (asset_name, expected) = asset(download, os, arch).ok_or_else(|| {
        Error::new(
            Code::Unsupported,
            format!(
                "{} {} has no {os}/{arch} build",
                spec.label, download.version
            ),
        )
    })?;

    let url = format!("{}{asset_name}", download.url_prefix);
    let bytes = fetch(&url).await?;

    let actual = crate::pkg::sha256_hex(&bytes);
    if actual != expected {
        return Err(Error::new(
            Code::Forbidden,
            format!("{url} is not the file this build pins — expected {expected}, found {actual}"),
        )
        .with_hint(crate::hints::TOOL_DIGEST_MISMATCH));
    }

    let dir = ensure_bin_dir()?;
    let target = dir.join(exe_name(spec.program));

    // Written beside the target and renamed, for `atomic::write`'s reason: a
    // half-written executable on `PATH` is worse than none, and a rename within
    // one directory is the only move that cannot leave one.
    let staging = target.with_extension("stackvo-part");
    std::fs::write(&staging, &bytes)
        .map_err(|e| Error::io(format!("writing {}", staging.display()), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        // 0755: readable and runnable by this user, and by nobody else's write.
        std::fs::set_permissions(&staging, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| Error::io(format!("making {} executable", staging.display()), e))?;
    }

    std::fs::rename(&staging, &target)
        .map_err(|e| Error::io(format!("installing {}", target.display()), e))?;

    Ok(target.display().to_string())
}

/// Remove the managed copy. A system copy is never touched.
pub fn remove(id: &str) -> Result<String> {
    let spec =
        tool(id).ok_or_else(|| Error::new(Code::InvalidInput, format!("unknown tool {id}")))?;
    let dir = bin_dir()
        .ok_or_else(|| Error::new(Code::IoError, "this platform has no data directory"))?;
    let target = dir.join(exe_name(spec.program));
    if !target.is_file() {
        return Err(Error::new(
            Code::NotFound,
            format!("{} was not installed by this app", spec.label),
        ));
    }
    std::fs::remove_file(&target)
        .map_err(|e| Error::io(format!("removing {}", target.display()), e))?;
    Ok(target.display().to_string())
}

/// One HTTPS GET, counted as it arrives.
///
/// `market::get`'s shape without its ETag half — there is no cache here, and a
/// pinned URL either answers with the bytes this build knows or is refused.
async fn fetch(url: &str) -> Result<Vec<u8>> {
    use futures_util::StreamExt as _;

    let client = reqwest::Client::builder()
        .user_agent(concat!("stackvo/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| Error::new(Code::NetworkError, format!("building an HTTP client: {e}")))?;

    let response = client.get(url).send().await.map_err(|e| {
        // The URL, always — it is the only part of this a person can act on.
        Error::new(
            Code::NetworkError,
            format!("{url} could not be reached: {e}"),
        )
    })?;

    if !response.status().is_success() {
        return Err(Error::new(
            Code::NetworkError,
            format!("{url} answered {}", response.status()),
        ));
    }

    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|e| Error::new(Code::NetworkError, format!("{url} stopped sending: {e}")))?;
        // Counted rather than trusted: `Content-Length` is something the sender
        // writes, and this is the number of bytes that actually arrived.
        if body.len() as u64 + chunk.len() as u64 > MOST_TOOL_BYTES {
            return Err(Error::new(
                Code::Forbidden,
                format!("{url} is larger than {MOST_TOOL_BYTES} bytes and was abandoned"),
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin() -> PathBuf {
        PathBuf::from("/home/x/.local/share/stackvo/bin")
    }

    #[test]
    fn every_shell_has_a_line_and_a_unique_id() {
        let mut ids: Vec<&str> = SHELLS.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "two shells share an id");

        for s in SHELLS {
            assert!(line(s.id, &bin()).is_some(), "{} has no line", s.id);
            assert!(block(s.id, &bin()).is_some(), "{} has no block", s.id);
        }
    }

    #[test]
    fn the_line_puts_us_first() {
        // Last would mean a managed tool losing to a half-removed system one,
        // which is the state somebody presses this button to leave.
        let posix = line("zsh", &bin()).unwrap();
        assert!(posix.ends_with(":$PATH\""), "{posix}");
        // And inside the quotes, or a PATH that already holds a space is
        // word-split into several arguments to `export`.
        assert!(!posix.contains("\":$PATH"), "{posix}");
        assert!(line("fish", &bin()).unwrap().ends_with(" $PATH"));
    }

    #[test]
    fn a_space_in_the_path_survives_every_shell() {
        // The default on macOS has one before anybody does anything unusual.
        let dir = PathBuf::from("/Users/x/Library/Application Support/StackVo/bin");
        for s in SHELLS {
            let line = line(s.id, &dir).unwrap();
            assert!(line.contains("Application Support"), "{}: {line}", s.id);
            assert!(
                line.contains('"') || line.contains('\''),
                "{} left the path unquoted: {line}",
                s.id
            );
        }
    }

    #[test]
    fn the_awkward_characters_are_escaped() {
        let dir = PathBuf::from("/home/o\"brien/$PATH/bin");
        let posix = line("bash", &dir).unwrap();
        assert!(posix.contains("o\\\"brien"), "{posix}");
        assert!(posix.contains("\\$PATH/bin"), "{posix}");

        let pwsh = line("powershell", &PathBuf::from("/home/o'brien/bin")).unwrap();
        assert!(pwsh.contains("o''brien"), "{pwsh}");
    }

    #[test]
    fn bash_reads_a_different_file_on_macos() {
        let home = Path::new("/home/x");
        assert_eq!(
            rc_path("bash", home, false),
            Some(home.join(".bashrc")),
            "Linux terminals open non-login shells"
        );
        assert_eq!(
            rc_path("bash", home, true),
            Some(home.join(".bash_profile")),
            "Terminal.app opens login shells"
        );
        // And only bash. zsh reads .zshrc on both.
        assert_eq!(rc_path("zsh", home, true), Some(home.join(".zshrc")));
    }

    /// The completion has to come **after** the `PATH` line, because it names
    /// the binary. Registered for a command that is not yet on `PATH` it does
    /// not fail — it simply never fires, which is the failure nobody reports.
    #[test]
    fn the_block_carries_the_completion_after_the_path_line() {
        for shell in SHELLS {
            let block =
                block(shell.id, &bin()).unwrap_or_else(|| panic!("no block for `{}`", shell.id));
            let path_line = line(shell.id, &bin()).unwrap();

            let at_path = block.find(&path_line).expect("the PATH line");
            let at_stub = block
                .find("stackvo complete --word")
                .unwrap_or_else(|| panic!("`{}`'s block carries no completion", shell.id));
            assert!(
                at_path < at_stub,
                "`{}` registers a completion before the binary is on PATH",
                shell.id
            );

            // Still one region, so one `path-remove` takes both back out.
            assert_eq!(block.matches(BEGIN).count(), 1);
            assert_eq!(block.matches(END).count(), 1);
            assert_eq!(strip(&merge("# mine\n", &block)).trim(), "# mine");
        }
    }

    #[test]
    fn an_empty_file_gains_only_the_block() {
        let block = block("zsh", &bin()).unwrap();
        assert_eq!(merge("", &block), block);
    }

    #[test]
    fn the_block_is_appended_never_prepended() {
        // The lines above ours are the ones the user put there deliberately.
        let block = block("zsh", &bin()).unwrap();
        let out = merge("export EDITOR=vim\n", &block);
        assert!(out.starts_with("export EDITOR=vim\n"), "{out}");
        assert!(out.ends_with(&block), "{out}");
    }

    #[test]
    fn a_second_apply_replaces_rather_than_repeats() {
        let block = block("zsh", &bin()).unwrap();
        let once = merge("# mine\n", &block);
        let twice = merge(&once, &block);
        assert_eq!(once, twice);
        assert_eq!(twice.matches(BEGIN).count(), 1);
    }

    #[test]
    fn a_moved_directory_is_a_replacement_not_a_second_entry() {
        // The state a user who moves between a checkout and an installed app is
        // in. Two entries would both be on PATH and the older one would win.
        let old = block("zsh", Path::new("/old/bin")).unwrap();
        let new = block("zsh", Path::new("/new/bin")).unwrap();
        let out = merge(&merge("# mine\n", &old), &new);
        assert!(out.contains("/new/bin"), "{out}");
        assert!(!out.contains("/old/bin"), "{out}");
    }

    #[test]
    fn removing_leaves_every_other_byte() {
        let block = block("zsh", &bin()).unwrap();
        let before = "export EDITOR=vim\nalias ll='ls -l'\n";
        assert_eq!(strip(&merge(before, &block)), before);
    }

    #[test]
    fn removing_from_a_file_with_no_block_changes_nothing() {
        let text = "export EDITOR=vim\n";
        assert_eq!(strip(text), text);
    }

    #[test]
    fn state_separates_installed_from_current() {
        let old = block("zsh", Path::new("/old/bin")).unwrap();
        let new = block("zsh", Path::new("/new/bin")).unwrap();
        let text = merge("", &old);
        assert_eq!(state(&text, &old), (true, true));
        assert_eq!(
            state(&text, &new),
            (true, false),
            "a block from another data directory is installed but not current"
        );
        assert_eq!(state("", &new), (false, false));
    }

    #[test]
    fn the_catalogue_is_host_tools_only() {
        // The rule this module exists to hold. A container's copy of composer
        // is the project's business; adding it here would be two answers to
        // "which composer runs" and the host's would be the wrong one.
        for banned in ["composer", "node", "npm", "npx", "bun", "wp", "php"] {
            assert!(
                !TOOLS.iter().any(|t| t.program == banned),
                "{banned} runs in the project's container"
            );
        }
    }

    #[test]
    fn every_tool_has_a_unique_id_and_a_reason() {
        let mut ids: Vec<&str> = TOOLS.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);

        for t in TOOLS {
            assert!(!t.why.is_empty(), "{} says nothing about why", t.id);
            assert!(!t.version_args.is_empty(), "{} cannot be probed", t.id);
        }
    }

    #[test]
    fn every_pinned_digest_is_a_sha256() {
        // A typo here is a tool that can never install, and the failure would
        // arrive on somebody's machine after a 5 MB download.
        for (os, arch, asset, sha) in MKCERT.assets {
            assert_eq!(sha.len(), 64, "{asset}");
            assert!(
                sha.chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "{asset}: {sha}"
            );
            assert!(
                asset.contains(MKCERT.version),
                "{asset} is not the pinned version {}",
                MKCERT.version
            );
            assert!(!os.is_empty() && !arch.is_empty());
        }
    }

    #[test]
    fn no_two_platforms_share_a_digest() {
        // Copy-paste is how a pin table goes wrong, and the result installs the
        // wrong architecture's binary on a machine that cannot run it.
        let mut seen: Vec<&str> = MKCERT.assets.iter().map(|(_, _, _, sha)| *sha).collect();
        seen.sort_unstable();
        let count = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), count, "two assets pin the same digest");
    }

    #[test]
    fn every_platform_this_app_ships_for_has_an_asset() {
        // release.yml builds six targets; a tool the pane offers on one of them
        // and cannot install is a button that fails after being pressed.
        for (os, arch) in [
            ("macos", "x86_64"),
            ("macos", "aarch64"),
            ("linux", "x86_64"),
            ("linux", "aarch64"),
            ("windows", "x86_64"),
            ("windows", "aarch64"),
        ] {
            assert!(
                asset(&MKCERT, os, arch).is_some(),
                "no mkcert build pinned for {os}/{arch}"
            );
        }
    }

    #[test]
    fn the_download_url_is_https_and_names_the_publisher() {
        assert!(MKCERT.url_prefix.starts_with("https://"));
        assert!(MKCERT.url_prefix.ends_with('/'));
        assert!(!MKCERT.publisher.is_empty());
    }

    #[test]
    fn an_unknown_shell_has_no_file_and_no_line() {
        assert!(line("csh", &bin()).is_none());
        assert!(rc_path("csh", Path::new("/home/x"), false).is_none());
        assert!(shell("csh").is_none());
    }

    /// Both branches, against a directory this test owns.
    ///
    /// It used to call `resolve` and assert the fallback, on the reasoning that
    /// "nothing is installed in a test run". That is a claim about the machine
    /// rather than about the code, and it stopped being true the moment
    /// somebody installed mkcert with the app they were building — a full suite
    /// that fails on the author's machine and passes on everyone else's.
    #[test]
    fn resolve_prefers_the_managed_copy_and_falls_back_to_the_bare_name() {
        let dir = std::env::temp_dir().join(format!("stackvo-resolve-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Nothing there: the caller gets the bare name and `PATH` decides.
        assert_eq!(resolve_in(Some(&dir), "mkcert"), PathBuf::from("mkcert"));
        // No directory at all — the state before `path-install` has ever run.
        assert_eq!(resolve_in(None, "mkcert"), PathBuf::from("mkcert"));

        // Installed: our copy wins, so the app finds a tool it just installed
        // without waiting for a terminal restart.
        let installed = dir.join(exe_name("mkcert"));
        std::fs::write(&installed, "").unwrap();
        assert_eq!(resolve_in(Some(&dir), "mkcert"), installed);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn both_of_this_repositorys_binaries_are_listed() {
        let ids: Vec<&str> = OWN.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&"stackvo"));
        assert!(ids.contains(&"stackvo-mcp"), "agents.rs registers this one");
    }
}
