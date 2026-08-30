import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * Two machines, held against each other.
 *
 * The comparison itself is settled in Rust, where `diagnostics::compare` is
 * pure and has the cases with teeth in them. What only exists here is the
 * decision about *what the screen does with the answer*, and there are two of
 * those, both about not blurring a result into an empty state: only the
 * differences are drawn, and "nothing differs" is a sentence rather than a
 * blank box — because it means the difference is somewhere this cannot see,
 * which is the one thing the reader needs to be told.
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

// The dialog is the user's own choice of file and is not what this is about.
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(async () => '/tmp/theirs.zip'),
  save: vi.fn(async () => null),
}));

const { i18n } = await import('@/i18n');
const DiagnosticsPane = (await import('@/components/settings/DiagnosticsPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const mountPane = async () => {
  const wrapper = mount(
    {
      components: { DiagnosticsPane },
      template: '<v-app><DiagnosticsPane /></v-app>',
    },
    { global: { plugins: [createPinia(), vuetify, i18n] } }
  );
  await flushPromises();
  return wrapper;
};

const press = async (wrapper, label) => {
  await wrapper
    .findAllComponents({ name: 'VBtn' })
    .find((b) => b.text() === label)
    .trigger('click');
  await flushPromises();
};

beforeEach(() => {
  for (const key of Object.keys(replies)) delete replies[key];
  replies.logsInfo = { directory: '/logs', newestFile: null, totalBytes: 0 };
});

describe('comparing with another machine', () => {
  it('draws the differences and does not draw the agreement', async () => {
    replies.diagnosticsCompare = {
      differences: [
        { key: 'engine.version', here: '27.1.1', there: '25.0.3' },
        { key: 'service.redis-7-2', here: '7.2 on', there: '7.2 off' },
      ],
      same: 41,
      theirVersion: '0.1.0',
    };

    const wrapper = await mountPane();
    await press(wrapper, 'Compare with another machine');

    const rows = wrapper.findAll('[data-test="diff-row"]');
    expect(rows).toHaveLength(2);
    expect(rows[0].text()).toContain('engine.version');
    // Both sides on the row: "yours is 27, theirs is 25" is the whole answer,
    // and a column showing only that they differ would be half of it.
    expect(rows[0].text()).toContain('27.1.1');
    expect(rows[0].text()).toContain('25.0.3');
    // The forty-one that agree are counted, not listed.
    expect(wrapper.text()).toContain('41');
  });

  it('says a fact only one side states rather than leaving a blank cell', async () => {
    replies.diagnosticsCompare = {
      differences: [{ key: 'service.mysql-8-4', here: null, there: '8.4 on' }],
      same: 12,
    };

    const wrapper = await mountPane();
    await press(wrapper, 'Compare with another machine');

    expect(wrapper.find('[data-test="diff-row"]').text()).toContain('not stated');
  });

  it('reports two identical machines as an answer, not as nothing found', async () => {
    replies.diagnosticsCompare = { differences: [], same: 53 };

    const wrapper = await mountPane();
    await press(wrapper, 'Compare with another machine');

    expect(wrapper.findAll('[data-test="diff-row"]')).toHaveLength(0);
    expect(wrapper.text()).toContain('Nothing differs');
    expect(wrapper.text()).toContain('somewhere this cannot see');
  });

  it('shows why a file could not be read instead of an empty comparison', async () => {
    replies.diagnosticsCompare = () =>
      Promise.reject({ code: 'NOT_FOUND', message: 'that bundle has no environment.json' });

    const wrapper = await mountPane();
    await press(wrapper, 'Compare with another machine');

    expect(wrapper.findComponent({ name: 'ErrorAlert' }).exists()).toBe(true);
    expect(wrapper.findAll('[data-test="diff-row"]')).toHaveLength(0);
  });
});
