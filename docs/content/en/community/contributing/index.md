# Contributing

StackVo is free, MIT-licensed and maintained by one person. Every bug report
that can be reproduced, every sentence made clearer and every pull request
that arrives with its tests is time given back to the project. This section
says how to do each of those so that it lands the first time.

## How you can contribute

### Creating an issue

<div class="grid cards" markdown>

- :material-bug-outline: **[Reporting a bug](reporting-a-bug.md)**

    ---

    Something behaves differently than it should. What to check first, and
    what a report needs so that it can be reproduced.

- :material-file-document-edit-outline: **[Reporting a docs issue](reporting-a-docs-issue.md)**

    ---

    A page on this site, a help card in the app or the README is wrong,
    unclear or missing. Most fixes are one pencil click away.

- :material-shield-alert-outline: **[Reporting a vulnerability](reporting-a-vulnerability.md)**

    ---

    Privately, never as an issue. What is in scope, and what you can expect
    back within 72 hours.

- :material-lightbulb-outline: **[Requesting a change](requesting-a-change.md)**

    ---

    Something it should do and does not. Describe the situation, not the
    button — the form asks for exactly that.

- :material-forum-outline: **[Asking a question](https://github.com/stackvo/stackvo/discussions)**

    ---

    "How do I…" and "should it…" belong in Discussions, where the answer
    helps the next person too.

</div>

### Contributing

<div class="grid cards" markdown>

- :material-translate: **[Adding translations](adding-translations.md)**

    ---

    The app, its help cards and this site are in English and Turkish.
    Improve either, or start the third.

- :material-source-pull: **[Making a pull request](making-a-pull-request.md)**

    ---

    Fork, branch, run the gate, open a draft. The house rules the tests
    enforce, so you meet them before CI does.

</div>

### Guides

<div class="grid cards" markdown>

- :material-test-tube: **[Creating a reproduction](../guides/creating-a-reproduction.md)**

    ---

    A fresh workspace, one project and the fewest settings that still show
    the bug — the thing that turns "cannot reproduce" into a fix.

</div>

## Before you open anything

A short check saves a round trip, and sometimes the whole issue:

- :material-checkbox-blank-circle-outline: **You are on the latest release.** Only the latest release is
      supported; there is no backport branch. **Settings → Updates** shows
      the version and installs the new one.
- :material-checkbox-blank-circle-outline: **Doctor has had a look.** **Settings → Doctor** names most of what
      goes wrong on a machine, with the repair beside it.
- :material-checkbox-blank-circle-outline: **Nobody has reported it yet.** Search the
      [issues](https://github.com/stackvo/stackvo/issues) and
      [discussions](https://github.com/stackvo/stackvo/discussions), closed
      ones included.
- :material-checkbox-blank-circle-outline: **It is one thing.** One bug, one request, one question per issue.
      Separate things can be judged separately.
- :material-checkbox-blank-circle-outline: **It is not a security problem.** If it is, it goes
      [here](reporting-a-vulnerability.md) and nowhere public.

What the project can and cannot promise — no paid tier, no support contract, one maintainer — is on [Sponsoring](../../insiders/sponsoring.md).

## Rights and responsibilities

The project has a
[code of conduct](https://github.com/stackvo/stackvo/blob/main/CODE_OF_CONDUCT.md).
The short version: be decent to people, argue with the work as hard as it
deserves, never with the person.

Issues are read by one maintainer. That means:

- An issue that skips the template, cannot be reproduced or duplicates
  another may be closed with a pointer rather than an answer. It is not a
  judgement of you; it is what one person can carry.
- A report produced by an automated tool is closed unless a person has
  confirmed it against the app.
- Everything written in the repository stays public. Write for the reader
  who finds it in a year.
