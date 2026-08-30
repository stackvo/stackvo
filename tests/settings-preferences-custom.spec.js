import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The `Other…` row, mounted — the half of it a person actually touches.
 *
 * `apps.rs` had four closed lists and nothing beside them. Detection is better
 * than the free-text box it replaced, and the free-text box was removed without
 * putting anything in its place, so somebody running Helix or Emacs opened this
 * pane and found no editor they could pick. Not a worse choice: no choice.
 *
 * Two things are worth pinning here and neither is the select itself.
 *
 * **The box appears for the choice and not before.** Four command lines on a
 * pane that is mostly about which app to open would be four fields nobody asked
 * for, so each is bound to its own picker being on `custom`.
 *
 * **The choice and the command are separate preferences.** `editorCommand` says
 * *which*, `editorCustom` says *what to run*, and typing in the box must not
 * silently rewrite the choice above it — the back end reads both, and a version
 * of this that stored one string would make "go back to VS Code" mean retyping
 * the custom command from memory.
 */

globalThis.visualViewport = undefined;

const APPS = {
  terminals: [
    { id: 'terminal', name: 'Terminal', icon: 'mdi-apple', available: true, default: true },
    { id: 'custom', name: 'Other…', icon: 'mdi-pencil-outline', available: true, default: false },
  ],
  editors: [
    { id: 'code', name: 'VS Code', icon: 'mdi-file-code', available: true, default: true },
    { id: 'custom', name: 'Other…', icon: 'mdi-pencil-outline', available: true, default: false },
  ],
  browsers: [
    { id: '', name: 'System default', icon: 'mdi-open-in-app', available: true, default: true },
    { id: 'custom', name: 'Other…', icon: 'mdi-pencil-outline', available: true, default: false },
  ],
};

let stored = {};
const patches = [];

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: {
    appsAvailable: () => Promise.resolve(APPS),
    prefsGet: () => Promise.resolve(stored),
    prefsSet: (patch) => {
      patches.push(patch);
      stored = { ...stored, ...patch };
      return Promise.resolve(stored);
    },
    autostartIsEnabled: () => Promise.resolve(false),
    autostartEnable: () => Promise.resolve(true),
    autostartDisable: () => Promise.resolve(false),
  },
}));

const { i18n } = await import('@/i18n');
const PreferencesPane = (await import('@/components/settings/PreferencesPane.vue')).default;
const { usePreferences } = await import('@/composables/usePreferences');

const vuetify = createVuetify({ components, directives });

async function pane(prefs = {}) {
  stored = { schemaVersion: 4, ...prefs };
  patches.length = 0;
  usePreferences().reset();
  const wrapper = mount(PreferencesPane, { global: { plugins: [vuetify, i18n] } });
  // Two awaited hops: `load()` resolves, then the pickers re-render with it.
  await Promise.resolve();
  await Promise.resolve();
  await wrapper.vm.$nextTick();
  return wrapper;
}

/** The four labels this file is about, in the order the pane lays them out. */
const CUSTOM_LABELS = [
  'settings.appCustomTerminal',
  'settings.appCustomEditor',
  'settings.appCustomBrowser',
  'settings.appCustomDbClient',
].map((key) => i18n.global.t(key));

/**
 * The command boxes on screen, by their label.
 *
 * Read off the components rather than off `label` elements — Vuetify renders
 * two of those per field, the floating one and the one inside the outline, so
 * a DOM scan reports every box twice. And filtered to the four above, because
 * `VSelect` is a `VTextField` underneath: an unfiltered list also carries the
 * three pickers and both snapshot fields.
 */
const commandLabels = (wrapper) =>
  wrapper
    .findAllComponents({ name: 'VTextField' })
    .map((field) => field.props('label'))
    .filter((label) => CUSTOM_LABELS.includes(label));

/** One box, by the label it carries. */
const commandBox = (wrapper, key) =>
  wrapper
    .findAllComponents({ name: 'VTextField' })
    .find((field) => field.props('label') === i18n.global.t(key));

beforeEach(() => {
  patches.length = 0;
});

describe('the custom command boxes', () => {
  it('shows no launcher box until a picker is on Other…', async () => {
    const wrapper = await pane();
    // The database one is always there — its picker lives on a service sheet,
    // per scheme, so this pane is the only place its command can be typed.
    expect(commandLabels(wrapper)).toEqual([i18n.global.t('settings.appCustomDbClient')]);
  });

  it('shows the editor box, and only that one, when the editor is Other…', async () => {
    const wrapper = await pane({ editorCommand: 'custom' });
    expect(commandLabels(wrapper)).toEqual([
      i18n.global.t('settings.appCustomEditor'),
      i18n.global.t('settings.appCustomDbClient'),
    ]);
  });

  it('shows one box per picker that is on Other…', async () => {
    const wrapper = await pane({
      terminalApp: 'custom',
      editorCommand: 'custom',
      browserCommand: 'custom',
    });
    expect(commandLabels(wrapper)).toEqual([
      i18n.global.t('settings.appCustomTerminal'),
      i18n.global.t('settings.appCustomEditor'),
      i18n.global.t('settings.appCustomBrowser'),
      i18n.global.t('settings.appCustomDbClient'),
    ]);
  });

  /**
   * The two keys are read by different code — `editorCommand` picks the branch,
   * `editorCustom` is what that branch runs — so writing one must never touch
   * the other. Asserted on the patch rather than on the merged document,
   * because a patch carrying both would overwrite a choice made a second ago.
   */
  it('stores the command under its own key, leaving the choice alone', async () => {
    const wrapper = await pane({ editorCommand: 'custom' });
    const box = commandBox(wrapper, 'settings.appCustomEditor');

    await box.setValue('hx --vsplit');

    expect(patches).toEqual([{ editorCustom: 'hx --vsplit' }]);
  });

  /** Clearing the box is "I have not typed one", which is `null` and not `''`. */
  it('clears to null rather than to an empty string', async () => {
    const wrapper = await pane({ editorCommand: 'custom', editorCustom: 'hx' });
    const box = commandBox(wrapper, 'settings.appCustomEditor');

    await box.setValue('');

    expect(patches).toEqual([{ editorCustom: null }]);
  });
});
