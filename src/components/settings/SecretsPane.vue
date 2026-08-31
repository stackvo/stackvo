<script setup>
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import SettingsGroup from '@/components/SettingsGroup.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';

/**
 * Where each credential lives, and the one button that changes it.
 *
 * The pane exists because the decision has a cost the user has to be told
 * before they make it: `stackvo.sh` reads `.env` line by line and would take
 * `keychain:…` for the password itself. So this is a row per key with a switch,
 * not a "secure my passwords" button — twelve silent decisions is what a sweep
 * would be.
 *
 * It also says what the move does **not** do. The password is still rendered
 * into `generated/docker-compose.dynamic.yml`, as it always has been, and a
 * keystore feature is normally read as meaning otherwise. `secrets.rs` carries
 * the reasoning; this pane carries the sentence.
 */
const { t } = useI18n();

const status = ref({ available: false, keys: [] });
const error = ref(null);
const busy = ref(null);
const loading = ref(false);

/**
 * The other direction.
 *
 * This pane moves a credential out of `.env` once somebody knows there is one.
 * Nothing said *"there is one, and it is also in a file git is tracking"* — so
 * nobody found out until the repository was already public. Asked rather than
 * run on mount: it reads every tracked file in a repository, which is work
 * nobody asked for while they were looking at a list of keys.
 */
const scan = ref(null);
const scanning = ref(false);
const scanError = ref(null);

/**
 * Whether anything found is a **model provider's** key.
 *
 * Not a severity — every finding here is a credential and none is more of one
 * than the others. What these three share is a second question, and it is one
 * this app already answers from Docker rather than by inference: an application
 * that sends every request to an outside model is one where *"which of my
 * containers can reach the internet at all"* stops being theoretical. The
 * sentence points at the egress card that answers it.
 */
const MODEL_PROVIDER_RULES = ['openaiKey', 'googleApiKey', 'anthropicKey'];
const modelKeyFound = computed(() =>
  (scan.value?.findings ?? []).some((finding) => MODEL_PROVIDER_RULES.includes(finding.rule))
);
const projects = ref([]);
/** `null` scans only this machine's `.env`; a project adds its repository. */
const project = ref(null);

async function findLeaks() {
  scanning.value = true;
  scanError.value = null;
  try {
    scan.value = await api.leaksScan(project.value ?? undefined);
  } catch (e) {
    scan.value = null;
    scanError.value = e;
  } finally {
    scanning.value = false;
  }
}

/**
 * The repair, and the two halves it does not do.
 *
 * A finding people cannot act on is a finding they turn off — and this one is
 * easy to get half right: the common half-fix is deleting the file in a later
 * commit, which takes it out of the working tree and leaves every value in the
 * history. So the button does the mechanical part and the result says the rest.
 */
const untracking = ref(false);
const untracked = ref(null);

async function untrackEnv() {
  untracking.value = true;
  scanError.value = null;
  try {
    untracked.value = await api.envUntrack(project.value);
    // Re-read, so the finding that prompted this stops being on screen once it
    // is no longer true.
    scan.value = await api.leaksScan(project.value ?? undefined);
  } catch (e) {
    untracked.value = null;
    scanError.value = e;
  } finally {
    untracking.value = false;
  }
}

const moved = computed(() => status.value.keys.filter((k) => k.moved));
const broken = computed(() => moved.value.filter((k) => !k.resolvable));

/** Keys with no value are not offered: there is nothing to move. */
const rows = computed(() => status.value.keys.filter((k) => k.set));

async function load() {
  loading.value = true;
  error.value = null;
  try {
    status.value = await api.secretsStatus();
    // Not fatal: a machine with no workspace still has a keystore and a list.
    projects.value = (await api.projectsList().catch(() => []))?.map((p) => p.name) ?? [];
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}

async function toggle(row) {
  busy.value = row.key;
  error.value = null;
  try {
    if (row.moved) await api.secretRestore(row.key);
    else await api.secretMove(row.key);
    await load();
  } catch (e) {
    // Re-read first, then report. A failed move may still have written the
    // keystore entry, so a row showing the old state would be a claim about the
    // disk that nobody checked — but `load()` clears `error` on the way in, so
    // setting it before the re-read wipes the only thing that told the user
    // anything happened. The test that caught this asserted the message was on
    // screen; the pane was showing a fresh, silent, apparently-fine list.
    await load();
    error.value = e;
  } finally {
    busy.value = null;
  }
}

onMounted(load);
</script>

<template>
  <ErrorAlert v-if="error" :error="error" class="mb-4" />

  <SettingsGroup
    help="settings-secrets"
    icon="mdi-key-chain-variant"
    :title="t('settings.secrets.title')"
    :description="t('settings.secrets.description')"
  >
    <!-- Said before the switch, not after it. -->
    <v-alert type="info" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.secrets.whatItDoes') }}</div>
      <div class="text-caption mt-2 text-medium-emphasis">
        {{ t('settings.secrets.stillGenerated') }}
      </div>
      <div class="text-caption mt-1 text-medium-emphasis">
        {{ t('settings.secrets.cliCannotRead') }}
      </div>
    </v-alert>

    <v-alert
      v-if="!status.available"
      type="warning"
      variant="tonal"
      density="comfortable"
      class="mb-4"
      :text="t('settings.secrets.noKeystore')"
    />

    <v-alert v-if="broken.length" type="error" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.secrets.unresolvable') }}</div>
      <div class="text-caption mt-1">{{ broken.map((k) => k.key).join(', ') }}</div>
    </v-alert>

    <v-progress-linear v-if="loading" indeterminate class="mb-2" />

    <div v-if="!rows.length && !loading" class="text-body-2 text-medium-emphasis">
      {{ t('settings.secrets.none') }}
    </div>

    <v-list v-else density="compact" class="bg-transparent">
      <v-list-item v-for="row in rows" :key="row.key" class="px-0">
        <template #prepend>
          <v-icon
            :icon="row.moved ? 'mdi-key-chain-variant' : 'mdi-file-document-outline'"
            :color="row.moved ? (row.resolvable ? 'success' : 'error') : undefined"
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2 font-monospace">{{ row.key }}</v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ row.moved ? t('settings.secrets.inKeystore') : t('settings.secrets.inEnvFile') }}
        </v-list-item-subtitle>

        <template #append>
          <v-btn
            size="small"
            variant="tonal"
            :color="row.moved ? undefined : 'primary'"
            :loading="busy === row.key"
            :disabled="!status.available || (busy !== null && busy !== row.key)"
            @click="toggle(row)"
          >
            {{ row.moved ? t('settings.secrets.restore') : t('settings.secrets.move') }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <!-- The scan. Below the list because it is about the same values from the
         other side: this pane says where a credential lives, and that says
         where one is that nobody moved. -->
    <v-divider class="my-4" />
    <div class="text-caption text-medium-emphasis mb-2">{{ t('leaks.explain') }}</div>
    <div class="d-flex align-center ga-2 flex-wrap mb-1">
      <!-- Without a project only this machine's .env is read. With one, every
           file git is tracking in that repository — which is the half that
           leaves the machine. -->
      <v-select
        v-model="project"
        :items="projects"
        :label="t('leaks.project')"
        :placeholder="t('leaks.machineOnly')"
        persistent-placeholder
        clearable
        density="compact"
        variant="outlined"
        hide-details
        style="min-width: 14rem"
      />
      <v-btn
        size="small"
        variant="tonal"
        prepend-icon="mdi-magnify-scan"
        :loading="scanning"
        @click="findLeaks"
      >
        {{ t('leaks.run') }}
      </v-btn>
    </div>

    <ErrorAlert v-if="scanError" :error="scanError" class="mt-3" />

    <template v-if="scan">
      <!-- The finding that outranks every other one: it means every value in
           the file is in the history whatever its shape. -->
      <v-alert
        v-if="scan.envTracked || scan.envInHistory"
        :type="scan.envTracked ? 'error' : 'warning'"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
      >
        <div>{{ scan.envTracked ? t('leaks.envTracked') : t('leaks.envInHistory') }}</div>
        <!-- The repair, offered only where it can be made: it needs a
             repository, and without a project named there is none. -->
        <v-btn
          v-if="scan.envTracked && project"
          size="small"
          variant="tonal"
          class="mt-2"
          prepend-icon="mdi-wrench-outline"
          :loading="untracking"
          @click="untrackEnv"
        >
          {{ t('leaks.untrack') }}
        </v-btn>
      </v-alert>

      <!-- What it did, and the two halves it did not: the history it cannot
           rewrite, and the commit it is not this app's to make. -->
      <v-alert
        v-if="untracked"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="untracked"
      >
        <div>
          {{
            t('leaks.untrackedDone', {
              example: untracked.exampleKeys,
            })
          }}
        </div>
        <div v-if="untracked.needsCommit" class="mt-1">{{ t('leaks.needsCommit') }}</div>
        <div v-if="untracked.stillInHistory" class="mt-1 font-weight-medium">
          {{ t('leaks.rotate') }}
        </div>
      </v-alert>

      <v-alert
        v-if="!scan.findings.length && !scan.envTracked"
        type="success"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
      >
        {{ t('leaks.none', { scanned: scan.scanned }) }}
      </v-alert>

      <!-- The sentence beside a model provider's key. It adds no finding and
           changes no severity: it names the question that follows from having
           one, and the card that already answers it. -->
      <v-alert
        v-if="modelKeyFound"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="leak-model-egress"
      >
        {{ t('leaks.modelKeyEgress') }}
      </v-alert>

      <v-list
        v-if="scan.findings.length || scan.envTracked"
        density="compact"
        class="bg-transparent pa-0 mt-2"
      >
        <v-list-item
          v-for="(finding, i) in scan.findings"
          :key="`${finding.subject}-${i}`"
          class="px-0"
          data-test="leak"
        >
          <template #prepend>
            <v-icon color="warning" size="18" class="mr-3">mdi-alert-circle-outline</v-icon>
          </template>
          <v-list-item-title class="text-body-2">
            <code>{{ finding.subject }}</code>
            <span v-if="finding.line" class="text-medium-emphasis">:{{ finding.line }}</span>
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption">
            {{ t(`leaks.rule.${finding.id}`) }} — {{ t(`leaks.source.${finding.source}`) }}
            <span v-if="finding.inHistory"> · {{ t('leaks.committed') }}</span>
          </v-list-item-subtitle>
          <!-- Enough to recognise which secret this is, and never enough to be
               it: the ends of the value, and a fingerprint two rows share when
               they are one key in two places. -->
          <template #append>
            <span class="text-caption text-medium-emphasis mono">
              {{ finding.preview }} · {{ finding.fingerprint }}
            </span>
          </template>
        </v-list-item>
      </v-list>

      <!-- A scan that passed over four hundred files and said nothing reads as
           a clean repository. -->
      <div v-if="scan.skipped" class="text-caption text-medium-emphasis mt-2">
        {{ t('leaks.skipped', { skipped: scan.skipped, scanned: scan.scanned }) }}
      </div>
    </template>
  </SettingsGroup>
</template>
