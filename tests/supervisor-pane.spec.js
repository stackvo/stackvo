import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The supervisord inside a project's own container.
 *
 * What these hold is the part that took the longest to get right: the three
 * ways this pane can have nothing to show look identical — an empty table —
 * and send somebody to three different places. A pane that said "cannot
 * connect" to all three would be worse than one that said nothing, because two
 * of the three are not failures at all.
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
const en = (await import('@/i18n/locales/en.js')).default;
const SupervisorPane = (await import('@/components/project/SupervisorPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const process = (over = {}) => ({
  fullName: 'php-fpm',
  name: 'php-fpm',
  group: 'php-fpm',
  state: 20,
  stateName: 'RUNNING',
  description: 'pid 8, uptime 0:04:00',
  pid: 8,
  uptime: 240,
  uptimeText: '',
  spawnErr: '',
  restarts: 0,
  flapping: false,
  ...over,
});

const view = (over = {}) => ({
  reach: 'ok',
  snapshot: {
    project: 'shop',
    daemon: 'RUNNING',
    version: '4.2.5',
    processes: [process(), process({ fullName: 'nginx', name: 'nginx', pid: 9 })],
    summary: { total: 2, running: 2, stopped: 0, fatal: 0, other: 0, flapping: 0 },
  },
  ...over,
});

const mountPane = (running = true) =>
  mount(
    {
      components: { SupervisorPane },
      props: ['running'],
      template: '<v-app><SupervisorPane name="shop" :running="running" /></v-app>',
    },
    // Pinia because the pane's one actionable state carries a `RemedyAlert`,
    // and the standard remedy reads the operations store to know whether this
    // project already has work in flight.
    { props: { running }, global: { plugins: [createPinia(), vuetify, i18n] } }
  );

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  replies.supervisorProject = view();
  replies.supervisorControl = true;
});

describe('what it shows', () => {
  it('lists what the project container is supervising, with nothing configured', async () => {
    const pane = mountPane();
    await flushPromises();

    // One argument: the project name. There is no server to add, and nothing
    // asks for a container, a host or a port.
    expect(calls).toContainEqual(['supervisorProject', 'shop']);
    expect(pane.text()).toContain('php-fpm');
    expect(pane.text()).toContain('nginx');
    expect(pane.text()).toContain('2 of 2 running');
  });

  it('restarts one process by the project it belongs to', async () => {
    const pane = mountPane();
    await flushPromises();

    await pane.find(`button[title="${en.supervisors.restart}"]`).trigger('click');
    await flushPromises();

    expect(calls).toContainEqual([
      'supervisorControl',
      'shop',
      'process',
      'restart',
      'php-fpm',
      undefined,
    ]);
    // And it re-reads rather than colouring the row on its own say-so.
    expect(calls.filter(([n]) => n === 'supervisorProject').length).toBeGreaterThan(1);
  });

  it('shows a flapping php-fpm as a problem even though it reports RUNNING', async () => {
    replies.supervisorProject = view({
      snapshot: {
        ...view().snapshot,
        processes: [process({ flapping: true, restarts: 5 })],
        summary: { total: 1, running: 1, stopped: 0, fatal: 0, other: 0, flapping: 1 },
      },
    });
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain('RUNNING');
    expect(pane.text()).toContain(en.supervisors.flapping);
  });
});

describe('the three ways it can be empty', () => {
  /**
   * Not a failure. This project's server simply does not use supervisord, and
   * a warning here would send somebody looking for a problem that is not one.
   */
  it('says the project does not use supervisord, rather than failing', async () => {
    replies.supervisorProject = view({ reach: 'noSupervisord', snapshot: null });
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain(en.projectSupervisor.noSupervisord);
    expect(pane.find('.v-alert--variant-tonal').classes().join(' ')).not.toContain('warning');
  });

  /**
   * The one that is actionable, and the one nothing else on screen would
   * explain: the image predates the socket in the generated config.
   */
  it('says to rebuild when the image predates the socket', async () => {
    replies.supervisorProject = view({ reach: 'noSocket', snapshot: null });
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain(en.projectSupervisor.noSocket);

    // And it is a button, not a sentence ending in "rebuild the project". The
    // sentence was all this pane had: the standard remedy is what turned it
    // into something that can be acted on where it is read.
    const rebuild = pane.find('[data-test="remedy-rebuild"]');
    expect(rebuild.exists()).toBe(true);
    expect(rebuild.text()).toBe(en.remedy.rebuild);

    await rebuild.trigger('click');
    await flushPromises();
    expect(calls).toContainEqual(['projectBuild', 'shop']);
  });

  it('says the container is not running', async () => {
    replies.supervisorProject = view({ reach: 'stopped', snapshot: null });
    const pane = mountPane();
    await flushPromises();
    expect(pane.text()).toContain(en.projectSupervisor.stopped);
  });

  /**
   * A `docker exec` every few seconds against a stopped project can only fail,
   * so a project that is down is not asked at all.
   */
  it('does not ask anything while the project is stopped', async () => {
    const pane = mountPane(false);
    await flushPromises();

    expect(calls.filter(([n]) => n === 'supervisorProject')).toHaveLength(0);
    expect(pane.text()).toContain(en.projectSupervisor.needsRunning);
  });
});
