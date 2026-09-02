#!/usr/bin/env bash
# Everything CI will ask, asked here first.
#
#   tools/before-push.sh          # the checks that run on this machine
#   tools/before-push.sh --all    # and the Linux and Windows ones, in a container
#
# ## Why this exists
#
# Every job in `ci.yml` can be run locally, and for three rounds none of them
# were: a push went out, a red run came back, and the next change was written
# from a screenshot of a log. `tools/linux/` had already been built for exactly
# this and was not used.
#
# The four failures that reached CI in those rounds were all findable here:
#
#   * a socket test that assumed a StackVo workspace — `cargo test` on any
#     machine without one;
#   * `flate2` with no compression backend — `--windows`;
#   * a Docker connector gated to the wrong platform — `--windows`;
#   * a `stop()` that returned before the port closed — `cargo test`, on the
#     third run rather than the first, which is what a race looks like.
#
# ## What `--all` costs
#
# The container builds once (several minutes) and caches. After that the Linux
# and Windows passes are a few minutes each. The default without `--all` is the
# fast set: it catches most things and is quick enough to run before every push.
set -uo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

all=0
[ "${1:-}" = "--all" ] && all=1

# One run at a time, on this tree.
#
# Two of these on one machine do not queue — they race, and what they race for
# is the cargo build lock and the CPU. That produced a `front end · tests`
# failure whose cause was the other run: thirteen vitest tests take more than
# three seconds on an idle machine, and a second gate holding the cores pushes
# them past even the 20-second ceiling. The suite passes on its own, so the
# report was a fact about the machine wearing the clothes of a defect.
#
# `mkdir` rather than a lock file, because it is the one create that is atomic
# everywhere this runs. The trap removes it on any exit — including the Ctrl-C
# that a long `--all` invites.
lock="${TMPDIR:-/tmp}/stackvo-before-push.lock"
if ! mkdir "$lock" 2>/dev/null; then
  printf '\033[31mAnother before-push.sh is already running.\033[0m\n' >&2
  printf 'Two at once race for the cargo lock and the cores, and the loser\n' >&2
  printf 'reports failures that are about the machine rather than the tree.\n\n' >&2
  printf 'Wait for it, or remove %s if nothing is running.\n' "$lock" >&2
  exit 1
fi
trap 'rmdir "$lock" 2>/dev/null || true' EXIT

failed=()
skipped=()

# Exit 3 means "this machine cannot answer this", and it is not a failure.
#
# Without it there are only two colours, and a step that cannot run has to pick
# one: green, which claims an answer nobody got, or red, which sends the reader
# looking for a defect that is not there. `windows · test` spent ten minutes
# producing the second — a wall of cc-rs output whose meaning was "this
# container cannot cross-build ARM64 Windows", which reads exactly like a
# Windows test failing.
#
# A third colour costs one branch and keeps the summary honest: the run is
# green when nothing failed, and it still says out loud what went unasked.
step() {
  local name="$1"
  shift
  printf '\n\033[1m▸ %s\033[0m\n' "$name"
  local status=0
  "$@" || status=$?
  if [ "$status" -eq 0 ]; then
    printf '\033[32m  ok\033[0m\n'
  elif [ "$status" -eq 3 ]; then
    printf '\033[33m  skipped\033[0m\n'
    skipped+=("$name")
  else
    printf '\033[31m  FAILED\033[0m\n'
    failed+=("$name")
  fi
}

step "rust · format"        bash -c 'cd src-tauri && cargo fmt --check'
step "rust · clippy"        bash -c 'cd src-tauri && cargo clippy --all-targets -- -D warnings'
step "rust · tests"         bash -c 'cd src-tauri && cargo test --tests'
step "front end · lint"     npm run --silent lint
step "front end · tests"    npm run --silent test:js
step "types · generated"    npm run --silent types:check
step "types · compile"      npm run --silent types:tsc
step "contracts · fixture"  npm run --silent test:js -- tests/validate-contracts.spec.js
step "contracts · tree"     node tools/validate-contracts.mjs --allow-no-manifests
step "front end · build"    npm run --silent build
# Three gates CI runs and this script did not, which is how three separate
# failures reached a push that this file had just called clean. The bundle
# budget had been red for three merges, the coverage floors were failing on an
# empty report, and `cargo deny` was never asked here at all. A script whose
# opening line is "everything CI will ask, asked here first" has to be true or
# it is worse than absent — people stop reading the runs.
step "front end · bundle"   npm run --silent bundle:budget
step "supply · audit"       npm audit --omit=dev --audit-level=moderate
step "supply · deny"        bash -c 'cd src-tauri && cargo deny check'
step "supply · notice"      npm run --silent notice:check
# Not what CI asks — CI cannot see the keys — but the same instinct, and the one
# check whose failure is unrecoverable rather than inconvenient: a private key
# in a commit is public the moment it is pushed. `keys.sh check` also asks
# whether each key on this machine is the one the build pins — the content key's
# version of that going wrong is caught nowhere else, and its symptom is an
# index every installed copy of the app refuses at once.
step "keys · ceremony"      tools/keys.sh check

if [ "$all" -eq 1 ]; then
  # Behind `--all` because `cargo llvm-cov` re-instruments and re-runs the whole
  # suite — five minutes, against seconds for everything above. CI asks it on
  # every push and this script cannot afford to; what it can do is be the place
  # somebody runs it before a release rather than reading about it afterwards.
  step "coverage · floors"  bash -c 'cd src-tauri && cargo llvm-cov --ignore-run-fail --summary-only >/dev/null && cargo llvm-cov report --json --summary-only > ../rust-coverage.json' \
    && step "coverage · gate" node tools/check-coverage.mjs --rust
  step "linux · probes"     tools/linux/run.sh
  step "linux · driver"     tools/linux/run.sh --driver
  step "windows · check"    tools/linux/run.sh --windows
  # `--windows` type-checks; it does not RUN anything. That distinction cost a
  # red `windows-latest` leg that this file had just called clean, and the three
  # tests behind it had been failing for as long as the leg had: a unix-socket
  # assertion that cannot hold where the daemon is a named pipe, and two that
  # shelled out to a bash whose only presence on that runner is WSL's launcher —
  # which starts, fails, and was accepted as a working shell.
  #
  # None of the three needed a Windows machine to find. `--windows-test` runs
  # the suite under wine and has been in `tools/linux/run.sh` since it was
  # written; the only reason it never ran is that nothing here asked for it.
  step "windows · test"     tools/linux/run.sh --windows-test
  # The fourth thing CI asks that this file did not, and the most expensive one
  # to have learned elsewhere. `ci.yml` compiles and tests; `release.yml`
  # BUNDLES, and the bundler is a different program — it runs `linuxdeploy`, it
  # shells out to `dpkg-deb`, it copies files off the build machine. None of the
  # three lines above ever produced an installer, so the only place one was ever
  # produced was a release run, which is how a screenshot of a log became the
  # way this repository learned that `ubuntu-24.04-arm` has no `xdg-utils`.
  #
  # On an Apple Silicon machine the container is `aarch64-unknown-linux-gnu` —
  # the row that failed. It was always reproducible here.
  #
  # NSIS and not MSI on the Windows line, and that is `tauri-bundler`'s division
  # rather than this file's: its `msi` module is `#[cfg(target_os = "windows")]`.
  # NSIS is the half worth having anyway — the updater downloads the
  # `-setup.exe`, never the `.msi`.
  step "linux · bundle"     tools/linux/run.sh --bundle
  step "windows · bundle"   tools/linux/run.sh --windows-bundle
else
  printf '\n\033[33m▸ linux and windows skipped — run with --all\033[0m\n'
fi

printf '\n'
if [ ${#skipped[@]} -ne 0 ]; then
  printf '\033[33m%d check(s) this machine cannot answer:\033[0m\n' "${#skipped[@]}"
  printf '  %s\n' "${skipped[@]}"
  printf '\n'
fi

if [ ${#failed[@]} -eq 0 ]; then
  printf '\033[32mEverything CI asks, answered here.\033[0m\n'
  exit 0
fi

printf '\033[31m%d check(s) failed:\033[0m\n' "${#failed[@]}"
printf '  %s\n' "${failed[@]}"
exit 1
