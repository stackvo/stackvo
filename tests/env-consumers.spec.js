import { describe, it, expect } from 'vitest';
import { existsSync } from 'node:fs';
import { audit, ROOTS, DECLARATION } from '../tools/measure-env-usage.mjs';

/**
 * `env.schema.json` says which files read each setting. This is whether that is
 * still true.
 *
 * The field had rotted in the way a documentation field rots when nothing reads
 * it: **39 of its 45 paths pointed at files that no longer exist**, every one of
 * them under `core/` — the Bash and Node implementation the Rust rewrite
 * replaced. Six pointed at the live tree. The tool that maintains the field
 * pointed at `core/` too, so it exited 2 on every run and said so to nobody.
 *
 * A contract field naming a deleted tree is worse than no field. It reads as
 * evidence, and somebody deciding whether a setting is safe to change reads it.
 *
 * Two things are checked and they fail for different reasons. **The paths
 * exist** — a file renamed or deleted is the ordinary way this goes stale, and
 * it is silent. **The lists are what a fresh scan finds** — a setting that
 * gained or lost a reader is the other way, and it is just as silent.
 *
 * Both run the same scanner the `--fix` flag uses, so the fix for a failure is
 * `node tools/measure-env-usage.mjs --fix` and a look at the diff.
 */

const { rows, files, schema } = audit();

describe('the env schema’s consumers', () => {
  /**
   * A scanner that finds nothing makes every assertion below pass by comparing
   * empty lists. Counts rather than "more than zero", because the numbers are
   * what was measured: four trees, ~300 files, 72 keys.
   */
  it('reads the tree at all', () => {
    expect(ROOTS).toEqual(['src-tauri/src', 'src', 'skeleton', 'tools']);
    expect(files).toBeGreaterThan(200);
    expect(rows.length).toBeGreaterThan(60);
    expect(
      rows.some((r) => r.found.length > 0),
      'no key has any consumer'
    ).toBe(true);
  });

  it('names only files that exist', () => {
    const named = [
      ...new Set(rows.flatMap((r) => r.spec.consumers ?? []).map((c) => c.split(' ')[0])),
    ];
    expect(named.length).toBeGreaterThan(10);

    const gone = named.filter((path) => !existsSync(path));
    expect(
      gone,
      'the schema names files that are not in the tree. That is how this field ' +
        'died last time — 39 of 45 paths were `core/…`, a directory the Rust ' +
        'rewrite removed. Run `node tools/measure-env-usage.mjs --fix`.'
    ).toEqual([]);
  });

  it('lists what a fresh scan finds, for every key', () => {
    const drifted = rows
      .filter((r) => {
        const stored = [...(r.spec.consumers ?? [])].sort().join('|');
        return stored !== [...r.found].sort().join('|');
      })
      .map((r) => r.key);

    expect(
      drifted,
      'the stored consumers disagree with a fresh scan. Run ' +
        '`node tools/measure-env-usage.mjs --fix` and read the diff — a key that ' +
        'gained a reader is a fact worth seeing, and one that lost its last ' +
        'reader is a setting that stopped doing anything.'
    ).toEqual([]);
  });

  /**
   * The status label and the measurement, which is the check the tool was
   * written for and the one that has been unrunnable since `core/` went.
   */
  it('labels a key active only when something reads it', () => {
    const wrong = rows
      .filter((r) => !r.agrees)
      .map((r) => `${r.key}: labelled ${r.labelled}, measured ${r.measured}`);

    expect(wrong).toEqual([]);
  });

  /**
   * The header used to name a file that is not here and a count that was not
   * this document's: `.env.example`, 159 keys. Both were fossils of the Bash
   * implementation — the file went with it, and 159 was *that* file's key count.
   * A contract whose first paragraph is wrong is one nobody trusts the rest of.
   */
  it('describes as many keys as it says it does, and names no file that is gone', () => {
    expect(schema.source.file, 'there is no .env.example — the defaults are in config.rs').toBe(
      null
    );
    expect(schema.source.keys).toBe(rows.length);
    expect(schema.contractVersion).toBeTruthy();
  });

  /**
   * Declaration is not consumption, and the whole measurement turns on it:
   * every key is written in `config.rs`'s settings tables, so counting that
   * file would report all 72 as active and `dead` would become unreachable.
   */
  it('does not count the file the keys are declared in', () => {
    expect(DECLARATION).toBe('src-tauri/src/config.rs');

    const counted = rows.flatMap((r) => r.found).filter((path) => path === DECLARATION);
    expect(counted, 'the declaration site is being counted as a reader').toEqual([]);

    // And there is still something to find, so the exclusion has not swallowed
    // the measurement whole.
    expect(rows.filter((r) => r.measured === 'dead').length).toBeGreaterThan(10);
    expect(rows.filter((r) => r.measured === 'active').length).toBeGreaterThan(10);
  });
});
