//! What this repository can decide about the windows the operating system draws.
//!
//! Y-2 in `docs/durum.md`: the measurement drives the front end in a browser
//! engine, so the window chrome, the menu bar and the tray menu are out of
//! scope. `tauri-driver` would cover them and does not run on macOS, which is
//! where this application is developed — so the row is blocked on a machine.
//!
//! That was read as "nothing here is checkable", and it is not. A driver is
//! needed to *operate* those surfaces; it is not needed to know whether they
//! have names, whether the names are in the interface's language, or whether
//! the window carries the properties the accessibility statement leans on.
//! Those are facts about this tree, and the way they regress is silent: a
//! window's title is a string in a builder, and nothing on screen looks wrong
//! when it is empty.
//!
//! **It was empty.** `menu::open_about` built the About window with `.title("")`
//! and a window's title *is* its accessible name — what the window list
//! announces, what the Window menu shows, and what a screen reader reads when
//! focus lands inside. The one window whose entire job is to say which version
//! is installed was handing that answer over unnamed.
//!
//! What still needs the machine, and what this file deliberately does not
//! claim: whether the menu bar can be *reached and operated* from the keyboard
//! on each platform, whether focus order through the native chrome is sane, and
//! how the tray menu behaves under a screen reader. Those are the audit, and
//! the audit is still owed.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

fn config() -> serde_json::Value {
    serde_json::from_str(&read("src-tauri/tauri.conf.json")).expect("tauri.conf.json parses")
}

/// Every window declared in the configuration.
fn declared_windows() -> Vec<serde_json::Value> {
    config()["app"]["windows"]
        .as_array()
        .expect("the configuration declares windows")
        .clone()
}

/// A window with no title has no accessible name.
///
/// The main window's is in the configuration; the About window's is built in
/// `menu.rs` and is checked separately below, because it is a string in Rust
/// rather than a value in JSON.
#[test]
fn every_declared_window_carries_a_title() {
    for window in declared_windows() {
        let title = window["title"].as_str().unwrap_or("");
        assert!(
            !title.trim().is_empty(),
            "a window is declared with no title, which is a window with no \
             accessible name: {window}"
        );
    }
}

/// The About window's title, which was `""` until Y-2.
///
/// Checked against the source rather than by building a window: constructing a
/// `WebviewWindow` needs a running Tauri application, and what is being kept
/// here is the decision — that the title comes from the label catalogue — not
/// the runtime behaviour of a builder.
#[test]
fn the_about_window_is_named_and_named_in_the_interfaces_language() {
    let source = read("src-tauri/src/menu.rs");

    assert!(
        !source.contains(".title(\"\")"),
        "the About window is built with an empty title again. A window's title \
         IS its accessible name — the window list, the Window menu and every \
         screen reader read it — and this is the window that answers \"which \
         version is installed\"."
    );
    assert!(
        source.contains(".title(about_title())"),
        "the About window's title no longer comes from `about_title`"
    );
    assert!(
        source.contains("crate::tray::menu_labels().about"),
        "`about_title` no longer draws from the label catalogue, so the window \
         is named in the build's language rather than the interface's"
    );
}

/// A title set once at build time is the boot language, forever.
///
/// The menu bar has been relabelled on a language change since the tray was;
/// the window this menu opens was not, and a window announcing itself in the
/// previous language is the same defect one window along.
#[test]
fn changing_language_renames_the_about_window_too() {
    let tray = read("src-tauri/src/tray.rs");
    let relabel = tray
        .split_once("pub fn relabel<R: Runtime>")
        .expect("tray.rs relabels on a language change")
        .1;
    // To the end of the function, which is the next item at column zero.
    let body = relabel.split("\n}").next().unwrap_or(relabel);

    assert!(
        body.contains("crate::menu::ABOUT_LABEL"),
        "`relabel` no longer reaches the About window, so it keeps whatever \
         language it was opened in"
    );
    assert!(
        body.contains("set_title"),
        "`relabel` reaches the About window and does not re-title it"
    );

    // And the other half: a window opened *after* the change never saw
    // `relabel` at all, so the reopen path has to re-title as well.
    let menu = read("src-tauri/src/menu.rs");
    let reopen = menu
        .split_once("if let Some(window) = app.get_webview_window(ABOUT_LABEL)")
        .expect("open_about focuses an existing window")
        .1;
    let block = reopen.split("return Ok(())").next().unwrap_or(reopen);
    assert!(
        block.contains("set_title"),
        "reopening the About window does not re-title it, so a window hidden \
         across a language change comes back in the old language"
    );
}

/// This app builds exactly one window of its own, and it is named.
///
/// A sweep rather than a list: the failure is a *new* window added with no
/// title, which is exactly the thing a list of known windows cannot catch.
#[test]
fn no_window_is_built_without_a_title() {
    let dir = repo_root().join("src-tauri/src");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("src-tauri/src is readable")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();

    let mut unnamed = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a source file is readable");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

        for (index, _) in text.match_indices("WebviewWindowBuilder::new(") {
            // The builder chain, to the `.build()` that ends it.
            let rest = &text[index..];
            let chain = rest.split(".build()").next().unwrap_or(rest);
            if !chain.contains(".title(") {
                unnamed.push(format!("{name} (byte {index})"));
            }
        }
    }

    assert!(
        unnamed.is_empty(),
        "these windows are built without a title, and a window with no title \
         has no accessible name: {unnamed:?}"
    );
}

/// The two app-menu items that interpolate the application's own name.
///
/// `PredefinedMenuItem::hide(app, None)` and its `quit` sibling fill that hole
/// with the **crate** name, so an application called StackVo offered "Hide
/// stackvo-desktop" and "Quit stackvo-desktop". Nothing on the front end could
/// see them, no test reached them, and a person reads a menu item by its verb —
/// a screen reader says the whole string. Read out of the running
/// application's accessibility tree by `examples/native_ax_probe.rs`.
///
/// `menu.rs` had already rebuilt that submenu once for this exact reason and
/// fixed only its title, which is why this is a test rather than a comment.
#[test]
fn the_app_menu_names_the_product_and_not_the_crate() {
    // Comments stripped first: the field these items read from documents the
    // default it replaced, and a scanner that read the explanation as the
    // offence would fail the file for saying why it is written that way.
    let source: String = read("src-tauri/src/menu.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !source.contains("PredefinedMenuItem::hide(app, None)"),
        "`hide` is back to Tauri's default label, which interpolates the crate \
         name: the menu bar reads \"Hide stackvo-desktop\""
    );
    assert!(
        !source.contains("PredefinedMenuItem::quit(app, None)"),
        "`quit` is back to Tauri's default label, which interpolates the crate \
         name: the menu bar reads \"Quit stackvo-desktop\""
    );
    assert!(
        source.contains("labels.hide.replace(\"{product}\", product)")
            && source.contains("labels.quit.replace(\"{product}\", product)"),
        "the two items no longer take their text from the label catalogue with \
         the product name substituted"
    );

    // And the catalogue carries them, so a language change reaches them the way
    // it reaches everything else in the menu bar.
    let tray = read("src-tauri/src/tray.rs");
    for key in ["menuHide", "menuQuit"] {
        assert!(
            tray.contains(&format!("\"{key}\"")),
            "`{key}` is not one of the keys the front end is asked for, so the \
             menu item falls back to the built-in table forever"
        );
    }
}

/// The status item is named from the moment it appears.
///
/// The tooltip is the only name this status item carries, and it was only ever
/// set by `refresh` — so between the icon appearing and the first engine check
/// landing there was nothing, and a check that hangs left it that way. This is
/// the third defect of the same family the accessibility probe found: a control
/// nobody could see was unnamed, because nothing on screen looks wrong when it
/// is.
///
/// Measured live, and the first reading was wrong: the tooltip lands in
/// `AXHelp`, not `AXTitle`. An icon-only status item has no `AXTitle` on macOS
/// by construction, so the probe asks for all three name attributes rather than
/// reporting an empty title as an unnamed control.
#[test]
fn the_status_item_is_named_before_the_first_refresh() {
    let source = read("src-tauri/src/tray.rs");
    let builder = source
        .split_once("TrayIconBuilder::with_id(\"main\")")
        .expect("tray.rs builds a status item")
        .1;
    let chain = builder.split(".build(app)").next().unwrap_or(builder);

    assert!(
        chain.contains(".tooltip("),
        "the status item is built with no tooltip, which on macOS is a status \
         item with no accessible name until the first refresh lands — and none \
         at all if it never does"
    );
}

/// The window properties the accessibility statement leans on.
///
/// `docs/accessibility.md` offers an interface scale as the answer to reflow,
/// and a scale is only an answer in a window that can be resized to meet it. A
/// build that shipped a fixed-size window would make that section false without
/// touching a word of it.
#[test]
fn the_main_window_can_be_resized_to_the_scale_the_statement_offers() {
    let windows = declared_windows();
    let main = windows.first().expect("there is a main window");

    assert_eq!(
        main["resizable"].as_bool(),
        Some(true),
        "the main window is not resizable, which makes the interface-scale \
         setting `docs/accessibility.md` offers unusable at the sizes it is for"
    );
    assert!(
        main["minWidth"].as_f64().is_some() && main["minHeight"].as_f64().is_some(),
        "the main window declares no minimum size, so nothing states the \
         smallest layout this application claims to work at"
    );
}

/// The claim in the statement, against the code.
///
/// `docs/accessibility.md` §4 says what is *not* covered. A limitation that has
/// been closed and left in the document is worse than one nobody wrote down:
/// the first is a statement somebody relies on.
#[test]
fn the_accessibility_statement_still_says_the_audit_is_owed() {
    let doc = read("docs/accessibility.md");

    assert!(
        doc.contains("tauri-driver"),
        "the statement no longer names what blocks the native-window audit. It \
         is still blocked — this file checks the names, not the operating."
    );
}
