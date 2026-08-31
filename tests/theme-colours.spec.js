import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import vuetify from '../src/plugins/vuetify';
import { CONSOLE_BACKGROUND, DEFAULT_APPEARANCE, harmonise } from '../src/lib/appearance';

/**
 * Colours that must come from the theme, and one that must come from one place.
 *
 * The application can be re-themed at runtime — accent, light/dark, the neutral
 * pack, and a status palette chosen for red-green deficiency — and four charts
 * ignored all of it. `#1976D2` was written three times in the stats composable
 * and once more in the pane, `#4CAF50` once, and every one of them was a
 * *copy*: of `DEFAULT_APPEARANCE.primary`, of the graphite theme's
 * `surface-variant`, of `success`. A user who moved the accent to purple got a
 * purple application with three blue pie charts in it, and on the light theme
 * the second slice of every pie was dark charcoal on a white card.
 *
 * The behaviour is pinned where it belongs, in `project-indicator.spec.js`,
 * which mounts the pane under a theme sharing nothing with those literals. This
 * file is the other half — the places a literal can come back that no mounted
 * test would see: a stylesheet, and a composable whose colours nothing reads
 * until they are on screen.
 */

const read = (path) => readFileSync(path, 'utf8');

/**
 * A file with its comments taken out.
 *
 * Load-bearing rather than tidy. This repository writes down the bug a rule
 * exists to prevent, in the file the rule lives in, so every comment below the
 * fixes here names `#1976D2` or `#12121a` — and a gate that read the prose
 * would fail on the sentence explaining why it must not fail. It measured that
 * on its first run, twice.
 *
 * `//` is required not to follow a colon so that `https://` survives, which is
 * the only shape in these files where the two characters are not a comment.
 */
const code = (path) =>
  read(path)
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1');

/** Any three- to eight-digit hex colour, which is how all of them were written. */
const HEX = /#[0-9a-fA-F]{3,8}\b/g;

describe('the charts and the theme', () => {
  /**
   * Colours left this file entirely rather than being replaced with theme
   * lookups, because `useTheme()` is only callable from a component. A hex
   * appearing here again is the whole bug returning, in its original shape.
   */
  it('leaves no colour in the stats composable', () => {
    const source = code('src/composables/useContainerStats.js');
    const found = [...source.matchAll(HEX)].map((m) => m[0]);

    expect(
      found,
      'a colour is being written into the pie data again. The pane paints them ' +
        'from `useTheme()`; a value here is a value that cannot follow the theme.'
    ).toEqual([]);
  });

  /**
   * The heat grid was five fixed greens tied to nothing — not the theme, not
   * the accent, not the status palette. On the light theme its two quiet bands
   * were near-black squares on a white card, and a person who had chosen the
   * Okabe-Ito palette for colour blindness got it everywhere except on the one
   * card that is nothing but a field of colour.
   */
  it('draws the heat ramp out of the theme', () => {
    const css = code('src/styles/project-panes.css');
    const rules = [...css.matchAll(/\.heat-cell\.(l[0-4])\s*\{([^}]*)\}/g)];

    expect(rules.length, 'the five intensity bands are not where this expects them').toBe(5);

    for (const [, band, body] of rules) {
      expect(
        [...body.matchAll(HEX)].map((m) => m[0]),
        `${band} is a fixed colour again`
      ).toEqual([]);
      expect(body, `${band} does not read the theme`).toContain('--v-theme-success');
    }
  });

  /**
   * The one colour that is deliberately not a theme value: `darkConsoles` asks
   * for a surface darker than any theme's, and xterm paints its own canvas so
   * it has to be told a value. One constant, then — it was two literals in one
   * file, the JS terminal theme and the CSS host frame, eight pixels of padding
   * apart. Two literals that must agree are a frame waiting to appear.
   */
  it('writes the console background exactly once', () => {
    const files = [];
    const walk = (dir) => {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const path = join(dir, entry.name);
        if (entry.isDirectory()) walk(path);
        else if (/\.(vue|js|css)$/.test(entry.name)) files.push(path);
      }
    };
    walk('src');

    const carrying = files.filter((path) => code(path).includes(CONSOLE_BACKGROUND));

    expect(carrying, `${CONSOLE_BACKGROUND} is written in more than one place`).toEqual([
      'src/lib/appearance.js',
    ]);
  });

  /**
   * And the reason the literals were confusing in the first place: they *were*
   * the right answer, once, for one theme. Asserting the identity keeps the
   * comments above honest — if the default accent moves, the story about
   * `#1976D2` being a copy of it stops being true and should be rewritten.
   */
  it('still has the default accent the old literals were copied from', () => {
    expect(DEFAULT_APPEARANCE.primary).toBe('#1976D2');
  });

  /**
   * The tonal utilities exist, and the accent is the one they follow.
   *
   * `variations` is three words of configuration that produce nothing visible
   * and fail silently: written wrong, or dropped in a merge, the classes simply
   * are not generated and the only symptom is an element with no colour. This
   * asks the running theme for them.
   *
   * The scope is asserted too — `primary` only, one step each way. Not
   * tidiness: the variations are generated into the *runtime* stylesheet, which
   * `check-bundle.mjs` cannot measure and which is rebuilt on every theme
   * change, so widening this is a cost no other guard in this repository would
   * report. See the note in `plugins/vuetify.js` for the measured numbers.
   */
  it('generates the accent variations, and only those', () => {
    const colors = vuetify.theme.computedThemes.value.dark.colors;

    expect(colors['primary-lighten-1'], 'the tonal utilities were not generated').toBeTruthy();
    expect(colors['primary-darken-1']).toBeTruthy();

    // Two of the six are not generated at all. `secondary-darken-1` is a colour
    // Vuetify's own stock light and dark themes declare — it has been in this
    // application's palette since the first commit, is never re-themed by
    // `applyAppearance`, and nothing draws it — and `on-secondary-darken-1` is
    // the on-colour derived from it. Written out rather than filtered away,
    // because a reader counting six here would otherwise conclude the scope is
    // wider than it is.
    const named = Object.keys(colors).filter((key) => /-(lighten|darken)-\d+$/.test(key));
    expect(named.toSorted(), 'the variation scope has widened unmeasured').toEqual([
      'on-primary-darken-1',
      'on-primary-lighten-1',
      'on-secondary-darken-1',
      'primary-darken-1',
      'primary-lighten-1',
      'secondary-darken-1',
    ]);

    // What widening actually looks like, asserted separately so the reason a
    // failure happened is visible in the message: a second step, or a second
    // colour, each of which was measured before this scope was chosen.
    expect(
      named.filter((key) => key.endsWith('-2')),
      'a second step was added'
    ).toEqual([]);
    expect(colors['secondary-lighten-1'], 'secondary was added to the scope').toBeUndefined();
  });

  /**
   * A copy that arrived with the fix for the opposite bug.
   *
   * `secondary` used to be a *chosen* colour that nothing ever moved, which is
   * why a purple accent had blue checkboxes. It is derived now — and the value
   * in `plugins/vuetify.js` is the seed the window paints with for the frame
   * before `applyAppearance` runs, hand-written because that file cannot import
   * the module that imports it.
   *
   * So it is a copy, of exactly the kind this file exists to catch. Nothing
   * recomputes it: move the default accent, or change the derivation, and the
   * seed goes stale in silence — the window would paint one secondary at boot
   * and a different one a tick later.
   *
   * Asserted against the derivation rather than against a second hex, so the
   * only way to break it is to change one of the two things it claims to
   * follow.
   */
  it('seeds the secondary colour with the value the first apply will derive', () => {
    const seed = code('src/plugins/vuetify.js').match(/secondary: '(#[0-9a-fA-F]{6})'/);

    expect(seed, 'the seed is gone, or was renamed out of reach of this test').toBeTruthy();
    expect(seed[1].toLowerCase(), 'the seed no longer matches what boot will compute').toBe(
      harmonise(DEFAULT_APPEARANCE.primary, DEFAULT_APPEARANCE.harmony)
    );
  });
});
