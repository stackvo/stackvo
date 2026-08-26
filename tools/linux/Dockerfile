# The Linux half of this repository, runnable without waiting for CI.
#
# §3 #12 and #35 both said the same thing: the code is written, the job is
# wired, and nobody has ever seen it run. On macOS `tauri-driver` does not
# exist and `elevate.rs`'s polkit branch compiles out, so "it will be fine"
# was the only available answer — which is the answer this repository refuses
# everywhere else.
#
# Deliberately the same base and the same packages as `.github/workflows/ci.yml`
# installs, so a green run here means something about the runner rather than
# about this file. `contract_spelling.rs` and its neighbours have the habit:
# when two places state one fact, the second one is the one that goes stale.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_TERM_COLOR=always

# The line below is kept identical to ci.yml's, plus what a container has to
# add for itself: a runner image already has curl, git, build-essential and a
# Rust toolchain.
RUN apt-get update && apt-get install -y --no-install-recommends \
      libwebkit2gtk-4.1-dev libappindicator3-dev \
      librsvg2-dev patchelf libssl-dev libdbus-1-dev \
      build-essential curl git ca-certificates pkg-config \
      xvfb webkit2gtk-driver \
    && rm -rf /var/lib/apt/lists/*

# Pinned from `src-tauri/rust-toolchain.toml`, which rustup reads from the
# working directory — installing "stable" here would put components on a
# toolchain the build never uses. That exact mistake is already commented in
# ci.yml's coverage job.
ARG RUST_VERSION=1.96.1
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain "${RUST_VERSION}" --profile minimal \
      --component rustfmt --component clippy
ENV PATH="/root/.cargo/bin:${PATH}"

# Node, because the driver suite (§3 #12) builds the front end and runs the
# tests with it. CI uses `actions/setup-node@v4` with 22; matching the major is
# what makes a green run here mean anything about a green run there.
ARG NODE_MAJOR=22
RUN curl -fsSL "https://deb.nodesource.com/setup_${NODE_MAJOR}.x" | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Cross-compiling to Windows, so `windows-latest` stops being a thing only CI
# can answer. `cargo-xwin` downloads Microsoft's own SDK headers and import
# libraries and points clang at them — which is what makes `aws-lc-sys` build,
# and `aws-lc-sys` needing `windows.h` is the reason `cargo check --target
# x86_64-pc-windows-msvc` fails on a Mac.
#
# `check`, not `test`: the tests cannot RUN here, and the errors this is for
# are the ones that stop the build before a test exists. Two of them have
# already been found this way — a `flate2` with no compression backend, and a
# Docker connector gated to the wrong platform.
RUN apt-get update && apt-get install -y --no-install-recommends \
      clang llvm lld \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-pc-windows-msvc \
    && cargo install cargo-xwin --locked

# `clang-cl` and `llvm-ml`, which Debian ships the code for and not the names.
#
# `cargo-xwin` hands MSVC-style flags to the compiler — `/imsvc <dir>` for the
# SDK headers — and those are only flags to clang's **cl-compatible driver**.
# The driver picks its mode from argv[0], so `clang-cl` is a symlink to `clang`
# and nothing more; without the name, `cc-rs` falls back to plain `clang` and
# every C file in `ring` fails with `no such file or directory: '/imsvc'` —
# an error that names a flag as if it were a path and says nothing about why.
#
# `llvm-ml` is the same story for the x86_64 row: `ring` and `aws-lc-sys` ship
# MASM assembly there, and it is the assembler that reads it.
# NSIS itself, because the bundler does not carry it off Windows.
#
# On Windows `tauri-bundler` downloads its own NSIS and runs
# `<toolset>/makensis.exe`; everywhere else it runs plain `makensis` off the
# PATH. The error it gives when that is absent still says `makensis.exe`, which
# reads like a cross-compilation dead end and is one `apt-get install` away.
RUN apt-get update && apt-get install -y --no-install-recommends \
      nsis \
    && rm -rf /var/lib/apt/lists/*

RUN ln -sf /usr/bin/clang /usr/bin/clang-cl \
    && ln -sf "$(ls -d /usr/lib/llvm-*/bin | head -1)/llvm-ml" /usr/bin/llvm-ml \
    && clang-cl --version >/dev/null && llvm-ml --version >/dev/null

# What the BUNDLER needs, which is a different list from what the compiler
# needs — and the distinction cost a release run to learn.
#
# `tauri-bundler` copies `/usr/bin/xdg-open` into the AppDir when the
# application can open a link, and this one can, through `tauri-plugin-opener`.
# It copies it from the build machine. `ubuntu-latest` ships `xdg-utils` and
# `ubuntu-24.04-arm` does not, so the first rehearsal wrote a `.deb` and an
# `.rpm` on ARM and then died on the AppImage — and this image could not have
# predicted it, because it was mirroring `ci.yml`, and `ci.yml` never bundles.
#
# `workflow_parity.rs` now holds this file against `release.yml`'s list rather
# than CI's, so the next package the bundler needs cannot be missing here.
#
# `file` is the other half of the same lesson, and it points the other way: it
# is not in any workflow and does not need to be, because a GitHub runner image
# ships it. `appimagetool` shells out to it — "file command is missing but
# required, please install it" — and a bare `ubuntu:24.04` has no such thing.
# So this image has to be a superset of the runner in two directions at once:
# every package the release declares, plus everything the runner image happens
# to give the release for free.
#
# Its own layer, at the end, so adding to it does not rebuild the Rust and Node
# layers above.
RUN apt-get update && apt-get install -y --no-install-recommends \
      xdg-utils file \
    && rm -rf /var/lib/apt/lists/*

# The ARM Windows target, so `--windows-bundle` can be asked about the row that
# §3 #22 is actually about. `cargo-xwin` handles both.
RUN rustup target add aarch64-pc-windows-msvc

# Wine, so the Windows suite can RUN here and not only type-check.
#
# `cargo xwin test` cross-builds the test binaries and hands them to wine — its
# own binary carries `WINEDEBUG` and "Failed to run cargo test" side by side,
# which is the whole mechanism. Until now `--windows` could only answer "does it
# compile", and §2 W's nineteen failures were all found on a runner, one round
# at a time, from a screenshot.
#
# Honest about which triple. On an arm64 host wine 9 executes **ARM64** PE
# binaries natively, so `aarch64-pc-windows-msvc` — the row §3 #22 is about — is
# the one this can really run. An x86_64 PE on arm64 needs an emulation layer
# wine 9 does not carry; that row wants a `--platform linux/amd64` container or
# a runner.
#
# And what wine is not: it is not Windows. A test that reads the registry, the
# credential store, an ACL or `FOLDERID_Profile` can pass here and fail there,
# or the reverse — `agents::home()` was exactly such a test. This is a first
# opinion that costs minutes, never a replacement for the job that costs a
# runner.
RUN apt-get update && apt-get install -y --no-install-recommends \
      wine \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /repo
