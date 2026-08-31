import { describe, it, expect } from 'vitest';
import { contrast, luminance, parse, readable, AA_TEXT } from '@/lib/contrast.js';
import { CONTRAST_LEVELS, STATUS_PALETTES, NEUTRALS } from '@/lib/appearance.js';

/**
 * The arithmetic behind "can this be read".
 *
 * `tests/e2e/a11y.e2e.js` is what proves the application passes; this is what
 * makes the number it depends on trustworthy. The two are not the same job: axe
 * measures a rendered page and takes six seconds a route, and this decides what
 * colour to render in the first place, for palettes and themes nobody has
 * opened yet.
 *
 * The bug it was written after is the one worth naming. `readable` mixes a
 * colour and then asks for the contrast of the mixture — and `parse` accepted
 * strings only, so every one of those questions answered `null`, which compares
 * false against everything. The loop ran its hundred steps, improved nothing,
 * and returned its input. Nothing threw, nothing logged, and the function is
 * *allowed* to return its input, so the only visible symptom was axe reporting
 * exactly the ratio it had reported before the fix.
 */

describe('the ratio', () => {
  /** The two endpoints of the scale, from the specification itself. */
  it('is 21 for black on white and 1 for a colour on itself', () => {
    expect(contrast('#000000', '#FFFFFF')).toBeCloseTo(21, 5);
    expect(contrast('#4CAF50', '#4CAF50')).toBeCloseTo(1, 5);
  });

  it('does not care which way round the two colours are given', () => {
    expect(contrast('#4CAF50', '#FFFFFF')).toBeCloseTo(contrast('#FFFFFF', '#4CAF50'), 10);
  });

  /**
   * Three spellings of one colour, because Vuetify's theme stores `r,g,b` and
   * `readable` passes triples back into these functions mid-calculation.
   */
  it('reads a hex, a short hex, a comma triple and an array as the same colour', () => {
    const forms = ['#ff0000', '#f00', '255,0,0', [255, 0, 0]];
    const luminances = forms.map((form) => luminance(form));
    expect(new Set(luminances).size, `${luminances}`).toBe(1);
    expect(parse('#f00')).toEqual([255, 0, 0]);
  });

  it('answers null for something that is not a colour, rather than guessing', () => {
    for (const bad of ['', 'rebeccapurple', '#12345', null, undefined, [1, 2]]) {
      expect(luminance(bad), `${bad}`).toBeNull();
    }
  });

  /** A published pair, so the implementation is checked against somebody else's
   *  arithmetic rather than against itself. */
  it('agrees with the canonical smallest passing grey', () => {
    // #767676 on white is the canonical "smallest passing grey" — 4.54:1.
    expect(contrast('#767676', '#FFFFFF')).toBeCloseTo(4.54, 2);
  });
});

describe('making a colour readable', () => {
  it('leaves a colour that already passes exactly as it was', () => {
    expect(readable('#1B2026', '#FFFFFF')).toBe('#1b2026');
  });

  /**
   * The case this exists for: every status colour, on every palette, against
   * both themes' surfaces. This is the assertion that would have failed on the
   * `parse` bug, and it is why it is written over the real palettes rather than
   * over one hand-picked green.
   */
  it('lifts every status colour of every palette to AA, in both themes', () => {
    const surfaces = [NEUTRALS[0].light.surface, NEUTRALS[0].dark.surface];
    for (const palette of STATUS_PALETTES) {
      for (const [role, colour] of Object.entries(palette.colors)) {
        for (const surface of surfaces) {
          const got = readable(colour, surface);
          expect(
            contrast(got, surface),
            `${palette.id}.${role} on ${surface} came back ${got}`
          ).toBeGreaterThanOrEqual(AA_TEXT);
        }
      }
    }
  });

  /**
   * And the same sweep once the contrast setting is allowed to move the
   * target, which is the whole of what that setting now does to these four
   * colours.
   *
   * Over every neutral family rather than graphite alone, because the surface
   * `readable` measures against is whichever family is selected — `warm`'s
   * `#FAF7F3` and `midnight`'s `#080D1A` are not the same problem, and a sweep
   * over one of them would pass while the other did not.
   *
   * Written as `>=` against the level's own target rather than against 4.5:
   * the point of three stops is that the number moves, and a test that only
   * knew about AA would go on passing if `high` silently stopped raising it.
   */
  it('reaches the target of every contrast level, on every neutral family', () => {
    for (const [name, level] of Object.entries(CONTRAST_LEVELS)) {
      for (const family of NEUTRALS) {
        for (const surface of [family.light.surface, family.dark.surface]) {
          for (const palette of STATUS_PALETTES) {
            for (const [role, colour] of Object.entries(palette.colors)) {
              const got = readable(colour, surface, level.target);
              expect(
                contrast(got, surface),
                `${name}: ${palette.id}.${role} on ${family.id} ${surface} came back ${got}`
              ).toBeGreaterThanOrEqual(level.target);
            }
          }
        }
      }
    }
  });

  /**
   * The three levels have to be ordered, and nothing in `CONTRAST_LEVELS`
   * enforces it — it is a literal somebody can edit. An unordered set would be
   * a "high contrast" that lowers the bar, which is worse than no setting.
   */
  it('offers three levels that actually climb', () => {
    const { standard, medium, high } = CONTRAST_LEVELS;
    expect(standard.target).toBeLessThan(medium.target);
    expect(medium.target).toBeLessThan(high.target);
    expect(standard.emphasis).toBeLessThan(medium.emphasis);
    expect(medium.emphasis).toBeLessThan(high.emphasis);
    expect(standard.border).toBeLessThan(medium.border);
    expect(medium.border).toBeLessThan(high.border);

    // The baseline is the one that has to be right on its own: the two above
    // it are enhancements, and this is the number axe measured.
    expect(standard.target).toBe(AA_TEXT);
  });

  /**
   * A status colour that changed hue would be a status colour that means
   * something else. Darker green, not brown.
   */
  it('keeps the hue, moving only toward black or white', () => {
    const green = readable('#4CAF50', '#FFFFFF');
    const [r, g, b] = parse(green);
    expect(g, `${green} should still be a green`).toBeGreaterThan(r);
    expect(g).toBeGreaterThan(b);
  });

  /** Toward black on a light surface, toward white on a dark one. */
  it('moves the direction the background is not', () => {
    expect(luminance(readable('#4CAF50', '#FFFFFF'))).toBeLessThan(luminance('#4CAF50'));
    expect(luminance(readable('#4CAF50', '#000000'))).toBeGreaterThanOrEqual(luminance('#4CAF50'));
  });

  /**
   * On a mid-grey neither endpoint reaches 4.5. The honest answer is the best
   * available rather than nothing at all — a colour at 4.2 is better than the
   * 1.1 it replaced, and returning null would put a `null` in a stylesheet.
   */
  it('returns its closest attempt when the target cannot be reached', () => {
    const got = readable('#808080', '#7F7F7F');
    expect(parse(got)).not.toBeNull();
    expect(contrast(got, '#7F7F7F')).toBeGreaterThan(contrast('#808080', '#7F7F7F'));
  });
});
