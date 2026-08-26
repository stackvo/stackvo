/**
 * Did this target actually produce installers, and are they for this target?
 *
 * ```sh
 * npm run installers:check -- --target aarch64-unknown-linux-gnu
 * npm run installers:check -- --target aarch64-pc-windows-msvc --unsigned
 * ```
 *
 * ## Why this exists
 *
 * §3 #22 is two rows of the release matrix — `aarch64-unknown-linux-gnu` on
 * `ubuntu-24.04-arm`, `aarch64-pc-windows-msvc` on `windows-11-arm` — and its
 * remaining question was never "does it compile there". That half was answered
 * by a real run: both jobs started, built and ran the suite. The open half is
 * **does the bundler produce a package on that architecture**, and nothing in
 * this repository could say. The rehearsal uploaded `bundle/` to the run page
 * for a person to download and look at, which is an answer somebody has to go
 * and fetch, once, by hand — the shape of check that is not performed again.
 *
 * ## What it refuses to call working
 *
 *   * **a format that is missing.** `bundle.targets` is `"all"`, so Linux owes
 *     a `.deb`, an `.rpm` and an `.AppImage`, and Windows owes a `.msi` and an
 *     NSIS `-setup.exe`. These are separate bundlers with separate native
 *     tools: the AppImage one downloads `linuxdeploy-aarch64.AppImage` and runs
 *     it, the MSI one runs WiX under x86 emulation. Either can be the one that
 *     is unhappy on ARM while the others are fine, and a directory that merely
 *     *exists* hides exactly that.
 *   * **a package named for the wrong architecture.** Every bundler writes the
 *     architecture into the file name — and writes it in its own vocabulary:
 *     the same aarch64 build is `arm64` to dpkg, `aarch64` to rpm and AppImage,
 *     `arm64` to WiX and NSIS, `aarch64` to the dmg. A cross-compile that
 *     silently fell back to the host, or a matrix row whose `--target` never
 *     reached the bundler, produces a green job and an x86 installer with an
 *     ARM release's name on it. This is the check #22 is actually asking for.
 *   * **an updater artifact with no signature**, unless the caller says the run
 *     was deliberately unsigned. An unsigned bundle installs by hand and is
 *     invisible to the updater — the failure that looks most like success, and
 *     the same one `check-updater-endpoint.mjs` catches one step later.
 *
 * ## What it deliberately does not do
 *
 * It does not open the packages. Whether the `.deb` unpacks to a working
 * application is a question for a machine of that architecture with a desktop
 * on it, and this repository has no such machine. The claim here is narrower
 * and it is the claim #22 needs: the bundler ran, on that architecture, and
 * produced each thing it owes.
 */

import { readdirSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

/**
 * What each platform owes, and what its bundler calls this architecture.
 *
 * The `arch` maps are not a style choice and cannot be collapsed into one: they
 * are what six different bundlers independently decided to write into a file
 * name, read out of `tauri-bundler` rather than guessed. `deb` says `arm64`
 * because that is dpkg's word; `rpm` says `aarch64` because that is rpm's. A
 * single table here would be a seventh opinion, and the one nobody validates.
 */
export const FORMATS = {
  deb: { os: 'linux', ext: '.deb', arch: { aarch64: 'arm64', x86_64: 'amd64' } },
  rpm: { os: 'linux', ext: '.rpm', arch: { aarch64: 'aarch64', x86_64: 'x86_64' } },
  appimage: { os: 'linux', ext: '.AppImage', arch: { aarch64: 'aarch64', x86_64: 'amd64' } },
  dmg: { os: 'macos', ext: '.dmg', arch: { aarch64: 'aarch64', x86_64: 'x64' } },
  msi: { os: 'windows', ext: '.msi', arch: { aarch64: 'arm64', x86_64: 'x64' } },
  nsis: { os: 'windows', ext: '-setup.exe', arch: { aarch64: 'arm64', x86_64: 'x64' } },
};

/**
 * The artifact the updater downloads, per platform.
 *
 * Only these have to carry a `.sig`. The bundler signs more than this — it
 * signs the `.deb` and the `.rpm` too — but nothing reads those signatures, so
 * demanding them here would be this file inventing a requirement rather than
 * checking one. What the updater cannot live without is on the left of
 * `check-updater-endpoint.mjs`'s platform table, and it is this.
 */
export const UPDATED = {
  linux: '.AppImage',
  windows: '-setup.exe',
  macos: '.app.tar.gz',
};

/** The OS half of a target triple, in the words `FORMATS` uses. */
export function osOf(triple) {
  if (triple.includes('-linux-')) return 'linux';
  if (triple.includes('-windows-')) return 'windows';
  if (triple.includes('-darwin')) return 'macos';
  throw new Error(
    `no idea what platform \`${triple}\` is. Every target in the release matrix has to be ` +
      `known here, because an unknown one would otherwise be checked against nothing and pass.`
  );
}

/** The architecture half, in the words `FORMATS` uses. */
export function archOf(triple) {
  const arch = triple.split('-')[0];
  if (arch === 'aarch64' || arch === 'x86_64') return arch;
  throw new Error(
    `\`${triple}\` builds for \`${arch}\`, and this checker only knows what aarch64 and x86_64 ` +
      `installers are called. Add it to FORMATS in tools/check-installers.mjs.`
  );
}

/** Every file under `dir`, as paths relative to it. Empty when it is not there. */
export function collect(dir) {
  const out = [];
  const walk = (at) => {
    let entries;
    try {
      entries = readdirSync(at, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const path = join(at, entry.name);
      // `.app` is a directory and it is one of the things owed, so the walk
      // stops at it rather than descending into a few thousand resources.
      if (entry.isDirectory() && !entry.name.endsWith('.app')) walk(path);
      else out.push(relative(dir, path).split(sep).join('/'));
    }
  };
  walk(dir);
  return out;
}

/**
 * Everything wrong with what a target produced, as sentences.
 *
 * A list rather than a throw, for the same reason the release suite runs
 * `--no-fail-fast`: a run that reports the first missing format and stops is a
 * run somebody has to spend again to learn the second.
 */
export function inspect(files, { triple, signed = true }) {
  const os = osOf(triple);
  const arch = archOf(triple);
  const problems = [];

  const owed = Object.entries(FORMATS).filter(([, format]) => format.os === os);
  for (const [name, format] of owed) {
    // The updater's copies live in their own folders (`nsis-updater`, and the
    // WiX one beside it) and carry the same extension. They are not the
    // installer, and counting them as one would let a run pass with no
    // installer at all.
    const found = files.filter((file) => file.endsWith(format.ext) && !file.includes('-updater/'));
    if (found.length === 0) {
      problems.push(
        `${triple} produced no ${name} package (nothing ending in \`${format.ext}\`). ` +
          `\`bundle.targets\` is "all", so this platform owes one — and each format is a ` +
          `separate bundler with its own native tools, so this one being absent says nothing ` +
          `about the others being present.`
      );
      continue;
    }

    const expected = format.arch[arch];
    const wrong = found.filter((file) => !file.includes(expected));
    if (wrong.length) {
      problems.push(
        `${wrong.join(', ')} does not carry \`${expected}\` in its name, and the ${name} ` +
          `bundler writes the architecture it built for into the file name. This target is ` +
          `${triple}, so either the bundler built for the host instead of the target, or this ` +
          `is a package from an earlier run of a different one.`
      );
    }
  }

  if (signed) {
    const updated = UPDATED[os];
    const artifacts = files.filter((file) => file.endsWith(updated));
    if (artifacts.length === 0) {
      problems.push(
        `${triple} produced nothing ending in \`${updated}\`, which is what the updater ` +
          `downloads on this platform. Without it the entry for this platform in latest.json ` +
          `has no file to point at.`
      );
    }
    for (const artifact of artifacts) {
      if (!files.includes(`${artifact}.sig`)) {
        problems.push(
          `${artifact} has no \`.sig\` beside it. The updater refuses an unsigned bundle, and ` +
            `it refuses it on the user's machine rather than here — the artifact installs by ` +
            `hand and is invisible to the updater.`
        );
      }
    }
  }

  return problems;
}

/** `--target <value>`, or nothing. */
function valueOf(flag) {
  const at = process.argv.indexOf(flag);
  return at === -1 ? undefined : process.argv[at + 1];
}

function main() {
  const triple = valueOf('--target');
  if (!triple) {
    console.error(
      'usage: node tools/check-installers.mjs --target <triple> [--unsigned] [--dir <path>]'
    );
    process.exit(1);
  }

  const dir = valueOf('--dir') ?? `src-tauri/target/${triple}/release/bundle`;
  const files = collect(dir);
  const signed = !process.argv.includes('--unsigned');

  console.log(`reading ${dir}`);
  if (files.length === 0) {
    // Said separately from "a format is missing", because it is a different
    // failure: the bundler never ran at all, and every per-format sentence
    // below would be a way of saying that same thing six times.
    console.error(`\nnothing is there. The bundler did not run, or it wrote somewhere else.`);
    process.exit(1);
  }
  for (const file of files) console.log(`  ${file}`);

  const problems = inspect(files, { triple, signed });
  if (problems.length) {
    console.error(`\n${triple} did not produce what it owes:\n`);
    for (const problem of problems) console.error(`  · ${problem}\n`);
    process.exit(1);
  }

  const owed = Object.entries(FORMATS)
    .filter(([, format]) => format.os === osOf(triple))
    .map(([name]) => name);
  console.log(`\n${triple}: ${owed.join(', ')} — each present and named for ${archOf(triple)}.`);
  if (!signed) console.log('Signatures were not checked: this run was told it is unsigned.');
}

// Importable for the tests without touching a bundle directory.
if (process.argv[1] && process.argv[1].endsWith('check-installers.mjs')) main();
