<script setup>
import { computed, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { stateColor, uptimeOf } from '@/composables/useSupervisors';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';
import SupervisorCheckDialog from '@/components/project/SupervisorCheckDialog.vue';

/**
 * The supervisord inside this project's own container.
 *
 * StackVo's generated image for an nginx or caddy project runs supervisord as
 * its command, with `php-fpm` and the web server under it. So this pane needs
 * nothing added and nothing configured: a project already names its container.
 *
 * The three ways it can be empty look identical on screen and send somebody to
 * three different places, so each says which one it is.
 */
const props = defineProps({
  name: { type: String, required: true },
  running: { type: Boolean, default: false },
});

const { t } = useI18n();

const view = ref(null);
const error = ref(null);
const busy = ref(null);
let timer = null;

const snapshot = computed(() => view.value?.snapshot ?? null);
const reach = computed(() => view.value?.reach ?? null);

async function load() {
  try {
    view.value = await api.supervisorProject(props.name);
    error.value = null;
  } catch (e) {
    view.value = null;
    error.value = e;
  }
  return view.value;
}

/**
 * Polled only while the container is up and answering. A timer against a
 * stopped project is a `docker exec` every few seconds that can only fail.
 */
watch(
  () => [props.name, props.running],
  async () => {
    clearInterval(timer);
    timer = null;
    if (!props.running) {
      view.value = null;
      return;
    }
    await load();
    if (reach.value === 'ok') timer = setInterval(load, 5000);
  },
  { immediate: true }
);

onUnmounted(() => clearInterval(timer));

async function control(verb, target) {
  busy.value = `${verb}:${target}`;
  error.value = null;
  try {
    await api.supervisorControl(props.name, 'process', verb, target, undefined);
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = null;
    await load();
  }
}

/** Owned by the dialog, driven from here — this pane owns the rows. */
const checkDialog = ref(null);

const viewing = ref(null);
const logText = ref('');

async function openLog(process) {
  viewing.value = process;
  logText.value = '';
  logText.value = await api.supervisorLog(props.name, process.fullName, 'stdout', 500);
}
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-supervisor"
      icon="mdi-server-network"
      :title="t('projectSupervisor.title')"
      :description="t('projectSupervisor.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <v-alert v-if="!running" type="info" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.needsRunning') }}</div>
    </v-alert>

    <!-- Each absence says which one it is, because the fix differs. -->
    <v-alert v-else-if="reach === 'noSupervisord'" type="info" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.noSupervisord') }}</div>
    </v-alert>

    <v-alert v-else-if="reach === 'noSocket'" type="warning" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.noSocket') }}</div>
    </v-alert>

    <v-alert v-else-if="reach === 'stopped'" type="info" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.stopped') }}</div>
    </v-alert>

    <template v-else-if="snapshot">
      <div class="text-caption text-medium-emphasis mb-2">
        {{
          t('projectSupervisor.counts', {
            running: snapshot.summary.running,
            total: snapshot.summary.total,
          })
        }}
        <span v-if="snapshot.summary.flapping" class="text-warning">
          · {{ t('supervisors.flappingCount', { count: snapshot.summary.flapping }) }}
        </span>
        <span v-if="snapshot.summary.failing" class="text-error">
          · {{ t('supervisorCheck.failing', { count: snapshot.summary.failing }) }}
        </span>
      </div>

      <div v-for="process in snapshot.processes" :key="process.fullName" class="sup-row">
        <v-icon size="16" :color="stateColor(process)">
          {{ process.flapping ? 'mdi-alert-circle' : 'mdi-circle' }}
        </v-icon>

        <div class="min-width-0 flex-grow-1">
          <div class="d-flex align-center ga-2">
            <span class="text-body-2">{{ process.name }}</span>
            <v-chip size="x-small" variant="tonal" :color="stateColor(process)">
              {{ process.stateName }}
            </v-chip>
            <v-chip v-if="process.flapping" size="x-small" color="warning" variant="flat">
              {{ t('supervisors.flapping') }}
            </v-chip>
          </div>
          <div class="text-caption text-medium-emphasis text-truncate">
            <span v-if="process.pid">pid {{ process.pid }}</span>
            <span v-if="uptimeOf(process)"> · {{ uptimeOf(process) }}</span>
            <span v-if="process.restarts">
              · {{ t('supervisors.restarts', { count: process.restarts }) }}
            </span>
            <span v-if="process.spawnErr" class="text-error"> · {{ process.spawnErr }}</span>
            <span v-if="process.check" :class="process.check.ok ? '' : 'text-error'">
              · {{ process.check.ok ? t('supervisorCheck.answering') : process.check.detail }}
            </span>
          </div>
        </div>

        <v-btn
          size="small"
          variant="text"
          icon="mdi-restart"
          :loading="busy === `restart:${process.fullName}`"
          :title="t('supervisors.restart')"
          @click="control('restart', process.fullName)"
        />
        <v-btn
          size="small"
          variant="text"
          :icon="
            process.check
              ? process.check.ok
                ? 'mdi-heart-pulse'
                : 'mdi-heart-broken'
              : 'mdi-heart-outline'
          "
          :color="process.check && !process.check.ok ? 'error' : undefined"
          :title="t('supervisorCheck.button')"
          @click="checkDialog?.open(process.fullName)"
        />
        <v-btn
          size="small"
          variant="text"
          icon="mdi-text-box-outline"
          :title="t('supervisors.log')"
          @click="openLog(process)"
        />
      </div>
    </template>

    <SupervisorCheckDialog ref="checkDialog" :project="name" @saved="load" />

    <v-dialog v-model="viewing" max-width="900">
      <v-card v-if="viewing">
        <v-card-title class="text-body-1">{{ viewing.fullName }}</v-card-title>
        <v-card-text>
          <pre v-if="logText" class="sup-log">{{ logText }}</pre>
          <div v-else class="text-caption text-medium-emphasis">
            {{ t('projectSupervisor.logToStdout') }}
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="viewing = null">{{ t('supervisors.close') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-card>
</template>

<style scoped>
.sup-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid rgb(var(--v-border-color), var(--v-border-opacity));
}

.sup-log {
  max-height: 60vh;
  overflow: auto;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
