# How to upgrade

Only the latest release is supported, so upgrading is the first answer to
most bug reports. Two ways, ending in the same place.

## From the app

**Settings → Updates** checks for a new version and installs it. The update
is verified against a minisign key compiled into the app; a package that
fails the check is not installed. Installing closes the app. Your containers
keep running — Docker manages them, not the window.

**Also receive beta releases** adds pre-releases to what the check offers. A
beta install still gets every stable release, and a stable install is never
offered a beta. The switch takes effect at the next launch.

## By hand

Download the installer for your platform from the
[releases page](https://github.com/stackvo/stackvo/releases/latest) and
install it over the existing one. Every release publishes `SHA256SUMS`
beside its files. [Installation](../getting-started/installation.md) has
the per-platform steps and the click each OS asks for on an unsigned build.

## What an upgrade touches

| | |
| --- | --- |
| The app | Replaced |
| Your workspace: projects, manifests, `.env` | Untouched |
| Running containers | Untouched — Docker manages them, not the app |
| The help cards | Not part of a release. The app fetches them from `main`, so they are already current |

## Going back

Install an older release from the
[releases page](https://github.com/stackvo/stackvo/releases) over the current
one. Read the entries between the two versions in the changelog first: a
release that changed how a file is written says so under *Changed*.

## Reading the changelog

[The changelog](changelog.md) is one file in the repository, in
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) form: an
**Unreleased** section for what `main` has that no release does yet, then one
section per version with *Added*, *Changed*, *Fixed* and *Removed*. Entries
say why as well as what, so they run long; the [What's new](whats-new.md) page is the short form.
