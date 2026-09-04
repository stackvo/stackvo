//! One frame of `stackvo tui`, as the bytes it would write to a terminal.
//!
//!   cargo run --example tui_frame          # 80 columns
//!   cargo run --example tui_frame -- 100
//!
//! `tools/screenshots.mjs` runs this for `docs/screenshots/tui.png`. The
//! screen is a terminal program, so there is no window for the tool to shoot;
//! what there is, is `tui::draw`, which builds the frame as a string precisely
//! so that something other than a terminal can read it. This prints that
//! string — escapes and all — for [`stackvo_desktop_lib::tui::sample`], the
//! staged stack every other picture shows, and the tool draws the cells.
//!
//! No pty, no raw mode, no Docker: `tui_probe.rs` is the one that takes a
//! real terminal, and it exists to check the terminal is given back. This one
//! never takes it.

fn main() {
    // The same floor `tui::width` keeps: below it the header cannot hold both
    // its halves, and the picture would be of a layout the screen refuses.
    let width = std::env::args()
        .nth(1)
        .and_then(|w| w.parse().ok())
        .filter(|w: &usize| *w > 20)
        .unwrap_or(80);

    let frame = stackvo_desktop_lib::tui::draw(
        &stackvo_desktop_lib::tui::sample(),
        &stackvo_desktop_lib::cli::Style::always(),
        width,
    );
    print!("{frame}");
}
