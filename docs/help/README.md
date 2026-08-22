# Card help

Every card on the project-detail and settings pages — and every page itself —
carries a help button in the corner opposite its name. Each button names a
**topic**: a stable slug that resolves to one file in this directory.

    help="project-tunnel"   →   docs/help/<locale>/project-tunnel.md

A card that draws its own heading — the dashboard's tiles — carries
`<HelpButton topic="…">` itself instead of passing `help` to a wrapper. Same
topic, same registry; the prop is called `topic` because that is the button's
own name for it.

The slug is written at the call site (`<PaneHeader help="…">`,
`<SettingsGroup help="…">`, `<CollapsiblePane help="…">`, `<PageLayout help="…">`)
and listed in [`src/lib/help.js`](../../src/lib/help.js). It is not derived from
the card's title on purpose: a title is a translated string, so deriving the
filename from it would rename the document whenever somebody rewords a heading,
and rename it differently in each locale.

## How a document reaches the screen

The help button writes its topic into a shared ref (`src/composables/useHelp.js`).
`HelpSheet.vue` — mounted once in `App.vue` — watches that ref, asks the backend
for the document (`help_doc` in `contracts/ipc.json`, implemented in
`src-tauri/src/help.rs`), renders it with `src/lib/markdown.js` and shows it in a
side sheet beside the card rather than a dialog over it: half of what these
documents say is "this button writes X and restarts Y", and reading that with
the button hidden behind the panel explaining it is the wrong way round.

The file is read off disk on **every** open — that is the point of keeping the
documents as files. Correct a sentence here and the next click shows the
correction, with no rebuild. `bundle.resources` in `tauri.conf.json` carries this
directory into the installed application.

## How a document reaches the reader

The copy in this directory is the one that ships, and it is also the one that is
served: the app pulls `docs/help/<locale>/<topic>.md` from `main` over HTTPS the
first time a topic is opened in a run, caches it, and falls back to the cache
and then to the bundled copy when the network says no.

**So a correction pushed to `main` reaches everybody on their next run, whatever
build they are on.** That is the reason it works this way. What it costs is one
request per topic per run, naming the topic — written down in `PRIVACY.md`
rather than left to be discovered.

A body that does not open with `# ` is refused rather than cached, which is what
stops a captive portal's login page from replacing a good document on the one
network where the real one cannot be fetched.

## Languages

Documents are written per locale, under `en/` and `tr/`, because the viewer
shows one document and a reader who set the interface to Turkish should not get
a page that opens in English and switches halfway down. `en` is the fallback: a
locale nobody has written for reads the English document rather than an empty
pane.

`tests/help-topics.spec.js` reads every `help="…"` in the sources and fails on

- a card that draws without one,
- a slug the registry does not know,
- a registered topic no card names,
- a document filed under a name no card opens,
- a topic written in one language and not the other,
- a document that does not open with an `#` heading.

So the map cannot drift from the screen in either direction, and a document
cannot quietly exist in one language only.

## Naming

    project-<pane>      a card on the project-detail page
    settings-<pane>     a card on the settings page
    page-<route>        a page's own help, from its top bar

A pane holding more than one card qualifies each with what that card is about —
`project-indicator-composition`, `settings-domain-hosts`.

## What a document should say

What the card is for, and what each control in it does — including what happens
after you press it: what gets written, what gets restarted, and whether the
change survives a rebuild. The card's description line is the one-sentence
version; this is where the rest goes.

Documents are being written topic by topic. A topic with no file yet is not a
bug in the map — it is a document that has not been written.

**Every topic is written**, in both languages: all 93 of them, covering the
project-detail page and its 28 cards, the settings page and its 37 cards, the
dashboard and its 11 cards, the new-project panel, and the projects, catalogue,
logs, dumps and mail pages.

A new card means a new topic here. `tests/help-topics.spec.js` fails on a card
whose topic the registry does not know, and on a topic written in one language
and not the other, so neither half can drift from the other.

## Shape of a document

    # <the card's name>

    <One or two sentences: what it is, and what it is not.>

    ## Controls
    <Every button and field, and what pressing it actually does — what gets
    written, what gets restarted, whether it survives a rebuild.>

    ## Worth knowing
    <The things somebody finds out the hard way: what it needs running, what
    silently does nothing, what costs money or exposes something.>

## How to write one

Plain sentences. Short ones. No flourishes — somebody is reading this because
something did not work, and they are not here for the prose.

- Say what the card is in one or two sentences, then get to the controls.
- A table for anything that is a list of fields or a list of buttons. Two
  columns: the name, and what it does.
- Say what pressing a button actually does: what gets written, what gets
  restarted, whether it survives a rebuild.
- End with **Worth knowing** — the things people find out the hard way. What
  has to be running. What silently does nothing. What costs money or exposes
  something.
- Where two things are easy to confuse (rebuild and restart; this network and
  Share; the two terminals), a small comparison table settles it faster than a
  paragraph.
