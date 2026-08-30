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
 * Measured 2026-08-29, on a clean `npm run build`.
 *
 * Kept beside the ceilings so the distance is visible — a ceiling far above a
 * number last measured a year ago is not a budget, it is a formality.
 */
export const measured = {
  /**
   * index.js 690 KB + vue.js 173 KB + index.css 404 KB.
   *
   * **Down 266 KB, and it is a trim rather than drift.** The stylesheet was 704
   * KB, of which 408 was Material Design Icons declaring 7,448 glyph rules for
   * an application that names 356 of them. The build now emits only the rules
   * something here can reach — see `mdiUsedIconsOnly` in `vite.config.js` and
   * the list in `tools/mdi-icons.mjs` — which takes that file to 32 KB.
   *
   * The 2.4 KB it went back up is the scan learning to read `src-tauri/src`.
   * Eighteen icons are named only there — the terminal, editor and browser
   * catalogues live in `apps.rs` — so the first version of this trim was
   * stripping every icon those three pickers draw. Kilobytes bought back three
   * lists of blank squares.
   *
   * `index.js` grew 48 KB over the same period, which is the growth this
   * ceiling exists to make visible and is now visible against a smaller number
   * rather than hidden inside a larger one.
   */
  eagerKb: 1248.8,
  /**
   * Every asset, including the lazy route chunks and xterm's 333 KB.
   *
   * Byte sums, not `du`. The first figure written here was 2612, taken from
   * `du -sk`, which reports disk blocks — a directory of many small files reads
   * ~4% larger than the bytes in it. A budget measured one way and enforced
   * another is a budget that drifts by a rounding rule.
   *
   * The largest single asset left is the icon **font**, 394 KB of woff2 holding
   * all 7,448 glyphs. Subsetting it needs a font toolchain where the stylesheet
   * needed no dependency at all, so it is written down here as the next piece
   * of this work rather than done badly.
   */
  totalKb: 2705.9,
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
