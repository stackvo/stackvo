import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The Localisation pane, mounted.
 *
 * Eighth and smallest out of `Settings.vue` in the pane split — three controls with
 * three different owners, which is exactly why it is worth mounting rather than
 * reading. The app locale goes through `setLocale`, because changing it also
 * persists the preference and relabels the tray; the console locale and the RTL
 * flag are appearance state and go straight to the store. Wiring any of the
 * three to the wrong owner produces a control that appears to work and forgets
 * itself on the next launch.
 */

globalThis.visualViewport = undefined;

const setLocale = vi.fn();

/** What the backend answers, per test. */
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
          return Promise.resolve(typeof reply === 'function' ? reply(...args) : (reply ?? null));
        },
    }
  ),
}));

vi.mock('@/i18n', async (importOriginal) => ({
  ...(await importOriginal()),
  setLocale: (...args) => setLocale(...args),
}));

const { i18n } = await import('@/i18n');
const LocalisationPane = (await import('@/components/settings/LocalisationPane.vue')).default;
const { useAppearanceStore } = await import('@/stores/appearance');

const vuetify = createVuetify({ components, directives });

async function render() {
  const host = document.createElement('div');
  document.body.appendChild(host);

  const pinia = createPinia();
  setActivePinia(pinia);

  const wrapper = mount(LocalisationPane, {
    attachTo: host,
    global: { plugins: [pinia, vuetify, i18n] },
  });

  await wrapper.vm.$nextTick();
  return { wrapper, store: useAppearanceStore(pinia) };
}

beforeEach(() => {
  setLocale.mockClear();
  for (const key of Object.keys(replies)) delete replies[key];
  setActivePinia(createPinia());
});

afterEach(() => {
  document.body.innerHTML = '';
});

describe('the localisation pane', () => {
  it('renders with no workspace and no engine', async () => {
    const { wrapper } = await render();
    expect(wrapper.text().trim().length).toBeGreaterThan(0);
    wrapper.unmount();
  });

  /**
   * Not `locale.value = x`. `setLocale` also writes the preference and relabels
   * the tray, so assigning the ref directly would change the window and forget
   * it on the next launch.
   */
  it('changes the app language through setLocale, not by assigning the ref', async () => {
    const { wrapper } = await render();

    const select = wrapper.findAllComponents({ name: 'VSelect' })[0];
    expect(select, 'no language select').toBeTruthy();
    await select.vm.$emit('update:modelValue', 'tr');

    expect(setLocale).toHaveBeenCalledWith('tr');
    wrapper.unmount();
  });

  /**
   * The console locale is a different question from the app's: the stack's
   * output is read by whoever debugs it, which is not always the language the
   * window is in. It belongs to appearance state.
   */
  /**
   * A pack that was just created is 0% translated, and has to say so.
   *
   * `startPack` seeds the file with **every English string** — which is what a
   * translation file is, and is the right thing to hand a translator. But the
   * progress figure counted the strings the file *holds*, so an untouched pack
   * reported `2000 of 2000 (100%)` the moment it was made: a progress bar that
   * is full before the work starts, on a language that is entirely English.
   *
   * `locale.rs` states the rule this broke, in its own doc comment — "a missing
   * string that falls back to English is honest; a fabricated one is a sentence
   * somebody has to find and disbelieve". Two thousand of them, with a number
   * saying the job was done.
   */
  it('reports a freshly seeded pack as untranslated, not as complete', async () => {
    const english = i18n.global.getLocaleMessage('en');
    replies.localePacks = [
      { tag: 'de', label: 'Deutsch', path: '/tmp/de.json', strings: 2000, broken: null },
    ];
    // What `startPack` writes: the English catalogue, relabelled.
    i18n.global.setLocaleMessage('de', { ...english, language: { label: 'Deutsch' } });

    const { wrapper } = await render();
    await new Promise((resolve) => setTimeout(resolve, 0));
    await wrapper.vm.$nextTick();

    const row = wrapper.find('[data-test="locale-pack"]');
    expect(row.exists()).toBe(true);
    expect(row.text()).toContain('0%');
    expect(row.text()).not.toContain('100%');
  });

  it('counts a string as translated when it stops being the English one', async () => {
    const english = i18n.global.getLocaleMessage('en');
    replies.localePacks = [
      { tag: 'de', label: 'Deutsch', path: '/tmp/de.json', strings: 2000, broken: null },
    ];
    i18n.global.setLocaleMessage('de', {
      ...english,
      language: { label: 'Deutsch' },
      app: { ...english.app, close: 'Schließen' },
    });

    const { wrapper } = await render();
    await new Promise((resolve) => setTimeout(resolve, 0));
    await wrapper.vm.$nextTick();

    // One string, out of the whole catalogue: the percentage rounds to zero
    // and the count does not.
    expect(wrapper.find('[data-test="locale-pack"]').text()).toContain('1 of ');
    expect(wrapper.find('[data-test="locale-pack"]').text()).toContain('(0%)');
  });

  /**
   * A language that reads the other way can say so, and the switch stops
   * deciding for it.
   *
   * Direction was one appearance flag applied to every locale at once. That is
   * right for the two languages shipped here — both left to right — and wrong
   * the moment a pack is Arabic or Farsi, which is not hypothetical: two of the
   * five languages the nearest competitor ships are right-to-left. Before this,
   * an Arabic pack rendered left to right until its reader found a switch in
   * Settings, and that switch then mirrored English as well.
   *
   * Fact beats preference, and only here. Arabic reads right to left whether or
   * not anybody chose it; the switch still decides for every locale that has
   * not stated a fact.
   */
  it('lets a pack declare that it reads right to left', async () => {
    replies.localePacks = [
      {
        tag: 'ar',
        label: 'العربية',
        path: '/tmp/ar.json',
        strings: 10,
        broken: null,
        direction: 'rtl',
      },
    ];

    const { wrapper } = await render();
    await new Promise((resolve) => setTimeout(resolve, 0));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[data-test="locale-pack"]').text()).toContain(
      i18n.global.t('settings.packRtl')
    );
  });

  /**
   * The file the button just made, named.
   *
   * "Adding a language is a JSON file somebody drops in the config directory"
   * is only a mechanism a person can use if they can find the file — and the
   * path was in the data the pane already had and on screen nowhere.
   */
  it('says where a pack lives', async () => {
    replies.localePacks = [
      { tag: 'de', label: 'Deutsch', path: '/home/a/.config/stackvo/locales/de.json', strings: 5 },
    ];

    const { wrapper } = await render();
    await new Promise((resolve) => setTimeout(resolve, 0));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[data-test="locale-pack"]').text()).toContain(
      '/home/a/.config/stackvo/locales/de.json'
    );
  });

  /**
   * A file that did not parse is listed with its error rather than quietly
   * missing from the picker — the worst failure this feature could have.
   */
  it('says why a broken pack is not in the picker', async () => {
    replies.localePacks = [
      { tag: 'de', label: 'de', path: '/tmp/de.json', strings: 0, broken: 'trailing comma at 12' },
    ];

    const { wrapper } = await render();
    await new Promise((resolve) => setTimeout(resolve, 0));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[data-test="locale-pack"]').text()).toContain('trailing comma at 12');
  });

  it('writes the console locale and the RTL flag to the appearance store', async () => {
    const { wrapper, store } = await render();
    const set = vi.spyOn(store, 'set');

    const selects = wrapper.findAllComponents({ name: 'VSelect' });
    await selects[1].vm.$emit('update:modelValue', 'en');
    expect(set).toHaveBeenCalledWith({ consoleLocale: 'en' });

    const toggle = wrapper.findComponent({ name: 'VSwitch' });
    await toggle.vm.$emit('update:modelValue', true);
    expect(set).toHaveBeenCalledWith({ rtl: true });

    expect(setLocale, 'appearance state went through the app locale').not.toHaveBeenCalled();
    wrapper.unmount();
  });
});
