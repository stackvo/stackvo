import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The card that says what Docker cost today.
 *
 * The arithmetic is settled in Rust, where `usage.rs` is pure and holds the
 * cases with teeth in them — a laptop that slept, a day boundary, a budget of
 * zero. What only exists here are the two decisions the screen makes: a shared
 * service is not offered a budget, because it is nobody's to be over on; and a
 * machine that has measured nothing yet is a sentence rather than an empty
 * table.
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
          if (typeof reply === 'function') return Promise.resolve(reply(...args));
          return Promise.resolve(reply ?? null);
        },
    }
  ),
}));

const { i18n } = await import('@/i18n');
const UsageToday = (await import('@/components/UsageToday.vue')).default;

const vuetify = createVuetify({ components, directives });

const mountCard = async () => {
  const wrapper = mount(
    { components: { UsageToday }, template: '<v-app><UsageToday /></v-app>' },
    { global: { plugins: [vuetify, i18n] } }
  );
  await flushPromises();
  return wrapper;
};

const row = (over = {}) => ({
  name: 'shop',
  kind: 'project',
  cpuSeconds: 2280,
  gbHours: 4.2,
  samples: 40,
  overBudget: false,
  ...over,
});

beforeEach(() => {
  for (const key of Object.keys(replies)) delete replies[key];
  replies.prefsGet = {};
});

describe('what it cost today', () => {
  it('names each container, what it is, and what it used', async () => {
    replies.usageReport = {
      date: '2026-08-30',
      rows: [row(), row({ name: 'mysql-8-4', kind: 'service', cpuSeconds: 60, gbHours: 2 })],
      cpuSeconds: 2340,
      gbHours: 6.2,
    };

    const wrapper = await mountCard();
    const rows = wrapper.findAll('[data-test="usage-row"]');

    expect(rows).toHaveLength(2);
    expect(rows[0].text()).toContain('shop');
    expect(rows[0].text()).toContain('project');
    // Thirty-eight minutes, which is the sentence the feature was described by.
    expect(rows[0].text()).toContain('38');
    expect(rows[1].text()).toContain('service');
  });

  it('offers a budget on a project and never on a shared service', async () => {
    replies.usageReport = {
      date: '2026-08-30',
      rows: [row(), row({ name: 'mysql-8-4', kind: 'service' })],
      cpuSeconds: 0,
      gbHours: 0,
    };

    const wrapper = await mountCard();
    const rows = wrapper.findAll('[data-test="usage-row"]');

    expect(rows[0].findAllComponents({ name: 'VBtn' })).toHaveLength(1);
    // A shared service is not any one project's to be over on, so a field for
    // it would be a setting that does nothing.
    expect(rows[1].findAllComponents({ name: 'VBtn' })).toHaveLength(0);
  });

  it('sends the whole budget map, so one project does not clear another', async () => {
    replies.usageReport = { date: '2026-08-30', rows: [row()], cpuSeconds: 0, gbHours: 0 };
    replies.prefsGet = { usageBudgets: { blog: { cpuMinutes: 10 } } };
    const sent = [];
    replies.prefsSet = (patch) => {
      sent.push(patch);
      return patch;
    };

    const wrapper = await mountCard();
    await wrapper
      .findAll('[data-test="usage-row"]')[0]
      .findComponent({ name: 'VBtn' })
      .trigger('click');
    await flushPromises();

    const fields = wrapper.findAllComponents({ name: 'VTextField' });
    await fields[0].setValue('30');
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Save')
      .trigger('click');
    await flushPromises();

    // `prefs_set` merges shallowly, so the map has to arrive whole.
    expect(sent[0].usageBudgets.shop).toEqual({ cpuMinutes: 30, gbHours: undefined });
    expect(sent[0].usageBudgets.blog).toEqual({ cpuMinutes: 10 });
  });

  it('says nothing has been measured rather than drawing an empty table', async () => {
    replies.usageReport = { date: '2026-08-30', rows: [], cpuSeconds: 0, gbHours: 0 };

    const wrapper = await mountCard();

    expect(wrapper.findAll('[data-test="usage-row"]')).toHaveLength(0);
    expect(wrapper.text()).toContain('Nothing has been measured today yet');
  });

  it('says the record could not be read instead of "nothing ran"', async () => {
    replies.usageReport = () => {
      throw new Error('no config directory');
    };

    const wrapper = await mountCard();

    expect(wrapper.text()).toContain('could not be read');
    expect(wrapper.text()).not.toContain('Nothing has been measured today yet');
  });
});
