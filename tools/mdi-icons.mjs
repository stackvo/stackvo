/**
 * Which Material Design Icons this application can ask for, in one place
 * because three things need the answer and would otherwise each have their own.
 *
 * The build strips the icon rules nothing here names — 7,092 of the 7,448,
 * against the 330 this repository writes down — so "which icons are used" stops
 * being a question about tidiness and becomes the thing standing between a
 * screen and a blank square. A second copy of the answer is a screen with a
 * blank square on it in some future release, and nobody would know which.
 *
 * The three readers:
 *
 *  * `vite.config.js` — the plugin that does the stripping;
 *  * `tools/check-bundle.mjs` — holds the built stylesheet against this list,
 *    after the build, which is the only moment the emitted file exists;
 *  * `tests/mdi-icons.spec.js` — holds the list against the icon set itself,
 *    which is where a name that is not an icon at all gets caught.
 */

import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

/**
 * One glyph rule, by its exact shape.
 *
 * The **double** colon is the whole discriminator, and it is a measurement
 * rather than a guess: every one of the 7,448 `::before` occurrences in the
 * stylesheet is a `.mdi-<name>::before {` at the start of a line, and the
 * modifiers that share the prefix — `.mdi-spin`, `.mdi-rotate-45`,
 * `.mdi-flip-h`, `.mdi-18px` — are all written with a single colon. So they
 * never match and are never dropped.
 *
 * The body is taken whole rather than required to be one `content` declaration,
 * and that too came from reading the file: exactly one rule has a second line.
 * `.mdi-blank` sets `visibility: hidden` beside its content — it is the icon
 * that occupies the space and draws nothing, which a menu uses to line up rows
 * that have no tick. A pattern that insisted on a single declaration read it as
 * "not an icon", which is the one wrong answer available about it.
 *
 * Fresh from a function rather than a shared constant: a global regex carries a
 * `lastIndex`, and two readers sharing one would each see half the file.
 */
export const glyphRule = () => /^\.mdi-([a-z0-9-]+)::before \{\n[^}]*\n\}\n/gm;

/** Where the icon set's own stylesheet lives. */
export const MDI_CSS = 'node_modules/@mdi/font/css/materialdesignicons.css';

/**
 * The trees an icon name can be written in.
 *
 * **Rust names icons too**, and leaving `src-tauri/src` out of this was a bug
 * with a measurement behind it: eighteen names appear only there — every icon
 * in the terminal, editor and browser pickers — so the subsetter stripped all
 * eighteen and those three lists shipped drawing blank squares beside their
 * entries. Reading them back cost 2.4 KB of the eager set, measured. `apps.rs` is a catalogue: it carries `mdi-apple`, `mdi-firefox`,
 * `mdi-powershell` and fifteen more that no `.vue` file ever repeats.
 *
 * Widening the scan found a nineteenth of a different kind on the first run:
 * `mdi-vim`, on the Neovim and Vim rows, is not an icon at all and never has
 * been. It could not have been caught before, because the file it was written
 * in was not being read.
 */
export const SOURCE_ROOTS = ['src', 'src-tauri/src'];

/**
 * Every icon name reachable from the source trees, as text.
 *
 * A loose scan rather than a parse. `mdi-foo` appears as a prop, as an element's
 * text, inside a ternary and inside a class string, and a matcher per shape is
 * four things to keep right. Over-collecting costs one kept rule; under-
 * collecting costs a blank square, so the scan is deliberately wide — wide
 * enough that a name written in a *comment* is collected too, which is a thing
 * to know when one of these lists disagrees with the icon set.
 *
 * **Rust is read narrowly**, and that is the one exception to the paragraph
 * above. In `.rs` an icon name is always a string literal — `apps.rs` is a
 * table of tuples — while the prose around it is thick with names that are not
 * icons: `mdi-vim` written down as the bug it was, `mdi-icons.mjs` naming this
 * file. Both were collected on the first wide run, and neither is a glyph, so
 * the pattern there requires the opening quote. Over-collecting costs one kept
 * rule; here it cost a test asserting that a sentence is an icon.
 *
 * A single root or a list of them. The single-root form is kept because it is
 * how the tests ask about one tree at a time, and because "which of these came
 * only from Rust" is a question worth being able to put.
 */
export function iconsInSource(srcDirs = SOURCE_ROOTS) {
  const names = new Set();

  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(path);
        continue;
      }
      const rust = entry.name.endsWith('.rs');
      if (!rust && !/\.(vue|js|ts|css)$/.test(entry.name)) continue;

      const source = readFileSync(path, 'utf8');
      const pattern = rust ? /"(mdi-[a-z0-9-]+)"/g : /(mdi-[a-z0-9-]+)/g;
      for (const match of source.matchAll(pattern)) {
        names.add(match[1]);
      }
    }
  };
  for (const dir of [srcDirs].flat()) walk(dir);

  return names;
}

/**
 * Everything the source names, plus the icons Vuetify renders on its own.
 *
 * Vuetify's `aliases` name 54 icons this repository never writes down: the
 * checkbox marks, the sort arrows, the pagination chevrons, the alert glyphs.
 * **Twenty-six of them appear nowhere in the source**, so a list built from
 * these trees alone ships an application whose checkboxes are empty — which is
 * exactly what the first version of this scan produced, and exactly the kind of
 * failure that looks like a Vuetify bug.
 *
 * `aliases` is passed in rather than imported here, because the two callers
 * that have it already imported it and the one that does not is a test that can.
 */
export function iconsUsed(srcDirs, aliases) {
  const names = iconsInSource(srcDirs);
  for (const value of Object.values(aliases)) {
    if (typeof value === 'string') names.add(value);
  }
  return names;
}
