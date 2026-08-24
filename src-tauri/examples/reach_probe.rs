//! What `supervisor_project`'s reachability check says about a real container.
//!
//! Three containers, three answers, and the three are the whole point: they
//! look identical on screen — an empty table — and send somebody to three
//! different places.
//!
//! The containers it looks at, and how to make them — see
//! `tests/fixtures/supervisord/README.md` for the image:
//!
//! ```text
//! docker run -d --name svproj-new  -v $PWD/new.conf:/etc/supervisor/conf.d/supervisord.conf:ro stackvo-supd-test supervisord -c /etc/supervisor/conf.d/supervisord.conf
//! docker run -d --name svproj-old  -v $PWD/old.conf:/etc/supervisor/conf.d/supervisord.conf:ro stackvo-supd-test supervisord -c /etc/supervisor/conf.d/supervisord.conf
//! docker run -d --name svproj-bare alpine sleep 3600
//! ```
//!
//! `new.conf` is what `generator::render_supervisord_conf` writes today;
//! `old.conf` is the same with `[unix_http_server]`, `[rpcinterface:supervisor]`
//! and `[supervisorctl]` taken out, which is what every StackVo image had
//! before those three were added. `svproj-gone` is deliberately never created.
//!
//!   cargo run --example reach_probe
use stackvo_desktop_lib::supervisor::{classify, Target};

fn target(container: &str) -> Target {
    Target {
        project: container.to_string(),
        container: container.to_string(),
    }
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("a runtime");
    for (label, container) in [
        ("generated config, as it is now", "svproj-new"),
        ("generated config, as it was", "svproj-old"),
        ("no supervisord in the image", "svproj-bare"),
        ("not running", "svproj-gone"),
    ] {
        let probe = runtime
            .block_on(target(container).exec(&["supervisorctl".to_string(), "status".to_string()]));
        let reach = match &probe {
            Ok(out) => classify(&format!("{}{}", out.stdout, out.stderr)),
            Err(_) => stackvo_desktop_lib::supervisor::Reach::Stopped,
        };
        let said = probe
            .as_ref()
            .map(|o| {
                format!("{}{}", o.stdout, o.stderr)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .unwrap_or_else(|e| e.message.clone());
        println!("{label:<34} {reach:?}");
        println!("{:<34} └ {said}", "");
    }
}
