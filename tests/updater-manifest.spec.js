import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { inspect, diagnose, platformsFrom, declared } from '../tools/check-updater-endpoint.mjs';

/**
 * The judgement half of `npm run updates:check`, exercised without a network.
 *
 * The tool asks the live endpoint, so nothing about it can be a build gate —
 * a check that fails when GitHub is slow, or before the first release exists,
 * is a check people learn to ignore. What can be gated is whether it would
 * *recognise* a broken manifest, and that is this file.
 *
 * The four manifests below are the four ways the update endpoint could still be broken after
 * the release page looks finished, and every one of them serves a 200.
 */

const ROOT = resolve(import.meta.dirname, '..');
const PLATFORMS = ['darwin-aarch64', 'linux-x86_64', 'windows-x86_64'];

/** A manifest with nothing wrong with it. */
function good() {
  return {
    version: '0.1.0',
    pub_date: '2026-08-25T00:00:00Z',
    platforms: Object.fromEntries(
      PLATFORMS.map((key) => [key, { signature: 'dW50cnVzdGVk…', url: `https://x/${key}` }])
    ),
  };
}

describe('what the updater manifest has to carry', () => {
  it('passes a manifest that is complete', () => {
    expect(inspect(good(), { platforms: PLATFORMS, version: '0.1.0' })).toEqual([]);
  });

  /**
   * The one the release page cannot show you.
   *
   * Six jobs write this one file. Whether `tauri-action` merges them or the
   * last one wins is its business and not observable from here; what is
   * observable is the result, and a platform missing from it is not an error
   * anywhere — those users are told they are current, for ever.
   */
  it('refuses a manifest that is missing a platform the matrix builds', () => {
    const manifest = good();
    delete manifest.platforms['windows-x86_64'];

    const problems = inspect(manifest, { platforms: PLATFORMS, version: '0.1.0' });
    expect(problems.join(' ')).toContain('windows-x86_64');
    expect(problems.join(' ')).toContain('told they are current');
  });

  /**
   * The failure that looks most like success: the artifact exists, it installs
   * by hand, and the updater refuses it — on the user's machine, months later.
   */
  it('refuses an entry whose signature is empty', () => {
    const manifest = good();
    manifest.platforms['darwin-aarch64'].signature = '';

    expect(inspect(manifest, { platforms: PLATFORMS, version: '0.1.0' }).join(' ')).toContain(
      'darwin-aarch64 carries an empty signature'
    );
  });

  it('refuses an entry with no url', () => {
    const manifest = good();
    delete manifest.platforms['linux-x86_64'].url;

    expect(inspect(manifest, { platforms: PLATFORMS, version: '0.1.0' }).join(' ')).toContain(
      'linux-x86_64 has no url'
    );
  });

  /** A release published from a tag nobody bumped the config to match. */
  it('refuses a manifest that offers the version already running', () => {
    const problems = inspect(good(), { platforms: PLATFORMS, version: '0.2.0' });
    expect(problems.join(' ')).toContain('offers 0.1.0 and this checkout is 0.2.0');
  });

  it('tolerates a leading v on the manifest version', () => {
    const manifest = { ...good(), version: 'v0.1.0' };
    expect(inspect(manifest, { platforms: PLATFORMS, version: '0.1.0' })).toEqual([]);
  });

  it('says so when the endpoint answers with something that is not an object', () => {
    expect(inspect('<html>404</html>', { platforms: PLATFORMS, version: '0.1.0' })).toEqual([
      'the endpoint answered with something that is not a JSON object',
    ]);
  });
});

describe('the diagnosis for a 404', () => {
  /**
   * Both causes, and neither ranked above the other.
   *
   * An earlier version of this said the draft was the likelier one, and the
   * repository's own release disproved it on the first look: `v0.1.0` is
   * published, not a draft, and carries zero assets — because the build never
   * reached the step that uploads them. A diagnosis that guesses wrong sends
   * somebody to the wrong half of the problem, which is the cost this item has
   * already paid once.
   */
  it('names both causes and ranks neither', () => {
    const text = diagnose(404, 'https://github.com/o/r/releases/latest/download/latest.json');
    expect(text).toContain('count them');
    expect(text).toContain('releaseDraft: true');
    expect(text).toContain('presses Publish');
    expect(text).toContain('--url');
    expect(text).not.toContain('likeliest');
  });
});

describe('the platforms it expects', () => {
  /**
   * Derived from the release matrix, so a seventh target widens the check on
   * its own. A hand-written list is the copy that goes stale, and the way it
   * goes stale is by passing.
   */
  it('is the release matrix, translated into what the updater calls them', () => {
    const workflow = readFileSync(resolve(ROOT, '.github/workflows/release.yml'), 'utf8');
    const platforms = platformsFrom(workflow);

    expect(platforms).toHaveLength(6);
    expect(platforms).toContain('darwin-aarch64');
    expect(platforms).toContain('windows-aarch64');
    expect(new Set(platforms).size).toBe(platforms.length);
  });

  it('refuses to guess at a target it has no name for', () => {
    expect(() => platformsFrom('      - target: riscv64gc-unknown-linux-gnu\n')).toThrow(
      /riscv64gc/
    );
  });
});

describe('what it reads out of the configuration', () => {
  it('takes the endpoint and the version from tauri.conf.json', () => {
    const conf = JSON.parse(readFileSync(resolve(ROOT, 'src-tauri/tauri.conf.json'), 'utf8'));
    const { endpoint, version } = declared(conf);

    expect(endpoint).toMatch(/^https:\/\/github\.com\/.+\/releases\/latest\/download\/latest\.json$/);
    expect(version).toMatch(/^\d+\.\d+\.\d+/);
  });
});
