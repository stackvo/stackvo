import { describe, it, expect } from 'vitest';
import { TONES, ramp } from '@/lib/tones.js';
import { luminance } from '@/lib/contrast.js';
import { PRIMARY_SWATCHES } from '@/lib/appearance.js';

/**
 * The one thing in this application drawn with Material's own engine.
 *
 * Everything else derives colour with arithmetic this repository owns, because
 * one derived colour does not justify 42 KB on the boot path. A *ramp* does not
 * survive that treatment: its entire claim is that the steps are perceptually
 * even, and the HSL lightness `harmonise` works in is not perceptual. So the
 * assertions here are about evenness and order — the properties that would be
 * silently wrong if somebody swapped the engine back out for a cheaper one.
 */
describe('the tonal ramp', () => {
  it('runs light to dark, without a step that goes the wrong way', () => {
    for (const swatch of PRIMARY_SWATCHES) {
      const steps = ramp(swatch, 'P');
      expect(steps).toHaveLength(TONES.length);

      const light = steps.map((s) => luminance(s.hex));
      for (let i = 1; i < light.length; i += 1) {
        expect(
          light[i],
          `${swatch}: ${steps[i - 1].code} → ${steps[i].code} did not get darker`
        ).toBeLessThan(light[i - 1]);
      }
    }
  });

  /**
   * The property the engine is here for. Tone is perceptual lightness, so a
   * ten-point step should look like a ten-point step wherever it is taken —
   * which relative luminance does not measure directly, but *ordering by tone*
   * does: tone 40 of a green and tone 40 of a blue must be nearer each other in
   * lightness than tone 40 and tone 80 of either.
   */
  it('puts the same tone of any two colours at a similar lightness', () => {
    for (const tone of TONES) {
      const across = PRIMARY_SWATCHES.map((swatch) =>
        luminance(ramp(swatch, 'P').find((s) => s.tone === tone).hex)
      );
      const spread = Math.max(...across) - Math.min(...across);

      // Generous, because chroma does move luminance and the swatches span the
      // wheel. The point is that it stays far below one tone step, which for
      // the mid tones is roughly 0.1 of luminance.
      expect(
        spread,
        `tone ${tone} varies by ${spread.toFixed(3)} across the swatches`
      ).toBeLessThan(0.09);
    }
  });

  it('labels each step the way Material names it', () => {
    expect(ramp('#1976D2', 'P').map((s) => s.code)).toEqual(TONES.map((t) => `P-${t}`));
    expect(ramp('#1976D2', 'S')[0].code).toBe('S-90');
  });

  it('emits real colours', () => {
    for (const step of ramp('#1976D2', 'P')) expect(step.hex).toMatch(/^#[0-9a-f]{6}$/);
  });

  /**
   * Called from a computed that follows a live setting. A hand-edited
   * preference must cost the strip, not the settings page.
   */
  it('answers with nothing rather than throwing on something that is not a colour', () => {
    for (const bad of ['', 'rebeccapurple', '#12345', null, undefined, {}]) {
      expect(ramp(bad, 'P'), `${bad}`).toEqual([]);
    }
  });
});
