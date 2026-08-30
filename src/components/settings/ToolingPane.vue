<script setup>
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import SettingsGroup from '@/components/SettingsGroup.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';
import { useCopyTick } from '@/composables/useCopyTick';
import { toolingOwnAbout, toolingWhy } from '@/lib/catalogue-text';

/**
 * The commands this app puts on `PATH`, and the host tools it needs.
 *
 * Three groups because the page answers three questions, and they have
 * different answers: where `stackvo` is installed from, whether a shell can
 * find it, and what this machine is missing. The rival pages this is measured
 * against put all of it in one list of Install buttons — which works for them,
 * because for them every row is the same kind of thing.
 *
 * It is not here. `mkcert` is a binary this app can fetch; Docker is an
 * application with an installer and a virtual machine behind it; `stackvo` is
 * a program this repository builds. A single Install column would have to be
 * disabled on three rows out of four, and a disabled button says "later"
 * where the honest answer is "not by this".
 *
 * What is deliberately *not* here: `composer`, `node`, `npm`, `wp`. They run
 * in the project's container at the version the project declared, and a copy
 * on the host would be a second answer to "which one runs" whose answer is
 * wrong. `src-tauri/src/tooling.rs` makes the argument in full.
 */
const { t } = useI18n();
const { copied, copy } = useCopyTick();

const status = ref({
  binDir: null,
  onPath: false,
  currentShell: null,
  own: [],
  shells: [],
  tools: [],
});
const error = ref(null);
const loading = ref(false);
const busy = ref(null);

/** Both commands, or neither: a checkout with nothing built is a real state. */
const built = computed(() => status.value.own.some((row) => row.built));
const linked = computed(() => status.value.own.some((row) => row.linked));

/**
 * Shells with a startup file first, then the rest.
 *
 * Absent ones stay on the list rather than being hidden — "fish is not set up
 * on this machine" is an answer, and a row that vanishes reads as a bug. Same
 * reasoning as the assistants pane.
 */
const shells = computed(() =>
  [...status.value.shells].sort((a, b) => Number(b.exists) - Number(a.exists))
);

/** Written, written by an older data directory, or not written. */
function shellState(row) {
  if (!row.installed) return row.exists ? 'absent' : 'noFile';
  return row.current ? 'installed' : 'stale';
}

/** Can this app fetch this one, on this platform? */
function installable(tool) {
  return Boolean(tool.offers) && tool.availableHere;
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    const next = await api.toolingStatus();
    status.value = {
      ...next,
      // `asList`, not the value: a command that answers with nothing must not
      // leave the computeds above iterating `undefined`.
      own: asList(next?.own),
      shells: asList(next?.shells),
      tools: asList(next?.tools),
    };
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}

/**
 * Re-read first, then report.
 *
 * The order every pane in this directory settled on: a failed write may still
 * have changed something, and a row showing the old state would be a claim
 * nobody checked. `load()` clears the error on the way in, so it is set after.
 */
async function act(key, fn) {
  busy.value = key;
  error.value = null;
  try {
    await fn();
    await load();
  } catch (e) {
    await load();
    error.value = e;
  } finally {
    busy.value = null;
  }
}

const addPath = (row) => act(`shell:${row.id}`, () => api.toolingPathApply(row.id));
const removePath = (row) => act(`shell:${row.id}`, () => api.toolingPathRemove(row.id));
const installTool = (tool) => act(`tool:${tool.id}`, () => api.toolingInstall(tool.id));
const removeTool = (tool) => act(`tool:${tool.id}`, () => api.toolingRemove(tool.id));

onMounted(load);

defineExpose({ shellState, installable });
</script>

<template>
  <ErrorAlert v-if="error" :error="error" class="mb-4" />

  <SettingsGroup
    help="settings-tooling"
    icon="mdi-console"
    :title="t('settings.tooling.commands.title')"
    :description="t('settings.tooling.commands.description')"
  >
    <v-alert type="info" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.tooling.commands.whatItDoes') }}</div>
      <div class="text-caption mt-2 text-medium-emphasis">
        {{ t('settings.tooling.commands.notShims') }}
      </div>
    </v-alert>

    <!-- Not a warning about a failure: neither binary is bundled with the app
         and never has been. What would be a failure is a PATH entry pointing at
         a directory with nothing in it. -->
    <v-alert
      v-if="!built && !loading"
      type="warning"
      variant="tonal"
      density="comfortable"
      class="mb-4"
    >
      <div class="text-body-2">{{ t('settings.tooling.commands.noBinary') }}</div>
      <code class="d-block mt-2 text-caption">{{
        t('settings.tooling.commands.buildCommand')
      }}</code>
      <v-btn
        size="small"
        variant="text"
        class="mt-1"
        :prepend-icon="copied === 'build' ? 'mdi-check' : 'mdi-content-copy'"
        @click="copy(t('settings.tooling.commands.buildCommand'), 'build')"
      >
        {{ t('app.copy') }}
      </v-btn>
    </v-alert>

    <div class="mb-4">
      <div class="text-caption text-medium-emphasis">{{ t('settings.tooling.binDir') }}</div>
      <code class="text-caption">{{ status.binDir ?? '—' }}</code>
    </div>

    <!-- The sentence nothing else says. A block written into .zshrc reaches the
         *next* shell; a user looking at a terminal opened an hour ago needs to
         be told that rather than left to conclude the button did nothing. -->
    <v-alert
      v-if="linked && !status.onPath"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-4"
    >
      <div class="text-caption">{{ t('settings.tooling.openANewShell') }}</div>
    </v-alert>

    <v-progress-linear v-if="loading" indeterminate class="mb-2" />

    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="row in status.own" :key="row.id" class="px-0">
        <template #prepend>
          <v-icon
            :icon="row.linked ? 'mdi-check-circle-outline' : 'mdi-application-outline'"
            :color="row.linked ? 'success' : undefined"
            class="mr-3"
          />
        </template>
        <v-list-item-title class="text-body-2">
          <code>{{ row.id }}</code>
        </v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ toolingOwnAbout(row) }} —
          <code>{{ row.built ?? t('settings.tooling.commands.notBuilt') }}</code>
        </v-list-item-subtitle>
      </v-list-item>
    </v-list>
  </SettingsGroup>

  <!-- The second group. Same help topic on purpose: a reader here has the same
       question as a reader above, one step later. -->
  <SettingsGroup
    help="settings-tooling"
    icon="mdi-console-line"
    :title="t('settings.tooling.shells.title')"
    :description="t('settings.tooling.shells.description')"
    class="mt-6"
  >
    <v-alert type="info" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.tooling.shells.whatItDoes') }}</div>
      <div class="text-caption mt-2 text-medium-emphasis">
        {{ t('settings.tooling.shells.markers') }}
      </div>
    </v-alert>

    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="row in shells" :key="row.id" class="px-0">
        <template #prepend>
          <v-icon
            :icon="row.installed ? 'mdi-check-circle-outline' : 'mdi-file-code-outline'"
            :color="{ installed: 'success', stale: 'warning' }[shellState(row)]"
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2">
          {{ row.label }}
          <v-chip v-if="row.id === status.currentShell" size="x-small" variant="tonal" class="ml-2">
            {{ t('settings.tooling.shells.yours') }}
          </v-chip>
        </v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ t(`settings.tooling.shells.state.${shellState(row)}`) }} — <code>{{ row.path }}</code>
        </v-list-item-subtitle>

        <template #append>
          <!-- The line itself, always available. It is what somebody needs when
               they keep their startup files in a repository this app has no
               business editing. -->
          <v-btn
            v-if="row.line"
            size="small"
            variant="text"
            :prepend-icon="copied === row.id ? 'mdi-check' : 'mdi-content-copy'"
            @click="copy(row.line, row.id)"
          >
            {{ t('settings.tooling.shells.copyLine') }}
          </v-btn>
          <v-btn
            v-if="row.installed"
            size="small"
            variant="text"
            class="ml-2"
            :loading="busy === `shell:${row.id}`"
            :disabled="busy !== null"
            @click="removePath(row)"
          >
            {{ t('settings.tooling.remove') }}
          </v-btn>
          <v-btn
            v-if="!row.current"
            size="small"
            variant="tonal"
            color="primary"
            class="ml-2"
            :loading="busy === `shell:${row.id}`"
            :disabled="busy !== null"
            @click="addPath(row)"
          >
            {{
              shellState(row) === 'stale'
                ? t('settings.tooling.update')
                : t('settings.tooling.shells.add')
            }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>
  </SettingsGroup>

  <SettingsGroup
    help="settings-tooling"
    icon="mdi-toolbox-outline"
    :title="t('settings.tooling.tools.title')"
    :description="t('settings.tooling.tools.description')"
    class="mt-6"
  >
    <v-alert type="info" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.tooling.tools.whatItDoes') }}</div>
      <div class="text-caption mt-2 text-medium-emphasis">
        {{ t('settings.tooling.tools.inTheContainer') }}
      </div>
    </v-alert>

    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="tool in status.tools" :key="tool.id" class="px-0">
        <template #prepend>
          <v-icon
            :icon="
              tool.source === 'missing' ? 'mdi-alert-circle-outline' : 'mdi-check-circle-outline'
            "
            :color="{ managed: 'success', system: 'success', missing: 'warning' }[tool.source]"
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2">
          {{ tool.label }}
          <!-- The badge Yerd calls "External". Named for what it means here:
               this copy is the user's and this app will not touch it. -->
          <v-chip v-if="tool.source === 'system'" size="x-small" variant="tonal" class="ml-2">
            {{ t('settings.tooling.tools.yours') }}
          </v-chip>
          <v-chip
            v-else-if="tool.source === 'managed'"
            size="x-small"
            variant="tonal"
            color="primary"
            class="ml-2"
          >
            {{ t('settings.tooling.tools.managed') }}
          </v-chip>
          <span v-if="tool.version" class="text-caption text-medium-emphasis ml-2">
            {{ tool.version }}
          </span>
        </v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ toolingWhy(tool) }}
        </v-list-item-subtitle>

        <template #append>
          <!-- A tool this app cannot fetch says so, once, instead of carrying a
               permanently disabled button. -->
          <span v-if="!installable(tool)" class="text-caption text-medium-emphasis">
            {{
              tool.offers
                ? t('settings.tooling.tools.noBuildHere')
                : t('settings.tooling.tools.ownInstaller')
            }}
          </span>
          <v-btn
            v-else-if="tool.source === 'managed'"
            size="small"
            variant="text"
            :loading="busy === `tool:${tool.id}`"
            :disabled="busy !== null"
            @click="removeTool(tool)"
          >
            {{ t('settings.tooling.remove') }}
          </v-btn>
          <v-btn
            v-else
            size="small"
            variant="tonal"
            color="primary"
            :loading="busy === `tool:${tool.id}`"
            :disabled="busy !== null"
            @click="installTool(tool)"
          >
            {{ t('settings.tooling.tools.install', { version: tool.offers }) }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <div class="text-caption text-medium-emphasis mt-4">
      {{ t('settings.tooling.tools.pinned') }}
    </div>
  </SettingsGroup>
</template>
