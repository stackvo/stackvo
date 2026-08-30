import { describe, it, expect, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { PRESET_FILE } from '@/composables/useStackPreset';

/**
 * Where a preset lives, and the notice that says one is here.
 *
 * `preset.rs` had solved the right problem — a clone brings `stackvo.json` and
 * not the stack, because the stack is in `.env` and `.env` is where every
 * password is — and had left one thing out: **the file had nowhere to be.** The
 * export wrote one, the import read one, and between them somebody had to say
 * out loud where they had put it. DDEV has no such question; the file is
 * `.ddev/config.yaml` and `ddev start` reads it.
 *
 * Two halves are checked here, and they fail for different reasons. **One
 * spelling**, because the export suggested `<name>.stackvo-preset.json` while
 * nothing looked for that — an export writing a file no reader looks for is the
 * feature quietly not working. And **the notice**, which is what turns "email
 * your colleague the file" into "clone the repository".
 */

globalThis.visualViewport = undefined;

let answer = null;
const applied = [];

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: {
    projectRequirements: () => Promise.resolve(answer),
    presetApply: (path) => {
      applied.push(path);
      return Promise.resolve({ changes: [] });
    },
    projectRequirementsApply: () => Promise.resolve({ changes: [] }),
    projectRequirementsDeclare: () => Promise.resolve(null),
  },
}));

const { i18n } = await import('@/i18n');
const Pane = (await import('@/components/project/RequirementsPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const AT = '/Users/x/StackVo/projects/shop/stackvo.preset.json';

const OFFER = (changes) => ({
  path: AT,
  name: 'Shop stack',
  description: 'MySQL 8.4 and Redis',
  plan: { changes },
});

async function pane(state) {
  answer = { declared: [], suggested: [], plan: null, ...state };
  applied.length = 0;
  const wrapper = mount(Pane, {
    props: { name: 'shop' },
    global: { plugins: [vuetify, i18n] },
  });
  await Promise.resolve();
  await Promise.resolve();
  await wrapper.vm.$nextTick();
  return wrapper;
}

describe('the preset convention', () => {
  /**
   * The two sides of one string. Rust looks for exactly this name and the save
   * dialog offers exactly this name; a divergence is an export nothing reads.
   */
  it('is one filename, in Rust and in the export dialog', () => {
    expect(PRESET_FILE).toBe('stackvo.preset.json');

    const rust = readFileSync('src-tauri/src/preset.rs', 'utf8');
    expect(rust).toContain(`pub const CONVENTIONAL_FILE: &str = "${PRESET_FILE}";`);

    // And the old spelling is gone from the *code* rather than merely unused —
    // a second name left in the source is one somebody restores.
    //
    // Comments stripped, for the fourth time in this repository and the same
    // reason each time: the sentence explaining why a name is wrong contains
    // the name, and a scanner that read prose would fail on its own rationale.
    const code = readFileSync('src/composables/useStackPreset.js', 'utf8')
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .replace(/(^|[^:])\/\/.*$/gm, '$1');
    expect(code).not.toContain('stackvo-preset.json');
  });
});

describe('the requirements pane', () => {
  it('says a project ships a preset, and what applying it would change', async () => {
    const wrapper = await pane({ preset: OFFER([{ key: 'SERVICE_MYSQL_ENABLE' }]) });

    expect(wrapper.text()).toContain(i18n.global.t('requirements.preset.pending', { count: 1 }));
    expect(wrapper.text()).toContain('Shop stack');
    expect(wrapper.text()).toContain('MySQL 8.4 and Redis');
  });

  /**
   * Hidden once the stack already matches — which is the state a project sits
   * in after somebody applied it. A permanent banner is a line nobody reads by
   * the third visit, and the moment this matters is the first open after a
   * clone.
   */
  it('says nothing when applying it would change nothing', async () => {
    const wrapper = await pane({ preset: OFFER([]) });

    expect(wrapper.text()).not.toContain(
      i18n.global.t('requirements.preset.pending', { count: 0 })
    );
    expect(wrapper.text()).not.toContain('Shop stack');
  });

  /** And nothing at all when the project ships none, which is most of them. */
  it('says nothing when there is no preset', async () => {
    const wrapper = await pane({ preset: null });
    expect(wrapper.text()).not.toContain('Shop stack');
  });

  /**
   * Applying goes through `preset_apply` with the path the back end supplied —
   * the front end never builds it and so cannot build it wrong — and that is
   * the same plan-then-apply command the Settings import uses.
   */
  it('applies it through the reviewed import, by the path the back end gave', async () => {
    const wrapper = await pane({ preset: OFFER([{ key: 'SERVICE_MYSQL_ENABLE' }]) });

    const button = wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === i18n.global.t('requirements.preset.apply'));
    expect(button, 'no apply button was drawn').toBeTruthy();

    await button.trigger('click');
    await Promise.resolve();

    expect(applied).toEqual([AT]);
  });
});
