/**
 * The beta channel's pointer: which release `beta.json` names, and what goes
 * in it.
 *
 * ```sh
 * node tools/beta-manifest.mjs newest releases.json        # prints a tag
 * node tools/beta-manifest.mjs stamp latest.json v0.3.0-beta.1 > beta.json
 * ```
 *
 * ## Why a pointer has to be maintained by hand
 *
 * The stable channel reads `releases/latest/download/latest.json`, and GitHub
 * keeps `releases/latest` pointing at the newest published non-prerelease on
 * its own. There is no such pointer for pre-releases, so `release.yml`'s
 * `channel` job keeps one: a release tagged `beta`, holding one `beta.json`
 * that is replaced every time a release is published. This file is the two
 * decisions that job makes, kept out of the YAML so they can be tested.
 *
 * ## The two decisions
 *
 * **Which release.** The newest *published* one by semver precedence — stable
 * or pre-release, whichever is later. Not "the pre-release that was just
 * published", and the difference is the whole point: the updater plugin stops
 * at the first endpoint that answers, and a beta install asks `beta.json`
 * before `latest.json`. A `beta.json` still naming `0.3.0-beta.1` after
 * `0.3.0` shipped would hide the stable release from every beta install until
 * the next beta. So a stable publish refreshes the pointer too, and the
 * pointer then names the stable release under its own `channel: stable`.
 *
 * Drafts are never candidates: a draft's assets cannot be downloaded, so a
 * pointer at one offers a file nobody can fetch. The rolling `beta` release
 * itself is not a candidate either, nor is anything not tagged like a version.
 *
 * **What the file says.** That release's own `latest.json`, which
 * `tauri-action` wrote and signed, with `channel` stamped in — `beta` for a
 * pre-release, `stable` otherwise — so `channel.rs` can refuse it to installs
 * that did not ask. Stamped, and checked first: a version that matches the
 * tag, and a URL plus a non-empty signature for every platform the release
 * matrix builds. The pointer never moves to a manifest the updater cannot
 * use, whatever the release page looks like.
 */

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { inspect, platformsFrom } from './check-updater-endpoint.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/** The tag of the rolling release; `channel.rs` names the same one. */
export const ROLLING_TAG = 'beta';

/** A version tag: `v` plus something semver-shaped. */
const VERSION_TAG = /^v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$/;

/** A pre-release is a version with a hyphen — semver's own marker. */
export const isPrerelease = (tag) => /^v?\d+\.\d+\.\d+-/.test(tag);

/**
 * Semver precedence, as the specification orders it.
 *
 * The three numbers first; then no pre-release beats a pre-release; then the
 * pre-release identifiers one by one — numbers as numbers and lower than any
 * word, words as strings, a shorter list that matches so far is lower. Build
 * metadata is ignored. Written out rather than taken from the `semver` package
 * so this tool needs no `npm ci` on the runner, and mirrored in `channel.rs`,
 * which orders the same versions on the user's machine.
 */
export function compare(a, b) {
  const parse = (v) => {
    const [body] = String(v).trim().replace(/^v/, '').split('+');
    const at = body.indexOf('-');
    const core = (at === -1 ? body : body.slice(0, at)).split('.').map((n) => Number(n) || 0);
    const pre = at === -1 ? [] : body.slice(at + 1).split('.');
    return { core, pre };
  };
  const x = parse(a);
  const y = parse(b);

  for (let i = 0; i < Math.max(x.core.length, y.core.length); i++) {
    const d = (x.core[i] ?? 0) - (y.core[i] ?? 0);
    if (d !== 0) return Math.sign(d);
  }
  if (x.pre.length === 0 || y.pre.length === 0) {
    return Math.sign(y.pre.length - x.pre.length);
  }
  for (let i = 0; i < Math.min(x.pre.length, y.pre.length); i++) {
    const p = x.pre[i];
    const q = y.pre[i];
    const pn = /^\d+$/.test(p);
    const qn = /^\d+$/.test(q);
    if (pn && qn) {
      const d = Number(p) - Number(q);
      if (d !== 0) return Math.sign(d);
    } else if (pn !== qn) {
      return pn ? -1 : 1;
    } else if (p !== q) {
      return p < q ? -1 : 1;
    }
  }
  return Math.sign(x.pre.length - y.pre.length);
}

/**
 * The tag `beta.json` should name, out of `gh release list --json
 * tagName,isDraft,isPrerelease`. `null` when nothing qualifies.
 */
export function newest(releases) {
  const candidates = (Array.isArray(releases) ? releases : []).filter(
    (r) => r && !r.isDraft && typeof r.tagName === 'string' && VERSION_TAG.test(r.tagName)
  );
  if (!candidates.length) return null;
  return candidates.reduce((best, r) => (compare(r.tagName, best.tagName) > 0 ? r : best)).tagName;
}

/**
 * `latest.json` from `tag`, stamped with its channel — or the reasons it must
 * not be published, as sentences.
 */
export function stamp(manifest, tag, { platforms }) {
  const version = String(tag).replace(/^v/, '');
  const problems = inspect(manifest, { platforms, version, channel: 'beta' });
  if (problems.length) return { problems };

  const channel = isPrerelease(tag) ? 'beta' : 'stable';
  return { manifest: { ...manifest, channel } };
}

function main() {
  const [mode, file, tag] = process.argv.slice(2);

  if (mode === 'newest' && file) {
    const tag = newest(JSON.parse(readFileSync(file, 'utf8')));
    if (!tag) {
      console.error(
        'no published release is tagged like a version, so there is nothing to point at'
      );
      process.exit(1);
    }
    console.log(tag);
    return;
  }

  if (mode === 'stamp' && file && tag) {
    const workflow = readFileSync(join(ROOT, '.github/workflows/release.yml'), 'utf8');
    const manifest = JSON.parse(readFileSync(file, 'utf8'));
    const result = stamp(manifest, tag, { platforms: platformsFrom(workflow) });
    if (result.problems) {
      console.error(`${tag}'s manifest must not become the beta channel's pointer:\n`);
      for (const problem of result.problems) console.error(`  · ${problem}\n`);
      process.exit(1);
    }
    console.log(JSON.stringify(result.manifest, null, 2));
    return;
  }

  console.error('usage: beta-manifest.mjs newest <releases.json> | stamp <latest.json> <tag>');
  process.exit(2);
}

// Importable for the tests without running anything.
if (process.argv[1] && process.argv[1].endsWith('beta-manifest.mjs')) main();
