# Making a pull request

The path from a fork to a merged change, and the house rules the tests
enforce so that you meet them before CI does.

## Before you start

- **A bug fix needs no permission.** Reference the issue if there is one.
- **A feature wants a sentence first.** Say in the
  [request](requesting-a-change.md) or in a
  [discussion](https://github.com/stackvo/stackvo/discussions) what you plan
  to build and roughly how. Ten minutes there saves a rewritten pull
  request.
- **One change per pull request.** A fix and a refactor in one pull request
  are reviewed as two and merged as neither. Do not reformat code you did
  not change, and do not bump the toolchain, a pinned action or a
  dependency in a pull request about something else.

## Steps

### 1. Fork and branch

```bash
gh repo fork stackvo/stackvo --clone
cd stackvo
git switch -c fix/snapshot-restore-after-rename
```

Branch names say what the branch does: `fix/…`, `feat/…`, `docs/…`, `ci/…`.

### 2. Set up

[Getting started](../../insiders/getting-started.md): Node 22, the pinned Rust toolchain,
`npm install`, `npm run tauri:dev`.

### 3. Make the change, with its test

Every fix comes with the test that would have caught it, named after the
behaviour it protects. Where a test guards a real bug, its comment says what
the bug was. Comments explain *why*, not what: if a line looks odd and is
correct, say what would go wrong without it.

### 4. Run the gate

```bash
tools/before-push.sh          # what this machine can answer
tools/before-push.sh --all    # and the Linux and Windows halves, in a container
```

**No branch is pushed without it having run.** It asks everything CI asks —
lint, vitest, cargo test, clippy with `-D warnings`, fmt, the contract check,
the audits — and prints green, red or *skipped* per check. Use `--all` for
anything that touches platform-specific code: `engine.rs`, `hosts.rs`,
`pty.rs` and their kin cannot be compiled from a Mac alone.

### 5. Push and open a draft

```bash
git push -u origin fix/snapshot-restore-after-rename
gh pr create --draft --fill
```

A draft says "not yet" without you having to. Open it early if you want a
look at the direction; mark it ready when the gate is green.

### 6. Fill in the template

The pull request template asks what the change does and why, and ends with
a checklist:

- :material-checkbox-blank-circle-outline: `npm test` passes (vitest + cargo test)
- :material-checkbox-blank-circle-outline: `npm run lint` passes
- :material-checkbox-blank-circle-outline: `npm run contracts:check` is clean
- :material-checkbox-blank-circle-outline: If behaviour and `contracts/` disagreed, the contract was updated too
- :material-checkbox-blank-circle-outline: The workspace the app manages is untouched

### 7. Review

CI runs the suite on Linux, macOS and Windows. The maintainer reviews;
`CODEOWNERS` routes the contracts, the tools, the workflows and the
security-relevant Rust modules to a deliberate look. Answer a review comment
by pushing a commit, not by rewriting history over it — the reviewer reads
the diff since last time.

### 8. Merge

Merged by the maintainer once CI is green and the review is done. Delete
your branch afterwards.

## The house rules

These are tested, so they fail on your machine before they fail in review.

**The contract comes first.** `contracts/ipc.json` names every command and
event the two halves share. A command absent from it can be neither
registered in `lib.rs` nor driven from the CLI. If behaviour and contract
disagree, one of them is a bug — decide which before writing code, and
change both in the same pull request.

**Generated files are never hand-edited.** `generated/` is rendered from the
manifest every time. The Rust generator is compared byte for byte against
the Bash one; do not "simplify" that by deleting the comparison.

**A pane's markup and its styles move together.** `<style scoped>` reaches
only its own component; `tests/pane-styles.spec.js` checks it.

**Documents are tested against the tree.** A number in ARCHITECTURE.md, a
host in PRIVACY.md, a topic name in a help card — each has a test that reads
the file and fails when the code disagrees. Change the document and the code
together.

**Never modify the workspace from code.** The folder the app manages is the
user's. The app reads it and writes into it only as an explicit user action.

**A dependency is a decision.** `cargo deny` and dependency review gate
licences and advisories on every pull request. Name a new crate or package
in the description, with why it is needed.
