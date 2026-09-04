/**
 * The coverage floors, in one place because two places drift.
 *
 * ## Why there is a floor now, when there deliberately was not one before
 *
 * The first version of this repository's coverage setup reported and never
 * failed, and the comment in `vitest.config.js` argued the case: a floor picked
 * before anyone has seen the number is either low enough to bless the gap or
 * high enough to fail on the first run, and both teach the same lesson — write
 * tests that move a percentage.
 *
 * That argument had an expiry date, and it has passed. The numbers have been
 * looked at across several rounds of work (61.60 → 63.12 → 63.34 → 64.34 →
 * 68.05 on the Rust side; 30.70 → 89.65 → 92.20 on the front end), so a floor
 * set today is set from evidence rather than from taste. Without one, the
 * measurement answers "how much is covered" and never "did that just get
 * worse", which is the only question a build can usefully ask.
 *
 * ## What a floor is for here
 *
 * It is a regression alarm, not a target. Nobody should write a test to move
 * these numbers; the number moves because tests were written for a reason. The
 * floors sit **below** today's measurement on purpose, and the gap is not
 * slack — it is the room the two facts below need:
 *
 *  1. **Coverage is platform-dependent on the Rust side.** These numbers come
 *     from macOS; CI measures on Ubuntu, where the `#[cfg(target_os = "macos")]`
 *     branches of `elevate.rs`, `certs.rs` and `apps.rs` compile out and their
 *     Linux counterparts compile in. The recorded history of this repository has
 *     the two within about a point of each other, and one point of drift must
 *     not turn a green build red.
 *  2. **A large, well-tested module still lowers the percentage while it is
 *     being written.** A floor with no headroom makes the first commit of a
 *     feature fail, which pushes people to write the test before the thing it
 *     tests is finished — the exact inversion this file is meant to prevent.
 *
 * Raising a floor after a real gain is a deliberate act. Lowering one is a
 * decision with a reason, and the reason belongs in the commit message.
 *
 * ## How far below, in numbers rather than in feel
 *
 * The floors were first set about four points under the measurement, and by
 * the time the numbers were looked at again the gap had grown to **eight** —
 * a regression of seven points would have passed in silence, which is not a
 * regression alarm, it is a decoration. So the distance is now arithmetic
 * rather than a habit, and each floor is the measurement minus what the two
 * facts above actually cost:
 *
 *  * **Platform**, Rust only: about one point, which is where this
 *    repository's history has put macOS and Ubuntu. The front end is measured
 *    by the same jsdom everywhere and pays nothing.
 *  * **A module in flight**: its uncovered lines over the tree's total. The
 *    Rust core is 59.5k counted lines, so a 1,200-line module written before
 *    its tests costs two points. The front end is 10.3k *executable* lines
 *    (see the next section for why that number used to read 37.6k), so a
 *    pane of 200 executable lines written before its tests costs two.
 *    Branches swing wider than lines on a new component full of
 *    conditionals, so that one is given three.
 *
 * Which makes the blind spot three points on the Rust side and two on the
 * front end, in place of eight and seven. Re-measure before moving one of
 * these; the whole argument rests on the pair below being the same age.
 *
 * ## The front-end number fell twenty points on 2026-09-04, and nothing got
 * ## worse
 *
 * Vitest 3's v8 provider counted every line of every file in `include` —
 * comments, blank lines, the `<template>` — and marked a file's lines covered
 * once its module had been evaluated. A component nobody rendered read as
 * 100%: `CloseDialog.vue`, which `app-shell.spec.js` stubs out, stood at
 * 72 lines, all covered. Vitest 4 remaps V8's byte ranges through the AST,
 * so only executable statements count and only executed ones count as
 * covered. Measured on the same tree, same day, with both providers:
 *
 *     vitest 3.2.7   37,595 lines counted, 34,702 covered   92.3%
 *     vitest 5.0.0   10,312 lines counted,  7,397 covered   71.7%
 *
 * The second row is the truth the first was hiding, and it is the one the
 * floors below now guard. `CloseDialog.vue` reads 0%, which is what a stubbed
 * component is. The way to move this number is the way it was always meant
 * to move: a test that renders something.
 */

/**
 * Measured on macOS, on a clean tree — Rust on 2026-08-29 with
 * `npm run test:rs:coverage`; the front end on 2026-09-04 with
 * `npm run test:js:coverage` under Vitest 5, the day the provider changed
 * what a line is (the section above).
 *
 * Kept next to the floors so the distance between them is visible: a floor
 * three points under a number that was last measured a year ago is not a floor,
 * it is a guess with a decimal point. Which is how the last pair drifted — the
 * numbers here stood at the 2026-08-07 measurement while the tree went on
 * gaining four points, and the floors kept guarding a tree that no longer
 * existed.
 */
export const measured = {
  rust: { lines: 68.05, functions: 64.98, regions: 68.46 },
  frontend: { lines: 71.73, statements: 70.59, branches: 62.14, functions: 63.26 },
};

export const floors = {
  /**
   * `cargo llvm-cov`'s line coverage for the whole crate.
   *
   * Lines rather than regions or functions: regions count each arm of every
   * `match` and swing with refactors that change nothing about what is tested,
   * and functions counts a one-line accessor the same as `run_operation`.
   *
   * 68.05 measured, less one point of platform and two for a module in
   * flight. Leaves about two points of margin under what Ubuntu is expected
   * to report, which is the number CI actually compares.
   */
  rust: { lines: 65 },

  /**
   * v8's numbers for `src/**`, as reported by `vitest run --coverage`.
   *
   * **`functions` is not floored, and that is not an oversight.** v8 counts
   * every arrow function as a function, and a Vue SFC compiles its template
   * into a render function full of them — inline handlers, slot props, `v-for`
   * bodies. The measured 61% does not mean two fifths of the front end's
   * behaviour is unexercised; it means two fifths of the closures a compiler
   * emitted were never called, and several of those cannot be called from
   * jsdom at all. A floor on a number nobody can act on teaches people to
   * ignore a failing gate.
   *
   * `statements` and `lines` were the same figure under the old
   * instrumentation; the AST-aware provider separates them by a point (a
   * line can hold two statements). Both are floored, two points under their
   * own measurement.
   *
   * 71.73 / 70.59 / 62.14 measured, less two for a module in flight (three
   * for branches). Not the 90 / 90 / 78 of the previous provider: those
   * guarded a number that counted comments as covered code.
   */
  frontend: { lines: 69.5, statements: 68.5, branches: 59 },
};

export default floors;
