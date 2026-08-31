import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The editor itself, inside the container.
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
const JETBRAINS = {
  installed: true,
  musl: false,
  path: '/root/generated/devcontainer/shop/devcontainer.json',
  exists: false,
  current: false,
  service: 'shop',
  recreates: true,
};

const READY = {
  project: 'shop',
  editorInstalled: true,
  jetbrains: JETBRAINS,
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

const withJetbrains = (patch) => ({ ...READY, jetbrains: { ...JETBRAINS, ...patch } });

const mountPane = () =>
  mount(
    {
      components: { EditorPane },
      template: '<v-app><EditorPane name="shop" :running="true" /></v-app>',
    },
    // Pinia because the one caveat this pane can act on carries a
    // `RemedyAlert`, which reads the operations store rather than handing the
    // work back to the page through an emit.
    { global: { plugins: [createPinia(), vuetify, i18n] } }
  );

beforeEach(() => {
  calls.length = 0;
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

    // The standard remedy, run where it is read rather than emitted to the
    // page. `compose_up_project` is the recreate: the image already has what it
    // needs, only the container is behind.
    const recreate = wrapper.find('[data-test="remedy-recreate"]');
    expect(recreate.exists()).toBe(true);

    await recreate.trigger('click');
    await flushPromises();
    expect(calls).toContainEqual(['composeUpProject', 'shop']);
  });

  it('says nothing about the volume when the container already has it', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).not.toContain('downloads its server again');
  });
});

/**
 * The other editor, and it is deliberately not the same shape.
 *
 * VS Code is handed an address; PhpStorm is handed a file, because JetBrains
 * has no connection type that attaches to a container already running. What
 * these hold is the part that would be invisible if it went wrong: a file that
 * is out of date opens a container assembled from fewer overlays than the one
 * StackVo starts, and nothing on screen would say so.
 */
describe('the PhpStorm half', () => {
  it('writes the file and re-reads what it wrote', async () => {
    replies.editorJetbrainsWrite = JETBRAINS.path;
    const wrapper = mountPane();
    await flushPromises();

    const write = wrapper.findAll('button').find((b) => b.text().includes('Write the file'));
    await write.trigger('click');
    await flushPromises();

    expect(calls.filter(([name]) => name === 'editorJetbrainsWrite')).toEqual([
      ['editorJetbrainsWrite', 'shop'],
    ]);
    expect(calls.filter(([name]) => name === 'editorStatus').length).toBe(2);
  });

  it('shows the path and the two clicks only once the file is there', async () => {
    const wrapper = mountPane();
    await flushPromises();
    expect(wrapper.text()).not.toContain(JETBRAINS.path);

    replies.editorStatus = withJetbrains({ exists: true, current: true });
    const written = mountPane();
    await flushPromises();
    expect(written.text()).toContain(JETBRAINS.path);
    expect(written.text()).toContain('Remote Development');
  });

  /** Worse than no file: it names a compose list from before an overlay moved. */
  it('says when the file on disk is out of date, and offers to rewrite it', async () => {
    replies.editorStatus = withJetbrains({ exists: true, current: false });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('written for a different set of compose files');
    expect(wrapper.findAll('button').some((b) => b.text().includes('Rewrite the file'))).toBe(true);
  });

  /** A machine without PhpStorm is not a reason to withhold the file. */
  it('still offers the file when PhpStorm is not on this machine', async () => {
    replies.editorStatus = withJetbrains({ installed: false });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('PhpStorm was not found on this machine');
    const write = wrapper.findAll('button').find((b) => b.text().includes('Write the file'));
    expect(write.attributes('disabled')).toBeUndefined();
  });

  /**
   * One fact, two weights. Alpine is a note in the VS Code half and a wall in
   * this one, and a screen that showed it once would be wrong on one of them.
   */
  it('says an Alpine image has no JetBrains backend, and still describes the file', async () => {
    replies.editorStatus = withJetbrains({ musl: true });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('JetBrains publishes none');
    expect(wrapper.findAll('button').some((b) => b.text().includes('Write the file'))).toBe(true);
  });

  /** The cost this half cannot design away is on screen, not in a doc. */
  it('states that attaching recreates the container', async () => {
    const wrapper = mountPane();
    await flushPromises();
    expect(wrapper.text()).toContain('recreates this project’s container');
  });
});
