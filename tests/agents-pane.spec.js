import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { i18n } from '@/i18n';
import AgentsPane from '@/components/settings/AgentsPane.vue';

/**
 * The pane that edits another application's configuration file.
 *
 * The tests that matter here are the ones about what it refuses to do. Writing
 * the entry is settled in Rust, where `agents.rs` drives the merge with real
 * client files; what only exists in this component is the decision about *when
 * a button is offered* — and every one of those decisions protects something
 * that cannot be undone from the app: a registration pointing at a binary that
 * is not there, a write flag nobody asked for, an edit to a file with comments
 * in it.
 */

const api = vi.hoisted(() => ({
  agentsStatus: vi.fn(),
  agentsInstall: vi.fn(),
  agentsRemove: vi.fn(),
  rulesStatus: vi.fn(),
  rulesApply: vi.fn(),
  rulesRemove: vi.fn(),
  projectsList: vi.fn(),
}));

vi.mock('@/lib/ipc', () => ({ api, asList: (v) => (Array.isArray(v) ? v : []) }));

const vuetify = createVuetify({ components, directives });

const mountPane = () => mount(AgentsPane, { global: { plugins: [vuetify, i18n] } });

const client = (over = {}) => ({
  id: 'cursor',
  label: 'Cursor',
  path: '/Users/x/.cursor/mcp.json',
  present: true,
  exists: true,
  parseable: true,
  command: null,
  current: false,
  ...over,
});

const STATUS = {
  binary: '/opt/stackvo/stackvo-mcp',
  source: 'build',
  root: '/Users/x/.stackvo',
  clients: [
    client(),
    client({ id: 'claude-code', label: 'Claude Code', path: '/Users/x/.claude.json' }),
  ],
};

const rule = (over = {}) => ({
  id: 'claude',
  label: 'Claude Code',
  scope: 'workspace',
  path: '/Users/x/.stackvo/projects/shop/CLAUDE.md',
  exists: true,
  installed: false,
  current: false,
  ...over,
});

const RULES = [rule(), rule({ scope: 'global', path: '/Users/x/.claude/CLAUDE.md' })];

beforeEach(() => {
  vi.clearAllMocks();
  api.agentsStatus.mockResolvedValue(STATUS);
  api.agentsInstall.mockResolvedValue('/Users/x/.cursor/mcp.json');
  api.agentsRemove.mockResolvedValue('/Users/x/.cursor/mcp.json');
  api.rulesStatus.mockResolvedValue(RULES);
  api.rulesApply.mockResolvedValue('/Users/x/.stackvo/projects/shop/CLAUDE.md');
  api.rulesRemove.mockResolvedValue('/Users/x/.stackvo/projects/shop/CLAUDE.md');
  api.projectsList.mockResolvedValue([{ name: 'shop' }, { name: 'api' }]);
});

describe('what it refuses to offer', () => {
  it('withholds every register button when the server binary was not found', async () => {
    api.agentsStatus.mockResolvedValue({ ...STATUS, binary: null, source: null });
    const wrapper = mountPane();
    await flushPromises();

    // The instruction, not just a disabled control: "it will not work" without
    // "here is the command" is the shape of a bug report.
    expect(wrapper.text()).toContain('cargo build --release --bin stackvo-mcp');
    const buttons = wrapper.findAllComponents({ name: 'VBtn' });
    const register = buttons.filter((b) => b.text() === 'Register');
    expect(register).not.toHaveLength(0);
    expect(register.every((b) => b.props('disabled'))).toBe(true);
  });

  it('offers a copyable block instead of a button for a file it cannot parse', async () => {
    api.agentsStatus.mockResolvedValue({
      ...STATUS,
      clients: [client({ id: 'vscode', label: 'VS Code', parseable: false })],
    });
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Copy block');
    expect(labels).not.toContain('Register');
    expect(wrapper.text()).toContain('cannot be edited safely');
  });

  it('names the file on every row, because that is where a refusal sends you', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('/Users/x/.cursor/mcp.json');
    expect(wrapper.text()).toContain('/Users/x/.claude.json');
  });
});

describe('the write flag', () => {
  it('is off until it is switched on, and is what gets sent', async () => {
    const wrapper = mountPane();
    await flushPromises();

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Register')
      .trigger('click');
    await flushPromises();
    expect(api.agentsInstall).toHaveBeenLastCalledWith('cursor', false);

    await wrapper.findComponent({ name: 'VSwitch' }).setValue(true);
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Register')
      .trigger('click');
    await flushPromises();
    expect(api.agentsInstall).toHaveBeenLastCalledWith('cursor', true);
  });

  it('says what it grants, by name', async () => {
    const wrapper = mountPane();
    await flushPromises();

    // Not "allows writes" — the specific one people would not expect.
    expect(wrapper.text()).toContain('stack_down');
    expect(wrapper.text()).toContain('stopping the whole stack');
  });
});

describe('rows already registered', () => {
  it('offers Remove for a current registration and Update for a stale one', async () => {
    api.agentsStatus.mockResolvedValue({
      ...STATUS,
      clients: [
        client({ command: '/opt/stackvo/stackvo-mcp', current: true }),
        client({
          id: 'claude-code',
          label: 'Claude Code',
          path: '/Users/x/.claude.json',
          command: '/old/checkout/stackvo-mcp',
          current: false,
        }),
      ],
    });
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Remove');
    expect(labels).toContain('Update');
    expect(wrapper.text()).toContain('pointing at another copy');
  });

  it('re-reads the status after a failure, and still shows the error', async () => {
    api.agentsInstall.mockRejectedValue({ code: 'IO_ERROR', message: 'no' });
    const wrapper = mountPane();
    await flushPromises();

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Register')
      .trigger('click');
    await flushPromises();

    // Both halves: the list is not left describing a file nobody re-read, and
    // the message survives the re-read that clears it.
    expect(api.agentsStatus).toHaveBeenCalledTimes(2);
    expect(wrapper.findComponent({ name: 'ErrorAlert' }).exists()).toBe(true);
  });
});

describe('the block it tells you to paste', () => {
  it('is what the installer would have written', async () => {
    const wrapper = mountPane();
    await flushPromises();

    const parsed = JSON.parse(wrapper.vm.snippet(client()));
    expect(parsed.mcpServers.stackvo.command).toBe('/opt/stackvo/stackvo-mcp');
    expect(parsed.mcpServers.stackvo.env.STACKVO_ROOT).toBe('/Users/x/.stackvo');
    expect(parsed.mcpServers.stackvo.args).toBeUndefined();

    // VS Code names the map differently and needs the transport, exactly as
    // `agents.rs` writes it — the two shapes are stated in two languages and
    // this is the half that would drift silently.
    const vscode = JSON.parse(wrapper.vm.snippet(client({ id: 'vscode' })));
    expect(vscode.servers.stackvo.type).toBe('stdio');
    expect(vscode.mcpServers).toBeUndefined();
  });
});

/**
 * The rules half.
 *
 * Registering the server is settled above; what this half decides is *where*
 * the rules go, and that decision is the one with a consequence — a rules file
 * lands in somebody's repository and is usually committed. The tests here are
 * about the two things only this component knows: which scope a row belongs to,
 * and which directory the write is aimed at.
 */
describe('the rules', () => {
  it('separates what travels with the repository from what stays on the machine', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('In the project');
    expect(wrapper.text()).toContain('On this machine');
    // Both paths named, for the same reason every client row names its file.
    expect(wrapper.text()).toContain('/Users/x/.stackvo/projects/shop/CLAUDE.md');
    expect(wrapper.text()).toContain('/Users/x/.claude/CLAUDE.md');
  });

  it('aims at the workspace root until a project is chosen, and then at the project', async () => {
    const wrapper = mountPane();
    await flushPromises();

    // `undefined`, not null: the wrapper omits the argument so the command
    // takes its own default rather than being handed one from the UI.
    expect(api.rulesStatus).toHaveBeenLastCalledWith(undefined);

    await wrapper.findComponent({ name: 'VSelect' }).setValue('shop');
    await flushPromises();
    expect(api.rulesStatus).toHaveBeenLastCalledWith('shop');

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Write rules')
      .trigger('click');
    await flushPromises();
    expect(api.rulesApply).toHaveBeenLastCalledWith('claude', 'workspace', 'shop');
  });

  it('offers Update rather than Write for a block an older version wrote', async () => {
    api.rulesStatus.mockResolvedValue([rule({ installed: true, current: false })]);
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Update');
    expect(wrapper.text()).toContain('Written by an older version');
  });

  it('offers only Remove once the current block is in place', async () => {
    api.rulesStatus.mockResolvedValue([rule({ installed: true, current: true })]);
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Remove');
    expect(labels).not.toContain('Write rules');
  });

  it('promises, on screen, that the rest of the file survives', async () => {
    const wrapper = mountPane();
    await flushPromises();

    // The one sentence that decides whether somebody presses the button on a
    // CLAUDE.md they have been writing for a year.
    expect(wrapper.text()).toContain('Only the region between the StackVo markers is written');
    expect(wrapper.text()).toContain('.stackvo-backup');
  });
});
