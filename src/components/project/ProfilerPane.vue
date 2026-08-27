<script setup>
import { computed, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { bytes } from '@/lib/format';
import { useOperationsStore } from '@/stores/operations';
import { useProfiler } from '@/composables/useProfiler';
import FlameView from '@/components/FlameView.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Xdebug's profiler: the mode switch, the recorded files, and one open report.
 *
 * `apply` rather than a call: recreating the container is the project's
 * lifecycle, owned by the view — this pane knows only that the running
 * container disagrees with the configured mode.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: '' },
  running: { type: Boolean, default: false },
});

const emit = defineEmits(['apply']);

const { t } = useI18n();
const ops = useOperationsStore();

const {
  status,
  report,
  tree,
  treeBusy,
  loadTree,
  flame,
  openTrace,
  openId,
  busy,
  error,
  needsRestart,
  cost,
  load,
  setMode,
  setDevelop,
  open,
  remove,
  clear,
} = useProfiler(toRef(props, 'name'));

/**
 * Does the chosen mode write a file this app reads back?
 *
 * Two of the four do. `debug` connects to an IDE and `coverage` switches on an
 * API PHPUnit calls — neither leaves anything in the recording directory, so
 * the "trigger a request and it will appear here" note is three wrong
 * sentences for them.
 */
const records = computed(() => ['profile', 'trace'].includes(status.value?.mode));

/**
 * The mode cannot be moved while the container is being rebuilt or recreated.
 *
 * Not tidiness: choosing a mode rewrites the compose overlay, and compose is
 * reading that file right now. The two racing produce a container whose
 * `XDEBUG_MODE` is neither of the things the screen said, and the only symptom
 * is a debugger that does not attach.
 *
 * It clears itself — the operation's finished event drops the busy flag, and
 * the watcher below re-reads — so this is a few seconds with a reason on
 * screen rather than a control that has gone quiet.
 */
const locked = computed(() => !!busy.value || ops.isBusy(props.name));

/**
 * A trace's own unit, which is not the profile's.
 *
 * `cost` formats whatever the cachegrind file declared — usually microseconds,
 * sometimes something else, and it reads that from the file rather than
 * assuming. A trace is always microseconds, because this app computes them
 * from the timestamps itself.
 */
const micros = (value) =>
  value >= 1000 ? `${(value / 1000).toFixed(1)} ms` : `${Math.round(value)} µs`;

watch(
  () => [props.name, props.runtime],
  () => load(props.runtime),
  { immediate: true }
);

/**
 * Re-read when the recreate this pane asked for finishes.
 *
 * `compose_up_project` returns an operation id as soon as the work starts, not
 * when it ends — so the caller's `await` resolves while docker is still
 * recreating the container. This pane never re-read at all, which is why the
 * "the container is in debug, the setting is profile" warning survived pressing
 * the button that fixed it: the work was done and nothing on screen knew.
 *
 * The falling edge of the busy flag, because that is set by the operation's own
 * finished event rather than by the call returning.
 */
watch(
  () => ops.isBusy(props.name),
  (busyNow, wasBusy) => {
    if (wasBusy && !busyNow) load(props.runtime);
  }
);
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-profiler"
      icon="mdi-speedometer"
      :title="t('profiler.title')"
      :description="t('profiler.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <template v-if="status">
      <!-- Compiled in first. Without the extension there is nothing to
           switch a mode on. -->
      <v-alert v-if="!status.xdebug.enabled" type="info" variant="tonal" class="mb-4">
        <div class="text-caption">{{ t('profiler.needsXdebug') }}</div>
      </v-alert>

      <template v-else>
        <v-btn-toggle
          :model-value="status.mode"
          mandatory
          density="comfortable"
          variant="outlined"
          divided
          @update:model-value="setMode($event)"
        >
          <v-btn value="debug" :disabled="locked" prepend-icon="mdi-bug-outline">
            {{ t('profiler.modeDebug') }}
          </v-btn>
          <v-btn value="profile" :disabled="locked" prepend-icon="mdi-speedometer">
            {{ t('profiler.modeProfile') }}
          </v-btn>
          <!-- A third mode rather than a checkbox on profiling: it writes
               a different file, read by a different parser, and it is the only
               one of the three that can produce a real flame graph. -->
          <v-btn value="trace" :disabled="locked" prepend-icon="mdi-fire">
            {{ t('profiler.modeTrace') }}
          </v-btn>
          <!-- The fourth mode, and the only one that records nothing of its
               own: it switches on the API PHPUnit calls, and PHPUnit writes
               the report. So there is no list below for it. -->
          <v-btn value="coverage" :disabled="locked" prepend-icon="mdi-shield-check-outline">
            {{ t('profiler.modeCoverage') }}
          </v-btn>
        </v-btn-toggle>
        <div class="text-caption text-medium-emphasis mt-2">
          {{ ops.isBusy(name) ? t('profiler.lockedWhileWorking') : t('profiler.modesExclusive') }}
        </div>

        <!-- Not a fifth button: `xdebug.mode` is a list and this is the second
             item in it, so it rides alongside whichever mode is chosen. -->
        <v-switch
          :model-value="status.develop"
          color="primary"
          density="compact"
          hide-details
          class="mt-2"
          :disabled="!!busy"
          :label="t('profiler.develop')"
          @update:model-value="setDevelop($event)"
        />
        <div class="text-caption text-medium-emphasis">
          {{ t('profiler.developDetail') }}
        </div>
        <div v-if="status.develop" class="text-caption text-medium-emphasis mt-1">
          <code>XDEBUG_MODE={{ status.modeValue }}</code>
        </div>

        <!-- The step people miss. Profiling waits for a trigger, so
             loading the page changes nothing until it carries one. -->
        <v-alert v-if="records" type="info" variant="tonal" class="mt-4">
          <div class="text-caption">
            {{ t('profiler.howToRecord', { trigger: status.trigger }) }}
          </div>
          <div v-if="status.mode === 'trace'" class="text-caption mt-1">
            {{ t('profiler.traceCost') }}
          </div>
        </v-alert>
        <!-- Coverage has no trigger and produces no file here, so the note
             above would be three wrong sentences. -->
        <v-alert v-else-if="status.mode === 'coverage'" type="info" variant="tonal" class="mt-4">
          <div class="text-caption">{{ t('profiler.coverageNote') }}</div>
        </v-alert>
        <!-- Fires for either mode, not just profiling: switching back
             to stepping leaves the container profiling, and that is the
             same silence pointing the other way. -->
        <v-alert v-if="needsRestart" type="warning" variant="tonal" class="mt-3">
          <div class="text-caption">{{ t('profiler.needsRecreate') }}</div>
          <div v-if="status.xdebug.activeMode" class="text-caption mt-1">
            {{
              t('profiler.modeMismatch', {
                running: status.xdebug.activeMode,
                wanted: status.mode,
              })
            }}
          </div>
          <v-btn
            size="small"
            color="warning"
            variant="tonal"
            class="mt-2"
            prepend-icon="mdi-autorenew"
            :loading="ops.isBusy(name)"
            @click="emit('apply')"
          >
            {{ t('projectDetail.applyToContainer') }}
          </v-btn>
        </v-alert>
      </template>

      <div class="section-head mt-5 mb-2 d-flex align-center">
        <v-icon size="18" class="mr-2">mdi-file-chart-outline</v-icon>
        {{ t('profiler.recorded', { n: status.profiles.length }) }}
        <v-spacer />
        <!-- One run of a tight loop produced 10 MB. Sixty delete buttons
             is not a disk-hygiene story. -->
        <v-btn
          v-if="status.profiles.length"
          size="x-small"
          variant="text"
          color="error"
          :loading="busy === 'clear'"
          @click="clear"
        >
          {{ t('profiler.clear', { size: bytes(status.bytes) }) }}
        </v-btn>
      </div>

      <div v-if="!status.profiles.length" class="text-caption text-medium-emphasis">
        {{ t('profiler.noneYet') }}
      </div>

      <div v-for="file in status.profiles" :key="file.id" class="cmd-row">
        <div class="flex-grow-1 min-width-0">
          <div class="mono text-body-2">{{ file.id }}</div>
          <div class="text-caption text-medium-emphasis">
            {{ bytes(file.bytes) }}
            <span v-if="file.modified">
              · {{ new Date(file.modified * 1000).toLocaleString() }}</span
            >
          </div>
        </div>
        <v-chip v-if="file.compressed" size="x-small" color="warning" variant="tonal">
          {{ t('profiler.compressed') }}
        </v-chip>
        <v-btn
          size="small"
          variant="tonal"
          :loading="busy === file.id"
          :disabled="file.compressed || !!busy"
          @click="open(file)"
        >
          {{ t('profiler.open') }}
        </v-btn>
        <!-- No confirmation: a profile is a recording you can make again
             by reloading the page, not something to lose. -->
        <v-btn
          icon
          size="x-small"
          variant="text"
          :aria-label="t('profiler.deleteOne')"
          :disabled="!!busy"
          @click="remove(file)"
        >
          <v-icon>mdi-delete-outline</v-icon>
          <v-tooltip activator="parent">{{ t('profiler.deleteOne') }}</v-tooltip>
        </v-btn>
      </div>

      <!-- Traces. A second list rather than rows mixed into the first:
           they are read by a different parser and open a different view, and a
           combined list would make somebody read the file name to know which. -->
      <template v-if="status.traces?.length">
        <div class="section-head mt-5 mb-2 d-flex align-center">
          <v-icon size="18" class="mr-2">mdi-fire</v-icon>
          {{ t('profiler.traces', { n: status.traces.length }) }}
        </div>
        <div v-for="file in status.traces" :key="file.id" class="cmd-row" data-test="trace-row">
          <div class="flex-grow-1 min-width-0">
            <div class="mono text-body-2">{{ file.id }}</div>
            <div class="text-caption text-medium-emphasis">
              {{ bytes(file.bytes) }}
              <span v-if="file.modified">
                · {{ new Date(file.modified * 1000).toLocaleString() }}</span
              >
            </div>
          </div>
          <v-btn
            size="small"
            variant="tonal"
            :loading="busy === file.id"
            :disabled="!!busy"
            @click="openTrace(file)"
          >
            {{ t('profiler.open') }}
          </v-btn>
          <v-btn
            icon
            size="x-small"
            variant="text"
            :aria-label="t('profiler.deleteOne')"
            :disabled="!!busy"
            @click="remove(file)"
          >
            <v-icon>mdi-delete-outline</v-icon>
            <v-tooltip activator="parent">{{ t('profiler.deleteOne') }}</v-tooltip>
          </v-btn>
        </div>
      </template>

      <!-- An open trace: the graph on its own. A trace has no per-function
           aggregate to tabulate — that is what a profile is for. -->
      <template v-if="flame">
        <div class="section-head mt-5 mb-1">
          <v-icon size="18" class="mr-2">mdi-fire</v-icon>{{ openId }}
        </div>
        <div class="text-caption text-medium-emphasis mb-2" data-test="flame-summary">
          {{
            t('profiler.flameSummary', {
              records: flame.records,
              stacks: flame.stacks,
              total: (flame.total / 1000).toFixed(1),
            })
          }}
        </div>
        <v-alert
          v-if="flame.truncated || flame.pruned || flame.depthCapped"
          type="info"
          variant="tonal"
          density="compact"
          class="mb-2"
        >
          <div v-if="flame.truncated" class="text-caption">{{ t('profiler.traceTruncated') }}</div>
          <div v-if="flame.pruned" class="text-caption">
            {{ t('profiler.tracePruned', { n: flame.pruned }) }}
          </div>
          <div v-if="flame.depthCapped" class="text-caption">
            {{ t('profiler.traceDepthCapped') }}
          </div>
        </v-alert>
        <FlameView :frames="flame.frames" :format="micros" class="mb-3">
          <template #empty>{{ t('profiler.noTree') }}</template>
        </FlameView>
      </template>

      <!-- The report. Self cost, because it is the one this parser can
           state exactly and the one that answers "what is slow". -->
      <template v-if="report">
        <div class="section-head mt-5 mb-1">
          <v-icon size="18" class="mr-2">mdi-podium</v-icon>{{ openId }}
        </div>
        <div class="text-caption text-medium-emphasis mb-2">
          {{
            t('profiler.summary', {
              n: report.functionCount,
              total: cost(report.selfTotal),
              creator: report.creator,
            })
          }}
        </div>
        <!-- The table says where the time went; this says what called it.
             Behind a button rather than open, because the tree is thousands of
             nodes and most visits to this pane want the top of the table. -->
        <div class="d-flex align-center ga-2 mb-2">
          <v-btn
            size="small"
            variant="tonal"
            prepend-icon="mdi-fire"
            :loading="treeBusy"
            @click="loadTree"
          >
            {{ t('profiler.flame') }}
          </v-btn>
          <span class="text-caption text-medium-emphasis">{{ t('profiler.flameHint') }}</span>
        </div>

        <FlameView v-if="tree" :frames="tree" :format="cost" class="mb-3">
          <template #empty>{{ t('profiler.noTree') }}</template>
        </FlameView>

        <v-alert
          v-if="report.truncated"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-2"
        >
          <div class="text-caption">{{ t('profiler.truncated') }}</div>
        </v-alert>

        <v-table density="compact">
          <thead>
            <tr>
              <th>{{ t('profiler.colFunction') }}</th>
              <th class="text-right">{{ t('profiler.colSelf') }}</th>
              <th class="text-right">{{ t('profiler.colInclusive') }}</th>
              <th class="text-right">{{ t('profiler.colCalls') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="fn in report.functions" :key="fn.name">
              <td class="min-width-0">
                <div class="mono text-truncate">{{ fn.name }}</div>
                <!-- The bar is the percentage, so the eye finds the hot
                     row before it reads a single number. -->
                <v-progress-linear
                  :model-value="fn.percent"
                  height="3"
                  color="primary"
                  class="mt-1"
                />
              </td>
              <td class="text-right mono">
                {{ cost(fn.selfTime) }}
                <div class="text-caption text-medium-emphasis">{{ fn.percent.toFixed(1) }}%</div>
              </td>
              <td class="text-right mono">{{ cost(fn.inclusiveTime) }}</td>
              <td class="text-right mono">{{ fn.calls }}</td>
            </tr>
          </tbody>
        </v-table>
      </template>
    </template>
  </v-card>
</template>
