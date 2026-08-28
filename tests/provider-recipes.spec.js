import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The starting points a project with no recipe of its own is offered.
 *
 * The mechanism behind this card was finished long before there was anything
 * in it, and the card hid itself when a project declared nothing — so the only
 * way to reach the feature was to already know the file format. These hold the
 * two halves of the fix that a Rust test cannot see: that the list appears for
 * a project with nothing and *disappears* for one that has something, and that
 * what is on screen before the button is the command the button adds.
 */

globalThis.visualViewport = undefined;

const projectProviders = vi.fn();
const providerRecipes = vi.fn();
const providerRecipeAdd = vi.fn();

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: {
    projectProviders: (...args) => projectProviders(...args),
    providerRecipes: (...args) => providerRecipes(...args),
    providerRecipeAdd: (...args) => providerRecipeAdd(...args),
    dbTargets: () => Promise.resolve([]),
  },
}));

const { i18n } = await import('@/i18n');
const ProvidersPane = (await import('@/components/project/ProvidersPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const RECIPES = [
  {
    name: 'mysql-remote',
    about: 'A MySQL or MariaDB server this machine can reach directly',
    edit: ['the host, port, user and database name in both commands'],
    image: 'mysql:8.4',
    pull: ['mysqldump', '--host=db.example.com', '--result-file=/stackvo/dump.sql', 'the_database'],
    push: ['mysql', '--execute=source /stackvo/dump.sql', 'the_database'],
    secrets: ['MYSQL_PWD'],
  },
  {
    name: 'upsun',
    about: 'An Upsun environment, through its own CLI',
    edit: ['the project id and the environment name'],
    image: 'ghcr.io/upsun/cli:latest',
    pull: ['db:dump', '--directory=/stackvo', '--file=dump.sql'],
    push: [],
    secrets: ['UPSUN_CLI_TOKEN'],
  },
];

// Counts are an assertion in one of these, and the spies are module-level.
beforeEach(() => {
  vi.clearAllMocks();
});

async function mountPane() {
  const wrapper = mount(
    {
      components: { ProvidersPane },
      template: '<v-app><ProvidersPane name="shop" /></v-app>',
    },
    { global: { plugins: [vuetify, i18n] } }
  );
  await new Promise((resolve) => setTimeout(resolve, 0));
  await wrapper.vm.$nextTick();
  return wrapper;
}

describe('the providers card, on a project that declares nothing', () => {
  it('offers the shipped recipes rather than hiding the whole card', async () => {
    projectProviders.mockResolvedValue({ recipes: [], plans: [], problems: [] });
    providerRecipes.mockResolvedValue(RECIPES);

    const wrapper = await mountPane();
    const list = wrapper.find('[data-test="provider-recipes"]');
    expect(list.exists()).toBe(true);
    expect(list.text()).toContain('mysql-remote');
    expect(list.text()).toContain('upsun');
  });

  /**
   * The command is the thing being added, so it is on screen before the
   * button — the same reasoning the approval half of this card is built on.
   */
  it('shows the image and the command each recipe would add', async () => {
    projectProviders.mockResolvedValue({ recipes: [], plans: [], problems: [] });
    providerRecipes.mockResolvedValue(RECIPES);

    const text = (await mountPane()).find('[data-test="provider-recipes"]').text();
    expect(text).toContain('mysql:8.4');
    expect(text).toContain('--result-file=/stackvo/dump.sql');
    expect(text).toContain('ghcr.io/upsun/cli:latest');
  });

  /**
   * Every shipped recipe carries a placeholder host or project id, so a card
   * that did not say which words they are would be a command somebody
   * approves as it stands.
   */
  it('says what has to be edited before the recipe can work', async () => {
    projectProviders.mockResolvedValue({ recipes: [], plans: [], problems: [] });
    providerRecipes.mockResolvedValue(RECIPES);

    const text = (await mountPane()).find('[data-test="provider-recipes"]').text();
    expect(text).toContain('the host, port, user and database name in both commands');
    expect(text).toContain('the project id and the environment name');
  });

  /** A recipe that only fetches is not offered as one that also sends. */
  it('marks a recipe with no push as fetch-only', async () => {
    projectProviders.mockResolvedValue({ recipes: [], plans: [], problems: [] });
    providerRecipes.mockResolvedValue([RECIPES[1]]);

    const chips = (await mountPane()).findAll('.v-chip');
    expect(chips).toHaveLength(1);
    expect(chips[0].text()).toBe(i18n.global.t('providers.pull'));
  });

  it('adds by name and reloads, so the pane shows what is now declared', async () => {
    projectProviders.mockResolvedValue({ recipes: [], plans: [], problems: [] });
    providerRecipes.mockResolvedValue([RECIPES[0]]);
    providerRecipeAdd.mockResolvedValue({});

    const wrapper = await mountPane();
    await wrapper.find('[data-test="provider-recipes"] .v-btn').trigger('click');
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(providerRecipeAdd).toHaveBeenCalledWith('shop', 'mysql-remote');
    expect(projectProviders).toHaveBeenCalledTimes(2);
  });
});

describe('the providers card, on a project that already declares one', () => {
  /**
   * The starting points answer a question this project has answered. Leaving
   * them under somebody's working configuration reads as an invitation to add
   * another one.
   */
  it('does not offer starting points', async () => {
    projectProviders.mockResolvedValue({
      recipes: [{ name: 'staging' }],
      plans: [
        {
          provider: 'staging',
          direction: 'pull',
          image: 'ghcr.io/example/dbtools:1',
          command: ['fetch-dump'],
          env: {},
          secrets: [],
          digest: 'abc',
          blocked: 'needs-consent',
        },
      ],
      problems: [],
    });
    providerRecipes.mockResolvedValue(RECIPES);

    const wrapper = await mountPane();
    expect(wrapper.find('[data-test="provider-recipes"]').exists()).toBe(false);
    expect(wrapper.find('[data-test="provider-plan"]').exists()).toBe(true);
  });
});
