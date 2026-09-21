# Reporting a docs issue

Three kinds of documentation, and each is a file in the repository that you
can edit yourself.

| Where you read it | Where it lives | How to edit |
| --- | --- | --- |
| This site | `docs/content/en/`, `docs/content/tr/` | The pencil at the top of every page |
| A help card in the app — the **?** on a card | `docs/help/en/`, `docs/help/tr/` | The same pencil, on that card's page under [In-app help](../../help/index.md) |
| The README | `README.md`, `README_TR.md` | On GitHub |

The help cards are the same files the app fetches from `main`, so a corrected
sentence reaches every install on its next run. No release needed.

## Fix it yourself

The pencil at the top of a page opens that file on GitHub. Edit, describe the
change in a sentence, and GitHub opens the pull request for you. For a typo or
a wrong word that is the whole process; it takes a minute and is merged as
soon as it is seen.

Write both languages if you can. The site assumes every page exists in
English and Turkish, and a help topic written in one language and not the
other fails the tests. If you can only write one, say so in the pull request
and the other half will follow.

## Report it

For something bigger than you want to write — a missing page, a section that
is wrong from its first line, a guide that no longer matches the app — open
an [issue](https://github.com/stackvo/stackvo/issues/new/choose) with:

- **Where.** The page's address, or the card's name and the page it is on.
- **What is wrong.** Wrong, unclear, out of date or missing — say which.
- **What you expected to read.** Even a rough sentence helps. The reader who
  was confused is the best person to say what would not have confused them.

Something that is *unclear* rather than wrong is often a question first.
[Discussions](https://github.com/stackvo/stackvo/discussions) is the place
for that, and the answer usually becomes a paragraph here.

## What a page should do

Plain sentences, short ones. Say what a control does, what gets written, what
gets restarted, and whether it survives a rebuild. The help cards'
[writing guide](https://github.com/stackvo/stackvo/blob/main/docs/help/README.md)
is the standard for the whole site.
