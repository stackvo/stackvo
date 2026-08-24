//! What a health probe says about endpoints that really are up, down and wrong.
//!
//! The unit tests hold the shapes; this holds the thing that cannot be faked —
//! that a probe of a live port comes back quickly and truthfully, and that a
//! status which is not the expected one reads as a failure rather than as a
//! success with a different number in it.
//!
//! Needs the fixture daemon — see `tests/fixtures/supervisord/README.md`.
//!
//!   cargo run --example check_probe
use stackvo_desktop_lib::supervisor::{probe, Check};

fn check(kind: &str, target: &str, expect: Option<u16>) -> Check {
    Check {
        project: "probe".into(),
        process: "web:app".into(),
        kind: kind.into(),
        target: target.into(),
        expect_status: expect,
        timeout_ms: Some(3000),
    }
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");

    for (label, check) in [
        (
            "tcp, something listening",
            check("tcp", "127.0.0.1:9001", None),
        ),
        ("tcp, nothing listening", check("tcp", "127.0.0.1:1", None)),
        (
            "tcp, a host that is not there",
            check("tcp", "10.255.255.1:9", None),
        ),
        // supervisord answers 401 without credentials, which is a service that
        // is working — so it passes only when that is what was asked for.
        (
            "http, 401 expected",
            check("http", "http://127.0.0.1:9001/RPC2", Some(401)),
        ),
        (
            "http, 200 expected",
            check("http", "http://127.0.0.1:9001/RPC2", Some(200)),
        ),
        (
            "http, nothing listening",
            check("http", "http://127.0.0.1:1/", Some(200)),
        ),
    ] {
        let result = runtime.block_on(probe(&check));
        println!(
            "{label:<32} {}  {:<44} {}ms",
            if result.ok { "PASS" } else { "FAIL" },
            result.detail,
            result.ms
        );
    }
}
