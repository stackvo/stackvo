import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';

/**
 * What a screen reader is actually handed, and whether it can be told apart.
 *
 * Y-1's first question — is a label *meaningful* — was written down as needing
 * a human, and most of it does. But "meaningful" has a mechanical floor that
 * nothing here was standing on, and it is the one a reviewer flags first:
 *
 *   * a control announced by a word that says nothing on its own ("more",
 *     "open", "…"), and
 *   * **the same name on several controls that do different things**. A page
 *     with twelve buttons all called "Help" is a page where a screen reader
 *     user hears the same sentence twelve times and cannot choose. Every one of
 *     those names passes an automated name check, because each control has a
 *     name. The failure is only visible across the page.
 *
 * That second one is why this file exists at page scale rather than per
 * component. `a11y.spec.js` proves every icon button has a name; `a11y-axe.spec
 * .js` proves the markup is valid. Neither can see that the names collide.
 *
 * ## What is still a judgement, and is not pretended otherwise
 *
 * Whether "Katalog" is the right word for that button is a question about
 * language and a person answers it. What this removes is the part that is not:
 * a reviewer should be reading names that are at least distinct and at least
 * say something, and should not be spending the audit finding duplicates a
 * script could have listed.
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

const PAGES = ['About', 'Dumps', 'Logs', 'Dashboard', 'Mail', 'Projects'];

/**
 * Names that say nothing without the thing beside them.
 *
 * Deliberately short and in both languages. A longer list starts rejecting
 * words that are fine in context — "Aç" on a row that names its project is a
 * good label — and the check below is about names that stand alone.
 */
const EMPTY_WORDS = new Set([
  '...',
  '…',
  '?',
  '-',
  '—',
  'more',
  'daha',
  'buraya',
  'here',
  'click',
  'tıkla',
  'link',
  'button',
  'düğme',
]);

/**
 * The accessible name of an element, by the parts of the algorithm that apply
 * to what this application writes.
 *
 * Not a full implementation of accname: no `aria-labelledby` chains through
 * shadow roots, no CSS-generated content. Those are not in this tree, and a
 * partial implementation that is honest about which parts it runs beats a
 * dependency that would have to be kept in step with jsdom.
 */
function accessibleName(element, root) {
  const label = element.getAttribute('aria-label');
  if (label?.trim()) return label.trim();

  const by = element.getAttribute('aria-labelledby');
  if (by) {
    const text = by
      .split(/\s+/)
      // Ids are attribute values written by this application and by Vuetify,
      // both of which produce plain identifiers — matched by attribute rather
      // than through `CSS.escape`, which jsdom's global scope does not carry.
      .map((id) => root.querySelector(`[id="${id}"]`)?.textContent ?? '')
      .join(' ')
      .trim();
    if (text) return text;
  }

  const title = element.getAttribute('title');
  if (title?.trim()) return title.trim();

  return (element.textContent ?? '').replace(/\s+/g, ' ').trim();
}

/** Everything a screen reader offers as an action, in reading order. */
function controls(root) {
  return [...root.querySelectorAll('button, a[href], [role="button"], [role="tab"], summary')]
    .filter((element) => element.getAttribute('aria-hidden') !== 'true')
    .map((element) => ({
      tag: element.tagName.toLowerCase(),
      name: accessibleName(element, root),
    }));
}

/** Mount one page and hand back its controls. */
async function pageControls(name) {
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

  await new Promise((resolve) => setTimeout(resolve, 0));
  const found = controls(host);
  wrapper.unmount();
  host.remove();
  return found;
}

describe('accessible names, at page scale', () => {
  it.each(PAGES)('%s names every control it offers', async (name) => {
    const unnamed = (await pageControls(name)).filter((c) => !c.name).map((c) => c.tag);

    expect(
      unnamed,
      `${name} offers ${unnamed.length} control(s) a screen reader announces by \
their role alone`
    ).toEqual([]);
  });

  it.each(PAGES)('%s names nothing with a word that says nothing', async (name) => {
    const hollow = (await pageControls(name))
      .filter((c) => EMPTY_WORDS.has(c.name.toLowerCase()))
      .map((c) => c.name);

    expect(
      hollow,
      `${name} announces these controls with a word that means nothing away \
from what is beside it`
    ).toEqual([]);
  });

  /**
   * The one no per-component check can see.
   *
   * Repeated names are not automatically wrong — a "Delete" button on each row
   * of a table is fine, because the row names itself and a screen reader reads
   * the row. What is wrong is a repeat with nothing around it to tell the
   * copies apart, and the honest threshold is not one: this asserts on how many
   * DISTINCT names a page has against how many controls it offers, so a page
   * where most controls share a handful of words fails and a table of rows does
   * not.
   */
  it.each(PAGES)('%s does not announce most of its controls identically', async (name) => {
    const found = await pageControls(name);
    if (found.length < 6) return;

    const distinct = new Set(found.map((c) => c.name)).size;
    const share = distinct / found.length;

    expect(
      share,
      `${name} offers ${found.length} controls under only ${distinct} distinct \
names, so a screen reader reads the same sentence for most of the page:\n` +
        [...new Set(found.map((c) => c.name))].map((n) => `  · ${n}`).join('\n')
    ).toBeGreaterThan(0.4);
  });
});
