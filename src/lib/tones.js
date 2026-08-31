import { Hct, TonalPalette, argbFromHex, hexFromArgb } from '@material/material-color-utilities';

/**
 * The tonal ramp behind a colour — Material's, not an approximation of it.
 *
 * ## Why this file is on its own, and why nothing at boot may import it
 *
 * It is the only place in this application that pulls
 * `@material/material-color-utilities`, and that package is about 42 KB. The
 * theme is applied before the first paint — `stores/appearance.js` calls
 * `applyAppearance` during boot, deliberately, so the window never flashes the
 * wrong palette — which means every byte `lib/appearance.js` reaches is a byte
 * on the critical path. Forty-two of them for a strip of swatches on a settings
 * page nobody has opened would be the whole budget argument lost in one import.
 *
 * So the split is the point: `lib/appearance.js` derives the colours the
 * application *renders*, with arithmetic it owns and `contrast.js` can measure;
 * this file derives the colours the settings page *explains*, and is reached
 * only from `AppearancePane.vue`, which lives in the lazily-loaded Settings
 * chunk. `tools/check-bundle.mjs` is what holds that apart — the eager figure
 * must not move when this lands.
 *
 * ## Why the real engine rather than the HSL trick used elsewhere
 *
 * `harmonise` rotates a hue in HSL and corrects the result by measuring
 * luminance, which is enough for one derived colour and costs nothing. A
 * *ramp* is a different problem: its whole claim is that the steps are
 * perceptually even, and HSL lightness is not perceptual — an HSL ramp drawn
 * next to Studio's would be visibly uneven at exactly the tones a designer
 * looks at. A ramp that is not tonal is not worth showing, so this one is the
 * real thing or it is nothing.
 */

/**
 * The tones Material publishes a ramp at, light to dark.
 *
 * Nine rather than the thirteen the spec defines: 0 and 100 are black and white
 * for every colour there is, and 95/99 are indistinguishable from 90 at swatch
 * size. These are the nine Vuetify Studio shows, which is also the set anybody
 * comparing the two will expect.
 */
export const TONES = [90, 80, 70, 60, 50, 40, 30, 20, 10];

/**
 * One ramp, as swatches ready to draw.
 *
 * `code` is Material's own shorthand — `P-40` is the primary palette at tone
 * 40 — and it is carried through rather than generated in the template because
 * it is the label a designer reads out to somebody else.
 *
 * Returns an empty list for anything that is not a colour rather than throwing:
 * this is called from a computed that follows a live setting, and a settings
 * page that goes blank because a preference was hand-edited is a worse failure
 * than a missing strip.
 */
export function ramp(color, code) {
  let palette;
  try {
    palette = TonalPalette.fromHct(Hct.fromInt(argbFromHex(String(color))));
  } catch {
    return [];
  }

  return TONES.map((tone) => ({
    code: `${code}-${tone}`,
    tone,
    hex: hexFromArgb(palette.tone(tone)),
  }));
}
