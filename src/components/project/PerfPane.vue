<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import { bytes } from '@/lib/format';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * The performance layer (I-1): the heavy directories, off the host filesystem.
 *
 * ## Why this is a list of directories and not a switch
 *
 * A bind mount crosses a filesystem boundary on macOS and Windows, and the
 * measured win depends entirely on *which* directory moves: `vendor` in a
 * volume buys the framework **boot** and does nothing at all for writes;
 * `storage/framework` is the one that buys the **writes**. A single "make it
 * fast" switch would hide that, and the directories it moved would be a guess
 * about somebody's project.
 *
 * ## The number is data now, and this comment is no longer where it lives
 *
 * This block used to hold the benchmark's output table — which made a Vue
 * file's comment the only written record of what `examples/perf_layer_bench.rs`
 * printed, unreachable from the program that produced it. Worse, `perf.explain`
 * restated a rounded **average** of the two rows, in two languages, as prose a
 * translator had to re-derive; and averaging them hides exactly the fact above.
 *
 * It is `perf::GAINS` now, arrives on each `layer.gain`, and renders on the row
 * it was measured for. The bench records what it printed and
 * `src-tauri/tests/perf_claims.rs` holds the three apart — including a check
 * that no locale has quietly grown the figure back.
 *
 * ## Two things this pane must never do quietly
 *
 * **Turning a layer on empties the container's view of that directory** unless
 * the host copy is put in first, so the backend seeds it and the switch is not
 * reported as done until that has happened.
 *
 * **The editor stops seeing it.** That is the price, and it is stated on the
 * row rather than discovered when autocomplete goes quiet — with the button
 * that copies a snapshot back out for the index.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: '' },
});

const emit = defineEmits(['apply']);

const { t } = useI18n();

const layers = ref([]);
const error = ref(null);
const busy = ref('');
const note = ref('');

/** Only the runtimes whose projects have a directory worth moving. */
const applies = computed(() => ['php', 'node'].includes(props.runtime));
const anyEnabled = computed(() => layers.value.some((l) => l.enabled));

async function load() {
  if (!applies.value) return;
  error.value = null;
  try {
    layers.value = asList(await api.perfStatus(props.name));
  } catch (e) {
    error.value = e;
  }
}

async function run(key, fn) {
  busy.value = key;
  error.value = null;
  note.value = '';
  try {
    return await fn();
  } catch (e) {
    error.value = e;
    return null;
  } finally {
    busy.value = '';
  }
}

const toggle = (layer) =>
  run(`set:${layer.path}`, async () => {
    layers.value = asList(await api.perfSet(props.name, layer.path, !layer.enabled));
    // The container is still reading the old arrangement until it is recreated,
    // and that is the same "applied to disk, not to the container" state the
    // Xdebug pane names.
    emit('apply');
  });

const exportToHost = (layer) =>
  run(`export:${layer.path}`, async () => {
    const result = await api.perfExport(props.name, layer.path);
    note.value = t('perf.exported', { path: layer.path, size: bytes(result?.bytes ?? 0) });
    await load();
  });

const forget = (layer) =>
  run(`forget:${layer.path}`, async () => {
    layers.value = asList(await api.perfForget(props.name, layer.path));
  });

watch(() => [props.name, props.runtime], load, { immediate: true });
</script>

<template>
  <v-card v-if="applies" variant="flat" class="pane">
    <PaneHeader
      help="project-perf"
      icon="mdi-rocket-launch-outline"
      :title="t('perf.title')"
      :description="t('perf.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />
    <v-alert v-if="note" type="success" variant="tonal" density="compact" class="mb-3">
      <div class="text-caption">{{ note }}</div>
    </v-alert>

    <div v-if="!layers.length" class="text-caption text-medium-emphasis">
      {{ t('perf.nothingToOffer') }}
    </div>

    <div v-for="layer in layers" :key="layer.path" class="cmd-row" data-test="perf-layer">
      <div class="flex-grow-1 min-width-0">
        <div class="d-flex align-center ga-2 min-width-0">
          <span class="mono text-body-2">{{ layer.path }}</span>

          <!-- The measurement, on the row it was measured for. It used to be
               one averaged figure in the header sentence, which hid the fact
               the whole design rests on: these two rows buy different things
               and each is useless for the other's workload. -->
          <v-chip
            v-if="layer.gain"
            size="x-small"
            variant="tonal"
            color="success"
            :title="t('perf.measuredOn')"
          >
            {{
              t('perf.gain', {
                times: layer.gain.times,
                workload: t(`perf.workload.${layer.gain.workload}`),
              })
            }}
          </v-chip>

          <!-- Said, rather than left blank. `bootstrap/cache` and
               `node_modules` are offered and have never been through the
               bench; a row with no number is a row nobody measured, and
               borrowing a neighbour's would be the average all over again. -->
          <span v-else class="text-caption text-disabled">{{ t('perf.notMeasured') }}</span>
        </div>
        <div class="text-caption text-medium-emphasis">
          <template v-if="layer.enabled">
            {{ t('perf.inVolume', { volume: layer.volume }) }}
            <span v-if="layer.bytes"> · {{ bytes(layer.bytes) }}</span>
          </template>
          <template v-else-if="layer.hostFiles">
            {{ t('perf.onHost', { files: layer.hostFiles }) }}
          </template>
          <template v-else>{{ t('perf.notThereYet') }}</template>
        </div>
        <!-- The price of the row, on the row. -->
        <div v-if="layer.enabled" class="text-caption text-warning">
          {{ t('perf.editorCannotSee') }}
        </div>
      </div>

      <v-btn
        v-if="layer.enabled && layer.exists"
        size="x-small"
        variant="text"
        :loading="busy === `export:${layer.path}`"
        :disabled="!!busy"
        @click="exportToHost(layer)"
      >
        {{ t('perf.export') }}
      </v-btn>
      <!-- Deleting is its own act and never a side effect of the switch: what
           is in there may be the only copy. -->
      <v-btn
        v-if="!layer.enabled && layer.exists"
        size="x-small"
        variant="text"
        color="error"
        :loading="busy === `forget:${layer.path}`"
        :disabled="!!busy"
        @click="forget(layer)"
      >
        {{ t('perf.forget') }}
      </v-btn>
      <v-switch
        :model-value="layer.enabled"
        color="primary"
        density="compact"
        hide-details
        :loading="busy === `set:${layer.path}`"
        :disabled="!!busy"
        :aria-label="t('perf.toggle', { path: layer.path })"
        @update:model-value="toggle(layer)"
      />
    </div>

    <p v-if="anyEnabled" class="text-caption text-medium-emphasis mt-3 mb-0">
      {{ t('perf.needsRecreate') }}
    </p>
  </v-card>
</template>
