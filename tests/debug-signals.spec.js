import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The three things the debug bridge's file now holds, and how a row for each
 * one reads.
 *
 * The `kind` field carried one value for as long as it existed, and everything
 * arriving through the bridge was drawn as a dump. Two more values arrived —
 * an execution, written by PHP's own shutdown hook, and a queued job, folded
 * into the same file by the host out of the worker's own output — and the two
 * of them are *stretches* rather than moments: they have an outcome and a
 * duration and no captured value at all. What is pinned here is the three
 * places that distinction is easy to lose again:
 *
 *  - a job carries no SAPI, and "no SAPI" was read as "a script somebody ran",
 *    which filed every job under CLI;
 *  - a stretch has no value to render, and the dump renderer's summary of one
 *    would have been the word for an empty array;
 *  - a kind a build has never seen has to be SHOWN. A queue worker keeps the
 *    bridge it booted with for as long as it lives, so a newer producer and an
 *    older reader is the normal state during an update, and dropping the row
 *    would make that look like a bug in the application.
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
const DumpView = (await import('@/components/DumpView.vue')).default;

const vuetify = createVuetify({ components, directives });

const DUMP = {
  at: 1_787_940_507.1,
  kind: 'dump',
  label: 'user',
  file: '/var/www/html/app/Http/Controllers/Home.php',
  line: 42,
  request: 'GET /checkout',
  sapi: 'fpm-fcgi',
  value: { t: 'str', len: 5, v: 'alice' },
};

const REQUEST = {
  at: 1_787_940_507.2,
  kind: 'request',
  request: 'GET /checkout',
  sapi: 'fpm-fcgi',
  outcome: '503',
  duration: 23.4,
  value: { t: 'arr', n: 1, items: [{ k: 'memory', v: { t: 'num', v: 2097152 } }] },
};

const JOB = {
  at: 1_787_940_507.3,
  kind: 'job',
  label: 'App\\Jobs\\SendReceipt',
  outcome: 'failed',
  duration: 120,
  value: null,
};

function mountView(events) {
  replies.debugBridgeOverview = [
    { project: 'shop', enabled: true, mounted: true, running: true, events: events.length },
  ];
  replies.debugBridgeEvents = { total: events.length, events };

  return mount(
    {
      components: { DumpView },
      template: '<v-app><DumpView project="shop" scope="project" /></v-app>',
    },
    { global: { plugins: [createPinia(), vuetify, i18n] } }
  );
}

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
});

describe('the three signals on one list', () => {
  it('says how an execution ended and how long it took, in place of a value', async () => {
    const wrapper = mountView([REQUEST]);
    await vi.waitFor(() => expect(wrapper.text()).toContain('GET /checkout'));

    expect(wrapper.text()).toContain('503');
    expect(wrapper.text()).toContain('23.4 ms');
    // Not the dump renderer's word for the tree behind it: that tree is one
    // memory figure, and "array:1" is not what this row is about.
    expect(wrapper.text()).not.toContain('array:1');
    wrapper.unmount();
  });

  it('names the job class and how it ended', async () => {
    const wrapper = mountView([JOB]);
    await vi.waitFor(() => expect(wrapper.text()).toContain('App\\Jobs\\SendReceipt'));

    expect(wrapper.text()).toContain('failed');
    expect(wrapper.text()).toContain('120.0 ms');
    wrapper.unmount();
  });

  /**
   * A job is reported by the worker rather than by PHP, so it carries no SAPI.
   * Reading that absence as "a script" put every job under CLI — the one
   * grouping that is certainly wrong for the one row that is certainly a job.
   */
  it('files a job under the queue although it has no SAPI', async () => {
    const wrapper = mountView([JOB]);
    await vi.waitFor(() => expect(wrapper.text()).toContain('App\\Jobs\\SendReceipt'));

    expect(wrapper.text()).toContain('Queue');
    expect(wrapper.text()).not.toContain('CLI');
    wrapper.unmount();
  });

  /**
   * A worker keeps the bridge it booted with, so a build meeting a kind it has
   * no case for is the normal state during an update rather than a corner. The
   * row is still an event that happened.
   */
  it('shows a kind it has never heard of rather than dropping it', async () => {
    const unknown = { ...DUMP, kind: 'cache', label: 'users.42', value: { t: 'bool', v: true } };
    const wrapper = mountView([unknown]);
    await vi.waitFor(() => expect(wrapper.text()).toContain('users.42'));
    wrapper.unmount();
  });

  it('keeps all three, oldest at the bottom', async () => {
    const wrapper = mountView([DUMP, REQUEST, JOB]);
    await vi.waitFor(() => expect(wrapper.text()).toContain('App\\Jobs\\SendReceipt'));

    const rows = wrapper.findAll('.dump-row');
    expect(rows).toHaveLength(3);
    // Newest first — the pane unshifts, so the job that happened last is the
    // row nearest the toolbar.
    expect(rows[0].text()).toContain('App\\Jobs\\SendReceipt');
    expect(rows[2].text()).toContain('user');
    wrapper.unmount();
  });
});
