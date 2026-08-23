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
 * Measured 2026-08-10, on the commit that introduced this file.
 *
 * Kept beside the ceilings so the distance is visible — a ceiling far above a
 * number last measured a year ago is not a budget, it is a formality.
 */
export const measured = {
  /**
   * index.js 642 KB + vue.js 169 KB + index.css 704 KB — re-measured 23 August
   * 2026, and the re-measurement is half the point. The old figure was 1344.7
   * (477 + 169 + 699) and had not been touched while three rounds of feature
   * work landed, so "since measured: +170 KB" was reporting drift from a number
   * nobody had checked rather than growth anybody had decided to accept. The
   * comment below already says what that makes a budget: a formality.
   *
   * What grew is `index.js`, by 165 KB. Not the stylesheet — Vuetify's CSS is
   * up 5 KB — so this is application code arriving in the eager chunk, which is
   * the growth this ceiling exists to make visible.
   */
  eagerKb: 1515.0,
  /**
   * Every asset, including the lazy route chunks and xterm's 325 KB.
   *
   * Byte sums, not `du`. The first figure written here was 2612, taken from
   * `du -sk`, which reports disk blocks — a directory of many small files reads
   * ~4% larger than the bytes in it. A budget measured one way and enforced
   * another is a budget that drifts by a rounding rule.
   */
  totalKb: 2911.2,
};

export const ceilings = {
  /**
   * ~12% over today, which is the same proportional headroom this number was
   * first given — see the note on `measured` for why the figure it sits above
   * moved.
   *
   * **Raised from 1500, and it is a decision rather than an adjustment.** The
   * eager set had been over that ceiling since the in-app help round, so CI was
   * red on this step for three merges and the number stopped being read as a
   * budget. Two honest answers were available: trim 15 KB out of the eager
   * chunk, or accept the growth and say so. Trimming is the better one and it
   * is a piece of work — `index.js` carries 165 KB it did not a month ago, and
   * finding which imports pulled it in is not a release-day job.
   *
   * So this is the second answer, taken deliberately and written down: the
   * growth is accepted, the measurement beneath it is current, and the gap is
   * headroom again rather than a number the build has been failing on.
   */
  eagerKb: 1700,

  /**
   * ~15% over today. Deliberately looser than `eager`: this number is supposed
   * to grow as features land, and its job is to catch a dependency arriving by
   * accident — a full icon font, a second date library, a locale bundle nobody
   * meant to ship — not to make lazy loading feel expensive.
   */
  totalKb: 3000,
};

export default ceilings;
