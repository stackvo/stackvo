<script setup>
import { computed, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { stateColor, uptimeOf } from '@/composables/useSupervisors';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';
import RemedyAlert from '@/components/project/RemedyAlert.vue';
import SupervisorCheckDialog from '@/components/project/SupervisorCheckDialog.vue';
import SupervisorProcessSheet from '@/components/project/SupervisorProcessSheet.vue';

/**
 * The supervisord inside this project's own container.
 *
 * StackVo's generated image for an nginx or caddy project runs supervisord as
 * its command, with `php-fpm` and the web server under it. So this pane needs
 * nothing added and nothing configured: a project already names its container.
 *
 * The three ways it can be empty look identical on screen and send somebody to
 * three different places, so each says which one it is.
 *
 * One of the three is also the one this pane can answer. An image built before
 * the generated config grew a supervisord socket has a supervisord in it that
 * cannot be talked to, and the fix is a rebuild — which the warning named in
 * prose and then left the reader to go and find. It carries the standard
 * button now, and re-reads itself when the rebuild finishes rather than sitting
 * on the warning it just made untrue.
 *
 * The rows are not only the image's two. A project declares its own long
 * processes — a queue worker, a scheduler — in `stackvo.json`, and the
 * generator writes them into the same config, so a rebuild keeps them. What a
 * rebuild does not do is reach a container that is already up: `pending` is
 * the list the manifest declares and the daemon is not running, and the
 * button beside it pushes the config in now. `stale` is the reverse — running,
 * not declared — and is named because the next rebuild will drop it.
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
const pending = computed(() => view.value?.pending ?? []);
const stale = computed(() => view.value?.stale ?? []);

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

/**
 * Push what stackvo.json declares into the running daemon. The answer is the
 * same view `load` fetches, so the banner clears on the daemon's say-so
 * rather than on this pane's.
 */
async function apply() {
  busy.value = 'apply';
  error.value = null;
  try {
    view.value = await api.supervisorApply(props.name);
  } catch (e) {
    error.value = e;
    await load();
  } finally {
    busy.value = null;
  }
}

/** Owned by the dialog, driven from here — this pane owns the rows. */
const checkDialog = ref(null);

/**
 * The detail sheet: the pane hands it the row it is showing, and which tab
 * to open on — the log button and the info button are the same sheet.
 */
const processSheet = ref(null);

/**
 * The daemon as a whole: what somebody types at `supervisorctl` when the
 * question is not about one row. A menu on the header, and the daemon's own
 * answer shown under it, because `queue: added process group` and
 * `ERROR: CANT_REREAD` are the point of asking.
 *
 * Two of the six take the site down — `stop all` and `restart all` stop the
 * web server and php-fpm with everything else — so those two ask first.
 */
const DAEMON_VERBS = [
  { verb: 'status', icon: 'mdi-format-list-bulleted' },
  { verb: 'reread', icon: 'mdi-file-refresh-outline' },
  { verb: 'update', icon: 'mdi-update' },
  { verb: 'start-all', icon: 'mdi-play' },
  { verb: 'stop-all', icon: 'mdi-stop', confirm: true },
  { verb: 'restart-all', icon: 'mdi-restart', confirm: true },
];
const daemonResult = ref(null);
const daemonBusy = ref(null);
// The verb waiting on a confirmation, and whether the question is up. Two
// refs and not one: the dialog closes itself *before* it says "confirm", so a
// verb cleared on close would be gone by the time it was needed.
const confirming = ref(null);
const confirmOpen = ref(false);

function ask(entry) {
  if (entry.confirm) {
    confirming.value = entry.verb;
    confirmOpen.value = true;
  } else {
    runDaemon(entry.verb);
  }
}

async function runDaemon(verb) {
  daemonBusy.value = verb;
  error.value = null;
  try {
    daemonResult.value = await api.supervisorDaemon(props.name, verb);
  } catch (e) {
    error.value = e;
  } finally {
    daemonBusy.value = null;
    await load();
  }
}
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-supervisor"
      icon="mdi-server-network"
      :title="t('projectSupervisor.title')"
      :description="t('projectSupervisor.explain')"
    >
      <template #append>
        <v-menu v-if="running && reach === 'ok'" location="bottom end">
          <template #activator="{ props: menu }">
            <v-btn
              icon
              variant="text"
              v-bind="menu"
              :loading="!!daemonBusy"
              :aria-label="t('projectSupervisor.daemon.button')"
            >
              <v-icon>mdi-console-line</v-icon>
              <v-tooltip activator="parent" location="bottom">
                {{ t('projectSupervisor.daemon.button') }}
              </v-tooltip>
            </v-btn>
          </template>
          <v-list density="compact" class="px-2 py-2">
            <v-list-item
              v-for="entry in DAEMON_VERBS"
              :key="entry.verb"
              :disabled="!!daemonBusy"
              @click="ask(entry)"
            >
              <template #prepend>
                <v-icon>{{ entry.icon }}</v-icon>
              </template>
              <v-list-item-title>{{
                t(`projectSupervisor.daemon.${entry.verb}`)
              }}</v-list-item-title>
              <v-list-item-subtitle class="font-mono">
                supervisorctl {{ entry.verb.replace('-all', ' all') }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </v-menu>
      </template>
    </PaneHeader>

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <!-- The daemon's answer, in its own words, until it is closed. -->
    <v-alert
      v-if="daemonResult"
      :type="daemonResult.ok ? 'success' : 'warning'"
      variant="tonal"
      density="compact"
      closable
      class="mb-4"
      data-testid="supervisor-daemon-result"
      @click:close="daemonResult = null"
    >
      <div class="text-caption font-mono mb-1">
        supervisorctl {{ daemonResult.verb.replace('-all', ' all') }}
      </div>
      <pre class="sup-daemon-output">{{
        daemonResult.output || t('projectSupervisor.daemon.silent')
      }}</pre>
    </v-alert>

    <ConfirmDialog
      v-model="confirmOpen"
      :title="confirming ? t(`projectSupervisor.daemon.confirm.${confirming}.title`) : ''"
      :message="
        confirming ? t(`projectSupervisor.daemon.confirm.${confirming}.message`, { name }) : ''
      "
      :confirm-text="confirming ? t(`projectSupervisor.daemon.${confirming}`) : ''"
      color="error"
      @confirm="runDaemon(confirming)"
    />

    <v-alert v-if="!running" type="info" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.needsRunning') }}</div>
    </v-alert>

    <!-- Each absence says which one it is, because the fix differs. -->
    <v-alert v-else-if="reach === 'noSupervisord'" type="info" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.noSupervisord') }}</div>
    </v-alert>

    <RemedyAlert
      v-else-if="reach === 'noSocket'"
      :name="name"
      remedy="rebuild"
      :text="t('projectSupervisor.noSocket')"
      class="mb-0"
      @done="load"
    />

    <v-alert v-else-if="reach === 'stopped'" type="info" variant="tonal" class="mb-0">
      <div class="text-caption">{{ t('projectSupervisor.stopped') }}</div>
    </v-alert>

    <template v-else-if="snapshot">
      <!--
        The manifest and the daemon disagree. Two directions, one alert: a
        declared process that is not running has a button; a running process
        that is not declared has a sentence, because the fix for that one is a
        decision — declare it or let the rebuild drop it — and not a click.
      -->
      <v-alert
        v-if="pending.length || stale.length"
        type="warning"
        variant="tonal"
        density="compact"
        class="mb-3"
        data-testid="supervisor-drift"
      >
        <div v-if="pending.length" class="text-caption">
          {{ t('projectSupervisor.pending', { list: pending.join(', ') }, pending.length) }}
        </div>
        <div v-if="stale.length" class="text-caption">
          {{ t('projectSupervisor.stale', { list: stale.join(', ') }, stale.length) }}
        </div>
        <template v-if="pending.length" #append>
          <v-btn
            size="small"
            variant="tonal"
            color="warning"
            :loading="busy === 'apply'"
            @click="apply"
          >
            {{ t('projectSupervisor.apply') }}
          </v-btn>
        </template>
      </v-alert>

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
          icon="mdi-information-outline"
          :title="t('projectSupervisor.detail.button')"
          @click="processSheet?.show(process)"
        />
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
          @click="processSheet?.show(process, 'logs')"
        />
      </div>
    </template>

    <SupervisorCheckDialog ref="checkDialog" :project="name" @saved="load" />
    <SupervisorProcessSheet ref="processSheet" :project="name" @changed="load" />
  </v-card>
</template>

<style scoped>
.font-mono,
.sup-daemon-output {
  font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
  font-size: 0.78rem;
}

.sup-daemon-output {
  margin: 0;
  max-height: 40vh;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.sup-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-top: 1px solid rgb(var(--v-border-color), var(--v-border-opacity));
}
</style>
