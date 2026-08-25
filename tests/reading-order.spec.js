import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

/**
 * Where the screen says one order and the markup says another.
 *
 * Y-1's third question, and the one of its three that is a **fact** rather than
 * a judgement: does the reading order match the visual order? A screen reader
 * and a keyboard follow the markup. Sighted readers follow the layout. CSS is
 * allowed to disagree with the markup, and every place it does is a place where
 * those two audiences are handed different sequences — WCAG 1.3.2, Meaningful
 * Sequence.
 *
 * There are exactly three ways to cause it in this application, and none of
 * them is visible from the thing they affect:
 *
 *   * `order:` on a flex or grid child,
 *   * `flex-direction: row-reverse` / `column-reverse`,
 *   * a positive `tabindex`, which reorders the keyboard without touching
 *     either the markup or the screen.
 *
 * ## What this checks, and why it is not "never do that"
 *
 * Re-ordering is legitimate. A rail that becomes a strip above the pane it
 * selects is a good narrow layout, and the markup that keeps content first is a
 * defensible reading order. What is not legitimate is doing it **silently**: a
 * divergence nobody wrote down is one no reviewer can find, which is exactly
 * why the one this file was written against survived — the chooser in the new
 * project drawer was pulled above the form with `order: -1`, so a screen reader
 * read the whole form and only then reached the control that decides what those
 * fields mean.
 *
 * So the rule is a sentence, not a ban: every re-ordering carries a note saying
 * which of the two sequences is the meaningful one. That is the list a human
 * audit needs and did not have.
 *
 * Positive `tabindex` has no such escape. It reorders the keyboard against both
 * the markup and the screen, and there is no layout it buys.
 */

const SRC = resolve(import.meta.dirname, '../src');

/** Every `.vue` and `.css` under `src/`, with its path. */
function sources(dir = SRC) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return sources(path);
    return /\.(vue|css)$/.test(entry) ? [[relative(SRC, path), readFileSync(path, 'utf8')]] : [];
  });
}

const FILES = sources();

/** The marker a re-ordering has to carry, in the comment above it. */
const NOTE = 'reading order';

/** How far above a rule the note may sit, in lines. */
const REACH = 12;

/**
 * Is there a note within reach above this line?
 *
 * Lines rather than a comment parser: the rules here are inside `<style>`
 * blocks and CSS files, both of which use `/* … *\/`, and the note is prose a
 * person wrote near the thing it explains. A parser would be exact about a
 * question that is not exact.
 */
function noted(lines, index) {
  const from = Math.max(0, index - REACH);
  // Whitespace collapsed before the match: the note is prose and prose wraps,
  // and the first version of this failed a file whose comment said exactly the
  // right thing across two lines — "keeps the markup in reading\n   order".
  return lines.slice(from, index).join(' ').toLowerCase().replace(/\s+/g, ' ').includes(NOTE);
}

/** Every re-ordering, with the file and line it is on. */
function reorderings() {
  const found = [];
  const RULES = [
    { re: /^\s*order:\s*-?\d/, what: 'order' },
    { re: /flex-direction:\s*(row|column)-reverse/, what: 'flex-direction reverse' },
  ];

  for (const [file, text] of FILES) {
    const lines = text.split('\n');
    lines.forEach((line, index) => {
      for (const rule of RULES) {
        if (rule.re.test(line)) {
          found.push({ file, line: index + 1, what: rule.what, noted: noted(lines, index) });
        }
      }
    });
  }
  return found;
}

describe('meaningful sequence (WCAG 1.3.2)', () => {
  /**
   * The guard on the guard. If the patterns stop matching, every assertion
   * below passes by finding nothing — which is the failure mode a scanner has,
   * and the one that would make this file look like coverage while being none.
   */
  it('still finds the re-orderings this application has', () => {
    expect(
      reorderings().length,
      'the scanner matched nothing at all, which means it stopped matching \
rather than that the application stopped re-ordering'
    ).toBeGreaterThan(0);
  });

  it('has a note on every place the screen and the markup disagree', () => {
    const silent = reorderings()
      .filter((r) => !r.noted)
      .map((r) => `${r.file}:${r.line} (${r.what})`);

    expect(
      silent,
      `these re-order the layout against the markup with nothing saying which \
sequence is the meaningful one. A screen reader and a keyboard follow the \
markup; the eye follows the layout. Write a comment within ${REACH} lines \
containing "${NOTE}" saying which of the two is correct here — the point is \
that a reviewer can find it, not that re-ordering is forbidden.`
    ).toEqual([]);
  });

  /**
   * A positive `tabindex` reorders the keyboard against both the markup and the
   * screen, and buys no layout at all. `0` and `-1` are not re-ordering: they
   * put an element in or out of the sequence, at the place it already occupies.
   */
  it('never reorders the keyboard with a positive tabindex', () => {
    const offenders = [];
    for (const [file, text] of FILES) {
      text.split('\n').forEach((line, index) => {
        if (/tabindex="[1-9]/.test(line)) offenders.push(`${file}:${index + 1}`);
      });
    }
    expect(
      offenders,
      'a positive tabindex moves an element in the keyboard sequence without \
moving it in the markup or on screen, so all three orders disagree'
    ).toEqual([]);
  });
});
