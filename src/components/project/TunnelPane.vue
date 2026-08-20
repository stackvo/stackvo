<script setup>
import { computed, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { useCopyTick } from '@/composables/useCopyTick';
import { useTunnel } from '@/composables/useTunnel';
import ErrorAlert from '@/components/ErrorAlert.vue';
import QrCode from '@/components/QrCode.vue';

/**
 * The public URL this project can be reached at while it is running.
 *
 * One of three panes out of the Container section under §14.16. `running` is a
 * prop rather than a fetch: a tunnel to a stopped container resolves to
 * nothing, and the view already knows the state.
 *
 * ## Why there is a picker here at all
 *
 * The pane used to be Cloudflare's, and said "no account needed" as though it
 * were a fact about tunnels rather than about one provider. It is a real
 * choice: an anonymous quick tunnel hands out an address in ten seconds and a
 * different one on every start, which is exactly right for "did the webhook
 * arrive" and useless for an OAuth redirect URI somebody has to register in a
 * dashboard once. The providers that keep an address are the ones that want
 * an account, and that trade is the user's to make.
 *
 * The list comes from Rust, which owns the invocation — a list kept here would
 * go on offering a provider the day one is removed, and offer it without the
 * one fact that decides whether it can be used: whether its token is stored.
 */
const props = defineProps({
  name: { type: String, required: true },
  running: { type: Boolean, default: false },
});

const { t } = useI18n();

const {
  tunnel,
  providers,
  provider,
  chosen,
  needsToken,
  busy,
  error,
  load,
  loadProviders,
  start,
  stop,
  saveToken,
} = useTunnel(toRef(props, 'name'));
const { copied, copy } = useCopyTick();

watch(() => props.name, load, { immediate: true });
loadProviders();

/** The picker's rows. Everything shown about a provider is the table's. */
const choices = computed(() =>
  providers.value.map((p) => ({
    ...p,
    title: t(`tunnel.providers.${p.id}`),
    subtitle: [
      p.anonymous ? t('tunnel.noAccount') : t('tunnel.needsAccount'),
      p.sessionMinutes ? t('tunnel.sessionCap', { minutes: p.sessionMinutes }) : null,
      p.tokenEnv && !p.hasToken ? t('tunnel.tokenMissing') : null,
    ]
      .filter(Boolean)
      .join(' · '),
  }))
);

/**
 * The token field, open only while somebody is filling it in.
 *
 * Never seeded and never read back: `tunnel_token_set` writes to the keystore
 * and there is no command that returns a token, so the field can replace one
 * or clear it and never display it.
 */
const token = ref('');
const editingToken = ref(false);

async function submitToken(value) {
  if (!(await saveToken(chosen.value, value))) return;
  token.value = '';
  editingToken.value = false;
}
</script>

<template>
  <ErrorAlert v-if="error" :error="error" class="mb-4" />

  <v-card variant="flat" class="pane">
    <div class="section-head mb-1">
      <v-icon size="18" class="mr-2">mdi-earth</v-icon>{{ t('tunnel.title') }}
    </div>
    <p class="text-caption text-medium-emphasis mb-4">{{ t('tunnel.explain') }}</p>

    <!-- The tunnel forwards to the container; a stopped container would
         serve 502s from a URL that looks like it worked. -->
    <v-alert v-if="!running" type="info" variant="tonal">
      <div class="text-caption">{{ t('tunnel.needsRunning') }}</div>
    </v-alert>

    <template v-else-if="tunnel?.running || tunnel?.failure">
      <v-alert v-if="tunnel.url" type="success" variant="tonal" class="mb-3">
        <div class="d-flex align-center ga-2 flex-wrap">
          <button type="button" class="field-link" @click="api.openInBrowser(tunnel.url)">
            {{ tunnel.url }}
          </button>
          <v-btn
            icon
            :aria-label="t('a11y.copy')"
            size="x-small"
            variant="text"
            @click="copy(tunnel.url, 'tunnel')"
          >
            <v-icon>{{ copied === 'tunnel' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
            <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
          </v-btn>
          <v-spacer />
          <v-btn size="small" color="error" variant="tonal" :loading="busy" @click="stop">
            {{ t('tunnel.stop') }}
          </v-btn>
        </div>
        <!-- Which provider is actually carrying it, because the address alone
             does not say and the answer decides how long it lasts. -->
        <div v-if="tunnel.provider" class="text-caption mt-1">
          {{ t('tunnel.via', { provider: t(`tunnel.providers.${tunnel.provider}`) }) }}
        </div>
      </v-alert>

      <!-- The client's own words. A rejected token is the likeliest failure
           four of these providers have, and it is the one thing a spinner
           that never resolves cannot say. -->
      <v-alert v-if="tunnel.failure" type="error" variant="tonal" class="mb-3">
        <div class="text-subtitle-2 mb-1">{{ t('tunnel.failed') }}</div>
        <div class="text-caption break">{{ tunnel.failure }}</div>
        <div class="d-flex ga-2 mt-2">
          <v-btn size="small" color="error" variant="tonal" :loading="busy" @click="stop">
            {{ t('tunnel.stop') }}
          </v-btn>
        </div>
      </v-alert>

      <!-- The URL is four random words and it is opened on a phone as often as
           on this machine — a webhook sender is not the only thing anybody
           points at a tunnel. -->
      <div v-if="tunnel.url" class="d-flex align-start ga-3 mb-3">
        <QrCode :text="tunnel.url" :size="152" />
        <div class="text-caption text-medium-emphasis pt-1">{{ t('tunnel.scan') }}</div>
      </div>

      <div v-if="!tunnel.url && !tunnel.failure" class="d-flex align-center ga-3">
        <v-progress-circular indeterminate size="18" width="2" color="primary" />
        <span class="text-caption text-medium-emphasis">{{ t('tunnel.connecting') }}</span>
        <v-spacer />
        <v-btn size="small" color="error" variant="tonal" :loading="busy" @click="stop">
          {{ t('tunnel.stop') }}
        </v-btn>
      </div>

      <!-- Said before anyone pastes the URL into a public issue: the
           link is live, unauthenticated, and reaches this machine. -->
      <v-alert v-if="tunnel.url" type="warning" variant="tonal" class="mt-3">
        <div class="text-caption">{{ t('tunnel.publicWarning') }}</div>
      </v-alert>
    </template>

    <template v-else>
      <v-select
        v-model="chosen"
        :items="choices"
        item-title="title"
        item-value="id"
        :label="t('tunnel.provider')"
        density="compact"
        variant="outlined"
        hide-details="auto"
        class="mb-3 provider-select"
      >
        <template #item="{ props: itemProps, item }">
          <v-list-item v-bind="itemProps" :subtitle="item.raw.subtitle">
            <template #append>
              <!-- Stated rather than implied: four of these were opened against
                   a real target in this repository and four could not be,
                   because nobody here has an account with them. -->
              <v-chip v-if="!item.raw.verified" size="x-small" variant="tonal" color="warning">
                {{ t('tunnel.unverified') }}
              </v-chip>
            </template>
          </v-list-item>
        </template>
      </v-select>

      <!-- Everything about the chosen provider that changes what happens
           next, in the order it matters. -->
      <div v-if="provider" class="text-caption text-medium-emphasis mb-3">
        <div>{{ t(`tunnel.providerNote.${provider.id}`) }}</div>
        <div v-if="!provider.rewritesHost" class="mt-1">{{ t('tunnel.noHostHeader') }}</div>
        <!-- The chip in the picker says this too, but the picker is closed by
             the time somebody presses the button. -->
        <div v-if="!provider.verified" class="mt-1">{{ t('tunnel.unverifiedNote') }}</div>
        <div v-if="provider.sessionMinutes" class="mt-1">
          {{ t('tunnel.sessionCapLong', { minutes: provider.sessionMinutes }) }}
        </div>
      </div>

      <!-- The token, where somebody hits the wall rather than in a settings
           pane they would have to go and find. -->
      <template v-if="provider?.tokenEnv">
        <v-alert
          v-if="needsToken && !editingToken"
          type="info"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          <div class="d-flex align-center ga-2 flex-wrap">
            <span class="text-caption">{{ t('tunnel.tokenNeeded') }}</span>
            <v-spacer />
            <v-btn size="small" variant="tonal" @click="editingToken = true">
              {{ t('tunnel.tokenAdd') }}
            </v-btn>
          </div>
        </v-alert>

        <div v-else-if="!editingToken" class="d-flex align-center ga-2 mb-3">
          <v-icon size="16" color="success">mdi-key-outline</v-icon>
          <span class="text-caption text-medium-emphasis">{{ t('tunnel.tokenStored') }}</span>
          <v-btn size="x-small" variant="text" @click="editingToken = true">
            {{ t('tunnel.tokenReplace') }}
          </v-btn>
          <v-btn
            size="x-small"
            variant="text"
            color="error"
            :loading="busy"
            @click="submitToken(null)"
          >
            {{ t('tunnel.tokenClear') }}
          </v-btn>
        </div>

        <div v-if="editingToken" class="mb-3">
          <v-text-field
            v-model="token"
            :label="t('tunnel.tokenLabel', { env: provider.tokenEnv })"
            :hint="t('tunnel.tokenHint')"
            type="password"
            autocomplete="off"
            persistent-hint
            density="compact"
            variant="outlined"
          />
          <div class="d-flex ga-2 mt-2">
            <v-btn
              size="small"
              color="primary"
              variant="flat"
              :disabled="!token || busy"
              :loading="busy"
              @click="submitToken(token)"
            >
              {{ t('tunnel.tokenSave') }}
            </v-btn>
            <v-btn size="small" variant="text" @click="editingToken = false">
              {{ t('app.cancel') }}
            </v-btn>
          </div>
        </div>
      </template>

      <v-btn
        color="primary"
        variant="flat"
        prepend-icon="mdi-earth"
        :loading="busy"
        :disabled="needsToken"
        @click="start"
      >
        {{ t('tunnel.start') }}
      </v-btn>
      <div class="text-caption text-medium-emphasis mt-3">
        {{ t('tunnel.startHint') }}
      </div>
    </template>
  </v-card>
</template>

<style scoped>
/* The picker is a choice between eight names, not a field that has to fill a
   pane the width of a window. */
.provider-select {
  max-width: 420px;
}

.break {
  word-break: break-word;
}
</style>
