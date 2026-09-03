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
 *
 * ## What the first version got wrong, twice
 *
 * Release run #6 was the first with every row past bundling, and this file
 * failed it in two different ways on two different platforms: `EISDIR` on
 * macOS, where `artifactPaths` includes the `.app` *directory*, and a silent
 * empty file on Windows, where the entry-point check never matched. Both
 * were platform-specific behaviour the tests did not exercise, on a file
 * whose whole reason to exist was platform independence. `isRegularFile`,
 * `checksumLines` and `isEntryPoint` below each carry the fix and the test
 * that would have caught it — `ci.yml` runs this file's tests on
 * `windows-latest` too, so the round trip is now checked on the OS it broke on.
 */

import { createHash } from 'node:crypto';
import { readFileSync, statSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

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

/**
 * Whether `path` is something a person can download and check — a regular
 * file — as opposed to a directory or nothing at all.
 *
 * This is the `[ -f "$path" ]` of the bash loop this file replaced, and the
 * first version of this file dropped it for `existsSync`. On macOS that was
 * the difference between passing and failing: `tauri-action` lists
 * `StackVo.app` in `artifactPaths` alongside the `.dmg`, and a `.app` is a
 * directory. `existsSync` said yes, `readFileSync` said `EISDIR`, and both
 * macOS rows of release run #6 went red at the very last step.
 */
export function isRegularFile(path) {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

/**
 * One checksum line per regular file in `paths`, in order; directories and
 * absent paths skipped.
 *
 * Throws when that leaves nothing, on purpose. Every matrix row builds at
 * least one installer, so an empty list means the step is misconfigured, not
 * that there was nothing to hash — and a checksums file with zero lines in it
 * is a release asset that lies. Release run #6 published exactly that for
 * both Windows rows, green (see `isEntryPoint`).
 */
export function checksumLines(paths) {
  const lines = paths.filter(isRegularFile).map((path) => checksumLine(path, readFileSync(path)));
  if (lines.length === 0) {
    throw new Error(`ARTIFACT_PATHS named no regular file to checksum: ${JSON.stringify(paths)}`);
  }
  return lines;
}

/**
 * Whether this module is the script Node was told to run, rather than an
 * import — so the tests can load `checksumLine` without `main` firing.
 *
 * Compared as URLs, not by pasting `file://` in front of `argv[1]`. The
 * pasted form is what this file first did, and it is only right on POSIX:
 * on Windows `argv[1]` is `D:\a\stackvo\...`, its URL is
 * `file:///D:/a/stackvo/...`, and the two never match — so `main` never ran,
 * nothing was printed, and the step passed with an empty checksums file on
 * both Windows rows of release run #6. `pathToFileURL` builds the URL the
 * way the module loader does, on every OS.
 */
export function isEntryPoint(argv1, moduleUrl) {
  return argv1 !== undefined && pathToFileURL(argv1).href === moduleUrl;
}

function main() {
  const paths = parseArtifactPaths(process.env.ARTIFACT_PATHS);
  for (const line of checksumLines(paths)) console.log(line);
}

if (isEntryPoint(process.argv[1], import.meta.url)) {
  main();
}
