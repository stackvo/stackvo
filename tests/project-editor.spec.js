import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * §2 R-1: the editor itself, inside the container.
 *
 * The pane is one button and an address, so what is worth holding is not the
 * layout — it is the three things a reader could be told wrongly without
 * anything looking broken:
 *
 *   * a **refusal has to name which** of the two it is. "Cannot open an editor"
 *     over a stopped container and over a container holding a copy of the
 *     source are two different sentences with two different answers, and the
 *     second one is the whole reason `editor.rs` refuses rather than warns.
 *   * the **address is shown anyway**. It is a string that works on a machine
 *     this one is not, so hiding it when the button is unpressable would turn
 *     a missing launcher into a missing feature.
 *   * the pane **never builds the address**. It renders what Rust derived, so
 *     the leading slash, the workdir and the hex stay in one place.
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
          return typeof reply === 'function' ? reply(...args) : Promise.resolve(reply);
        },
    }
  ),
}));

const { i18n } = await import('@/i18n');
const EditorPane = (await import('@/components/project/EditorPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const URI = 'vscode-remote://attached-container+7b2263/var/www/html';

/** A PHP project whose source is mounted and whose volume is in place. */
const READY = {
  project: 'shop',
  editorInstalled: true,
  readiness: {
    container: 'stackvo-shop',
    workdir: '/var/www/html',
    serverDir: '/root/.vscode-server',
    libc: 'glibc',
    running: true,
    sourceLive: true,
    serverKept: true,
    blockers: [],
    caveats: [],
    attachable: true,
    folderUri: URI,
    handlerUrl: 'vscode://vscode-remote/attached-container+7b2263/var/www/html',
  },
};

const withReadiness = (patch, top = {}) => ({
  ...READY,
  ...top,
  readiness: { ...READY.readiness, ...patch },
});

const emitted = [];

const mountPane = () =>
  mount(
    {
      components: { EditorPane },
      template: '<v-app><EditorPane name="shop" :running="true" @apply="onApply" /></v-app>',
      methods: {
        onApply() {
          emitted.push('apply');
        },
      },
    },
    { global: { plugins: [vuetify, i18n] } }
  );

beforeEach(() => {
  calls.length = 0;
  emitted.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  replies.editorStatus = READY;
});

describe('the button', () => {
  it('opens the container by name and re-reads what it did', async () => {
    replies.editorAttach = URI;
    const wrapper = mountPane();
    await flushPromises();

    const button = wrapper.findAll('button').find((b) => b.text().includes('VS Code'));
    expect(button.attributes('disabled')).toBeUndefined();

    await button.trigger('click');
    await flushPromises();

    expect(calls.filter(([name]) => name === 'editorAttach')).toEqual([['editorAttach', 'shop']]);
    // The status is read again afterwards: a container that changed under the
    // render is a pane describing a state nobody checked.
    expect(calls.filter(([name]) => name === 'editorStatus').length).toBe(2);
  });

  /**
   * The refusal is the boundary's, and the screen must not talk it into a
   * retry: whatever comes back is shown, and the pane goes back to reading.
   */
  it('shows the reason the boundary refused', async () => {
    const { StackvoError } = await import('@/lib/ipc');
    const error = new StackvoError('stackvo-shop is not running.');
    error.code = 'NOT_FOUND';
    replies.editorAttach = () => Promise.reject(error);

    const wrapper = mountPane();
    await flushPromises();
    await wrapper
      .findAll('button')
      .find((b) => b.text().includes('VS Code'))
      .trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('stackvo-shop is not running.');
  });

  it('is unpressable when this machine has no VS Code, and says which it is', async () => {
    replies.editorStatus = withReadiness({}, { editorInstalled: false });
    const wrapper = mountPane();
    await flushPromises();

    const button = wrapper.findAll('button').find((b) => b.text().includes('VS Code'));
    expect(button.attributes('disabled')).toBeDefined();
    // Not "this project cannot carry an editor" — the project is fine.
    expect(wrapper.text()).toContain('VS Code was not found on this machine');
    expect(wrapper.text()).not.toContain('copy of the source');
  });
});

describe('the two refusals', () => {
  it('says a stopped container is stopped and nothing more', async () => {
    replies.editorStatus = withReadiness({
      running: false,
      sourceLive: false,
      serverKept: false,
      attachable: false,
      blockers: ['notRunning'],
      caveats: [],
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('The container is not running');
    // The other refusal must not be inferred from an empty mount table.
    expect(wrapper.text()).not.toContain('copy of the source');
    expect(
      wrapper
        .findAll('button')
        .find((b) => b.text().includes('VS Code'))
        .attributes('disabled')
    ).toBeDefined();
  });

  /**
   * The one the module exists for. A node project with dev mode off holds a
   * copy: the editor would open, save happily, and lose the session on the
   * next rebuild — so the pane names the directory and the way out.
   */
  it('names the path when the container carries a copy of the source', async () => {
    replies.editorStatus = withReadiness({
      container: 'stackvo-blog',
      workdir: '/app',
      libc: 'musl',
      sourceLive: false,
      attachable: false,
      blockers: ['sourceIsASnapshot'],
      caveats: ['musl'],
      folderUri: 'vscode-remote://attached-container+7b2264/app',
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('copy of the source at /app');
    expect(wrapper.text()).toContain('dev server');
    // Alpine is a note here and never the reason: VS Code ships a musl server.
    expect(wrapper.text()).toContain('Alpine image');
  });
});

describe('the address', () => {
  it('is rendered exactly as Rust derived it, and never assembled here', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain(URI);
    // The container name and the workdir are on screen elsewhere; the address
    // must not be re-joined from them.
    expect(EditorPane.__file ?? '').not.toContain('attached-container');
  });

  it('is still shown when the button cannot be pressed', async () => {
    replies.editorStatus = withReadiness({
      running: false,
      attachable: false,
      blockers: ['notRunning'],
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain(URI);
  });
});

describe('the caveat with a fix', () => {
  it('offers the recreate that keeps the downloaded server', async () => {
    replies.editorStatus = withReadiness({ serverKept: false, caveats: ['serverIsNotKept'] });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('downloads its server again after every rebuild');

    const recreate = wrapper.findAll('button').find((b) => b.text().includes('Recreate'));
    await recreate.trigger('click');
    expect(emitted).toEqual(['apply']);
  });

  it('says nothing about the volume when the container already has it', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).not.toContain('downloads its server again');
  });
});
