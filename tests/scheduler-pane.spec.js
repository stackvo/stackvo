import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * Named jobs on a timer, for one project.
 *
 * What these cover is the part that is not obvious from looking at the pane:
 * the form assembles an argv rather than a command line, an existing job has to
 * reopen in the form it was written in, and a schedule with no sidecar running
 * it must not read as scheduled. The cron grammar itself is not tested here —
 * `cron.rs` holds that, and the tick script is held against it by its own
 * test; this holds what the screen does with the answers.
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
const SchedulerPane = (await import('@/components/project/SchedulerPane.vue')).default;
const { argvFor, kindOf, presetFor, textOf } = await import('@/composables/useScheduler');

const vuetify = createVuetify({ components, directives });

const CLEANUP = {
  id: 'cache-cleanup',
  label: 'Cache cleanup',
  cron: '*/5 * * * *',
  exec: ['php', 'artisan', 'cache:clear'],
  command: 'php artisan cache:clear',
  enabled: true,
  lastRun: { at: '2026-08-24 09:05:00', ok: true, status: 0 },
};

const view = (over = {}) => ({
  jobs: [CLEANUP],
  running: true,
  restarts: null,
  buildable: true,
  ...over,
});

const mountPane = (name = 'shop', running = true) =>
  mount(
    {
      components: { SchedulerPane },
      props: ['name', 'running'],
      template: '<v-app><SchedulerPane :name="name" :running="running" /></v-app>',
    },
    { props: { name, running }, global: { plugins: [vuetify, i18n] } }
  );

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  replies.schedulerJobs = view();
  replies.schedulerSave = view();
});

describe('the form', () => {
  /**
   * A command line typed into a box is not what gets stored. The manifest
   * holds an argv, and the whole no-shell rule depends on the pane never
   * handing a string down to be re-split later.
   */
  it('builds an argv from the kind and what was typed', () => {
    expect(argvFor('laravel', '')).toEqual(['php', 'artisan', 'schedule:run']);
    expect(argvFor('artisan', 'cache:clear --quiet')).toEqual([
      'php',
      'artisan',
      'cache:clear',
      '--quiet',
    ]);
    expect(argvFor('custom', 'sh scripts/nightly.sh')).toEqual(['sh', 'scripts/nightly.sh']);
    expect(argvFor('custom', '   ')).toEqual([]);
  });

  /**
   * The kind is not stored — it is read back out of the argv. A kind in the
   * file would be a second description of the same command, and the two would
   * eventually disagree about what a job is.
   */
  it('reopens an existing job in the form it was written in', () => {
    expect(kindOf(['php', 'artisan', 'schedule:run'])).toBe('laravel');
    expect(kindOf(['php', 'artisan', 'cache:clear'])).toBe('artisan');
    expect(kindOf(['sh', 'scripts/nightly.sh'])).toBe('custom');

    // And what the text box should hold for each, which is not the argv.
    expect(textOf(['php', 'artisan', 'cache:clear', '--quiet'])).toBe('cache:clear --quiet');
    expect(textOf(['sh', 'scripts/nightly.sh'])).toBe('sh scripts/nightly.sh');
  });

  /**
   * A preset is a spelling of an expression, never a second way to store one.
   * An expression nobody has a preset for has to fall through to Advanced
   * rather than silently become the nearest preset.
   */
  it('maps an expression back to the preset it is, or to none', () => {
    expect(presetFor('*/5 * * * *')).toBe('every5');
    expect(presetFor('0 3 * * *')).toBe('nightly');
    expect(presetFor('30 2 * * 1')).toBe(null);
  });

  it('saves the whole list, and only the fields the manifest stores', async () => {
    const pane = mountPane();
    await flushPromises();

    await pane.findAll('button').find((b) => b.text() === 'New job').trigger('click');
    await flushPromises();

    // Found by its own label rather than by position: a v-select renders a
    // text input too, so an index here would silently start typing into the
    // frequency dropdown the next time the form is reordered.
    const label = pane
      .findAll('.v-text-field')
      .find((f) => f.text().includes(en.scheduler.label));
    await label.find('input').setValue('Nightly prune');
    await flushPromises();

    await pane.findAll('button').find((b) => b.text() === 'Save').trigger('click');
    await flushPromises();

    const [, name, jobs] = calls.find(([n]) => n === 'schedulerSave');
    expect(name).toBe('shop');
    // The existing job travels with the new one: the schedule is one value,
    // and saving one row means saving all of them.
    expect(jobs).toHaveLength(2);
    expect(jobs[1]).toEqual({
      label: 'Nightly prune',
      cron: '* * * * *',
      exec: ['php', 'artisan', 'schedule:run'],
      enabled: true,
    });
    // Nothing the screen added — no id, no command line, no last run.
    expect(Object.keys(jobs[0]).sort()).toEqual(['cron', 'enabled', 'exec', 'label']);
  });
});

describe('the list', () => {
  it('says a frequency in words when it has one, and the expression when it does not', async () => {
    replies.schedulerJobs = view({
      jobs: [CLEANUP, { ...CLEANUP, id: 'odd', label: 'Odd', cron: '30 2 * * 1' }],
    });
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain('Every 5 minutes');
    expect(pane.text()).toContain('30 2 * * 1');
  });

  /**
   * A schedule with nothing running it is a list of intentions. Showing the
   * jobs without saying that would be the screen agreeing that they are
   * scheduled when nothing is going to fire.
   */
  it('says the scheduler is stopped rather than showing the jobs as scheduled', async () => {
    replies.schedulerJobs = view({ running: false });
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain('Scheduler is stopped');
    expect(pane.text()).not.toContain('Scheduler is running');
  });

  it('distinguishes never-ran from ran-and-failed', async () => {
    replies.schedulerJobs = view({
      jobs: [
        { ...CLEANUP, id: 'fresh', label: 'Fresh', lastRun: null },
        {
          ...CLEANUP,
          id: 'broken',
          label: 'Broken',
          lastRun: { at: '2026-08-24 09:05:00', ok: false, status: 123 },
        },
      ],
    });
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain('Has not run yet');
    expect(pane.text()).toContain('Last run failed: 2026-08-24 09:05:00');
  });

  /**
   * Pausing keeps the command, which is the part that took effort to write.
   * A pane that paused by deleting would be one where undo means retyping.
   */
  it('pauses a job by saving it disabled rather than by removing it', async () => {
    const pane = mountPane();
    await flushPromises();

    await pane.find('button[title="Pause"]').trigger('click');
    await flushPromises();

    const [, , jobs] = calls.find(([n]) => n === 'schedulerSave');
    expect(jobs).toHaveLength(1);
    expect(jobs[0].enabled).toBe(false);
    expect(jobs[0].exec).toEqual(['php', 'artisan', 'cache:clear']);
  });

  /**
   * Running a job by hand is only possible through the sidecar, because that
   * is what writes the log and the last run. Offering the button while it is
   * down would be offering a failure.
   */
  it('offers "run now" only while the scheduler is up', async () => {
    const up = mountPane();
    await flushPromises();
    expect(up.find('button[title="Run now"]').attributes('disabled')).toBeUndefined();

    replies.schedulerJobs = view({ running: false });
    const down = mountPane();
    await flushPromises();
    expect(down.find('button[title="Run now"]').attributes('disabled')).toBeDefined();
  });
});
