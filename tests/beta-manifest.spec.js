import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { compare, isPrerelease, newest, stamp, ROLLING_TAG } from '../tools/beta-manifest.mjs';

/**
 * The beta channel's pointer, decided without a runner.
 *
 * `release.yml`'s `channel` job replaces `beta.json` on the rolling `beta`
 * release every time somebody presses Publish. Which release it names and what
 * the file says are the two decisions in `tools/beta-manifest.mjs`, and each
 * has a way of being wrong that the release page would not show:
 *
 *   * a pointer that follows the *latest publish* rather than the *newest
 *     version* hides a stable release from every beta install the day the
 *     stable ships after its own beta — the plugin stops at the first endpoint
 *     that answers;
 *   * a pointer at a draft offers a file nobody can download;
 *   * a pointer at a manifest missing a platform tells that platform's beta
 *     installs they are current, for ever.
 */

const ROOT = resolve(import.meta.dirname, '..');
const PLATFORMS = ['darwin-aarch64', 'linux-x86_64'];

/** A manifest `tauri-action` would have written for `version`. */
function manifest(version) {
  return {
    version,
    notes: '',
    pub_date: '2026-09-04T00:00:00Z',
    platforms: Object.fromEntries(
      PLATFORMS.map((key) => [key, { signature: 'dW50cnVzdGVk…', url: `https://x/${key}` }])
    ),
  };
}

describe('semver precedence', () => {
  it('puts a release after its own pre-releases', () => {
    // The mistake a "split on dots and compare numbers" version makes, and
    // the one `channel.rs` had: 0.3.0-beta.1 read as newer than 0.3.0.
    expect(compare('0.3.0', '0.3.0-beta.1')).toBe(1);
    expect(compare('v0.3.0-beta.1', 'v0.3.0')).toBe(-1);
    expect(compare('0.3.0-beta.1', '0.2.0')).toBe(1);
  });

  it('orders pre-release identifiers the way the specification says', () => {
    expect(compare('0.3.0-beta.2', '0.3.0-beta.1')).toBe(1);
    expect(compare('0.3.0-beta.10', '0.3.0-beta.9')).toBe(1);
    expect(compare('0.3.0-rc.1', '0.3.0-beta.9')).toBe(1);
    expect(compare('0.3.0-beta', '0.3.0-1')).toBe(1);
    expect(compare('0.3.0-beta.1', '0.3.0-beta')).toBe(1);
    expect(compare('0.3.0-beta.1', '0.3.0-beta.1')).toBe(0);
    expect(compare('0.3.0+build.7', 'v0.3.0')).toBe(0);
    expect(compare('0.10.0', '0.9.0')).toBe(1);
  });

  it('knows a pre-release by its hyphen', () => {
    expect(isPrerelease('v0.3.0-beta.1')).toBe(true);
    expect(isPrerelease('v0.3.0-rc.1')).toBe(true);
    expect(isPrerelease('v0.3.0')).toBe(false);
    expect(isPrerelease('0.3.0+build')).toBe(false);
  });
});

describe('which release beta.json names', () => {
  const release = (tagName, extra = {}) => ({
    tagName,
    isDraft: false,
    isPrerelease: false,
    ...extra,
  });

  it('is the newest by version, not the one that was just published', () => {
    // A 0.2.1 hotfix published after 0.3.0-beta.1 must not drag the pointer
    // backwards; and the list order is GitHub's, which is by date.
    expect(
      newest([
        release('v0.2.1'),
        release('v0.3.0-beta.1', { isPrerelease: true }),
        release('v0.2.0'),
      ])
    ).toBe('v0.3.0-beta.1');
  });

  it('moves to a stable release that supersedes the newest beta', () => {
    // The load-bearing case: the plugin stops at the first endpoint that
    // answers, so a pointer still at 0.3.0-beta.1 would hide 0.3.0 from every
    // beta install until the next beta.
    expect(newest([release('v0.3.0-beta.2', { isPrerelease: true }), release('v0.3.0')])).toBe(
      'v0.3.0'
    );
  });

  it('never names a draft', () => {
    expect(newest([release('v0.4.0', { isDraft: true }), release('v0.3.0')])).toBe('v0.3.0');
  });

  it('ignores the rolling release itself and anything not tagged like a version', () => {
    expect(
      newest([release(ROLLING_TAG, { isPrerelease: true }), release('nightly'), release('v0.3.0')])
    ).toBe('v0.3.0');
    expect(newest([release(ROLLING_TAG, { isPrerelease: true })])).toBeNull();
    expect(newest([])).toBeNull();
    expect(newest(undefined)).toBeNull();
  });
});

describe('what beta.json says', () => {
  it('is that release’s own manifest with the channel stamped in', () => {
    const source = manifest('0.3.0-beta.1');
    const { manifest: stamped, problems } = stamp(source, 'v0.3.0-beta.1', {
      platforms: PLATFORMS,
    });

    expect(problems).toBeUndefined();
    expect(stamped.channel).toBe('beta');
    expect(stamped.platforms).toEqual(source.platforms);
    expect(stamped.version).toBe('0.3.0-beta.1');
    // Stamped onto a copy. The file tauri-action signed is left as it is.
    expect(source.channel).toBeUndefined();
  });

  it('marks a stable release as stable, so the pointer at it is honest', () => {
    const { manifest: stamped } = stamp(manifest('0.3.0'), 'v0.3.0', { platforms: PLATFORMS });
    expect(stamped.channel).toBe('stable');
  });

  it('refuses a manifest that does not name the tag it came from', () => {
    const { problems } = stamp(manifest('0.2.0'), 'v0.3.0', { platforms: PLATFORMS });
    expect(problems.join(' ')).toContain('offers 0.2.0');
  });

  it('refuses a manifest missing a platform the matrix builds', () => {
    const partial = manifest('0.3.0-beta.1');
    delete partial.platforms['linux-x86_64'];
    const { problems, manifest: stamped } = stamp(partial, 'v0.3.0-beta.1', {
      platforms: PLATFORMS,
    });
    expect(stamped).toBeUndefined();
    expect(problems.join(' ')).toContain('linux-x86_64');
  });
});

describe('the workflow and this tool agree', () => {
  const workflow = readFileSync(resolve(ROOT, '.github/workflows/release.yml'), 'utf8');

  it('calls the two modes this file exports, in the channel job', () => {
    const job = workflow.slice(workflow.indexOf('\n  channel:'));
    expect(job).toContain('node tools/beta-manifest.mjs newest');
    expect(job).toContain('node tools/beta-manifest.mjs stamp');
    // Fed the list the tool filters on. Without `isDraft` a draft is a
    // candidate, and a pointer at a draft offers a file nobody can download.
    expect(job).toMatch(/gh release list .*--json [^\n]*isDraft/);
  });

  it('publishes to the release this file names, and never as latest', () => {
    expect(workflow).toContain(`gh release upload ${ROLLING_TAG} beta.json --clobber`);
    const create = workflow
      .split('\n')
      .find((line) => line.includes(`gh release create ${ROLLING_TAG}`));
    expect(create, 'the rolling release is created by the workflow').toBeDefined();
    expect(create).toContain('--prerelease');
  });
});
