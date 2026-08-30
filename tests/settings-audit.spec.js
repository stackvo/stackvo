import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The pane that finally reads the audit trail.
 *
 * `audit.rs` was written from eighteen call sites and read from none — there
 * was no command, so the record kept "for whoever has to account for the
 * machine" could only be produced by someone who knew it was JSON Lines and
 * knew which directory the logs go in.
 *
 * What is worth holding here is not the list markup but the three answers the
 * pane must not blur: an empty trail is a sentence rather than a blank box, a
 * capped list has to say it is capped, and damage in the file has to be
 * reported rather than showing up as a quietly shorter history.
 */

globalThis.visualViewport = undefined;

const replies = {};

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
          const reply = replies[name];
          if (reply === undefined) return Promise.resolve(null);
          return typeof reply === 'function' ? reply(...args) : Promise.resolve(reply);
        },
    }
  ),
}));

const { i18n } = await import('@/i18n');
const AuditPane = (await import('@/components/settings/AuditPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const entry = (action, subject, outcome = 'ok', detail = undefined) => ({
  at: '2026-08-10T09:00:00Z',
  action,
  subject,
  outcome,
  detail,
});

const mountPane = async () => {
  const wrapper = mount(
    {
      components: { AuditPane },
      template: '<v-app><AuditPane /></v-app>',
    },
    { global: { plugins: [vuetify, i18n] } }
  );
  await flushPromises();
  return wrapper;
};

beforeEach(() => {
  for (const key of Object.keys(replies)) delete replies[key];
});

describe('the audit pane', () => {
  it('says nothing irreversible has happened rather than showing an empty box', async () => {
    replies.auditTrail = { entries: [], total: 0, unreadable: 0 };
    const text = (await mountPane()).text();
    expect(text).toContain('Nothing irreversible has been done yet.');
  });

  it('shows the act, what it was done to, and the detail that makes it worth reading', async () => {
    replies.auditTrail = {
      entries: [entry('hosts_apply', 'shop.stackvo.loc', 'ok', '1 added')],
      total: 1,
      unreadable: 0,
    };
    const text = (await mountPane()).text();
    expect(text).toContain('hosts_apply');
    expect(text).toContain('shop.stackvo.loc');
    expect(text).toContain('1 added');
  });

  // A record that showed its cap as the whole history would understate what the
  // machine has been through, which is the one direction it must not be wrong.
  it('says when it is showing a tail rather than the history', async () => {
    replies.auditTrail = {
      entries: [entry('env_write', 'SERVER_GZIP')],
      total: 9000,
      unreadable: 0,
    };
    const text = (await mountPane()).text();
    expect(text).toContain('9000');
  });

  it('reports damage in the file instead of quietly showing fewer entries', async () => {
    replies.auditTrail = {
      entries: [entry('cert_apply', 'shop.stackvo.loc')],
      total: 2,
      unreadable: 1,
    };
    const text = (await mountPane()).text();
    expect(text).toMatch(/could not be read/);
  });

  it('offers to put back only the acts that recorded how', async () => {
    const undone = entry('stackvo_project_stop', 'shop');
    undone.at = '2026-08-10T09:00:01Z';
    undone.undo = { kind: 'steps', steps: [{ tool: 'stackvo_project_start', arguments: {} }] };

    const restart = entry('stackvo_project_restart', 'blog');
    restart.at = '2026-08-10T09:00:02Z';
    restart.undo = { kind: 'none', because: 'a restart has already stopped and started it' };

    replies.auditTrail = { entries: [undone, restart], total: 2, unreadable: 0 };
    const wrapper = await mountPane();

    // One button, on the one line that carries a plan.
    const buttons = wrapper
      .findAllComponents({ name: 'VBtn' })
      .filter((b) => b.text() === 'Put it back');
    expect(buttons).toHaveLength(1);

    // And the row that cannot says so, in the words the plan recorded — a row
    // with neither would read as an app that had not thought about it.
    expect(wrapper.text()).toContain('a restart has already stopped and started it');
    // The plan is shown before it is run, as the calls it would make.
    expect(wrapper.text()).toContain('project_start');
  });

  it('does not offer an act that was already put back, and says it was', async () => {
    const line = entry('stackvo_stack_down', 'the stack');
    line.undone = true;
    line.undo = { kind: 'steps', steps: [{ tool: 'stackvo_project_start', arguments: {} }] };

    replies.auditTrail = { entries: [line], total: 1, unreadable: 0 };
    const wrapper = await mountPane();

    expect(wrapper.text()).toContain('Put back');
    expect(
      wrapper.findAllComponents({ name: 'VBtn' }).filter((b) => b.text() === 'Put it back')
    ).toHaveLength(0);
  });

  it('re-reads the trail after an undo fails, and still shows the error', async () => {
    const line = entry('stackvo_project_stop', 'shop');
    line.undo = { kind: 'steps', steps: [{ tool: 'stackvo_project_start', arguments: {} }] };

    let reads = 0;
    replies.auditTrail = () => {
      reads += 1;
      return Promise.resolve({ entries: [line], total: 1, unreadable: 0 });
    };
    replies.auditUndo = () => Promise.reject({ code: 'CONFLICT', message: 'already put back' });

    const wrapper = await mountPane();
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Put it back')
      .trigger('click');
    await flushPromises();

    // The machine may have changed even though the call failed, so the screen
    // is re-read before the message is shown.
    expect(reads).toBe(2);
    expect(wrapper.findComponent({ name: 'ErrorAlert' }).exists()).toBe(true);
  });

  // A trail that fails to load must not render as an empty one: "nothing has
  // happened" and "I could not look" are different answers.
  it('reports a failure rather than presenting it as an empty trail', async () => {
    replies.auditTrail = () => Promise.reject(new Error('no log directory'));
    const text = (await mountPane()).text();
    expect(text).not.toContain('Nothing irreversible has been done yet.');
  });
});
