import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { sectionFor, dateFor, releaseNotes } from '../tools/release-notes.mjs';

/**
 * The release page's notes are the changelog's section, found by the heading
 * the version bump writes. The shape of that heading is what this holds: a
 * bump that writes it differently would publish a release with no notes, or
 * stop the release in preflight — and either way this is where it says so.
 */
const CHANGELOG = `# Changelog

## [Unreleased]

## [0.3.3] - 2026-10-24

### Fixed

- **One thing.** Why it was wrong.

### Security

- **Another.**

## [0.3.2] - 2026-10-12

### Added

- Older.
`;

describe('the changelog section for a version', () => {
  it('runs from its heading to the next version, heading left out', () => {
    expect(sectionFor(CHANGELOG, '0.3.3')).toBe(
      '### Fixed\n\n- **One thing.** Why it was wrong.\n\n### Security\n\n- **Another.**'
    );
  });

  it('takes the last section to the end of the file', () => {
    expect(sectionFor(CHANGELOG, '0.3.2')).toBe('### Added\n\n- Older.');
  });

  it('is null for a version with no section, and for an empty one', () => {
    expect(sectionFor(CHANGELOG, '0.3.4')).toBe(null);
    expect(sectionFor('## [0.1.0] - 2026-01-01\n\n## [0.0.9] - 2025-12-01\n', '0.1.0')).toBe(null);
  });

  it('matches the whole version, not a prefix of one', () => {
    expect(sectionFor(CHANGELOG, '0.3')).toBe(null);
  });

  it('reads the date off the heading', () => {
    expect(dateFor(CHANGELOG, '0.3.3')).toBe('2026-10-24');
    expect(dateFor(CHANGELOG, '0.3.4')).toBe(null);
  });
});

describe('the notes handed to the release page', () => {
  it('are the section and a link to the heading in the full file', () => {
    const notes = releaseNotes(CHANGELOG, '0.3.3');
    expect(notes.startsWith('### Fixed\n')).toBe(true);
    expect(notes).toContain(
      'Full changelog: [CHANGELOG.md](https://github.com/stackvo/stackvo/blob/main/CHANGELOG.md#033---2026-10-24)'
    );
  });

  it('exist for the version this checkout declares', () => {
    // The real file and the real version: the preflight asks exactly this of
    // the tag it is about to build, and a bump that forgot the heading should
    // fail here first.
    const root = resolve(import.meta.dirname, '..');
    const changelog = readFileSync(resolve(root, 'CHANGELOG.md'), 'utf8');
    const { version } = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
    expect(releaseNotes(changelog, version)).not.toBe(null);
  });
});
