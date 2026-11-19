<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Three per-project settings that reach the container through this app's own
 * layers rather than through the manifest.
 *
 * They share a pane because they share a file and a moment: somebody setting up
 * one project sets all three at once, and three panes would be three places to
 * look for the same `.stackvo/site.json`.
 *
 * ## Why each switch can be unavailable, and says so
 *
 * A directory listing is a **server** directive, and Apache and Swoole have no
 * configuration file for one — Apache is configured by `sed` inside its own
 * Dockerfile. Forwarding an SSH agent needs an agent to forward. In both cases
 * the backend answers whether it is possible here, and the pane draws the
 * reason rather than a control that would do nothing.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: '' },
});

const emit = defineEmits(['apply']);

const { t } = useI18n();

const settings = ref(null);
const rows = ref([]);
const error = ref(null);
const busy = ref(false);

/**
 * Host directories beside the project.
 *
 * `stored` is what the backend last answered, with whether each row is
 * approved on this machine; `mounts` is the editor. They are kept apart so a
 * row being typed does not lose its status chip, and so the chip always
 * describes what is on disk rather than what is in the text field.
 */
const stored = ref([]);
const mounts = ref([]);
const mountsBusy = ref(false);
const mountsError = ref(null);

function toMountRows(list) {
  return (list || []).map((m) => ({
    source: m.source,
    target: m.target,
    readOnly: m.readOnly,
  }));
}

/** The stored row an edited row still matches, for its status chip. */
function storedFor(row) {
  return stored.value.find(
    (s) => s.source === row.source && s.target === row.target && s.readOnly === row.readOnly
  );
}

const pendingMounts = computed(() => stored.value.some((s) => !s.approved && !s.error));

/** Node projects have a compose service too, so the variables apply there. */
const applies = computed(() => !!props.runtime);

/** The editor is a list so a half-typed key does not vanish on re-render. */
function toRows(env) {
  return Object.entries(env || {}).map(([key, value]) => ({ key, value }));
}

async function load() {
  if (!applies.value) return;
  error.value = null;
  try {
    settings.value = await api.siteSettings(props.name);
    rows.value = toRows(settings.value.env);
    stored.value = await api.siteMounts(props.name);
    mounts.value = toMountRows(stored.value);
  } catch (e) {
    error.value = e;
  }
}

/**
 * Saving is approving: every row sent is mounted on this machine from now on.
 * The backend refuses the whole list when one row does not resolve, so the
 * rows are only replaced from its answer, never from what was typed.
 */
async function saveMounts() {
  mountsBusy.value = true;
  mountsError.value = null;
  try {
    const list = mounts.value
      .map((m) => ({ source: m.source.trim(), target: m.target.trim(), readOnly: m.readOnly }))
      .filter((m) => m.source && m.target);
    stored.value = await api.siteMountsSave(props.name, list);
    mounts.value = toMountRows(stored.value);
    // A compose overlay: the running container keeps its old mounts until it
    // is recreated.
    emit('apply');
  } catch (e) {
    mountsError.value = e;
  } finally {
    mountsBusy.value = false;
  }
}

/**
 * Blank rows are dropped rather than sent: an empty key is what a row that is
 * being typed looks like, and the backend refusing it would turn every save
 * during editing into an error.
 */
async function save(patch = {}) {
  busy.value = true;
  error.value = null;
  try {
    const env = {};
    for (const row of rows.value) {
      const key = row.key.trim();
      if (key) env[key] = row.value;
    }
    settings.value = await api.siteSave(
      props.name,
      env,
      patch.directoryListing ?? settings.value.directoryListing,
      patch.sshAgent ?? settings.value.sshAgent
    );
    rows.value = toRows(settings.value.env);
    // The variables and the agent are a compose overlay: the running container
    // is still the old one until it is recreated.
    emit('apply');
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}

watch(() => [props.name, props.runtime], load, { immediate: true });
</script>

<template>
  <v-card v-if="applies && settings" variant="flat" class="pane">
    <PaneHeader
      help="project-site"
      icon="mdi-tune-variant"
      :title="t('site.title')"
      :description="t('site.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <div class="section-head mt-2 mb-2">
      <v-icon size="16" class="mr-2">mdi-code-braces</v-icon>{{ t('site.envTitle') }}
    </div>
    <p class="text-caption text-medium-emphasis mb-2">{{ t('site.envExplain') }}</p>

    <div v-for="(row, index) in rows" :key="index" class="d-flex ga-2 mb-2" data-test="env-row">
      <v-text-field
        v-model="row.key"
        :label="t('site.key')"
        density="compact"
        variant="outlined"
        hide-details
        class="flex-grow-0"
        style="max-width: 220px"
      />
      <v-text-field
        v-model="row.value"
        :label="t('site.value')"
        density="compact"
        variant="outlined"
        hide-details
      />
      <v-btn
        icon
        size="small"
        variant="text"
        :aria-label="t('site.removeRow')"
        @click="rows.splice(index, 1)"
      >
        <v-icon>mdi-close</v-icon>
      </v-btn>
    </div>

    <div class="d-flex ga-2 mb-4">
      <v-btn
        size="small"
        variant="text"
        prepend-icon="mdi-plus"
        @click="rows.push({ key: '', value: '' })"
      >
        {{ t('site.addRow') }}
      </v-btn>
      <v-spacer />
      <v-btn size="small" color="primary" variant="tonal" :loading="busy" @click="save()">
        {{ t('site.save') }}
      </v-btn>
    </div>

    <v-divider class="mb-3" />

    <div class="d-flex align-center ga-3">
      <v-switch
        :model-value="settings.directoryListing"
        color="primary"
        density="compact"
        hide-details
        :disabled="!settings.listingSupported || busy"
        :label="t('site.listing')"
        @update:model-value="save({ directoryListing: !settings.directoryListing })"
      />
    </div>
    <p class="text-caption text-medium-emphasis mb-3">
      {{
        settings.listingSupported
          ? t('site.listingHint')
          : t('site.listingUnsupported', { server: settings.server })
      }}
    </p>

    <div class="d-flex align-center ga-3">
      <v-switch
        :model-value="settings.sshAgent"
        color="primary"
        density="compact"
        hide-details
        :disabled="!settings.agentAvailable || busy"
        :label="t('site.sshAgent')"
        @update:model-value="save({ sshAgent: !settings.sshAgent })"
      />
    </div>
    <!-- The cost is on the switch that charges it: anything in that container
         can sign with every key in the agent for as long as it is up. -->
    <p
      class="text-caption mb-0"
      :class="settings.agentAvailable ? 'text-medium-emphasis' : 'text-warning'"
      data-test="ssh-hint"
    >
      {{ settings.agentAvailable ? t('site.sshAgentHint') : t('site.sshAgentNone') }}
    </p>

    <v-divider class="my-3" />

    <div class="section-head mb-2">
      <v-icon size="16" class="mr-2">mdi-folder-network-outline</v-icon>{{ t('site.mountsTitle') }}
    </div>
    <p class="text-caption text-medium-emphasis mb-2">{{ t('site.mountsExplain') }}</p>

    <ErrorAlert v-if="mountsError" :error="mountsError" class="mb-3" />

    <!-- A mount that arrived with a clone is shown, not applied, until it is
         saved here — the pane says so rather than leaving it to be guessed. -->
    <v-alert
      v-if="pendingMounts"
      type="warning"
      variant="tonal"
      density="compact"
      class="mb-3"
      data-test="mounts-pending"
    >
      {{ t('site.mountsPending') }}
    </v-alert>

    <div v-for="(row, index) in mounts" :key="index" class="mb-2" data-test="mount-row">
      <div class="d-flex ga-2 align-center">
        <v-text-field
          v-model="row.source"
          :label="t('site.mountSource')"
          placeholder="../cbglib"
          density="compact"
          variant="outlined"
          hide-details
        />
        <v-text-field
          v-model="row.target"
          :label="t('site.mountTarget')"
          placeholder="/var/www/cbglib"
          density="compact"
          variant="outlined"
          hide-details
        />
        <v-checkbox
          v-model="row.readOnly"
          :label="t('site.mountReadOnly')"
          density="compact"
          hide-details
          class="flex-grow-0"
        />
        <v-btn
          icon
          size="small"
          variant="text"
          :aria-label="t('site.removeMount')"
          @click="mounts.splice(index, 1)"
        >
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </div>
      <template v-if="storedFor(row)">
        <p v-if="storedFor(row).error" class="text-caption text-error mb-0 mt-1">
          {{ storedFor(row).error }}
        </p>
        <p
          v-else
          class="text-caption mb-0 mt-1"
          :class="storedFor(row).approved ? 'text-medium-emphasis' : 'text-warning'"
        >
          {{
            storedFor(row).approved
              ? t('site.mountApproved', { path: storedFor(row).resolved })
              : t('site.mountNotApproved', { path: storedFor(row).resolved })
          }}
        </p>
      </template>
    </div>

    <div class="d-flex ga-2 mb-1">
      <v-btn
        size="small"
        variant="text"
        prepend-icon="mdi-plus"
        @click="mounts.push({ source: '', target: '', readOnly: true })"
      >
        {{ t('site.addMount') }}
      </v-btn>
      <v-spacer />
      <v-btn
        size="small"
        color="primary"
        variant="tonal"
        :loading="mountsBusy"
        data-test="mounts-save"
        @click="saveMounts()"
      >
        {{ t('site.saveMounts') }}
      </v-btn>
    </div>
    <p class="text-caption text-medium-emphasis mb-0">{{ t('site.mountsConsent') }}</p>
  </v-card>
</template>
