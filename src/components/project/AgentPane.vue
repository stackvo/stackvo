<script setup>
import { onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * The two files this app writes into *this* project for an assistant.
 *
 * ## Why it is here and not only in Settings
 *
 * Settings → AI rules can write these too, and had to be able to: the global
 * copies live in a home directory and belong to no project. But the workspace
 * copy is a file **in this repository**, and asking somebody to leave the page
 * they are looking at, find the project again in a dropdown and press a button
 * there is asking them to hold a name in their head for no reason. The rules
 * are per project; this is where a project is.
 *
 * Both places drive the same three commands, so neither is a copy of the other
 * — there is one implementation and two doors to it. The scope is fixed to
 * `workspace` here, because "on this machine" is not a fact about this project
 * and a global row on a project page would be a button that changes something
 * the page does not show.
 *
 * ## The context file is reported, not offered
 *
 * `.stackvo/context.json` is written by the generator for every project, on
 * every generate — it is not something to switch on, and a button here would
 * imply it could be off. So it is named and explained, and the honest thing to
 * say about it is where it is and when it is refreshed.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: null },
});

const { t } = useI18n();

const rows = ref([]);
const error = ref(null);
const loading = ref(false);
const busy = ref(null);

/** Absent, current, or written by an older release. */
function state(row) {
  if (!row.installed) return 'absent';
  return row.current ? 'installed' : 'stale';
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    // Only the workspace half. The global rows are about the machine and are
    // Settings' question, not this page's.
    rows.value = asList(await api.rulesStatus(props.name)).filter((r) => r.scope === 'workspace');
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}

async function apply(row) {
  busy.value = row.id;
  error.value = null;
  try {
    await api.rulesApply(row.id, 'workspace', props.name);
    await load();
  } catch (e) {
    // Re-read first, then report: a failed write may still have changed the
    // file, and a row describing the old state would be a claim nobody checked.
    await load();
    error.value = e;
  } finally {
    busy.value = null;
  }
}

async function remove(row) {
  busy.value = row.id;
  error.value = null;
  try {
    await api.rulesRemove(row.id, 'workspace', props.name);
    await load();
  } catch (e) {
    await load();
    error.value = e;
  } finally {
    busy.value = null;
  }
}

onMounted(load);
watch(() => props.name, load);

defineExpose({ state });
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-agent"
      icon="mdi-robot-outline"
      :title="t('projectAgent.title')"
      :description="t('projectAgent.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <v-alert type="info" variant="tonal" density="compact" class="mb-3">
      <div class="text-caption">{{ t('projectAgent.markers') }}</div>
    </v-alert>

    <v-progress-linear v-if="loading" indeterminate class="mb-2" />

    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="row in rows" :key="row.id" class="px-0">
        <template #prepend>
          <v-icon
            :icon="row.installed ? 'mdi-check-circle-outline' : 'mdi-file-document-outline'"
            :color="{ installed: 'success', stale: 'warning' }[state(row)]"
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2">{{ row.label }}</v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ t(`settings.agents.rules.state.${state(row)}`) }} — <code>{{ row.path }}</code>
        </v-list-item-subtitle>

        <template #append>
          <v-btn
            v-if="row.installed"
            size="small"
            variant="text"
            :loading="busy === row.id"
            :disabled="busy !== null"
            @click="remove(row)"
          >
            {{ t('settings.agents.remove') }}
          </v-btn>
          <v-btn
            v-if="!row.current"
            size="small"
            variant="tonal"
            color="primary"
            class="ml-2"
            :loading="busy === row.id"
            :disabled="busy !== null"
            @click="apply(row)"
          >
            {{
              state(row) === 'stale' ? t('settings.agents.update') : t('settings.agents.rules.add')
            }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <!-- Reported rather than offered: the generator writes it for every
         project on every run, so a switch here would imply it could be off. -->
    <v-divider class="my-3" />
    <div class="text-subtitle-2">{{ t('projectAgent.contextTitle') }}</div>
    <div class="text-caption text-medium-emphasis mt-1">
      {{ t('projectAgent.contextBody') }}
    </div>
    <code class="d-block text-caption mt-1">.stackvo/context.json</code>
    <div v-if="runtime && runtime !== 'php'" class="text-caption text-medium-emphasis mt-2">
      {{ t('projectAgent.contextNoMount') }}
    </div>

    <div class="text-caption text-medium-emphasis mt-4">
      {{ t('projectAgent.serverElsewhere') }}
    </div>
  </v-card>
</template>
