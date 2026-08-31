<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * F-2 — Telescope, Horizon and Pulse.
 *
 * ## Why this is not three links
 *
 * All three open in `local` with no authentication, and this app already serves
 * the project on its own domain, so `https://shop.loc/horizon` works today.
 * Nothing said so, which is why nobody clicked it — but a link on its own would
 * leave the useful half undone. Each of the three goes **quietly empty** for a
 * reason the developer cannot see, and every container is green while it
 * happens.
 *
 * ## Every row says where it looked
 *
 * This app reads `.env` and `composer.lock`. It does not read `config/*.php`,
 * and a project that has run `config:cache` can make both of them lie. So a row
 * names the key and quotes the value, and the caveat sits **beside** it rather
 * than at the top of the pane — a warning up here and a row down there are two
 * things a reader has to join up themselves.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const report = ref(null);
const error = ref(null);
const loading = ref(false);
/** The `needs.id` currently being written into the schedule. */
const adding = ref(null);

/** Only the ones the project actually has. The rest are not this card's news. */
const boards = computed(() => (report.value?.boards ?? []).filter((b) => b.installed));

const missing = computed(() =>
  (report.value?.boards ?? []).filter((b) => !b.installed).map((b) => b.board)
);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    report.value = await api.dashboardsReport(props.name);
  } catch (e) {
    report.value = null;
    error.value = e;
  } finally {
    loading.value = false;
  }
}

/**
 * Add one offered job to this project's schedule.
 *
 * Through `schedulerSave` with the whole list, because that is the schedule's
 * only writer — a second verb here would be a second chance for `stackvo.json`
 * and the generated directory to disagree about what is scheduled.
 */
async function add(need) {
  adding.value = need.id;
  error.value = null;
  try {
    const view = await api.schedulerJobs(props.name);
    // The four fields the backend stores, and nothing the status view added on
    // top of them — sending an `id` or a `lastRun` back would be this card
    // inventing schedule fields.
    const kept = (view?.jobs ?? []).map((job) => ({
      label: job.label,
      cron: job.cron,
      exec: job.exec,
      enabled: job.enabled !== false,
    }));
    await api.schedulerSave(props.name, [...kept, need.job]);
    report.value = await api.dashboardsReport(props.name);
  } catch (e) {
    error.value = e;
  } finally {
    adding.value = null;
  }
}

watch(
  () => props.name,
  () => {
    report.value = null;
    error.value = null;
  }
);
</script>

<template>
  <section class="pane">
    <PaneHeader
      help="project-dashboards"
      icon="mdi-view-dashboard-outline"
      :title="t('boards.title')"
      :description="t('boards.desc')"
    />

    <v-btn
      size="small"
      variant="tonal"
      prepend-icon="mdi-magnify"
      :loading="loading"
      data-test="boards-read"
      @click="load"
    >
      {{ t('boards.read') }}
    </v-btn>

    <ErrorAlert v-if="error" :error="error" class="mt-3" />

    <template v-if="report">
      <!-- "Nothing was read" and "nothing is wrong" are different answers. -->
      <v-alert
        v-if="!report.readEnv"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="boards-noenv"
      >
        {{ t('boards.noEnv') }}
      </v-alert>

      <v-alert
        v-if="!boards.length"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="boards-none"
      >
        {{ t('boards.noneInstalled', { names: missing.join(', ') }) }}
      </v-alert>

      <div v-for="board in boards" :key="board.board" class="mt-4" data-test="board">
        <div class="text-body-2 font-weight-medium">
          {{ t(`boards.${board.board}.title`) }}
          <span class="text-caption text-medium-emphasis">{{ board.installed }}</span>
        </div>
        <p class="text-caption text-medium-emphasis mb-2">{{ t(`boards.${board.board}.what`) }}</p>

        <p v-if="board.url" class="text-caption mb-2">
          <a :href="board.url" target="_blank" rel="noopener">{{ board.url }}</a>
          <span class="text-medium-emphasis"> — {{ t('boards.defaultPath') }}</span>
        </p>

        <!-- The observations. Each one names the key it read, which is what
             makes it an observation rather than a verdict. -->
        <v-list v-if="board.observations.length" density="compact" class="bg-transparent pa-0 mb-2">
          <v-list-item
            v-for="obs in board.observations"
            :key="obs.id"
            class="px-0"
            data-test="board-observation"
          >
            <template #prepend>
              <v-icon color="warning" size="18" class="mr-3">mdi-alert-circle-outline</v-icon>
            </template>
            <v-list-item-title class="text-body-2">
              {{ t(`boards.observation.${obs.id}`, { value: obs.value ?? '' }) }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ t('boards.readFrom', { key: obs.key }) }} {{ t('boards.cachedConfig') }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>

        <!-- The long processes. Named rather than started here: the Workers
             card owns starting one, and two buttons that start the same
             sidecar are two places that can disagree about whether it is up. -->
        <p
          v-for="worker in board.workers"
          :key="worker.kind"
          class="text-caption mb-1"
          data-test="board-worker"
        >
          <v-icon :color="worker.running ? 'success' : 'warning'" size="16" class="mr-2">
            {{ worker.running ? 'mdi-check-circle-outline' : 'mdi-stop-circle-outline' }}
          </v-icon>
          {{
            worker.running
              ? t('boards.workerUp', { name: t(`workers.${worker.kind}`) })
              : t('boards.workerDown', { name: t(`workers.${worker.kind}`) })
          }}
        </p>

        <!-- The scheduled commands. Offered as a job because the project
             already has a table of exactly this shape. -->
        <div v-for="need in board.needs" :key="need.id" class="mt-2" data-test="board-need">
          <p class="text-caption text-medium-emphasis mb-1">
            {{ t(`boards.need.${need.id}`) }}
            <code>{{ need.job.exec.join(' ') }}</code> — <code>{{ need.job.cron }}</code>
          </p>
          <v-chip v-if="need.scheduled" size="x-small" label color="success" data-test="board-has">
            {{ t('boards.alreadyScheduled') }}
          </v-chip>
          <v-btn
            v-else
            size="x-small"
            variant="tonal"
            :loading="adding === need.id"
            data-test="board-add"
            @click="add(need)"
          >
            {{ t('boards.addToSchedule') }}
          </v-btn>
        </div>

        <!-- Telescope's own precondition is a precondition, not a state: it is
             a question about a database this app does not query. -->
        <p
          v-if="board.board === 'telescope'"
          class="text-caption text-medium-emphasis mt-2"
          data-test="board-telescope-note"
        >
          {{ t('boards.telescope.migrations') }}
        </p>
      </div>

      <!-- Scout: the purest form of this card's pattern. A sentence and not a
           button — `scout:import` takes a model class name this application
           cannot know, and a button that filled in a guess and ran it is what
           the command catalogue refuses. -->
      <template v-if="report.scout">
        <v-divider class="my-3" />
        <p class="text-caption text-medium-emphasis" data-test="boards-scout">
          {{ t('boards.scout', { driver: report.scout.driver }) }}
        </p>
        <p class="text-caption text-medium-emphasis">
          {{ t('boards.scoutHow') }}
          <code>scout:import "App\Models\Post"</code>,
          <code>scout:sync-index-settings</code>
        </p>
      </template>
    </template>
  </section>
</template>
