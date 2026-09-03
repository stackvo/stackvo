/**
 * SHA-256 for the files a release actually produced, in `sha256sum -c`'s own
 * format — one line per file, the hash then two spaces then the name a person
 * downloads.
 *
 * ## Why not `shasum`
 *
 * `shasum -a 256` is what `release.yml`'s "Publish checksums" step ran until
 * 3 September 2026, and it is the one command in that step that is not on
 * every runner the workflow builds on. macOS and Linux carry the Perl script;
 * GitHub's Windows runners do not — Git for Windows ships `sha256sum`
 * instead, a different program with a different name. Nothing caught this
 * for two rounds because Windows never reached the step: it died earlier, in
 * WiX, on every prior run. The day that stopped being true, `Publish
 * checksums` was the very next thing to fail — `shasum: command not found`,
 * exit code 127, on both Windows rows.
 *
 * The step runs on all three platforms, so it needs one command all three
 * have. The only one that qualifies without adding anything is the
 * interpreter already required to run this file: Node's own `crypto`.
 *
 * ## Where it fits
 *
 * `release.yml` passes it `steps.tauri.outputs.artifactPaths` — a JSON array
 * `tauri-action` prints, absolute and already in the runner's own path
 * separators — through the `ARTIFACT_PATHS` environment variable, which
 * sidesteps the quoting a shell argument would need for a Windows path full
 * of backslashes. Output goes to stdout only; the workflow step still pipes
 * it through `tee` to write the checksums file, exactly as it did before.
 *
 * Run: `ARTIFACT_PATHS='["a.exe","b.dmg"]' node tools/checksum-artifacts.mjs`
 */

import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';

/**
 * One `sha256sum`-format line for the bytes in `buffer`, named `path`.
 *
 * Pure — no filesystem access — so it is the half this file's own tests
 * exercise without a real installer to hash. `basename`, not the runner's
 * absolute path: the person checking a checksum has the file in their
 * downloads folder, not in `D:\a\stackvo\stackvo\...`.
 */
export function checksumLine(path, buffer) {
  const hash = createHash('sha256').update(buffer).digest('hex');
  // Not `node:path`'s `basename`: it picks POSIX or Windows rules from the
  // *host* running this script, not from the path it is given. That is right
  // for the CLI below — each matrix row runs this on its own OS, against
  // paths from that same OS — and wrong for a path copied out of a Windows
  // CI log and handed to this function from a Mac, which `ci.yml`'s own
  // `vitest` step does every time it runs this file's tests on
  // `ubuntu-latest` and `macos-latest`. Splitting on both separators is
  // correct everywhere a Windows path's backslash could appear, host included.
  const name = path.split(/[/\\]/).pop();
  return `${hash}  ${name}`;
}

/**
 * `ARTIFACT_PATHS` parsed, or a message that says what was wrong with it.
 *
 * A malformed value here is a bug in the step that sets the variable, not in
 * the file it names — and `JSON.parse`'s own error names neither.
 */
export function parseArtifactPaths(json) {
  if (!json) throw new Error('ARTIFACT_PATHS is not set');
  const parsed = JSON.parse(json);
  if (!Array.isArray(parsed)) {
    throw new Error(`ARTIFACT_PATHS is not a JSON array: ${json}`);
  }
  return parsed;
}

function main() {
  const paths = parseArtifactPaths(process.env.ARTIFACT_PATHS);
  for (const path of paths) {
    // Silent, matching the bash loop this replaces: `tauri-action`'s list can
    // name a format a rehearsal skipped (`--bundles nsis` on Windows leaves no
    // `.msi` in `artifactPaths` to begin with, so this is not expected to
    // trigger there) — but a path that is simply absent is not this script's
    // question to raise.
    if (!existsSync(path)) continue;
    console.log(checksumLine(path, readFileSync(path)));
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
