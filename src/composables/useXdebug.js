import { computed, ref } from 'vue';
import { api } from '@/lib/ipc';

/**
 * Xdebug, across its three layers: configured in the manifest, compiled into
 * the image, and present in the running container.
 *
 * Loaded with the project rather than when the section opens, because the state
 * worth badging in the rail — enabled but never rebuilt, so nothing happens
 * when you set a breakpoint — is precisely the one a user will not go looking
 * for.
 *
 * Lifted out of `ProjectDetail.vue` with the Xdebug pane under §14.16.
 */
export function useXdebug(name) {
  const status = ref(null);
  const busy = ref(false);
  const error = ref(null);

  /** Enabled, but not yet doing anything — the state that needs saying out loud. */
  const pending = computed(
    () => status.value?.enabled && (status.value.needsRebuild || status.value.active === false)
  );

  async function load(runtime) {
    // Node projects have no PHP extension to report on, and the pane is not in
    // the rail for them either.
    if (runtime !== 'php') {
      status.value = null;
      return null;
    }
    try {
      status.value = await api.xdebugStatus(name.value);
      error.value = null;
    } catch (e) {
      // A failed *refresh* keeps what is on screen. The whole pane hangs off
      // `v-if="status"`, so blanking it empties the switch, the warnings and
      // the IDE list — and the refreshes happen exactly when the engine is
      // busiest, because this now re-reads as a container is recreated. Only
      // the first read has nothing to fall back to.
      if (!status.value) return null;
      error.value = e;
    }
    return status.value;
  }

  async function toggle(enabled) {
    busy.value = true;
    error.value = null;
    try {
      status.value = await api.xdebugSet(name.value, enabled);
      return status.value;
    } catch (e) {
      error.value = e;
      return null;
    } finally {
      busy.value = false;
    }
  }

  return { status, busy, error, pending, load, toggle };
}
