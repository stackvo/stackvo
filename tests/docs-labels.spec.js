import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import en from '@/i18n/locales/en.js';
import tr from '@/i18n/locales/tr.js';

/**
 * The documentation site names things in the window — **Settings → Updates**,
 * **Project → Share → Get a public URL** — and a name that is not the one on
 * screen sends the reader looking for a card that does not exist. The site
 * said "Settings → Diagnostics" in four places while the card is called
 * "Application log", and the Turkish pages said "Doctor" where the window says
 * "Doktor". This test reads every such path out of `docs/content/<locale>` and
 * checks each segment against what the app can actually show in that
 * language: every string in the locale file, plus the name of every help card
 * (the `# heading` of `docs/help/<locale>/*.md`, which is the card's title).
 *
 * Two conventions follow from it. A path in the window is always written in
 * bold with `→` between the segments, so that it can be found; and a bare
 * "Settings → …" outside bold is reported, so that the convention holds.
 */

const ROOT = resolve(import.meta.dirname, '..');
const LOCALES = { en, tr };

// Where a path may start: the pages and top-level areas of the window. The
// locale files carry these too, but naming them keeps the error message
// useful when a path starts somewhere odd.
const ROOTS = {
  en: [
    'Settings',
    'Project',
    'Projects',
    'Catalogue',
    'Mail',
    'Logs',
    'Dumps',
    'Dashboard',
    'New project',
  ],
  tr: ['Ayarlar', 'Proje', 'Projeler', 'Katalog', 'Mail', 'Loglar', 'Dump', 'Panel', 'Yeni proje'],
};

function strings(node, out = []) {
  if (typeof node === 'string') out.push(node);
  else if (node && typeof node === 'object') Object.values(node).forEach((v) => strings(v, out));
  return out;
}

function markdownFiles(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    return statSync(path).isDirectory() ? markdownFiles(path) : entry.endsWith('.md') ? [path] : [];
  });
}

/**
 * What the help cards call things: the card's own name (its `# heading`) and
 * every control its tables name in the first column. The cards are written
 * against the window and tested against the code, so a name found here is a
 * name the window shows.
 */
function cardNames(locale) {
  const dir = join(ROOT, 'docs/help', locale);
  const names = [];
  for (const f of readdirSync(dir).filter((f) => f.endsWith('.md'))) {
    const lines = readFileSync(join(dir, f), 'utf8').split('\n');
    names.push(lines[0].replace(/^#\s+/, '').trim());
    for (const line of lines) {
      const cell = line.match(/^\|\s*([^|]+?)\s*\|/);
      if (cell && !/^-+$/.test(cell[1])) names.push(cell[1].replace(/[*`]/g, '').trim());
    }
  }
  return names;
}

/** A locale string with placeholders — `Enable {service}` — as a matcher. */
function template(value) {
  if (!value.includes('{')) return null;
  const source = value
    .split(/\{[^}]+\}/)
    .map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
    .join('.+');
  return new RegExp(`^${source}$`, 'u');
}

/** A segment as written, minus the punctuation a sentence adds around it. */
function clean(segment) {
  return segment
    .replace(/[.:,;!]+$/u, '')
    .replace(/^["“„]|["”“]$/gu, '')
    .trim();
}

// Segments that are a symbol on a button rather than a word: the `+` that
// creates a project, the `⋮` menu, the `?` of a help card.
const SYMBOL = /^[+⋮?…]$/u;

for (const [locale, messages] of Object.entries(LOCALES)) {
  describe(`documentation labels (${locale})`, () => {
    const known = new Set(
      [...strings(messages), ...cardNames(locale), ...ROOTS[locale]].map((s) => s.trim())
    );
    const templates = strings(messages).map(template).filter(Boolean);
    const shown = (segment) => known.has(segment) || templates.some((t) => t.test(segment));
    const files = markdownFiles(join(ROOT, 'docs/content', locale));
    const roots = ROOTS[locale].map((r) => r.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|');

    it('name only what the window shows', () => {
      const unknown = [];
      for (const file of files) {
        const text = readFileSync(file, 'utf8');
        // Bold spans in order, so that a closing `**` is never taken for an
        // opening one; a path may wrap onto the next line inside its bold.
        for (const match of text.matchAll(/\*\*([^*]+?)\*\*/gu)) {
          if (!match[1].includes('→')) continue;
          const path = match[1].replace(/\s+/g, ' ');
          const segments = path.split('→').map(clean).filter(Boolean);
          for (const segment of segments) {
            if (SYMBOL.test(segment) || shown(segment)) continue;
            unknown.push(
              `${file.replace(ROOT + '/', '')}: "${path}" — no "${segment}" in the ${locale} window`
            );
          }
        }
      }
      expect(unknown, unknown.join('\n')).toEqual([]);
    });

    it('write every path in the window in bold', () => {
      const bare = [];
      for (const file of files) {
        // Strip the bold paths first; what is left must not contain one.
        const stripped = readFileSync(file, 'utf8').replace(/\*\*[^*]+?\*\*/gu, (m) =>
          m.replace(/→/g, '·')
        );
        stripped.split('\n').forEach((rest, i) => {
          const line = rest;
          if (new RegExp(`(?:^|[^\\p{L}])(?:${roots}) → `, 'u').test(rest)) {
            bare.push(`${file.replace(ROOT + '/', '')}:${i + 1}: ${line.trim()}`);
          }
        });
      }
      expect(bare, bare.join('\n')).toEqual([]);
    });
  });
}
