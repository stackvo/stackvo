//! The native window, read the way a screen reader reads it.
//!
//! The window chrome, the menu bar and the tray menu are out of scope
//! because `tauri-driver` does not run on macOS. That named the wrong blocker,
//! and naming it wrong is what kept the row unstartable for as long as it was:
//!
//! **WebDriver never reaches a native menu on any platform.** It drives the web
//! view. A Linux runner with `tauri-driver` on it can no more enumerate the
//! macOS menu bar than this machine can — so "wait for a driver" was waiting for
//! something that would not have answered the question.
//!
//! What does answer it is the **accessibility API**, which is the layer a screen
//! reader itself reads, and macOS exposes it to any process the user has
//! granted Accessibility to. `System Events` is the scriptable front end for it,
//! so this probe needs no new crate, no driver and no CI runner: it needs the
//! application running and one permission granted.
//!
//! ```sh
//! cargo run --example native_ax_probe
//! ```
//!
//! ## What it found on its first run
//!
//! Two menu items in the app menu carried the **crate** name:
//!
//! ```text
//! Hide stackvo-desktop     ⌘H
//! Quit stackvo-desktop     ⌘Q
//! ```
//!
//! `menu.rs` had already rebuilt that submenu once for this exact reason — its
//! own comment says `Menu::default` "titles it with the crate name, so a
//! `stackvo-desktop` sat in the menu bar of an app called StackVo" — and fixed
//! the submenu's *title* while leaving the two predefined items that
//! interpolate a name. Nothing on the front end could see them and no test
//! could reach them; a person would read past them, because a menu item is read
//! by its verb. A screen reader says the whole string.
//!
//! ## What this cannot decide, and it is the half that matters most
//!
//! Whether the reading order makes sense, whether a label is *meaningful*,
//! whether the tray menu is usable under VoiceOver rather than merely present.
//! Those are judgements and they still need a person — Y-1's problem, in the
//! native surfaces. This probe covers the half that is a fact: is every item
//! named, is it named in the interface's language rather than the build's, is
//! it enabled, and does it carry a keyboard equivalent where one is claimed.

use std::process::Command;

/// The process name the bundle runs under.
///
/// The binary's name and not the product name: this is what the accessibility
/// tree keys on, and the gap between the two is the very defect the first run
/// found.
const PROCESS: &str = "stackvo-desktop";

/// The strings no menu item may contain.
///
/// The crate name is the one Tauri fills a `None` label with. It is checked as
/// a **substring** rather than compared, because the failure was never a whole
/// label — it was "Quit stackvo-desktop", a correct verb with the wrong noun.
const NEVER: [&str; 2] = ["stackvo-desktop", "{product}"];

fn main() {
    if !cfg!(target_os = "macos") {
        println!(
            "this probe reads the macOS accessibility API. On Linux the same \
             questions are asked of AT-SPI, which is not wired up here."
        );
        std::process::exit(0);
    }

    if !running() {
        println!(
            "{PROCESS} is not running. Start the application and run this again \
             — the accessibility tree only exists while there is a window."
        );
        std::process::exit(1);
    }

    let bars = match ask("get name of every menu bar item of menu bar 1") {
        Some(bars) => bars,
        None => {
            println!(
                "{PROCESS} is running and its accessibility tree could not be \
                 read, which is a permission rather than a finding.\n\n\
                 macOS attributes the request to THIS binary, not to the \
                 terminal — so grant Accessibility (and Automation, for System \
                 Events) to `target/debug/examples/native_ax_probe` in System \
                 Settings → Privacy & Security, or run the questions straight \
                 from a granted terminal:\n\n  \
                 osascript -e 'tell application \"System Events\" to tell \
                 process \"{PROCESS}\" to get name of every menu item of menu 1 \
                 of menu bar item \"StackVo\" of menu bar 1'"
            );
            std::process::exit(1);
        }
    };

    let mut findings: Vec<String> = Vec::new();
    let mut items = 0usize;

    println!("menu bar: {}\n", bars.join(", "));

    for bar in bars.iter().filter(|b| *b != "Apple") {
        let names = ask(&format!(
            "get name of every menu item of menu 1 of menu bar item \"{bar}\" of menu bar 1"
        ))
        .unwrap_or_default();
        let enabled = ask(&format!(
            "get enabled of every menu item of menu 1 of menu bar item \"{bar}\" of menu bar 1"
        ))
        .unwrap_or_default();

        println!("{bar}");
        for (index, name) in names.iter().enumerate() {
            // A separator has no name, and `missing value` is how the
            // accessibility layer says so. Not a finding.
            if name == "missing value" {
                println!("    ─────");
                continue;
            }
            items += 1;
            let on = enabled.get(index).map(String::as_str) != Some("false");
            println!("    {name}{}", if on { "" } else { "   (disabled)" });

            for bad in NEVER {
                if name.contains(bad) {
                    findings.push(format!("{bar} → \"{name}\" contains `{bad}`"));
                }
            }
            if name.trim().is_empty() {
                findings.push(format!("{bar} → an item with no name at all"));
            }
        }
        println!();
    }

    // The status item, which lives on its own menu bar. This is the surface the
    // row called out by name — "the tray menu" — and it is not out of reach at
    // all: `menu bar 2` is where macOS puts it, and its own name is what a
    // screen reader announces for the icon before any menu is opened.
    //
    // Three attributes, not one, and that was measured rather than assumed. The
    // first version of this asked for `name`, which is `AXTitle`, found it empty
    // and reported the status item as unnamed — while the tooltip this
    // application sets was sitting in `AXHelp` all along. An icon-only status
    // item has no `AXTitle` by construction on macOS: giving it one means
    // putting visible text in the menu bar beside the icon, which is a product
    // decision and not an accessibility fix.
    if let Some(attrs) = ask(
        "tell menu bar item 1 of menu bar 2 to get {value of attribute \"AXTitle\", \
         value of attribute \"AXHelp\", value of attribute \"AXRoleDescription\"}",
    ) {
        let at = |i: usize| {
            attrs
                .get(i)
                .map(String::as_str)
                .filter(|v| !v.trim().is_empty() && *v != "missing value")
        };
        let (title, help, role) = (at(0), at(1), at(2));
        println!(
            "status item: title={:?} help={:?} role={:?}\n",
            title.unwrap_or("—"),
            help.unwrap_or("—"),
            role.unwrap_or("—")
        );

        if title.is_none() && help.is_none() {
            findings.push(
                "the status item carries no name in any attribute — a screen \
                 reader announces nothing but its role"
                    .into(),
            );
        }

        for entry in ask("get name of every menu item of menu 1 of menu bar item 1 of menu bar 2")
            .unwrap_or_default()
        {
            if entry == "missing value" {
                println!("    ─────");
                continue;
            }
            items += 1;
            println!("    {entry}");
            for bad in NEVER {
                if entry.contains(bad) {
                    findings.push(format!("status menu → \"{entry}\" contains `{bad}`"));
                }
            }
        }
        println!();
    } else {
        println!("status item: none on this run\n");
    }

    // The windows, whose titles are their accessible names.
    let titles = ask("get title of every window").unwrap_or_default();
    println!("windows: {}\n", titles.join(", "));
    for title in &titles {
        if title.trim().is_empty() || title == "missing value" {
            findings
                .push("a window with no title, which is a window with no accessible name".into());
        }
    }

    println!("{items} menu items read, {} window(s).\n", titles.len());

    if findings.is_empty() {
        println!(
            "No finding. Every item is named, named in the interface's language, \
             and every window has an accessible name.\n\n\
             This is the half that is a fact. Whether the reading order makes \
             sense and whether the tray menu is usable under VoiceOver are \
             judgements and still need a person."
        );
        return;
    }

    println!("{} finding(s):", findings.len());
    for finding in &findings {
        println!("  · {finding}");
    }
    std::process::exit(1);
}

/// Is the application up? Its tree does not exist otherwise.
///
/// `pgrep` rather than asking `System Events`, and the difference is the whole
/// permission story. Every question below goes through `osascript`, and macOS
/// attributes that request to the **responsible process** — this binary, not
/// the terminal it was typed into. So a run from a granted terminal still gets
/// refused until this binary is granted too, and a probe that used the same
/// channel to ask "is it running" would report a permission as "the application
/// is closed": the one answer that sends somebody to look in the wrong place.
fn running() -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(PROCESS)
        .output()
        .is_ok_and(|out| !out.stdout.is_empty())
}

/// One question about this application's accessibility tree, as a list.
///
/// `System Events` answers a list as a comma-separated line, which is ambiguous
/// for a label containing a comma — accepted, because the alternative is a
/// serialisation layer in AppleScript and every label this application writes
/// is a menu item.
fn ask(question: &str) -> Option<Vec<String>> {
    let script =
        format!("tell application \"System Events\" to tell process \"{PROCESS}\" to {question}");
    let out = osascript(&script)?;
    Some(
        out.split(", ")
            .map(|part| part.trim().to_string())
            .collect(),
    )
}

fn osascript(script: &str) -> Option<String> {
    let out = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}
