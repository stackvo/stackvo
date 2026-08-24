import { computed, ref } from 'vue';
import { api, asList } from '@/lib/ipc';

/**
 * This project's tunnel sidecar, when one exists, and the providers it could
 * have been opened through.
 *
 * The URL is the provider's to assign and arrives seconds after the sidecar
 * starts, so starting polls until the status call can read it out of the
 * sidecar's log — the same place it is read from after an app restart.
 *
 * Lifted out of `ProjectDetail.vue` with the Tunnel pane under §14.16, and
 * given the provider table when the pane stopped being cloudflared's.
 *
 * B-7 added the two things a link is asked about the moment it is shared: who
 * else can open it, and whether it will still be this address tomorrow. Both
 * are state of the *project* rather than of a running tunnel, so they load
 * beside the status rather than out of it — and `tunnel.guarded` is still read
 * off the running sidecar, because turning authentication on does not protect
 * a link handed out before it.
 */

/** How long to wait between polls, and how many times. */
export const TUNNEL_POLL_MS = 1500;
const TUNNEL_POLL_TRIES = 20;

export function useTunnel(name) {
  const tunnel = ref(null);
  const providers = ref([]);
  const busy = ref(false);
  const error = ref(null);
  /** Who may open this project's tunnel, and what its address is called. */
  const identity = ref(null);
  /** The password, only once somebody has asked to see it. */
  const revealed = ref(null);
  /** Which provider the next start uses. Seeded from the table, not hard-coded
   *  here: "the default" is a fact about the providers and Rust owns them. */
  const chosen = ref('cloudflare');

  const provider = computed(() => providers.value.find((p) => p.id === chosen.value) ?? null);

  /** A provider that needs a token and has none cannot be started, and the
   *  pane says so before the button is pressed rather than after a pull. */
  const needsToken = computed(() => !!provider.value?.tokenEnv && !provider.value?.hasToken);

  /**
   * The status call answers for every project at once; only this one's row
   * matters, and its absence is "no tunnel" rather than a failure.
   */
  async function load() {
    try {
      const all = await api.tunnelStatus();
      tunnel.value = all.find((t) => t.project === name.value) ?? null;
      // A running tunnel decides the selection: the picker must show what is
      // actually connected, not what somebody chose before it was.
      if (tunnel.value?.provider) chosen.value = tunnel.value.provider;
    } catch {
      tunnel.value = null;
    }
    return tunnel.value;
  }

  /** Whether this link asks for a password at all. */
  const authenticated = computed(() => !!identity.value?.authUser);

  /** The name the chosen provider has been asked to keep, if any. */
  const reservedName = computed(() => identity.value?.reserved?.[chosen.value] ?? '');

  /**
   * The identity, which is two questions and no password.
   *
   * Absent is not a failure: a machine with no keystore answers `keystore:
   * false` and the pane says so rather than offering a switch that would fail
   * when pressed.
   */
  async function loadIdentity() {
    try {
      identity.value = await api.tunnelIdentity(name.value);
    } catch {
      identity.value = null;
    }
    // A password shown for one project must not still be on screen after the
    // pane moves to another.
    revealed.value = null;
    return identity.value;
  }

  /**
   * Turn authentication on, generating the password when none was typed, or
   * off with `null`.
   */
  async function saveAuth(credentials) {
    busy.value = true;
    error.value = null;
    try {
      const now = await api.tunnelAuthSet(name.value, credentials);
      await loadIdentity();
      // What was just set is worth showing: it is the thing that has to be
      // handed to somebody along with the link.
      revealed.value = now;
      return now;
    } catch (e) {
      error.value = e;
      return null;
    } finally {
      busy.value = false;
    }
  }

  /** Ask for the password, deliberately and one project at a time. */
  async function reveal() {
    error.value = null;
    try {
      revealed.value = await api.tunnelAuthReveal(name.value);
    } catch (e) {
      error.value = e;
    }
    return revealed.value;
  }

  /** Remember the address this provider should keep, or forget it. */
  async function saveName(reserved) {
    busy.value = true;
    error.value = null;
    try {
      await api.tunnelNameSet(name.value, chosen.value, reserved || null);
      await loadIdentity();
      return true;
    } catch (e) {
      error.value = e;
      return false;
    } finally {
      busy.value = false;
    }
  }

  /** The table, and with it which providers this machine has a token for. */
  async function loadProviders() {
    try {
      // `asList` and not the value: a command a stub does not know answers
      // `null`, and a picker built from `null.map` takes the whole pane down.
      providers.value = asList(await api.tunnelProviders());
    } catch {
      providers.value = [];
    }
    return providers.value;
  }

  async function start() {
    busy.value = true;
    error.value = null;
    try {
      await api.tunnelStart(name.value, chosen.value);
      for (let i = 0; i < TUNNEL_POLL_TRIES; i++) {
        await new Promise((r) => setTimeout(r, TUNNEL_POLL_MS));
        await load();
        // A failure ends the wait as surely as a URL does: four of these
        // providers can be refused, and polling twenty times against a
        // container that already exited says "connecting" for half a minute
        // about something that is never going to connect.
        if (tunnel.value?.url || tunnel.value?.failure) break;
      }
      return tunnel.value;
    } catch (e) {
      error.value = e;
      return null;
    } finally {
      // Cleared even when the poll ran out: the sidecar is up either way, and
      // a button left spinning would say otherwise.
      busy.value = false;
    }
  }

  async function stop() {
    busy.value = true;
    error.value = null;
    try {
      await api.tunnelStop(name.value);
      await load();
      return true;
    } catch (e) {
      error.value = e;
      return false;
    } finally {
      busy.value = false;
    }
  }

  /**
   * Store a provider's token, or clear it with `null`.
   *
   * Reloads the table rather than assuming: `hasToken` is the keystore's
   * answer, and a keychain that refused the write is exactly the case where
   * believing the optimistic one would be wrong.
   */
  async function saveToken(providerId, token) {
    busy.value = true;
    error.value = null;
    try {
      await api.tunnelTokenSet(providerId, token);
      await loadProviders();
      return true;
    } catch (e) {
      error.value = e;
      return false;
    } finally {
      busy.value = false;
    }
  }

  return {
    tunnel,
    providers,
    provider,
    chosen,
    needsToken,
    identity,
    authenticated,
    revealed,
    reservedName,
    loadIdentity,
    saveAuth,
    reveal,
    saveName,
    busy,
    error,
    load,
    loadProviders,
    start,
    stop,
    saveToken,
  };
}
