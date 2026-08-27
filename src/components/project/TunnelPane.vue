<script setup>
import { computed, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { useCopyTick } from '@/composables/useCopyTick';
import { useTunnel } from '@/composables/useTunnel';
import ErrorAlert from '@/components/ErrorAlert.vue';
import QrCode from '@/components/QrCode.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * The public URL this project can be reached at while it is running.
 *
 * One of three panes out of the Container section in the pane split. `running` is a
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
  identity,
  authenticated,
  revealed,
  reservedName,
  busy,
  error,
  load,
  loadIdentity,
  loadProviders,
  start,
  stop,
  saveToken,
  saveAuth,
  reveal,
  saveName,
} = useTunnel(toRef(props, 'name'));
const { copied, copy } = useCopyTick();

watch(
  () => props.name,
  async (value) => {
    await load(value);
    await loadIdentity();
  },
  { immediate: true }
);
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

/**
 * The user name the visitor is asked for. Only ever edited before
 * authentication is switched on — changing it afterwards is switching it on
 * again, which is what the Replace button does.
 */
const authUser = ref('');

/** Switch protection on. An empty password means Rust generates one. */
async function protect() {
  await saveAuth({ user: authUser.value, password: '' });
  authUser.value = '';
}

/**
 * The address this provider is asked to keep, as a field.
 *
 * Seeded from the identity rather than bound to it: a half-typed name must not
 * become the stored one, and the stored one must come back when the picker
 * moves to another provider.
 */
const nameField = ref('');
watch(reservedName, (value) => (nameField.value = value), { immediate: true });

const nameDirty = computed(() => nameField.value.trim() !== (reservedName.value ?? ''));

/** A tunnel that is up while the credential was added afterwards. */
const unprotectedLink = computed(
  () => authenticated.value && !!tunnel.value?.running && !tunnel.value?.guarded
);
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-tunnel"
      icon="mdi-earth"
      :title="t('tunnel.title')"
      :description="t('tunnel.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

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

      <!-- The name that was asked for and not granted. Measured on a
           real provider: the tunnel is up, the pane is green, and the address
           registered in somebody's dashboard points nowhere. -->
      <v-alert v-if="tunnel.reservedHonoured === false" type="warning" variant="tonal" class="mb-3">
        <div class="text-caption">
          {{ t('tunnel.reservedMissed', { name: tunnel.reserved }) }}
        </div>
      </v-alert>

      <!-- Who can open the link. Said about the tunnel that is running rather
           than about the keystore: switching authentication on does not
           protect a link handed out before it. -->
      <v-alert v-if="tunnel.url && tunnel.guarded" type="info" variant="tonal" class="mt-3">
        <div class="text-caption mb-2">
          {{ t('tunnel.protected', { user: identity?.authUser }) }}
        </div>
        <div class="d-flex align-center ga-2 flex-wrap">
          <v-btn v-if="!revealed" size="x-small" variant="tonal" @click="reveal">
            {{ t('tunnel.authShow') }}
          </v-btn>
          <template v-else>
            <code class="credential">{{ revealed.user }} / {{ revealed.password }}</code>
            <v-btn
              icon
              :aria-label="t('a11y.copy')"
              size="x-small"
              variant="text"
              @click="copy(revealed.password, 'password')"
            >
              <v-icon>{{ copied === 'password' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
              <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
            </v-btn>
          </template>
        </div>
      </v-alert>

      <!-- Said before anyone pastes the URL into a public issue: the
           link is live, unauthenticated, and reaches this machine. -->
      <v-alert v-if="tunnel.url && !tunnel.guarded" type="warning" variant="tonal" class="mt-3">
        <div class="text-caption">{{ t('tunnel.publicWarning') }}</div>
        <!-- The one case a warning alone would be wrong about: a credential
             exists, and this link predates it. -->
        <div v-if="unprotectedLink" class="text-caption mt-2">
          {{ t('tunnel.restartToProtect') }}
        </div>
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

      <!-- Who else can open the link, before it exists rather than after it
           has been pasted somewhere. -->
      <div class="mb-3">
        <div class="text-caption text-medium-emphasis mb-1">{{ t('tunnel.authTitle') }}</div>

        <v-alert
          v-if="identity && !identity.keystore"
          type="info"
          variant="tonal"
          density="compact"
        >
          <div class="text-caption">{{ t('tunnel.authNoKeystore') }}</div>
        </v-alert>

        <template v-else-if="!authenticated">
          <div class="d-flex align-center ga-2 flex-wrap">
            <v-text-field
              v-model="authUser"
              :label="t('tunnel.authUser')"
              :placeholder="'stackvo'"
              density="compact"
              variant="outlined"
              hide-details
              class="user-field"
            />
            <v-btn size="small" variant="tonal" :loading="busy" @click="protect">
              {{ t('tunnel.authOn') }}
            </v-btn>
          </div>
          <div class="text-caption text-medium-emphasis mt-1">{{ t('tunnel.authHint') }}</div>
        </template>

        <template v-else>
          <div class="d-flex align-center ga-2 flex-wrap">
            <v-icon size="16" color="success">mdi-lock-outline</v-icon>
            <span class="text-caption">{{
              t('tunnel.authOnFor', { user: identity.authUser })
            }}</span>
            <v-btn v-if="!revealed" size="x-small" variant="text" @click="reveal">
              {{ t('tunnel.authShow') }}
            </v-btn>
            <code v-else class="credential">{{ revealed.user }} / {{ revealed.password }}</code>
            <v-btn
              v-if="revealed"
              icon
              :aria-label="t('a11y.copy')"
              size="x-small"
              variant="text"
              @click="copy(revealed.password, 'password')"
            >
              <v-icon>{{ copied === 'password' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
              <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
            </v-btn>
            <v-btn
              size="x-small"
              variant="text"
              :loading="busy"
              @click="saveAuth({ user: identity.authUser, password: '' })"
            >
              {{ t('tunnel.authRegenerate') }}
            </v-btn>
            <v-btn
              size="x-small"
              variant="text"
              color="error"
              :loading="busy"
              @click="saveAuth(null)"
            >
              {{ t('tunnel.authOff') }}
            </v-btn>
          </div>
        </template>
      </div>

      <!-- The address, for the providers that can keep one. -->
      <div v-if="provider" class="mb-3">
        <div class="text-caption text-medium-emphasis mb-1">{{ t('tunnel.reservedTitle') }}</div>
        <template v-if="provider.reserved">
          <div class="d-flex align-center ga-2 flex-wrap">
            <v-text-field
              v-model="nameField"
              :label="t(`tunnel.reservedKind.${provider.reserved.kind}`)"
              density="compact"
              variant="outlined"
              hide-details
              class="name-field"
            />
            <v-btn
              size="small"
              variant="tonal"
              :disabled="!nameDirty"
              :loading="busy"
              @click="saveName(nameField)"
            >
              {{ t('tunnel.reservedSave') }}
            </v-btn>
          </div>
          <div class="text-caption text-medium-emphasis mt-1">
            {{ t(`tunnel.reservedNote.${provider.id}`) }}
          </div>
        </template>
        <div v-else class="text-caption text-medium-emphasis">{{ t('tunnel.reservedNone') }}</div>
      </div>

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

/* Both fields hold a hostname label, not a sentence. */
.user-field,
.name-field {
  max-width: 260px;
}

/* Read off one screen and typed into another device, so it is monospaced and
   it wraps rather than being cut off. */
.credential {
  font-family: var(--v-font-monospace, monospace);
  word-break: break-all;
}
</style>
