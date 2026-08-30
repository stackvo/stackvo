import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The PHP / runtime-defaults pane, mounted.
 *
 * Seventh out of `Settings.vue` in the pane split. The version choices come from the
 * catalog compiled into the binary rather than a list typed into the view, so a
 * release added there shows up without a second edit.
 *
 * The rule worth pinning is the one in `itemsFor`: **the value currently in
 * `.env` is always offered**, even when the catalog does not list it. There are
 * two ordinary ways to reach that state — a catalog call that failed, and a
 * value written by an older build — and in both a select whose only item is
 * missing renders blank, which reads as data loss rather than as "that version
 * is no longer shipped".
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
      get: (_t, name) => () => {
        const reply = replies[name];
        return typeof reply === 'function' ? reply() : Promise.resolve(reply);
      },
    }
  ),
}));

const { i18n } = await import('@/i18n');
const PhpPane = (await import('@/components/settings/PhpPane.vue')).default;
const { useEnvEditor, provideEnvEditor } = await import('@/composables/useEnvEditor');
const { useCatalog } = await import('@/composables/useCatalog');

const vuetify = createVuetify({ components, directives });

const CATALOG = {
  runtimes: [
    { id: 'php', versions: ['8.3', '8.4'] },
    { id: 'node', versions: ['20', '22'] },
    { id: 'python', versions: ['3.12'] },
  ],
  servers: ['nginx', 'caddy'],
};

let editor;

/**
 * Under a host that provides a *loaded* editor, which is what `Settings.vue`
 * does — a pane that loaded `.env` on its own mount would discard whatever the
 * other five had typed every time the user changed tab.
 */
async function render() {
  const host = document.createElement('div');
  document.body.appendChild(host);

  const wrapper = mount(
    {
      components: { PhpPane },
      setup() {
        editor = provideEnvEditor(useEnvEditor());
        return {};
      },
      template: '<PhpPane />',
    },
    { attachTo: host, global: { plugins: [vuetify, i18n] } }
  );

  await editor.loadDefaults();
  await editor.load();
  await new Promise((resolve) => setTimeout(resolve, 0));
  await wrapper.vm.$nextTick();
  return wrapper;
}

const pane = (wrapper) => wrapper.findComponent(PhpPane).vm;

beforeEach(() => {
  // The catalog is module-scoped so two panes share one fetch; a test must not
  // inherit the previous one's.
  useCatalog().reset();
  for (const key of Object.keys(replies)) delete replies[key];
  replies.envGet = { SUPPORTED_LANGUAGES_PHP_DEFAULT: '8.4', PHP_TOOL_NODEJS_VERSION: '22' };
  replies.envDefaults = {};
  replies.catalogGet = { ...CATALOG };
});

afterEach(() => {
  document.body.innerHTML = '';
});

describe('the version choices', () => {
  it('come from the catalog, not from a list in the view', async () => {
    const wrapper = await render();

    expect(pane(wrapper).phpVersions).toEqual(['8.3', '8.4']);
    expect(pane(wrapper).nodeVersions).toEqual(['20', '22']);
    expect(pane(wrapper).serverChoices).toEqual(['nginx', 'caddy']);

    wrapper.unmount();
  });

  /** A value written by an older build, or one the catalog stopped shipping. */
  it('always offer the value that is actually set', async () => {
    replies.envGet = { SUPPORTED_LANGUAGES_PHP_DEFAULT: '8.1' };

    const wrapper = await render();
    expect(pane(wrapper).phpVersions, 'the configured version was not among the choices').toEqual([
      '8.1',
      '8.3',
      '8.4',
    ]);

    wrapper.unmount();
  });

  /**
   * A catalog that cannot be read is a narrower list, not a blank select — and
   * a blank select over a configured value reads as data loss.
   */
  it('survive the catalog being unreadable', async () => {
    replies.catalogGet = () => Promise.reject(new Error('binary is odd'));

    const wrapper = await render();
    expect(pane(wrapper).phpVersions, 'the pane lost the value it had').toEqual(['8.4']);
    expect(wrapper.text().trim().length).toBeGreaterThan(0);

    wrapper.unmount();
  });

  it('offer nothing rather than undefined when neither knows a value', async () => {
    replies.envGet = {};
    replies.catalogGet = { runtimes: [], servers: [] };

    const wrapper = await render();
    expect(pane(wrapper).phpVersions).toEqual([]);
    expect(pane(wrapper).serverChoices).toEqual([]);

    wrapper.unmount();
  });

  /** Every runtime the app can build gets a default, from the same list. */
  it('cover every runtime the catalog knows', async () => {
    const wrapper = await render();
    expect(
      pane(wrapper).runtimeItems({ id: 'python', key: 'SUPPORTED_LANGUAGES_PYTHON_DEFAULT' })
    ).toEqual(['3.12']);
    wrapper.unmount();
  });
});

describe('editing', () => {
  it('writes through the shared editor rather than its own', async () => {
    const wrapper = await render();

    pane(wrapper).edit('SUPPORTED_LANGUAGES_PHP_DEFAULT', '8.3');
    expect(editor.edits.value.SUPPORTED_LANGUAGES_PHP_DEFAULT, 'the pane kept its own diff').toBe(
      '8.3'
    );
    expect(editor.dirty.value).toBe(true);

    wrapper.unmount();
  });

  it('asks the view to save, because the file is shared', async () => {
    const wrapper = await render();

    pane(wrapper).edit('SUPPORTED_LANGUAGES_PHP_DEFAULT', '8.3');
    await wrapper.vm.$nextTick();

    const save = wrapper
      .findAll('button')
      .find((b) => b.text().includes(i18n.global.t('settings.save', { count: 1 })));
    expect(save, 'no save button once something changed').toBeTruthy();

    await save.trigger('click');
    expect(wrapper.findComponent(PhpPane).emitted('save')).toBeTruthy();

    wrapper.unmount();
  });
});

/**
 * The lists behind the pickers, and the reason they became editable.
 *
 * `build_catalog` reads `SUPPORTED_LANGUAGES_{KEY}_VERSIONS` out of `.env` and
 * the pickers offer what came back, so those six lists decide what anybody can
 * select. They ship compiled into the binary and go out of date between
 * releases — Go stopped at 1.23, Ruby at 3.3, Node at 23, Rust at 1.84 — and
 * until this pane grew a control for them, a version the application had never
 * heard of could be reached only by editing `.env` by hand or by waiting for a
 * release of the application. For a number in a list.
 */
describe('which versions are offered', () => {
  /**
   * The lists sit behind a disclosure, and Vuetify does not mount an expansion
   * panel's content until it opens — so a test that asserted on them without
   * opening it would be asserting about nothing, and would go on passing after
   * the controls were deleted.
   */
  async function open(wrapper) {
    const title = wrapper
      .findAllComponents({ name: 'VExpansionPanelTitle' })
      .find((t) => t.text().includes(i18n.global.t('settings.runtimes.offered')));
    expect(title, 'no disclosure for the offered versions').toBeTruthy();
    await title.trigger('click');
    await wrapper.vm.$nextTick();
    await new Promise((resolve) => setTimeout(resolve, 0));
    await wrapper.vm.$nextTick();
  }

  it('renders one editor per list the binary reads', async () => {
    const wrapper = await render();
    await open(wrapper);

    const labels = wrapper.findAllComponents({ name: 'VCombobox' }).map((c) => c.props('label'));

    // The runtimes, plus the two PHP-image lists that were already here.
    for (const id of ['php', 'python', 'go', 'ruby', 'rust', 'nodejs']) {
      expect(labels, `no editor for ${id}`).toContain(id);
    }

    wrapper.unmount();
  });

  it('shows what .env holds, and writes a comma-separated list back', async () => {
    replies.envGet = {
      ...replies.envGet,
      SUPPORTED_LANGUAGES_GO_VERSIONS: '1.22,1.23',
    };
    const wrapper = await render();
    await open(wrapper);

    const go = wrapper
      .findAllComponents({ name: 'VCombobox' })
      .find((c) => c.props('label') === 'go');
    expect(go.props('modelValue')).toEqual(['1.22', '1.23']);

    // The shape `.env` wants: one key, one comma-separated value. A list
    // written back as an array would be `[object Object]` in a file the
    // generator reads.
    pane(wrapper).setList('SUPPORTED_LANGUAGES_GO_VERSIONS', ['1.22', '1.23', '1.24']);
    expect(editor.effective('SUPPORTED_LANGUAGES_GO_VERSIONS')).toBe('1.22,1.23,1.24');

    wrapper.unmount();
  });

  /**
   * The whole point: a version the binary's catalog has never heard of can be
   * added, because the catalog is what goes stale.
   */
  it('accepts a version the catalog does not know', async () => {
    const wrapper = await render();

    pane(wrapper).setList('SUPPORTED_LANGUAGES_PHP_VERSIONS', ['8.3', '8.4', '8.6']);
    expect(editor.effective('SUPPORTED_LANGUAGES_PHP_VERSIONS')).toBe('8.3,8.4,8.6');
    expect(editor.dirty.value, 'the change has to reach the diff to be saved').toBe(true);

    wrapper.unmount();
  });
});
