<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import { bytes, micros } from '@/lib/format';
import { useOperationsStore } from '@/stores/operations';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * php-spx: the profiler you can leave on.
 *
 * Beside the Xdebug profiler rather than replacing it, because they answer
 * different questions. Xdebug records every call exactly and costs several
 * times the request, which is right for "what does this one function do" and
 * useless for "why is this page slow" — you cannot browse a site under it.
 * SPX samples, so the page still feels like the page.
 *
 * ## Three states, in the order they have to be satisfied
 *
 * **Built** — the extension is compiled from source in a throwaway container of
 * this project's own image, because it has to match the ABI of the php-fpm that
 * loads it. Once per PHP version, shared by every project on it.
 *
 * **Switched on** — a flag and a compose overlay.
 *
 * **In the container** — mounts are fixed when a container is created, so the
 * switch reaches a running project only when it is recreated. That is the same
 * three-layer split the Xdebug pane reports, and it comes apart in practice for
 * the same reason.
 *
 * ## Three ways to record, and one of them is not this window's
 *
 * SPX's own control panel is the documented way in and the only one that can
 * profile a page somebody is *using* — a click, a form, a session. It is served
 * by the extension from inside the site's own address, and its switch is a
 * cookie this app cannot set in somebody's browser.
 *
 * The other two are here. A **request** is recorded by sending it with the
 * profiler's cookie on it, which is php-spx's own documented trigger; a
 * **command** by running it with `SPX_REPORT=full` in its environment. Neither
 * needs a browser, which is what puts profiling within reach of the terminal
 * and of an assistant.
 *
 * And what a recording *says* is read here too: the trace half of the pair is
 * replayed on the Rust side into the functions that held the time, so the
 * question the list could never answer has an answer without leaving the app.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: '' },
});

const emit = defineEmits(['apply']);

const { t } = useI18n();
const ops = useOperationsStore();

const status = ref(null);
const error = ref(null);
const busy = ref('');

const path = ref('/');
const commands = ref([]);
const command = ref(null);

/** The recording whose hotspots are open, and what they were. */
const opened = ref('');
const analysis = ref(null);

const reports = computed(() => asList(status.value?.reports));

/** Nothing can be pressed while the build or a recreate is running. */
const working = computed(() => !!busy.value || ops.isBusy(props.name));

/**
 * A recording needs the extension in the container that will serve the request.
 * Asking otherwise sends a request that succeeds and records nothing, which
 * reads as a broken button rather than as a container that predates the switch.
 */
const recordable = computed(
  () => !!status.value?.enabled && status.value?.active === true && !!status.value?.domain
);

/** The sampling periods offered, in microseconds. */
const periods = computed(() => [
  { value: 0, title: t('spx.detailExact') },
  ...[10, 100, 1000].map((us) => ({ value: us, title: t('spx.detailSampled', { us }) })),
]);

async function load() {
  if (props.runtime !== 'php') {
    status.value = null;
    return;
  }
  try {
    status.value = await api.spxStatus(props.name);
    error.value = null;
  } catch (e) {
    // A failed refresh keeps what is on screen: the whole pane hangs off
    // `v-if="status"`, and these refreshes happen while the engine is busiest.
    if (!status.value) return;
    error.value = e;
  }
}

/**
 * The commands this project can run, for the second way to record.
 *
 * The same catalogue the quick-command buttons use — the id is the only thing
 * that ever crosses, and the argv is built on the Rust side. Interactive ones
 * are dropped rather than offered and refused: a recording has to finish, and
 * `artisan tinker` ends when somebody types exit.
 */
async function loadCommands() {
  if (props.runtime !== 'php') {
    commands.value = [];
    return;
  }
  try {
    commands.value = asList(await api.quickCommands(props.name)).filter((c) => !c.interactive);
  } catch {
    // Not an error worth a banner: it costs one of three ways to record, and
    // the pane's own subject is the profiler.
    commands.value = [];
  }
  if (!commands.value.some((c) => c.id === command.value)) {
    command.value = commands.value[0]?.id ?? null;
  }
}

async function run(what, fn) {
  busy.value = what;
  error.value = null;
  try {
    await fn();
  } catch (e) {
    error.value = e;
  } finally {
    await load();
    busy.value = '';
  }
}

const build = () => run('build', () => api.spxBuild(props.name));
const toggle = (on) => run('toggle', () => api.spxSet(props.name, on));
const clear = () =>
  run('clear', async () => {
    await api.spxClear(props.name);
    close();
  });
const remove = (report) =>
  run(report.key, async () => {
    await api.spxDelete(props.name, report.key);
    if (opened.value === report.key) close();
  });

const setPeriod = (us) => run('detail', () => api.spxOptions(props.name, us, null));
const setBuiltins = (on) => run('detail', () => api.spxOptions(props.name, null, on));

/** The report the last recording produced, so the pane can say what it got. */
const justRecorded = ref(null);

const record = () =>
  run('record', async () => {
    justRecorded.value = await api.spxRecordRequest(props.name, path.value);
  });

/**
 * The same request again, and both recordings side by side.
 *
 * The loop this closes is the commonest one there is — change the code, does
 * the page get faster — and it took four steps: open the site, find the page,
 * come back, hunt for the new recording among twenty.
 *
 * Only a GET can be replayed. The refusal comes back from the backend with its
 * reason in it and is shown as an error rather than as a disabled button,
 * because the reason is the useful part: a recording names the request line and
 * nothing else, so a POST re-sent without its body and its session would be a
 * different request.
 */
const replayed = ref(null);

/**
 * K-5 — the snapshot the second run starts from.
 *
 * A captured session made a POST replayable, and what took the old refusal's
 * place is a request that **does the thing again**: a second order, a second
 * charge, a second row. Not a reason to refuse it — somebody replaying a POST
 * is doing it deliberately — but a reason to offer the one thing that makes
 * pressing it twice safe, which this app already has.
 *
 * Never chosen for them. This app cannot know which snapshot holds the state
 * the original ran under, so picking one would be answering a question nobody
 * asked with data it does not have.
 */
const snapshots = ref([]);
const startFrom = ref(null);

/** `{ service, name }` for the backend, or nothing at all. */
const chosenSnapshot = computed(() => {
  const found = snapshots.value.find((s) => `${s.service}/${s.name}` === startFrom.value);
  return found ? { service: found.service, name: found.name } : undefined;
});

async function loadSnapshots() {
  snapshots.value = asList(await api.dbSnapshots().catch(() => []));
}

const replay = (report) =>
  run(`replay:${report.key}`, async () => {
    replayed.value = await api.requestReplay(props.name, report.key, chosenSnapshot.value);
  });

/**
 * The other half: replaying a POST needs the request's cookies and its body,
 * which means writing somebody's session token to disk.
 *
 * So this is a permission and it is drawn as one. Off until pressed, armed in
 * minutes with the length chosen here, and the button that turns it off says
 * how many captures it deleted — because a permission that ends leaving its
 * harvest behind is a permission the person believes ended.
 *
 * Nothing on this screen ever shows a captured value. The status call answers
 * with a count of cookie names and a size of body: enough to say *there is
 * something here to replay*, and not a second place the credential exists.
 */
const capture = ref(null);
const minutes = ref(15);

async function loadCapture() {
  capture.value = await api.captureStatus(props.name).catch(() => null);
}

const arm = () =>
  run('capture', async () => {
    await api.captureArm(props.name, Number(minutes.value));
    await loadCapture();
  });

const disarm = () =>
  run('capture', async () => {
    await api.captureDisarm(props.name);
    await loadCapture();
  });

/**
 * A command recording goes through the operation console, like a build: it can
 * be a test suite. The list is re-read when the operation finishes rather than
 * when this call returns.
 */
const recordCommand = () => run('command', () => api.spxRecordCommand(props.name, command.value));

async function open(report) {
  if (opened.value === report.key) {
    close();
    return;
  }
  opened.value = report.key;
  analysis.value = null;
  busy.value = `open:${report.key}`;
  try {
    analysis.value = await api.spxReport(props.name, report.key);
    error.value = null;
  } catch (e) {
    error.value = e;
    close();
  } finally {
    busy.value = '';
  }
}

function close() {
  opened.value = '';
  analysis.value = null;
}

const openedReport = computed(() => reports.value.find((r) => r.key === opened.value) ?? null);

const label = (report) => report.request || report.command || t('spx.unnamedRun');

watch(
  () => [props.name, props.runtime],
  () => {
    close();
    justRecorded.value = null;
    capture.value = null;
    replayed.value = null;
    startFrom.value = null;
    load();
    loadCommands();
    loadCapture();
    loadSnapshots();
  },
  { immediate: true }
);

/**
 * Re-read when the operation finishes rather than when the call returns.
 *
 * `spx_build` resolves with an operation id as soon as the work starts — that
 * is what the console is for — so a pane that only re-read at that moment would
 * still say "not built" over a finished build. A recorded command lands the
 * same way, and its report only exists once the command has exited.
 */
watch(
  () => ops.isBusy(props.name),
  (busyNow, wasBusy) => {
    if (wasBusy && !busyNow) load();
  }
);

const when = (seconds) => new Date(seconds * 1000).toLocaleString();
const percent = (value) => `${(value ?? 0).toFixed(1)}%`;

defineExpose({ load });
</script>

<template>
  <v-card v-if="runtime === 'php'" variant="flat" class="pane">
    <PaneHeader
      help="project-spx"
      icon="mdi-chart-timeline-variant"
      :title="t('spx.title')"
      :description="t('spx.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <template v-if="status">
      <!-- The extension has to exist before anything else is worth offering. -->
      <v-alert v-if="!status.built" type="info" variant="tonal" class="mb-4">
        <div class="text-caption">
          {{ t('spx.notBuilt', { php: status.phpVersion ?? '?' }) }}
        </div>
        <v-btn
          size="small"
          color="primary"
          variant="tonal"
          class="mt-2"
          prepend-icon="mdi-hammer"
          :loading="busy === 'build' || ops.isBusy(name)"
          :disabled="working"
          @click="build()"
        >
          {{ t('spx.build') }}
        </v-btn>
      </v-alert>

      <template v-else>
        <v-switch
          :model-value="status.enabled"
          color="primary"
          density="comfortable"
          hide-details
          :disabled="working"
          :loading="busy === 'toggle'"
          :label="status.enabled ? t('spx.on') : t('spx.off')"
          @update:model-value="toggle($event)"
        />
        <div class="text-caption text-medium-emphasis mt-1">{{ t('spx.cost') }}</div>

        <!-- Mounts are fixed when a container is created, so the switch has not
             reached one that was already up. -->
        <v-alert
          v-if="status.enabled && status.running && status.active === false"
          type="warning"
          variant="tonal"
          class="mt-3"
        >
          <div class="text-caption">{{ t('spx.needsRecreate') }}</div>
          <v-btn
            size="small"
            color="warning"
            variant="tonal"
            class="mt-2"
            prepend-icon="mdi-autorenew"
            :loading="ops.isBusy(name)"
            :disabled="working"
            @click="emit('apply')"
          >
            {{ t('projectDetail.applyToContainer') }}
          </v-btn>
        </v-alert>

        <!-- Two profilers hooking one engine is unsupported in both projects,
             and the symptom is wrong numbers rather than an error — so it is
             said rather than prevented: which one to turn off is not this
             app's decision. -->
        <v-alert v-if="status.xdebugConflict" type="warning" variant="tonal" class="mt-3">
          <div class="text-caption">{{ t('spx.xdebugConflict') }}</div>
        </v-alert>

        <!-- Recording without a browser: the request carries the trigger. -->
        <template v-if="recordable">
          <div class="section-head mt-5 mb-1 d-flex align-center">
            <v-icon size="18" class="mr-2">mdi-record-circle-outline</v-icon>
            {{ t('spx.recordHere') }}
          </div>
          <div class="text-caption text-medium-emphasis mb-3">{{ t('spx.recordExplain') }}</div>

          <div class="d-flex align-start ga-2">
            <v-text-field
              v-model="path"
              density="compact"
              variant="outlined"
              hide-details
              :label="t('spx.recordPath')"
              :prefix="status.domain"
              :disabled="working"
              @keyup.enter="record()"
            />
            <v-btn
              color="primary"
              variant="tonal"
              prepend-icon="mdi-play"
              :loading="busy === 'record'"
              :disabled="working"
              @click="record()"
            >
              {{ busy === 'record' ? t('spx.recording') : t('spx.record') }}
            </v-btn>
          </div>
          <div class="text-caption text-medium-emphasis mt-1">{{ t('spx.recordPathHint') }}</div>

          <v-alert
            v-if="justRecorded"
            type="success"
            variant="tonal"
            density="compact"
            class="mt-3 text-caption"
          >
            {{
              t('spx.recordedOne', {
                what: label(justRecorded),
                took: micros(justRecorded.wallTimeUs),
              })
            }}
          </v-alert>

          <!-- K-5. Offered whenever there is a snapshot to offer, rather than
               only beside a POST: a GET can write too, and this app cannot know
               which routes do. The sentence says what it buys and what it does
               not, because a snapshot makes a replay repeatable and does not
               make two runs a controlled experiment. -->
          <template v-if="snapshots.length">
            <v-select
              v-model="startFrom"
              :items="
                snapshots.map((s) => ({
                  title: `${s.service} / ${s.name}`,
                  value: `${s.service}/${s.name}`,
                }))
              "
              :label="t('spx.startFrom')"
              :placeholder="t('spx.startFromNone')"
              persistent-placeholder
              clearable
              density="compact"
              variant="outlined"
              hide-details
              class="mt-3"
              data-test="replay-snapshot"
            />
            <p class="text-caption text-medium-emphasis mt-1">
              {{ startFrom ? t('spx.startFromChosen') : t('spx.startFromWhy') }}
            </p>
          </template>

          <!-- Both numbers, and the difference — never a verdict. One run
               against one run is not a benchmark, and a green "faster" would
               invite a conclusion the measurement cannot carry. -->
          <v-alert
            v-if="replayed"
            type="info"
            variant="tonal"
            density="compact"
            class="mt-3 text-caption"
            data-test="replay-result"
          >
            <div>{{ t('spx.replayedWhat', { what: label(replayed.after) }) }}</div>
            <div class="mono mt-1">
              {{ micros(replayed.before.wallTimeUs) }} → {{ micros(replayed.after.wallTimeUs) }}
              <span :class="replayed.wallTimeUs < 0 ? 'text-success' : 'text-medium-emphasis'">
                ({{ replayed.wallTimeUs < 0 ? '' : '+' }}{{ micros(replayed.wallTimeUs) }})
              </span>
            </div>
            <div v-if="replayed.restored" class="text-medium-emphasis mt-1">
              {{ t('spx.replayedFrom', { snapshot: replayed.restored }) }}
            </div>
            <div class="text-medium-emphasis mt-1">{{ t('spx.replayCaveat') }}</div>
          </v-alert>

          <!-- ---- the session half ------------------------------------- -->
          <!-- Drawn as a permission and not a setting: what it produces is a
               session token on disk, so it is off until pressed, it ends by
               itself, and turning it off says what it deleted. -->
          <div v-if="capture" class="capture mt-4" data-test="capture">
            <div class="text-caption text-medium-emphasis mb-2">{{ t('spx.captureWhat') }}</div>

            <div v-if="!capture.armed" class="d-flex align-center ga-2 flex-wrap">
              <v-select
                v-model="minutes"
                :items="[5, 15, 30, 60]"
                :label="t('spx.captureMinutes')"
                density="compact"
                variant="outlined"
                hide-details
                style="max-width: 150px"
                :disabled="!capture.bridge"
              />
              <v-btn
                size="small"
                variant="tonal"
                color="warning"
                prepend-icon="mdi-record-circle-outline"
                :loading="busy === 'capture'"
                :disabled="!capture.bridge"
                data-test="capture-arm"
                @click="arm"
              >
                {{ t('spx.captureArm') }}
              </v-btn>
            </div>
            <!-- The two flags are separate permissions and not independent: the
                 bridge's prepend file is the only thing that reads this one's.
                 The backend refuses the arm, and this is the same fact said
                 before the press — a sentence rather than an error. -->
            <div
              v-if="!capture.armed && !capture.bridge"
              class="text-caption text-medium-emphasis mt-2"
              data-test="capture-needs-bridge"
            >
              {{ t('spx.captureNeedsBridge') }}
            </div>

            <template v-else>
              <v-alert type="warning" variant="tonal" density="compact" class="text-caption">
                {{
                  t('spx.captureArmed', {
                    minutes: capture.remainingMinutes,
                    count: capture.captured,
                  })
                }}
                <!-- Counts and sizes. Never a cookie value, never a body. -->
                <div
                  v-for="row in capture.recent"
                  :key="`${row.request}-${row.bodyBytes}`"
                  class="mono mt-1"
                  data-test="capture-row"
                >
                  {{ row.request }} · {{ t('spx.captureCookies', { count: row.cookies }) }} ·
                  {{ row.bodyBytes }} B
                </div>
              </v-alert>
              <v-btn
                size="small"
                variant="tonal"
                class="mt-2"
                prepend-icon="mdi-delete-outline"
                :loading="busy === 'capture'"
                data-test="capture-disarm"
                @click="disarm"
              >
                {{ t('spx.captureDisarm') }}
              </v-btn>
            </template>
          </div>

          <div class="d-flex align-start ga-2 mt-4">
            <v-select
              v-model="command"
              density="compact"
              variant="outlined"
              hide-details
              item-title="display"
              item-value="id"
              :items="commands"
              :label="t('spx.recordCommand')"
              :disabled="working || !commands.length"
              :placeholder="commands.length ? '' : t('spx.recordNoCommands')"
            />
            <v-btn
              variant="tonal"
              prepend-icon="mdi-console-line"
              :loading="busy === 'command' || ops.isBusy(name)"
              :disabled="working || !command"
              @click="recordCommand()"
            >
              {{ t('spx.recordCommandGo') }}
            </v-btn>
          </div>
          <div class="text-caption text-medium-emphasis mt-1">{{ t('spx.recordCommandHint') }}</div>
        </template>

        <!-- The panel is SPX's own, served by the extension from inside this
             project's vhost. No port, no second server. -->
        <div v-if="status.enabled && status.controlUrl" class="mt-4">
          <v-btn
            size="small"
            variant="tonal"
            prepend-icon="mdi-open-in-new"
            :disabled="working"
            @click="api.openInBrowser(status.controlUrl)"
          >
            {{ t('spx.openPanel') }}
          </v-btn>
          <div class="text-caption text-medium-emphasis mt-2">{{ t('spx.howToRecord') }}</div>
        </div>

        <!-- How much detail a recording carries. php-spx records every call
             unless a period is set, which is the cost this pane's own first
             sentence claims to avoid. -->
        <div class="section-head mt-5 mb-2 d-flex align-center">
          <v-icon size="18" class="mr-2">mdi-tune-variant</v-icon>
          {{ t('spx.detail') }}
        </div>
        <v-select
          :model-value="status.samplingPeriod"
          density="compact"
          variant="outlined"
          hide-details
          :items="periods"
          :label="t('spx.sampling')"
          :disabled="working"
          :loading="busy === 'detail'"
          @update:model-value="setPeriod($event)"
        />
        <div class="text-caption text-medium-emphasis mt-1">{{ t('spx.detailHint') }}</div>
        <v-switch
          :model-value="status.builtins"
          color="primary"
          density="comfortable"
          hide-details
          class="mt-2"
          :disabled="working"
          :label="t('spx.builtins')"
          @update:model-value="setBuiltins($event)"
        />
        <div class="text-caption text-medium-emphasis">{{ t('spx.builtinsHint') }}</div>
        <div class="text-caption text-medium-emphasis mt-2">{{ t('spx.settingsHere') }}</div>
      </template>

      <div class="section-head mt-5 mb-2 d-flex align-center">
        <v-icon size="18" class="mr-2">mdi-file-chart-outline</v-icon>
        {{ t('spx.recorded', { n: reports.length }) }}
        <v-spacer />
        <v-btn
          v-if="reports.length"
          size="small"
          variant="text"
          prepend-icon="mdi-delete-sweep-outline"
          :loading="busy === 'clear'"
          :disabled="working"
          @click="clear()"
        >
          {{ t('spx.clear', { size: bytes(status.bytes) }) }}
        </v-btn>
      </div>

      <div v-if="!reports.length" class="text-caption text-medium-emphasis">
        {{ t('spx.nothingYet') }}
      </div>

      <v-table v-else density="compact">
        <tbody>
          <tr v-for="report in reports" :key="report.key">
            <td>
              <div class="text-body-2">{{ label(report) }}</div>
              <div class="text-caption text-medium-emphasis">
                {{ when(report.recordedAt) }} ·
                {{ report.cli ? t('spx.cli') : t('spx.request') }}
              </div>
            </td>
            <td class="mono text-right">{{ micros(report.wallTimeUs) }}</td>
            <td class="mono text-right">{{ bytes(report.peakMemory) }}</td>
            <td class="mono text-right">{{ report.callCount }}</td>
            <td class="text-right">
              <!-- The question the row could never answer, read from the trace
                   half of the pair without leaving the app. -->
              <v-btn
                icon
                size="x-small"
                variant="text"
                :aria-label="t('spx.hotspots')"
                :color="opened === report.key ? 'primary' : undefined"
                :loading="busy === `open:${report.key}`"
                :disabled="working"
                @click="open(report)"
              >
                <v-icon size="18">mdi-fire</v-icon>
              </v-btn>
              <!-- Offered on every row, refused with a reason on the ones it
                   cannot serve. A button hidden for a POST would leave somebody
                   wondering why the row above has one. -->
              <!-- A replay that would write says so on the row, and the
                   backend is what decided it: the rule lives on the recording
                   (`spx::Report::mutates`) rather than being re-derived here
                   from a string. -->
              <v-btn
                icon
                size="x-small"
                variant="text"
                :aria-label="report.mutates ? t('spx.replayWrites') : t('spx.replay')"
                :color="report.mutates ? 'warning' : undefined"
                :loading="busy === `replay:${report.key}`"
                :disabled="working"
                @click="replay(report)"
              >
                <v-icon size="18">{{ report.mutates ? 'mdi-database-edit-outline' : 'mdi-replay' }}</v-icon>
                <v-tooltip v-if="report.mutates" activator="parent" location="top">
                  {{ t('spx.replayWrites') }}
                </v-tooltip>
              </v-btn>
              <v-btn
                v-if="status.viewBase"
                icon
                size="x-small"
                variant="text"
                :aria-label="t('spx.view')"
                :disabled="working"
                @click="api.openInBrowser(status.viewBase + report.key)"
              >
                <v-icon size="18">mdi-chart-areaspline</v-icon>
              </v-btn>
              <v-btn
                icon
                size="x-small"
                variant="text"
                :aria-label="t('spx.remove')"
                :loading="busy === report.key"
                :disabled="working"
                @click="remove(report)"
              >
                <v-icon size="18">mdi-delete-outline</v-icon>
              </v-btn>
            </td>
          </tr>
        </tbody>
      </v-table>

      <!-- Where the time went, for the one recording that is open. -->
      <v-card v-if="analysis" variant="tonal" class="mt-4 pa-3">
        <div class="d-flex align-center mb-2">
          <v-icon size="18" class="mr-2">mdi-fire</v-icon>
          <span class="text-body-2">
            {{ t('spx.hotspotsFor', { what: openedReport ? label(openedReport) : analysis.key }) }}
          </span>
          <v-spacer />
          <v-btn size="small" variant="text" @click="close()">{{ t('spx.hotspotsClose') }}</v-btn>
        </div>

        <!-- Said rather than hidden: the shares below are then about the start
             of the run, which is a true answer to a smaller question. -->
        <v-alert
          v-if="analysis.truncated"
          type="info"
          variant="tonal"
          density="compact"
          class="mb-2 text-caption"
        >
          {{ t('spx.hotspotsTruncated') }}
        </v-alert>

        <div v-if="!analysis.hotspots.length" class="text-caption text-medium-emphasis">
          {{ t('spx.hotspotsEmpty') }}
        </div>
        <v-table v-else density="compact">
          <thead>
            <tr>
              <th>{{ t('spx.hotspotFunction') }}</th>
              <th class="text-right">{{ t('spx.hotspotSelf') }}</th>
              <th class="text-right">{{ t('spx.hotspotTotal') }}</th>
              <th class="text-right">{{ t('spx.hotspotCalls') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="spot in analysis.hotspots" :key="spot.function">
              <td class="mono text-caption">{{ spot.function }}</td>
              <td class="mono text-right">
                {{ percent(spot.exclusivePercent) }}
                <span class="text-medium-emphasis">· {{ micros(spot.exclusiveUs) }}</span>
              </td>
              <td class="mono text-right">{{ percent(spot.inclusivePercent) }}</td>
              <td class="mono text-right">{{ spot.calls }}</td>
            </tr>
          </tbody>
        </v-table>
      </v-card>
    </template>
  </v-card>
</template>
