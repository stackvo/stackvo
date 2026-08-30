#!/usr/bin/env node
/**
 * Hold the bundle ceilings defined in `tools/bundle-budget.mjs`.
 *
 * ## A missing build is a failure, not a pass
 *
 * The same hole `check-coverage.mjs` closes. With no `dist/`, a naive gate has
 * nothing to compare and reports success — and a green tick is read as an
 * answer, not as "there was nothing to look at". Here the failure mode is
 * worse than usual, because `dist/` is gitignored: the ordinary state of a
 * fresh clone is *no build at all*, so the silent pass would be the normal
 * outcome rather than a rare one.
 *
 * Usage:
 *   node tools/check-bundle.mjs            after `npm run build`
 *   node tools/check-bundle.mjs --dir out  a different build directory
 */

import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { join, basename } from 'node:path';
import { ceilings, measured } from './bundle-budget.mjs';
import { MDI_CSS, SOURCE_ROOTS, glyphRule, iconsUsed } from './mdi-icons.mjs';
import { aliases as vuetifyAliases } from 'vuetify/iconsets/mdi';

const argv = process.argv.slice(2);
const dirFlag = argv.indexOf('--dir');
const DIST = dirFlag === -1 ? 'dist' : argv[dirFlag + 1];

const kb = (bytes) => bytes / 1024;
const fmt = (n) => `${n.toFixed(1)} KB`;

function fail(message) {
  console.error(`\n  ✗ ${message}\n`);
  process.exit(1);
}

if (!existsSync(DIST)) {
  fail(`no build at ${DIST}/ — run \`npm run build\` first. Nothing was checked.`);
}

const indexHtml = join(DIST, 'index.html');
if (!existsSync(indexHtml)) {
  fail(`${DIST}/ has no index.html, so the eager set cannot be read. Nothing was checked.`);
}

/**
 * The assets the window loads before it can paint.
 *
 * Taken from the built HTML rather than a list kept here: the entry `<script>`,
 * every `modulepreload`, and every stylesheet. Vite writes exactly this set,
 * and reading it means a renamed or newly split chunk is accounted for without
 * anyone remembering to.
 */
function eagerAssets(html) {
  const refs = [
    ...html.matchAll(/<script[^>]+src="([^"]+)"/g),
    ...html.matchAll(/<link[^>]+rel="modulepreload"[^>]+href="([^"]+)"/g),
    ...html.matchAll(/<link[^>]+rel="stylesheet"[^>]+href="([^"]+)"/g),
  ].map((m) => m[1]);

  return [...new Set(refs)].map((ref) => basename(ref));
}

/** Every emitted file under `dist/`, recursively. */
function allFiles(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    return statSync(path).isDirectory() ? allFiles(path) : [path];
  });
}

const html = readFileSync(indexHtml, 'utf8');
const eagerNames = eagerAssets(html);

if (!eagerNames.length) {
  fail(`${DIST}/index.html references no scripts or stylesheets — the parse found nothing.`);
}

const files = allFiles(DIST);
const sizeOf = (name) => {
  const match = files.find((path) => basename(path) === name);
  if (!match) fail(`${DIST}/index.html references ${name}, which is not in the build.`);
  return statSync(match).size;
};

const eagerBytes = eagerNames.reduce((sum, name) => sum + sizeOf(name), 0);
const totalBytes = files.reduce((sum, path) => sum + statSync(path).size, 0);

const rows = [
  {
    what: 'eager (first paint)',
    got: kb(eagerBytes),
    ceiling: ceilings.eagerKb,
    was: measured.eagerKb,
  },
  {
    what: 'total (every asset)',
    got: kb(totalBytes),
    ceiling: ceilings.totalKb,
    was: measured.totalKb,
  },
];

console.log('\n  Bundle budget — raw bytes, because nothing here crosses a network.\n');
console.log(`  eager set, from ${DIST}/index.html: ${eagerNames.join(', ')}\n`);

let over = false;
for (const { what, got, ceiling, was } of rows) {
  const ok = got <= ceiling;
  over ||= !ok;
  const drift = got - was;
  const arrow = drift > 0.05 ? `+${fmt(drift)}` : drift < -0.05 ? fmt(drift) : 'unchanged';
  console.log(
    `  ${ok ? '✓' : '✗'} ${what.padEnd(20)} ${fmt(got).padStart(10)}` +
      `   ceiling ${fmt(ceiling).padStart(9)}   since measured: ${arrow}`
  );
}

/**
 * The icons the build stripped, held against the icons the app can ask for.
 *
 * Three hundred of the four hundred kilobytes above were bought by dropping the
 * 7,447 icon rules nothing here names, and the price of getting that list wrong
 * is not a number — it is a blank square on a screen, in a release, for one
 * icon nobody happens to look at.
 *
 * Here rather than in the test suite because this is the only moment the
 * emitted stylesheet exists: the suite runs before the build, and a test that
 * needed `dist/` would either fail in CI or learn to skip, which is worse.
 */
function heldIcons() {
  const css = files
    .filter((path) => path.endsWith('.css'))
    .map((path) => readFileSync(path, 'utf8'))
    .join('\n');

  // Minified, so `::before` has become `:before` — both spellings are the same
  // rule and the check is about the name, not the colons.
  const present = new Set(
    [...css.matchAll(/\.(mdi-[a-z0-9-]+):{1,2}before/g)].map((match) => match[1])
  );

  // Only names the icon set actually carries. A name that is not an icon at
  // all is a different fault with a different answer, and `mdi-icons.spec.js`
  // is where it is caught — reporting it here would blame the subsetter for
  // something it did right.
  const upstream = new Set(
    [...readFileSync(MDI_CSS, 'utf8').matchAll(glyphRule())].map((match) => `mdi-${match[1]}`)
  );

  const wanted = iconsUsed(SOURCE_ROOTS, vuetifyAliases);
  const missing = [...wanted].filter((name) => upstream.has(name) && !present.has(name));

  if (present.size < 100) {
    console.error(
      `\n  Only ${present.size} icon rules survived into the bundle. That is not a\n` +
        '  subset, it is an application with no icons.\n'
    );
    return false;
  }

  if (missing.length) {
    console.error(
      `\n  ${missing.length} icon(s) the app asks for were stripped out of the bundle:\n` +
        `    ${missing.slice(0, 12).join(', ')}${missing.length > 12 ? ', …' : ''}\n\n` +
        '  Each of those renders as a blank square. See tools/mdi-icons.mjs.\n'
    );
    return false;
  }

  console.log(`\n  icons: ${present.size} rules kept, every one the app asks for.`);
  return true;
}

const iconsOk = heldIcons();

if (over || !iconsOk) {
  if (over) {
    console.error(
      '\n  A ceiling is not a suggestion. Either trim what grew, or raise the number\n' +
        '  in tools/bundle-budget.mjs with the reason in the commit message.\n'
    );
  }
  process.exit(1);
}

console.log('\n  Within budget.\n');
