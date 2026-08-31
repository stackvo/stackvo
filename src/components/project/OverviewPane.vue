<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { useCopyTick } from '@/composables/useCopyTick';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * What this project *is*: its domain, its runtime, and the addresses it answers
 * on — the first thing the Configuration section shows.
 *
 * Read-only, and everything comes from the project the view already loaded. The
 * two things it cannot do itself are opening the settings sheet and the hosts
 * repair dialog, both of which are the view's own dialogs, so it asks.
 */
const props = defineProps({
  project: { type: Object, default: null },
  details: { type: Object, default: null },
});

const emit = defineEmits(['settings', 'fixHosts']);

const { t } = useI18n();
const { copied, copy } = useCopyTick();

const manifest = computed(() => props.project?.manifest);

// Both schemes, because which one answers is the whole question a user has
// when a certificate is missing: `http://` works and `https://` does not.
const httpUrl = computed(() => (props.project?.domain ? `http://${props.project.domain}` : null));
const httpsUrl = computed(() => (props.project?.domain ? `https://${props.project.domain}` : null));

// Where the generator actually put `WORKDIR`. PHP images work out of the web
// root; node and every language runtime get `/app`. This line was `/var/www/html`
// for everything, with no `v-if` under two rows that had one, so a Go project's
// page offered a copy button for a path that does not exist in its container.
//
// Written as PHP-or-else rather than as a list of the runtimes that use `/app`,
// because that is the shape of the dispatch it mirrors: `render_dockerfile`
// sends node and the language runtimes elsewhere and falls through to PHP. A
// ninth runtime added tomorrow gets `/app` in both places without an edit here.
const containerPath = computed(() =>
  (props.project?.runtime ?? 'php') === 'php' ? '/var/www/html' : '/app'
);

/**
 * Does this machine match what the repository declares?
 *
 * Asked rather than shown by default, and that is the point of the button:
 * everything else on this card is what the project *is*, and this is a question
 * about the machine underneath it. Somebody an hour into a clone is the one who
 * presses it.
 *
 * Every line comes back, not only the failing ones — a verifier that answered
 * with nothing when everything matched would leave the reader unable to tell
 * "it checked and I am fine" from "it did not check".
 */
const verification = ref(null);
const verifying = ref(false);
const verifyError = ref(null);

async function verify() {
  verifying.value = true;
  verifyError.value = null;
  try {
    verification.value = await api.projectVerify(props.project.name);
  } catch (e) {
    verification.value = null;
    verifyError.value = e;
  } finally {
    verifying.value = false;
  }
}

// A result about one project must not stay on screen for the next one.
watch(
  () => props.project?.name,
  () => {
    verification.value = null;
    verifyError.value = null;
    locked.value = null;
    lockError.value = null;
  }
);

/**
 * Write down what this project is actually running against.
 *
 * Beside the verify button because the two are one loop: verify asks whether
 * this machine matches, and until something wrote a lock the answer could not
 * be stronger than "a redis is installed". A press here is what makes the next
 * verify — on this machine or on somebody else's — able to say which redis.
 *
 * Never automatic. A lock the app refreshed on its own would record whatever
 * the machine drifted to, so it would always agree with the machine and could
 * never disagree with it.
 */
const locking = ref(false);
const locked = ref(null);
const lockError = ref(null);

async function lock() {
  locking.value = true;
  lockError.value = null;
  try {
    locked.value = await api.projectLock(props.project.name);
    // Re-asked immediately, because the answer has changed: every declared
    // service the lock now names is checked against a version rather than
    // against presence. Showing the old verification beside a new lock would be
    // showing two answers to one question.
    if (verification.value) await verify();
  } catch (e) {
    locked.value = null;
    lockError.value = e;
  } finally {
    locking.value = false;
  }
}

const CHECK_STATE = { ok: 'success', missing: 'error', different: 'warning', unknown: 'info' };
const CHECK_ICON = {
  ok: 'mdi-check-circle-outline',
  missing: 'mdi-close-circle-outline',
  different: 'mdi-alert-circle-outline',
  unknown: 'mdi-help-circle-outline',
};
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-overview"
      icon="mdi-folder-cog"
      :title="t('projectDetail.configuration')"
      :description="t('projectDetail.configurationExplain')"
    />

    <v-row>
      <v-col cols="12" md="4">
        <div class="field">
          <span class="field-key">{{ t('projectsView.colDomain') }}</span>
          <button
            v-if="httpUrl"
            type="button"
            class="field-link"
            @click="project.domainConfigured && api.openInBrowser(httpsUrl)"
          >
            {{ project.domain }}
          </button>
          <span v-else class="field-val">—</span>
        </div>
        <!-- Beside the domain, because that is what they extend. A wildcard is
             marked: it reaches the certificate and the router and cannot reach
             /etc/hosts, so it is the one name here that does not resolve on its
             own. -->
        <div v-if="manifest?.aliases?.length" class="field">
          <span class="field-key">{{ t('newProject.aliases') }}</span>
          <span class="field-val">
            <v-chip
              v-for="host in manifest.aliases"
              :key="host"
              size="x-small"
              class="mr-1"
              :color="host.startsWith('*.') ? 'warning' : undefined"
              :title="host.startsWith('*.') ? t('newProject.aliasesWildcard') : host"
            >
              {{ host }}
            </v-chip>
          </span>
        </div>
        <div v-if="manifest?.php" class="field">
          <span class="field-key">{{ t('newProject.phpVersion') }}</span>
          <span class="field-val">PHP {{ manifest.php.version }}</span>
        </div>
        <div v-if="manifest?.node" class="field">
          <span class="field-key">{{ t('newProject.nodeVersion') }}</span>
          <span class="field-val">Node {{ manifest.node.version }}</span>
        </div>
        <div class="field">
          <span class="field-key">{{ t('projectDetail.containerPath') }}</span>
          <code class="field-mono">{{ containerPath }}</code>
          <v-btn
            icon
            :aria-label="t('a11y.copy')"
            size="x-small"
            variant="text"
            @click="copy(containerPath, 'cpath')"
          >
            <v-icon>{{ copied === 'cpath' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
            <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
          </v-btn>
        </div>
        <div class="field">
          <span class="field-key">{{ t('projectDetail.accessHttp') }}</span>
          <button
            v-if="httpUrl"
            type="button"
            class="field-link"
            @click="project.domainConfigured && api.openInBrowser(httpUrl)"
          >
            {{ httpUrl }}
          </button>
          <span v-else class="field-val">—</span>
        </div>
      </v-col>

      <v-col cols="12" md="4">
        <div class="field">
          <span class="field-key">{{ t('projectDetail.sslStatus') }}</span>
          <span class="field-val text-success">
            <v-icon size="14" color="success">mdi-lock</v-icon>
            {{ t('projectDetail.sslEnabled') }}
          </span>
        </div>
        <div class="field">
          <span class="field-key">{{ t('projectsView.colServer') }}</span>
          <span class="field-val">{{ manifest?.server || '—' }}</span>
        </div>
        <div class="field">
          <span class="field-key">{{ t('projectDetail.hostPath') }}</span>
          <code class="field-mono">{{ project.path }}</code>
          <v-btn
            icon
            :aria-label="t('a11y.copy')"
            size="x-small"
            variant="text"
            @click="copy(project.path, 'hpath')"
          >
            <v-icon>{{ copied === 'hpath' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
            <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
          </v-btn>
        </div>
      </v-col>

      <v-col cols="12" md="4">
        <div class="field">
          <span class="field-key">{{ t('projectDetail.type') }}</span>
          <span class="field-val">
            <v-icon size="14">mdi-cog</v-icon> {{ t('projectsView.default') }}
          </span>
        </div>
        <div v-if="manifest?.documentRoot" class="field">
          <span class="field-key">{{ t('newProject.documentRoot') }}</span>
          <code class="field-mono">{{ manifest.documentRoot }}</code>
          <v-btn
            icon
            :aria-label="t('a11y.copy')"
            size="x-small"
            variant="text"
            @click="copy(manifest.documentRoot, 'droot')"
          >
            <v-icon>{{ copied === 'droot' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
            <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
          </v-btn>
        </div>
        <div class="field">
          <span class="field-key">{{ t('projectDetail.accessHttps') }}</span>
          <button
            v-if="httpsUrl"
            type="button"
            class="field-link"
            @click="project.domainConfigured && api.openInBrowser(httpsUrl)"
          >
            {{ httpsUrl }}
          </button>
          <span v-else class="field-val">—</span>
        </div>
      </v-col>
    </v-row>

    <!-- The domain is unreachable without a hosts entry, and the fix is
         one elevated write away — so offer it here rather than only
         reporting the problem. -->
    <v-alert
      v-if="project.domain && !project.domainConfigured"
      type="warning"
      variant="tonal"
      class="mt-2"
    >
      <div class="d-flex align-center ga-2">
        <span class="text-caption">{{ t('projects.domainMissingHint') }}</span>
        <v-spacer />
        <v-btn size="x-small" variant="tonal" @click="emit('fixHosts')">{{ t('hosts.fix') }}</v-btn>
      </div>
    </v-alert>

    <template v-if="manifest?.php?.extensions?.length">
      <div class="section-head mt-8 mb-3">
        <v-icon size="18" class="mr-2">mdi-puzzle</v-icon>{{ t('projectDetail.phpExtensions') }}
        <v-chip size="x-small" class="ml-2">{{ manifest.php.extensions.length }}</v-chip>
      </div>
      <div class="d-flex flex-wrap ga-2">
        <v-chip
          v-for="ext in manifest.php.extensions"
          :key="ext"
          size="small"
          label
          variant="tonal"
        >
          {{ ext }}
        </v-chip>
      </div>
    </template>

    <!-- Contract violations, shown rather than swallowed: the Bash
         generator skips such projects without a word. -->
    <template v-if="manifest?.errors?.length || manifest?.warnings?.length">
      <div class="section-head mt-8 mb-3">
        <v-icon size="18" class="mr-2">mdi-file-alert</v-icon>{{ t('projects.problems') }}
      </div>
      <v-alert v-if="manifest.errors.length" type="error" variant="tonal" class="mb-2">
        <div v-for="(i, k) in manifest.errors" :key="k" class="text-caption">
          <strong>{{ i.code }}</strong> {{ i.path }} — {{ i.message }}
        </div>
      </v-alert>
      <v-alert v-if="manifest.warnings.length" type="warning" variant="tonal">
        <div v-for="(i, k) in manifest.warnings" :key="k" class="text-caption">
          <strong>{{ i.code }}</strong> {{ i.path }} — {{ i.message }}
        </div>
      </v-alert>
    </template>

    <!-- The other half of onboarding: the repository says what it needs, and
         this answers whether this machine has it. Asked, not polled — it is a
         question about the machine, and nobody wants it re-answered on every
         render of a card about the project. -->
    <div class="section-head mt-8 mb-3">
      <v-icon size="18" class="mr-2">mdi-clipboard-check-outline</v-icon>
      {{ t('verify.title') }}
    </div>
    <p class="text-caption text-medium-emphasis mb-2">{{ t('verify.explain') }}</p>
    <div class="d-flex ga-2 flex-wrap">
      <v-btn
        size="small"
        variant="tonal"
        prepend-icon="mdi-clipboard-check-outline"
        :loading="verifying"
        :disabled="!project"
        @click="verify"
      >
        {{ t('verify.run') }}
      </v-btn>
      <!-- The write half, and it says so: this one puts a file in the
           repository, which is not what the button beside it does. -->
      <v-btn
        size="small"
        variant="tonal"
        prepend-icon="mdi-lock-outline"
        :loading="locking"
        :disabled="!project"
        data-test="lock"
        @click="lock"
      >
        {{ t('verify.lock') }}
      </v-btn>
    </div>
    <p class="text-caption text-medium-emphasis mt-2">{{ t('verify.lockExplain') }}</p>

    <v-alert
      v-if="lockError"
      type="error"
      variant="tonal"
      density="compact"
      class="mt-3 text-caption"
    >
      {{ lockError.message }}
    </v-alert>

    <v-alert
      v-if="locked"
      type="success"
      variant="tonal"
      density="compact"
      class="mt-3 text-caption"
      data-test="locked"
    >
      {{ t('verify.locked', { count: locked.locked.length }) }}
      <!-- What it could not lock, named with the reason. A lock file that
           quietly covers three of five services is one somebody believes
           covers five. -->
      <div v-for="row in locked.skipped" :key="row.service" class="mt-1">
        {{ t(`verify.skipped.${row.reason}`, { service: row.service }) }}
      </div>
      <div class="mt-1 font-monospace">{{ locked.path }}</div>
    </v-alert>

    <v-alert
      v-if="verifyError"
      type="error"
      variant="tonal"
      density="compact"
      class="mt-3 text-caption"
    >
      {{ verifyError.message }}
    </v-alert>

    <template v-if="verification">
      <v-alert
        :type="verification.ready ? 'success' : 'warning'"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
      >
        {{ verification.ready ? t('verify.ready') : t('verify.notReady') }}
      </v-alert>

      <v-list density="compact" class="bg-transparent pa-0 mt-1">
        <v-list-item
          v-for="(check, i) in verification.checks"
          :key="`${check.id}-${check.subject}-${i}`"
          class="px-0"
          data-test="verify-check"
        >
          <template #prepend>
            <v-icon :color="CHECK_STATE[check.state]" size="18" class="mr-3">
              {{ CHECK_ICON[check.state] }}
            </v-icon>
          </template>
          <v-list-item-title class="text-body-2">
            {{ t(`verify.check.${check.id}`, { subject: check.subject }) }}
          </v-list-item-title>
          <v-list-item-subtitle v-if="check.state !== 'ok'" class="text-caption">
            {{ t(`verify.fix.${check.id}`) }}
          </v-list-item-subtitle>
          <template v-if="check.detail" #append>
            <span class="text-caption text-medium-emphasis">{{ check.detail }}</span>
          </template>
        </v-list-item>
      </v-list>
    </template>

    <!-- Every value read above is a field in stackvo.json, so the way to change
         one belongs with them rather than only in the raw JSON pane further
         down the rail. -->
    <div class="pane-foot">
      <v-spacer />
      <v-btn size="small" variant="tonal" prepend-icon="mdi-tune-variant" @click="emit('settings')">
        {{ t('projectSettings.open') }}
      </v-btn>
    </div>
  </v-card>
</template>
