import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The pane that tells somebody `commands.json` exists.
 *
 * Four rivals sell "add your own command" on the front page and this
 * application had one way to do it: edit a repository somebody else owns. The
 * file is the fix; this pane is how anybody finds out about it, which is the
 * half a file-shaped interface always owes — **where it goes, what it found,
 * and what it refused.**
 *
 * Read-only on purpose, and that is asserted here rather than assumed: a form
 * would be a second way to write the same JSON, and the two would disagree the
 * first time somebody used an editor.
 */

globalThis.visualViewport = undefined;

let reply = null;

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: { machineCommands: () => Promise.resolve(reply) },
}));

const { i18n } = await import('@/i18n');
const Pane = (await import('@/components/settings/MachineCommandsPane.vue')).default;

const vuetify = createVuetify({ components, directives });

async function pane(state) {
  reply = state;
  const wrapper = mount(Pane, { global: { plugins: [vuetify, i18n] } });
  await Promise.resolve();
  await Promise.resolve();
  await wrapper.vm.$nextTick();
  return wrapper;
}

const AT = '/Users/x/StackVo/commands.json';

describe('the machine-wide commands pane', () => {
  /**
   * An absent file is the ordinary case, not an error, and it is the state
   * everybody is in before they have heard of the feature — so it is the one
   * that has to carry the instructions.
   */
  it('tells somebody how to start when there is no file', async () => {
    const wrapper = await pane({ path: AT, exists: false, commands: {}, problems: [] });

    expect(wrapper.text()).toContain(AT);
    expect(wrapper.text()).toContain(i18n.global.t('settings.machineCommands.absent'));
    expect(wrapper.text()).not.toContain(i18n.global.t('settings.machineCommands.empty'));
  });

  /**
   * "You have not written one" and "yours declares nothing" are different
   * situations to be in, and a pane that said the same thing for both would
   * send somebody looking for a file they already have.
   */
  it('says something different for a file that exists and declares nothing', async () => {
    const wrapper = await pane({ path: AT, exists: true, commands: {}, problems: [] });

    expect(wrapper.text()).toContain(i18n.global.t('settings.machineCommands.empty'));
    expect(wrapper.text()).not.toContain(i18n.global.t('settings.machineCommands.absent'));
  });

  it('shows each command as the argv it is, not as a line', async () => {
    const wrapper = await pane({
      path: AT,
      exists: true,
      commands: {
        tail: { exec: ['tail', '-f', 'storage/logs/laravel.log'], about: 'Follow the log' },
        shell: { exec: ['bash'], interactive: true },
      },
      problems: [],
    });

    const text = wrapper.text();
    expect(text).toContain('tail');
    expect(text).toContain('tail -f storage/logs/laravel.log');
    expect(text).toContain('Follow the log');
    expect(text).toContain(i18n.global.t('settings.machineCommands.interactive'));
  });

  /**
   * A refused row is the whole reason this pane is not just documentation: the
   * file is edited in an editor that cannot tell you `migrate` is taken.
   */
  it('shows what the file got wrong', async () => {
    const wrapper = await pane({
      path: AT,
      exists: true,
      commands: {},
      problems: [
        {
          code: 'COMMAND',
          path: 'commands.migrate',
          message: '"migrate" is already a built-in command',
        },
      ],
    });

    expect(wrapper.text()).toContain('commands.migrate');
    expect(wrapper.text()).toContain('already a built-in command');
  });

  /** Read-only, stated as a test so it is a decision rather than a gap. */
  it('offers no way to write the file from here', async () => {
    const wrapper = await pane({
      path: AT,
      exists: true,
      commands: { tail: { exec: ['tail'] } },
      problems: [],
    });

    expect(wrapper.findAllComponents({ name: 'VTextField' })).toHaveLength(0);
    expect(wrapper.findAllComponents({ name: 'VTextarea' })).toHaveLength(0);
  });

  it('renders in Turkish', async () => {
    i18n.global.locale.value = 'tr';
    try {
      const wrapper = await pane({ path: AT, exists: false, commands: {}, problems: [] });
      expect(wrapper.text()).toContain(i18n.global.t('settings.machineCommands.absent'));
    } finally {
      i18n.global.locale.value = 'en';
    }
  });
});
