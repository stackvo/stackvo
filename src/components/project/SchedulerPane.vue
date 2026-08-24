<script setup>
import { computed, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  KINDS,
  PRESETS,
  argvFor,
  kindOf,
  presetFor,
  textOf,
  useScheduler,
} from '@/composables/useScheduler';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Named jobs on a timer, for this project.
 *
 * The Workers pane above runs Laravel's own scheduler as one process; this is
 * the table of individual jobs, each with its own frequency, its own last run
 * and its own log. Both exist because they answer different questions, and the
 * question this one answers is "did *that* job run?".
 */
const props = defineProps({
  name: { type: String, required: true },
  running: { type: Boolean, default: false },
});

const { t } = useI18n();

const {
  jobs,
  running: schedulerUp,
  restarts,
  busy,
  error,
  load,
  upsert,
  remove,
  toggleJob,
  toggleScheduler,
  runNow,
  log,
} = useScheduler(toRef(props, 'name'));

watch(() => props.name, load, { immediate: true });

// ---- the form ------------------------------------------------------------
//
// Open only while something is being written. A form that is always on screen
// makes an empty list look like a mistake rather than like a project with
// nothing scheduled yet.
const form = ref(null);

function blank() {
  return { editing: null, label: '', kind: 'laravel', text: '', preset: 'everyMinute', cron: '' };
}

function openNew() {
  form.value = blank();
}

function openEdit(job) {
  const preset = presetFor(job.cron);
  form.value = {
    editing: job.id,
    label: job.label,
    kind: kindOf(job.exec),
    text: textOf(job.exec),
    preset: preset ?? 'advanced',
    cron: job.cron,
  };
}

/** The expression the form currently means, preset or typed. */
const formCron = computed(() => {
  if (!form.value) return '';
  if (form.value.preset === 'advanced') return form.value.cron.trim();
  return PRESETS.find((p) => p.key === form.value.preset)?.cron ?? '';
});

const formArgv = computed(() => (form.value ? argvFor(form.value.kind, form.value.text) : []));

const formValid = computed(
  () => Boolean(form.value?.label.trim()) && formArgv.value.length > 0 && Boolean(formCron.value)
);

async function submit() {
  if (!formValid.value) return;
  const ok = await upsert(
    {
      id: form.value.editing,
      label: form.value.label.trim(),
      cron: formCron.value,
      exec: formArgv.value,
      enabled: true,
    },
    form.value.editing
  );
  if (ok) form.value = null;
}

// ---- the log ------------------------------------------------------------
const viewing = ref(null);
const logText = ref('');

async function openLog(job) {
  viewing.value = job;
  logText.value = '';
  logText.value = await log(job.id);
}

/** A frequency as words, falling back to the expression somebody wrote. */
function frequency(cron) {
  const preset = presetFor(cron);
  return preset ? t(`scheduler.presets.${preset}`) : cron;
}
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-scheduler"
      icon="mdi-calendar-clock"
      :title="t('scheduler.title')"
      :description="t('scheduler.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <!-- A schedule with nothing running it is a list of intentions, and the
         screen must not let that read as "scheduled". -->
    <div class="d-flex align-center ga-3 mb-4">
      <v-icon :color="schedulerUp ? 'success' : 'grey'" size="18">
        {{ schedulerUp ? 'mdi-check-circle' : 'mdi-stop-circle-outline' }}
      </v-icon>
      <div class="min-width-0">
        <div class="text-body-2 font-weight-medium">
          {{ schedulerUp ? t('scheduler.up') : t('scheduler.down') }}
        </div>
        <div
          v-if="restarts"
          class="text-caption"
          :class="restarts > 3 ? 'text-error' : 'text-warning'"
        >
          {{ t('scheduler.restarts', { count: restarts }) }}
        </div>
      </div>
      <v-spacer />
      <v-btn
        size="small"
        variant="tonal"
        :color="schedulerUp ? 'error' : 'primary'"
        :loading="busy === 'scheduler'"
        :disabled="!schedulerUp && !running"
        @click="toggleScheduler"
      >
        {{ schedulerUp ? t('scheduler.stop') : t('scheduler.start') }}
      </v-btn>
      <v-btn size="small" variant="tonal" prepend-icon="mdi-plus" @click="openNew">
        {{ t('scheduler.newJob') }}
      </v-btn>
    </div>

    <v-alert v-if="!running && !schedulerUp" type="info" variant="tonal" class="mb-3">
      <div class="text-caption">{{ t('scheduler.needsRunning') }}</div>
    </v-alert>

    <!-- ---- the form ---------------------------------------------------- -->
    <v-card v-if="form" variant="tonal" class="pa-4 mb-4">
      <div class="text-body-2 font-weight-medium mb-3">
        {{ form.editing ? t('scheduler.editJob') : t('scheduler.newJob') }}
      </div>

      <!-- What it runs, then when, then what to call it. The name came first
           while the form was two even columns, which asked for the label
           before there was anything to label — and the label is also the one
           field with no right answer until the other two are settled. -->
      <v-row dense>
        <v-col cols="12" md="6">
          <v-select
            v-model="form.kind"
            :items="KINDS.map((k) => ({ title: t(`scheduler.kinds.${k}`), value: k }))"
            :label="t('scheduler.kind')"
            density="compact"
            variant="outlined"
          />
        </v-col>
        <v-col cols="12" md="6">
          <v-select
            v-model="form.preset"
            :items="[
              ...PRESETS.map((p) => ({ title: t(`scheduler.presets.${p.key}`), value: p.key })),
              { title: t('scheduler.presets.advanced'), value: 'advanced' },
            ]"
            :label="t('scheduler.frequency')"
            density="compact"
            variant="outlined"
          />
        </v-col>

        <!-- Both of these belong to the choice above them, so they open
             underneath it rather than in a column beside it. -->
        <v-col v-if="form.preset === 'advanced'" cols="12">
          <v-text-field
            v-model="form.cron"
            :label="t('scheduler.cron')"
            :hint="t('scheduler.cronHint')"
            persistent-hint
            density="compact"
            variant="outlined"
            placeholder="0 3 * * 1"
          />
        </v-col>

        <v-col v-if="form.kind !== 'laravel'" cols="12">
          <v-text-field
            v-model="form.text"
            :label="t(`scheduler.command.${form.kind}`)"
            :hint="t('scheduler.commandHint')"
            persistent-hint
            density="compact"
            variant="outlined"
          />
        </v-col>

        <!-- Full width, and last. It is free text with a hint under it about
             the filename it becomes, and half a row left both the field and
             its hint clipped. -->
        <v-col cols="12">
          <v-text-field
            v-model="form.label"
            :label="t('scheduler.label')"
            :hint="t('scheduler.labelHint')"
            persistent-hint
            density="compact"
            variant="outlined"
          />
        </v-col>
      </v-row>

      <!-- What will actually be run, before it is saved: an argv assembled
           from a text field is one the reader should get to check. -->
      <div v-if="formArgv.length" class="mt-3 text-caption text-medium-emphasis">
        {{ t('scheduler.willRun') }}
        <code class="ml-1">{{ formArgv.join(' ') }}</code>
      </div>

      <div class="d-flex ga-2 mt-4">
        <v-btn
          size="small"
          color="primary"
          :disabled="!formValid"
          :loading="busy === (form.editing ?? 'new')"
          @click="submit"
        >
          {{ t('scheduler.save') }}
        </v-btn>
        <v-btn size="small" variant="text" @click="form = null">
          {{ t('scheduler.cancel') }}
        </v-btn>
      </div>
    </v-card>

    <!-- ---- the list ---------------------------------------------------- -->
    <v-alert v-if="!jobs.length" type="info" variant="tonal">
      <div class="text-caption">{{ t('scheduler.none') }}</div>
    </v-alert>

    <div v-for="job in jobs" :key="job.id" class="job-row">
      <v-icon :color="job.enabled ? 'success' : 'grey'" size="18">
        {{ job.enabled ? 'mdi-clock-outline' : 'mdi-pause-circle-outline' }}
      </v-icon>

      <div class="min-width-0 flex-grow-1">
        <div class="d-flex align-center ga-2">
          <span class="text-body-2 font-weight-medium">{{ job.label }}</span>
          <v-chip size="x-small" variant="tonal">{{ frequency(job.cron) }}</v-chip>
        </div>
        <code class="text-caption d-block text-truncate">{{ job.command }}</code>
        <div class="text-caption text-medium-emphasis">
          <span v-if="!job.lastRun">{{ t('scheduler.neverRan') }}</span>
          <span v-else :class="job.lastRun.ok ? '' : 'text-error'">
            {{
              job.lastRun.ok
                ? t('scheduler.lastRun', { at: job.lastRun.at })
                : t('scheduler.lastFailed', { at: job.lastRun.at })
            }}
          </span>
        </div>
      </div>

      <v-btn
        size="small"
        variant="text"
        icon="mdi-play"
        :loading="busy === job.id"
        :disabled="!schedulerUp"
        :title="t('scheduler.runNow')"
        @click="runNow(job.id)"
      />
      <v-btn
        size="small"
        variant="text"
        :icon="job.enabled ? 'mdi-pause' : 'mdi-play-circle-outline'"
        :title="job.enabled ? t('scheduler.pause') : t('scheduler.resume')"
        @click="toggleJob(job.id)"
      />
      <v-btn
        size="small"
        variant="text"
        icon="mdi-text-box-outline"
        :title="t('scheduler.log')"
        @click="openLog(job)"
      />
      <v-btn
        size="small"
        variant="text"
        icon="mdi-pencil"
        :title="t('scheduler.edit')"
        @click="openEdit(job)"
      />
      <v-btn
        size="small"
        variant="text"
        icon="mdi-delete-outline"
        color="error"
        :title="t('scheduler.delete')"
        @click="remove(job.id)"
      />
    </div>

    <v-dialog v-model="viewing" max-width="900">
      <v-card v-if="viewing">
        <v-card-title class="text-body-1">{{ viewing.label }}</v-card-title>
        <v-card-text>
          <pre v-if="logText" class="job-log">{{ logText }}</pre>
          <div v-else class="text-caption text-medium-emphasis">
            {{ t('scheduler.neverRan') }}
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="viewing = null">{{ t('scheduler.close') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-card>
</template>

<style scoped>
.job-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 0;
  border-top: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.job-log {
  max-height: 60vh;
  overflow: auto;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
