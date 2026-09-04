/**
 * The bundle ceilings, in one place because two places drift.
 *
 * The same argument `coverage-floors.mjs` makes, pointed the other way: a
 * coverage number must not fall and a bundle must not rise, and neither
 * question can be asked by a build that only prints the figure. Vite has
 * printed these sizes on every build since the first commit and nothing has
 * ever read them.
 *
 * ## Raw bytes, not gzip — and that is not the usual answer
 *
 * Every bundle-size tool defaults to gzip because the number that matters on
 * the web is what crosses a network. **Nothing here crosses a network.** These
 * files are read off local disk by a WebView that Tauri points at the app
 * bundle, so the compressed size describes a transfer that never happens. What
 * the user actually pays is bytes read and bytes parsed, and both are raw.
 *
 * Budgeting the gzip figure would also hide the specific regression this is
 * most likely to catch: a large, highly compressible dependency — a locale
 * table, an icon set, a syntax-highlighting grammar — barely moves gzip and
 * moves parse time a lot.
 *
 * ## Two numbers, because they answer different questions
 *
 * `eager` is what the window loads before it can paint anything: the entry
 * chunk, whatever it preloads, and the stylesheet. This is the one that is felt,
 * and it is the one a careless static `import` quietly ruins.
 *
 * `total` is every emitted asset. It exists so that route chunks and on-demand
 * imports cannot grow without limit just because they are not on the critical
 * path — `xterm` is 325 KB of exactly that kind, and it is right that it costs
 * nothing until somebody opens a shell and right that it still counts somewhere.
 *
 * Which files are eager is **read from `dist/index.html`** rather than listed
 * here. The build already writes down the answer, and a hand-kept list is a
 * second copy that goes stale the first time a chunk is renamed.
 *
 * ## The ceilings sit above the measurement, and the gap is not slack
 *
 * The same reasoning as the coverage floors. Room is needed for a dependency
 * bump that lands a few kilobytes, and for the first commit of a feature whose
 * code is in before the code that trims it. A ceiling with no headroom is one
 * that fails for reasons nobody chose, and those get raised without being read.
 *
 * Raising a ceiling is a deliberate act, and the reason belongs in the commit
 * message. "It went over" is not a reason.
 */

/**
 * Measured 2026-08-31, on a clean `npm run build`.
 *
 * Kept beside the ceilings so the distance is visible — a ceiling far above a
 * number last measured a year ago is not a budget, it is a formality.
 *
 * ## Read the gap before the numbers
 *
 * These were two days stale and the drift was not small: `eager` was recorded
 * at 1248.8 and the tree had been at 1333.4 for some time, so every build was
 * printing "+85 KB since measured" and nobody could tell which commit had
 * spent it. That is the failure mode this docblock warns about, arriving from
 * the other direction — not a ceiling too far above the number, but a number
 * too far behind the tree.
 *
 * The headroom that leaves is now thin, and it is written here rather than
 * fixed here: `eager` has 4.2% under its ceiling and `total` 2.2%, against the
 * ~12% and ~11% the ceilings below describe as the proportion they have always
 * been given. Raising a ceiling is a deliberate act with its reason in the
 * commit message, and "the measurement caught up with the tree" is not by
 * itself one. The next commit to need the room is the one that should argue
 * for it.
 */
export const measured = {
  /**
   * Re-measured 2026-09-04 under Vite 8: 1327.4 KB, down 13.8 on the figure
   * below. Rolldown splits the eager set differently — a `rolldown-runtime`
   * chunk and a preload helper appear, `vue.js` is 160 KB, the CSS is 314 —
   * and Oxc minifies a little tighter than esbuild did. Nothing in `src/`
   * moved for it. The paragraph below is the previous measurement's story and
   * still explains what is on this path.
   *
   * index.js 766 KB + vue.js 169 KB + index.css 406 KB.
   *
   * Up 92.4 KB on the last figure written here, and almost none of it is the
   * work that took this measurement. 84.6 of the 92.4 were already in the tree
   * before it started — the drift the docblock above describes. The remainder
   * is roughly eight kilobytes of theme derivation in `lib/appearance.js` and
   * `lib/contrast.js`, which are on this path deliberately: the theme is
   * applied before the first paint so the window never shows the wrong palette.
   *
   * The icon trim that took the stylesheet from 704 KB to 32 is still holding —
   * `check-bundle.mjs` prints the kept-rule count on every run, and the build
   * emits only the 385 rules something in `src/` or `src-tauri/src/` names.
   */
  eagerKb: 1327.4,
  /**
   * Every asset, including the lazy route chunks and xterm's 333 KB.
   *
   * Byte sums, not `du`. The first figure written here was 2612, taken from
   * `du -sk`, which reports disk blocks — a directory of many small files reads
   * ~4% larger than the bytes in it. A budget measured one way and enforced
   * another is a budget that drifts by a rounding rule.
   *
   * Up 227 KB, and unlike `eager` most of this one *is* deliberate: 66 KB of it
   * is `@material/material-color-utilities`, which draws the tonal ramp on the
   * appearance page. It is the reason the two numbers moved so differently —
   * the package is reached only from `lib/tones.js`, which only the settings
   * pane imports, so it lands in the lazily-loaded Settings chunk (245 KB) and
   * costs the first paint nothing. That separation is the whole design, and
   * these two figures are what proves it held: `eager` moved 0.5 KB when it
   * landed and `total` moved 66.
   *
   * The largest single asset is still the icon **font**, 394 KB of woff2
   * holding all 7,448 glyphs. Subsetting it needs a font toolchain where the
   * stylesheet needed no dependency at all, so it stays written down here as
   * the next piece of this work rather than done badly — and it is now also
   * where the headroom for the next feature is going to have to come from.
   */
  totalKb: 2938.0,
};

export const ceilings = {
  /**
   * ~12% over today, which is the proportional headroom this number has always
   * been given.
   *
   * **Lowered from 1700, and that is the other half of a decision made a round
   * ago.** The last time this ceiling was in the way, two honest answers were
   * available — trim, or accept the growth and say so — and the note here
   * recorded taking the second while calling the first the better one. It was
   * still the better one. The trim has now been done and it was worth 268 KB,
   * which is eighteen times what was being argued over; a ceiling left at 1700
   * over a measurement of 1246 would have been 36% of slack, and slack is what
   * a budget stops being an alarm inside of.
   */
  eagerKb: 1400,

  /**
   * ~11% over today. Deliberately looser than `eager`: this number is supposed
   * to grow as features land, and its job is to catch a dependency arriving by
   * accident — a full icon font, a second date library, a locale bundle nobody
   * meant to ship — not to make lazy loading feel expensive.
   *
   * Left where it was rather than lowered with the measurement. The trim that
   * moved the number was the icon stylesheet, and the icon *font* is still
   * whole: this ceiling has a 300 KB drop coming that nobody has done yet, and
   * moving it down now only to move it down again is two decisions where one
   * will do.
   */
  totalKb: 3000,
};

export default ceilings;
