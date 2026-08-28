//! Can the app create what it can recognise, and recognise what it creates?
//!
//! Two catalogues had grown next to each other with nothing between them.
//! `scaffold::Template` knows how to fill an empty directory with a framework;
//! `detect::FRAMEWORKS` knows how to look at a filled one and say which
//! framework it is. Measured before this file existed: 28 templates, 16
//! detectable frameworks, and **no test relating them at all** — four templates
//! (Yii, Laminas, TYPO3, PrestaShop) scaffolded a project the app could not
//! then identify, and two frameworks (Magento, Statamic) could be identified
//! and not created.
//!
//! That gap is not cosmetic, because the two halves are the same user story.
//! `project_scaffold` deliberately ends where adoption begins: the installer
//! writes the files, and `project_adopt` reads runtime, server and document
//! root back off what is on disk. So a template whose result detection does not
//! recognise produces a project adopted by the *generic* PHP rule — document
//! root `public`, framework `None` — and Yii serves from `web/`. The button
//! works, the container starts, and the site is a 404 with no error anywhere.
//!
//! ## What this file asserts, and what it cannot
//!
//! It cannot run the installers: `composer create-project` needs a network and
//! a Docker daemon, and 29 of them is a CI job rather than a unit test. What it
//! can do is require the relationship to be **written down** — one table saying
//! what detection makes of each template's output, with every entry either a
//! framework `detect` can actually return or a stated reason it is not one.
//!
//! A declared gap and an undeclared gap are the same gap. The difference is
//! that the declared one was somebody's decision, and shows up in review as a
//! line somebody has to keep or delete.
//!
//! It also binds the four hand-written copies of the template list — the Rust
//! enum, the IPC contract's union type, the drawer's three maps, and both
//! locale files. Any one of them can grow without the others, and the failure
//! is silent in each direction: a template missing from the drawer cannot be
//! chosen, and a template missing from the enum is a `project_scaffold` that
//! refuses with "unknown template" after the user picked it from a list.

use stackvo_desktop_lib::detect;
use stackvo_desktop_lib::scaffold::Template;
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

/// What `detect::infer` says about the directory a template's installer leaves.
enum Outcome {
    /// A name `detect::FRAMEWORKS` carries. The pair is what closes the loop.
    Framework(&'static str),
    /// Detection answers with the runtime and no framework, and that is the
    /// right answer rather than a gap. The string is why, and it is required:
    /// an unexplained `None` here is indistinguishable from a missing rule.
    RuntimeOnly(&'static str),
}

use Outcome::{Framework, RuntimeOnly};

/// Every template, and what the app makes of what it just created.
///
/// The `RuntimeOnly` rows are not oversights and are worth reading as a group.
/// Vue, React and Angular all scaffold through Vite, so detection answers
/// `vite` for the first two — one framework name for three templates is the
/// honest answer, because after `npm create vite` the tree genuinely does not
/// say which template produced it. The six written templates (Gin, Echo, Flask,
/// FastAPI, Sinatra, Rocket) are recognised by their **runtime** marker —
/// `go.mod`, `requirements.txt`, `Gemfile`, `Cargo.toml` — which is all a
/// two-file hello world has to offer and all the generator needs.
const TEMPLATES: [(&str, Outcome); 29] = [
    ("laravel", Framework("laravel")),
    ("wordpress", Framework("wordpress")),
    ("symfony", Framework("symfony")),
    ("cakephp", Framework("cakephp")),
    ("yii", Framework("yii")),
    ("codeigniter", Framework("codeigniter")),
    ("laminas", Framework("laminas")),
    ("slim", Framework("slim")),
    ("drupal", Framework("drupal")),
    ("prestashop", Framework("prestashop")),
    ("statamic", Framework("statamic")),
    ("typo3", Framework("typo3")),
    ("nextjs", Framework("next")),
    ("nuxt", Framework("nuxt")),
    ("svelte", Framework("sveltekit")),
    ("astro", Framework("astro")),
    ("nest", Framework("nestjs")),
    // Vite's own templates. Detection reads `vite` from the dependency list and
    // stops there, which is as far as the tree goes: a Vue and a React project
    // created this way differ only in the contents of `src/`.
    ("vue", Framework("vite")),
    ("react", Framework("vite")),
    (
        "angular",
        RuntimeOnly("the Angular CLI ships no dependency the node rules look for; the project runs as a plain node app on `npm run dev`, which is what its generator produces"),
    ),
    // TinaCMS is a layer over whichever framework hosts it, and
    // `create-tina-app --template basic` produces a Next.js app. So detection
    // answers `next`, which is right about the project and says nothing about
    // the template — recorded as what it is rather than smoothed over, because
    // the alternative is a `tina` detection rule that would have to fire on
    // something no Tina project uniquely has.
    ("tina", Framework("next")),
    (
        "django",
        RuntimeOnly("`manage.py` is Django's marker and the rule is a *runtime* rule — nothing else puts one at the root, so the python runtime is already the specific answer"),
    ),
    (
        "rails",
        RuntimeOnly("`Gemfile` gives the ruby runtime; `bin/rails` is fingerprinted and is what the rails-specific affordances key off, but it is not a framework name"),
    ),
    ("gin", RuntimeOnly("written template: `go.mod`, and a Go module is the answer")),
    ("echo", RuntimeOnly("written template: `go.mod`, and a Go module is the answer")),
    ("flask", RuntimeOnly("written template: `requirements.txt` gives the python runtime")),
    ("fastapi", RuntimeOnly("written template: `requirements.txt` gives the python runtime")),
    ("sinatra", RuntimeOnly("written template: `Gemfile` gives the ruby runtime")),
    ("rocket", RuntimeOnly("written template: `Cargo.toml` gives the rust runtime")),
];

/// Frameworks the app can recognise but cannot create, each with the reason.
///
/// Empty would be the better state and is not the goal — "create every
/// framework we can name" is not a promise worth making. The goal is that the
/// list is short and each line survives being read.
const DETECTABLE_WITHOUT_A_TEMPLATE: [(&str, &str); 2] = [
    (
        "magento",
        "installing it needs an authenticated `repo.magento.com` key pair the \
         user creates in their own Adobe account — a button that always fails \
         until they have been somewhere else. See the `scaffold` module docs.",
    ),
    (
        "remix",
        "Remix merged into React Router v7 and `create-remix` now scaffolds \
         that; the detection rule stays because the checkouts people already \
         have still carry `@remix-run/dev`, but shipping a template would \
         create a project against a name its own maintainers have retired.",
    ),
];

#[test]
fn every_template_says_what_the_app_will_make_of_what_it_created() {
    let declared: Vec<&str> = TEMPLATES.iter().map(|(name, _)| *name).collect();

    for template in Template::ALL {
        assert!(
            declared.contains(&template.as_str()),
            "`{}` scaffolds a project and nothing here says what detection \
             makes of it — an unrecognised result adopts through the generic \
             rule, which gets the document root wrong without erroring",
            template.as_str()
        );
    }
    for name in &declared {
        assert!(
            Template::parse(name).is_some(),
            "`{name}` is declared here and is not a template"
        );
    }

    // The half that catches a rename. `nextjs` maps to the framework name
    // `next`, `nest` to `nestjs`, `svelte` to `sveltekit` — three places where
    // the two catalogues deliberately spell one thing two ways, and each is a
    // string that could be changed on one side alone.
    for (template, outcome) in &TEMPLATES {
        match outcome {
            Framework(framework) => assert!(
                detect::FRAMEWORKS.contains(framework),
                "`{template}` claims detection answers `{framework}`, which is \
                 not a name `infer` can return"
            ),
            // The reason is the whole point of the row: an unexplained "no
            // framework" is indistinguishable from a rule somebody forgot to
            // write, which is the state this file was created to end.
            RuntimeOnly(reason) => assert!(
                reason.len() > 40,
                "`{template}` detects as a runtime with no framework and the \
                 reason given is too short to be one"
            ),
        }
    }
}

#[test]
fn every_detectable_framework_is_creatable_or_declared_as_not() {
    let creatable: Vec<&str> = TEMPLATES
        .iter()
        .filter_map(|(_, outcome)| match outcome {
            Framework(name) => Some(*name),
            RuntimeOnly(_) => None,
        })
        .collect();

    for framework in detect::FRAMEWORKS {
        if creatable.contains(&framework) {
            continue;
        }
        let excused = DETECTABLE_WITHOUT_A_TEMPLATE
            .iter()
            .find(|(name, _)| *name == framework);
        assert!(
            excused.is_some(),
            "the app can recognise `{framework}` and cannot create one, and \
             nothing says why — add a template, or a reason here"
        );
        let (_, reason) = excused.unwrap();
        assert!(
            reason.len() > 40,
            "`{framework}` is excused by a reason too short to be one"
        );
    }

    // The other direction: an excuse left behind after the template arrived.
    for (framework, _) in DETECTABLE_WITHOUT_A_TEMPLATE {
        assert!(
            detect::FRAMEWORKS.contains(&framework),
            "`{framework}` is excused from having a template and is not \
             something this app detects — delete the line"
        );
        assert!(
            !creatable.contains(&framework),
            "`{framework}` has a template now; delete its excuse"
        );
    }
}

/// The four hand-written copies of one list.
///
/// Each is a separate file in a separate language, and each fails differently
/// when it drifts: the drawer cannot offer a template it does not list, the
/// contract's union type is what `ipc.d.ts` is generated from, and a missing
/// locale key renders as the key itself in the one screen a new user sees
/// first.
#[test]
fn every_hand_written_template_list_holds_the_same_templates() {
    let contract = read("contracts/ipc.json");
    let drawer = read("src/components/NewProjectDrawer.vue");
    let en = read("src/i18n/locales/en.js");
    let tr = read("src/i18n/locales/tr.js");

    let value: serde_json::Value = serde_json::from_str(&contract).expect("valid JSON");
    let union = value["commands"]["project_scaffold"]["args"]["template"]
        .as_str()
        .expect("project_scaffold declares a template argument");
    let contract_names: Vec<&str> = union.split('|').map(|s| s.trim_matches('\'')).collect();

    // The drawer holds three maps and the groups; a template absent from any
    // one of them is a different bug, so all four are asked separately.
    //
    // Matched by the spelling each map actually uses rather than by a bare
    // search for the word: `astro` appears in the drawer's prose too, and a
    // whole-file search would have been satisfied by a comment.
    let groups = section(&drawer, "const TEMPLATE_GROUPS = [", "];");
    let runtimes = section(&drawer, "const TEMPLATE_RUNTIME = {", "};");
    let icons = section(&drawer, "const TEMPLATE_ICONS = {", "};");

    for template in Template::ALL {
        let name = template.as_str();
        assert!(
            contract_names.contains(&name),
            "`{name}` is not in the `project_scaffold` union in contracts/ipc.json"
        );
        assert!(
            groups.contains(&format!("'{name}'")),
            "`{name}` is in no TEMPLATE_GROUPS group, so nobody can pick it"
        );
        assert!(
            runtimes.contains(&format!("{name}:")),
            "`{name}` has no entry in TEMPLATE_RUNTIME, so the form cannot \
             decide which fields to ask for"
        );
        assert!(
            icons.contains(&format!("{name}:")),
            "`{name}` has no entry in TEMPLATE_ICONS"
        );
        for (locale, text) in [("en", &en), ("tr", &tr)] {
            assert!(
                text.contains(&format!("      {name}: '")),
                "`{name}` has no label in {locale}.js — it would render as the \
                 translation key in the new-project drawer"
            );
        }
    }

    // And nothing in the contract that the enum does not carry: a union member
    // with no variant is a value `Template::parse` refuses after the user has
    // already chosen it.
    for name in contract_names {
        assert!(
            Template::parse(name).is_some(),
            "contracts/ipc.json offers `{name}` and `Template::parse` refuses it"
        );
    }
}

/// The slice of `text` between the first `open` and the next `close` after it.
///
/// Panics rather than returning empty: a section that has been renamed would
/// otherwise make every assertion above it pass for free, which is the failure
/// mode a whole-file search has and this exists to avoid.
fn section(text: &str, open: &str, close: &str) -> String {
    let start = text
        .find(open)
        .unwrap_or_else(|| panic!("NewProjectDrawer.vue no longer has `{open}`"));
    let rest = &text[start + open.len()..];
    let end = rest
        .find(close)
        .unwrap_or_else(|| panic!("`{open}` is never closed by `{close}`"));
    rest[..end].to_string()
}
