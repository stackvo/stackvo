import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { RUNTIME_DEFAULTS, VERSION_LISTS } from '../src/composables/useCatalog';

/**
 * The version lists a person can edit, against the ones the binary reads.
 *
 * `build_catalog` builds its key with `format!("SUPPORTED_LANGUAGES_{key}_VERSIONS")`,
 * so a seventh runtime added to `config::SETTINGS` is read by Rust immediately
 * and offered by the pickers immediately — and would be editable by nobody,
 * because the settings pane iterates a list written by hand over here.
 *
 * That is the exact shape of the bug this pane was added for. Six `.env` keys
 * decided which versions every picker offered, the application shipped them
 * compiled in, they went out of date, and the only way to change one was to
 * edit `.env` by hand or wait for a release. A seventh arriving unnoticed puts
 * one runtime straight back into that state, so the two lists are held
 * together here rather than trusted to stay level.
 *
 * Read out of `config.rs` as text. The alternative is asking the running app
 * through `catalogGet`, which answers with what a *workspace* has rather than
 * what this build embeds — and a workspace whose `.env` narrows the set would
 * make this pass while the pane was missing a key.
 */

const CONFIG = readFileSync('src-tauri/src/config.rs', 'utf8');

/** Every `SUPPORTED_LANGUAGES_*_VERSIONS` key the binary embeds a default for. */
function embeddedKeys() {
  return [...CONFIG.matchAll(/"(SUPPORTED_LANGUAGES_[A-Z0-9]+_VERSIONS)"/g)]
    .map((match) => match[1])
    .filter((key, index, all) => all.indexOf(key) === index)
    .sort();
}

describe('the version lists the settings pane offers to edit', () => {
  it('reads the Rust settings table at all', () => {
    // Six today. A number rather than "more than zero", because the regex
    // silently matching nothing is the way this test stops being a test.
    expect(embeddedKeys().length).toBeGreaterThanOrEqual(6);
    expect(embeddedKeys()).toContain('SUPPORTED_LANGUAGES_PHP_VERSIONS');
  });

  it('offers exactly the keys the binary embeds a default for', () => {
    const declared = VERSION_LISTS.map((list) => list.key).sort();
    expect(
      declared,
      'a `SUPPORTED_LANGUAGES_*_VERSIONS` key exists in config.rs that the settings ' +
        'pane does not offer, or the pane offers one that no longer exists. The first ' +
        'is a runtime whose offered versions nobody can edit — which is the state this ' +
        'pane was written to end.'
    ).toEqual(embeddedKeys());
  });

  /**
   * The two lists are near-identical and mean different things: one is *which
   * versions exist*, the other is *which one a new project starts on*. Getting
   * a key from the wrong family into either would edit the wrong setting while
   * looking right on screen.
   */
  it('does not confuse the offered list with the default', () => {
    for (const list of VERSION_LISTS) {
      expect(list.key.endsWith('_VERSIONS'), list.key).toBe(true);
    }
    for (const runtime of RUNTIME_DEFAULTS) {
      expect(runtime.key.endsWith('_DEFAULT'), runtime.key).toBe(true);
    }
  });

  /**
   * `.env` spells it `nodejs` and the manifest spells it `node` — C-01, and
   * `build_catalog` translates between them. The key here has to be `.env`'s
   * spelling or it writes a setting nothing reads.
   */
  it('uses the spelling .env uses for Node', () => {
    const node = VERSION_LISTS.find((list) => list.id === 'nodejs');
    expect(node, 'the Node entry is keyed by .env’s spelling').toBeTruthy();
    expect(node.key).toBe('SUPPORTED_LANGUAGES_NODEJS_VERSIONS');
  });
});
