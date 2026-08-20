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
 */

/** How long to wait between polls, and how many times. */
export const TUNNEL_POLL_MS = 1500;
const TUNNEL_POLL_TRIES = 20;

export function useTunnel(name) {
  const tunnel = ref(null);
  const providers = ref([]);
  const busy = ref(false);
  const error = ref(null);
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
    busy,
    error,
    load,
    loadProviders,
    start,
    stop,
    saveToken,
  };
}
