import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { HELP_LOCALES, HELP_TOPICS, helpDoc, isHelpTopic } from '../src/lib/help.js';

/**
 * Every help button opens something, and every card offers one.
 *
 * The button is the same control on eighty-one cards, and the thing it opens is
 * a file named by a slug typed at the call site. Two ways that goes wrong and
 * neither is visible on screen: a slug the registry does not know (the button
 * renders, opens nothing), and a card that never got one (no button at all,
 * which looks exactly like a card that deliberately has no help).
 *
 * Read from the sources, because a mount test cannot see the second failure —
 * a missing attribute renders a perfectly valid card.
 */
const DIRS = ['src/components/project', 'src/components/settings', 'src/views'];

/**
 * Panels that carry help, named one by one rather than by sweeping
 * `src/components`.
 *
 * That directory also holds `DoctorPanel.vue`, which draws seven cards and has
 * no help written for it yet. Sweeping the directory would fail this file for a
 * gap nobody has filled rather than for something that has drifted — so the
 * panels that do carry help are listed, and the day the doctor's cards get
 * theirs, its file joins this list.
 */
const EXTRA = ['src/components/NewProjectDrawer.vue', 'src/components/LandingCard.vue'];

/**
 * Every card-shaped tag, with whatever `help` it was given.
 *
 * `SideSheet` is in the list but marked optional: a panel that only confirms
 * something has nothing to explain, and a help button on it would be an offer
 * of nothing. What is not optional is that a topic it *does* name is one the
 * registry knows.
 */
const OPTIONAL = ['SideSheet'];

function cardsIn(source) {
  const template = source.slice(source.indexOf('<template>'));
  const found = [];
  const tags = ['PaneHeader', 'SettingsGroup', 'CollapsiblePane', 'PageLayout', ...OPTIONAL];

  for (const m of template.matchAll(new RegExp(`<(${tags.join('|')})\\b`, 'g'))) {
    const block = template.slice(m.index, template.indexOf('>', m.index));
    const help = /\shelp="([^"]*)"/.exec(block);
    found.push({ tag: m[1], help: help?.[1] ?? null });
  }

  // A card that draws its own heading — the dashboard's tiles do — carries the
  // button itself rather than passing a topic to a wrapper. Same topic, same
  // registry; it is only spelled `topic` because that is the button's own prop.
  for (const m of template.matchAll(/<HelpButton\s+topic="([^"]*)"/g)) {
    found.push({ tag: 'HelpButton', help: m[1] });
  }
  return found;
}

const files = [
  ...DIRS.flatMap((dir) =>
    readdirSync(dir)
      .filter((f) => f.endsWith('.vue'))
      .map((f) => join(dir, f))
  ),
  ...EXTRA,
].filter((path) => cardsIn(readFileSync(path, 'utf8')).length);

describe('the help topics', () => {
  it('are the whole of what the sources ask for', () => {
    expect(HELP_TOPICS.length).toBeGreaterThan(50);
    expect([...new Set(HELP_TOPICS)]).toHaveLength(HELP_TOPICS.length);
  });

  it.each(files)('%s names a topic on every card it draws', (path) => {
    const cards = cardsIn(readFileSync(path, 'utf8'));

    const unnamed = cards.filter((c) => !c.help && !OPTIONAL.includes(c.tag)).map((c) => c.tag);
    expect(unnamed, `${path} draws a card with no help topic`).toEqual([]);

    const unknown = cards
      .map((c) => c.help)
      .filter((topic) => topic !== null)
      .filter((topic) => !isHelpTopic(topic));
    expect(unknown, `${path} names a topic the registry does not know`).toEqual([]);
  });

  /** A topic nothing opens is a document nobody reaches. */
  it('registers nothing the sources never name', () => {
    const used = new Set(
      files.flatMap((path) => cardsIn(readFileSync(path, 'utf8')).map((c) => c.help))
    );
    expect(HELP_TOPICS.filter((t) => !used.has(t))).toEqual([]);
  });

  it('maps each topic to its own document', () => {
    const docs = new Set(HELP_TOPICS.map((topic) => helpDoc(topic)));
    expect(docs.size).toBe(HELP_TOPICS.length);
    expect(helpDoc('project-tunnel')).toBe('docs/help/en/project-tunnel.md');
    expect(helpDoc('project-tunnel', 'tr')).toBe('docs/help/tr/project-tunnel.md');
    // A locale nobody writes documents in reads the fallback, not a 404.
    expect(helpDoc('project-tunnel', 'de')).toBe('docs/help/en/project-tunnel.md');
  });
});

/**
 * The documents that have been written so far.
 *
 * They land topic by topic, so a topic with no document is not a failure. What
 * would be: a document filed under a name no card opens — nobody would ever
 * reach it — or one written in one language and not the other, which is how a
 * help system ends up half translated without anybody noticing.
 */
describe('the help documents', () => {
  const written = Object.fromEntries(
    HELP_LOCALES.map((locale) => {
      const dir = `docs/help/${locale}`;
      const files = existsSync(dir)
        ? readdirSync(dir)
            .filter((f) => f.endsWith('.md'))
            .map((f) => f.replace(/\.md$/, ''))
        : [];
      return [locale, files];
    })
  );

  it.each(HELP_LOCALES)('%s: every document is a topic some card opens', (locale) => {
    const stray = written[locale].filter((topic) => !isHelpTopic(topic));
    expect(stray, `docs/help/${locale} holds documents no card opens`).toEqual([]);
  });

  it('writes a topic in every locale or in none', () => {
    const [first, ...rest] = HELP_LOCALES;
    for (const locale of rest) {
      expect(
        written[first].filter((t) => !written[locale].includes(t)),
        `written in ${first} but not in ${locale}`
      ).toEqual([]);
      expect(
        written[locale].filter((t) => !written[first].includes(t)),
        `written in ${locale} but not in ${first}`
      ).toEqual([]);
    }
  });

  /** It opens with the card's name, so the viewer has a title to put on it. */
  it.each(HELP_LOCALES)('%s: every document opens with a heading', (locale) => {
    for (const topic of written[locale]) {
      const text = readFileSync(`docs/help/${locale}/${topic}.md`, 'utf8');
      expect(text.trimStart(), `${locale}/${topic}.md`).toMatch(/^# \S/);
    }
  });
});
