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

failed=()
step() {
  local name="$1"
  shift
  printf '\n\033[1m▸ %s\033[0m\n' "$name"
  if "$@"; then
    printf '\033[32m  ok\033[0m\n'
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
# in a commit is public the moment it is pushed. `keys.sh check` also reports
# the states nobody has closed yet (no pinned registry key, no release), which
# is the last place they are visible before a push.
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
else
  printf '\n\033[33m▸ linux and windows skipped — run with --all\033[0m\n'
fi

printf '\n'
if [ ${#failed[@]} -eq 0 ]; then
  printf '\033[32mEverything CI asks, answered here.\033[0m\n'
  exit 0
fi

printf '\033[31m%d check(s) failed:\033[0m\n' "${#failed[@]}"
printf '  %s\n' "${failed[@]}"
exit 1
