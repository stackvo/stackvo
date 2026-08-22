import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The panel that answers "what is this card for".
 *
 * Every help button in the application writes a topic into one module-level
 * ref; this panel is what reads it. What could go wrong quietly and is checked
 * here: asking the backend for the wrong locale (a Turkish reader served the
 * English file forever), rendering the document's markdown as its source text,
 * and treating an unwritten topic as a crash instead of as the normal state it
 * is while the documents are still being written.
 */
globalThis.visualViewport = undefined;

const replies = {};
const calls = [];

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: new Proxy(
    {},
    {
      get:
        (_t, name) =>
        (...args) => {
          calls.push([String(name), ...args]);
          const reply = replies[name];
          if (reply instanceof Error) return Promise.reject(reply);
          return typeof reply === 'function' ? reply(...args) : Promise.resolve(reply);
        },
    }
  ),
}));

const { nextTick } = await import('vue');
const { i18n } = await import('@/i18n');
const { useHelp } = await import('@/composables/useHelp');
const HelpSheet = (await import('@/components/HelpSheet.vue')).default;

const vuetify = createVuetify({ components, directives });

const DOC = [
  '# Container',
  '',
  'What Docker reports about the container behind this project.',
  '',
  '## Controls',
  '',
  '| Control | What it does |',
  '| --- | --- |',
  '| **Copy** | puts it on the clipboard |',
].join('\n');

/**
 * Inside a `v-app`: the panel is a `v-navigation-drawer`, and Vuetify's layout
 * injection is what tells one where its edges are. Attached to the body because
 * the sheet teleports there — which is the whole reason it clears the app bar.
 */
/** The watcher awaits an IPC call, so one flush is not the whole of it. */
const settle = async () => {
  await flushPromises();
  await nextTick();
  await flushPromises();
};

const mountSheet = () =>
  (wrapper = mount(
    { components: { HelpSheet }, template: '<v-app><HelpSheet /></v-app>' },
    { attachTo: document.body, global: { plugins: [vuetify, i18n] } }
  ));

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  useHelp().closeHelp();
  i18n.global.locale.value = 'en';
});

/**
 * Unmounted rather than the body wiped between tests: the panel teleports into
 * the body, so emptying it by hand leaves Vue holding references to nodes that
 * are gone and the next mount renders into nothing. Every one of these tests
 * passed on its own and failed in the file, which is the shape of that bug.
 */
let wrapper = null;
afterEach(() => {
  wrapper?.unmount();
  wrapper = null;
});

describe('opening help for a card', () => {
  it('asks for the topic the button named, in the reader’s language', async () => {
    replies.helpDoc = DOC;
    mountSheet();

    useHelp().openHelp('project-container');
    await flushPromises();

    expect(calls).toContainEqual(['helpDoc', 'project-container', 'en']);
  });

  /**
   * The one that would go unnoticed: a Turkish reader gets the English file and
   * nothing in the interface says which language the document was written in.
   */
  it('asks in Turkish when the interface is Turkish', async () => {
    replies.helpDoc = DOC;
    i18n.global.locale.value = 'tr';
    mountSheet();

    useHelp().openHelp('project-tunnel');
    await flushPromises();

    expect(calls).toContainEqual(['helpDoc', 'project-tunnel', 'tr']);
  });

  it('renders the markdown rather than showing its source', async () => {
    replies.helpDoc = DOC;
    mountSheet();

    useHelp().openHelp('project-container');
    await settle();

    const html = document.body.innerHTML;
    expect(html).toContain('<table>');
    expect(html).toContain('<strong>Copy</strong>');
    expect(html).not.toContain('| Control |');
  });

  /** Moving from one card's help to another swaps the content, not the panel. */
  it('follows the next card without being closed first', async () => {
    replies.helpDoc = (topic) => Promise.resolve(`# ${topic}\n\nabout ${topic}.`);
    mountSheet();

    useHelp().openHelp('project-container');
    await settle();
    useHelp().openHelp('project-tunnel');
    await settle();

    expect(document.body.innerHTML).toContain('about project-tunnel.');
    expect(document.body.innerHTML).not.toContain('about project-container.');
  });

  /** A topic nobody has written yet is a state, not a failure. */
  it('says so when the document has not been written', async () => {
    replies.helpDoc = new Error('no help document for project-hooks in en');
    mountSheet();

    useHelp().openHelp('project-hooks');
    await settle();

    expect(document.body.textContent).toContain('project-hooks');
    expect(document.body.innerHTML).not.toContain('<table>');
  });
});
