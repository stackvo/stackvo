#[test]
fn probe_preview() {
    let root = std::env::temp_dir().join(format!("stackvo-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("services")).unwrap();
    std::fs::write(
        root.join(".env"),
        "SERVICE_MYSQL_ENABLE=true\nSERVICE_MYSQL_VERSION=8.0\n",
    )
    .unwrap();
    let env = stackvo_desktop_lib::config::Env::load(&root).unwrap();
    println!("MYSQL_ENABLE={:?}", env.get("SERVICE_MYSQL_ENABLE"));
    let tree = stackvo_desktop_lib::market::catalogue(&root).unwrap();
    println!(
        "pending={}",
        stackvo_desktop_lib::handover::is_pending(&root, &env, &tree)
    );
    let p = stackvo_desktop_lib::handover::preview(&root).unwrap();
    println!(
        "migrated={} pending={} blockers={:?}",
        p.migrated, p.pending, p.blockers
    );
    let _ = std::fs::remove_dir_all(&root);
}
