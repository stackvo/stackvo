import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createRouter, createMemoryHistory } from 'vue-router';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The beta switch on the About card, mounted.
 *
 * What it must do is small and every part of it has a way of being wrong that
 * looks fine on screen: the switch has to show the stored preference rather
 * than a default, save under the key `channel.rs` reads at launch, and hand
 * the channel to the check — because the check is what refuses a beta
 * manifest to an install that wants stable, and a check that always said
 * `stable` would make the switch a decoration.
 */

globalThis.visualViewport = undefined;

let stored = {};
const patches = [];
const checkForUpdate = vi.fn(async () => null);

const replies = {
  envGet: { DEFAULT_TLD_SUFFIX: 'stackvo.loc' },
  envDefaults: {},
  catalogGet: { runtimes: [], servers: [] },
  hostsOverview: { entries: [], stale: [] },
  templatesList: [],
  appsAvailable: { terminals: [], editors: [], browsers: [] },
  logsInfo: { directory: '/logs', newestFile: null, totalBytes: 0 },
  certStatus: { sslEnabled: false },
  presetExport: { services: {} },
};

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: new Proxy(
    {},
    {
      get: (_t, name) => {
        if (name === 'prefsGet') return () => Promise.resolve(stored);
        if (name === 'prefsSet') {
          return (patch) => {
            patches.push(patch);
            stored = { ...stored, ...patch };
            return Promise.resolve(stored);
          };
        }
        return () => Promise.resolve(replies[name]);
      },
    }
  ),
}));

vi.mock('@/lib/events', async (importOriginal) => ({
  ...(await importOriginal()),
  listenAll: async () => () => {},
  listen: async () => () => {},
}));

vi.mock('@tauri-apps/api/app', () => ({ getVersion: async () => '0.2.0' }));
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn(), openPath: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ save: vi.fn(), open: vi.fn() }));
vi.mock('@tauri-apps/plugin-autostart', () => ({
  isEnabled: async () => false,
  enable: vi.fn(),
  disable: vi.fn(),
}));
vi.mock('@/lib/updates', async (importOriginal) => ({
  ...(await importOriginal()),
  checkForUpdate: (...args) => checkForUpdate(...args),
  updatesConfigured: async () => true,
}));

const { i18n } = await import('@/i18n');
const Settings = (await import('@/views/Settings.vue')).default;
const { usePreferences } = await import('@/composables/usePreferences');

const vuetify = createVuetify({ components, directives });

const settle = async (wrapper) => {
  await new Promise((resolve) => setTimeout(resolve, 0));
  await wrapper.vm.$nextTick();
};

async function renderAbout() {
  const host = document.createElement('div');
  document.body.appendChild(host);
  const pinia = createPinia();
  setActivePinia(pinia);

  const wrapper = mount(
    { components: { Settings }, template: '<v-app><Settings /></v-app>' },
    {
      attachTo: host,
      global: {
        plugins: [
          pinia,
          createRouter({
            history: createMemoryHistory(),
            routes: [{ path: '/:pathMatch(.*)*', component: { template: '<div />' } }],
          }),
          vuetify,
          i18n,
        ],
      },
    }
  );
  await settle(wrapper);
  wrapper.findComponent(Settings).vm.tab = 'about';
  await settle(wrapper);
  return wrapper;
}

/** The switch, found by what it says rather than by its position. */
const betaSwitch = (wrapper) => wrapper.findAll('.v-switch').find((s) => s.text().includes('beta'));

beforeEach(() => {
  stored = {};
  patches.length = 0;
  checkForUpdate.mockClear();
  usePreferences().reset();
});

describe('the beta switch', () => {
  it('shows the stored preference, and is off when there is none', async () => {
    const wrapper = await renderAbout();
    const off = betaSwitch(wrapper);
    expect(off, 'the About card offers the switch').toBeDefined();
    expect(off.find('input').element.checked).toBe(false);
    wrapper.unmount();

    stored = { updateChannel: 'beta' };
    usePreferences().reset();
    const again = await renderAbout();
    expect(betaSwitch(again).find('input').element.checked).toBe(true);
    again.unmount();
  });

  it('hands the stored channel to the check on mount', async () => {
    stored = { updateChannel: 'beta' };
    const wrapper = await renderAbout();
    expect(checkForUpdate).toHaveBeenCalledWith({ channel: 'beta' });
    wrapper.unmount();
  });

  it('saves under the key the launch reads, and checks again on the new channel', async () => {
    const wrapper = await renderAbout();
    checkForUpdate.mockClear();

    await betaSwitch(wrapper).find('input').setValue(true);
    await settle(wrapper);

    expect(patches).toContainEqual({ updateChannel: 'beta' });
    expect(checkForUpdate).toHaveBeenCalledWith({ channel: 'beta' });

    await betaSwitch(wrapper).find('input').setValue(false);
    await settle(wrapper);
    expect(patches).toContainEqual({ updateChannel: 'stable' });
    expect(checkForUpdate).toHaveBeenLastCalledWith({ channel: 'stable' });

    wrapper.unmount();
  });

  it('says that the endpoint changes at the next launch', async () => {
    // The updater plugin reads its endpoint list once, when the app starts;
    // a switch that implied otherwise would be believed.
    const wrapper = await renderAbout();
    expect(betaSwitch(wrapper).text()).toMatch(/next time StackVo starts/);
    wrapper.unmount();
  });
});
