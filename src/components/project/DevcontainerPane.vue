<script setup>
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * This project as a devcontainer (A-7).
 *
 * Beside the release pane rather than in Configuration, because those panes
 * describe what the project *is* and these two are the artefacts that leave
 * the machine — one as an image, one as files in the repository.
 *
 * ## Read before written, and that is the whole shape of the pane
 *
 * The destination is somebody's git tree. `.devcontainer/` exists to be
 * committed, which is exactly why what lands there should be readable before
 * it is there — so the plan comes back first, with every file's contents, and
 * the write is a second press.
 *
 * Nothing is planned on mount either. The backend renders a Dockerfile and
 * every declared service's compose fragment to answer, and a tab that costs
 * that on every visit is a tab that pays for a question most people ask once.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const plan = ref(null);
const busy = ref(false);
const error = ref(null);
const written = ref(null);
const open = ref('');

async function preview() {
  busy.value = true;
  error.value = null;
  written.value = null;
  try {
    plan.value = await api.projectDevcontainerPlan(props.name);
    // The first file open, so the pane arrives showing something rather than a
    // list of names to press. `devcontainer.json` is the one a reader looks
    // for; it is also the shortest.
    open.value = plan.value.files.find((f) => f.path === 'devcontainer.json')?.path ?? '';
  } catch (e) {
    plan.value = null;
    error.value = e;
  } finally {
    busy.value = false;
  }
}

async function write() {
  busy.value = true;
  error.value = null;
  try {
    written.value = asList(await api.projectDevcontainerWrite(props.name));
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-devcontainer"
      icon="mdi-microsoft-visual-studio-code"
      :title="t('devcontainer.title')"
      :description="t('devcontainer.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <div class="d-flex align-center ga-2 mb-3">
      <v-btn
        size="small"
        variant="tonal"
        color="primary"
        prepend-icon="mdi-file-search-outline"
        :loading="busy && !plan"
        @click="preview"
      >
        {{ t('devcontainer.preview') }}
      </v-btn>

      <!-- Absent until there is a plan. A write button that has nothing to
           write is a button whose only answer is an error. -->
      <v-btn
        v-if="plan"
        size="small"
        variant="tonal"
        prepend-icon="mdi-content-save-outline"
        :loading="busy"
        @click="write"
      >
        {{ t('devcontainer.write', { n: plan.files.length }) }}
      </v-btn>
    </div>

    <v-alert v-if="written" type="success" variant="tonal" density="compact" class="mb-3">
      <div class="text-caption">{{ t('devcontainer.written', { n: written.length }) }}</div>
      <div v-for="path in written" :key="path" class="text-caption text-medium-emphasis">
        {{ path }}
      </div>
    </v-alert>

    <template v-if="plan">
      <!-- The passwords, first and by name. They are the one thing that does
           not travel, and finding that out after the commit is finding it out
           from a teammate. -->
      <v-alert
        v-if="plan.secrets.length"
        type="warning"
        variant="tonal"
        density="compact"
        class="mb-3"
      >
        <div class="text-caption">{{ t('devcontainer.secrets', { n: plan.secrets.length }) }}</div>
        <code v-for="key in plan.secrets" :key="key" class="d-block text-caption">{{ key }}</code>
      </v-alert>

      <v-alert
        v-if="plan.skipped.length"
        type="info"
        variant="tonal"
        density="compact"
        class="mb-3"
      >
        <div v-for="note in plan.skipped" :key="note" class="text-caption">{{ note }}</div>
      </v-alert>

      <v-expansion-panels v-model="open" variant="accordion" class="mb-3">
        <v-expansion-panel v-for="file in plan.files" :key="file.path" :value="file.path">
          <v-expansion-panel-title>
            <code class="text-caption">.devcontainer/{{ file.path }}</code>
          </v-expansion-panel-title>
          <v-expansion-panel-text>
            <pre class="file">{{ file.contents }}</pre>
          </v-expansion-panel-text>
        </v-expansion-panel>
      </v-expansion-panels>

      <ul class="notes">
        <li v-for="note in plan.notes" :key="note" class="text-caption text-medium-emphasis">
          {{ note }}
        </li>
      </ul>
    </template>
  </v-card>
</template>

<style scoped>
.file {
  max-height: 420px;
  overflow: auto;
  font-size: 0.75rem;
  line-height: 1.5;
  white-space: pre;
}

.notes {
  padding-left: 18px;
}
</style>
