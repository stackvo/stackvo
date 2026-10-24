/**
 * The release notes for a version, out of CHANGELOG.md.
 *
 * ```sh
 * node tools/release-notes.mjs 0.3.3      # the section, as the release page gets it
 * ```
 *
 * ## Why this exists
 *
 * `release.yml` handed `tauri-action` a title and no body, so every draft
 * opened with an empty "Release notes" field and the notes were typed into
 * the form by hand before Publish — v0.3.2's were, and v0.3.3's draft was
 * found empty with the installers already on it. The changelog had the whole
 * section the entire time, under the heading the bump commit wrote.
 *
 * So the section is the notes. The build job prints it through this file and
 * passes it as `releaseBody`; the preflight asks first, and a tag whose
 * version has no section stops there — a version bump that skipped the
 * changelog is a release with nothing to say, and six runners should not find
 * that out. The page's one addition is a link back to the full file.
 *
 * ## What a section is
 *
 * Keep-a-Changelog's: from `## [<version>] - <date>` to the next `## [` or
 * the end of the file, with the heading itself left out — the release page
 * already names the version. The one in the `Unreleased` block is not a
 * section for anything; a tag has a date.
 */
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const REPOSITORY = 'https://github.com/stackvo/stackvo';

/**
 * The body of `## [version]`'s section, trimmed, or null when there is none.
 */
export function sectionFor(changelog, version) {
  const lines = changelog.split('\n');
  const heading = new RegExp(`^## \\[${version.replace(/[.+-]/g, '\\$&')}\\]`);
  const start = lines.findIndex((line) => heading.test(line));
  if (start === -1) return null;
  let end = lines.findIndex((line, i) => i > start && /^## \[/.test(line));
  if (end === -1) end = lines.length;
  const body = lines
    .slice(start + 1, end)
    .join('\n')
    .trim();
  return body || null;
}

/** The heading's date, `## [0.3.3] - 2026-10-24` → `2026-10-24`, or null. */
export function dateFor(changelog, version) {
  const heading = new RegExp(
    `^## \\[${version.replace(/[.+-]/g, '\\$&')}\\] - (\\d{4}-\\d{2}-\\d{2})`,
    'm'
  );
  return heading.exec(changelog)?.[1] ?? null;
}

/** The notes as the release page shows them: the section and a link back. */
export function releaseNotes(changelog, version) {
  const section = sectionFor(changelog, version);
  if (!section) return null;
  const date = dateFor(changelog, version);
  // GitHub's anchor for `## [0.3.3] - 2026-10-24` is `033---2026-10-24`.
  const anchor = `${version.replace(/[^a-z0-9-]/gi, '')}${date ? `---${date}` : ''}`;
  return `${section}\n\nFull changelog: [CHANGELOG.md](${REPOSITORY}/blob/main/CHANGELOG.md#${anchor})\n`;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const version = (process.argv[2] ?? '').replace(/^v/, '');
  if (!version) {
    console.error('usage: node tools/release-notes.mjs <version>   (0.3.3 or v0.3.3)');
    process.exit(2);
  }
  const notes = releaseNotes(readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf8'), version);
  if (!notes) {
    console.error(
      `CHANGELOG.md has no "## [${version}]" section. The version bump writes that heading over the Unreleased entries; a tag without one has no release notes.`
    );
    process.exit(1);
  }
  process.stdout.write(notes);
}
