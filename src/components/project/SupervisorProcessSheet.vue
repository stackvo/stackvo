<script setup>
import { computed, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { stateColor, uptimeOf } from '@/composables/useSupervisors';
import ErrorAlert from '@/components/ErrorAlert.vue';
import SideSheet from '@/components/SideSheet.vue';

/**
 * One supervised process, in full.
 *
 * The table's row says RUNNING, a pid and an uptime. What it cannot say is
 * what the process *is* — the command it was started with, the stop timeout
 * it will be given, how many copies there are, who put it there — and what it
 * is costing. This sheet asks once, when opened, and labels each answer by
 * where it came from: the daemon, the manifest, the config file in the
 * container, and /proc.
 *
 * A side sheet rather than a dialog, like every other detail in this app: the
 * table stays in view beside it, so the row being read and the numbers being
 * read about it are on screen together. Two tabs — the detail, and the log —
 * because both are questions about the same row and opening two things for
 * one process is one too many.
 *
 * `row` is the table's own row and is what the runtime section reads from:
 * the restart count and the flapping flag are derived by the watch across
 * polls, and the fresh row the detail command returns has neither.
 */
const props = defineProps({
  project: { type: String, required: true },
});

const emit = defineEmits(['changed']);

const { t } = useI18n();

const open = ref(false);
const tab = ref('detail');
const row = ref(null);
const detail = ref(null);
const error = ref(null);
const loading = ref(false);
const busy = ref(null);

const SIGNAL_NAMES = ['HUP', 'INT', 'QUIT', 'USR1', 'USR2', 'TERM', 'KILL'];
const signal = ref('HUP');

/**
 * Each signal with one line on what it does, under its name in the list. What
 * a signal means is the program's business — USR1 reopens php-fpm's logs and
 * means nothing to a queue worker — so the line says the convention and names
 * the one common case.
 */
const SIGNALS = computed(() =>
  SIGNAL_NAMES.map((name) => ({
    title: name,
    value: name,
    props: { subtitle: t(`projectSupervisor.detail.signals.${name}`) },
  }))
);

async function load() {
  if (!row.value) return;
  loading.value = true;
  try {
    detail.value = await api.supervisorProcess(props.project, row.value.fullName);
    error.value = null;
  } catch (e) {
    detail.value = null;
    error.value = e;
  } finally {
    loading.value = false;
  }
}

/** Opened by the pane with the row it is showing, on either tab. */
function show(process, initialTab = 'detail') {
  row.value = process;
  detail.value = null;
  error.value = null;
  logText.value = '';
  logError.value = null;
  tab.value = initialTab;
  open.value = true;
  load();
}

watch(open, (isOpen) => {
  if (!isOpen) {
    row.value = null;
    detail.value = null;
    stopFollowing();
  }
});

defineExpose({ show });

const process = computed(() => detail.value?.process ?? row.value);
const running = computed(() => process.value?.stateName === 'RUNNING');

/** Runtime, from the freshest row there is, with the watch's two numbers. */
const runtime = computed(() => {
  const p = process.value;
  if (!p) return [];
  return [
    { label: t('projectSupervisor.detail.state'), value: p.stateName },
    { label: t('projectSupervisor.detail.group'), value: p.group },
    { label: t('projectSupervisor.detail.pid'), value: p.pid || '—' },
    { label: t('projectSupervisor.detail.uptime'), value: uptimeOf(p) || '—' },
    { label: t('projectSupervisor.detail.description'), value: p.description || '—' },
    {
      label: t('projectSupervisor.detail.restarts'),
      value: row.value?.flapping
        ? `${row.value.restarts} · ${t('supervisors.flapping')}`
        : String(row.value?.restarts ?? 0),
    },
  ];
});

const declared = computed(() => {
  const d = detail.value?.declared;
  if (!d) return [];
  return [
    { label: t('projectSupervisor.detail.exec'), value: d.exec.join(' '), mono: true },
    { label: t('projectSupervisor.detail.replicas'), value: String(d.replicas) },
    { label: t('projectSupervisor.detail.stopWait'), value: `${d.stopWait} s` },
    {
      label: t('projectSupervisor.detail.enabled'),
      value: d.enabled ? t('projectSupervisor.detail.yes') : t('projectSupervisor.detail.no'),
    },
  ];
});

const resources = computed(() => {
  const r = detail.value?.resources;
  if (!r) return [];
  return [
    { label: t('projectSupervisor.detail.memory'), value: `${(r.rssKb / 1024).toFixed(1)} MB` },
    { label: t('projectSupervisor.detail.cpu'), value: `${r.cpuPercent} %` },
    { label: t('projectSupervisor.detail.cpuSeconds'), value: `${r.cpuSeconds} s` },
    { label: t('projectSupervisor.detail.threads'), value: String(r.threads) },
  ];
});

async function control(verb, sig) {
  if (!row.value) return;
  busy.value = verb;
  error.value = null;
  try {
    await api.supervisorControl(props.project, 'process', verb, row.value.fullName, sig);
    emit('changed');
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = null;
    await load();
  }
}

// ------------------------------------------------------------------ the log

const LINE_CHOICES = [200, 500, 2000];
const lines = ref(500);
const follow = ref(true);
const logText = ref('');
const logError = ref(null);
const logLoading = ref(false);
let followTimer = null;

/**
 * Where the process writes, read out of the block the daemon runs with.
 *
 * The image's own two programs write to the container's stdout, and
 * supervisord cannot read a device back — `supervisorctl tail` on it answers
 * an error, not a log. Knowing that from the config means asking the right
 * thing: the container's own output, which is where those lines are.
 */
const logFile = computed(
  () => detail.value?.config.find((entry) => entry.key === 'stdout_logfile')?.value ?? null
);
const logOnStdout = computed(() => !!logFile.value && logFile.value.startsWith('/dev/'));

async function readLog() {
  if (!row.value) return;
  logLoading.value = true;
  try {
    logText.value = logOnStdout.value
      ? await api.supervisorStdout(props.project, lines.value)
      : await api.supervisorLog(props.project, row.value.fullName, 'stdout', lines.value);
    logError.value = null;
  } catch (e) {
    logError.value = e;
  } finally {
    logLoading.value = false;
  }
}

function stopFollowing() {
  clearInterval(followTimer);
  followTimer = null;
}

/**
 * Read when the tab is shown and the config is known, and every few seconds
 * after while "follow" is on — a queue worker's log is watched, not read
 * once. Nothing polls while the other tab is up or the sheet is closed.
 */
watch(
  () => [open.value, tab.value, detail.value, follow.value, lines.value],
  ([isOpen, which, known, keepFollowing]) => {
    stopFollowing();
    if (!isOpen || which !== 'logs' || !known) return;
    readLog();
    if (keepFollowing) followTimer = setInterval(readLog, 3000);
  }
);

onUnmounted(stopFollowing);
</script>

<template>
  <SideSheet
    v-model="open"
    :title="row?.fullName ?? ''"
    icon="mdi-cog-play-outline"
    :width="640"
    :flush="tab === 'logs'"
  >
    <template #header-append>
      <v-chip
        v-if="detail"
        size="small"
        variant="tonal"
        :color="detail.origin === 'unknown' ? 'warning' : undefined"
      >
        {{ t(`projectSupervisor.detail.origin.${detail.origin}`) }}
      </v-chip>
      <v-btn
        icon
        :loading="loading"
        :aria-label="t('projectSupervisor.detail.refresh')"
        @click="load"
      >
        <v-icon>mdi-refresh</v-icon>
        <v-tooltip activator="parent">{{ t('projectSupervisor.detail.refresh') }}</v-tooltip>
      </v-btn>
    </template>

    <template #tabs>
      <v-tabs v-model="tab" bg-color="primary" color="on-primary" density="comfortable" grow>
        <v-tab value="detail" prepend-icon="mdi-information-outline">
          {{ t('projectSupervisor.detail.tabs.detail') }}
        </v-tab>
        <v-tab value="logs" prepend-icon="mdi-text-box-outline">
          {{ t('projectSupervisor.detail.tabs.logs') }}
        </v-tab>
      </v-tabs>
    </template>

    <div v-if="row && tab === 'detail'" data-testid="supervisor-process-detail">
      <ErrorAlert v-if="error" :error="error" class="mb-4" />

      <!-- An unknown origin is the one worth a sentence: it is going away. -->
      <v-alert
        v-if="detail && detail.origin === 'unknown'"
        type="warning"
        variant="tonal"
        density="compact"
        class="mb-4"
      >
        <div class="text-caption">{{ t('projectSupervisor.detail.unknownHint') }}</div>
      </v-alert>

      <div class="d-flex align-center ga-2 mb-1">
        <v-icon size="16" :color="process ? stateColor(process) : undefined">mdi-circle</v-icon>
        <span class="text-overline">{{ t('projectSupervisor.detail.runtime') }}</span>
      </div>
      <v-table density="compact" class="mb-4">
        <tbody>
          <tr v-for="r in runtime" :key="r.label">
            <td class="text-medium-emphasis" style="width: 40%">{{ r.label }}</td>
            <td>{{ r.value }}</td>
          </tr>
        </tbody>
      </v-table>

      <template v-if="detail">
        <template v-if="detail.declared">
          <div class="text-overline">{{ t('projectSupervisor.detail.declared') }}</div>
          <v-table density="compact" class="mb-4">
            <tbody>
              <tr v-for="r in declared" :key="r.label">
                <td class="text-medium-emphasis" style="width: 40%">{{ r.label }}</td>
                <td :class="{ 'font-mono': r.mono }">{{ r.value }}</td>
              </tr>
            </tbody>
          </v-table>
        </template>

        <div class="text-overline">{{ t('projectSupervisor.detail.config') }}</div>
        <div class="text-caption text-medium-emphasis mb-1 font-mono">
          {{ detail.configPath }}
        </div>
        <v-table v-if="detail.config.length" density="compact" class="mb-4">
          <tbody>
            <tr v-for="entry in detail.config" :key="entry.key">
              <td class="text-medium-emphasis font-mono" style="width: 40%">{{ entry.key }}</td>
              <td class="font-mono text-break">{{ entry.value }}</td>
            </tr>
          </tbody>
        </v-table>
        <div v-else class="text-caption text-medium-emphasis mb-4">
          {{ t('projectSupervisor.detail.noConfig') }}
        </div>

        <div class="text-overline">{{ t('projectSupervisor.detail.resources') }}</div>
        <v-table v-if="detail.resources" density="compact" class="mb-4">
          <tbody>
            <tr v-for="r in resources" :key="r.label">
              <td class="text-medium-emphasis" style="width: 40%">{{ r.label }}</td>
              <td>{{ r.value }}</td>
            </tr>
          </tbody>
        </v-table>
        <div v-else class="text-caption text-medium-emphasis mb-4">
          {{ t('projectSupervisor.detail.noResources') }}
        </div>
      </template>

      <v-progress-linear v-else-if="loading" indeterminate class="mb-4" />

      <div class="text-overline">{{ t('projectSupervisor.detail.signal') }}</div>
      <div class="d-flex align-center ga-2">
        <v-select
          v-model="signal"
          :items="SIGNALS"
          density="compact"
          variant="outlined"
          hide-details
          style="max-width: 160px"
        />
        <v-btn
          size="small"
          variant="tonal"
          :disabled="!running"
          :loading="busy === 'signal'"
          @click="control('signal', signal)"
        >
          {{ t('projectSupervisor.detail.send') }}
        </v-btn>
      </div>
    </div>

    <div v-else-if="row" class="sup-log-tab" data-testid="supervisor-process-log">
      <div class="d-flex align-center ga-3 px-4 py-2">
        <span class="text-caption text-medium-emphasis font-mono text-truncate flex-grow-1">
          {{ logFile ?? '' }}
        </span>
        <v-select
          v-model="lines"
          :items="LINE_CHOICES"
          :label="t('projectSupervisor.detail.logs.lines')"
          density="compact"
          variant="outlined"
          hide-details
          style="max-width: 120px"
        />
        <v-switch
          v-model="follow"
          :label="t('projectSupervisor.detail.logs.follow')"
          density="compact"
          hide-details
          color="primary"
        />
        <v-btn
          size="small"
          variant="text"
          icon="mdi-refresh"
          :loading="logLoading"
          :title="t('projectSupervisor.detail.refresh')"
          @click="readLog"
        />
      </div>
      <v-divider />

      <ErrorAlert v-if="logError" :error="logError" class="ma-4" />

      <!-- Said above the text rather than instead of it: what follows is the
           whole container's output, and a line from nginx under a php-fpm
           heading needs that sentence to make sense. -->
      <div v-if="logOnStdout" class="text-caption text-medium-emphasis px-4 pt-3">
        {{ t('projectSupervisor.logToStdout') }}
      </div>
      <pre v-if="logText" class="sup-log pa-4">{{ logText }}</pre>
      <div v-else-if="detail && !logLoading" class="text-caption text-medium-emphasis pa-4">
        {{ t('projectSupervisor.detail.logs.empty') }}
      </div>
      <v-progress-linear v-else indeterminate />
    </div>

    <template #footer>
      <v-btn
        v-if="running"
        variant="tonal"
        color="error"
        :loading="busy === 'stop'"
        @click="control('stop')"
      >
        {{ t('projectSupervisor.detail.stop') }}
      </v-btn>
      <v-btn v-else variant="tonal" :loading="busy === 'start'" @click="control('start')">
        {{ t('projectSupervisor.detail.start') }}
      </v-btn>
      <v-btn variant="text" :loading="busy === 'restart'" @click="control('restart')">
        {{ t('supervisors.restart') }}
      </v-btn>
    </template>
  </SideSheet>
</template>

<style scoped>
.font-mono {
  font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
  font-size: 0.8rem;
}

/* The tab fills the sheet's body (`flush`) and the log scrolls inside it. */
.sup-log-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.sup-log {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
