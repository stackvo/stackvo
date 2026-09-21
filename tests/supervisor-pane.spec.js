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
    {
      props: { running },
      global: { plugins: [createPinia(), vuetify, i18n] },
      // The detail sheet teleports to the body, so a test that reads it needs
      // the pane in the document rather than detached.
      attachTo: document.body,
    }
  );

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  replies.supervisorProject = view();
  replies.supervisorControl = true;
});

describe('the daemon as a whole', () => {
  /** Open the header menu and return its entries, which live in an overlay. */
  async function openMenu(pane) {
    await pane.find(`button[aria-label="${en.projectSupervisor.daemon.button}"]`).trigger('click');
    await flushPromises();
    return [...document.querySelectorAll('.v-overlay .v-list-item')];
  }
  const entry = (items, label) =>
    items.find((el) => el.querySelector('.v-list-item-title')?.textContent.trim() === label);

  it('offers the six supervisorctl verbs, and shows what the daemon said', async () => {
    replies.supervisorDaemon = {
      verb: 'reread',
      ok: true,
      output: 'queue: available\nscheduler: available',
    };
    const pane = mountPane();
    await flushPromises();

    const items = await openMenu(pane);
    const titles = items.map((el) => el.querySelector('.v-list-item-title')?.textContent.trim());
    for (const verb of ['status', 'reread', 'update', 'start-all', 'stop-all', 'restart-all']) {
      expect(titles).toContain(en.projectSupervisor.daemon[verb]);
    }

    entry(items, en.projectSupervisor.daemon.reread).click();
    await flushPromises();

    expect(calls).toContainEqual(['supervisorDaemon', 'shop', 'reread']);
    const result = pane.find('[data-testid="supervisor-daemon-result"]');
    expect(result.exists()).toBe(true);
    expect(result.text()).toContain('supervisorctl reread');
    expect(result.text()).toContain('scheduler: available');
    // And the table is re-read afterwards rather than trusted to be the same.
    expect(calls.filter(([n]) => n === 'supervisorProject').length).toBeGreaterThan(1);
    pane.unmount();
  });

  it('asks before stopping everything, because the web server is in "everything"', async () => {
    replies.supervisorDaemon = { verb: 'stop-all', ok: true, output: 'nginx: stopped' };
    const pane = mountPane();
    await flushPromises();

    const items = await openMenu(pane);
    entry(items, en.projectSupervisor.daemon['stop-all']).click();
    await flushPromises();

    // Nothing ran yet: a confirmation is up, and it names the project.
    expect(calls.some(([n]) => n === 'supervisorDaemon')).toBe(false);
    const dialog = document.querySelector('[data-test="confirm-dialog"]');
    expect(dialog).toBeTruthy();
    expect(dialog.textContent).toContain('shop goes down');

    const go = [...dialog.querySelectorAll('button')].find((b) =>
      b.textContent.includes(en.projectSupervisor.daemon['stop-all'])
    );
    go.click();
    await flushPromises();
    expect(calls).toContainEqual(['supervisorDaemon', 'shop', 'stop-all']);
    pane.unmount();
  });

  it('is not offered while the daemon cannot be reached', async () => {
    replies.supervisorProject = { reach: 'stopped', snapshot: null, pending: [], stale: [] };
    const pane = mountPane();
    await flushPromises();
    expect(pane.find(`button[aria-label="${en.projectSupervisor.daemon.button}"]`).exists()).toBe(
      false
    );
    pane.unmount();
  });
});

describe('one process in full', () => {
  it('opens a detail that names where each value came from, and can stop the process', async () => {
    replies.supervisorProcess = {
      process: process({ fullName: 'queue', name: 'queue', group: 'queue', pid: 12 }),
      origin: 'manifest',
      declared: {
        id: 'queue',
        exec: ['php', 'artisan', 'queue:work', 'rabbitmq'],
        enabled: true,
        replicas: 1,
        stopWait: 30,
      },
      config: [
        { key: 'command', value: 'php artisan queue:work rabbitmq' },
        { key: 'stopwaitsecs', value: '30' },
      ],
      configPath: '/etc/supervisor/conf.d/supervisord.conf',
      resources: { rssKb: 58492, threads: 1, cpuSeconds: 1.14, cpuPercent: 0.3 },
    };
    replies.supervisorProject = view({
      snapshot: {
        ...view().snapshot,
        processes: [process({ fullName: 'queue', name: 'queue', group: 'queue', pid: 12 })],
        summary: { total: 1, running: 1, stopped: 0, fatal: 0, other: 0, flapping: 0 },
      },
    });
    const pane = mountPane();
    await flushPromises();

    await pane.find(`button[title="${en.projectSupervisor.detail.button}"]`).trigger('click');
    await flushPromises();

    // Asked for by the row's full name, on demand and not on the poll.
    expect(calls).toContainEqual(['supervisorProcess', 'shop', 'queue']);

    // The sheet teleports to the body, so the document is where its text is.
    const text = document.body.textContent;
    expect(text).toContain(en.projectSupervisor.detail.origin.manifest);
    expect(text).toContain('php artisan queue:work rabbitmq');
    expect(text).toContain('stopwaitsecs');
    expect(text).toContain('57.1 MB');
    expect(text).toContain('0.3 %');
    expect(text).toContain('/etc/supervisor/conf.d/supervisord.conf');

    const stop = [...document.body.querySelectorAll('.side-sheet__footer button')].find((b) =>
      b.textContent.includes(en.projectSupervisor.detail.stop)
    );
    expect(stop).toBeTruthy();
    stop.click();
    await flushPromises();
    expect(calls).toContainEqual([
      'supervisorControl',
      'shop',
      'process',
      'stop',
      'queue',
      undefined,
    ]);
    // And the pane re-reads on the dialog's say-so.
    expect(calls.filter(([n]) => n === 'supervisorProject').length).toBeGreaterThan(1);
    pane.unmount();
  });

  it('opens the log tab of the same sheet, tails the file the daemon writes, and follows it', async () => {
    vi.useFakeTimers();
    replies.supervisorProcess = {
      process: process({ fullName: 'queue', name: 'queue', group: 'queue', pid: 12 }),
      origin: 'manifest',
      declared: {
        id: 'queue',
        exec: ['php', 'artisan', 'queue:work'],
        enabled: true,
        replicas: 1,
        stopWait: 10,
      },
      config: [
        { key: 'command', value: 'php artisan queue:work' },
        { key: 'stdout_logfile', value: '/var/log/supervisor-queue.log' },
      ],
      configPath: '/etc/supervisor/conf.d/supervisord.conf',
      resources: null,
    };
    replies.supervisorLog = 'Processing: App\\Jobs\\CropPhoto\nProcessed:  App\\Jobs\\CropPhoto\n';
    replies.supervisorProject = view({
      snapshot: {
        ...view().snapshot,
        processes: [process({ fullName: 'queue', name: 'queue', group: 'queue', pid: 12 })],
        summary: { total: 1, running: 1, stopped: 0, fatal: 0, other: 0, flapping: 0 },
      },
    });
    const pane = mountPane();
    await flushPromises();

    await pane.find(`button[title="${en.supervisors.log}"]`).trigger('click');
    await flushPromises();
    await flushPromises();

    // Read only once the config said where the file is, with the default depth.
    expect(calls).toContainEqual(['supervisorLog', 'shop', 'queue', 'stdout', 500]);
    expect(document.body.textContent).toContain('Processed:  App\\Jobs\\CropPhoto');
    expect(document.body.textContent).toContain('/var/log/supervisor-queue.log');

    // Following: read again a few seconds later, without a click.
    const before = calls.filter(([n]) => n === 'supervisorLog').length;
    await vi.advanceTimersByTimeAsync(3100);
    expect(calls.filter(([n]) => n === 'supervisorLog').length).toBeGreaterThan(before);

    pane.unmount();
    vi.useRealTimers();
  });

  it("shows the container's own output for a process that writes to stdout", async () => {
    replies.supervisorProcess = {
      process: process(),
      origin: 'image',
      declared: null,
      config: [
        { key: 'command', value: '/usr/local/sbin/php-fpm -F' },
        { key: 'stdout_logfile', value: '/dev/stdout' },
      ],
      configPath: '/etc/supervisor/conf.d/supervisord.conf',
      resources: null,
    };
    replies.supervisorStdout = '[21-Sep-2026 09:09:51] NOTICE: ready to handle connections\n';
    const pane = mountPane();
    await flushPromises();
    await pane.find(`button[title="${en.supervisors.log}"]`).trigger('click');
    await flushPromises();
    await flushPromises();

    // Not the daemon's tail, which would be an error, but the container's log.
    expect(calls.some(([n]) => n === 'supervisorLog')).toBe(false);
    expect(calls).toContainEqual(['supervisorStdout', 'shop', 500]);
    expect(document.body.textContent).toContain(en.projectSupervisor.logToStdout);
    expect(document.body.textContent).toContain('NOTICE: ready to handle connections');
    pane.unmount();
  });

  it('says out loud that a process nobody declares will not survive a rebuild', async () => {
    replies.supervisorProcess = {
      process: process({ fullName: 'by-hand', name: 'by-hand', group: 'by-hand', pid: 40 }),
      origin: 'unknown',
      declared: null,
      config: [],
      configPath: '/etc/supervisor/conf.d/supervisord.conf',
      resources: null,
    };
    replies.supervisorProject = view({
      stale: ['by-hand'],
      snapshot: {
        ...view().snapshot,
        processes: [process({ fullName: 'by-hand', name: 'by-hand', group: 'by-hand', pid: 40 })],
        summary: { total: 1, running: 1, stopped: 0, fatal: 0, other: 0, flapping: 0 },
      },
    });
    const pane = mountPane();
    await flushPromises();
    await pane.find(`button[title="${en.projectSupervisor.detail.button}"]`).trigger('click');
    await flushPromises();

    const text = document.body.textContent;
    expect(text).toContain(en.projectSupervisor.detail.unknownHint);
    expect(text).toContain(en.projectSupervisor.detail.noConfig);
    expect(text).toContain(en.projectSupervisor.detail.noResources);
    pane.unmount();
  });
});

describe('what the manifest declares', () => {
  it('names a declared process the daemon is not running, and applies it on request', async () => {
    replies.supervisorProject = view({ pending: ['queue', 'scheduler'], stale: [] });
    // Applying answers with the same view, and the daemon now runs both.
    replies.supervisorApply = view({
      pending: [],
      stale: [],
      snapshot: {
        ...view().snapshot,
        processes: [
          process(),
          process({ fullName: 'nginx', name: 'nginx', group: 'nginx', pid: 9 }),
          process({ fullName: 'queue', name: 'queue', group: 'queue', pid: 12 }),
          process({ fullName: 'scheduler', name: 'scheduler', group: 'scheduler', pid: 13 }),
        ],
        summary: { total: 4, running: 4, stopped: 0, fatal: 0, other: 0, flapping: 0 },
      },
    });
    const pane = mountPane();
    await flushPromises();

    const banner = pane.find('[data-testid="supervisor-drift"]');
    expect(banner.exists()).toBe(true);
    expect(banner.text()).toContain('queue, scheduler');
    expect(banner.text()).toContain(en.projectSupervisor.apply);

    await banner.find('button').trigger('click');
    await flushPromises();

    expect(calls).toContainEqual(['supervisorApply', 'shop']);
    // The banner clears on the daemon's answer, and the rows are the answer.
    expect(pane.find('[data-testid="supervisor-drift"]').exists()).toBe(false);
    expect(pane.text()).toContain('4 of 4 running');
    expect(pane.text()).toContain('scheduler');
  });

  it('names a running process the manifest does not declare, without a button', async () => {
    replies.supervisorProject = view({ pending: [], stale: ['by-hand'] });
    const pane = mountPane();
    await flushPromises();

    const banner = pane.find('[data-testid="supervisor-drift"]');
    expect(banner.text()).toContain('by-hand');
    // Declaring it is a decision, not a click: no apply button for this one.
    expect(banner.find('button').exists()).toBe(false);
  });

  it('says nothing when the manifest and the daemon agree', async () => {
    const pane = mountPane();
    await flushPromises();
    expect(pane.find('[data-testid="supervisor-drift"]').exists()).toBe(false);
  });
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
