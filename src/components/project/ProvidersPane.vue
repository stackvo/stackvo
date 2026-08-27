<script setup>
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Where this project's data really lives.
 *
 * ## The card is a review screen, not a button
 *
 * A provider is a command **the repository declares** that reaches the network
 * **with the developer's credentials**. So the shape here is the one this
 * application uses for every act it cannot take back: what would run is on
 * screen — the image, every word of the command, the plain environment, the
 * names of the secrets — and the button is under it.
 *
 * The alternative was a `Pull` button per row with the command behind a
 * disclosure. That reads as an action with details attached, when the details
 * are the decision.
 *
 * ## Approval is per direction, and the two look different on purpose
 *
 * Agreeing to fetch is not agreeing to send. Push carries a second sentence,
 * a different colour and its own approval; nothing about approving a pull
 * makes a push cheaper.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const set = ref(null);
/**
 * The database this pulls into, or pushes from.
 *
 * Asked here rather than declared in the recipe. A recipe travels in the
 * repository and says how to reach *somewhere else*; which of this machine's
 * instances it lands in is a property of this machine, and a committed file
 * naming `mysql-8-4` would be wrong on the first teammate's laptop.
 */
const services = ref([]);
const error = ref(null);
const busy = ref('');
const service = ref('');
const snapshotFirst = ref(true);
const secretDrafts = ref({});

const plans = computed(() => set.value?.plans ?? []);
/** A direction a recipe does not offer is not a row — it is not a choice. */
const offered = computed(() => plans.value.filter((p) => p.blocked !== 'not-offered'));

async function load() {
  error.value = null;
  try {
    set.value = await api.projectProviders(props.name);
    if (!services.value.length) {
      services.value = asList(await api.dbTargets()).map((target) => target.service ?? target);
    }
    if (!service.value) service.value = services.value[0] ?? '';
  } catch (e) {
    set.value = null;
    error.value = e;
  }
}

const key = (plan) => `${plan.provider}:${plan.direction}`;

async function approve(plan, granted) {
  busy.value = key(plan);
  error.value = null;
  try {
    await api.providerConsent(props.name, plan.provider, plan.direction, granted);
    await load();
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = '';
  }
}

async function saveSecret(plan, name) {
  const field = `${plan.provider}/${name}`;
  busy.value = field;
  error.value = null;
  try {
    await api.providerSecretSet(props.name, plan.provider, name, secretDrafts.value[field] ?? '');
    secretDrafts.value[field] = '';
    await load();
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = '';
  }
}

async function run(plan) {
  busy.value = key(plan);
  error.value = null;
  try {
    await api.providerRun(
      props.name,
      plan.provider,
      plan.direction,
      service.value,
      // Only a pull replaces something on this machine. A push reads the local
      // database and sends it; there is nothing here for a net to catch.
      plan.direction === 'pull' ? snapshotFirst.value : false
    );
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = '';
  }
}

/** `blocked` is a string for three of its four cases and an object for one. */
const missing = (plan) =>
  plan.blocked && typeof plan.blocked === 'object'
    ? (plan.blocked.missingSecrets?.names ?? [])
    : [];

const needsConsent = (plan) => plan.blocked === 'needs-consent';

onMounted(load);
watch(() => props.name, load);
</script>

<template>
  <!-- Absent, not empty, when the project declares none. Every project until
       somebody writes a recipe — and a pane reading "no providers" answers a
       question nobody asked. -->
  <v-card v-if="offered.length || set?.problems?.length" variant="flat" class="pane">
    <PaneHeader
      help="project-providers"
      icon="mdi-cloud-download-outline"
      :title="t('providers.title')"
      :description="t('providers.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <!-- A recipe that could not be read, named. Three good ones and one typo
         is three cards and one sentence. -->
    <v-alert
      v-for="problem in set?.problems ?? []"
      :key="problem.provider"
      type="warning"
      variant="tonal"
      density="compact"
      class="mb-3"
    >
      <span class="text-caption"
        ><code>{{ problem.provider }}</code> — {{ problem.message }}</span
      >
    </v-alert>

    <v-select
      v-if="offered.length && services.length"
      v-model="service"
      :items="services"
      :label="t('providers.database')"
      density="compact"
      variant="outlined"
      class="mb-3"
      style="max-width: 320px"
    />

    <div v-for="plan in offered" :key="key(plan)" class="plan" data-test="provider-plan">
      <div class="d-flex align-center ga-2 mb-1">
        <v-icon size="16">
          {{ plan.direction === 'pull' ? 'mdi-tray-arrow-down' : 'mdi-tray-arrow-up' }}
        </v-icon>
        <span class="text-body-2 font-weight-medium">{{ plan.provider }}</span>
        <v-chip size="x-small" variant="tonal" :color="plan.direction === 'push' ? 'error' : ''">
          {{ t(`providers.${plan.direction}`) }}
        </v-chip>
      </div>

      <!-- The decision, in full. This is the card. -->
      <pre class="command"
        >{{ plan.image }}
{{ plan.command.join(' ') }}</pre>

      <div
        v-for="(value, envKey) in plan.env"
        :key="envKey"
        class="text-caption text-medium-emphasis"
      >
        <code>{{ envKey }}={{ value }}</code>
      </div>

      <!-- Names. There is nowhere in this payload for a value. -->
      <div v-if="plan.secrets.length" class="text-caption text-medium-emphasis mb-1">
        {{ t('providers.usesSecrets', { names: plan.secrets.join(', ') }) }}
      </div>

      <div
        v-if="plan.direction === 'push'"
        class="text-caption text-error mb-1"
        data-test="push-warning"
      >
        <v-icon size="14" class="mr-1">mdi-alert-outline</v-icon>{{ t('providers.pushWarning') }}
      </div>

      <template v-if="plan.blocked === 'policy-off'">
        <div class="text-caption text-medium-emphasis">{{ t('providers.policyOff') }}</div>
      </template>

      <template v-else-if="needsConsent(plan)">
        <v-btn
          size="x-small"
          variant="tonal"
          :color="plan.direction === 'push' ? 'error' : 'primary'"
          :loading="busy === key(plan)"
          @click="approve(plan, true)"
        >
          {{ t(`providers.approve.${plan.direction}`) }}
        </v-btn>
      </template>

      <template v-else-if="missing(plan).length">
        <div class="text-caption mb-1">
          {{ t('providers.fillIn', { names: missing(plan).join(', ') }) }}
        </div>
        <div v-for="secret in missing(plan)" :key="secret" class="d-flex ga-2 align-center mb-1">
          <v-text-field
            v-model="secretDrafts[`${plan.provider}/${secret}`]"
            :label="secret"
            type="password"
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 320px"
          />
          <v-btn
            size="x-small"
            variant="tonal"
            :loading="busy === `${plan.provider}/${secret}`"
            @click="saveSecret(plan, secret)"
          >
            {{ t('providers.saveSecret') }}
          </v-btn>
        </div>
      </template>

      <template v-else>
        <div class="d-flex align-center ga-3">
          <v-btn
            size="x-small"
            variant="tonal"
            :color="plan.direction === 'push' ? 'error' : 'primary'"
            :disabled="!service"
            :loading="busy === key(plan)"
            @click="run(plan)"
          >
            {{ t(`providers.run.${plan.direction}`) }}
          </v-btn>

          <!-- Only a pull replaces something here. -->
          <v-checkbox
            v-if="plan.direction === 'pull'"
            v-model="snapshotFirst"
            :label="t('providers.snapshotFirst')"
            density="compact"
            hide-details
          />

          <v-spacer />
          <v-btn size="x-small" variant="text" @click="approve(plan, false)">
            {{ t('providers.revoke') }}
          </v-btn>
        </div>
      </template>
    </div>
  </v-card>
</template>

<style scoped>
.plan {
  padding: 10px 0;
}

.plan + .plan {
  border-top: 1px solid rgb(var(--v-border-color), var(--v-border-opacity));
}

.command {
  overflow-x: auto;
  padding: 6px 8px;
  font-size: 0.75rem;
  line-height: 1.5;
  white-space: pre;
  border-radius: 6px;
  background: rgba(var(--v-theme-on-surface), 0.05);
}
</style>
