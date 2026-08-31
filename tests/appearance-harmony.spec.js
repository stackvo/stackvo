import { describe, it, expect } from 'vitest';
import { luminance, parse } from '@/lib/contrast.js';
import {
  APPEARANCE_RULES,
  DEFAULT_APPEARANCE,
  HARMONIES,
  PRIMARY_SWATCHES,
  RANGES,
  harmonise,
  migrate,
  parseAppearance,
  themeSnippet,
} from '@/lib/appearance.js';

/**
 * Where `secondary` comes from, and whether it comes from anywhere at all.
 *
 * The bug this was written after is not a maths bug. `applyAppearance` wrote
 * `primary` and nothing else, and `plugins/vuetify.js` declared `secondary` as
 * a constant — so the accent setting moved one colour and left the other where
 * it had always been. Every checkbox in the application is drawn in
 * `secondary` (the md3 blueprint's own default), the timeline rule reads
 * `--v-theme-secondary` out of the stylesheet, and `ProjectDetail`'s replay
 * button asks for it by name. A purple accent had blue tick boxes for the
 * whole life of the setting, and no test could have noticed, because no test
 * asked the question.
 *
 * So the first assertion here is the cheapest one: the two colours move
 * together.
 */

/**
 * The hue of a colour, measured independently of the implementation.
 *
 * Deliberately a second implementation rather than an import: `harmonise`
 * claims to turn the hue by a stated number of degrees, and checking that
 * claim with the same conversion the claim is built on would only prove the
 * function is self-consistent.
 */
function hueOf(hex) {
  const [r, g, b] = parse(hex).map((c) => c / 255);
  const max = Math.max(r, g, b);
  const span = max - Math.min(r, g, b);
  if (span === 0) return null;

  const h =
    max === r
      ? ((g - b) / span + (g < b ? 6 : 0)) * 60
      : max === g
        ? ((b - r) / span + 2) * 60
        : ((r - g) / span + 4) * 60;
  return h;
}

/** Signed shortest distance from `a` to `b` around the wheel, in degrees. */
const apart = (a, b) => ((((b - a) % 360) + 540) % 360) - 180;

/**
 * Two degrees, and the number is measured rather than picked.
 *
 * The rotation is exact in floating point and then rounded to eight bits per
 * channel, which moves the hue back by a fraction of a degree — by *more* the
 * less saturated the colour is, since a fixed rounding step is a larger share
 * of a smaller span between the channels. The worst case over the twenty
 * swatches this application offers is 1.30°, at `#546E7A`, which is the least
 * saturated of them and therefore exactly where it should be. Two leaves room
 * for a swatch quieter than that one without leaving room for a wrong angle:
 * the smallest gap between two harmonies here is 30°.
 */
const HUE_TOLERANCE = 2;

describe('deriving the secondary colour', () => {
  /**
   * The documented angles, measured. Studio names these in its own UI —
   * analogous "adds and subtracts 30° from hue", triadic 120, split 150 — and
   * the labels this app shows say the same thing, so the numbers are a promise
   * to the reader rather than an internal detail.
   */
  it('turns the hue by the angle its label claims', () => {
    const angles = { analog: -30, triadic: -120, split: -150 };
    for (const swatch of PRIMARY_SWATCHES) {
      for (const [harmony, expected] of Object.entries(angles)) {
        const got = hueOf(harmonise(swatch, harmony));
        const moved = apart(hueOf(swatch), got);
        expect(
          Math.abs(moved - expected),
          `${swatch} ${harmony} moved ${moved.toFixed(2)}°, not ${expected}°`
        ).toBeLessThan(HUE_TOLERANCE);
      }
    }
  });

  /** `mono` is the one that does not turn: same hue, less of it. */
  it('leaves the hue alone for mono and takes the saturation out instead', () => {
    for (const swatch of PRIMARY_SWATCHES) {
      const got = harmonise(swatch, 'mono');
      expect(Math.abs(apart(hueOf(swatch), hueOf(got))), `${swatch} moved to ${got}`).toBeLessThan(
        HUE_TOLERANCE
      );

      // Less saturated means the channels are closer together.
      const spread = (hex) => {
        const rgb = parse(hex);
        return Math.max(...rgb) - Math.min(...rgb);
      };
      expect(spread(got), `${swatch} → ${got} is not quieter`).toBeLessThan(spread(swatch));
    }
  });

  /**
   * The correction that makes this usable, and the reason it is not a plain
   * hue rotation.
   *
   * HSL's `l` is not perceived lightness. Rotating `#1976D2` by −30° at the
   * same `s` and `l` gives `#19D2D1`, and a cyan at half lightness is far
   * brighter than a blue at half lightness — the accent was a calm mid-blue
   * and its partner was a highway sign. `harmonise` recovers HCT's constant
   * *tone* by measuring: it moves `l` until the relative luminance is back
   * where it started.
   *
   * 1.5% rather than an exact match: the answer is rounded to eight bits per
   * channel before it can be measured again, and at the dark end of the scale
   * one step of the last channel is worth more than it is in the middle.
   */
  it('gives the partner the same visual weight as the accent', () => {
    for (const swatch of PRIMARY_SWATCHES) {
      const before = luminance(swatch);
      for (const harmony of HARMONIES) {
        const after = luminance(harmonise(swatch, harmony));
        expect(
          Math.abs(after - before),
          `${swatch} ${harmony}: ${before.toFixed(4)} → ${after.toFixed(4)}`
        ).toBeLessThan(0.015);
      }
    }
  });

  /** Every swatch, every harmony, an actual colour — nothing returns null. */
  it('answers with a colour for every swatch the app offers', () => {
    for (const swatch of PRIMARY_SWATCHES) {
      for (const harmony of HARMONIES) {
        const got = harmonise(swatch, harmony);
        expect(got, `${swatch} ${harmony}`).toMatch(/^#[0-9a-f]{6}$/);
      }
    }
  });

  /**
   * The same contract `readable` has, and for the same reason: this value goes
   * straight into `theme.colors.secondary`, and Vuetify writes whatever is
   * there into the stylesheet. A `null` there is a broken checkbox, not a
   * caught error.
   */
  it('hands back what it was given rather than null when that is not a colour', () => {
    for (const bad of ['', 'rebeccapurple', '#12345']) {
      expect(harmonise(bad, 'analog'), `${bad}`).toBe(bad);
    }
    expect(harmonise(null, 'analog')).toBeNull();
  });

  /** An unknown harmony is a preference file somebody hand-edited, not a crash. */
  it('falls back to the default harmony rather than throwing', () => {
    expect(harmonise('#1976D2', 'nonsense')).toBe(harmonise('#1976D2', 'analog'));
    expect(harmonise('#1976D2')).toBe(harmonise('#1976D2', DEFAULT_APPEARANCE.harmony));
  });
});

describe('migrating a saved look', () => {
  /**
   * `highContrast` was a switch and became three stops. `true` meant "as much
   * help as this application can give", so it maps to the top — a reader who
   * had asked for the most and silently received the middle would have no way
   * to notice, because the thing they would be comparing against is gone.
   */
  it('turns the old high-contrast switch into the top stop', () => {
    expect(migrate({ highContrast: true }).contrast).toBe('high');
  });

  it('leaves the default alone when the switch was off', () => {
    expect(migrate({ highContrast: false, contrast: 'standard' }).contrast).toBe('standard');
  });

  /**
   * Both keys are present in exactly one situation — a preset saved before the
   * change, applied after it — and there the explicit value is the one the
   * user last chose.
   */
  it('lets an explicit level win over the legacy switch', () => {
    expect(migrate({ highContrast: true, contrast: 'medium' }).contrast).toBe('medium');
  });

  /**
   * Deleted rather than ignored. A key left in place rides along in every
   * future write of preferences.json and is copied into every preset saved
   * from then on, forever, for a setting nothing reads.
   */
  it('drops the stale key so it stops being written back', () => {
    expect('highContrast' in migrate({ highContrast: true })).toBe(false);
  });

  it('does not invent keys for a look that never had one', () => {
    expect(migrate({ ...DEFAULT_APPEARANCE })).toEqual(DEFAULT_APPEARANCE);
  });
});

/**
 * The JSON body of a generated snippet, pulled back out of it.
 *
 * Brace-counted rather than sliced at a marker, because the body contains
 * braces of its own and the closing one that matters is not the last in the
 * file. Eight lines to prove the emitted code carries valid JSON, which is the
 * only thing about a snippet that cannot be checked by reading it.
 */
function themesIn(snippet) {
  const start = snippet.indexOf('{', snippet.indexOf('themes:'));
  let depth = 0;

  for (let i = start; i < snippet.length; i += 1) {
    if (snippet[i] === '{') depth += 1;
    if (snippet[i] === '}') depth -= 1;
    if (depth === 0) return JSON.parse(snippet.slice(start, i + 1));
  }
  return null;
}

describe('reading a look somebody pasted', () => {
  /**
   * The guard that matters most here, and the one a reviewer cannot perform by
   * eye: a setting added to `DEFAULT_APPEARANCE` with no rule beside it is a
   * setting that silently fails to import, and the only symptom is a field
   * that "did not come across" for one person, once.
   */
  it('has a rule for every setting, and a setting for every rule', () => {
    expect(Object.keys(APPEARANCE_RULES).toSorted()).toEqual(
      Object.keys(DEFAULT_APPEARANCE).toSorted()
    );
  });

  it('round-trips the defaults with nothing lost and nothing ignored', () => {
    const { values, ignored } = parseAppearance(JSON.stringify(DEFAULT_APPEARANCE));
    expect(ignored).toEqual([]);
    expect(values).toEqual(DEFAULT_APPEARANCE);
  });

  it('round-trips a customised look', () => {
    const look = {
      ...DEFAULT_APPEARANCE,
      theme: 'light',
      primary: PRIMARY_SWATCHES.at(-1),
      harmony: 'triadic',
      neutral: 'warm',
      contrast: 'high',
      radius: RANGES.radius.max,
      fontSize: RANGES.fontSize.min,
    };
    expect(parseAppearance(JSON.stringify(look))).toEqual({ values: look, ignored: [] });
  });

  /**
   * A paste that went wrong is not a partial import. Merging an unrecognised
   * object over the defaults would reset the look somebody was trying to add
   * to, and report success while doing it.
   */
  it('refuses anything that is not a look at all', () => {
    for (const bad of ['', 'not json', '[]', 'null', '"a string"', '42', '{"nope":1}']) {
      expect(parseAppearance(bad).values, `${bad} was accepted`).toBeNull();
    }
    expect(parseAppearance(undefined).values).toBeNull();
  });

  /**
   * The values a truncated or hand-edited paste actually produces. None of
   * these is an attack; all of them leave an application that cannot be got
   * back to normal from its own settings page.
   */
  it('drops a field that is out of range, and says which', () => {
    const { values, ignored } = parseAppearance(
      JSON.stringify({ ...DEFAULT_APPEARANCE, radius: 9000, density: 'roomy', fontSize: 1.5 })
    );

    expect(ignored.toSorted()).toEqual(['density', 'fontSize', 'radius']);
    expect(values.radius).toBe(DEFAULT_APPEARANCE.radius);
    expect(values.density).toBe(DEFAULT_APPEARANCE.density);
    // And everything it did understand still came across.
    expect(values.neutral).toBe(DEFAULT_APPEARANCE.neutral);
  });

  /**
   * Held to the grid the pane offers rather than to "is a hex": a colour from
   * outside it is one no swatch can show as selected, so the page would be
   * unable to say what it had just done.
   */
  it('refuses an accent the pane could not display', () => {
    const { values, ignored } = parseAppearance(
      JSON.stringify({ ...DEFAULT_APPEARANCE, primary: '#123456' })
    );
    expect(ignored).toEqual(['primary']);
    expect(values.primary).toBe(DEFAULT_APPEARANCE.primary);
  });

  /** A look exported before the contrast switch became three stops. */
  it('migrates on the way in rather than reporting the old key as junk', () => {
    const { values, ignored } = parseAppearance(
      JSON.stringify({ ...DEFAULT_APPEARANCE, highContrast: true, contrast: undefined })
    );
    expect(ignored).toEqual([]);
    expect(values.contrast).toBe('high');
  });

  /** A field from a newer release is a usable look, not a failure. */
  it('takes what it knows from a look with a field it does not', () => {
    const { values, ignored } = parseAppearance(
      JSON.stringify({ ...DEFAULT_APPEARANCE, neutral: 'warm', somethingNewer: 'x' })
    );
    expect(ignored).toEqual(['somethingNewer']);
    expect(values.neutral).toBe('warm');
  });
});

describe('exporting the look as a Vuetify theme', () => {
  /**
   * The only thing about generated code that cannot be checked by reading it:
   * whether it parses. `JSON.stringify` writes `null` for a value that came
   * back missing, and a theme with a `null` colour in it is a stylesheet with
   * `rgb(null)` in it — valid JSON, broken CSS.
   */
  it('emits both themes, as valid JSON, with no holes in them', () => {
    const themes = themesIn(themeSnippet(DEFAULT_APPEARANCE));

    expect(Object.keys(themes).toSorted()).toEqual(['dark', 'light']);
    for (const [name, theme] of Object.entries(themes)) {
      expect(theme.dark, `${name}.dark`).toBe(name === 'dark');
      for (const [role, value] of Object.entries(theme.colors)) {
        expect(value, `${name}.${role} is empty`).toBeTruthy();
        expect(typeof value, `${name}.${role}`).toBe('string');
      }
    }
  });

  /**
   * The derived colour has to survive the export, or the snippet describes a
   * theme the application never rendered — which is the failure the constant
   * `secondary` was, moved into a file somebody pastes elsewhere.
   */
  it('carries the accent and the colour derived from it', () => {
    const look = { ...DEFAULT_APPEARANCE, primary: '#D81B60', harmony: 'triadic' };
    const themes = themesIn(themeSnippet(look));

    for (const theme of Object.values(themes)) {
      expect(theme.colors.primary).toBe('#D81B60');
      expect(theme.colors.secondary).toBe(harmonise('#D81B60', 'triadic'));
    }
  });

  it('names the chosen default theme in the call it writes', () => {
    expect(themeSnippet({ ...DEFAULT_APPEARANCE, theme: 'light' })).toContain(
      "defaultTheme: 'light'"
    );
    expect(themeSnippet(DEFAULT_APPEARANCE)).toContain('createVuetify');
  });
});
