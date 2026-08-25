import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * B-1's screen — the three instruments around one request.
 *
 * The window, the split, the N+1 counting and the ranking are `explain.rs`'s
 * own tests and belong there: they are arithmetic over payloads and need no
 * browser. What is left for here is everything that only exists on screen, and
 * it is the half where this feature can go quietly wrong.
 *
 * Three claims in particular:
 *
 *   * **The join is stated.** The whole design rests on telling the reader that
 *     statements are attached to this request by time and not by attribution.
 *     A pane that dropped that sentence would look better and be dishonest, and
 *     nothing in Rust can notice.
 *   * **Absent is not empty.** `queriesRecording: false` and "the log was on
 *     and this request asked nothing" are one empty list and two different
 *     answers.
 *   * **Every finding renders as a sentence.** The payload carries a kind
 *     precisely so the window can say it in the reader's language; a kind with
 *     no translation renders as its own key, in an app that otherwise speaks
 *     two languages.
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
const WhySlowPane = (await import('@/components/project/WhySlowPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const REPORT = {
  key: 'spx-full-20260825-101500-abc',
  recordedAt: 1_786_801_015,
  cli: false,
  request: 'GET /checkout',
  wallTimeUs: 940_000,
  peakMemory: 12_582_912,
  callCount: 41_233,
  bytes: 4096,
};

/** The shape `spx_status` answers with, trimmed to what this pane reads. */
const STATUS = { supported: true, enabled: true, built: true, reports: [REPORT], bytes: 4096 };

/** A complete explanation, which each test then bends to the case it is about. */
const EXPLANATION = {
  key: REPORT.key,
  request: 'GET /checkout',
  cli: false,
  recordedAt: REPORT.recordedAt,
  wallTimeUs: REPORT.wallTimeUs,
  window: { from: 1_786_801_014, to: 1_786_801_016.94, basis: 'observed' },
  traceRead: true,
  truncated: false,
  functions: 812,
  split: {
    databaseUs: 600_000,
    databasePercent: 63.8,
    phpUs: 340_000,
    phpPercent: 36.2,
    drivers: [
      {
        function: 'PDOStatement::execute',
        calls: 240,
        exclusiveUs: 600_000,
        exclusivePercent: 63.8,
        inclusiveUs: 600_000,
        inclusivePercent: 63.8,
      },
    ],
  },
  hotspots: [
    {
      function: 'PDOStatement::execute',
      calls: 240,
      exclusiveUs: 600_000,
      exclusivePercent: 63.8,
      inclusiveUs: 600_000,
      inclusivePercent: 63.8,
    },
  ],
  queries: [
    {
      at: 1_786_801_015.2,
      sql: 'SELECT * FROM line_items WHERE order_id = 7',
      shape: 'SELECT * FROM line_items WHERE order_id = ?',
    },
  ],
  queryCount: 240,
  repeats: [
    {
      shape: 'SELECT * FROM line_items WHERE order_id = ?',
      count: 240,
      example: 'SELECT * FROM line_items WHERE order_id = 7',
    },
  ],
  queriesRecording: true,
  queriesElsewhere: 12,
  moments: [
    {
      at: 1_786_801_015.2,
      source: 'query',
      summary: 'SELECT * FROM line_items WHERE order_id = 7',
      request: null,
      shape: 'SELECT * FROM line_items WHERE order_id = ?',
    },
  ],
  requests: ['GET /checkout'],
  overlaps: [],
  builtins: true,
  findings: [
    { kind: 'nPlusOne', subject: 'SELECT * FROM line_items WHERE order_id = ?', count: 240 },
  ],
};

const mountPane = (runtime = 'php') =>
  mount(
    {
      components: { WhySlowPane },
      template: `<v-app><WhySlowPane name="shop" runtime="${runtime}" /></v-app>`,
    },
    { global: { plugins: [vuetify, i18n] } }
  );

/** Mounted and loaded, with the explanation bent by `patch`. */
async function paneWith(patch = {}) {
  replies.spxStatus = STATUS;
  replies.dbTargets = [{ service: 'mysql', kind: 'mysql', running: true, enabled: true }];
  replies.requestExplain = { ...EXPLANATION, ...patch };
  const pane = mountPane();
  await flushPromises();
  return pane;
}

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  i18n.global.locale.value = 'en';
});

describe('opening', () => {
  it('opens on the newest recording rather than on an empty picker', async () => {
    const pane = await paneWith();

    const call = calls.find(([name]) => name === 'requestExplain');
    expect(call, 'the pane asked for an explanation without being told to').toBeTruthy();
    expect(call[1]).toBe('shop');
    expect(call[2]).toBe(REPORT.key);
    expect(pane.text()).toContain('GET /checkout');
  });

  it('passes the first readable database, so the query half is not silently absent', async () => {
    await paneWith();
    expect(calls.find(([name]) => name === 'requestExplain')[3]).toBe('mysql');
  });

  it('says how to get a recording rather than showing an empty picker', async () => {
    replies.spxStatus = { ...STATUS, reports: [] };
    replies.dbTargets = [];
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain(en.whySlow.nothingRecorded);
    expect(calls.some(([name]) => name === 'requestExplain')).toBe(false);
  });

  /**
   * php-spx is a PHP extension, so a Node project has no recording this could
   * ever open on. A card explaining that it does not apply is an entry the
   * reader has to scroll past forever.
   */
  it('is not on the page at all for a project that cannot record', async () => {
    replies.spxStatus = STATUS;
    const pane = mountPane('node');
    await flushPromises();

    expect(pane.find('.pane').exists()).toBe(false);
    expect(calls.some(([name]) => name === 'spxStatus')).toBe(false);
  });
});

describe('refreshing', () => {
  /**
   * The recordings are made and deleted in the php-spx card on this same tab,
   * and one of its buttons deletes all of them. A refresh that kept pointing at
   * a key nothing answers to would turn the pane into an error, and
   * `request_explain` is deliberately an error on an unknown key rather than an
   * empty explanation.
   */
  it('drops to the newest recording when the pinned one is gone', async () => {
    const pane = await paneWith();
    const survivor = { ...REPORT, key: 'spx-full-20260825-104500-def' };
    replies.spxStatus = { ...STATUS, reports: [survivor] };

    await pane.find('.pane-head button').trigger('click');
    await flushPromises();

    const last = calls.filter(([name]) => name === 'requestExplain').at(-1);
    expect(last[2]).toBe(survivor.key);
  });

  it('stays on the recording it was reading when that one survived', async () => {
    const pane = await paneWith();
    replies.spxStatus = { ...STATUS, reports: [{ ...REPORT, key: 'newer' }, REPORT] };

    await pane.find('.pane-head button').trigger('click');
    await flushPromises();

    const last = calls.filter(([name]) => name === 'requestExplain').at(-1);
    expect(last[2]).toBe(REPORT.key);
  });
});

describe('the join', () => {
  /**
   * The sentence the whole design rests on. Statements carry no request and are
   * attached to this one by time; a pane that showed them without saying so
   * would be claiming an attribution nothing measured.
   */
  it('says the statements are joined by time and not attributed', async () => {
    const pane = await paneWith();
    expect(pane.text()).toContain(en.whySlow.window);
  });

  /**
   * The evidence is closed until it is asked for — it is evidence, not the
   * finding. Opened here so the assertion is about what the list holds rather
   * than about whether Vuetify renders a collapsed panel's contents.
   */
  it('shows the statement on the axis, still carrying no request', async () => {
    const pane = await paneWith();

    const axis = pane
      .findAll('.v-expansion-panel-title')
      .find((title) => title.text().startsWith('On one axis'));
    expect(axis, 'the axis has its own list').toBeTruthy();
    await axis.trigger('click');
    await flushPromises();

    // The payload's own claim, which the pane must not decorate away.
    expect(EXPLANATION.moments[0].request).toBeNull();
    expect(pane.text()).toContain('SELECT * FROM line_items WHERE order_id = 7');
    // Placed by how far into the request it landed, not by clock time — the
    // axis reads as one page load.
    expect(pane.text()).toContain('+1.200s');
  });
});

describe('the window basis', () => {
  /**
   * A window this app watched and a window it worked out are not equally
   * trustworthy — the second rests on a reading of php-spx's `exec_ts` that
   * nothing in this repository has measured. A pane that rendered them
   * identically would be hiding the difference the payload exists to carry.
   */
  it('says the stretch was watched when this app sent the request', async () => {
    const pane = await paneWith();
    expect(pane.text()).toContain(en.whySlow.windowObserved);
    expect(pane.text()).not.toContain(en.whySlow.windowDerived);
  });

  it('says the stretch was worked out when it was not', async () => {
    const pane = await paneWith({
      window: { from: 1_786_801_014, to: 1_786_801_016.94, basis: 'derived' },
    });
    expect(pane.text()).toContain(en.whySlow.windowDerived);
    expect(pane.text()).not.toContain(en.whySlow.windowObserved);
  });
});

describe('findings', () => {
  /**
   * A kind with no translation renders as `whySlow.finding.someKind`, which is
   * a sentence nobody can read and a bug no Rust test can see.
   */
  it('renders every kind the contract declares, in both locales', async () => {
    const kinds = Object.keys(en.whySlow.finding);
    expect(kinds.length).toBeGreaterThan(0);

    for (const locale of ['en', 'tr']) {
      i18n.global.locale.value = locale;
      const pane = await paneWith({
        findings: kinds.map((kind) => ({ kind, subject: 'x', count: 3, percent: 40 })),
      });
      const text = pane.text();
      for (const kind of kinds) {
        expect(text, `${locale}: ${kind} rendered as its own key`).not.toContain(
          `whySlow.finding.${kind}`
        );
      }
    }
  });

  it('puts the numbers the payload carries into the sentence', async () => {
    const pane = await paneWith();
    expect(pane.text()).toContain('240');
  });

  it('says so plainly when nothing stood out', async () => {
    const pane = await paneWith({ findings: [] });
    expect(pane.text()).toContain(en.whySlow.nothingToSay);
  });
});

describe('the database half', () => {
  /**
   * `queriesRecording: false` and "recording, and this request asked nothing"
   * are one empty list and two different answers — which is the distinction
   * the flag exists to carry, and the one a screen is most likely to flatten.
   */
  it('tells an unrecorded log apart from a request that asked nothing', async () => {
    const off = await paneWith({
      queries: [],
      queryCount: 0,
      repeats: [],
      queriesRecording: false,
      queriesElsewhere: 0,
      findings: [{ kind: 'queriesUnrecorded' }],
    });
    expect(off.text()).toContain(en.whySlow.finding.queriesUnrecorded);

    const quiet = await paneWith({
      queries: [],
      queryCount: 0,
      repeats: [],
      queriesRecording: true,
      queriesElsewhere: 0,
      findings: [],
    });
    expect(quiet.text()).not.toContain(en.whySlow.finding.queriesUnrecorded);
  });

  it('names the count outside the window rather than showing an empty list', async () => {
    const pane = await paneWith({
      queries: [],
      queryCount: 0,
      repeats: [],
      queriesElsewhere: 12,
      findings: [{ kind: 'queriesOutsideWindow', count: 12 }],
    });
    expect(pane.text()).toContain('12');
  });
});

describe('where the time went', () => {
  it('draws the split and labels both halves', async () => {
    const pane = await paneWith();

    expect(pane.find('.bar-db').attributes('style')).toContain('63.8%');
    expect(pane.find('.bar-php').attributes('style')).toContain('36.2%');
    expect(pane.text()).toContain(en.whySlow.inDatabase);
    expect(pane.text()).toContain(en.whySlow.inPhp);
  });

  /**
   * The bar is the only place the proportion is drawn rather than written, so
   * it carries the same figures for a reader who cannot see it.
   */
  it('gives the bar a label a screen reader can read', async () => {
    const pane = await paneWith();
    const label = pane.find('.bar').attributes('aria-label');
    expect(label).toContain('64');
    expect(label).toContain('36');
  });

  /**
   * A recording with no trace half still says what the request was, when it
   * ran, and what the database was asked. Half an explanation is worth more
   * than an error page — the rule the whole feature follows.
   */
  it('survives a recording whose trace could not be read', async () => {
    const pane = await paneWith({
      traceRead: false,
      split: null,
      hotspots: [],
      functions: 0,
      findings: [{ kind: 'traceMissing' }],
    });

    expect(pane.find('.bar').exists()).toBe(false);
    expect(pane.text()).toContain('GET /checkout');
    expect(pane.text()).toContain(en.whySlow.finding.traceMissing);
  });
});

describe('the counts', () => {
  /**
   * The lists are capped and the counts are not, and the heading is where that
   * is visible: a heading that said "Statements (1)" over a capped list would
   * report the cap as the finding.
   */
  it('heads the statements with what was found, not with what is shown', async () => {
    const pane = await paneWith();
    const text = pane.text();

    expect(text).toContain('Statements (240)');
    expect(text).toContain('Functions (812 in the trace)');
  });
});
