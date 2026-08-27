import { describe, it, expect, vi } from 'vitest';
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mount } from '@vue/test-utils';

/**
 * What a screen reader is handed, written down so a person can read it.
 *
 * ```sh
 * npm run a11y:transcript      # writes docs/accessibility-transcript.md
 * ```
 *
 * This is the piece that was actually missing. The known limitation says a human has to
 * decide whether a label *makes sense* and that nobody had done it — and the
 * reason nobody had done it was never unwillingness. It was that the job, as it
 * stood, meant installing a screen reader, learning its rotor, and driving it
 * blind across thirty screens in two languages to find out what it says. That
 * is a week, and it is a week that has to be repeated after every change.
 *
 * The machine can do all of that except the deciding. So it does: every page is
 * mounted, its headings and controls are read in the order the markup puts
 * them, their accessible names are computed, and the result is a document.
 * The remaining task is to read it and mark the lines whose wording is wrong —
 * which is an hour, and an hour anybody can spend without a screen reader
 * installed.
 *
 * ## Both languages, because only one of them was ever reviewed
 *
 * The English is what the strings were written in. The Turkish is what most of
 * this application's users hear, and it has never been read aloud by anyone.
 * A name that is fine in English and wrong in Turkish is invisible to every
 * check in this repository — they compare keys, not meanings.
 *
 * ## What this is not
 *
 * Not a gate. `accessible-names.spec.js` holds the mechanical floor — nothing
 * unnamed, nothing named with a word that says nothing, no page announcing
 * most of its controls identically — and those fail a build. This one produces
 * evidence for the question no assertion can settle, and a document that is
 * regenerated rather than kept in step: the point is what it says today.
 */

/**
 * The application's own Vuetify, not a fresh one.
 *
 * A bare `createVuetify()` has no locale adapter, so every string Vuetify names
 * itself — a clearable field's "Clear {label}", a pager, an empty table — comes
 * out in English however the interface is set. That is not a small difference
 * here: the first transcript run reported `Clear Proje ara...` as a Turkish
 * window announcing an English control, which would have been a real finding
 * and was the harness. The app's instance carries `createVueI18nAdapter`, so
 * Vuetify answers in whatever language vue-i18n is in.
 */
const vuetify = (await import('@/plugins/vuetify')).default;
const { i18n } = await import('@/i18n');

const OUT = resolve(import.meta.dirname, '../docs/accessibility-transcript.md');

const PAGES = ['Dashboard', 'Projects', 'Logs', 'Dumps', 'Mail', 'About'];
const LOCALES = ['tr', 'en'];

/** The accessible name, by the parts of the algorithm this application uses. */
function accessibleName(element, root) {
  const label = element.getAttribute('aria-label');
  if (label?.trim()) return label.trim();

  const by = element.getAttribute('aria-labelledby');
  if (by) {
    const text = by
      .split(/\s+/)
      .map((id) => root.querySelector(`[id="${id}"]`)?.textContent ?? '')
      .join(' ')
      .trim();
    if (text) return text;
  }

  const title = element.getAttribute('title');
  if (title?.trim()) return title.trim();

  return (element.textContent ?? '').replace(/\s+/g, ' ').trim();
}

/**
 * What the rotor lands on, in the order the markup puts it.
 *
 * Headings and controls, because those are the two lists a screen reader user
 * navigates by. Paragraph text is read in place and is the same text the eye
 * gets; the names below are the ones only a screen reader hears.
 */
function announcements(root) {
  const WANTED = 'h1, h2, h3, h4, h5, h6, button, a[href], [role="button"], [role="tab"], summary';
  return [...root.querySelectorAll(WANTED)]
    .filter((element) => element.getAttribute('aria-hidden') !== 'true')
    .map((element) => {
      const tag = element.tagName.toLowerCase();
      const heading = /^h[1-6]$/.test(tag);
      return {
        role: heading ? `heading ${tag[1]}` : element.getAttribute('role') || 'button',
        name: accessibleName(element, root),
      };
    });
}

async function readPage(name) {
  vi.resetModules();
  vi.doMock('@/lib/ipc', () => ({
    StackvoError: class extends Error {},
    call: vi.fn(),
    asList: (value) => (Array.isArray(value) ? value : []),
    api: new Proxy({}, { get: () => () => Promise.resolve(undefined) }),
  }));
  vi.doMock('@/lib/events', async (importOriginal) => ({
    ...(await importOriginal()),
    listenAll: async () => () => {},
    listen: async () => () => {},
  }));
  vi.doMock('@tauri-apps/api/app', () => ({ getVersion: async () => '0.1.0' }));
  vi.doMock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn(), openPath: vi.fn() }));

  const { createPinia } = await import('pinia');
  const { createRouter, createMemoryHistory } = await import('vue-router');
  const page = (await import(`@/views/${name}.vue`)).default;

  const host = document.createElement('div');
  document.body.appendChild(host);

  const wrapper = mount(
    { components: { Page: page }, template: '<v-app><Page /></v-app>' },
    {
      attachTo: host,
      global: {
        plugins: [
          createPinia(),
          createRouter({
            history: createMemoryHistory(),
            routes: [{ path: '/:pathMatch(.*)*', component: { template: '<div />' } }],
          }),
          vuetify,
          i18n,
        ],
      },
    }
  );

  await new Promise((r) => setTimeout(r, 0));
  const found = announcements(host);
  wrapper.unmount();
  host.remove();
  return found;
}

const HEADER = `# What a screen reader announces

**Generated — do not edit. \`npm run a11y:transcript\`.**

Every page below is mounted, and its headings and controls are listed in the
order the markup puts them, under the name a screen reader announces. This is
what a screen-reader audit needs a person for, and it is the only part a person is needed
for: the mechanical floor is held by \`tests/accessible-names.spec.js\` and
\`tests/reading-order.spec.js\`, which fail the build.

## How to review this

Read down each page and mark any line where:

* **the name does not say what the control does.** "Aç" on its own says nothing
  about what is being opened; "Bu kart ne işe yarar: CPU" does.
* **two lines say the same thing** and a listener could not choose between them.
  Some repetition is fine — a verb per table row, where the row names itself —
  and the rest is not.
* **the order is wrong.** A screen reader reads down this list. If a control
  that decides the meaning of the ones above it appears below them, that is the
  defect this transcript was written to make visible; one of exactly that shape
  was found in the new project drawer.
* **the Turkish is not what a Turkish speaker would say.** The strings were
  written in English and translated. Nothing in this repository compares
  meanings — only keys — so this is the pass no test can stand in for.

A page is mounted with no data behind it, so rows, projects and messages are
absent. What is listed is the frame: what somebody hears before anything loads,
which is also what they hear if nothing ever does.

## Known, and not this application's to fix

A search field's clear button announces as **"temizle" / "Clear"** with nothing
after it. Vuetify builds that name from the field's \`label\` prop and from
nothing else — not its \`aria-label\`, not its placeholder — and these fields
carry a placeholder instead of a label on purpose, because a floating label
above a one-line filter costs a row of height on every one of them. So the
choice is a visual one, and it is recorded here rather than changed quietly:
the field itself is named, its clear button is not.
`;

/**
 * Twelve page mounts, and the default timeout is five seconds.
 *
 * That was enough everywhere except the one place it mattered: the coverage
 * job instruments every module it loads, which turns a 2.3-second run into
 * something past the limit — so `test:js` was green on the same commit where
 * `test:js:coverage` died, the front-end report was never written, and the
 * floors gate failed two steps later naming itself instead of this.
 *
 * A generous number rather than a tuned one. This is a generator, not a gate:
 * the only thing a timeout here can catch is a mount that hangs, and there is
 * no version of that which finishes in ninety seconds.
 */
const MOUNTING_TWELVE_PAGES = 90_000;

describe('screen reader transcript', () => {
  it(
    'writes what every page announces, in both languages',
    async () => {
      let out = HEADER;
      let lines = 0;

      for (const locale of LOCALES) {
        i18n.global.locale.value = locale;
        out += `\n---\n\n# ${locale === 'tr' ? 'Türkçe' : 'English'}\n`;

        for (const page of PAGES) {
          const found = await readPage(page);
          const distinct = new Set(found.map((f) => f.name)).size;
          lines += found.length;

          out += `\n## ${page}\n\n`;
          out += `${found.length} announced, ${distinct} distinct.\n\n`;
          out += `| # | Role | Announced as |\n| --- | --- | --- |\n`;
          found.forEach((f, i) => {
            const name = f.name || '**(nothing — announced by its role alone)**';
            out += `| ${i + 1} | ${f.role} | ${name.replace(/\|/g, '\\|')} |\n`;
          });
        }
      }

      i18n.global.locale.value = 'en';
      writeFileSync(OUT, out);

      // The guard on the generator: an empty transcript is a mounting failure
      // that would otherwise look like a clean page.
      expect(lines, 'the transcript came out empty, which is the harness failing').toBeGreaterThan(
        PAGES.length * LOCALES.length
      );
    },
    MOUNTING_TWELVE_PAGES
  );
});
