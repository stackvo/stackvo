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
 * There is no filter box. The trail is short on a normal machine, `total` says
 * when it is not, and a filter over a record is a way to look at a subset and
 * believe it is the whole — which is the one thing a record must not invite.
 *
 * The one thing it does besides read is **put an act back**, and only where the
 * line carries a plan. That plan was written before the act ran — what
 * `stackvo_stack_down` stopped exists only before it stopped it — so the button
 * runs what was true at the time rather than something worked out now against a
 * machine that has changed. Where there is no plan the line says why in its own
 * words, which is the half that keeps the button honest: an Undo on every row
 * would be an offer the app cannot keep.
 */
const { t } = useI18n();

const trail = ref(null);
const error = ref(null);
const loading = ref(true);
/** The `at` of the line being put back — one at a time, and the row says so. */
const undoing = ref(null);

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

/**
 * Put one act back, then re-read.
 *
 * Re-read whichever way it went, and *before* the error is shown — the same
 * order AgentsPane settled on for the same reason: a partly-completed undo
 * changed the machine, so a screen still showing the old state would be a claim
 * nobody checked. `load()` clears the error on its way in, so it is set after.
 */
async function undo(entry) {
  undoing.value = entry.at;
  error.value = null;
  try {
    await api.auditUndo(entry.at);
    await load();
  } catch (e) {
    await load();
    error.value = e;
  } finally {
    undoing.value = null;
  }
}

/** The plan, as the calls it will make. */
function steps(entry) {
  return (entry.undo?.steps ?? []).map((s) => s.tool.replace(/^stackvo_/, '')).join(', ');
}

const undoable = (entry) => entry.undo?.kind === 'steps' && !entry.undone;

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
    icon="mdi-clipboard-text-clock-outline"
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

      <!-- The trail scrolls inside the card rather than lengthening it.
           `total` is capped on the Rust side, but the cap is a tail of the
           file and not a screenful: fifty entries made this the one card on
           the page taller than the window, so the two sentences under the
           list — the one naming the assistant, and the one saying this is a
           tail rather than the history — were below the fold on exactly the
           pane whose whole point is that you can see what was done. -->
      <div class="audit-trail">
        <v-list density="compact" class="pa-0 bg-transparent">
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
              <!-- Why there is no button, in the words the plan recorded. A row
                 that simply had no button would read as an app that had not
                 thought about it. -->
              <span v-if="entry.undo?.kind === 'none'">
                · {{ t('audit.noUndo', { because: entry.undo.because }) }}
              </span>
              <span v-else-if="undoable(entry)">
                ·
                {{ t('audit.undoSteps', { count: entry.undo.steps.length, steps: steps(entry) }) }}
              </span>
            </v-list-item-subtitle>

            <template #append>
              <!-- The append-only join: the original line still says what it
                 said, and the undo that names it is what makes this chip
                 true. -->
              <v-chip v-if="entry.undone" size="x-small" variant="tonal" color="success">
                {{ t('audit.undone') }}
              </v-chip>
              <v-btn
                v-else-if="undoable(entry)"
                size="small"
                variant="tonal"
                prepend-icon="mdi-undo-variant"
                :loading="undoing === entry.at"
                :disabled="undoing !== null && undoing !== entry.at"
                @click="undo(entry)"
              >
                {{ t('audit.undo') }}
              </v-btn>
            </template>
          </v-list-item>
        </v-list>
      </div>

      <p class="text-caption text-medium-emphasis mt-3">{{ t('audit.assistant') }}</p>

      <p v-if="truncated" class="text-caption text-medium-emphasis mt-3">
        {{ t('audit.truncated', { shown: entries.length, total: trail.total }) }}
      </p>
    </template>
  </SettingsGroup>
</template>

<style scoped>
/* Tall enough to be a list rather than a peephole, short enough that the card
   still ends inside the window on a laptop. `vh` rather than a pixel count for
   the same reason `.env-table` next door uses one: the pane is measured
   against the screen it is read on, not against a number someone typed. */
.audit-trail {
  max-height: 52vh;
  overflow-y: auto;
}
</style>
