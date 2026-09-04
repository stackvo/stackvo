import { describe, it, expect } from 'vitest';
import { readdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { columnsOf, htmlOf, linesOf, sgr, widthOf } from '../tools/screenshots/ansi-frame.mjs';
import {
  BRANCHES,
  PARENT,
  PLANNED_BRANCH,
  WORKTREES,
  WORKTREE_STAGE,
} from '../tools/screenshots/worktree-stage.mjs';

/**
 * The parts of `npm run screenshots` that can be held still (#101).
 *
 * The tool itself builds, serves and shoots, and nothing about that belongs
 * in a unit test. What does is the two pieces the issue added: the stage the
 * worktree screen is shot against, and the reading of a terminal frame into
 * cells. Both are pure, both rot silently — a stage that stops matching the
 * contract still draws *something*, and a renderer that drops a code it does
 * not know draws a frame the screen never showed — and neither would be
 * noticed until somebody looked at the picture.
 *
 * The last block is about the gallery: a picture in the directory that the
 * index does not list is one nobody finds, and a picture the index lists
 * that is not in the directory is a broken image on the README.
 */

const ROOT = resolve(import.meta.dirname, '..');
const SHOTS = resolve(ROOT, 'docs/screenshots');

// ------------------------------------------------------------- the frame

/** A frame as `tui::draw` writes one: home, styled lines, clear below. */
const FRAME =
  '\x1b[H' +
  '\x1b[1mStackVo\x1b[0m   \x1b[2mengine up · 2 projects\x1b[0m\x1b[K\r\n' +
  '\x1b[2m────\x1b[0m\x1b[K\r\n' +
  '\x1b[K\r\n' +
  '▸ \x1b[32mup  \x1b[0m  \x1b[1mshop\x1b[0m  \x1b[2mshop.loc\x1b[0m\x1b[K\r\n' +
  '  \x1b[2mdown\x1b[0m  storefront  \x1b[2mstorefront.loc\x1b[0m\x1b[K\r\n' +
  '\x1b[J';

describe('reading a terminal frame', () => {
  it('turns the frame into lines of styled runs and nothing else', () => {
    const lines = linesOf(FRAME);
    expect(lines).toHaveLength(5);
    expect(lines[0]).toEqual([
      { text: 'StackVo', bold: true, dim: false, colour: null },
      { text: '   ', bold: false, dim: false, colour: null },
      { text: 'engine up · 2 projects', bold: false, dim: true, colour: null },
    ]);
    // The blank line is a line: dropping it would move every row up one.
    expect(lines[2]).toEqual([]);
    expect(lines[3][1]).toEqual({ text: 'up  ', bold: false, dim: false, colour: 'green' });
    expect(lines[3][3]).toEqual({ text: 'shop', bold: true, dim: false, colour: null });
  });

  it('draws nothing for the cursor and erase sequences', () => {
    // Home, clear-to-end and clear-below are the three `draw` writes; on a
    // blank surface they are invisible and a renderer that printed their
    // letters would show `H`, `K` and `J` in the picture.
    const text = linesOf(FRAME)
      .flat()
      .map((run) => run.text)
      .join('');
    expect(text).not.toMatch(/[HKJ]\b/);
    expect(linesOf('\x1b[H\x1b[J')).toEqual([]);
  });

  it('knows the six codes the screen uses and ignores the rest', () => {
    const plain = { bold: false, dim: false, colour: null };
    expect(sgr(plain, '1')).toEqual({ ...plain, bold: true });
    expect(sgr(plain, '2')).toEqual({ ...plain, dim: true });
    expect(sgr(plain, '32')).toEqual({ ...plain, colour: 'green' });
    expect(sgr(plain, '33')).toEqual({ ...plain, colour: 'yellow' });
    expect(sgr(plain, '31')).toEqual({ ...plain, colour: 'red' });
    expect(sgr({ bold: true, dim: true, colour: 'red' }, '0')).toEqual(plain);
    // `\x1b[m` is the short reset; `22` is "normal intensity", which is
    // neither bold nor dim; `39` is the default foreground.
    expect(sgr({ bold: true, dim: true, colour: 'red' }, '')).toEqual(plain);
    expect(sgr({ bold: true, dim: true, colour: 'red' }, '22')).toEqual({
      ...plain,
      colour: 'red',
    });
    expect(sgr({ ...plain, colour: 'red' }, '39')).toEqual(plain);
    // Backgrounds, 256-colour and italics are not written by `draw`.
    expect(sgr(plain, '41;3;38;5;200')).toEqual(plain);
    // Several at once, in one sequence.
    expect(sgr(plain, '1;32')).toEqual({ ...plain, bold: true, colour: 'green' });
  });

  it('reads a bare newline the way it reads a raw-mode one', () => {
    // A frame captured through a cooked terminal has `\n` where raw mode
    // wants `\r\n`; the rows are the same rows.
    expect(linesOf('a\r\nb\r\n')).toEqual(linesOf('a\nb\n'));
    expect(linesOf('a\r\nb')).toHaveLength(2);
  });

  it('counts cells in characters, so a box-drawing rule is as wide as it looks', () => {
    const lines = linesOf(FRAME);
    expect(columnsOf(lines[1])).toBe(4);
    expect(columnsOf(lines[0])).toBe('StackVo   engine up · 2 projects'.length);
    // The widest line is the storefront row, and the width is its cells with
    // the escapes around `down` and the domain not counted.
    expect(widthOf(lines)).toBe('  down  storefront  storefront.loc'.length);
    expect(widthOf([])).toBe(0);
  });

  it('draws the runs as spans and leaves plain text bare', () => {
    const html = htmlOf(linesOf(FRAME), { columns: 80, rows: 24 });
    expect(html).toContain('<span class="b">StackVo</span>');
    expect(html).toContain('<span class="d">engine up · 2 projects</span>');
    expect(html).toContain('<span class="c-green">up  </span>');
    expect(html).toContain('  storefront  ');
    expect(html).not.toContain('<span class="">');
    // The terminal's size is the font's own cell, not a pixel guess.
    expect(html).toContain('width:80ch');
    expect(html).toContain(`height:${24 * 22}px`);
  });

  it('escapes what a frame could contain that HTML would read', () => {
    const html = htmlOf(linesOf('<shop> & co\r\n'));
    expect(html).toContain('&lt;shop&gt; &amp; co');
    expect(html).not.toContain('<shop>');
  });
});

// ---------------------------------------------------------- the worktrees

describe('the staged worktrees', () => {
  it('are the same rows on the projects page and the detail page', () => {
    // `worktree_list` is what puts "branch of shop" on a projects row and
    // `worktree_support.worktrees` is what the detail page lists. Two arrays
    // would be two branches of one project that only one page knows about.
    expect(WORKTREE_STAGE.worktree_list).toBe(WORKTREES);
    expect(WORKTREE_STAGE.worktree_support.worktrees).toBe(WORKTREES);
    expect(WORKTREES.length).toBeGreaterThanOrEqual(2);
  });

  it('carry every field of a WorktreeRow, in the shape the contract declares', () => {
    for (const row of WORKTREES) {
      // The record's own fields, flattened in.
      for (const key of ['name', 'parent', 'branch', 'domain', 'path', 'createdAt']) {
        expect(typeof row[key], `${row.name}.${key}`).toBe('string');
      }
      expect(row.env).toEqual(expect.any(Object));
      // What is true right now, beside them.
      for (const key of ['exists', 'orphaned', 'isolated', 'expired']) {
        expect(typeof row[key], `${row.name}.${key}`).toBe('boolean');
      }
      expect([true, false, null]).toContain(row.dirty);
      if (row.database) {
        expect(typeof row.database.instance).toBe('string');
        expect(typeof row.database.name).toBe('string');
      }
    }
  });

  it('branch from shop and answer under its hostname', () => {
    for (const row of WORKTREES) {
      expect(row.parent).toBe(PARENT.name);
      expect(row.name.startsWith(`${PARENT.name}-`)).toBe(true);
      // A subdomain of the parent's, so it stays inside the parent's
      // wildcard route and certificate — `worktree.rs`'s own rule.
      expect(row.domain.endsWith(`.${PARENT.domain}`)).toBe(true);
      expect(row.domain).toMatch(/^[a-z0-9-]+\./);
      expect(row.path.endsWith(`/${row.name}`)).toBe(true);
    }
  });

  it('are marked checked out in the list the form offers, and the planned one is not', () => {
    const taken = new Set(BRANCHES.filter((b) => b.checkedOut).map((b) => b.name));
    for (const row of WORKTREES) expect(taken.has(row.branch), row.branch).toBe(true);
    expect(taken.has(PARENT.branch)).toBe(true);
    expect(BRANCHES.filter((b) => b.current).map((b) => b.name)).toEqual([PARENT.branch]);

    const planned = BRANCHES.find((b) => b.name === PLANNED_BRANCH);
    expect(planned).toBeDefined();
    expect(planned.checkedOut).toBe(false);
  });

  it('preview the branch the shot types, on the same rules', () => {
    const plan = WORKTREE_STAGE.worktree_plan;
    expect(plan.branch).toBe(PLANNED_BRANCH);
    expect(plan.parent).toBe(PARENT.name);
    expect(plan.possible).toBe(true);
    expect(plan.refused).toBeNull();
    expect(plan.domain.endsWith(`.${PARENT.domain}`)).toBe(true);
    expect(plan.name.startsWith(`${PARENT.name}-`)).toBe(true);
  });

  it('show both states the pane draws differently', () => {
    // One kept branch with its own copied database and uncommitted work; one
    // sandbox with a duration and no database. Neither has gone wrong.
    const kept = WORKTREES.find((row) => row.database);
    const sandbox = WORKTREES.find((row) => row.remainingMinutes !== undefined);
    expect(kept).toBeDefined();
    expect(sandbox).toBeDefined();
    expect(kept).not.toBe(sandbox);

    expect(kept.isolated).toBe(true);
    expect(kept.dirty).toBe(true);
    expect(kept.database.seededFrom).toBeDefined();
    expect(kept.expiresAt).toBeUndefined();

    expect(typeof sandbox.expiresAt).toBe('string');
    expect(sandbox.remainingMinutes).toBeGreaterThan(0);
    expect(sandbox.database).toBeNull();

    for (const row of WORKTREES) {
      expect(row.orphaned).toBe(false);
      expect(row.expired).toBe(false);
      expect(row.exists).toBe(true);
    }
  });

  it('name an instance the support answer offers', () => {
    const offered = new Set(WORKTREE_STAGE.worktree_support.instances.map((i) => i.id));
    for (const row of WORKTREES.filter((r) => r.database)) {
      expect(offered.has(row.database.instance), row.database.instance).toBe(true);
    }
    expect(WORKTREE_STAGE.worktree_support.reason).toBeNull();
    expect(WORKTREE_STAGE.worktree_support.record).toBeNull();
  });
});

// -------------------------------------------------------------- the index

describe('the screenshot index', () => {
  const onDisk = readdirSync(SHOTS)
    .filter((f) => f.endsWith('.png'))
    .sort();
  const index = readFileSync(resolve(SHOTS, 'README.md'), 'utf8');
  const listed = [...new Set([...index.matchAll(/href="([^"/]+\.png)"/g)].map((m) => m[1]))].sort();

  it('lists every picture in the directory, and nothing that is not there', () => {
    expect(listed).toEqual(onDisk);
  });

  it('has the two the browser could not take (#101)', () => {
    expect(onDisk).toContain('project-detail-worktrees.png');
    expect(onDisk).toContain('tui.png');
  });

  it('is only pointed at from the READMEs where a file exists', () => {
    for (const readme of ['README.md', 'README_TR.md', 'docs/screenshots/README.md']) {
      const text = readFileSync(resolve(ROOT, readme), 'utf8');
      const refs = [...text.matchAll(/(?:docs\/screenshots\/|href=")([\w-]+\.png)/g)].map(
        (m) => m[1]
      );
      expect(refs.length, readme).toBeGreaterThan(0);
      for (const ref of refs) expect(onDisk, `${readme} → ${ref}`).toContain(ref);
    }
  });
});
