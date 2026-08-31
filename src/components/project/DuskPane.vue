<script setup>
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * F-4 — a browser in a container, and a certificate it will accept.
 *
 * ## Two pieces, and neither works without the other
 *
 * The sidecar is the easy half — the manifest has supported one since W-01. The
 * hard half is the one no `docker-compose.yml` fixes: the browser has to open
 * `https://<domain>` from inside a container that does not know this machine's
 * certificate authority, and a test that dies on a certificate warning reads as
 * an application bug.
 *
 * ## Three results, not one tick
 *
 * The trust step writes the CA into two places that fail separately — the
 * system bundle and Chromium's NSS database — so each is reported as itself.
 * And it writes into the container's writable layer, so recreating the
 * container loses it. That is said beside the button rather than discovered.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const plan = ref(null);
const steps = ref(null);
const error = ref(null);
const loading = ref(false);
const busy = ref(null);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    plan.value = await api.duskPlan(props.name);
  } catch (e) {
    plan.value = null;
    error.value = e;
  } finally {
    loading.value = false;
  }
}

async function apply() {
  busy.value = 'apply';
  error.value = null;
  try {
    plan.value = await api.duskApply(props.name);
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = null;
  }
}

async function trust() {
  busy.value = 'trust';
  error.value = null;
  steps.value = null;
  try {
    steps.value = await api.duskTrust(props.name);
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = null;
  }
}

watch(
  () => props.name,
  () => {
    plan.value = null;
    steps.value = null;
    error.value = null;
  }
);
</script>

<template>
  <section class="pane">
    <PaneHeader
      help="project-dusk"
      icon="mdi-google-chrome"
      :title="t('dusk.title')"
      :description="t('dusk.desc')"
    />

    <v-btn
      size="small"
      variant="tonal"
      prepend-icon="mdi-magnify"
      :loading="loading"
      data-test="dusk-read"
      @click="load"
    >
      {{ t('dusk.read') }}
    </v-btn>

    <ErrorAlert v-if="error" :error="error" class="mt-3" />

    <template v-if="plan">
      <v-alert
        v-if="!plan.installed"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="dusk-none"
      >
        {{ t('dusk.notInstalled') }}
      </v-alert>

      <template v-else>
        <p class="text-caption text-medium-emphasis mt-3 mb-2">
          {{ t('dusk.version', { version: plan.installed }) }}
          <br />
          {{ t('dusk.image') }} <code>{{ plan.image }}</code>
          <br />
          {{ t('dusk.imageWhy') }}
        </p>

        <!-- Step one: the container. -->
        <div class="mb-3" data-test="dusk-sidecar">
          <p class="text-caption mb-1">
            <v-icon :color="plan.declared ? 'success' : 'warning'" size="16" class="mr-2">
              {{ plan.declared ? 'mdi-check-circle-outline' : 'mdi-circle-outline' }}
            </v-icon>
            {{ plan.declared ? t('dusk.declared') : t('dusk.notDeclared') }}
            <template v-if="plan.declared">
              — {{ plan.running ? t('dusk.up') : t('dusk.down') }}
            </template>
          </p>
          <p class="text-caption text-medium-emphasis mb-1">
            {{ plan.envFilePresent ? t('dusk.envPresent') : t('dusk.envWillWrite') }}
          </p>
          <!-- Shown before it is written. A file that overrides .env for the
               length of a run is one to read first. -->
          <pre v-if="!plan.envFilePresent" class="dusk-file text-caption">{{ plan.envFile }}</pre>
          <v-btn
            size="x-small"
            variant="tonal"
            :loading="busy === 'apply'"
            data-test="dusk-apply"
            @click="apply"
          >
            {{ plan.declared ? t('dusk.applyAgain') : t('dusk.apply') }}
          </v-btn>
        </div>

        <!-- Step two: the certificate, which is the half nobody solves. -->
        <div class="mb-2" data-test="dusk-trust">
          <p class="text-caption text-medium-emphasis mb-1">{{ t('dusk.trustWhy') }}</p>
          <p class="text-caption text-medium-emphasis mb-2">{{ t('dusk.trustAgain') }}</p>
          <v-btn
            size="x-small"
            variant="tonal"
            prepend-icon="mdi-certificate-outline"
            :disabled="!plan.running"
            :loading="busy === 'trust'"
            data-test="dusk-trust-run"
            @click="trust"
          >
            {{ t('dusk.trust') }}
          </v-btn>
          <span v-if="!plan.running" class="text-caption text-medium-emphasis ml-2">
            {{ t('dusk.trustNeedsRunning') }}
          </span>
        </div>

        <v-list v-if="steps" density="compact" class="bg-transparent pa-0">
          <v-list-item v-for="step in steps" :key="step.id" class="px-0" data-test="dusk-step">
            <template #prepend>
              <v-icon
                :color="step.ok ? 'success' : step.optional ? 'warning' : 'error'"
                size="18"
                class="mr-3"
              >
                {{ step.ok ? 'mdi-check-circle-outline' : 'mdi-alert-circle-outline' }}
              </v-icon>
            </template>
            <v-list-item-title class="text-body-2">
              {{ t(`dusk.step.${step.id}`) }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              <code>{{ step.command }}</code>
              <template v-if="step.output"><br />{{ step.output }}</template>
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>

        <!-- Dusk writes to a real database. This app's own answer is a branch
             with one of its own — suggested, never done for somebody. -->
        <p class="text-caption text-medium-emphasis mt-3" data-test="dusk-database">
          {{ plan.isolatedDatabase ? t('dusk.databaseIsolated') : t('dusk.databaseShared') }}
        </p>

        <p class="text-caption text-medium-emphasis mt-2">{{ t('dusk.limit') }}</p>
      </template>
    </template>
  </section>
</template>

<style scoped>
.dusk-file {
  background: rgba(var(--v-theme-on-surface), 0.04);
  border-radius: 4px;
  padding: 8px 10px;
  margin-bottom: 8px;
  overflow-x: auto;
  white-space: pre;
}
</style>
