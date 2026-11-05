import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { createPinia } from 'pinia';

/**
 * The Configuration pane's read-only summary of what a project *is*.
 *
 * One row on it is worth a test on its own. "Container path" was the string
 * `/var/www/html`, with no `v-if`, directly under two rows that had one — so
 * every project claimed the PHP web root, including the ones the generator
 * writes `WORKDIR /app` for. The copy button put that path on the clipboard,
 * and a `docker exec … cd` with it fails inside the project's own container.
 *
 * What these hold is the pairing rather than either value: the page has to say
 * what `render_dockerfile` did, and the two runtimes below are the two sides of
 * that dispatch. The Rust side owns the paths; this owns the agreement.
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

beforeEach(() => {
  for (const key of Object.keys(replies)) delete replies[key];
});

const { i18n } = await import('@/i18n');
const OverviewPane = (await import('@/components/project/OverviewPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const project = (runtime) => ({
  name: 'shop',
  domain: 'shop.stackvo.loc',
  runtime,
  manifest: { name: 'shop', runtime },
});

const mountPane = (runtime) =>
  mount(
    {
      components: { OverviewPane },
      props: ['project'],
      template: '<v-app><OverviewPane :project="project" /></v-app>',
    },
    // Pinia because the verify list's long repairs wait on the operations
    // store's busy flag, the one RemedyAlert and the page share.
    { props: { project: project(runtime) }, global: { plugins: [createPinia(), vuetify, i18n] } }
  );

describe('the container path a project page offers', () => {
  it('is the web root for a PHP project', () => {
    const text = mountPane('php').text();
    expect(text).toContain('/var/www/html');
    expect(text).not.toContain('/app');
  });

  // node and the six language runtimes all reach `WORKDIR /app`, by two
  // different generator functions. Both are listed so a change to either one
  // shows up here.
  it.each(['node', 'python', 'go', 'ruby', 'rust', 'bun', 'deno'])(
    'is /app for a %s project, which has no /var/www/html to cd into',
    (runtime) => {
      const text = mountPane(runtime).text();
      expect(text).toContain('/app');
      expect(text).not.toContain('/var/www/html');
    }
  );

  // A project loaded before its runtime is known must not guess `/app`: the
  // manifest contract defaults a missing `runtime` to `php`, and the page has
  // to default the same way or it disagrees with the file on disk.
  it('falls back to the web root when the runtime is not loaded yet', () => {
    expect(mountPane(undefined).text()).toContain('/var/www/html');
  });
});

/**
 * The other half of onboarding.
 *
 * The comparison is settled in Rust, where `verify.rs` is a pure function with
 * the cases that have teeth in them. What only exists here are the two
 * decisions the screen makes: every line is drawn and not only the failing
 * ones, and a line that failed says what to do about it — an id with no
 * sentence beside it is a check nobody can act on.
 */
describe('checking the machine against the declaration', () => {
  const press = async (wrapper) => {
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Check my setup')
      .trigger('click');
    await flushPromises();
  };

  it('draws every line, not only the ones that failed', async () => {
    replies.projectVerify = {
      project: 'shop',
      ready: false,
      checks: [
        { id: 'manifest', subject: 'shop', state: 'ok' },
        { id: 'service', subject: 'mysql', state: 'missing' },
        { id: 'serviceOff', subject: 'redis', state: 'different', detail: '7.2' },
      ],
    };

    const wrapper = mountPane('php');
    await press(wrapper);

    expect(wrapper.findAll('[data-test="verify-check"]')).toHaveLength(3);
    // The failing line says what to do; the passing one does not need to.
    expect(wrapper.text()).toContain('Install it from the Market');
    expect(wrapper.text()).toContain('Turn it on in Services');
    // The versions that ARE there travel with the row, because "install redis"
    // would be the wrong instruction.
    expect(wrapper.text()).toContain('7.2');
    expect(wrapper.text()).toContain('missing or switched off');
  });

  it('says everything matched rather than showing an empty result', async () => {
    replies.projectVerify = {
      project: 'shop',
      ready: true,
      checks: [{ id: 'manifest', subject: 'shop', state: 'ok' }],
    };

    const wrapper = mountPane('php');
    await press(wrapper);

    expect(wrapper.text()).toContain('Everything this project declares is here');
  });

  it('reports why it could not check instead of showing nothing', async () => {
    replies.projectVerify = () => {
      throw Object.assign(new Error('no workspace'), { message: 'no workspace' });
    };

    const wrapper = mountPane('php');
    await press(wrapper);

    expect(wrapper.text()).toContain('no workspace');
    expect(wrapper.findAll('[data-test="verify-check"]')).toHaveLength(0);
  });
});

/**
 * A line that failed offers the one act that answers it.
 *
 * The acts are the app's own commands — a build, a regenerate, the doctor's
 * extension removal, the manifest's PHP line — so what is held here is the
 * pairing: which line gets which button, with which argument, and that a line
 * that passed, or one nobody can act on from here, gets none.
 */
describe('repairing a line from the report', () => {
  const report = (checks, ready = false) => ({ project: 'shop', ready, checks });
  const press = async (wrapper) => {
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Check my setup')
      .trigger('click');
    await flushPromises();
  };
  const repairButton = (wrapper, id) => wrapper.find(`[data-test="verify-repair-${id}"]`);

  it('offers a repair only beside a line that failed and has one', async () => {
    replies.projectVerify = report([
      { id: 'manifest', subject: 'shop', state: 'ok' },
      { id: 'built', subject: 'shop', state: 'missing' },
      { id: 'service', subject: 'mysql', state: 'missing' },
      { id: 'extensionRemoved', subject: 'imap', state: 'different', detail: 'removed in PHP 8.2' },
    ]);

    const wrapper = mountPane('php');
    await press(wrapper);

    expect(repairButton(wrapper, 'built').text()).toContain('Build');
    expect(repairButton(wrapper, 'extensionRemoved').text()).toContain('Remove from stackvo.json');
    // The facts still travel with the row.
    expect(wrapper.text()).toContain('removed in PHP 8.2');
    // A missing service is installed from the Market, not from here; a line
    // that passed has nothing to repair.
    expect(repairButton(wrapper, 'service').exists()).toBe(false);
    expect(repairButton(wrapper, 'manifest').exists()).toBe(false);
  });

  it("removes an extension through the doctor's own command and asks again", async () => {
    const asked = vi.fn(() =>
      report([{ id: 'extensionRemoved', subject: 'imap', state: 'different' }])
    );
    replies.projectVerify = asked;
    const drop = vi.fn(() => ({ extensions: [] }));
    replies.doctorDropExtension = drop;

    const wrapper = mountPane('php');
    await press(wrapper);
    await repairButton(wrapper, 'extensionRemoved').trigger('click');
    await flushPromises();

    expect(drop).toHaveBeenCalledWith('shop', 'imap');
    // The doctor answers with its own report, so the line is re-asked.
    expect(asked).toHaveBeenCalledTimes(2);
  });

  it('writes the PHP version composer wants, with the target the report carried', async () => {
    replies.projectVerify = report([
      {
        id: 'phpVersion',
        subject: '^8.3',
        state: 'different',
        detail: 'composer.json wants 8.3, stackvo.json gives 8.2',
        target: '8.3',
      },
    ]);
    const set = vi.fn(() =>
      report([{ id: 'phpVersion', subject: '^8.3', state: 'ok', detail: '8.3' }], true)
    );
    replies.projectPhpSet = set;

    const wrapper = mountPane('php');
    await press(wrapper);
    expect(repairButton(wrapper, 'phpVersion').text()).toContain('8.3');

    await repairButton(wrapper, 'phpVersion').trigger('click');
    await flushPromises();

    expect(set).toHaveBeenCalledWith('shop', { version: '8.3' });
    // The write came back with the report re-run, and that is what is shown.
    expect(wrapper.text()).toContain('Everything this project declares is here');
  });

  it('adds the extension composer requires by its name', async () => {
    replies.projectVerify = report([{ id: 'phpExtension', subject: 'intl', state: 'missing' }]);
    const set = vi.fn(() => report([], true));
    replies.projectPhpSet = set;

    const wrapper = mountPane('php');
    await press(wrapper);
    await repairButton(wrapper, 'phpExtension').trigger('click');
    await flushPromises();

    expect(set).toHaveBeenCalledWith('shop', { addExtensions: ['intl'] });
  });

  it('has nothing to write when the constraint did not resolve to a version', async () => {
    replies.projectVerify = report([
      { id: 'phpVersion', subject: '^8', state: 'unknown', detail: 'stackvo.json gives 8.2' },
    ]);

    const wrapper = mountPane('php');
    await press(wrapper);

    expect(repairButton(wrapper, 'phpVersion').exists()).toBe(false);
  });

  it('starts a build from the row and holds the row until it finishes', async () => {
    replies.projectVerify = report([{ id: 'built', subject: 'shop', state: 'missing' }]);
    const build = vi.fn(() => 'op-1');
    replies.projectBuild = build;

    const wrapper = mountPane('php');
    await press(wrapper);
    await repairButton(wrapper, 'built').trigger('click');
    await flushPromises();

    expect(build).toHaveBeenCalledWith('shop');
    // Busy on the project — the flag RemedyAlert and the page share — until
    // the operation's own finish clears it.
    expect(repairButton(wrapper, 'built').attributes('disabled')).toBeDefined();
  });

  it('hands the dialogs it does not own up to the page', async () => {
    replies.projectVerify = report([
      { id: 'phpSupported', subject: '8.6', state: 'different', detail: '8.3, 8.4, 8.5' },
      { id: 'domain', subject: 'shop.loc', state: 'missing' },
    ]);

    const wrapper = mountPane('php');
    await press(wrapper);
    await repairButton(wrapper, 'phpSupported').trigger('click');
    await flushPromises();
    await repairButton(wrapper, 'domain').trigger('click');
    await flushPromises();

    const pane = wrapper.findComponent(OverviewPane);
    expect(pane.emitted('settings')).toHaveLength(1);
    expect(pane.emitted('fixHosts')).toHaveLength(1);
  });

  it('shows a refused repair against the list rather than swallowing it', async () => {
    replies.projectVerify = report([{ id: 'phpExtension', subject: 'intl', state: 'missing' }]);
    replies.projectPhpSet = () => {
      throw new Error('refusing to write a manifest carrying this machine’s overrides');
    };

    const wrapper = mountPane('php');
    await press(wrapper);
    await repairButton(wrapper, 'phpExtension').trigger('click');
    await flushPromises();

    expect(wrapper.find('[data-test="verify-repair-error"]').text()).toContain('refusing to write');
  });
});
