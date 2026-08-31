import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The Appearance pane, mounted.
 *
 * Sixth out of `Settings.vue` in the pane split and the cleanest seam so far: the
 * only pane that touches neither the `.env` editor nor the operation console.
 * Everything it changes lives in `useAppearanceStore`, which persists and
 * applies on its own.
 *
 * What is worth asserting is therefore not "does the store work" — `appearance`
 * has its own coverage — but the three things that only exist once the markup
 * runs and that a reviewer cannot see by reading it:
 *
 *   * the reset affordance tells the truth about whether anything is customised;
 *   * a preset cannot be saved without a name, and saving clears the field;
 *   * every swatch, font and palette the library ships is actually offered,
 *     rather than a subset somebody transcribed by hand.
 */

globalThis.visualViewport = undefined;

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: new Proxy({}, { get: () => () => Promise.resolve(null) }),
}));

const { i18n } = await import('@/i18n');
const AppearancePane = (await import('@/components/settings/AppearancePane.vue')).default;
const { useAppearanceStore } = await import('@/stores/appearance');
const {
  AUDIT_PAIRS,
  DEFAULT_APPEARANCE,
  PRIMARY_SWATCHES,
  HARMONIES,
  FONT_FAMILIES,
  STATUS_PALETTES,
} = await import('@/lib/appearance');

const vuetify = createVuetify({ components, directives });

/**
 * One pinia, handed to `mount` and resolved explicitly.
 *
 * `setActivePinia` alone gave the test one store and the component another in
 * `settings-workspace.spec.js` — state written by the test was invisible to the
 * thing under test, and the assertion failed for no stated reason.
 */
async function render() {
  const host = document.createElement('div');
  document.body.appendChild(host);

  const pinia = createPinia();
  setActivePinia(pinia);

  const wrapper = mount(AppearancePane, {
    attachTo: host,
    global: { plugins: [pinia, vuetify, i18n] },
  });

  await wrapper.vm.$nextTick();
  return { wrapper, store: useAppearanceStore(pinia) };
}

const button = (wrapper, label) => wrapper.findAll('button').find((b) => b.text().includes(label));

beforeEach(() => {
  setActivePinia(createPinia());
});

afterEach(() => {
  document.body.innerHTML = '';
});

describe('the appearance pane', () => {
  it('renders without a workspace, an engine or a .env', async () => {
    const { wrapper } = await render();
    expect(wrapper.text().trim().length).toBeGreaterThan(0);
    wrapper.unmount();
  });

  /**
   * "Back to defaults" should not be a leap of faith. The flag has to follow
   * the store rather than being set once — a stale one either offers a reset
   * that does nothing, or hides one that would help.
   */
  it('knows whether anything has been customised', async () => {
    const { wrapper, store } = await render();
    expect(wrapper.vm.isDefaultAppearance, 'a fresh store is not the default').toBe(true);

    const key = Object.keys(DEFAULT_APPEARANCE)[0];
    store.value[key] = 'something-else';
    await wrapper.vm.$nextTick();
    expect(wrapper.vm.isDefaultAppearance, 'a change went unnoticed').toBe(false);

    store.value[key] = DEFAULT_APPEARANCE[key];
    await wrapper.vm.$nextTick();
    expect(wrapper.vm.isDefaultAppearance, 'undoing the change did not restore it').toBe(true);

    wrapper.unmount();
  });
});

describe('saving a preset', () => {
  it('will not save one without a name', async () => {
    const { wrapper, store } = await render();
    const saved = vi.spyOn(store, 'savePreset');

    const save = button(wrapper, i18n.global.t('settings.savePreset'));
    expect(save, 'no save button').toBeTruthy();
    expect(save.attributes('disabled'), 'an unnamed preset was saveable').toBeDefined();

    await save.trigger('click');
    expect(saved).not.toHaveBeenCalled();

    wrapper.unmount();
  });

  /**
   * The field has to clear, or the next preset is offered the previous one's
   * name and a second click quietly overwrites it.
   */
  it('saves under the typed name and then clears the field', async () => {
    const { wrapper, store } = await render();
    const saved = vi.spyOn(store, 'savePreset').mockResolvedValue(undefined);

    wrapper.vm.presetName = 'midnight';
    await wrapper.vm.$nextTick();

    const save = button(wrapper, i18n.global.t('settings.savePreset'));
    expect(save.attributes('disabled'), 'a named preset was not saveable').toBeUndefined();

    await save.trigger('click');
    await vi.waitFor(() => expect(saved).toHaveBeenCalledWith('midnight'));
    expect(wrapper.vm.presetName, 'the name stayed in the box').toBe('');

    wrapper.unmount();
  });
});

describe('what the pane offers', () => {
  /**
   * Derived from the library, not transcribed. A hand-written subset is how a
   * palette gets added to `appearance.js` and never appears in the app — and
   * nothing would report it.
   */
  it('offers every status palette, font and harmony the library ships', async () => {
    const { wrapper } = await render();

    expect(wrapper.vm.statusItems.map((i) => i.value)).toEqual(STATUS_PALETTES.map((p) => p.id));
    expect(wrapper.vm.fontItems.map((i) => i.value)).toEqual(FONT_FAMILIES.map((f) => f.id));
    expect(wrapper.vm.harmonyItems.map((i) => i.value)).toEqual([...HARMONIES]);

    // And each carries a translated label rather than the raw id.
    for (const item of [
      ...wrapper.vm.statusItems,
      ...wrapper.vm.fontItems,
      ...wrapper.vm.harmonyItems,
    ]) {
      expect(item.title, `${item.value} has no label`).toBeTruthy();
      expect(item.title).not.toContain('settings.');
    }

    wrapper.unmount();
  });

  /**
   * The contrast control is three buttons rather than the switch it replaced,
   * and the three have to actually be there: a `v-btn-toggle` with `mandatory`
   * and a value nothing in it matches renders an empty group and silently
   * accepts no input.
   */
  it('offers all three contrast stops, each with a label', async () => {
    const { wrapper } = await render();

    for (const stop of ['Standard', 'Medium', 'High']) {
      const label = i18n.global.t(`settings.contrast${stop}`);
      expect(label, `settings.contrast${stop} is untranslated`).not.toContain('settings.');
      expect(button(wrapper, label), `${label} is not offered`).toBeTruthy();
    }

    wrapper.unmount();
  });

  it('renders a swatch for every primary colour', async () => {
    const { wrapper } = await render();
    const text = wrapper.html();

    // The swatches are drawn from the same list; a missing one is a colour
    // nobody can choose.
    for (const swatch of PRIMARY_SWATCHES) {
      const value = typeof swatch === 'string' ? swatch : (swatch.value ?? swatch.id);
      expect(text.includes(value), `${value} is not offered`).toBe(true);
    }

    wrapper.unmount();
  });
});

/**
 * The audit, mounted — which is the only place it can be checked.
 *
 * Half of every pair it measures is an `on-*` colour, and those do not exist
 * in `lib/appearance.js` at all: Vuetify derives them from the fill, inside
 * `computedThemes`, at render time. A unit test over `auditTheme` would be
 * checking arithmetic against colours the application never uses. This mounts
 * the pane, lets Vuetify build both preview themes, and reads what the table
 * will actually print.
 */
describe('the contrast audit', () => {
  const rowsFor = (wrapper, index) => wrapper.vm.audits[index];

  it('measures every pair in both themes, with nothing unreadable', async () => {
    const { wrapper } = await render();

    expect(wrapper.vm.audits, 'both preview themes should be audited').toHaveLength(2);

    for (const [n, rows] of wrapper.vm.audits.entries()) {
      expect(
        rows.map((r) => r.id),
        `theme ${n} audited the wrong pairs`
      ).toEqual(AUDIT_PAIRS.map((p) => p.id));

      for (const row of rows) {
        // `null` means a role was missing, which is a different and worse
        // problem than a low ratio — and the one a silent `—` in the table
        // would hide.
        expect(row.ratio, `theme ${n}: ${row.id} could not be measured`).not.toBeNull();
        expect(row.ratio).toBeGreaterThan(1);
      }
    }

    wrapper.unmount();
  });

  /**
   * The claim the help document makes, and the reason the table is worth
   * showing at all: the colours this application ships with pass, everywhere,
   * in both themes, out of the box.
   */
  it('passes AA on every pair at the default settings', async () => {
    const { wrapper } = await render();

    for (const [n, rows] of wrapper.vm.audits.entries()) {
      const failing = rows.filter((r) => r.grade === 'fail');
      expect(
        failing.map((r) => `${r.id} ${r.ratio.toFixed(2)}:1`),
        `theme ${n} ships a pair below AA`
      ).toEqual([]);
    }

    wrapper.unmount();
  });

  /**
   * A "high contrast" setting that lowered a ratio would be worse than no
   * setting, and nothing in `CONTRAST_LEVELS` makes that impossible — it is a
   * literal three numbers wide. Measured end to end rather than asserted on
   * the table, because the levels reach the screen through two different
   * mechanisms: `readable`'s target moves the status rows, and the emphasis
   * opacity moves the caption row.
   */
  it('never lowers a ratio when the contrast level is raised', async () => {
    const { wrapper, store } = await render();

    const snapshot = () => wrapper.vm.audits.map((rows) => rows.map((r) => r.ratio));

    store.value = { ...store.value, contrast: 'standard' };
    await wrapper.vm.$nextTick();
    const standard = snapshot();

    for (const level of ['medium', 'high']) {
      store.value = { ...store.value, contrast: level };
      await wrapper.vm.$nextTick();

      snapshot().forEach((rows, n) =>
        rows.forEach((ratio, i) => {
          expect(
            ratio,
            `${level}: ${AUDIT_PAIRS[i].id} in theme ${n} fell from ${standard[n][i].toFixed(2)}`
          ).toBeGreaterThanOrEqual(standard[n][i] - 0.001);
        })
      );
    }

    wrapper.unmount();
  });

  /** And the rows that are supposed to move actually move. */
  it('raises the secondary-text row when the level is raised', async () => {
    const { wrapper, store } = await render();
    const caption = () =>
      rowsFor(wrapper, 0)[AUDIT_PAIRS.findIndex((p) => p.id === 'caption')].ratio;

    store.value = { ...store.value, contrast: 'standard' };
    await wrapper.vm.$nextTick();
    const before = caption();

    store.value = { ...store.value, contrast: 'high' };
    await wrapper.vm.$nextTick();

    expect(caption(), 'the contrast setting moved nothing').toBeGreaterThan(before);

    wrapper.unmount();
  });
});

/**
 * Sharing a look, from the page rather than from the library.
 *
 * `parseAppearance` has its own coverage; what only exists once the markup runs
 * is the wiring — that the button reaches the store at all, that a bad paste
 * does not, and that the field clears. The last one is the same failure the
 * preset name field had: a box that keeps its contents offers the next import
 * the previous one's text.
 */
describe('importing a look', () => {
  const importButton = (wrapper) => button(wrapper, i18n.global.t('settings.importAction'));

  it('will not import an empty box', async () => {
    const { wrapper, store } = await render();
    const set = vi.spyOn(store, 'set');

    expect(
      importButton(wrapper).attributes('disabled'),
      'an empty import was offered'
    ).toBeDefined();
    expect(set).not.toHaveBeenCalled();

    wrapper.unmount();
  });

  it('applies a pasted look and clears the box', async () => {
    const { wrapper, store } = await render();
    const set = vi.spyOn(store, 'set').mockResolvedValue(undefined);

    wrapper.vm.importText = JSON.stringify({ ...DEFAULT_APPEARANCE, neutral: 'warm' });
    await wrapper.vm.$nextTick();

    await importButton(wrapper).trigger('click');
    await vi.waitFor(() => expect(set).toHaveBeenCalled());

    expect(set.mock.calls[0][0].neutral, 'the pasted look did not reach the store').toBe('warm');
    expect(wrapper.vm.importText, 'the paste stayed in the box').toBe('');

    wrapper.unmount();
  });

  /** A wrong paste must not reset the look it was pasted next to. */
  it('leaves the current look alone when the paste is not one', async () => {
    const { wrapper, store } = await render();
    const set = vi.spyOn(store, 'set').mockResolvedValue(undefined);

    wrapper.vm.importText = 'not a look';
    await wrapper.vm.$nextTick();
    await importButton(wrapper).trigger('click');
    await wrapper.vm.$nextTick();

    expect(set, 'a bad paste was applied').not.toHaveBeenCalled();
    expect(wrapper.vm.importText, 'a rejected paste was thrown away').toBe('not a look');

    wrapper.unmount();
  });

  /**
   * The two things the copy buttons put on the clipboard. Checked as values
   * rather than through a click, because jsdom has no clipboard and
   * `useCopyTick` swallows that failure by design.
   */
  it('offers the look as settings and as a Vuetify theme', async () => {
    const { wrapper } = await render();

    expect(JSON.parse(wrapper.vm.lookJson).neutral).toBe(DEFAULT_APPEARANCE.neutral);
    expect(wrapper.vm.lookSnippet).toContain('createVuetify');
    expect(wrapper.vm.lookSnippet).toContain(DEFAULT_APPEARANCE.primary);

    wrapper.unmount();
  });
});
