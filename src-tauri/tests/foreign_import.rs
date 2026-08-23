//! Reading a rival's installation off a real disk, and copying a site out of it.
//!
//! The pure tests in `imports.rs` settle what a site is and what a vhost says.
//! What only exists once there are directories is the rest of it: that an
//! XAMPP tree is recognised by its `htdocs` and a Laragon one by its `www`,
//! that the tool's own dashboard is not offered as a project, that a Laragon
//! vhost reaches the site it belongs to, and — the one that matters — that a
//! copy is a copy and the other installation is left exactly as it was.

use stackvo_desktop_lib::imports::{self, Source};

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("stackvo-foreign-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("creating the scratch directory");
    dir
}

fn write(path: &std::path::Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A XAMPP installation with two real sites and the dashboard it ships.
fn xampp(root: &std::path::Path) {
    let htdocs = root.join("htdocs");

    // A Laravel application: `artisan` is a marker only Laravel has.
    write(&htdocs.join("shop/artisan"), "#!/usr/bin/env php\n");
    write(
        &htdocs.join("shop/composer.json"),
        r#"{"require":{"php":"^8.2","laravel/framework":"^11.0"}}"#,
    );
    write(&htdocs.join("shop/public/index.php"), "<?php\n");

    // Plain PHP, served from the root.
    write(&htdocs.join("legacy/index.php"), "<?php\n");

    // XAMPP's own, which must not be offered.
    write(&htdocs.join("dashboard/index.html"), "<html>\n");
    write(&htdocs.join("webalizer/index.html"), "<html>\n");
    write(&htdocs.join("applications.html"), "<html>\n");
}

#[test]
fn an_xampp_tree_yields_its_sites_and_not_its_dashboard() {
    let root = scratch("xampp");
    xampp(&root);

    let install = imports::scan_at(Source::Xampp, &root, None).expect("an XAMPP installation");
    let names: Vec<&str> = install.sites.iter().map(|s| s.name.as_str()).collect();

    assert_eq!(
        names,
        ["legacy", "shop"],
        "sorted, and the tool's own excluded"
    );

    let shop = install.sites.iter().find(|s| s.name == "shop").unwrap();
    // The same inference an ordinary adoption makes — an imported project is
    // not a second class of project.
    assert_eq!(shop.detected.framework, Some("laravel"));
    assert_eq!(shop.detected.document_root.as_deref(), Some("public"));
    assert_eq!(shop.detected.php_version.as_deref(), Some("8.2"));
    // XAMPP serves by path and has no hostname to read, so adoption asks.
    assert!(shop.domain.is_none());
    assert!(shop.bytes > 0 && !shop.partial);

    // Legacy PHP is served from the directory itself, not from `public/` —
    // getting this wrong builds, starts and serves a 404 with no error.
    let legacy = install.sites.iter().find(|s| s.name == "legacy").unwrap();
    assert_eq!(legacy.detected.document_root.as_deref(), Some("."));

    // A directory that is not an installation is not one.
    assert!(imports::scan_at(Source::Laragon, &root, None).is_none());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_laragon_site_carries_the_hostname_its_vhost_declares() {
    let root = scratch("laragon");

    write(&root.join("www/blog/index.php"), "<?php\n");
    write(&root.join("www/api/index.php"), "<?php\n");
    write(
        &root.join("etc/apache2/sites-enabled/auto.blog.test.conf"),
        "<VirtualHost *:80>\n  DocumentRoot \"C:/laragon/www/blog\"\n  ServerName blog.test\n</VirtualHost>\n",
    );

    let install = imports::scan_at(Source::Laragon, &root, None).expect("a Laragon installation");

    let blog = install.sites.iter().find(|s| s.name == "blog").unwrap();
    assert_eq!(blog.domain.as_deref(), Some("blog.test"));

    // A site with no vhost still imports; it simply has no name to carry, and
    // adoption falls back to the suffix as it does for every other project.
    let api = install.sites.iter().find(|s| s.name == "api").unwrap();
    assert!(api.domain.is_none());

    let _ = std::fs::remove_dir_all(&root);
}

/// A name already under `projects/` is reported before anything is clicked.
#[test]
fn a_name_that_is_taken_is_said_so_in_the_listing() {
    let root = scratch("taken");
    xampp(&root);

    let projects = root.join("workspace-projects");
    std::fs::create_dir_all(projects.join("shop")).unwrap();

    let install = imports::scan_at(Source::Xampp, &root, Some(&projects)).unwrap();
    assert!(
        install
            .sites
            .iter()
            .find(|s| s.name == "shop")
            .unwrap()
            .taken
    );
    assert!(
        !install
            .sites
            .iter()
            .find(|s| s.name == "legacy")
            .unwrap()
            .taken
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The promise the module opens with: nothing is written to the other
/// installation, and a copy is a copy.
#[test]
fn copying_a_site_leaves_the_original_untouched() {
    let root = scratch("copy");
    xampp(&root);

    let source = root.join("htdocs/shop");
    let target = root.join("projects/shop");

    imports::copy_tree(&source, &target).expect("copying the tree");

    for relative in ["artisan", "composer.json", "public/index.php"] {
        assert!(
            target.join(relative).is_file(),
            "{relative} did not come over"
        );
        assert!(
            source.join(relative).is_file(),
            "{relative} left the source"
        );
    }

    // Byte for byte, not "a file of that name exists".
    assert_eq!(
        std::fs::read(source.join("composer.json")).unwrap(),
        std::fs::read(target.join("composer.json")).unwrap()
    );

    // And the copy is a project the ordinary detection recognises, which is
    // what makes the adoption that follows the same adoption as any other.
    assert_eq!(
        stackvo_desktop_lib::detect::detect(&target).framework,
        Some("laravel")
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A symlink is not followed. One pointing at `/` turns a copy of a site into a
/// copy of the disk, and one pointing back into the tree is a loop.
#[test]
fn a_symlink_is_not_walked_into() {
    let root = scratch("symlink");
    write(&root.join("htdocs/site/index.php"), "<?php\n");
    std::fs::create_dir_all(root.join("elsewhere/secret")).unwrap();
    write(&root.join("elsewhere/secret/keys.txt"), "hunter2\n");

    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("htdocs/site/link")).unwrap();

    let target = root.join("projects/site");
    imports::copy_tree(&root.join("htdocs/site"), &target).expect("copying");

    assert!(target.join("index.php").is_file());
    assert!(
        !target.join("link").exists(),
        "a symlink was followed out of the site being copied"
    );

    let _ = std::fs::remove_dir_all(&root);
}

// --------------------------------------------------------------- the gate

/// Every source the backend reads has a button, and every button reaches a
/// source the backend reads.
///
/// The drift is silent in both directions and that is the whole reason this is
/// a test rather than a comment: an id in the view the backend refuses is a
/// button that errors when clicked, and a source the backend can read with no
/// id in the view is a tool nobody can point at. Three of the five sources sat
/// in the second state once already.
#[test]
fn the_views_source_list_and_the_backends_agree() {
    let view = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../src/views/Projects.vue"
    ))
    .expect("the projects view");

    let line = view
        .lines()
        .find(|line| line.contains("const IMPORT_SOURCES"))
        .expect("IMPORT_SOURCES in the view");
    let inside = line
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .expect("an array literal")
        .0;

    let mut from_view: Vec<String> = inside
        .split(',')
        .map(|item| item.trim().trim_matches('\'').trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect();
    let mut from_backend: Vec<String> = imports::ALL
        .iter()
        .map(|source| source.as_str().to_string())
        .collect();

    from_view.sort();
    from_backend.sort();
    assert_eq!(
        from_view, from_backend,
        "Projects.vue's IMPORT_SOURCES and imports::ALL have come apart"
    );

    // A button is only worth anything if the command layer accepts its id.
    for id in &from_view {
        assert!(
            Source::from_id(id).is_some(),
            "the view offers `{id}` and from_id refuses it"
        );
    }
}

/// Herd is Valet's shape with a different root, and the point of the test is
/// that a real tree goes through the reader that already existed — and comes
/// out carrying the one thing Valet cannot say.
#[test]
fn a_herd_tree_is_read_by_valets_reader_and_keeps_its_pinned_version() {
    let root = scratch("herd");
    let config = root.join("config");
    let sites = config.join("Sites");
    let code = root.join("code");

    write(&code.join("shop/artisan"), "#!/usr/bin/env php\n");
    write(
        &code.join("shop/composer.json"),
        r#"{"require":{"php":"^8.1","laravel/framework":"^11.0"}}"#,
    );
    std::fs::create_dir_all(code.join("shop/public")).unwrap();
    std::fs::create_dir_all(&sites).unwrap();
    write(&config.join("config.json"), r#"{"tld":"test","paths":[]}"#);
    #[cfg(unix)]
    std::os::unix::fs::symlink(code.join("shop"), sites.join("shop")).unwrap();
    write(
        &config.join("Nginx/shop.test.conf"),
        "server {\n  location ~ \\.php$ {\n    fastcgi_pass unix:/x/herd-83.sock;\n  }\n}\n",
    );

    let install = imports::scan_at(Source::Herd, &config, None).expect("a Herd installation");
    assert_eq!(install.source, Source::Herd);
    assert_eq!(install.sites.len(), 1, "{:?}", install.sites);

    let site = &install.sites[0];
    assert_eq!(site.name, "shop");
    assert_eq!(site.domain.as_deref(), Some("shop.test"));
    assert_eq!(site.detected.framework, Some("laravel"));
    // `^8.1` is what the framework needs. `8.3` is what Herd was serving it
    // with, and only Herd writes that down.
    assert_eq!(site.detected.php_version.as_deref(), Some("8.3"));

    let _ = std::fs::remove_dir_all(&root);
}

/// The one source that declares instead of implying, read off a real tree:
/// the registry finds the project, and the project's own file settles the
/// version, the server, the document root and the database.
#[test]
fn a_ddev_registry_finds_a_project_and_its_file_settles_the_rest() {
    let root = scratch("ddev");
    let home = root.join("ddev-home");
    let project = root.join("work/shop");

    write(&project.join("artisan"), "#!/usr/bin/env php\n");
    write(
        &project.join("composer.json"),
        r#"{"require":{"php":"^8.1","laravel/framework":"^11.0"}}"#,
    );
    std::fs::create_dir_all(project.join("public")).unwrap();
    write(
        &project.join(".ddev/config.yaml"),
        "name: shop\ntype: laravel\ndocroot: public\nphp_version: \"8.4\"\n\
         webserver_type: apache-fpm\ndatabase:\n  type: mysql\n  version: \"8.0\"\n\n\
         # php_version: \"7.4\"  # PHP version to use\n",
    );
    write(
        &home.join("global_config.yaml"),
        &format!(
            "project_tld: ddev.site\nproject_info:\n  shop:\n    approot: {}\n",
            project.display()
        ),
    );

    let install = imports::scan_at(Source::Ddev, &home, None).expect("a DDEV registry");
    assert_eq!(install.sites.len(), 1, "{:?}", install.sites);

    let site = &install.sites[0];
    assert_eq!(site.name, "shop");
    assert_eq!(site.domain.as_deref(), Some("shop.ddev.site"));
    // The declaration wins over the constraint, and over detection's default
    // server — and the commented example at the bottom of the file loses.
    assert_eq!(site.detected.php_version.as_deref(), Some("8.4"));
    assert_eq!(site.detected.server, "apache");
    assert_eq!(site.detected.document_root.as_deref(), Some("public"));
    // Detection still owns what it reads from the code.
    assert_eq!(site.detected.framework, Some("laravel"));
    // And the database is the same question the catalogue answers.
    assert_eq!(site.services, vec!["mysql".to_string()]);

    let _ = std::fs::remove_dir_all(&root);
}
