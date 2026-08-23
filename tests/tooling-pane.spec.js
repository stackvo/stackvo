import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { i18n } from '@/i18n';
import ToolingPane from '@/components/settings/ToolingPane.vue';

/**
 * The pane that edits a shell startup file and installs a downloaded binary.
 *
 * What the writing does is settled in Rust — `tooling.rs` owns the markers, the
 * backup and the digest, and its tests are the ones that describe those. What
 * exists only here is the decision about **which button is offered**, and every
 * one of those decisions stands in front of something that is confusing rather
 * than reversible: an Install button on a tool this app will never install, a
 * silent "done" for a change the open terminal cannot see, an Add on a shell
 * that is already set up.
 */

const api = vi.hoisted(() => ({
  toolingStatus: vi.fn(),
  toolingPathApply: vi.fn(),
  toolingPathRemove: vi.fn(),
  toolingInstall: vi.fn(),
  toolingRemove: vi.fn(),
}));

vi.mock('@/lib/ipc', () => ({ api, asList: (v) => (Array.isArray(v) ? v : []) }));

const vuetify = createVuetify({ components, directives });
const mountPane = () => mount(ToolingPane, { global: { plugins: [vuetify, i18n] } });

const BIN = '/Users/x/Library/Application Support/StackVo/bin';

const shell = (over = {}) => ({
  id: 'zsh',
  label: 'zsh',
  path: '/Users/x/.zshrc',
  exists: true,
  installed: false,
  current: false,
  line: `export PATH="${BIN}:$PATH"`,
  ...over,
});

const tool = (over = {}) => ({
  id: 'mkcert',
  label: 'mkcert',
  program: 'mkcert',
  why: 'Trusted HTTPS for .loc domains.',
  source: 'missing',
  version: null,
  path: null,
  offers: '1.4.4',
  publisher: 'Filippo Valsorda',
  availableHere: true,
  ...over,
});

const STATUS = {
  binDir: BIN,
  onPath: false,
  currentShell: 'zsh',
  own: [
    { id: 'stackvo', about: 'The stack from a terminal.', built: '/build/stackvo', linked: null },
    {
      id: 'stackvo-mcp',
      about: 'The MCP server assistants talk to.',
      built: '/build/stackvo-mcp',
      linked: null,
    },
  ],
  shells: [shell(), shell({ id: 'fish', label: 'fish', exists: false, path: '/Users/x/.config/fish/config.fish' })],
  tools: [
    tool(),
    tool({ id: 'docker', label: 'Docker', source: 'system', version: '29.7.2', offers: null }),
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  api.toolingStatus.mockResolvedValue(STATUS);
  api.toolingPathApply.mockResolvedValue('/Users/x/.zshrc');
  api.toolingPathRemove.mockResolvedValue('/Users/x/.zshrc');
  api.toolingInstall.mockResolvedValue(`${BIN}/mkcert`);
  api.toolingRemove.mockResolvedValue(`${BIN}/mkcert`);
});

describe('what it refuses to offer', () => {
  it('gives a tool with no download a sentence, not a disabled button', async () => {
    // A disabled Install says "later". The honest answer for Docker is "not by
    // this", and that is a different sentence.
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('Installed by its own installer');
    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels.some((l) => l.includes('Install 1.4.4'))).toBe(true);
    // One offer, for the one tool that has one.
    expect(labels.filter((l) => l.includes('Install')).length).toBe(1);
  });

  it('says so when the publisher has no build for this platform', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      tools: [tool({ availableHere: false })],
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('No build for this platform');
    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels.some((l) => l.includes('Install'))).toBe(false);
  });

  it('offers the build command when neither binary has been compiled', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      own: STATUS.own.map((row) => ({ ...row, built: null })),
    });
    const wrapper = mountPane();
    await flushPromises();

    // The instruction, not just an absence: "nothing to link" without "here is
    // how to make one" is the shape of a bug report.
    expect(wrapper.text()).toContain('npm run sidecars');
  });
});

describe('what it says about a shell', () => {
  it('names the startup file on every row, because that is where a refusal sends you', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('/Users/x/.zshrc');
    expect(wrapper.text()).toContain('/Users/x/.config/fish/config.fish');
  });

  it('separates a shell with no startup file from one that simply lacks the line', async () => {
    // Two different answers: the first means that shell is not used here.
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('Not on your PATH');
    expect(wrapper.text()).toContain('No startup file here');
  });

  it('offers Update rather than Add for a line pointing at an older directory', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      shells: [shell({ installed: true, current: false })],
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('Points at an older directory');
    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Update');
    expect(labels).not.toContain('Add');
  });

  it('drops Add once the line is the current one, and keeps Remove', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      shells: [shell({ installed: true, current: true })],
    });
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).not.toContain('Add');
    expect(labels).toContain('Remove');
  });

  it('offers the line itself, for startup files this app should not edit', async () => {
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Copy line');
  });
});

describe('what it says about the shell you are in', () => {
  it('warns that the change reaches the next shell, not this one', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      onPath: false,
      own: STATUS.own.map((row) => ({ ...row, linked: `${BIN}/${row.id}` })),
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('open a new terminal');
  });

  it('drops that warning once the directory is on this process PATH', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      onPath: true,
      own: STATUS.own.map((row) => ({ ...row, linked: `${BIN}/${row.id}` })),
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).not.toContain('open a new terminal');
  });
});

describe('what it sends', () => {
  it('writes the shell it was asked about, never the one it guessed', async () => {
    const wrapper = mountPane();
    await flushPromises();

    const add = wrapper.findAllComponents({ name: 'VBtn' }).find((b) => b.text() === 'Add');
    await add.trigger('click');
    await flushPromises();

    expect(api.toolingPathApply).toHaveBeenCalledWith('zsh');
    // Re-read afterwards: a failed write may still have changed the file, and a
    // row showing the old state would be a claim nobody checked.
    expect(api.toolingStatus).toHaveBeenCalledTimes(2);
  });

  it('re-reads and then reports when a write fails', async () => {
    api.toolingPathApply.mockRejectedValue(Object.assign(new Error('nope'), { code: 'IO_ERROR' }));
    const wrapper = mountPane();
    await flushPromises();

    const add = wrapper.findAllComponents({ name: 'VBtn' }).find((b) => b.text() === 'Add');
    await add.trigger('click');
    await flushPromises();

    expect(api.toolingStatus).toHaveBeenCalledTimes(2);
    expect(wrapper.findComponent({ name: 'ErrorAlert' }).exists()).toBe(true);
  });

  it('installs the tool by id', async () => {
    const wrapper = mountPane();
    await flushPromises();

    const install = wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text().includes('Install 1.4.4'));
    await install.trigger('click');
    await flushPromises();

    expect(api.toolingInstall).toHaveBeenCalledWith('mkcert');
  });

  it('offers Remove only for the copy this app installed', async () => {
    api.toolingStatus.mockResolvedValue({
      ...STATUS,
      tools: [
        tool({ source: 'managed', version: 'v1.4.4' }),
        tool({ id: 'git', label: 'Git', source: 'system', version: '2.50.1', offers: null }),
      ],
    });
    const wrapper = mountPane();
    await flushPromises();

    // A system copy is somebody else's. The badge says so and no button offers
    // to delete it.
    expect(wrapper.text()).toContain('yours');
    expect(wrapper.text()).toContain('managed');
    const remove = wrapper.findAllComponents({ name: 'VBtn' }).filter((b) => b.text() === 'Remove');
    expect(remove).toHaveLength(1);

    await remove[0].trigger('click');
    await flushPromises();
    expect(api.toolingRemove).toHaveBeenCalledWith('mkcert');
  });
});

describe('what it never shows', () => {
  it('names no container tool anywhere on the page', async () => {
    // The rule the module exists to hold, checked where a future row would be
    // added: composer and friends run in the project's container.
    const wrapper = mountPane();
    await flushPromises();

    const text = wrapper.text();
    for (const banned of ['composer install', 'wp-cli', 'Bun', 'Laravel installer']) {
      expect(text).not.toContain(banned);
    }
  });
});
