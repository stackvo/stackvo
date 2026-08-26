#!/usr/bin/env node
/**
 * Build `stackvo` and `stackvo-mcp`, and put them where Tauri expects a sidecar.
 *
 * ## Why this file exists
 *
 * Both are real programs this repository builds and neither was bundled. The
 * README documented `stackvo`, `agents.rs` registered `stackvo-mcp` with six
 * assistants, `tooling.rs` offers to link both onto your `PATH` — and on an
 * *installed* app all three found nothing, because the only copies that had
 * ever existed were in somebody's `target/release`. The instruction was "clone
 * the repository and run cargo", which is not an instruction you can give the
 * person who downloaded a `.dmg`.
 *
 * `bundle.externalBin` is how Tauri carries an extra executable. It looks for
 * `<name>-<target-triple>` — the triple is the point, because one bundle is
 * built per target and the bundler has to know which of the six files on disk
 * belongs in this one. Nothing in cargo writes that name, so this does.
 *
 * ## The cycle, and the placeholder that breaks it
 *
 * Measured, not assumed: `tauri-build` checks that every `externalBin` file
 * exists, and it runs for **any** cargo build of this package — including the
 * one that produces the sidecars, because they are `[[bin]]` targets of the
 * crate that carries the build script. So building them requires them:
 *
 *   resource path `binaries/stackvo-aarch64-apple-darwin` doesn't exist
 *
 * The check is existence and nothing else, so [`stubs`] writes a placeholder
 * first and the real binary is copied over it. The placeholder is **text**, and
 * deliberately: if one ever escaped into a bundle it would fail loudly on the
 * first run rather than be a zero-byte file that looks like a truncated
 * download. [`verify`] is what stops that from being possible at all — it runs
 * from `beforeBuildCommand`, so no `tauri build` on any path can bundle one.
 *
 * The alternative was `TAURI_CONFIG` with `externalBin` emptied for the sidecar
 * build. It works and it is worse: the build script declares
 * `rerun-if-env-changed=TAURI_CONFIG`, so the two builds would invalidate each
 * other and every `cargo test` after a `tauri build` would recompile the whole
 * crate.
 *
 * ## Why it runs before *dev* as well as before *build*
 *
 * A missing `externalBin` is a hard error, not a warning. Hooking only the
 * release path would mean every developer's next `npm run tauri:dev` failing on
 * a file they had never heard of.
 *
 * ## What it does not do
 *
 * It does not strip, sign or compress. Signing is the bundler's — on macOS a
 * sidecar signed here is re-signed there anyway — and stripping would make a
 * crash report from a released `stackvo` unreadable to save about a megabyte.
 *
 * Usage:
 *   node tools/sidecars.mjs                    debug, for this machine
 *   node tools/sidecars.mjs --release          release, for this machine
 *   node tools/sidecars.mjs --release --target aarch64-apple-darwin
 *   node tools/sidecars.mjs --stubs            placeholders only, for `cargo test`
 *   node tools/sidecars.mjs --verify           refuse to bundle a placeholder
 *   node tools/sidecars.mjs --release --target aarch64-pc-windows-msvc --runner cargo-xwin
 */

import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, existsSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const MANIFEST = join(ROOT, 'src-tauri', 'Cargo.toml');
const OUT_DIR = join(ROOT, 'src-tauri', 'binaries');

/**
 * Where cargo writes, which is not always `src-tauri/target`.
 *
 * `tools/linux/run.sh` sets `CARGO_TARGET_DIR` to `target-linux/` on purpose —
 * so a Linux object file never lands in the macOS target directory and
 * invalidates it. Hard-coding `target/` here meant cargo succeeded and the copy
 * below failed on a path nobody had chosen, with "cargo reported success but
 * ... does not exist": the one error message this file was written to avoid.
 */
const TARGET_DIR = process.env.CARGO_TARGET_DIR ?? join(ROOT, 'src-tauri', 'target');

/**
 * The two binaries, named once.
 *
 * `tauri.conf.json` names them again, because that file is what the bundler
 * reads. `sidecar_bundle.rs` fails the build if the two lists disagree — a
 * sidecar added here and not there is one that gets built and never shipped,
 * which is the failure this whole file exists to end.
 */
export const SIDECARS = ['stackvo', 'stackvo-mcp'];

/** Smaller than this is not a build of anything. A debug `stackvo` is ~40 MiB
 *  and a release one ~5 MiB; the placeholder is a few hundred bytes. */
const LEAST_REAL_BYTES = 1024 * 1024;

const argv = process.argv.slice(2);
const has = (flag) => argv.includes(flag);
const valueOf = (flag) => {
  const at = argv.indexOf(flag);
  return at === -1 ? null : argv[at + 1];
};

/**
 * The triple to name the files after.
 *
 * `--target` when the caller passed one. Otherwise `TAURI_ENV_TARGET_TRIPLE`,
 * which Tauri sets for `beforeBuildCommand` — that is how [`verify`] knows
 * which of six files is about to be bundled, since the flag on `tauri build`
 * does not reach this process. Otherwise the host's, read from `rustc` rather
 * than assembled from `process.platform` and `arch`: those two do not know the
 * difference between `-gnu` and `-musl`, and a wrong guess produces a file the
 * bundler silently does not find.
 */
function triple() {
  const named = valueOf('--target') ?? process.env.TAURI_ENV_TARGET_TRIPLE;
  if (named) return named;
  const out = execFileSync('rustc', ['-vV'], { encoding: 'utf8' });
  const line = out.split('\n').find((l) => l.startsWith('host:'));
  if (!line) throw new Error('`rustc -vV` printed no host triple');
  return line.slice('host:'.length).trim();
}

const target = triple();
const exe = target.includes('windows') ? '.exe' : '';
const destination = (name) => join(OUT_DIR, `${name}-${target}${exe}`);

/** Placeholders, so `tauri-build`'s existence check passes and cargo can run. */
function stubs() {
  mkdirSync(OUT_DIR, { recursive: true });
  for (const name of SIDECARS) {
    const to = destination(name);
    if (existsSync(to)) continue;
    writeFileSync(
      to,
      `#!/bin/sh\n` +
        `# Placeholder written by tools/sidecars.mjs so that tauri-build's\n` +
        `# externalBin check passes and cargo can build the real ${name}.\n` +
        `# If you are reading this inside an installed application, the build\n` +
        `# skipped 'npm run sidecars:release' — please report it.\n` +
        `echo "${name}: placeholder, not the real binary" >&2\n` +
        `exit 1\n`
    );
    console.log(`sidecars: placeholder ${to}`);
  }
}

/** Refuse to go on unless both files are real builds. */
function verify() {
  const missing = [];
  for (const name of SIDECARS) {
    const to = destination(name);
    const size = existsSync(to) ? statSync(to).size : 0;
    if (size < LEAST_REAL_BYTES) missing.push(`${to} (${size} bytes)`);
  }
  if (missing.length) {
    console.error(
      `sidecars: these are placeholders, not builds:\n  ${missing.join('\n  ')}\n\n` +
        `Bundling them would ship an application whose \`stackvo\` command is a\n` +
        `shell script that exits 1. Run:\n\n` +
        `  npm run sidecars:release${valueOf('--target') ? ` -- --target ${target}` : ''}\n`
    );
    process.exit(1);
  }
  console.log(`sidecars: verified for ${target}`);
}

/** Build them, then copy them over whatever is there. */
function build({ release }) {
  stubs();

  const profile = release ? 'release' : 'debug';
  const args = ['build', '--manifest-path', MANIFEST];
  if (release) args.push('--release');
  // Only when asked. Passing the host triple explicitly is not a no-op — cargo
  // then writes into `target/<triple>/` instead of `target/`, so doing it
  // unconditionally would move every developer's build directory the first time
  // this ran and rebuild the world.
  const explicit = valueOf('--target');
  if (explicit) args.push('--target', explicit);
  for (const name of SIDECARS) args.push('--bin', name);

  // `--runner`, and it is `tauri build`'s own flag by the same name and for the
  // same reason. Cross-building these two to `*-pc-windows-msvc` from Linux
  // needs `cargo-xwin`, which carries Microsoft's SDK headers and import
  // libraries — plain cargo has no CRT to link against and stops at 101 with
  // nothing readable. `tools/linux/run.sh --windows-bundle` passes the runner
  // to `tauri build` already; without it here, the sidecars fail first and the
  // bundler is never reached.
  const runner = valueOf('--runner') ?? 'cargo';
  console.log(`sidecars: ${runner} ${args.join(' ')}`);
  execFileSync(runner, args, { stdio: 'inherit' });

  for (const name of SIDECARS) {
    const from = join(TARGET_DIR, ...(explicit ? [explicit] : []), profile, `${name}${exe}`);
    if (!existsSync(from)) {
      // cargo exited 0 and the file is not there: a renamed `[[bin]]`, a
      // profile that writes elsewhere. Saying so here is the difference between
      // one clear line and a bundler error about a name nobody chose.
      throw new Error(`cargo reported success but ${from} does not exist`);
    }
    const to = destination(name);
    copyFileSync(from, to);
    console.log(`sidecars: ${to} (${(statSync(to).size / 1024 / 1024).toFixed(1)} MiB)`);
  }

  verify();
}

if (has('--stubs')) stubs();
else if (has('--verify')) verify();
else build({ release: has('--release') });
