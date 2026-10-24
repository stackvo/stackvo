/**
 * A crate that went DOWN a version on this branch, and nothing asked it to.
 *
 * ## The failure this catches
 *
 * `Cargo.lock` was found committed with four crates — rustls, rustls-webpki,
 * tokio, webpki-root-certs — at the versions a security bump had moved them
 * off the day before. Nobody typed those versions. The reflog had the answer:
 * a branch switch in GitHub Desktop wrote the old lockfile, a pull one second
 * later wrote the new one, and an editor's rust-analyzer — woken by the first
 * write — ran `cargo metadata`, which had read the old lockfile, resolved
 * against it, and wrote that resolution back over the new one seven seconds
 * on. The commit that followed took every changed file, as a commit does, and
 * the pre-push hook refused it with "supply · deny FAILED" and "supply ·
 * notice FAILED": true, and not a word about which file had done it.
 *
 * This is that word. It names the crates, says what most likely happened,
 * and gives the one-line fix — before the two checks that can only report the
 * symptom.
 *
 * ## What counts
 *
 * Per crate name, the highest version in the branch's lockfile is compared
 * with the highest at the merge base with `origin/main`. Lower is a
 * downgrade. A lockfile may hold two majors of one crate at once; the
 * highest is the one a bump moves, so it is the one compared.
 *
 * A downgrade with `Cargo.toml` untouched on the branch is reported as an
 * error: no manifest asked for it, so the lockfile is not saying what this
 * branch means. With `Cargo.toml` changed, the same list is printed as
 * information and the run goes on — pinning a crate below a broken release
 * is a manifest change, and that is how an intended downgrade is told from a
 * clobbered file.
 *
 *   node tools/lock-downgrades.mjs                # merge base vs the working tree
 *   node tools/lock-downgrades.mjs <base> <head>  # any two refs
 */
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const LOCK = 'src-tauri/Cargo.lock';
const MANIFEST = 'src-tauri/Cargo.toml';

function git(...args) {
  return execFileSync('git', args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'ignore'],
  });
}

function tryGit(...args) {
  try {
    return git(...args).trim();
  } catch {
    return null;
  }
}

/** The merge base with `origin/main`, or the root commit without a remote. */
function branchBase() {
  return (
    tryGit('merge-base', 'HEAD', 'origin/main') ||
    tryGit('rev-list', '--max-parents=0', 'HEAD')?.split('\n').pop() ||
    null
  );
}

/** `name` → highest version in one lockfile's text. */
function highestVersions(text) {
  const out = new Map();
  for (const block of text.split('[[package]]').slice(1)) {
    const name = /^name = "([^"]+)"/m.exec(block)?.[1];
    const version = /^version = "([^"]+)"/m.exec(block)?.[1];
    if (!name || !version) continue;
    const seen = out.get(name);
    if (!seen || compare(version, seen) > 0) out.set(name, version);
  }
  return out;
}

/** Semver-shaped compare: numeric per dotted part, text where a part is not a number. */
function compare(a, b) {
  const pa = a.split(/[.+-]/);
  const pb = b.split(/[.+-]/);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? '0';
    const y = pb[i] ?? '0';
    const nx = Number(x);
    const ny = Number(y);
    const c = Number.isInteger(nx) && Number.isInteger(ny) ? nx - ny : x.localeCompare(y);
    if (c !== 0) return c < 0 ? -1 : 1;
  }
  return 0;
}

const [baseArg, headArg] = process.argv.slice(2);
const base = baseArg || branchBase();
if (!base) {
  console.log('No base to compare with; nothing to say.');
  process.exit(0);
}

const baseText = tryGit('show', `${base}:${LOCK}`);
if (baseText === null) {
  console.log(`${LOCK} does not exist at ${base}; nothing to compare.`);
  process.exit(0);
}
const headText = headArg
  ? git('show', `${headArg}:${LOCK}`)
  : readFileSync(join(ROOT, LOCK), 'utf8');

const before = highestVersions(baseText);
const after = highestVersions(headText);
const downgrades = [...after]
  .filter(([name, version]) => before.has(name) && compare(version, before.get(name)) < 0)
  .map(([name, version]) => `${name} ${before.get(name)} → ${version}`)
  .sort();

if (downgrades.length === 0) {
  console.log(`No crate is older here than at ${base.slice(0, 7)}.`);
  process.exit(0);
}

const manifestChanged = headArg
  ? tryGit('diff', '--quiet', base, headArg, '--', MANIFEST) === null
  : tryGit('diff', '--quiet', base, '--', MANIFEST) === null;

console.log(`${downgrades.length} crate(s) are older in ${LOCK} than at ${base.slice(0, 7)}:`);
for (const line of downgrades) console.log(`  ${line}`);
console.log('');

if (manifestChanged) {
  console.log(`${MANIFEST} changed on this branch, so this may be meant. Going on.`);
  process.exit(0);
}

console.log(`${MANIFEST} did not change on this branch, so nothing asked for this.`);
console.log('Most likely an editor ran cargo across a branch switch and wrote the old');
console.log('resolution back over the new file. Put the base lockfile back:');
console.log('');
console.log(`  git checkout ${base.slice(0, 7)} -- ${LOCK}`);
console.log('');
console.log(`and commit that. A downgrade that IS meant belongs in ${MANIFEST} as a pin`);
console.log('(`= "x.y.z"`), so the next `cargo update` cannot undo it either.');
process.exit(1);
