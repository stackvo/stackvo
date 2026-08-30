import { describe, it, expect, vi, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The Indicator pane, mounted.
 *
 * First pane out of `ProjectDetail.vue` in the pane split — 3,007 lines at 0%
 * coverage, and the last view still there.
 *
 * It takes every number as a prop, which is the design worth pinning: the
 * polling timer belongs to the view, so it can start and stop with the
 * container. A pane that polled on its own mount would keep a stopped
 * container's chart moving.
 */

globalThis.visualViewport = undefined;

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: new Proxy({}, { get: () => () => Promise.resolve(null) }),
}));

const { i18n } = await import('@/i18n');
const IndicatorPane = (await import('@/components/project/IndicatorPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const STATS = {
  cpuPercent: 12.5,
  memoryUsed: 300 * 1024 * 1024,
  memoryLimit: 1024 * 1024 * 1024,
  netRx: 5000,
  netTx: 2000,
  blockRead: 400,
  blockWrite: 600,
};

/**
 * Two slices with no colours on them, which is what the composable returns now.
 *
 * They used to arrive painted `#1976D2` and `#2A313C` — a copy of the default
 * accent and a copy of one theme's `surface-variant` — so the pane is where the
 * theme is read and the tests below are about what it reads.
 */
const PIE = [
  { key: 'a', title: 'A', value: 1 },
  { key: 'b', title: 'B', value: 2 },
];

/** A theme with nothing in common with the old literals. */
const THEME_COLOURS = {
  primary: '#7B1FA2',
  success: '#FF6D00',
  'surface-variant': '#EEDDCC',
};

const themed = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'probe',
    themes: { probe: { dark: false, colors: THEME_COLOURS } },
  },
});

function render(props = {}, plugin = vuetify) {
  const host = document.createElement('div');
  document.body.appendChild(host);

  return mount(
    { components: { IndicatorPane }, template: '<v-app><IndicatorPane v-bind="$attrs" /></v-app>' },
    { props, attrs: props, attachTo: host, global: { plugins: [plugin, i18n] } }
  );
}

afterEach(() => {
  document.body.innerHTML = '';
});

describe('the indicator pane', () => {
  /**
   * A project that has never been started has no sample at all. The pane has to
   * render its empty state rather than throwing on `stats.cpuPercent`.
   */
  it('renders with no stats at all', () => {
    const wrapper = render({ running: false });
    expect(wrapper.text().trim().length).toBeGreaterThan(0);
    wrapper.unmount();
  });

  it('shows the live figures once there is a sample', () => {
    const wrapper = render({
      running: true,
      stats: STATS,
      cpuSeries: [10, 11, 12],
      memoryPie: PIE,
      networkPie: PIE,
      diskPie: PIE,
    });

    // The CPU reading is the headline number on the page. Rounded to whole
    // percent by `percent(…, 0)` — a tenth of a percent of CPU is noise.
    expect(wrapper.text()).toContain('13%');
    wrapper.unmount();
  });

  /**
   * The grid distinguishes an hour nobody measured from an hour that measured
   * zero, and the class is what carries that — see `heatLevel`.
   */
  it('draws an unmeasured hour differently from an idle one', () => {
    const hours = Array.from({ length: 24 }, () => null);
    hours[9] = 0;
    hours[10] = 80;

    const wrapper = render({
      running: true,
      stats: STATS,
      heatmap: [{ label: new Date('2026-08-06T00:00:00'), hours }],
    });

    const html = wrapper.html();
    expect(html).toContain('heat-cell');
    expect(html, 'an unmeasured hour is not marked').toContain('empty');
    expect(html, 'an idle hour was drawn as unmeasured').toContain('l0');
    expect(html, 'a busy hour was not drawn hot').toContain('l4');

    wrapper.unmount();
  });

  /**
   * The charts follow the accent, which for a year they did not.
   *
   * Four colours were written into this pane and its composable as hex, and all
   * four were copies of a theme value. Move the accent to purple and the
   * application turned purple with three blue pie charts in it; switch to the
   * light theme and the second slice of each was dark charcoal on a white card.
   *
   * Mounted under a theme sharing nothing with the old literals, so a
   * reintroduced `#1976D2` cannot pass by resembling the answer.
   */
  it('paints the pies out of the theme, not out of a copy of it', () => {
    const wrapper = render(
      { running: true, stats: STATS, memoryPie: PIE, networkPie: PIE, diskPie: PIE },
      themed
    );

    const pies = wrapper.findAllComponents({ name: 'VPie' });
    expect(pies.length, 'the three pies did not render').toBe(3);

    for (const pie of pies) {
      const [measured, rest] = pie.props('items');
      expect(measured.color, 'the measured slice is not the accent').toBe(THEME_COLOURS.primary);
      expect(rest.color, 'the remainder is not the theme’s ground').toBe(
        THEME_COLOURS['surface-variant']
      );
    }

    wrapper.unmount();
  });

  /**
   * `success` and not a second literal green, and this is the half that matters
   * most: `success` is one of the three colours the colour-blind status palette
   * rewrites, so a hardcoded green ignored that choice on a chart made of
   * nothing but colour.
   */
  it('takes the sparkline gradient from the theme too', () => {
    const wrapper = render({ running: true, stats: STATS, cpuSeries: [1, 2, 3] }, themed);

    const spark = wrapper.findComponent({ name: 'VSparkline' });
    expect(spark.props('gradient')).toEqual([THEME_COLOURS.primary, THEME_COLOURS.success]);

    wrapper.unmount();
  });

  it('renders in Turkish', () => {
    i18n.global.locale.value = 'tr';
    try {
      const wrapper = render({ running: true, stats: STATS, memoryPie: PIE });
      expect(wrapper.text().trim().length).toBeGreaterThan(0);
      wrapper.unmount();
    } finally {
      i18n.global.locale.value = 'en';
    }
  });
});
