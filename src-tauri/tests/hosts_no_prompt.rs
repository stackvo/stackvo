//! A test can never raise a password dialog.
//!
//! `hosts::apply` falls back to an elevated copy when it cannot write the file,
//! and on macOS that is `osascript`'s administrator prompt — a window behind
//! every other window, with nobody to answer it. It cost a hung `cargo test`
//! sitting at 0% CPU once, which is the failure exactly: a hanging suite
//! looks like a slow one.
//!
//! The comment above `write_in_place` claimed the unelevated branch "is the one
//! that runs, on every platform, without a prompt nobody could answer in CI".
//! It was a claim and nothing held it. This holds it.
//!
//! **Its own binary, and that is not tidiness.** The seam is an environment
//! variable, so every test that sets it must have the process to itself —
//! `hosts_roundtrip.rs` says the same thing at the top and is one test for the
//! same reason. Adding this one beside it made both fail, which is the
//! cheapest possible demonstration of why.

#[cfg(unix)]
mod unix {
    use stackvo_desktop_lib::hosts;
    use std::os::unix::fs::PermissionsExt;

    /// The file is made read-only rather than the code stubbed, so what runs is
    /// the real fallback on a real failure to write.
    #[test]
    fn a_file_the_seam_points_at_is_never_worth_a_password() {
        let path =
            std::env::temp_dir().join(format!("stackvo-hosts-noprompt-{}", std::process::id()));
        std::fs::write(&path, "127.0.0.1\tlocalhost\n").expect("the fixture is writable");
        // SAFETY: this binary runs one test and sets this before touching `hosts`.
        unsafe { std::env::set_var("STACKVO_HOSTS_PATH", &path) };

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444))
            .expect("making the fixture read-only");

        let outcome = hosts::apply(&["shop.loc".into()], &[]);

        // Restored before the assertion, so a failure does not leave an
        // unwritable file behind for the next run to trip over.
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));

        let error = outcome.expect_err("an unwritable hosts file must not succeed");
        assert!(
            error.message.contains("STACKVO_HOSTS_PATH"),
            "the refusal must name the seam it is refusing for: {}",
            error.message
        );
        assert!(
            error.message.contains("refusing to ask for a password"),
            "{}",
            error.message
        );

        let _ = std::fs::remove_file(&path);
    }
}
