<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * `git bisect`, with the environment the commit was written against.
 *
 * ## What this pane adds, and it is not the bisect
 *
 * `git bisect` is thirty years old and nothing here improves on it. What is on
 * screen that a terminal cannot show is the **drift**: what runtime and which
 * service versions the commit under test expected, and how this machine
 * differs. That difference is why a bisect can accuse an innocent commit — old
 * code against a new runtime is not the experiment anybody thought they were
 * running.
 *
 * ## It reports and does not install
 *
 * There is deliberately no "bring the environment along" button. Doing it would
 * mean replacing containers whose volumes hold the developer's data, twenty
 * times over a ten-step search, to answer a question about a diff.
 *
 * ## Three buttons, not two
 *
 * `skip` sits beside good and bad because git offers it, and leaving it out is
 * worse than it looks: a commit that does not build has to be marked
 * *something*, and without a third answer people mark it good — which poisons
 * the search in a way nothing downstream can detect.
 */
const props = defineProps({
  name: { type: String, required: true },
  hasGit: { type: Boolean, default: false },
});

const { t } = useI18n();

const status = ref(null);
const error = ref(null);
const busy = ref(false);

// Sensible defaults, and only these two: `HEAD` is where somebody noticed the
// problem and a revision they have to think about is the other end. Guessing
// the good one would be guessing the answer.
const bad = ref('HEAD');
const good = ref('');

const running = computed(() => status.value?.running === true);
const found = computed(() => status.value?.culprit ?? null);
const drift = computed(() => status.value?.drift ?? []);

async function refresh() {
  try {
    status.value = await api.bisectStatus(props.name);
    error.value = null;
  } catch (e) {
    // Kept on screen rather than blanking the pane: "I could not ask git" and
    // "no bisect is running" are different answers.
    error.value = e;
  }
}

async function run(fn) {
  busy.value = true;
  error.value = null;
  try {
    status.value = await fn();
  } catch (e) {
    error.value = e;
    // The refusal may have left git somewhere; ask rather than assume.
    await refresh();
  } finally {
    busy.value = false;
  }
}

const start = () => run(() => api.bisectStart(props.name, bad.value.trim(), good.value.trim()));
const mark = (verdict) => run(() => api.bisectMark(props.name, verdict));
const reset = () =>
  run(async () => {
    await api.bisectReset(props.name);
    return await api.bisectStatus(props.name);
  });

// Asked when the pane's project changes, and never polled: a bisect only moves
// when somebody presses one of these buttons.
watch(
  () => props.name,
  () => {
    status.value = null;
    error.value = null;
    if (props.hasGit) refresh();
  },
  { immediate: true }
);
</script>

<template>
  <section class="pane">
    <PaneHeader
      help="project-bisect"
      icon="mdi-source-branch-check"
      :title="t('bisect.title')"
      :description="t('bisect.desc')"
    />

    <!-- A directory that was never versioned has no history to search, and
         saying so beats a form that refuses on submit. -->
    <v-alert v-if="!hasGit" type="info" variant="tonal" density="compact" class="text-caption">
      {{ t('bisect.noRepository') }}
    </v-alert>

    <template v-else>
      <ErrorAlert v-if="error" :error="error" class="mb-3" />

      <!-- ---- not started ------------------------------------------------ -->
      <template v-if="!running">
        <p class="text-caption text-medium-emphasis mb-3">{{ t('bisect.explain') }}</p>
        <div class="d-flex ga-3 flex-wrap align-start">
          <v-text-field
            v-model="bad"
            :label="t('bisect.bad')"
            :hint="t('bisect.badHint')"
            persistent-hint
            density="compact"
            variant="outlined"
            style="min-width: 180px"
          />
          <v-text-field
            v-model="good"
            :label="t('bisect.good')"
            :hint="t('bisect.goodHint')"
            persistent-hint
            density="compact"
            variant="outlined"
            style="min-width: 180px"
          />
          <v-btn
            :loading="busy"
            :disabled="!bad.trim() || !good.trim()"
            size="small"
            variant="flat"
            color="primary"
            class="mt-1"
            data-test="bisect-start"
            @click="start"
          >
            {{ t('bisect.start') }}
          </v-btn>
        </div>
      </template>

      <!-- ---- running, or finished --------------------------------------- -->
      <template v-else>
        <v-alert
          v-if="found"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-3 text-caption"
          data-test="bisect-culprit"
        >
          {{ t('bisect.found', { commit: found }) }}
        </v-alert>

        <div class="d-flex align-center ga-3 flex-wrap mb-3">
          <div>
            <code class="text-body-2">{{ status.commit }}</code>
            <div class="text-caption text-medium-emphasis">{{ status.subject }}</div>
          </div>
          <v-spacer />
          <!-- Git's own estimate, not one computed here. -->
          <span v-if="status.steps !== undefined && !found" class="text-caption">
            {{ t('bisect.steps', { count: status.steps }) }}
          </span>
        </div>

        <!-- The half a terminal cannot show. Empty is a real answer and it is
             said out loud: the environment is not in this bisect, so whatever
             the search accuses is the code. -->
        <v-alert
          v-if="!drift.length"
          type="success"
          variant="tonal"
          density="compact"
          class="mb-3 text-caption"
        >
          {{ t('bisect.noDrift') }}
        </v-alert>
        <template v-else>
          <div class="text-caption text-medium-emphasis mb-1">{{ t('bisect.driftTitle') }}</div>
          <v-list density="compact" class="bg-transparent pa-0 mb-3">
            <v-list-item
              v-for="row in drift"
              :key="`${row.id}-${row.subject}`"
              class="px-0"
              data-test="drift"
            >
              <template #prepend>
                <v-icon color="warning" size="18" class="mr-3">mdi-alert-circle-outline</v-icon>
              </template>
              <v-list-item-title class="text-body-2">
                {{ t(`bisect.drift.${row.id}`, { subject: row.subject, wanted: row.wanted }) }}
              </v-list-item-title>
              <v-list-item-subtitle class="text-caption">
                {{
                  row.found ? t('bisect.driftFound', { found: row.found }) : t('bisect.driftAbsent')
                }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
          <!-- No button. Downgrading a service replaces a container whose
               volume holds the developer's data, and doing that twenty times
               to answer a question about a diff is not this app's call. -->
          <p class="text-caption text-medium-emphasis mb-3">{{ t('bisect.driftHint') }}</p>
        </template>

        <div class="d-flex ga-2 flex-wrap">
          <template v-if="!found">
            <v-btn
              :loading="busy"
              size="small"
              variant="flat"
              color="error"
              data-test="bisect-bad"
              @click="mark('bad')"
            >
              {{ t('bisect.markBad') }}
            </v-btn>
            <v-btn
              :loading="busy"
              size="small"
              variant="flat"
              color="success"
              data-test="bisect-good"
              @click="mark('good')"
            >
              {{ t('bisect.markGood') }}
            </v-btn>
            <v-btn :loading="busy" size="small" variant="tonal" @click="mark('skip')">
              {{ t('bisect.markSkip') }}
            </v-btn>
          </template>
          <v-spacer />
          <!-- Offered after the answer as well as during the search: the found
               screen is a detached HEAD like every other step, and somebody who
               has copied the hash still has a repository to put back. -->
          <v-btn
            :loading="busy"
            size="small"
            variant="text"
            data-test="bisect-reset"
            @click="reset"
          >
            {{ t('bisect.reset') }}
          </v-btn>
        </div>
      </template>
    </template>
  </section>
</template>
