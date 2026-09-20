# Insiders

An insider runs StackVo from `main`: the tree the next release is cut from,
with every fix the changelog lists under *Unreleased* and none of the
waiting. There is no separate build, no sponsor gate and nothing to unlock —
the repository is public, and the difference between an insider and everybody
else is a `git clone`.

<div class="grid cards" markdown>

- :material-source-branch: **[Getting started](getting-started.md)**

    ---

    From a clone to the app running on your machine: the tools, the
    first build, the checks that CI will ask for anyway.

- :material-new-box: **[What's new](whats-new.md)**

    ---

    Each release in a page: what changed for the person using the app,
    without the reasoning the changelog carries.

- :material-history: **[Changelog](changelog.md)**

    ---

    Every change, with why. The file from the repository, as it is —
    *Unreleased* first.

- :material-update: **[How to upgrade](upgrade.md)**

    ---

    From the app, by hand, and what an upgrade does and does not touch.

- :material-heart-outline: **[Sponsoring](sponsoring.md)**

    ---

    What the project runs on, what a sponsorship pays for, and what it does
    not buy.

</div>

## What running `main` means

- **You are ahead of the release.** A fix the changelog lists under
  *Unreleased* is on your machine before it is on anybody else's. So is a
  regression, which is why the pre-push gate and the three-OS CI exist.
- **Your workspace is the same one.** A development build reads and writes
  `~/.stackvo` like an installed copy does. Nothing is migrated on the way
  in or out, and a release that changes how a file is written says so in
  the changelog.
- **A report from `main` is the most useful kind.** It names a commit rather
  than a version, and it arrives before the release does. The
  [bug report](../community/contributing/reporting-a-bug.md) form fits; say
  the commit.
