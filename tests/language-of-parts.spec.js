import { describe, it, expect, beforeEach } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

/**
 * Which language is this, and which language is *that* passage?
 *
 * Y-3 in `docs/durum.md`, and it turned out to be two criteria rather than one.
 *
 * **3.1.1, Language of Page — and it was failing.** `index.html` ships
 * `lang="en"` and nothing ever changed it, so a Turkish window announced itself
 * as English for its whole life. A screen reader picks its voice and its
 * pronunciation rules from that attribute, so the entire interface was being
 * read out with English phonetics. `docs/accessibility.md` said the interface
 * language "is announced on the document" — the sentence was true about the
 * attribute existing and false about what it said.
 *
 * **3.1.2, Language of Parts.** The known limitation as written: a view mixing
 * two languages does not mark the change per passage. There are two kinds of
 * mixing here and they take different values:
 *
 *   * The app's **own** English, which is the message Rust wrote. That is a
 *     fact and it is marked `en`.
 *   * **Everything a container produced** — a log line, a captured dump, docker's
 *     output, a statement's literals. Nothing here knows what language somebody
 *     else's application writes in, so those carry `lang=""`, HTML's
 *     "undetermined". Marking them `en` would be a guess stated as a fact, and
 *     an app that guesses wrong is worse for a screen reader than one that says
 *     it does not know.
 *
 * Read from the sources for the passages, because a mount test asserts on text
 * and roles and would pass with every attribute missing — the same reason
 * `a11y.spec.js` scans rather than renders.
 */

const ROOT = resolve(import.meta.dirname, '..');
const read = (path) => readFileSync(resolve(ROOT, path), 'utf8');

/**
 * Every element that renders text this application did not write, with the
 * marking it has to carry.
 *
 * A list rather than a sweep, and that is the honest shape: "does this element
 * hold foreign text" is a judgement about what flows into it, not something a
 * scanner can decide. What a scanner *can* do is fail the day one of these
 * loses its attribute, which is the way this regresses — an attribute is
 * invisible on screen and no other test looks at one.
 */
const PASSAGES = [
  {
    file: 'src/components/ErrorAlert.vue',
    anchor: 'class="text-caption" lang="en"',
    what: 'the message Rust wrote, which is English whatever the window speaks',
  },
  {
    file: 'src/components/LogView.vue',
    anchor: 'lang=""',
    what: "a container's own stdout",
  },
  {
    file: 'src/components/OperationConsole.vue',
    anchor: 'lang=""',
    what: "docker's output and the image's build output",
  },
  {
    file: 'src/components/DumpValue.vue',
    anchor: 'lang=""',
    what: 'a captured value, which is the application data',
  },
  {
    file: 'src/components/project/QueryLogPane.vue',
    anchor: 'lang=""',
    what: "a statement's literals",
  },
  {
    file: 'src/components/project/TimelinePane.vue',
    anchor: 'lang=""',
    what: 'a dump label, a statement or a subject line',
  },
  {
    file: 'src/components/project/WhySlowPane.vue',
    anchor: 'lang=""',
    what: 'the same three, joined to one request',
  },
];

describe('language of parts (WCAG 3.1.2)', () => {
  for (const passage of PASSAGES) {
    it(`marks ${passage.what} in ${passage.file.split('/').pop()}`, () => {
      expect(
        read(passage.file),
        `${passage.file} no longer marks the language of ${passage.what}`
      ).toContain(passage.anchor);
    });
  }

  /**
   * `lang="en"` on a container's output would be a guess presented as a fact,
   * and the guess is wrong for every project not written in English — which is
   * the population this application's own second language exists for.
   */
  it('never claims a language for text a container produced', () => {
    for (const file of [
      'src/components/LogView.vue',
      'src/components/OperationConsole.vue',
      'src/components/DumpValue.vue',
    ]) {
      const source = read(file);
      // Comments stripped first: the markup explains *why* the value is empty
      // rather than `en`, and a scanner that read the explanation as the
      // offence would fail the file for saying the right thing.
      const template = source.slice(source.indexOf('<template>')).replace(/<!--[\s\S]*?-->/g, '');
      expect(template, `${file} claims English for somebody else's output`).not.toMatch(
        /lang="en"/
      );
    }
  });
});

describe('language of page (WCAG 3.1.1)', () => {
  /**
   * The static file is the first frame and the wrong answer for half the
   * users. It is kept — a document with no `lang` at all is a worse start than
   * one with a stale value — and `applyAppearance` is what corrects it.
   */
  it('ships a document language for the first frame', () => {
    expect(read('index.html')).toContain('<html lang="en">');
  });

  it('sets the document language from the active locale, not from the file', async () => {
    const { i18n } = await import('@/i18n');
    const { applyAppearance } = await import('@/lib/appearance');

    for (const locale of ['tr', 'en']) {
      i18n.global.locale.value = locale;
      applyAppearance();
      expect(
        document.documentElement.getAttribute('lang'),
        `a ${locale} window announced itself as something else`
      ).toBe(locale);
    }
  });

  /**
   * The direction was already right and is asserted beside the language on
   * purpose: they are set from the same value on the same element, and the way
   * this breaks is somebody editing one of the two lines.
   */
  it('sets the direction from the same value on the same element', async () => {
    const { i18n } = await import('@/i18n');
    const { applyAppearance } = await import('@/lib/appearance');

    i18n.global.locale.value = 'tr';
    applyAppearance();
    expect(document.documentElement.getAttribute('dir')).toBe('ltr');
    expect(document.documentElement.getAttribute('lang')).toBe('tr');
  });
});

beforeEach(async () => {
  const { i18n } = await import('@/i18n');
  i18n.global.locale.value = 'en';
});
