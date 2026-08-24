import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * B-7: who can open the tunnel, and what its address is called.
 *
 * The pane's older half — the picker, the token field, the URL — is covered by
 * `tunnel.rs`'s own tests, because everything shown there is the provider
 * table's. What is tested here is the part where the screen can be *wrong*
 * rather than merely empty, and there are three of those:
 *
 * * a link that asks for a password and a link that does not must not look the
 *   same, and the warning about an open link must disappear exactly when it
 *   stops being true;
 * * a credential stored *after* a tunnel was opened protects nothing, and a
 *   pane that showed a padlock for it would be lying about a live public URL;
 * * a reserved name that the provider quietly did not grant — measured, real,
 *   and invisible without this — has to be said next to the address that was
 *   granted instead.
 */

globalThis.visualViewport = undefined;

const replies = {};
const calls = [];

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
          calls.push([String(name), ...args]);
          const reply = replies[name];
          return typeof reply === 'function' ? reply(...args) : Promise.resolve(reply);
        },
    }
  ),
}));

const { i18n } = await import('@/i18n');
const en = (await import('@/i18n/locales/en.js')).default;
const TunnelPane = (await import('@/components/project/TunnelPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const PROVIDERS = [
  {
    id: 'cloudflare',
    image: 'cloudflare/cloudflared:latest',
    anonymous: true,
    tokenEnv: null,
    urlSuffixes: ['.trycloudflare.com'],
    rewritesHost: true,
    sessionMinutes: null,
    verified: true,
    reserved: null,
    hasToken: false,
  },
  {
    id: 'localtunnel',
    image: 'node:22-alpine',
    anonymous: true,
    tokenEnv: null,
    urlSuffixes: ['.loca.lt'],
    rewritesHost: false,
    sessionMinutes: null,
    verified: true,
    reserved: { kind: 'subdomain', dotted: false, inLog: true },
    hasToken: false,
  },
];

/** A tunnel that is up, unguarded, on a quick-tunnel address. */
const OPEN = {
  project: 'shop',
  running: true,
  url: 'https://four-random-words.trycloudflare.com',
  container: 'stackvo-tunnel-shop',
  provider: 'cloudflare',
  failure: null,
  guarded: false,
  reserved: null,
  reservedHonoured: null,
};

const mountPane = (name = 'shop') =>
  mount(
    {
      components: { TunnelPane },
      props: ['name'],
      template: '<v-app><TunnelPane :name="name" :running="true" /></v-app>',
    },
    { props: { name }, global: { plugins: [vuetify, i18n] } }
  );

beforeEach(() => {
  calls.length = 0;
  for (const key of Object.keys(replies)) delete replies[key];
  replies.tunnelStatus = [];
  replies.tunnelProviders = PROVIDERS;
  replies.tunnelIdentity = { authUser: null, keystore: true, reserved: {} };
});

describe('who can open the link', () => {
  it('warns that an open tunnel is open, and stops warning when it is guarded', async () => {
    replies.tunnelStatus = [OPEN];
    const open = mountPane();
    await flushPromises();
    expect(open.text()).toContain(en.tunnel.publicWarning);

    replies.tunnelStatus = [{ ...OPEN, guarded: true }];
    replies.tunnelIdentity = { authUser: 'stackvo', keystore: true, reserved: {} };
    const guarded = mountPane();
    await flushPromises();
    expect(guarded.text()).not.toContain(en.tunnel.publicWarning);
    expect(guarded.text()).toContain('This link asks for a password');
  });

  /**
   * The case a padlock read off the keystore would get wrong: the credential
   * exists, and the tunnel in front of it was opened before it did.
   */
  it('says a credential added after the tunnel does not protect it yet', async () => {
    replies.tunnelStatus = [OPEN];
    replies.tunnelIdentity = { authUser: 'stackvo', keystore: true, reserved: {} };
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain(en.tunnel.publicWarning);
    expect(pane.text()).toContain(en.tunnel.restartToProtect);
  });

  /**
   * The password is never in the identity, so it cannot be on screen until
   * somebody asks — and asking is a command of its own.
   */
  it('shows the password only when it is asked for', async () => {
    replies.tunnelStatus = [{ ...OPEN, guarded: true }];
    replies.tunnelIdentity = { authUser: 'stackvo', keystore: true, reserved: {} };
    replies.tunnelAuthReveal = { user: 'stackvo', password: 'Kf3xQ9prTnWbMvJdRhZs' };

    const pane = mountPane();
    await flushPromises();
    expect(pane.text()).not.toContain('Kf3xQ9prTnWbMvJdRhZs');
    expect(calls.some(([name]) => name === 'tunnelAuthReveal')).toBe(false);

    await pane.findAll('button').find((b) => b.text() === en.tunnel.authShow).trigger('click');
    await flushPromises();
    expect(pane.text()).toContain('Kf3xQ9prTnWbMvJdRhZs');
  });

  /**
   * A machine with no keystore is told so instead of being offered a switch
   * that would fail when pressed.
   */
  it('offers no switch when there is nowhere to keep a password', async () => {
    replies.tunnelIdentity = { authUser: null, keystore: false, reserved: {} };
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain(en.tunnel.authNoKeystore);
    expect(pane.findAll('button').some((b) => b.text() === en.tunnel.authOn)).toBe(false);
  });

  it('asks Rust to generate the password rather than sending one', async () => {
    const pane = mountPane();
    await flushPromises();

    await pane.findAll('button').find((b) => b.text() === en.tunnel.authOn).trigger('click');
    await flushPromises();

    // An empty password is the request to generate one; the pane never invents
    // a credential of its own.
    expect(calls).toContainEqual(['tunnelAuthSet', 'shop', { user: '', password: '' }]);
  });
});

describe('the address', () => {
  /**
   * Measured on a real provider: the tunnel is up, the pane is green, and the
   * address somebody registered in a dashboard points nowhere.
   */
  it('says when the provider did not give the name that was asked for', async () => {
    replies.tunnelStatus = [
      {
        ...OPEN,
        provider: 'localtunnel',
        url: 'https://bitter-bulldog-88.loca.lt',
        reserved: 'shop-dev',
        reservedHonoured: false,
      },
    ];
    const pane = mountPane();
    await flushPromises();

    expect(pane.text()).toContain('shop-dev');
    expect(pane.text()).toContain('did not give this tunnel the address it asked for');
    // And the address that was actually granted is still the one on offer.
    expect(pane.text()).toContain('bitter-bulldog-88.loca.lt');
  });

  it('is silent about a name that was granted', async () => {
    replies.tunnelStatus = [
      {
        ...OPEN,
        provider: 'localtunnel',
        url: 'https://shop-dev.loca.lt',
        reserved: 'shop-dev',
        reservedHonoured: true,
      },
    ];
    const pane = mountPane();
    await flushPromises();
    expect(pane.text()).not.toContain('did not give this tunnel the address it asked for');
  });

  /**
   * Three of the nine hand out a new address every time, and the field is not
   * offered for them — a name stored for a provider that ignores it would look
   * like configuration that works.
   */
  it('offers a name field only for a provider that can keep one', async () => {
    const pane = mountPane();
    await flushPromises();
    expect(pane.text()).toContain(en.tunnel.reservedNone);

    await pane.findComponent({ name: 'VSelect' }).setValue('localtunnel');
    await flushPromises();
    expect(pane.text()).not.toContain(en.tunnel.reservedNone);
    expect(pane.text()).toContain(en.tunnel.reservedNote.localtunnel);
  });

  it('stores the name against the provider it was typed for', async () => {
    replies.tunnelIdentity = { authUser: null, keystore: true, reserved: {} };
    const pane = mountPane();
    await flushPromises();

    await pane.findComponent({ name: 'VSelect' }).setValue('localtunnel');
    await flushPromises();

    const field = pane
      .findAllComponents({ name: 'VTextField' })
      .find((f) => f.props('label') === en.tunnel.reservedKind.subdomain);
    await field.setValue('shop-dev');
    await pane
      .findAll('button')
      .find((b) => b.text() === en.tunnel.reservedSave)
      .trigger('click');
    await flushPromises();

    expect(calls).toContainEqual(['tunnelNameSet', 'shop', 'localtunnel', 'shop-dev']);
  });
});
