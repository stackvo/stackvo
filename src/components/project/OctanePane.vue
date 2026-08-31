<script setup>
import { onBeforeUnmount, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { listen } from '@tauri-apps/api/event';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * F-5 — reloading Octane, without adding a watcher to the image.
 *
 * Octane boots the application once and keeps it in memory, so an edited file
 * changes nothing until the workers are replaced. Laravel's answer is
 * `octane:start --watch` and its price is Node and chokidar inside the image;
 * this app already watches the host filesystem, so the answer here is one
 * action bound to a watcher that is already running.
 *
 * The switch is **off by default** and this pane does not soften that: a reload
 * arriving mid-request kills that request, and the sentence saying so sits on
 * the switch rather than in the help document.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const status = ref(null);
const error = ref(null);
const loading = ref(false);
const busy = ref(null);
/** The last reload the watcher did, so an automatic one is visible. */
const last = ref(null);

let stop = null;

async function load() {
  loading.value = true;
  error.value = null;
  try {
    status.value = await api.octaneStatus(props.name);
  } catch (e) {
    status.value = null;
    error.value = e;
  } finally {
    loading.value = false;
  }
}

async function setAuto(enabled) {
  busy.value = 'auto';
  error.value = null;
  try {
    status.value = await api.octaneAutoReload(props.name, enabled);
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = null;
  }
}

async function reloadNow() {
  busy.value = 'reload';
  error.value = null;
  try {
    await api.octaneReload(props.name);
    last.value = { ok: true, at: new Date().toLocaleTimeString() };
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = null;
  }
}

// The watcher's reloads happen whether or not this pane is open; while it is,
// they show up here. A failed one is shown too — a reload that silently did
// not happen is the same experience as the bug this removes.
listen('octane:reloaded', ({ payload }) => {
  if (payload?.project !== props.name) return;
  last.value = { ok: payload.ok, at: new Date().toLocaleTimeString(), auto: true };
}).then((off) => {
  stop = off;
});

onBeforeUnmount(() => stop?.());

watch(
  () => props.name,
  () => {
    status.value = null;
    error.value = null;
    last.value = null;
  }
);
</script>

<template>
  <section class="pane">
    <PaneHeader
      help="project-octane"
      icon="mdi-rocket-launch-outline"
      :title="t('octane.title')"
      :description="t('octane.desc')"
    />

    <v-btn
      size="small"
      variant="tonal"
      prepend-icon="mdi-magnify"
      :loading="loading"
      data-test="octane-read"
      @click="load"
    >
      {{ t('octane.read') }}
    </v-btn>

    <ErrorAlert v-if="error" :error="error" class="mt-3" />

    <template v-if="status">
      <!-- PHP-FPM reads the file on every request. There is nothing to reload,
           and a button here would be a no-op with a story. -->
      <v-alert
        v-if="!status.octane"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="octane-not"
      >
        {{ t('octane.notOctane', { server: status.server }) }}
      </v-alert>

      <template v-else>
        <p class="text-caption text-medium-emphasis mt-3 mb-2">
          {{ t('octane.isOctane', { server: status.server }) }}
        </p>

        <v-switch
          :model-value="status.autoReload"
          color="primary"
          density="compact"
          hide-details
          :loading="busy === 'auto'"
          data-test="octane-auto"
          @update:model-value="setAuto($event)"
        >
          <template #label>
            <span class="text-body-2">{{ t('octane.auto') }}</span>
          </template>
        </v-switch>
        <p class="text-caption text-medium-emphasis mt-1 mb-1">{{ t('octane.autoCost') }}</p>
        <p class="text-caption text-medium-emphasis mb-3">
          {{ t('octane.watched') }}
          <span v-for="(path, i) in status.watched" :key="path">
            <template v-if="i">, </template><code>{{ path }}</code>
          </span>
        </p>

        <v-btn
          size="x-small"
          variant="tonal"
          prepend-icon="mdi-refresh"
          :loading="busy === 'reload'"
          data-test="octane-now"
          @click="reloadNow"
        >
          {{ t('octane.now') }}
        </v-btn>

        <p v-if="last" class="text-caption mt-2" data-test="octane-last">
          <v-icon :color="last.ok ? 'success' : 'error'" size="16" class="mr-2">
            {{ last.ok ? 'mdi-check-circle-outline' : 'mdi-alert-circle-outline' }}
          </v-icon>
          {{
            last.ok
              ? t(last.auto ? 'octane.lastAuto' : 'octane.lastManual', { at: last.at })
              : t('octane.lastFailed', { at: last.at })
          }}
        </p>

        <p class="text-caption text-medium-emphasis mt-3">{{ t('octane.notWatch') }}</p>
      </template>
    </template>
  </section>
</template>
