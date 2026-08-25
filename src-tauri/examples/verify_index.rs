//! Does the app accept this signature? Asked with the app's own verifier.
//!
//! `tools/keys.sh sign` produces a `registry.json.minisig`, and until this
//! existed nothing between that file and a user's machine ever checked it. The
//! failure that leaves is quiet and total: the content key sits in the same
//! directory as the updater key, the two files differ by one word, and an index
//! signed with the wrong one is a file that looks finished, uploads cleanly, and
//! is refused by **every** shipped build the moment somebody refreshes. Nothing
//! in the publishing machine's hands would have said so.
//!
//! So the ceremony asks the thing that will actually judge it. Not a
//! re-implementation of the check — `signing::Keys::pinned()` is the set a
//! release trusts, `verify` is the function it calls, and a second copy of
//! either would be a copy that eventually disagrees with the first while both
//! pass their own tests.
//!
//!   cargo run --example verify_index -- <file> [<file>.minisig] [--key <line>]…
//!
//! `--key` is the mirror case, and it is the same key a machine would carry in
//! `policy.market.additionalKeys` — an organisation signing its own index is
//! not asking whether *this* build trusts it, it is asking whether the machines
//! it configured will.
//!
//! Exit status is the answer: 0 verified, 1 refused, 2 the arguments were wrong.
//! A ceremony step reads that rather than the prose.

use stackvo_desktop_lib::signing::Keys;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut file: Option<PathBuf> = None;
    let mut signature: Option<PathBuf> = None;
    let mut extra: Vec<String> = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--key" => match args.next() {
                Some(key) => extra.push(key),
                None => return usage("--key needs the key line that follows it"),
            },
            "-h" | "--help" => return usage("usage"),
            other if other.starts_with('-') => {
                return usage(&format!("{other} is not an option this reads"))
            }
            other if file.is_none() => file = Some(PathBuf::from(other)),
            other if signature.is_none() => signature = Some(PathBuf::from(other)),
            other => return usage(&format!("{other} is one argument too many")),
        }
    }

    let Some(file) = file else {
        return usage("no file to verify");
    };
    // The name `market::refresh` fetches, so the common call is one argument.
    let signature = signature.unwrap_or_else(|| {
        let mut name = file.clone().into_os_string();
        name.push(".minisig");
        PathBuf::from(name)
    });

    let bytes = match std::fs::read(&file) {
        Ok(bytes) => bytes,
        Err(e) => return fail(&format!("reading {}: {e}", file.display())),
    };
    let text = match std::fs::read_to_string(&signature) {
        Ok(text) => text,
        Err(e) => return fail(&format!("reading {}: {e}", signature.display())),
    };

    // `pinned()` and not `pinned().with_policy(policy::current())`: this asks
    // what a shipped build trusts, and the machine running the ceremony is not
    // one of the machines being asked about. A mirror's key is named on the
    // command line instead, deliberately visible in the command somebody ran.
    let keys = Keys::pinned().with_policy(&extra);

    match keys.verify(&bytes, &text) {
        Ok(key) => {
            println!("verified  {}  by key {}", file.display(), key.id());
            ExitCode::SUCCESS
        }
        Err(e) => {
            let mut message = format!("REFUSED   {}: {}", file.display(), e.message);
            if let Some(hint) = e.hint {
                message.push_str(&format!("\n          {hint}"));
            }
            fail(&message)
        }
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(1)
}

fn usage(message: &str) -> ExitCode {
    eprintln!("{message}");
    eprintln!("usage: verify_index <file> [<signature>] [--key <public key line>]…");
    eprintln!("       the signature defaults to <file>.minisig, the name the app fetches");
    ExitCode::from(2)
}
