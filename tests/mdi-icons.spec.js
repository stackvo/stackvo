import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { aliases } from 'vuetify/iconsets/mdi';
import { MDI_CSS, SOURCE_ROOTS, glyphRule, iconsInSource, iconsUsed } from '../tools/mdi-icons.mjs';

/**
 * Every icon this application names has to be an icon.
 *
 * A missing glyph is the quietest failure a screen can have: `mdi-nonsense`
 * renders as nothing, in the exact space an icon would have taken, and looks
 * like a spacing decision. Nothing warns — not the compiler, not the linter,
 * not the browser.
 *
 * It had already happened twice, and this file exists because the count came
 * out of a measurement rather than a suspicion. `mdi-discord` had not been in
 * Material Design Icons for two major versions — the set dropped its brand
 * marks — so the Discord row of the community links carried an empty square,
 * and `mdi-format-textdirection-r-to-l` was renamed upstream, so did the
 * heading of the Localisation panel's direction group.
 *
 * The build now also *strips* the icon rules nothing here names, which turns
 * this from tidiness into load-bearing: a name misspelt here is a name the
 * subsetter cannot keep. `tools/check-bundle.mjs` holds the other half — that
 * everything on this list survived into the built stylesheet — because that is
 * the only moment the built stylesheet exists.
 *
 * It happened a third time, and that one only became findable when the scan
 * learned to read `src-tauri/src`. **Eighteen icons are named nowhere but
 * Rust** — `apps.rs` is a catalogue, and `mdi-apple`, `mdi-firefox` and
 * `mdi-powershell` are its rows — so the subsetter had been stripping every
 * icon in the terminal, editor and browser pickers. The first run of the wider
 * scan then found `mdi-vim`, on the Neovim and Vim rows, which is not an icon
 * and never has been.
 */

const upstream = new Set(
  [...readFileSync(MDI_CSS, 'utf8').matchAll(glyphRule())].map((match) => `mdi-${match[1]}`)
);

describe('the icons this app names', () => {
  it('reads the icon set at all', () => {
    // Thousands, or the regex stopped matching the stylesheet's shape and every
    // assertion below would pass by comparing against nothing.
    expect(upstream.size).toBeGreaterThan(5000);
    expect(upstream.has('mdi-account')).toBe(true);
  });

  it('names only glyphs that exist', () => {
    const named = [...iconsInSource(SOURCE_ROOTS)];
    expect(named.length).toBeGreaterThan(100);

    const missing = named.filter((name) => !upstream.has(name));
    expect(
      missing,
      'These names look like icons and are not in Material Design Icons — each ' +
        'renders as a blank square. Either the name is wrong (it may have been ' +
        'renamed or dropped upstream), or it is prose: the scan reads whole files, ' +
        'comments included, so an icon named in a comment is collected too.'
    ).toEqual([]);
  });

  /**
   * Vuetify renders icons this repository never writes down — the checkbox
   * marks, the sort arrows, the pagination chevrons, the alert glyphs. Twenty-
   * six of them appear nowhere in `src/`, so a subsetter fed only by a scan of
   * this tree ships an application whose checkboxes are empty, and that reads
   * as a Vuetify bug rather than as a build one.
   */
  it('counts the icons Vuetify renders on its own, which the source never mentions', () => {
    const fromSource = iconsInSource(SOURCE_ROOTS);
    const everything = iconsUsed(SOURCE_ROOTS, aliases);

    const onlyVuetify = [...everything].filter((name) => !fromSource.has(name));
    expect(onlyVuetify.length).toBeGreaterThan(0);
    // The ones a checkbox and a select cannot do without, spelled out: if the
    // alias set is ever dropped from the used list these are what goes blank.
    expect(everything.has('mdi-checkbox-marked')).toBe(true);
    expect(everything.has('mdi-radiobox-blank')).toBe(true);
    expect(everything.has('mdi-menu-down')).toBe(true);
  });

  /**
   * The catalogues live in Rust, and for a while the scan could not see them.
   *
   * Not a count of what `src/` happens to duplicate — a named few, so that the
   * scan losing the Rust tree fails here rather than quietly shipping three
   * pickers of blank squares again. Each of these is one row of `apps.rs`, and
   * no `.vue` file repeats any of them.
   */
  it('reads the icons only the Rust catalogues name', () => {
    const everything = iconsInSource(SOURCE_ROOTS);
    const frontendOnly = iconsInSource('src');

    for (const name of ['mdi-apple', 'mdi-firefox', 'mdi-powershell', 'mdi-pencil-outline']) {
      expect(everything.has(name), `${name} is named in apps.rs and was not read`).toBe(true);
      expect(frontendOnly.has(name), `${name} is now in src/ too — pick another`).toBe(false);
    }
  });

  /**
   * The modifiers share the prefix and are not glyphs. Dropping them would take
   * away `mdi-spin` — the loading indicator — and every rotation, and neither
   * failure looks like a missing icon: the icon is there and simply does not
   * turn.
   */
  it('leaves the modifier classes out of the glyph list', () => {
    for (const modifier of ['mdi-spin', 'mdi-flip-h', 'mdi-rotate-45', 'mdi-18px']) {
      expect(upstream.has(modifier), `${modifier} was read as a glyph rule`).toBe(false);
    }
  });
});
