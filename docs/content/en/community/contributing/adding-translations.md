# Adding translations

StackVo speaks English and Turkish, in three places at once. A translation
touches all three, or only the one you are improving — both are welcome.

| What | Where | Checked by |
| --- | --- | --- |
| The app's interface | `src/i18n/locales/en.js`, `tr.js` | `tests/i18n.spec.js` — the same keys in every locale, every key in use defined |
| The help cards — the **?** on a card | `docs/help/en/`, `docs/help/tr/` | `tests/help-topics.spec.js` — every topic in both languages |
| This site | `docs/content/en/`, `docs/content/tr/` | the strict build — every page listed in both configs |

## Improving a translation

The most useful kind, and the quickest.

1. **Interface text.** Find the string in `src/i18n/locales/tr.js` (or
   `en.js`); the keys are grouped by view, so the settings page's strings sit
   in the `settings` block. Change the value, never the key.
   `npm run test:js` confirms the two files still agree.
2. **A help card.** The pencil on that card's page under
   [In-app help](../../help/index.md) opens the file. Keep the `# heading`
   on the first line; the app refuses a document that does not start with
   one.
3. **A site page.** The pencil at the top of the page. The two configs list
   the same pages, so the page keeps its place.

Words that stay as they are in both languages: *StackVo*, *Docker*,
*snapshot*, *worktree*, *tunnel*, product names and command names. A
sentence that reads naturally beats one that is word for word.

## Adding a language

There is no third language yet, and adding one is a code change rather than a
file drop — say so in an
[issue](https://github.com/stackvo/stackvo/issues/new/choose) first, so the
pieces can be planned together. The pieces:

1. **The interface.** A new `src/i18n/locales/<code>.js` with every key
   `en.js` has, registered in `src/i18n/index.js` beside Vuetify's locale for
   that language, and added to the codes the app accepts.
   `tests/i18n.spec.js` refuses a missing key.
2. **The help cards.** `docs/help/<code>/` with one document per topic — more
   than a hundred. Until they are all there the app shows the English
   document for that topic: `en` is the fallback, so a half-translated
   language is usable from the first pull request.
3. **This site.** A `docs/mkdocs.<code>.yml` that inherits `mkdocs.yml` the
   way `mkdocs.tr.yml` does, a `docs/content/<code>/` tree, a line in
   `extra.alternate` so the switcher offers it, and a build line in the
   publishing workflow.

Start with the interface. It is what a person sees first, and the fallback
carries the rest until it is written.

## Style

Short sentences. The reader has a problem and is not here for the prose. Say
what a button does and what it writes. The help cards'
[writing guide](https://github.com/stackvo/stackvo/blob/main/docs/help/README.md)
is the standard.
