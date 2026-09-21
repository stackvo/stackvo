# The documentation site

`https://stackvo.github.io/stackvo/` — built with [Material for MkDocs] from
this directory and published by `.github/workflows/docs-publish.yml`.

[Material for MkDocs]: https://squidfunk.github.io/mkdocs-material/

## What is where

    docs/
      help/            the application's help documents — see help/README.md
      screenshots/     the README's screenshots, shared with the site
      screenshots/web/ the same at 1600 px as WebP, what the pages show
      assets/          logo and stylesheets, shared by both languages
      content/en/      the English pages
      content/tr/      the Turkish pages
      overrides/       the landing page, the footer, the head tags and the announcement bar
      hooks/stackvo.py mounts help/, assets/, screenshots/ and the root documents into a build
      tools/           screenshots-web.mjs, which writes screenshots/web/
      mkdocs.yml       the English site
      mkdocs.tr.yml    the Turkish site: inherits mkdocs.yml, changes labels
      requirements.txt pinned MkDocs, theme and plugins

Two builds make one site: `mkdocs.yml` renders into `site/`, `mkdocs.tr.yml`
into `site/tr/`, and the language switcher in the header links the two.
There is no i18n plugin — the theme's own `extra.alternate` and two configs
are the whole mechanism, which is one fewer thing to carry when the theme's
successor arrives.

## The help documents are not copied

`docs/help/<locale>/` is the application's. The running app pulls those files
from `main`, and `tests/help-topics.spec.js` fails on any file in there that
no card opens — so the site cannot put an index page beside them, and copying
them under `content/` would make `mkdocs serve` rebuild itself in a loop.
Instead `hooks/stackvo.py` mounts them into each build as virtual files under
`help/`, titles each from its own `# heading`, and lists them on
`help/index.md` grouped by topic prefix — on the index page rather than in
the navigation, where a hundred entries were most of every page's sidebar
(`not_in_nav` keeps `--strict` quiet about it). A new card's document
appears on the site with nobody editing a config.

The same hook mounts `assets/` and `screenshots/` into both languages, so the
eleven megabytes of screenshots are checked in once. It also mounts the
documents at the repository root — `CHANGELOG.md` under Insiders;
`SECURITY.md`, `PRIVACY.md`, `CODE_OF_CONDUCT.md`, `ARCHITECTURE.md`,
`ACCESSIBILITY.md` and `LICENSE` under Reference › Project documents — in
both languages, with their relative links pointed at GitHub and a note under
the heading in the Turkish build (`extra.mounted.note`). They are one file
each, written in English; the site reads them rather than carrying copies.
The changelog is kept out of the search index and every release but the
newest is folded away.

## What the hook fixes on the way out

- **A description per page.** The first paragraph, when the front matter
  sets none, so a shared link shows what the page is about. `overrides/
  main.html` turns it into Open Graph and Twitter tags; `extra.social_image`
  is the picture.
- **The language switcher goes to the same page.** The theme renders
  `extra.alternate` as fixed addresses; the hook appends the page's path to
  the switcher's links and the `hreflang` tags.
- **The announcement bar** names the latest release these pages are ahead
  of, from the same lookup the download links use (`extra.announce`).

## Screenshots on pages

Pages show `screenshots/web/<name>.webp`, 1600 px wide, written by
`node docs/tools/screenshots-web.mjs` from the README's 3200 px PNGs. Re-run
it after `npm run screenshots` has reshot the originals. Inside a page:

    <figure markdown>
    ![What it shows](../screenshots/web/projects.webp){ loading=lazy }
    <figcaption>One sentence.</figcaption>
    </figure>

## Naming what the window shows

A path in the window — **Settings → Updates** — is always bold with `→`
between the segments. `tests/docs-labels.spec.js` reads every one out of
`docs/content/<locale>` and checks each segment against the locale file and
the help cards, so a name the window does not show fails `npm run test:js`.

## Running it

With Docker and nothing else installed — the theme's own image carries MkDocs:

    docker run --rm -p 8000:8000 -v "$PWD":/docs squidfunk/mkdocs-material:9.7.7 \
      serve -f docs/mkdocs.yml -a 0.0.0.0:8000

Then open <http://127.0.0.1:8000/stackvo/> — the dev server mounts at the
same sub-path GitHub Pages will use, so the language switcher and the 404 page
behave as they will in production. The Turkish site is the same command with
`-f docs/mkdocs.tr.yml -p 8001:8000`.

With Python:

    python3 -m venv .venv && . .venv/bin/activate
    pip install -r docs/requirements.txt
    npm run docs:serve          # English, http://127.0.0.1:8000/stackvo/
    npm run docs:serve:tr       # Turkish, http://127.0.0.1:8001/stackvo/tr/
    npm run docs:build          # both, strict — what CI runs

`--strict` fails the build on a page that is written and not in `nav`, and on
a link to a page that does not exist. `tests/docs-labels.spec.js` is the
other gate, for what the pages call things.

"Last updated" under each page comes from the file's last commit
(`git-revision-date-localized`); CI checks out the full history for it.

One thing the dev server does not pick up: a change to `hooks/stackvo.py`.
MkDocs imports a hook once, at start, and rebuilds with that module however
many times the file changes afterwards. After editing the hook, stop the
server and start it again.

## The landing page

`content/<locale>/index.md` is not an article: its front matter says
`template: home.html` and `overrides/home.html` renders it as a landing page
— a hero with the dashboard screenshot, feature cards, a three-step walk
through, and a list of what comes after — the way Material's own site does.
Every word on it comes from that front matter (`hero`, `features`, `steps`,
`more`), so the two languages share one template and the copy lives beside
the other copy. The cards are `div`s, not a list: Material's typeset rules
give every `ul` a marker and `flow-root` at a specificity a class does not
beat. `assets/home.css` is linked only there.

## The download links

The landing page's Download section names the latest release and links each
installer directly. The tag comes from `hooks/stackvo.py` at build time — it
asks the releases API once, with `GITHUB_TOKEN` when there is one (CI passes
its own), because the anonymous limit is sixty requests an hour per address
and a page that asked at every visit would hit it from any office. The URLs
are then written from the file-name patterns in the front matter
(`StackVo_{v}_x64.dmg`, `StackVo-{v}-1.x86_64.rpm` …), which is what
tauri-action names them. Nothing asks the API again from the browser: an
earlier version did, to move the links to a release published since the
last build, and CodeQL's `js/xss-through-dom` flagged the fetch()-derived
tag reaching an `href` — correctly, since a regex check is not a sanitizer
its analysis recognises. The gap a rebuild leaves is closed by the site
rebuilding on every push to `docs/**` and, once the commented-out trigger
in `docs-publish.yml` is turned on, on every published release.

    STACKVO_RELEASE_TAG=v0.2.0 npm run docs:build   # pin it: offline, reproducible

With neither a tag nor a network the build still succeeds, warns, and links
the releases page instead. A new application release does not rebuild the
site by itself until the `release: published` trigger in the workflow is
enabled; until then, run the workflow by hand after a release.

## Writing a page

Plain Markdown under `content/<locale>/`, listed in the matching config's
`nav`. Write both languages — the switcher assumes every page exists in both.
Screenshots are referenced from the mounted directory, so from
`getting-started/first-run.md` the path is `../screenshots/project-new.png`.

## If the site ever moves to its own repository

Everything it needs is under `docs/` except `docs/help/`, which stays with
the application. Split the directory with its history
(`git subtree split --prefix=docs`), delete `help/` in the new repository,
check out `stackvo/stackvo` sparsely with only `docs/help`, and point
`STACKVO_HELP_DIR` at it. The hook does not change.
