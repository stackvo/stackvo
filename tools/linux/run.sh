#!/usr/bin/env bash
# Run the Linux-only halves of the suite, on this machine, in a container.
#
#   tools/linux/run.sh                      # the probes that only exist on Linux
#   tools/linux/run.sh --test elevate_probe # one of them
#   tools/linux/run.sh --driver             # the tauri-driver suite (#12), with a display
#   tools/linux/run.sh --windows            # type-check the Windows branch (#35)
#   tools/linux/run.sh --bundle             # build the installers this arch ships (#22)
#   tools/linux/run.sh --windows-bundle     # the NSIS installer, cross-built (#22)
#   tools/linux/run.sh --windows-test       # the Windows suite, under wine (W)
#   tools/linux/run.sh --shell              # a prompt inside the image
#
# The image caches; the cargo registry and target directory are bind-mounted to
# named volumes so a second run is a rebuild rather than a fresh one.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
image=stackvo-linux

# Rebuilt when the Dockerfile changes, and not only when the image is absent.
#
# `docker image inspect` alone was the test, and it answers "is there an image
# called this" rather than "is it this file's image". So every layer added after
# somebody's first build was invisible to them: the machine that found this had
# an image from the 25th and a Dockerfile from the 27th, and the two days in
# between were `xdg-utils`, `file`, `wine` and the ARM Windows target — which is
# to say, exactly the packages three of the modes below exist to use.
#
# What that produced was worse than an old image. `--bundle` failed on
# `xdg-open binary not found` and `--windows-test` on `can't find crate for
# core`, and both of those read as a broken build rather than as a stale
# container. The Dockerfile beside this file had already been taught both
# lessons, in comments, at length; the image just was not carrying them.
#
# The digest is a label rather than a file beside the image, so it travels with
# the thing it describes and cannot be left behind by a `docker rmi`.
dockerfile_digest="$(shasum -a 256 "$repo/tools/linux/Dockerfile" | cut -d' ' -f1)"
built_from="$(docker image inspect "$image" \
  --format '{{index .Config.Labels "stackvo.dockerfile"}}' 2>/dev/null || true)"

if [ "$built_from" != "$dockerfile_digest" ]; then
  if [ -n "$built_from" ]; then
    echo "$image was built from a different Dockerfile — rebuilding ..."
  else
    echo "building $image (once) ..."
  fi
  docker build -t "$image" \
    --label "stackvo.dockerfile=$dockerfile_digest" \
    "$repo/tools/linux"
fi

docker volume create stackvo-cargo >/dev/null
docker volume create stackvo-target >/dev/null
# node_modules gets its OWN volume, mounted over the bind. Without it the
# container's `npm ci` writes Linux binaries into the host checkout and the next
# `npx vitest` on the Mac dies on a missing `@rollup/rollup-darwin-arm64` — a
# container that breaks the machine it was run from is not a convenience.
docker volume create stackvo-node-modules >/dev/null
# `cargo-xwin` downloads Microsoft's CRT and SDK into `~/.cache` — a minute
# each time, on every run, because the container is `--rm`. It is the same
# argument the three volumes above already make, for the one directory nobody
# had noticed was being thrown away.
docker volume create stackvo-cache >/dev/null

# `--jobs 1` for the link step, not for speed. Docker Desktop's VM defaults to
# well under what this crate's linker wants — GTK, WebKit, bollard, rustls and
# aws-lc in one binary — and parallel `ld` invocations get OOM-killed with
# `signal 9`, which reads as a linker bug rather than as memory. Compiling is
# not the expensive part; linking three test binaries at once is.
run() {
  docker run --rm -i \
    -v "$repo:/repo" \
    -v stackvo-cargo:/root/.cargo/registry \
    -v stackvo-target:/repo/src-tauri/target-linux \
    -v stackvo-node-modules:/repo/node_modules \
    -v stackvo-cache:/root/.cache \
    -e CARGO_TARGET_DIR=/repo/src-tauri/target-linux \
    -e CARGO_BUILD_JOBS="${STACKVO_LINUX_JOBS:-1}" \
    -w /repo/src-tauri \
    "$image" "$@"
}

if [ "${1:-}" = "--shell" ]; then
  exec docker run --rm -it \
    -v "$repo:/repo" \
    -v stackvo-cargo:/root/.cargo/registry \
    -v stackvo-target:/repo/src-tauri/target-linux \
    -v stackvo-node-modules:/repo/node_modules \
    -v stackvo-cache:/root/.cache \
    -e CARGO_TARGET_DIR=/repo/src-tauri/target-linux \
    -w /repo "$image" bash
fi

# The driver suite needs more than a plain `cargo test`: the front end
# built, the application built the way it SHIPS, `tauri-driver` on PATH, and a
# display, because the app opens a window.
#
# `npx tauri build --debug --no-bundle`, not `cargo build`. A plain cargo build
# embeds `devUrl` — the webview opens http://localhost:1420, gets "connection
# refused", and every test reports an empty `#app` with no explanation. So the
# headline test of this suite, "the built bundle renders inside the real
# webview", could never pass with the profile the suite itself chose.
# `--no-bundle` because what is wanted is a binary to drive, not a .deb.
#
# `STACKVO_DRIVER_BINARY` because the container builds into `target-linux/` (so
# a Linux object file never lands in the macOS target dir and invalidates it),
# while `binaryPath()` looks in `target/`.
if [ "${1:-}" = "--driver" ]; then
  run bash -lc '
    set -euo pipefail
    cd /repo
    npm ci --no-audit --no-fund
    # Real sidecars, not stubs: `beforeBuildCommand` runs `sidecars.mjs
    # --verify`, which refuses a placeholder — correctly, since this step builds
    # the application the way it ships. Without this the run stopped at
    #   sidecars: these are placeholders, not builds
    # on any machine that had never built the Linux binaries, which reads as a
    # broken driver suite rather than as a missing step.
    npm run sidecars:release
    npm run build
    npx tauri build --debug --no-bundle
    command -v tauri-driver >/dev/null || cargo install tauri-driver --locked
    STACKVO_DRIVER_BINARY=/repo/src-tauri/target-linux/debug/stackvo-desktop \
      xvfb-run -a --server-args="-screen 0 1280x820x24" npm run test:driver
  '
  exit $?
fi

# The half nothing here could answer, and the reason a release run was
# being used as a test environment.
#
# Every other mode in this file compiles or tests. None of them **bundle**, and
# the bundler is a different program with different needs: it runs `linuxdeploy`,
# it shells out to `dpkg-deb`, it copies files off the build machine. The first
# rehearsal proved the distinction the expensive way — `ubuntu-24.04-arm` wrote
# `StackVo_0.1.0_arm64.deb` and `StackVo-0.1.0-1.aarch64.rpm` and then died with
# `xdg-open binary not found`, a package this image did not have either, because
# it was mirroring `ci.yml` and `ci.yml` never bundles.
#
# On an Apple Silicon machine this container **is** `aarch64-unknown-linux-gnu`
# — the same row that failed. So that failure was always reproducible here, in
# one command, and nobody could run it because the command did not exist.
#
# `--platform linux/amd64` for the other Linux row; it works under emulation and
# is slow enough that it is not the default.
if [ "${1:-}" = "--bundle" ]; then
  run bash -lc '
    set -euo pipefail
    cd /repo
    triple="$(uname -m)-unknown-linux-gnu"
    npm ci --no-audit --no-fund
    # Real builds, not stubs: `beforeBuildCommand` runs `sidecars.mjs --verify`,
    # which refuses a placeholder — correctly, since this produces the bundle
    # the way it ships.
    node tools/sidecars.mjs --release --target "$triple"
    npm run build
    # `--no-sign` because there is no private key here and there should not be.
    # The release job passes it too on a rehearsal, and for the same reason:
    # nothing is being published, so there is nothing for a signature to protect.
    npx tauri build --target "$triple" --no-sign
    # The same verdict the release job writes into its run summary — every
    # format this platform owes, each named for this architecture.
    node tools/check-installers.mjs --target "$triple" --unsigned \
      --dir "$CARGO_TARGET_DIR/$triple/release/bundle"
  '
  exit $?
fi

# The Windows half — as far as it can be taken off a Windows machine.
#
# NSIS is buildable here and MSI is not, and that is `tauri-bundler`'s own
# division rather than a limitation of this script: its `msi` module is
# `#[cfg(target_os = "windows")]`, while the `nsis` one carries the comment
# "don't restrict to windows as NSIS installers can be built in linux+macOS
# using cargo-xwin".
#
# Worth having anyway, because NSIS is the artifact that matters most: the
# updater downloads the `-setup.exe` on Windows, not the `.msi`. So this answers
# "does the Windows installer build for this target" for the one installer a
# running application will ever fetch — including `aarch64-pc-windows-msvc`,
# which is half of what #22 is about.
#
# What it cannot answer stays honest and stays on the runner: the MSI, and
# anything about how the installer behaves once it is running.
if [ "${1:-}" = "--windows-bundle" ]; then
  triple="${2:-x86_64-pc-windows-msvc}"

  # And one row it cannot answer, measured rather than assumed.
  #
  # `ring` compiles its ARM64 Windows assembly with plain `clang` rather than
  # `clang-cl`, because MSVC's assembler cannot read that syntax. Plain clang
  # then cannot read Microsoft's ARM64 headers: `winnt.h` stops at
  # `#define __MACHINEARM_ARM64 __MACHINE`, which needs the cl driver. The two
  # requirements are in the same `cc::Build`, so there is no flag that satisfies
  # both — checked, on this machine: `clang` accepts only `-isystem`, `clang-cl`
  # only `-imsvc`/`/imsvc`, and `clang-cl` reading `INCLUDE` does not help a
  # compile that is not clang-cl's.
  #
  # Said here, in a second, rather than found after ten minutes of compiling.
  if [ "$triple" = "aarch64-pc-windows-msvc" ]; then
    echo "aarch64-pc-windows-msvc cannot be cross-built here." >&2
    echo >&2
    echo "  ring builds its ARM64 Windows assembly with plain clang, and plain clang" >&2
    echo "  cannot compile Microsoft's ARM64 headers — winnt.h stops at" >&2
    echo "  \`#define __MACHINEARM_ARM64 __MACHINE\`, which needs the cl driver." >&2
    echo >&2
    echo "  The x86_64 row does build here:" >&2
    echo "    tools/linux/run.sh --windows-bundle x86_64-pc-windows-msvc" >&2
    echo >&2
    echo "  The ARM row is answered by \`windows-11-arm\` in the release workflow," >&2
    echo "  or by a Windows ARM64 machine." >&2
    exit 2
  fi
  run bash -lc "
    set -euo pipefail
    cd /repo
    npm ci --no-audit --no-fund
    node tools/sidecars.mjs --release --target '$triple' --runner cargo-xwin
    npm run build
    npx tauri build --runner cargo-xwin --target '$triple' --bundles nsis --no-sign
    node tools/check-installers.mjs --target '$triple' --unsigned --only nsis \
      --dir "\$CARGO_TARGET_DIR/$triple/release/bundle"
  "
  exit $?
fi

# `tauri-build` checks every `externalBin` file exists on any cargo build of
# this package, and it looks for the CONTAINER's triple — so the host's stubs
# are not ones. The `--windows` branch below has done this since the day it was
# written, for exactly the reason it gives; the Linux side never did, so on a
# machine that had not built Linux sidecars this script stopped at
#   resource path `binaries/stackvo-aarch64-unknown-linux-gnu` doesn't exist
# which reads as a missing file rather than as a missing step. Stubs are enough
# here: nothing in the probe run executes them.
# `uname -m` inside the container, not out here: the host says `arm64` and
# the triple wants `aarch64`.
run bash -lc 'node ../tools/sidecars.mjs --stubs --target "$(uname -m)-unknown-linux-gnu"'

# The exit status is the point of running this at all: the first version of
# this script ended on a pipeline whose status was the tail's, so a failing
# compile reported success. A runner that cannot fail is not a runner.
# The other half. The Windows branch of this crate is only ever read on a
# Mac, and `cargo check --target x86_64-pc-windows-msvc` cannot help there:
# `aws-lc-sys` wants `windows.h`. `cargo-xwin` fetches Microsoft's SDK and
# points clang at it, so the type checker finally reads those lines.
# The Windows suite, run rather than type-checked.
#
# `--windows` above answers "does it compile", and that has been the ceiling
# since it was written: every one of W's nineteen failures was found on a
# runner, one round at a time, and two of those rounds were spent learning that
# `cargo test` stops at the first failing binary.
#
# `cargo xwin test` cross-builds the test binaries and hands them to wine. On
# this arm64 machine the triple that really runs is `aarch64-pc-windows-msvc` —
# wine 9 executes ARM64 PE natively — which happens to be the row this
# about. The x86_64 row wants a `--platform linux/amd64` container or a runner.
#
# Wine is not Windows, and the gap is exactly the shape of the bugs W is made
# of: the registry, the credential store, ACLs, `FOLDERID_Profile`. A green run
# here is a first opinion worth minutes, not the answer the runner gives.
if [ "${1:-}" = "--windows-test" ]; then
  # The triple is chosen by what wine on THIS host can execute, not by taste.
  # On x86_64 that is the x86_64 row, which is also the row CI runs.
  case "$(uname -m)" in
  x86_64 | amd64) triple="${2:-x86_64-pc-windows-msvc}" ;;
  *) triple="${2:-aarch64-pc-windows-msvc}" ;;
  esac

  # And on an arm64 host the two halves do not meet.
  #
  # Wine 9 here executes ARM64 PE and nothing else, so `aarch64-pc-windows-msvc`
  # is the only triple that could run — and it is the one triple this image
  # cannot build. `--windows-bundle` above refuses it in a second for `ring`;
  # this ran it anyway and spent ten minutes finding the same wall from the
  # other side. Both backends were measured today:
  #
  #   clang-cl (default) — `ring` compiles its ARM64 Windows assembly from `.S`,
  #     which the cl driver cannot assemble, so `cc-rs` falls back to plain
  #     clang and carries the MSVC flags with it: `clang: error: no such file
  #     or directory: '/imsvc'`, five times per object.
  #   clang (XWIN_CROSS_COMPILER=clang) — the `/imsvc` flags come out GNU-style
  #     and `ring` gets past it, and then `aws-lc-sys` stops in
  #     `jitterentropy-timer.c` because that backend's sysroot does not carry
  #     Microsoft's ARM64 intrinsics: 1309 warnings and one error.
  #
  # So this is not a red step, and it is not a green one either — nobody got an
  # answer. Exit 3 is `tools/before-push.sh`'s third colour: skipped, named in
  # the summary, counted as neither.
  #
  # Where the answer does come from: `windows-latest` in `ci.yml` runs this
  # suite for real on every push, and it is x86_64. An x86_64 Linux or Intel Mac
  # runs it here for real too — the branch above picks that triple by itself.
  if [ "$triple" = "aarch64-pc-windows-msvc" ]; then
    echo "windows · test cannot be answered on an arm64 host." >&2
    echo >&2
    echo "  wine here executes ARM64 PE only, and aarch64-pc-windows-msvc is the" >&2
    echo "  one Windows triple this image cannot cross-build: ring stops on" >&2
    echo "  \`/imsvc\` under clang-cl, and aws-lc-sys stops on Microsoft's ARM64" >&2
    echo "  intrinsics under clang. Both were measured; see the comment above." >&2
    echo >&2
    echo "  The suite is run for real by \`windows-latest\` in ci.yml, and by" >&2
    echo "  this same command on an x86_64 host, where it picks x86_64 by itself." >&2
    exit 3
  fi

  run bash -lc "
    set -euo pipefail
    node ../tools/sidecars.mjs --stubs --target '$triple'
    cargo xwin test --target '$triple' --no-fail-fast
  "
  exit $?
fi

if [ "${1:-}" = "--windows" ]; then
  # `tauri-build` checks the `externalBin` files exist on every cargo build of
  # this package, and it looks for the **target's** triple — so a host stub is
  # not one. Without this the check stopped at "resource path
  # binaries/stackvo-x86_64-pc-windows-msvc.exe doesn't exist", which reads as a
  # missing file rather than as a missing step and is the last thing anybody
  # wants between them and a Windows type error.
  # `run` works from /repo/src-tauri, so the tool is one level up.
  run node ../tools/sidecars.mjs --stubs --target x86_64-pc-windows-msvc
  run cargo xwin check --target x86_64-pc-windows-msvc --all-targets || exit $?

  # Clippy too, because `ci.yml` runs clippy on `windows-latest` with
  # `-D warnings` and `check` does not answer that question.
  #
  # The gap has a shape, and it is `cfg`. A `#[cfg(windows)]` block is the tail
  # of its function on Windows and dead text everywhere else, so a `return`
  # inside it is needless on exactly one platform — and `needless_return` is
  # denied there. Two of those reached main: `engine.rs`'s named-pipe connect
  # and `xdebug.rs`'s output directory, both invisible to every clippy run on
  # this machine and to `check` on that one.
  run cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings
  exit $?
fi

if [ $# -gt 0 ]; then
  run cargo test "$@"
  exit $?
else
  # The two that cannot run on the developer's machine at all. Named rather
  # than "everything": a full run here would be a slower copy of what `cargo
  # test` already answers on macOS, and this is for the answers it cannot give.
  run cargo test --test elevate_probe --test hosts_roundtrip
  exit $?
fi
