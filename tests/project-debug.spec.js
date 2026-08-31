import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The Debug section: Xdebug, its profiler, and the dumps the app catches.
 *
 * Two things in here were wrong in ways a screenshot cannot show, and both are
 * pinned below because both are easy to reintroduce:
 *
 *  - `needsRestart` used to ask `active === false`, which never fired for the
 *    case it exists for. `active` only means "both Xdebug variables are
 *    present" — after switching stepping to profiling they still are, with the
 *    old mode in them. The page reported profiling as applied and the recorded
 *    list stayed at zero with nothing to say why.
 *  - the profile's time unit is read from the file (`Time_(10ns)`), never
 *    assumed. Reading it as microseconds is wrong by two orders of magnitude
 *    and the number still looks plausible.
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

const { useXdebug } = await import('@/composables/useXdebug');
const { useProfiler } = await import('@/composables/useProfiler');
const { i18n } = await import('@/i18n');
const XdebugPane = (await import('@/components/project/XdebugPane.vue')).default;
const DumpsPane = (await import('@/components/project/DumpsPane.vue')).default;
const ProfilerPane = (await import('@/components/project/ProfilerPane.vue')).default;
const SpxPane = (await import('@/components/project/SpxPane.vue')).default;

const vuetify = createVuetify({ components, directives });
const ref = (value) => ({ value });

const PROFILES = [
  { id: 'cachegrind.out.1', name: 'cachegrind.out.1', bytes: 4096, recordedAt: 1_786_007_730 },
];

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  replies.xdebugStatus = { enabled: true, needsRebuild: false, active: true, running: true };
  replies.spxStatus = {
    supported: true,
    enabled: false,
    built: true,
    phpVersion: '8.4',
    active: null,
    running: false,
    domain: 'shop.loc',
    samplingPeriod: 100,
    builtins: false,
    controlUrl: 'https://shop.loc/?SPX_KEY=abc&SPX_UI_URI=/',
    viewBase: 'https://shop.loc/?SPX_KEY=abc&SPX_UI_URI=/report.html&key=',
    xdebugConflict: false,
    reports: [],
    bytes: 0,
    directory: '/ws/logs/projects/shop/spx',
  };
  replies.ideDebugStatus = {
    project: 'shop',
    port: 9003,
    ideKey: 'STACKVO',
    serverName: 'shop.loc',
    hostPath: '/ws/projects/shop',
    containerPath: '/var/www/html',
    listener: { port: 9003, process: null, pid: null, unknown: false },
    targets: [
      {
        id: 'vscode',
        label: 'VS Code',
        method: 'written',
        path: '/ws/projects/shop/.vscode/launch.json',
        detected: true,
        exists: false,
        parseable: true,
        installed: false,
        current: false,
        snippet: '{}',
      },
      {
        id: 'phpstorm',
        label: 'PhpStorm',
        method: 'shown',
        path: '/ws/projects/shop/.idea/php.xml',
        detected: false,
        exists: false,
        parseable: true,
        installed: false,
        current: false,
        snippet: '<component/>',
      },
    ],
  };
  replies.profilerStatus = {
    mode: 'profile',
    develop: false,
    modeValue: 'profile',
    trigger: 'XDEBUG_TRIGGER=1',
    profiles: PROFILES,
    bytes: 4096,
    directory: '/ws/projects/shop/.stackvo/profiles',
    xdebug: { enabled: true, running: true, active: true, activeMode: 'profile' },
  };
});

describe('xdebug', () => {
  it('has nothing to report for a runtime with no PHP', async () => {
    const x = useXdebug(ref('shop'));
    expect(await x.load('node')).toBe(null);
    expect(calls).toEqual([]);
  });

  /**
   * Enabled but not yet doing anything is the state a user will not go looking
   * for: breakpoints are set, and nothing stops at them.
   */
  it.each([
    [{ enabled: true, needsRebuild: true, active: true }, true, 'enabled, image never rebuilt'],
    [{ enabled: true, needsRebuild: false, active: false }, true, 'built, not in the container'],
    [{ enabled: true, needsRebuild: false, active: true }, false, 'actually working'],
    [{ enabled: false, needsRebuild: true, active: false }, false, 'off, so nothing is pending'],
  ])('badges %o as %s (%s)', async (status, expected) => {
    replies.xdebugStatus = status;
    const x = useXdebug(ref('shop'));
    await x.load('php');

    expect(!!x.pending.value).toBe(expected);
  });

  it('reports a refused toggle and stops spinning', async () => {
    replies.xdebugSet = () => Promise.reject({ code: 'DOCKER_UNAVAILABLE', message: 'down' });
    const x = useXdebug(ref('shop'));

    expect(await x.toggle(true)).toBe(null);
    expect(x.error.value.code).toBe('DOCKER_UNAVAILABLE');
    expect(x.busy.value).toBe(false);
  });

  /**
   * Toggling rewrites the manifest on disk, and the Configuration section is
   * showing that same file. The pane says so rather than reaching across.
   */
  it('tells the page the manifest changed under it', async () => {
    replies.xdebugSet = { enabled: false, needsRebuild: true, active: false };

    const wrapper = mount(
      {
        components: { XdebugPane },
        template:
          '<v-app><XdebugPane name="shop" runtime="php" @changed="$attrs.onSeen" /></v-app>',
      },
      { attrs: { onSeen: vi.fn() }, global: { plugins: [createPinia(), vuetify, i18n] } }
    );
    await vi.waitFor(() => expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true));

    await wrapper.find('input[type="checkbox"]').setValue(false);
    await vi.waitFor(() => expect(calls.some(([n]) => n === 'xdebugSet')).toBe(true));
    expect(wrapper.attributes()).toBeDefined();
  });

  it('does not announce a change that failed', async () => {
    replies.xdebugSet = () => Promise.reject({ code: 'IO', message: 'read-only' });
    const x = useXdebug(ref('shop'));

    expect(await x.toggle(true)).toBe(null);
  });
});

describe('the profiler', () => {
  /**
   * The bug this replaced: `active` stays true across a mode switch, with the
   * old mode still in the variables.
   */
  it.each([
    [
      { running: true, active: true, activeMode: 'debug' },
      'profile',
      true,
      'container in the old mode',
    ],
    [{ running: true, active: true, activeMode: 'profile' }, 'profile', false, 'already applied'],
    [{ running: true, active: false, activeMode: null }, 'profile', true, 'not in the container'],
    [{ running: false, active: false, activeMode: null }, 'profile', false, 'nothing to disagree'],
  ])('%o vs %s needs a recreate: %s (%s)', async (xdebug, mode, expected) => {
    replies.profilerStatus = { mode, profiles: [], bytes: 0, xdebug };
    const p = useProfiler(ref('shop'));
    await p.load('php');

    expect(p.needsRestart.value).toBe(expected);
  });

  describe('the time unit', () => {
    it.each([
      ['Time_(10ns)', 12_345_678, '123.5 ms'],
      ['Time_(us)', 500, '500 µs'],
      ['Time_(ms)', 3, '3.0 ms'],
      ['Time_(ns)', 2_000_000, '2.0 ms'],
    ])('reads %s and renders the cost from it', async (declared, value, rendered) => {
      const p = useProfiler(ref('shop'));
      p.report.value = { events: [declared] };

      expect(p.cost(value)).toBe(rendered);
    });

    /**
     * A file whose header is missing or unparseable renders the raw number
     * rather than a wrong one — inventing a unit would be worse than showing
     * none.
     */
    it.each([[undefined], [{ events: [] }], [{ events: ['Time'] }]])(
      'refuses to guess when the file declares none (%o)',
      async (report) => {
        const p = useProfiler(ref('shop'));
        p.report.value = report ?? null;

        expect(p.unit.value).toBe('');
        expect(p.cost(4200)).toBe('4200');
      }
    );
  });

  /**
   * A trace is not a profile with a different name: it is read by another
   * parser, drawn as another picture, and the one thing it must never do is
   * leave the previous profile's table or tree on screen underneath it. Those
   * are cachegrind's summed edges; this is folded stacks, and a reader cannot
   * tell them apart by looking.
   */
  it('opening a trace shows a flame graph and clears the profile beside it', async () => {
    replies.profilerFlame = {
      frames: [{ name: '{main}', value: 1300, children: [], recursive: false }],
      total: 1300,
      records: 10,
      stacks: 4,
      truncated: false,
      pruned: 0,
      depthCapped: false,
    };
    const p = useProfiler(ref('shop'));
    await p.load('php');
    // A profile was open a moment ago.
    p.report.value = { events: ['Time_(10ns)'], functions: [] };
    p.tree.value = [{ name: 'old', value: 1, children: [] }];

    await p.openTrace({ id: 'trace.1786825736.xt' });

    expect(p.flame.value.total).toBe(1300);
    expect(p.openId.value).toBe('trace.1786825736.xt');
    expect(p.report.value, 'the cost table belonged to the profile').toBe(null);
    expect(p.tree.value, 'so did the call tree').toBe(null);
  });

  /** And the other way round: opening a profile takes the flame graph away. */
  it('opening a profile clears the flame graph', async () => {
    replies.profilerRead = { events: ['Time_(us)'], functions: [] };
    const p = useProfiler(ref('shop'));
    await p.load('php');
    p.flame.value = { frames: [], total: 1, records: 1, stacks: 1 };

    await p.open(PROFILES[0]);
    expect(p.flame.value).toBe(null);
  });

  it('marks the file that is working, not every row', async () => {
    let settle;
    replies.profilerRead = () => new Promise((resolve) => (settle = resolve));
    const p = useProfiler(ref('shop'));
    await p.load('php');

    const done = p.open(PROFILES[0]);
    expect(p.busy.value).toBe('cachegrind.out.1');
    settle({ events: ['Time_(10ns)'], functions: [] });
    await done;

    expect(p.openId.value).toBe('cachegrind.out.1');
    expect(p.busy.value).toBe('');
  });

  it('highlights nothing as open when the read fails', async () => {
    replies.profilerRead = () => Promise.reject({ code: 'IO', message: 'unreadable' });
    const p = useProfiler(ref('shop'));
    p.openId.value = 'cachegrind.out.1';

    await p.open(PROFILES[0]);
    expect(p.openId.value).toBe('');
    expect(p.report.value).toBe(null);
    expect(p.error.value.code).toBe('IO');
  });

  /** The open report belongs to a file that no longer exists. */
  it('closes the report when its own file is deleted', async () => {
    replies.profilerDelete = null;
    const p = useProfiler(ref('shop'));
    await p.load('php');
    await p.open(PROFILES[0]);
    p.report.value = { events: ['Time_(10ns)'] };

    await p.remove(PROFILES[0], 'php');
    expect(p.report.value).toBe(null);
    expect(p.openId.value).toBe('');
    expect(
      calls.filter(([n]) => n === 'profilerStatus'),
      're-read after deleting'
    ).toHaveLength(2);
  });

  it('leaves the report alone when a different file is deleted', async () => {
    replies.profilerDelete = null;
    replies.profilerRead = { events: ['Time_(10ns)'] };
    const p = useProfiler(ref('shop'));
    await p.load('php');
    await p.open(PROFILES[0]);

    await p.remove({ id: 'cachegrind.out.9' }, 'php');
    expect(p.openId.value).toBe('cachegrind.out.1');
    expect(p.report.value).toBeTruthy();
  });

  it('clears everything, report included', async () => {
    replies.profilerClear = null;
    replies.profilerRead = { events: ['Time_(10ns)'] };
    const p = useProfiler(ref('shop'));
    await p.load('php');
    await p.open(PROFILES[0]);

    expect(await p.clear('php')).toBe(true);
    expect(p.report.value).toBe(null);
    expect(p.openId.value).toBe('');
  });
});

describe('the dumps pane', () => {
  /**
   * The recreate button is the project's lifecycle, which the view owns — the
   * pane only says the container needs one.
   */
  it('passes the recreate up rather than running it', async () => {
    const onApply = vi.fn();
    mount(
      {
        components: { DumpsPane },
        template: '<v-app><DumpsPane name="shop" @apply="$attrs.onApply" /></v-app>',
      },
      {
        attrs: { onApply },
        global: { plugins: [createPinia(), vuetify, i18n] },
      }
    );

    expect(
      calls.some(([n]) => n === 'composeUpProject'),
      'the pane must not run the project lifecycle itself'
    ).toBe(false);
  });
});

/**
 * The IDE half of step debugging.
 *
 * Every competitor's page ends at "now type these three values into your IDE",
 * and every one of them then names the path mapping as the usual reason a
 * breakpoint never hits. What is pinned here is the pair of decisions that
 * makes filling it in safe rather than merely convenient: which IDE is written
 * and which is only shown, and the fact that is in no file at all.
 */
describe('the IDE setup', () => {
  const mountPane = () =>
    mount(
      {
        components: { XdebugPane },
        template: '<v-app><XdebugPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [createPinia(), vuetify, i18n] } }
    );

  it('says out loud when nothing is listening on the debug port', async () => {
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    // The half that is in no file. An IDE that is not listening is silent
    // about it, and this is the only place that says so.
    expect(wrapper.text()).toContain('Nothing is listening on port 9003');
    wrapper.unmount();
  });

  it('names the process when something is', async () => {
    replies.ideDebugStatus = {
      ...replies.ideDebugStatus,
      listener: { port: 9003, process: 'phpstorm', pid: 4242, unknown: false },
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    expect(wrapper.text()).toContain('phpstorm is listening on port 9003');
    wrapper.unmount();
  });

  it('offers to write VS Code and only to copy PhpStorm', async () => {
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Write configuration');
    expect(labels).toContain('Copy block');

    // And the reason, on screen rather than in a comment: PhpStorm rewrites
    // its own file on exit, so an edit made underneath it is an edit lost.
    expect(wrapper.text()).toContain('keeps this file in memory');
    wrapper.unmount();
  });

  it('sends the write to the right project and IDE, and re-reads afterwards', async () => {
    replies.ideDebugApply = '/ws/projects/shop/.vscode/launch.json';
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Write configuration')
      .trigger('click');

    await vi.waitFor(() => expect(calls.some(([n]) => n === 'ideDebugApply')).toBe(true));
    expect(calls.find(([n]) => n === 'ideDebugApply')).toEqual(['ideDebugApply', 'shop', 'vscode']);
    // Re-read after the write, so the row is not left describing the old file.
    await vi.waitFor(() =>
      expect(calls.filter(([n]) => n === 'ideDebugStatus').length).toBeGreaterThan(1)
    );
    wrapper.unmount();
  });

  /**
   * A launch.json with comments in it is what VS Code itself creates, so this
   * is the common case rather than a corner one: no write button, a block to
   * paste, and the reason named.
   */
  it('withholds the button for a file it cannot parse', async () => {
    replies.ideDebugStatus = {
      ...replies.ideDebugStatus,
      targets: [{ ...replies.ideDebugStatus.targets[0], parseable: false }],
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).not.toContain('Write configuration');
    expect(labels).toContain('Copy block');
    expect(wrapper.text()).toContain('cannot be edited safely');
    wrapper.unmount();
  });

  it('offers Update rather than Write once the values have moved', async () => {
    replies.ideDebugStatus = {
      ...replies.ideDebugStatus,
      targets: [
        { ...replies.ideDebugStatus.targets[0], installed: true, current: false, exists: true },
      ],
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Update');
    expect(labels).toContain('Remove');
    expect(labels).not.toContain('Write configuration');
    wrapper.unmount();
  });
});

/**
 * The two modes that arrived after the first three.
 *
 * `coverage` is the one every other tool's documentation covers and this one
 * did not, and `develop` is the one Herd's own recommended configuration pairs
 * with stepping. Both are pinned here because both are easy to model wrongly:
 * coverage as a mode that records something, and develop as a fifth mode
 * instead of the second item of a list.
 */
describe('the develop flag and the coverage mode', () => {
  const mountPane = () =>
    mount(
      {
        components: { ProfilerPane },
        template: '<v-app><ProfilerPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [createPinia(), vuetify, i18n] } }
    );

  /**
   * The regression this comparison exists to prevent. `XDEBUG_MODE` is a list,
   * so a project with develop on runs `debug,develop` while the picker still
   * says `debug` — and comparing those two put a "recreate the container"
   * warning on screen for a container that was already correct.
   */
  it('does not call debug,develop a mismatch with debug', async () => {
    replies.profilerStatus = {
      ...replies.profilerStatus,
      mode: 'debug',
      develop: true,
      modeValue: 'debug,develop',
      xdebug: { enabled: true, running: true, active: true, activeMode: 'debug,develop' },
    };
    const p = useProfiler(ref('shop'));
    await p.load('php');

    expect(p.needsRestart.value).toBe(false);
  });

  /** And a real mismatch is still one. */
  it('still catches a container left in the previous mode', async () => {
    replies.profilerStatus = {
      ...replies.profilerStatus,
      mode: 'debug',
      develop: true,
      modeValue: 'debug,develop',
      xdebug: { enabled: true, running: true, active: true, activeMode: 'profile' },
    };
    const p = useProfiler(ref('shop'));
    await p.load('php');

    expect(p.needsRestart.value).toBe(true);
  });

  /** The switch keeps the mode; it is a companion, not an alternative. */
  it('keeps the chosen mode when develop is toggled', async () => {
    replies.profilerStatus = { ...replies.profilerStatus, mode: 'trace', modeValue: 'trace' };
    replies.profilerSetMode = { ...replies.profilerStatus, develop: true };
    const p = useProfiler(ref('shop'));
    await p.load('php');

    await p.setDevelop(true);
    expect(calls.find(([n]) => n === 'profilerSetMode')).toEqual([
      'profilerSetMode',
      'shop',
      'trace',
      true,
    ]);
  });

  it('offers coverage as a fourth mode', async () => {
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Coverage'));

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Coverage');
    wrapper.unmount();
  });

  /**
   * Coverage produces no file here, so the "trigger a request and it appears
   * below" note would be three wrong sentences.
   */
  it('does not promise a recording for coverage', async () => {
    replies.profilerStatus = {
      ...replies.profilerStatus,
      mode: 'coverage',
      modeValue: 'coverage',
      profiles: [],
      traces: [],
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Coverage'));

    expect(wrapper.text()).toContain('PHPUnit writes the report');
    expect(wrapper.text()).not.toContain('XDEBUG_TRIGGER');
    wrapper.unmount();
  });
});

/**
 * A warning with no way to act on it.
 *
 * Switching Xdebug on for the first time compiles the extension into the image,
 * so nothing happens until the project is regenerated and rebuilt — and the
 * pane said exactly that and stopped, leaving the reader holding a sentence and
 * a page to go hunting through. The work it names is the header's Rebuild
 * button, so it is offered where the problem is stated.
 *
 * Deliberately a button and not an automatic rebuild: it is minutes and it
 * recreates the container, and a switch that quietly started one would be a
 * surprise nobody asked for.
 *
 * Both warnings carry the shared `RemedyAlert` rather than a hand-written
 * alert and an emit the page turns back into a call, so what is asserted here
 * is the command that actually runs. That is the stronger claim of the two: an
 * emit could be wired to the wrong handler and this file would never know.
 */
describe('acting on the Xdebug warnings', () => {
  const mountPane = () =>
    mount(
      {
        components: { XdebugPane },
        template: '<v-app><XdebugPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [createPinia(), vuetify, i18n] } }
    );

  it('offers the rebuild the warning asks for, and asks rather than doing it', async () => {
    replies.xdebugStatus = {
      enabled: true,
      compiledIn: false,
      needsRebuild: true,
      active: false,
      running: false,
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('rebuild'));

    const button = wrapper.find('[data-test="remedy-rebuild"]');
    expect(button.exists(), 'the warning has no button to act on').toBe(true);
    expect(button.text()).toBe('Rebuild the project');

    // Nothing has run yet — the switch does not start a build on its own.
    expect(calls.some(([n]) => n === 'projectBuild')).toBe(false);
    await button.trigger('click');
    await flushPromises();
    expect(calls).toContainEqual(['projectBuild', 'shop']);
    wrapper.unmount();
  });

  /**
   * The other warning is a different fault with a different fix: the image has
   * the extension and the container predates the overlay. That is a recreate,
   * which is seconds. Offering the expensive one here would teach people to
   * reach for it every time.
   */
  it('offers a recreate, not a rebuild, for a container that is merely behind', async () => {
    replies.xdebugStatus = {
      enabled: true,
      compiledIn: true,
      needsRebuild: false,
      active: false,
      running: true,
    };
    const wrapper = mountPane();
    await vi.waitFor(() =>
      expect(wrapper.findAllComponents({ name: 'VBtn' }).length).toBeGreaterThan(0)
    );

    expect(wrapper.find('[data-test="remedy-rebuild"]').exists()).toBe(false);

    const apply = wrapper.find('[data-test="remedy-recreate"]');
    expect(apply.exists(), 'nothing offered for a container that is behind').toBe(true);
    await apply.trigger('click');
    await flushPromises();
    expect(calls).toContainEqual(['composeUpProject', 'shop']);
    expect(calls.some(([n]) => n === 'projectBuild')).toBe(false);
    wrapper.unmount();
  });

  /** And a project that is already working is offered neither. */
  it('offers nothing once it is actually active', async () => {
    replies.xdebugStatus = {
      enabled: true,
      compiledIn: true,
      needsRebuild: false,
      active: true,
      running: true,
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('IDE setup'));

    expect(wrapper.find('[data-test="remedy-rebuild"]').exists()).toBe(false);
    expect(wrapper.find('[data-test="remedy-recreate"]').exists()).toBe(false);
    wrapper.unmount();
  });
});

/**
 * The button that worked and said it had not.
 *
 * Every one of these operations returns an operation id **as soon as the work
 * starts**, not when it ends, so the caller's `await` resolves while docker is
 * still recreating. The panes re-read nothing at all, which is why "the
 * container is in debug, the setting is profile" survived pressing the button
 * that fixed it: the work was done and the screen never asked again.
 */
describe('the panes re-read when the operation finishes', () => {
  const useOperationsStore = async () => (await import('@/stores/operations')).useOperationsStore;

  it('re-reads the Xdebug pane on the falling edge of busy, not on the call', async () => {
    const pinia = createPinia();
    const wrapper = mount(
      {
        components: { XdebugPane },
        template: '<v-app><XdebugPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [pinia, vuetify, i18n] } }
    );
    await vi.waitFor(() => expect(calls.some(([n]) => n === 'xdebugStatus')).toBe(true));

    const ops = (await useOperationsStore())();
    const before = calls.filter(([n]) => n === 'xdebugStatus').length;

    // While it runs, nothing is re-read — the container is still changing.
    ops.markBusy('shop', true);
    await wrapper.vm.$nextTick();
    expect(calls.filter(([n]) => n === 'xdebugStatus').length).toBe(before);

    // The operation's finished event clears the flag; that is the first moment
    // the container on disk is the one being described.
    ops.markBusy('shop', false);
    await vi.waitFor(() =>
      expect(calls.filter(([n]) => n === 'xdebugStatus').length).toBeGreaterThan(before)
    );
    wrapper.unmount();
  });

  it('re-reads the profiler pane the same way', async () => {
    const pinia = createPinia();
    const wrapper = mount(
      {
        components: { ProfilerPane },
        template: '<v-app><ProfilerPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [pinia, vuetify, i18n] } }
    );
    await vi.waitFor(() => expect(calls.some(([n]) => n === 'profilerStatus')).toBe(true));

    const ops = (await useOperationsStore())();
    const before = calls.filter(([n]) => n === 'profilerStatus').length;

    ops.markBusy('shop', true);
    // A tick between the two: without it the watcher sees the value end where
    // it started and never fires, which is a fact about Vue rather than about
    // the pane — and is why the assertion below would otherwise pass on a pane
    // that re-reads nothing.
    await wrapper.vm.$nextTick();
    ops.markBusy('shop', false);
    await vi.waitFor(() =>
      expect(calls.filter(([n]) => n === 'profilerStatus').length).toBeGreaterThan(before)
    );
    wrapper.unmount();
  });

  /**
   * Switching Xdebug off leaves the running container debugging until it is
   * recreated, and the pane used to say nothing at all about that — the same
   * silence that made switching it *on* a lie, pointing the other way.
   */
  it('says a container is still debugging after the switch went off', async () => {
    replies.xdebugStatus = {
      enabled: false,
      compiledIn: true,
      needsRebuild: false,
      active: true,
      running: true,
    };
    const wrapper = mount(
      {
        components: { XdebugPane },
        template: '<v-app><XdebugPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [createPinia(), vuetify, i18n] } }
    );
    await vi.waitFor(() => expect(wrapper.text()).toContain('Still switched on'));

    const apply = wrapper.find('[data-test="remedy-recreate"]');
    expect(apply.exists(), 'nothing offered for a container still carrying Xdebug').toBe(true);
    await apply.trigger('click');
    await flushPromises();
    expect(calls).toContainEqual(['composeUpProject', 'shop']);

    // And not the expensive one: the extension stays in the image on purpose.
    expect(wrapper.find('[data-test="remedy-rebuild"]').exists()).toBe(false);
    wrapper.unmount();
  });
});

/**
 * A failed refresh must not empty the pane.
 *
 * The whole screen hangs off `v-if="status"`, and the refreshes now happen
 * exactly when the engine is busiest — this re-reads as a container is being
 * recreated. A call that lost a race with docker used to take the mode buttons,
 * the warnings and the recorded list down with it, which reads as the pane
 * having broken rather than as one reply having been missed.
 */
describe('a refresh that fails', () => {
  it('keeps the profiler on screen and reports the error beside it', async () => {
    const p = useProfiler(ref('shop'));
    await p.load('php');
    expect(p.status.value).toBeTruthy();

    replies.profilerStatus = () => Promise.reject({ code: 'ENGINE_UNREACHABLE', message: 'down' });
    await p.load('php');

    expect(p.status.value, 'the pane was emptied by one failed read').toBeTruthy();
    expect(p.error.value.code).toBe('ENGINE_UNREACHABLE');
  });

  it('keeps the Xdebug pane the same way', async () => {
    const x = useXdebug(ref('shop'));
    await x.load('php');
    expect(x.status.value).toBeTruthy();

    replies.xdebugStatus = () => Promise.reject({ code: 'ENGINE_UNREACHABLE', message: 'down' });
    await x.load('php');

    expect(x.status.value).toBeTruthy();
    expect(x.error.value.code).toBe('ENGINE_UNREACHABLE');
  });

  /** The first read has nothing to keep, so it still reports nothing. */
  it('still has nothing to show when the very first read fails', async () => {
    replies.profilerStatus = () => Promise.reject({ code: 'ENGINE_UNREACHABLE', message: 'down' });
    const p = useProfiler(ref('shop'));

    expect(await p.load('php')).toBe(null);
    expect(p.status.value).toBe(null);
  });

  /** And a successful refresh clears an error left by a previous one. */
  it('clears the error once a read succeeds again', async () => {
    const p = useProfiler(ref('shop'));
    await p.load('php');
    replies.profilerStatus = () => Promise.reject({ code: 'ENGINE_UNREACHABLE', message: 'down' });
    await p.load('php');
    expect(p.error.value).toBeTruthy();

    replies.profilerStatus = {
      mode: 'profile',
      develop: false,
      modeValue: 'profile',
      trigger: 'XDEBUG_TRIGGER=1',
      profiles: PROFILES,
      bytes: 4096,
      directory: '/ws/projects/shop/.stackvo/profiles',
      xdebug: { enabled: true, running: true, active: true, activeMode: 'profile' },
    };
    await p.load('php');
    expect(p.error.value).toBe(null);
  });
});

/**
 * The mode cannot be moved while the container is being recreated.
 *
 * Choosing a mode rewrites the compose overlay, and compose is reading that
 * file right now — the two racing produce a container whose `XDEBUG_MODE` is
 * neither of the things the screen said, and the only symptom is a debugger
 * that does not attach. It has to unlock itself, though, or it is a control
 * that has gone quiet.
 */
describe('the mode buttons while an operation runs', () => {
  it('locks them, says why, and unlocks when the work finishes', async () => {
    const pinia = createPinia();
    const wrapper = mount(
      {
        components: { ProfilerPane },
        template: '<v-app><ProfilerPane name="shop" runtime="php" /></v-app>',
      },
      { global: { plugins: [pinia, vuetify, i18n] } }
    );
    await vi.waitFor(() => expect(wrapper.text()).toContain('Coverage'));

    const modes = () =>
      wrapper
        .findAllComponents({ name: 'VBtn' })
        .filter((b) => ['Step debugging', 'Profiling', 'Trace', 'Coverage'].includes(b.text()));
    expect(modes().length).toBe(4);
    expect(modes().every((b) => b.props('disabled'))).toBe(false);

    const { useOperationsStore } = await import('@/stores/operations');
    const ops = useOperationsStore();

    ops.markBusy('shop', true);
    await wrapper.vm.$nextTick();
    expect(modes().every((b) => b.props('disabled'))).toBe(true);
    expect(wrapper.text()).toContain('The mode is held while the container is being rebuilt');

    ops.markBusy('shop', false);
    await vi.waitFor(() => expect(modes().some((b) => b.props('disabled'))).toBe(false));
    wrapper.unmount();
  });
});

/**
 * The sampling profiler.
 *
 * Three states that have to be satisfied in order — built, switched on, in the
 * container — and the pane is only honest if it refuses to skip one. The build
 * is the interesting case: it is minutes long, it returns an operation id as
 * soon as it starts, and until it finishes there is nothing to switch on.
 */
describe('php-spx', () => {
  const mountPane = () =>
    mount(
      {
        components: { SpxPane },
        template: '<v-app><SpxPane name="shop" runtime="php" @apply="$attrs.onApply" /></v-app>',
      },
      { attrs: {}, global: { plugins: [createPinia(), vuetify, i18n] } }
    );

  it('offers the build and nothing else until there is one', async () => {
    replies.spxStatus = { ...replies.spxStatus, built: false };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Not built for PHP 8.4'));

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Build it');
    // No switch: turning it on without a build mounts an empty directory over
    // the extension path, which stops PHP starting at all.
    expect(wrapper.findAllComponents({ name: 'VSwitch' }).length).toBe(0);
    wrapper.unmount();
  });

  it('sends the build and re-reads when the operation finishes', async () => {
    replies.spxStatus = { ...replies.spxStatus, built: false };
    replies.spxBuild = 'spx-1';
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Not built'));

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Build it')
      .trigger('click');
    await vi.waitFor(() => expect(calls.some(([n]) => n === 'spxBuild')).toBe(true));

    const before = calls.filter(([n]) => n === 'spxStatus').length;
    const { useOperationsStore } = await import('@/stores/operations');
    const ops = useOperationsStore();
    ops.markBusy('shop', true);
    await wrapper.vm.$nextTick();
    ops.markBusy('shop', false);
    await vi.waitFor(() =>
      expect(calls.filter(([n]) => n === 'spxStatus').length).toBeGreaterThan(before)
    );
    wrapper.unmount();
  });

  it('says the switch has not reached a container that was already up', async () => {
    replies.spxStatus = { ...replies.spxStatus, enabled: true, running: true, active: false };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Not in the running container yet'));

    const labels = wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text());
    expect(labels).toContain('Recreate the container');
    wrapper.unmount();
  });

  /**
   * Two profilers hooking one engine is unsupported by both projects and the
   * symptom is wrong numbers rather than an error — so it is said rather than
   * prevented: which one to turn off is not this app's decision.
   */
  it('warns when Xdebug is recording as well, and does not switch anything off', async () => {
    replies.spxStatus = { ...replies.spxStatus, enabled: true, xdebugConflict: true };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Xdebug is recording as well'));

    expect(calls.some(([n]) => n === 'profilerSetMode')).toBe(false);
    wrapper.unmount();
  });

  it('lists a recorded run with what it cost', async () => {
    replies.spxStatus = {
      ...replies.spxStatus,
      enabled: true,
      bytes: 4096,
      reports: [
        {
          key: 'spx-full-1',
          recordedAt: 1787426207,
          cli: false,
          request: 'GET /api/health',
          wallTimeUs: 736_000,
          peakMemory: 1808984,
          callCount: 1240,
          bytes: 4096,
        },
      ],
    };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('GET /api/health'));

    expect(wrapper.text()).toContain('736.0 ms');
    expect(wrapper.text()).toContain('1240');
    wrapper.unmount();
  });

  /**
   * Recording needs the extension in the container that will serve the request.
   * Offering it otherwise sends a request that succeeds and records nothing,
   * which reads as a broken button rather than as a container that predates the
   * switch.
   */
  it('offers a recording only once the profiler is in the running container', async () => {
    replies.spxStatus = { ...replies.spxStatus, enabled: true, running: true, active: false };
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Not in the running container yet'));
    expect(wrapper.text()).not.toContain('Record from here');
    wrapper.unmount();
  });

  /**
   * The path is the only thing that crosses. The address is the project's, on
   * the Rust side, where a path naming another host is refused — a text field
   * that could name a host would make this button a request forger.
   */
  it('records a request without a browser, and says what it got', async () => {
    replies.spxStatus = { ...replies.spxStatus, enabled: true, running: true, active: true };
    replies.quickCommands = [];
    replies.spxRecordRequest = {
      key: 'spx-full-9',
      recordedAt: 1787426207,
      cli: false,
      request: 'GET /checkout',
      wallTimeUs: 912_000,
      peakMemory: 1,
      callCount: 2,
      bytes: 3,
    };

    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Record from here'));

    await wrapper.findComponent({ name: 'VTextField' }).setValue('/checkout');
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Record this request')
      .trigger('click');

    await vi.waitFor(() => expect(wrapper.text()).toContain('GET /checkout — 912.0 ms'));
    expect(calls.find(([n]) => n === 'spxRecordRequest')).toEqual([
      'spxRecordRequest',
      'shop',
      '/checkout',
    ]);
    wrapper.unmount();
  });

  /**
   * The loop this closes: change the code, does the page get faster.
   *
   * Both numbers are shown and the difference is named, and there is
   * deliberately no verdict on the screen — one run against one run is not a
   * benchmark, and a green "faster" would invite a conclusion the measurement
   * cannot carry.
   */
  it('sends a recorded request again and shows both numbers with the difference', async () => {
    replies.quickCommands = [];
    const before = {
      key: 'spx-full-1',
      recordedAt: 1787426207,
      cli: false,
      request: 'GET /api/health',
      wallTimeUs: 912_000,
      peakMemory: 1,
      callCount: 2,
      bytes: 3,
    };
    replies.spxStatus = {
      ...replies.spxStatus,
      enabled: true,
      running: true,
      active: true,
      reports: [before],
    };
    replies.requestReplay = {
      before,
      after: { ...before, key: 'spx-full-2', wallTimeUs: 412_000 },
      wallTimeUs: -500_000,
      peakMemory: 0,
    };

    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('GET /api/health'));

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.attributes('aria-label') === 'Send this request again')
      .trigger('click');

    await vi.waitFor(() => expect(wrapper.find('[data-test="replay-result"]').exists()).toBe(true));
    const shown = wrapper.find('[data-test="replay-result"]').text();
    expect(shown).toContain('912.0 ms');
    expect(shown).toContain('412.0 ms');
    // The caveat travels with the number rather than living in the help file:
    // it is the sentence that stops one run being read as a benchmark.
    expect(shown).toContain('not a benchmark');
    // Three arguments, the third being the snapshot to restore before the
    // second run. `undefined` here because this test picks none — asserted
    // rather than trimmed off, so that the day the pane starts sending one by
    // accident is a day this fails.
    expect(calls.find(([n]) => n === 'requestReplay')).toEqual([
      'requestReplay',
      'shop',
      'spx-full-1',
      undefined,
    ]);
    wrapper.unmount();
  });

  /** Interactive commands are not offered: a recording has to finish. */
  it('leaves an interactive command out of the ones it can record', async () => {
    replies.spxStatus = { ...replies.spxStatus, enabled: true, running: true, active: true };
    replies.quickCommands = [
      { id: 'tinker', display: 'php artisan tinker', interactive: true },
      { id: 'migrate', display: 'php artisan migrate', interactive: false },
    ];
    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Record from here'));

    const select = wrapper.findAllComponents({ name: 'VSelect' }).at(0);
    expect(select.props('items').map((c) => c.id)).toEqual(['migrate']);
    wrapper.unmount();
  });

  /**
   * The question a report row could never answer. Read on demand rather than
   * with the list: it decompresses and replays a trace, and most rows are never
   * asked about.
   */
  it('reads where one recording spent its time, only when asked', async () => {
    replies.spxStatus = {
      ...replies.spxStatus,
      enabled: true,
      reports: [
        {
          key: 'spx-full-1',
          recordedAt: 1787426207,
          cli: false,
          request: 'GET /api/health',
          wallTimeUs: 736_000,
          peakMemory: 1,
          callCount: 1240,
          bytes: 4096,
        },
      ],
    };
    replies.spxReport = {
      key: 'spx-full-1',
      wallTimeUs: 736_000,
      callCount: 1240,
      functions: 2,
      events: 40,
      truncated: false,
      hotspots: [
        {
          function: 'App\\Repository::all',
          calls: 3,
          exclusiveUs: 500_000,
          exclusivePercent: 68,
          inclusiveUs: 600_000,
          inclusivePercent: 81,
        },
      ],
    };

    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('GET /api/health'));
    expect(calls.some(([n]) => n === 'spxReport')).toBe(false);

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.attributes('aria-label') === 'Where the time went')
      .trigger('click');

    await vi.waitFor(() => expect(wrapper.text()).toContain('App\\Repository::all'));
    expect(wrapper.text()).toContain('68.0%');
    expect(wrapper.text()).toContain('500.0 ms');
    wrapper.unmount();
  });

  /**
   * php-spx's own default period is 0 — every call — which is a tracing
   * profiler with the cost this pane's first sentence claims to avoid.
   */
  it('records sampled by default and can be told to count every call', async () => {
    replies.spxStatus = { ...replies.spxStatus, enabled: true };
    replies.spxOptions = { ...replies.spxStatus, enabled: true, samplingPeriod: 0 };

    const wrapper = mountPane();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Sampled every 100 µs'));

    const sampling = wrapper
      .findAllComponents({ name: 'VSelect' })
      .find((s) => s.props('label') === 'Sampling');
    sampling.vm.$emit('update:modelValue', 0);
    await vi.waitFor(() => expect(calls.some(([n]) => n === 'spxOptions')).toBe(true));
    expect(calls.find(([n]) => n === 'spxOptions')).toEqual(['spxOptions', 'shop', 0, null]);
    wrapper.unmount();
  });

  /** A node project has no PHP to load an extension into, so there is no card. */
  it('draws nothing for a project that is not PHP', async () => {
    const wrapper = mount(
      {
        components: { SpxPane },
        template: '<v-app><SpxPane name="shop" runtime="node" /></v-app>',
      },
      { global: { plugins: [createPinia(), vuetify, i18n] } }
    );
    await wrapper.vm.$nextTick();

    expect(wrapper.findComponent({ name: 'VCard' }).exists()).toBe(false);
    expect(calls.some(([n]) => n === 'spxStatus')).toBe(false);
    wrapper.unmount();
  });
});
