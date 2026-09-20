# Getting started

Insiders run StackVo from `main` — the tree the next release is cut from,
with every fix the [changelog](changelog.md) lists under *Unreleased* and
none of the waiting. There is no separate build and nothing to unlock: the
repository is public, and this page takes you from a clone to the app
running on your machine.

Everything below runs on a laptop with Docker on it. Ten minutes for the
tools, a few more for the first Rust build, and after that the app opens in
seconds.

## Requirements

| What | Version | Where it is pinned |
| --- | --- | --- |
| Node.js | 22 | `.nvmrc` |
| Rust | 1.96.1, with `rustfmt` and `clippy` | `src-tauri/rust-toolchain.toml` |
| Tauri 2 system libraries | per platform | [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/) |
| Docker | any current engine | to run what the app manages |

`rustup` reads the toolchain file and installs that exact version by itself.
Node comes from `nvm use` if you have nvm, or from nodejs.org.

## Get the code

```bash
git clone https://github.com/stackvo/stackvo.git
cd stackvo
npm install
```

`npm install` also brings the Tauri CLI; nothing is installed globally.

## Run it

```bash
npm run tauri:dev
```

The first build compiles the Rust half and takes a few minutes. After that a
change to the Vue front end reloads in place, and a change to Rust rebuilds
the binary and reopens the window.

The app needs a workspace to drive — the same `~/.stackvo` an installed copy
uses. It is found through `STACKVO_ROOT`, then in the usual places; you can
also pick one in **Settings → Workspace**.

!!! warning "One StackVo at a time"

    A development build manages the same workspace and the same containers as
    an installed copy. Quit one before you start the other.

## Run the checks

```bash
npm run lint               # eslint + prettier
npm run test:js            # vitest, the front end
npm test                   # the above plus cargo test
npm run test:e2e           # Playwright, accessibility included
npm run contracts:check    # the IPC contract against the code
npm run audit              # cargo-deny + npm audit
```

Before pushing, run them all at once, the way CI does:

```bash
tools/before-push.sh          # what this machine can answer
tools/before-push.sh --all    # plus the Linux and Windows halves, in a container
```

The gate prints green, red or *skipped* per check, and says why for the
skipped ones. **No branch is pushed without it having run.** CI asks the same
list on Linux, macOS and Windows, with `cargo clippy -D warnings` and
`cargo fmt --check`, and a red run there costs a lot more than a red run here.

## Build the installers

```bash
npm run tauri:build
```

Produces the same bundles a release does, unsigned.
[Installation](../getting-started/installation.md) has the click each OS asks
for on an unsigned build.

## Where things live

- `src/` — the Vue 3 front end: views, components, composables, and
  `lib/ipc.js`, the one place that calls the back end.
- `src-tauri/src/` — the Rust back end, flat, one module per concern;
  `commands.rs` is the IPC surface.
- `contracts/` — the manifest schema, the `.env` keys, the IPC surface.
- `docs/` — this site and the help cards; `docs/README.md` says how the site
  is built.
- `tests/` and `src-tauri/tests/` — the vitest, Playwright and Rust suites.

[ARCHITECTURE.md](https://github.com/stackvo/stackvo/blob/main/ARCHITECTURE.md)
is the map: the one flow worth knowing, the layers, and why the two halves
share a contract rather than a type. The rules the tests enforce — the
contract first, generated files never hand-edited — are on
[Making a pull request](../community/contributing/making-a-pull-request.md),
for the day a fix of yours is worth sending back.

## This site

```bash
python3 -m venv .venv && . .venv/bin/activate
pip install -r docs/requirements.txt
npm run docs:serve          # http://127.0.0.1:8000/stackvo/
```

Or the theme's Docker image, if you would rather not install Python.
`docs/README.md` has that command and everything else about the site.
