# Contributing

## Setup

```bash
npm install
npm run tauri:dev
```

You need a StackVo checkout for the app to drive. It is found through
`STACKVO_ROOT`, then by looking in the usual places; you can also pick it in
Settings.

## Before you push

**No branch is pushed without `tools/before-push.sh` having run** — `--all` for
anything that touches the platform-specific parts, which runs the Linux and
Windows halves in a container. The rule was learnt twice at the same price: once
a tree was called green on `cargo test` plus `npm test` alone and the next run
was red on all four legs; once the last edit was made *after* the gate had run.
Both times the gate had the same answers in seconds; the way they were learnt
was a merge and a three-OS matrix. The gate has a third colour, `skipped`, for
what this machine cannot answer, and it says why.

The pieces, if you want them one at a time:

```bash
npm run lint        # eslint + prettier
npm run test:js     # vitest
npm test            # the above, plus cargo test
npm run audit       # cargo-deny + npm audit
```

CI runs `cargo clippy -- -D warnings` and `cargo fmt --check` on Linux, macOS
and Windows. The Rust toolchain is pinned in `src-tauri/rust-toolchain.toml` —
bump it in a commit rather than letting it drift.

One test is not in any of the above and will not run by accident:

```bash
STACKVO_DOCKER_SMOKE=1 cargo test --test docker_smoke   # from src-tauri/
```

It lays out a throwaway workspace, builds a PHP image and brings a project up
under Docker — minutes rather than milliseconds, which is why it runs nightly
rather than on a push. **It refuses to run while you have a stack of your own
up**, and that is not caution: the generated compose project is called
`stackvo` on every machine, so it would adopt yours and remove the containers it
did not put there. Take your stack down first, or read the nightly.

## The parts that are not obvious

**`contracts/` is the agreement, not documentation.** It describes the shared
truth between the Bash CLI and this app: the manifest schema, the 159 `.env`
keys, the PHP extension catalog, the IPC surface. `npm run contracts:check`
validates the code against it, including that every command in `ipc.json` exists
in Rust and is reachable from the front end. If behaviour and contract disagree,
one of them is a bug — decide which before writing code.

**The generator is being taken over in stages.** Bash still produces the real
build inputs. The Rust port renders the same files and is compared byte-for-byte
against the Bash output, by fixture tests and by a live check in Settings. The
`generate_with` mode selector (`bash` | `verify` | `rust`) is how that
transition is driven; `rust` refuses to write on any mismatch. Do not "simplify"
this by deleting the comparison.

**`CONFLICTS.md` records upstream bugs, not ours.** C-01 through C-20 are
defects in the StackVo shell implementation, found while writing the contract.
The desktop app must not silently reproduce them; where it deviates, the entry
says so.

**Never modify the StackVo checkout.** It is a separate project. This app reads
it and writes only inside it as an explicit user action.

## Style

Comments explain _why_, not what. If a line looks odd and is correct, say what
would go wrong without it. Tests are named after the behaviour they protect, and
where a test guards a real bug, the comment says what the bug was.
