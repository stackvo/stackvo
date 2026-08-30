import { describe, it, expect, afterEach } from 'vitest';
import { i18n } from '@/i18n';
import { catalogueText, quickCommandAbout, oauthNote, toolingWhy } from '@/lib/catalogue-text';

/**
 * The sentences three Rust catalogues send into a bilingual window.
 *
 * Thirty-seven of them — 26 quick commands, 7 identity providers, 4 required
 * tools — arrived as English literals and were printed raw, so a Turkish user
 * got a translated screen with English inside it. `hints.rs` had already
 * written down why this class is the worst one to leave alone: it is the
 * sentence that tells someone what a thing will *do*, and here it sits under a
 * command they are about to run in their own container.
 *
 * `hint_translations.rs` holds the two sides equal — every row translated in
 * both locales, no orphans, the English identical to Rust's. What it cannot see
 * is the lookup: whether the window asks for the translation at all, and what
 * it does when there is not one. That is this file.
 */

const SPEC = { id: 'migrate', about: 'Run pending migrations.' };

afterEach(() => {
  i18n.global.locale.value = 'en';
});

describe('catalogue prose', () => {
  it('reads the locale rather than the English the back end sent', () => {
    i18n.global.locale.value = 'tr';

    const turkish = quickCommandAbout(SPEC);
    expect(turkish).toBe('Bekleyen göçleri çalıştırır.');
    expect(turkish, 'the English was printed to a Turkish reader').not.toBe(SPEC.about);
  });

  it('reads all three catalogues', () => {
    i18n.global.locale.value = 'tr';

    expect(oauthNote({ id: 'github', note: 'x' })).toContain('OAuth');
    expect(oauthNote({ id: 'github', note: 'x' })).not.toBe('x');
    expect(toolingWhy({ id: 'docker', why: 'x' })).toContain('Her proje');
  });

  /**
   * The one case the Rust gate cannot prevent: an older back end offering a row
   * this build's locales have never heard of. English then, because English is
   * what the catalogue carries and a blank line under a command is worse.
   */
  it('falls back to the English the row carries', () => {
    i18n.global.locale.value = 'tr';

    expect(quickCommandAbout({ id: 'not-a-command', about: 'Do a thing.' })).toBe('Do a thing.');
  });

  /** A row with neither is an empty string, never `undefined` in the template. */
  it('renders nothing rather than the word undefined', () => {
    expect(quickCommandAbout({ id: 'not-a-command' })).toBe('');
    expect(quickCommandAbout(null)).toBe('');
    expect(catalogueText('quickCommands', null, undefined)).toBe('');
  });
});
