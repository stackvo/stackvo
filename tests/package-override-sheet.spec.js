import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { i18n } from '@/i18n';
import PackageOverrideSheet from '@/components/PackageOverrideSheet.vue';

/**
 * Taking over one file of a package somebody else published (decision 0031).
 *
 * The sheet is deliberately not an editor: it hands back a path and gets out of
 * the way, the same answer `PackageAuthorDialog` and `quickcmd` give. So what
 * is worth holding here is the small set of decisions it *does* make — that
 * revert asks first, that the path is shown after a take-over, and that the
 * page behind it is told to reload.
 */

const api = vi.hoisted(() => ({
  packageFiles: vi.fn(),
  packageOverride: vi.fn(),
  packageOverrideRevert: vi.fn(),
}));

vi.mock('@/lib/ipc', () => ({ api, asList: (v) => (Array.isArray(v) ? v : []) }));

const vuetify = createVuetify({ components, directives });

const PUBLISHED = {
  path: 'compose.yml.tpl',
  kind: 'compose',
  companion: null,
  overridden: false,
  at: '/w/overrides/mysql/8.0/compose.yml.tpl',
};

const TAKEN = {
  path: 'files/my.cnf.tpl',
  kind: 'config',
  companion: null,
  overridden: true,
  at: '/w/overrides/mysql/8.0/files/my.cnf.tpl',
};

/**
 * Mounted into the document, for the reason the settings-sheet spec gives: the
 * sheet is a navigation drawer and teleports out of the wrapper's subtree.
 */
function mountSheet() {
  return mount(
    {
      components: { PackageOverrideSheet },
      template: `
        <v-app>
          <PackageOverrideSheet service="mysql" version="8.0" :model-value="true" />
        </v-app>`,
    },
    { attachTo: document.body, global: { plugins: [vuetify, i18n] } }
  );
}

const buttons = () => [...document.body.querySelectorAll('button')];
const button = (text) => buttons().find((b) => b.textContent.trim() === text);
const bodyText = () => document.body.textContent;

describe('the package override sheet', () => {
  let wrapper;

  beforeEach(() => {
    vi.clearAllMocks();
    api.packageFiles.mockResolvedValue([PUBLISHED, TAKEN]);
    api.packageOverride.mockResolvedValue('/w/overrides/mysql/8.0/compose.yml.tpl');
    api.packageOverrideRevert.mockResolvedValue(undefined);
  });

  afterEach(() => wrapper?.unmount());

  it('asks for the files of one version, by service and version', async () => {
    wrapper = mountSheet();
    await flushPromises();

    expect(api.packageFiles).toHaveBeenCalledWith('mysql', '8.0');
  });

  it('offers Take over on a published file and Revert on one already taken', async () => {
    wrapper = mountSheet();
    await flushPromises();

    expect(bodyText()).toContain('compose.yml.tpl');
    expect(bodyText()).toContain('files/my.cnf.tpl');
    expect(button('Take over')).toBeTruthy();
    expect(button('Revert')).toBeTruthy();
  });

  /**
   * The count is the sentence somebody reads before they trust the page: a
   * version with an override renders from bytes the catalogue never published.
   */
  it('says how many files in this workspace are what render', async () => {
    wrapper = mountSheet();
    await flushPromises();

    expect(bodyText()).toContain('1 file(s) in this workspace are what render');
  });

  /**
   * Deleting somebody's edit is not undoable, so the first click asks. Inline
   * rather than in a second dialog — an overlay over a side sheet is two things
   * covering the list being worked from.
   */
  it('asks before deleting an edit, and only deletes on the second click', async () => {
    wrapper = mountSheet();
    await flushPromises();

    await button('Revert').click();
    await flushPromises();
    expect(api.packageOverrideRevert).not.toHaveBeenCalled();

    await button('Delete my copy').click();
    await flushPromises();
    expect(api.packageOverrideRevert).toHaveBeenCalledWith('mysql', '8.0', 'files/my.cnf.tpl');
  });

  /**
   * The path is the whole point of the return value: the next step is opening
   * the file in whatever the person already uses.
   */
  it('shows where the copy landed, and says a render still has to run', async () => {
    wrapper = mountSheet();
    await flushPromises();

    await button('Take over').click();
    await flushPromises();

    expect(api.packageOverride).toHaveBeenCalledWith('mysql', '8.0', 'compose.yml.tpl');
    expect(bodyText()).toContain('/w/overrides/mysql/8.0/compose.yml.tpl');
    expect(bodyText()).toContain('Regenerate afterwards');
  });

  /**
   * The catalogue row behind this sheet carries the override count, so it is
   * stale the moment anything here succeeds.
   */
  it('tells the page to reload after a change', async () => {
    wrapper = mountSheet();
    await flushPromises();

    await button('Take over').click();
    await flushPromises();

    const sheet = wrapper.findComponent(PackageOverrideSheet);
    expect(sheet.emitted('changed')).toHaveLength(1);
  });

  it('surfaces a refusal rather than swallowing it', async () => {
    api.packageOverride.mockRejectedValue(new Error('already overridden'));
    wrapper = mountSheet();
    await flushPromises();

    await button('Take over').click();
    await flushPromises();

    expect(bodyText()).toContain('already overridden');
  });
});
