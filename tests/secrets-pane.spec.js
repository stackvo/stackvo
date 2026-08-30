import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';
import { i18n } from '@/i18n';
import SecretsPane from '@/components/settings/SecretsPane.vue';

/**
 * The pane that changes where a password lives.
 *
 * Two of these are about what it *says* rather than what it does, and they are
 * the ones worth having: a keystore feature is read as "the secret left the
 * disk", and this one leaves it in `generated/docker-compose.dynamic.yml`. A
 * pane that quietly stopped saying so would be the difference between a partial
 * win and a false claim, and nothing else in the build would notice.
 */

const api = vi.hoisted(() => ({
  secretsStatus: vi.fn(),
  secretMove: vi.fn(),
  secretRestore: vi.fn(),
  leaksScan: vi.fn(),
  envUntrack: vi.fn(),
  projectsList: vi.fn(),
}));

vi.mock('@/lib/ipc', () => ({ api }));

const vuetify = createVuetify({ components, directives });

const mountPane = () => mount(SecretsPane, { global: { plugins: [vuetify, i18n] } });

const STATUS = {
  available: true,
  keys: [
    { key: 'SERVICE_MYSQL_ROOT_PASSWORD', moved: false, resolvable: true, set: true },
    { key: 'SERVICE_GRAFANA_ADMIN_PASSWORD', moved: true, resolvable: true, set: true },
    { key: 'SERVICE_POSTGRES_PASSWORD', moved: false, resolvable: false, set: false },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  api.secretsStatus.mockResolvedValue(STATUS);
  api.secretMove.mockResolvedValue(undefined);
  api.secretRestore.mockResolvedValue(undefined);
  api.leaksScan.mockResolvedValue({
    findings: [],
    scanned: 0,
    skipped: 0,
    truncated: false,
    envTracked: false,
    envInHistory: false,
  });
  api.projectsList.mockResolvedValue([{ name: 'shop' }]);
  api.envUntrack.mockResolvedValue({
    untracked: true,
    ignored: true,
    gitignoreWritten: true,
    exampleKeys: 12,
    stillInHistory: true,
    needsCommit: true,
  });
});

describe('what the pane says before it offers a button', () => {
  it('names the file the password is still written into', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('generated/docker-compose.dynamic.yml');
    expect(wrapper.text()).toContain('does not take it off the disk');
  });

  it('warns that the command-line tool cannot read a moved credential', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('stackvo.sh');
  });
});

describe('the rows', () => {
  it('shows a key with no value nowhere, because there is nothing to move', async () => {
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('SERVICE_MYSQL_ROOT_PASSWORD');
    expect(wrapper.text()).not.toContain('SERVICE_POSTGRES_PASSWORD');
  });

  it('offers Move for a key in the file and Restore for one in the keystore', async () => {
    const wrapper = mountPane();
    await flushPromises();

    const labels = wrapper.findAll('.v-list-item button').map((b) => b.text());
    expect(labels).toEqual(['Move', 'Restore']);
  });

  it('moves the key it was asked about and re-reads afterwards', async () => {
    const wrapper = mountPane();
    await flushPromises();

    await wrapper.findAll('.v-list-item button')[0].trigger('click');
    await flushPromises();

    expect(api.secretMove).toHaveBeenCalledWith('SERVICE_MYSQL_ROOT_PASSWORD');
    expect(api.secretRestore).not.toHaveBeenCalled();
    expect(api.secretsStatus).toHaveBeenCalledTimes(2);
  });

  /**
   * A failed move may still have written the keystore entry, so the row's old
   * state is not something to keep showing — it would be a claim about the disk
   * that nobody checked.
   */
  it('re-reads even when the move failed', async () => {
    api.secretMove.mockRejectedValue(new Error('keychain is locked'));
    const wrapper = mountPane();
    await flushPromises();

    await wrapper.findAll('.v-list-item button')[0].trigger('click');
    await flushPromises();

    expect(api.secretsStatus).toHaveBeenCalledTimes(2);
    expect(wrapper.text()).toContain('keychain is locked');
  });
});

describe('a machine with no keystore', () => {
  it('says so and disables every button rather than offering one that cannot work', async () => {
    api.secretsStatus.mockResolvedValue({ ...STATUS, available: false });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('no keystore');
    const buttons = wrapper.findAll('.v-list-item button');
    expect(buttons.length).toBeGreaterThan(0);
    expect(buttons.every((b) => b.attributes('disabled') !== undefined)).toBe(true);
  });
});

describe('a reference the keystore will not answer', () => {
  /** Generation is blocked while this is true, so the row has to be loud. */
  it('is reported by name', async () => {
    api.secretsStatus.mockResolvedValue({
      available: true,
      keys: [{ key: 'SERVICE_MYSQL_ROOT_PASSWORD', moved: true, resolvable: false, set: true }],
    });
    const wrapper = mountPane();
    await flushPromises();

    expect(wrapper.text()).toContain('did not answer');
    expect(wrapper.text()).toContain('SERVICE_MYSQL_ROOT_PASSWORD');
  });
});

/**
 * The other direction.
 *
 * The matching is settled in Rust, where `leaks.rs` holds the cases that decide
 * whether the feature is usable at all — the shapes that must not match. What
 * only exists here is what the screen does with a finding, and the one thing it
 * must never do: show the value. A report that quotes the secret is a second
 * copy of it, on a screen people photograph and paste.
 */
describe('scanning for credentials nobody moved', () => {
  const scan = async (wrapper) => {
    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Scan for credentials')
      .trigger('click');
    await flushPromises();
  };

  it('names where each finding is and what it looks like, never the value', async () => {
    api.leaksScan.mockResolvedValue({
      findings: [
        { id: 'awsAccessKey', source: 'tracked', subject: 'deploy.sh', line: 12 },
        { id: 'unstoredSecret', source: 'env', subject: 'SERVICE_REDIS_PASSWORD' },
      ],
      scanned: 214,
      skipped: 0,
      truncated: false,
      envTracked: false,
    });

    const wrapper = mountPane();
    await flushPromises();
    await scan(wrapper);

    const found = wrapper.findAll('[data-test="leak"]');
    expect(found).toHaveLength(2);
    expect(found[0].text()).toContain('deploy.sh');
    expect(found[0].text()).toContain('12');
    expect(found[0].text()).toContain('AWS access key');
    expect(found[1].text()).toContain('SERVICE_REDIS_PASSWORD');
    expect(found[1].text()).toContain('.env');
  });

  /**
   * "Never print the secret" on its own leaves somebody with a finding they
   * cannot act on: two rows saying `awsAccessKey` do not say whether that is
   * one key in two places or two keys.
   */
  it('carries a fingerprint and a masked preview, and never the value', async () => {
    api.leaksScan.mockResolvedValue({
      findings: [
        {
          id: 'awsAccessKey',
          source: 'tracked',
          subject: 'deploy.sh',
          line: 12,
          fingerprint: 'a1b2c3d4e5f6',
          preview: 'AKIA…MPLE',
          inHistory: true,
        },
      ],
      scanned: 1,
      skipped: 0,
      truncated: false,
      envTracked: false,
      envInHistory: false,
    });

    const wrapper = mountPane();
    await flushPromises();
    await scan(wrapper);

    const row = wrapper.find('[data-test="leak"]').text();
    expect(row).toContain('AKIA…MPLE');
    expect(row).toContain('a1b2c3d4e5f6');
    // And the one it cannot be: the value put back together.
    expect(row).not.toContain('AKIAIOSFODNN7EXAMPLE');
    // A file already in a commit is a different sentence from one only staged.
    expect(row).toContain('already committed');
  });

  /**
   * The repair, and the two halves it does not do. A fix somebody thinks they
   * made is worse than no fix.
   */
  it('offers the standard repair and then says what is left to do', async () => {
    api.leaksScan.mockResolvedValue({
      findings: [],
      scanned: 3,
      skipped: 0,
      truncated: false,
      envTracked: true,
      envInHistory: true,
    });

    const wrapper = mountPane();
    await flushPromises();

    // The repair needs a repository, so it is offered only once one is named.
    await scan(wrapper);
    expect(wrapper.findAllComponents({ name: 'VBtn' }).map((b) => b.text())).not.toContain(
      'Take .env out of git'
    );

    await wrapper
      .findAllComponents({ name: 'VSelect' })
      .find((sel) => sel.props('label') === 'Also scan a repository')
      .setValue('shop');
    await scan(wrapper);

    await wrapper
      .findAllComponents({ name: 'VBtn' })
      .find((b) => b.text() === 'Take .env out of git')
      .trigger('click');
    await flushPromises();

    expect(api.envUntrack).toHaveBeenCalledWith('shop');
    const said = wrapper.find('[data-test="untracked"]').text();
    expect(said).toContain('12');
    // The two halves it did not do, both said out loud.
    expect(said).toContain('staged');
    expect(said).toContain('Rotate them');
  });

  it('puts a tracked .env above everything else, because it outranks everything else', async () => {
    api.leaksScan.mockResolvedValue({
      findings: [],
      scanned: 3,
      skipped: 0,
      truncated: false,
      envTracked: true,
    });

    const wrapper = mountPane();
    await flushPromises();
    await scan(wrapper);

    // No rule matched and it is still the worst possible result: every value in
    // that file is in the history whatever it looks like.
    expect(wrapper.text()).toContain('tracked by git');
    expect(wrapper.text()).not.toContain('Nothing found');
  });

  it('says how many files it did not read, so a short scan is not read as a clean one', async () => {
    api.leaksScan.mockResolvedValue({
      findings: [],
      scanned: 2000,
      skipped: 431,
      truncated: true,
      envTracked: false,
    });

    const wrapper = mountPane();
    await flushPromises();
    await scan(wrapper);

    expect(wrapper.text()).toContain('431');
  });

  it('says nothing was found rather than showing an empty list', async () => {
    const wrapper = mountPane();
    await flushPromises();
    await scan(wrapper);

    expect(wrapper.findAll('[data-test="leak"]')).toHaveLength(0);
    expect(wrapper.text()).toContain('Nothing found');
  });
});
