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
 *
 * ## Why the card is no longer absent on a project that declares nothing
 *
 * It used to be, on the reading that a pane saying "no providers" answers a
 * question nobody asked. That was right while there was nothing to offer and
 * wrong the moment there was: with the mechanism finished and the catalogue
 * empty, the only way to reach this feature was to already know the file
 * format. So a project with no recipes gets the starting points instead of
 * getting nothing — and a project that has some does not, because it is past
 * that question.
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
/**
 * The starting points this build ships.
 *
 * Fetched beside the plans rather than lazily behind a disclosure: the whole
 * problem being fixed is that somebody had to know this existed.
 */
const recipes = ref([]);

const plans = computed(() => set.value?.plans ?? []);
/** A direction a recipe does not offer is not a row — it is not a choice. */
const offered = computed(() => plans.value.filter((p) => p.blocked !== 'not-offered'));

async function load() {
  error.value = null;
  try {
    set.value = await api.projectProviders(props.name);
    if (!recipes.value.length) recipes.value = asList(await api.providerRecipes());
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

/**
 * Offered only to a project with nothing declared.
 *
 * A project that already has recipes has answered the question these exist to
 * ask, and a permanent list of starting points under somebody's working
 * configuration reads as an invitation to add another.
 */
const startingPoints = computed(() =>
  set.value && !set.value.recipes?.length ? recipes.value : []
);

async function addRecipe(recipe) {
  busy.value = `recipe:${recipe.name}`;
  error.value = null;
  try {
    await api.providerRecipeAdd(props.name, recipe.name);
    await load();
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = '';
  }
}

onMounted(load);
watch(() => props.name, load);
</script>

<template>
  <!-- Absent, not empty, when the project declares none. Every project until
       somebody writes a recipe — and a pane reading "no providers" answers a
       question nobody asked. -->
  <v-card
    v-if="offered.length || set?.problems?.length || startingPoints.length"
    variant="flat"
    class="pane"
  >
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

    <!-- The starting points, and only for a project that declares nothing.
         Each shows the command it would add, because that is the thing being
         added — and the list under it says which words are placeholders, since
         the recipe cannot work until they are replaced. -->
    <div v-if="startingPoints.length" data-test="provider-recipes">
      <p class="text-caption text-medium-emphasis mb-3">{{ t('providers.recipesIntro') }}</p>

      <div v-for="recipe in startingPoints" :key="recipe.name" class="plan">
        <div class="d-flex align-center ga-2 mb-1">
          <v-icon size="16">mdi-file-document-outline</v-icon>
          <span class="text-body-2 font-weight-medium">{{ recipe.name }}</span>
          <v-chip v-if="recipe.pull.length" size="x-small" variant="tonal">
            {{ t('providers.pull') }}
          </v-chip>
          <v-chip v-if="recipe.push.length" size="x-small" variant="tonal" color="error">
            {{ t('providers.push') }}
          </v-chip>
        </div>

        <p class="text-caption text-medium-emphasis mb-1">{{ recipe.about }}</p>

        <pre class="command"
          >{{ recipe.image }}
{{ (recipe.pull.length ? recipe.pull : recipe.push).join(' ') }}</pre>

        <!-- Never empty: every shipped recipe carries a placeholder. -->
        <ul class="text-caption text-medium-emphasis mb-2 ms-4">
          <li v-for="(what, i) in recipe.edit" :key="i">{{ what }}</li>
        </ul>

        <v-btn
          size="x-small"
          variant="tonal"
          :loading="busy === `recipe:${recipe.name}`"
          @click="addRecipe(recipe)"
        >
          {{ t('providers.addRecipe') }}
        </v-btn>
      </div>
    </div>

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
