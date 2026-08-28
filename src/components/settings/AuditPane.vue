<script setup>
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import SettingsGroup from '@/components/SettingsGroup.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';

/**
 * The record of what cannot be taken back.
 *
 * `audit.rs` has been writing this file from eighteen call sites since it was
 * added and nothing has ever read it: there was no command, so the only way to
 * see the trail was to know it is JSON Lines and know which directory the logs
 * go in. The module states its audience as "whoever has to account for the
 * machine", and that person is usually not the one who wrote the file.
 *
 * Read-only on purpose, and there is no filter box. The trail is short on a
 * normal machine, `total` says when it is not, and a filter over a record is a
 * way to look at a subset and believe it is the whole — which is the one thing
 * a record must not invite.
 */
const { t } = useI18n();

const trail = ref(null);
const error = ref(null);
const loading = ref(true);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    trail.value = await api.auditTrail();
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const entries = computed(() => trail.value?.entries ?? []);

// `total` is every line in the file; `entries` is the capped tail. A screen
// showing fifty of nine thousand has to say so or it reads as the history.
const truncated = computed(() => (trail.value?.total ?? 0) > entries.value.length);

const ICONS = {
  ok: 'mdi-check-circle-outline',
  refused: 'mdi-cancel',
  failed: 'mdi-alert-circle-outline',
};
const COLOURS = { ok: 'success', refused: 'warning', failed: 'error' };

/**
 * The timestamp is RFC 3339 UTC; show it where the reader is.
 *
 * `toLocaleString`, which is what every other pane in this app uses, rather
 * than a named `$d` format — there is no such format registered and asking for
 * one is a runtime miss that renders as the key. A line the parser cannot read
 * is shown as written rather than as `Invalid Date`: this is a record, and the
 * raw value is the evidence.
 */
function when(at) {
  const parsed = new Date(at);
  return Number.isNaN(parsed.getTime()) ? at : parsed.toLocaleString();
}
</script>

<template>
  <SettingsGroup
    help="settings-audit"
    :title="t('audit.title')"
    :description="t('audit.description')"
  >
    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <v-skeleton-loader v-if="loading" type="list-item-two-line@3" />

    <!-- Nothing irreversible having been done yet is the normal state of a new
         workspace, so an empty trail is a sentence rather than an empty box.
         Guarded on `error` as well as on `loading`: a trail that could not be
         read has no entries either, and saying "nothing has been done" to
         somebody whose log directory is missing is the one wrong answer here.
         "I could not look" and "there is nothing" are different sentences. -->
    <v-alert v-else-if="!error && !entries.length" type="info" variant="tonal" density="compact">
      {{ t('audit.empty') }}
    </v-alert>

    <template v-else-if="entries.length">
      <!-- Damage in the file is itself something the person reading a record
           needs to be told, rather than a quietly shorter list. -->
      <v-alert
        v-if="trail.unreadable"
        type="warning"
        variant="tonal"
        density="compact"
        class="mb-3"
      >
        {{ t('audit.unreadable', { count: trail.unreadable }) }}
      </v-alert>

      <v-list density="compact" class="pa-0">
        <v-list-item v-for="(entry, i) in entries" :key="`${entry.at}-${i}`" class="px-0">
          <template #prepend>
            <v-icon :color="COLOURS[entry.outcome]" size="18">
              {{ ICONS[entry.outcome] ?? 'mdi-circle-small' }}
            </v-icon>
          </template>
          <v-list-item-title class="text-body-2">
            <code>{{ entry.action }}</code>
            <span class="text-medium-emphasis"> — {{ entry.subject }}</span>
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption">
            {{ when(entry.at) }}
            <span v-if="entry.detail"> · {{ entry.detail }}</span>
          </v-list-item-subtitle>
        </v-list-item>
      </v-list>

      <p v-if="truncated" class="text-caption text-medium-emphasis mt-3">
        {{ t('audit.truncated', { shown: entries.length, total: trail.total }) }}
      </p>
    </template>
  </SettingsGroup>
</template>
