/**
 * Does the update endpoint answer, and is what it answers usable?
 *
 * ```sh
 * npm run updates:check                 # the endpoint in tauri.conf.json
 * npm run updates:check -- --url <url>  # a draft's asset, before publishing
 * ```
 *
 * ## Why this exists
 *
 * §3 #2 has been "half done" through three rounds, and every round the sentence
 * was some version of *the keys are in place, the workflow ran, and
 * `latest.json` is still 404.* Nothing in this repository could say more than
 * that, because nothing here had ever **asked the endpoint**. The Rust tests
 * check that the URL is spelled the way the workflow publishes and that the
 * flag writing the file is still set — both true today, and both true while the
 * endpoint answers 404.
 *
 * The gap between those two claims is where every remaining failure of this
 * item lives, and it is one HTTP request wide.
 *
 * ## What it refuses to call working
 *
 * A 200 is not the answer. The updater reads this file and then decides, from
 * its contents, whether a running application has an update — so a manifest
 * that parses and names four of six platforms is a manifest that silently tells
 * two of them they are current forever. Each check below is a way that has
 * actually happened to somebody:
 *
 *   * **404** — the release is a draft. `releases/latest/download/` resolves to
 *     the latest *published*, non-prerelease release and never to a draft, so a
 *     fully green run still leaves this 404 until a person presses Publish.
 *     That step is nowhere in this repository's prose, and it is the reason a
 *     green pipeline was read as a broken endpoint.
 *   * **a missing platform** — six jobs write this one file. Whether they merge
 *     or overwrite is `tauri-action`'s business and not observable from here;
 *     what is observable is the result, and the result is the only thing that
 *     matters to the machine that will read it.
 *   * **an empty signature** — the updater refuses an unsigned bundle, which is
 *     correct and arrives on the user's machine rather than here. An artifact
 *     built without `TAURI_SIGNING_PRIVATE_KEY` is installable by hand and
 *     invisible to the updater; it is the failure that looks most like success.
 *   * **a version that is not ahead** — a release published from a tag that
 *     does not match `tauri.conf.json` offers every user an update to the
 *     version they are already running.
 *
 * ## No network, no verdict
 *
 * A checker that cannot reach the endpoint says so and exits 2. Green on
 * "could not ask" is the failure mode this file was written against.
 */

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const read = (relative) => readFileSync(join(ROOT, relative), 'utf8');

/** `--url <value>`, or nothing. */
function valueOf(flag) {
  const at = process.argv.indexOf(flag);
  return at === -1 ? undefined : process.argv[at + 1];
}

/** The declared endpoint, and the version the application says it is. */
export function declared(conf) {
  const endpoints = conf.plugins?.updater?.endpoints ?? [];
  return { endpoint: endpoints[0], version: conf.version };
}

/**
 * The platform keys the updater will look itself up under.
 *
 * Derived from the release matrix rather than listed here, because a seventh
 * target added to the workflow has to widen this check on its own — a hand
 * -written list is the copy that goes stale, and the way it goes stale is by
 * passing.
 */
export function platformsFrom(workflow) {
  const KEYS = {
    'aarch64-apple-darwin': 'darwin-aarch64',
    'x86_64-apple-darwin': 'darwin-x86_64',
    'x86_64-unknown-linux-gnu': 'linux-x86_64',
    'aarch64-unknown-linux-gnu': 'linux-aarch64',
    'x86_64-pc-windows-msvc': 'windows-x86_64',
    'aarch64-pc-windows-msvc': 'windows-aarch64',
  };

  const targets = [...workflow.matchAll(/^\s*-?\s*target:\s*(\S+)\s*$/gm)].map((m) => m[1]);
  const unknown = targets.filter((t) => !(t in KEYS));
  if (unknown.length) {
    throw new Error(
      `the release matrix builds ${unknown.join(', ')} and this checker does not know what ` +
        `the updater calls that platform. Add it to KEYS in tools/check-updater-endpoint.mjs — ` +
        `a target nobody checks is a target whose users never get an update.`
    );
  }
  return [...new Set(targets.map((t) => KEYS[t]))];
}

/**
 * Everything wrong with a manifest, as sentences.
 *
 * Pure, and separated from the fetch for one reason: this is the half that can
 * be tested, and `tests/updater-manifest.spec.js` runs it against manifests
 * that have each of these faults. A checker whose judgement is only exercised
 * by the thing it checks is a checker nobody knows the shape of.
 */
export function inspect(manifest, { platforms, version }) {
  const problems = [];

  if (!manifest || typeof manifest !== 'object') {
    return ['the endpoint answered with something that is not a JSON object'];
  }

  if (!manifest.version) {
    problems.push('the manifest names no version, so the updater cannot compare anything');
  } else if (version && manifest.version.replace(/^v/, '') !== version) {
    problems.push(
      `the manifest offers ${manifest.version} and this checkout is ${version}. ` +
        `A release published from a tag that does not match tauri.conf.json offers ` +
        `every user an update to the version they are already running.`
    );
  }

  const found = manifest.platforms ?? {};
  const missing = platforms.filter((key) => !found[key]);
  if (missing.length) {
    problems.push(
      `no entry for ${missing.join(', ')} — the release matrix builds ${platforms.length} ` +
        `platforms and this manifest carries ${Object.keys(found).length}. Those users are ` +
        `told they are current, forever, with no error anywhere.`
    );
  }

  for (const key of platforms) {
    const entry = found[key];
    if (!entry) continue;
    if (!entry.url) problems.push(`${key} has no url`);
    if (!entry.signature) {
      problems.push(
        `${key} carries an empty signature. The updater refuses an unsigned bundle, and it ` +
          `refuses it on the user's machine rather than here — this is the failure that ` +
          `looks most like success.`
      );
    }
  }

  return problems;
}

/** The diagnosis for a status code, because 404 here has one likely cause. */
export function diagnose(status, endpoint) {
  if (status === 404) {
    return (
      `404 — nothing is served at ${endpoint}.\n\n` +
      `Two things cause this and they look identical from here, so check both in order:\n\n` +
      `1. THE FILE IS NOT THERE. \`tauri-action\` writes latest.json only after the build ` +
      `succeeds, so a matrix that failed leaves a release with no assets on it. Open the ` +
      `release and count them.\n` +
      `2. THE RELEASE IS A DRAFT. \`releases/latest/download/\` resolves to the latest ` +
      `PUBLISHED, non-prerelease release and never to a draft, and \`release.yml\` creates ` +
      `releases with \`releaseDraft: true\`. This one is worth knowing because it survives a ` +
      `fully green run: six green targets, every artifact built and signed, and this URL still ` +
      `404 until somebody presses Publish.\n\n` +
      `To check a draft's manifest before publishing, take the asset URL off the release page:\n` +
      `  npm run updates:check -- --url <that url>`
    );
  }
  return `${status} — the endpoint did not answer with the manifest`;
}

async function main() {
  const conf = JSON.parse(read('src-tauri/tauri.conf.json'));
  const { endpoint: configured, version } = declared(conf);
  const endpoint = valueOf('--url') ?? configured;

  if (!endpoint) {
    console.error('tauri.conf.json declares no updater endpoint.');
    process.exit(1);
  }

  const platforms = platformsFrom(read('.github/workflows/release.yml'));
  console.log(`asking ${endpoint}`);
  console.log(`expecting ${version} for ${platforms.join(', ')}\n`);

  let response;
  try {
    response = await fetch(endpoint, { redirect: 'follow' });
  } catch (error) {
    // Exit 2, and deliberately not 1: "the endpoint is wrong" and "nobody could
    // ask" are different answers, and a caller that treats them the same is how
    // an offline run gets read as a passing one.
    console.error(`could not reach the endpoint: ${error.message}`);
    console.error('\nThis is not a verdict on the endpoint. Nothing was measured.');
    process.exit(2);
  }

  if (!response.ok) {
    console.error(diagnose(response.status, endpoint));
    process.exit(1);
  }

  let manifest;
  try {
    manifest = await response.json();
  } catch {
    console.error('the endpoint answered, and what it answered is not JSON');
    process.exit(1);
  }

  const problems = inspect(manifest, { platforms, version });
  if (problems.length) {
    console.error(`the manifest is served and the updater cannot use it:\n`);
    for (const problem of problems) console.error(`  · ${problem}\n`);
    process.exit(1);
  }

  console.log(`the endpoint serves ${manifest.version} for all ${platforms.length} platforms,`);
  console.log('each with a url and a signature.');
}

// Importable for the tests without running the fetch.
if (process.argv[1] && process.argv[1].endsWith('check-updater-endpoint.mjs')) await main();
