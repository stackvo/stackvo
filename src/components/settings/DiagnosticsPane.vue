<script setup>
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAppStore } from '@/stores/app';
import { api } from '@/lib/ipc';
import { bytes } from '@/lib/format';
import SettingsGroup from '@/components/SettingsGroup.vue';
import DoctorPanel from '@/components/DoctorPanel.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';

/**
 * What to send when something is wrong: the engine as this app sees it, the
 * doctor report, the log folder, and the archive that packages all of it.
 *
 * Twelfth and last pane out of `Settings.vue` in the pane split. With it gone the
 * view holds no pane markup at all — only the rail, the shared `.env` editor
 * and the About card.
 */
const { t } = useI18n();
const app = useAppStore();

const logs = ref(null);

// The diagnostic archive. `bundle` holds the last result so the pane can name
// what went in — a success toast that says "saved" leaves the user to open the
// zip to find out whether the thing they were asked for is in it.
const bundling = ref(false);
const bundle = ref(null);
const bundleError = ref(null);

/**
 * Collect the bundle to a path the user picks.
 *
 * The save dialog rather than a fixed location, for the reason
 * `mail_attachment_save` uses one: this writes a file outside everything the
 * app owns, and the only acceptable authority for that is the person at the
 * keyboard. A cancelled dialog is an answer, not a failure — it returns null
 * and nothing is reported.
 */
async function saveDiagnosticBundle() {
  bundleError.value = null;
  bundle.value = null;
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save({
      defaultPath: `stackvo-diagnostics.zip`,
      filters: [{ name: 'Zip archive', extensions: ['zip'] }],
    });
    if (!path) return;

    bundling.value = true;
    bundle.value = await api.diagnosticsBundle(path);
  } catch (e) {
    bundleError.value = e;
  } finally {
    bundling.value = false;
  }
}
// Somebody else's machine, held against this one. `comparison` is kept so the
// pane can show the answer rather than a toast that says it looked.
const comparing = ref(false);
const comparison = ref(null);
const comparisonError = ref(null);

/**
 * Compare a bundle somebody sent with what this machine is right now.
 *
 * The open dialog, for the mirror of the reason the save one is used: this
 * reads a file outside everything the app owns, and the only acceptable
 * authority for choosing it is the person at the keyboard. A cancelled dialog
 * is an answer.
 *
 * Both filters are offered because both are what people actually send — the
 * zip the app produces, and the `environment.json` somebody extracted and
 * pasted into a chat window.
 */
async function compareWithBundle() {
  comparisonError.value = null;
  comparison.value = null;
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const path = await open({
      multiple: false,
      filters: [{ name: 'Diagnostic bundle', extensions: ['zip', 'json', 'txt'] }],
    });
    if (!path) return;

    comparing.value = true;
    comparison.value = await api.diagnosticsCompare(path);
  } catch (e) {
    comparisonError.value = e;
  } finally {
    comparing.value = false;
  }
}

/**
 * The open pane, persisted for the session only.
 *
 * Deliberately not in preferences.json: which pane you last had open is not a
 * setting, and writing the config file on every click would be noise in a file
 * the user may be reading.
 */

const engineRows = computed(() => {
  const e = app.engine;
  if (!e) return [];
  return [
    { label: t('engine.title'), value: e.reachable ? t('engine.running') : t('engine.down') },
    { label: 'Platform', value: t(`engine.platform.${e.platform}`) },
    { label: t('engine.version'), value: e.version || t('app.never') },
    { label: t('engine.apiVersion'), value: e.apiVersion || t('app.never') },
    { label: t('engine.context'), value: e.context || t('app.never') },
    { label: t('engine.socket'), value: e.socketPath || t('app.never') },
  ];
});

/**
 * The images this app runs and did not build.
 *
 * `pkg::MOVING_TAGS` forbids `latest` for a third-party package — *"an image
 * that changes under a fixed manifest has no digest the manifest can pin"* —
 * and six of these ship on it. So the rule applied to everybody except this
 * application, and the day a broken `cloudflared:latest` is published every
 * user's tunnels stop at once with no version to go back to.
 *
 * Shown here because "what will this machine pull, and from where" is a fact
 * about the machine, which is what this pane is. Shown to everyone rather than
 * only on a managed machine: the moving tag is true either way, and the person
 * whose tunnels break is not usually the one with a policy file.
 */
const images = ref([]);
const moving = computed(() => images.value.filter((i) => i.moving));

/**
 * Is any of the administrator's policy actually in force on this machine?
 *
 * `policyStatus` above answers what the file says. This answers whether it
 * holds here, and most of what it finds is not somebody breaking a rule — it is
 * a rule that arrived on a machine that was already set up and still has work
 * to do: a project generated before the registry mirror, a package installed
 * before the list that would have refused it, an index cached before the
 * signature became mandatory.
 *
 * Rendered only on a managed machine. On every other one the honest report is
 * "there is no policy", which the pane above already says by not appearing.
 *
 * The states are ordered by what somebody has to do about them, not by name:
 * bypassed is work, unmeasured is a question, silent is nothing at all.
 */
const compliance = ref(null);
const ORDER = { bypassed: 0, unmeasured: 1, holding: 2, silent: 3 };
const STATE_COLOUR = {
  bypassed: 'warning',
  unmeasured: 'info',
  holding: 'success',
  silent: 'surface-variant',
};

const clauses = computed(() =>
  [...(compliance.value?.clauses ?? [])].sort(
    (a, b) => (ORDER[a.state] ?? 9) - (ORDER[b.state] ?? 9)
  )
);

/**
 * `market.allowedPackages` → `market_allowedPackages`.
 *
 * The ids are dotted because they name a path into the policy file, which is
 * how an administrator reads them. vue-i18n splits a dotted key into a nested
 * lookup, and two of these ids are both a leaf and a prefix — `settings` and
 * `settings.locked` — which no nested object can hold at once. Substituting the
 * separator keeps the id meaningful on the wire and flat in the locale file.
 */
const labelKey = (id) => `settings.compliance.clause.${id.replace(/\./g, '_')}`;

/**
 * What can leave this machine, as far as Docker can say.
 *
 * Beside the images card because the two are halves of one question: that one
 * answers what this machine *pulls* and from where, and this answers what its
 * containers *can reach* — and, for anybody who set a registry mirror, which
 * containers did not come through it.
 *
 * Asked rather than loaded with the pane. It inspects every container and reads
 * a stats sample per running one; cheap when somebody presses a button, rude on
 * a timer.
 */
const egress = ref(null);
const egressing = ref(false);
const egressError = ref(null);

async function loadEgress() {
  egressing.value = true;
  egressError.value = null;
  try {
    egress.value = await api.egressReport();
  } catch (e) {
    egress.value = null;
    egressError.value = e;
  } finally {
    egressing.value = false;
  }
}

const REACH_COLOUR = { outside: 'warning', contained: 'success', unknown: 'surface-variant' };

// Bytes are shown at all only because "this container has sent nothing" is
// worth being sure of. The unit is deliberately coarse: a precise number would
// invite reading it as a measurement of internet traffic, which it is not.
const outgoing = (n) => (n > 0 ? bytes(n) : '—');

onMounted(async () => {
  logs.value = await api.logsInfo().catch(() => null);
  const policy = await api.policyStatus().catch(() => null);
  images.value = Array.isArray(policy?.images) ? policy.images : [];
  // Only on a managed machine, and only after the status call has said so —
  // one round trip rather than two on the overwhelming majority of machines.
  if (policy?.active) compliance.value = await api.policyCompliance().catch(() => null);
});
</script>

<template>
  <SettingsGroup
    help="settings-diagnostics-engine"
    icon="mdi-docker"
    :title="t('engine.title')"
    :description="t('settings.engineGroupDesc')"
  >
    <template #append>
      <v-chip size="small" :color="app.engineUp ? 'success' : 'error'">
        {{ app.engineUp ? t('engine.running') : t('engine.down') }}
      </v-chip>
    </template>

    <div v-for="row in engineRows" :key="row.label" class="d-flex justify-space-between py-1 ga-4">
      <span class="text-caption text-medium-emphasis">{{ row.label }}</span>
      <span class="text-caption text-right break">{{ row.value }}</span>
    </div>
    <div v-if="app.engine?.error" class="text-caption text-error mt-2">
      {{ app.engine.error }}
    </div>
  </SettingsGroup>

  <DoctorPanel />

  <SettingsGroup
    help="settings-diagnostics"
    icon="mdi-bug-outline"
    :title="t('settings.diagnostics')"
    :description="t('settings.diagnosticsHint')"
  >
    <div v-if="!logs?.directory" class="text-caption text-medium-emphasis">
      {{ t('settings.logsUnavailable') }}
    </div>
    <template v-else>
      <div class="d-flex align-center ga-2 flex-wrap">
        <code class="text-caption log-path">{{ logs.directory }}</code>
        <v-spacer />
        <v-chip size="x-small" variant="tonal">{{ bytes(logs.totalBytes) }}</v-chip>
        <v-btn
          size="small"
          variant="tonal"
          prepend-icon="mdi-folder-open"
          @click="api.openFolder(logs.directory)"
        >
          {{ t('settings.openLogs') }}
        </v-btn>
        <!-- The folder button leaves the reporter to find the right
               file among seven and to know the doctor output is a
               separate thing. This is the one that answers the whole
               question. -->
        <v-btn
          size="small"
          variant="flat"
          color="primary"
          prepend-icon="mdi-package-variant-closed"
          :loading="bundling"
          @click="saveDiagnosticBundle"
        >
          {{ t('settings.saveBundle') }}
        </v-btn>
      </div>
      <!-- Said out loud because the alternative is a user who assumes
             the opposite and attaches nothing, or one who assumes it is
             safe when it is not. -->
      <div class="text-caption text-medium-emphasis mt-2">
        {{ t('settings.logsRedacted') }}
      </div>
      <div class="text-caption text-medium-emphasis mt-1">
        {{ t('settings.saveBundleHint') }}
      </div>
      <!-- Named, not counted. "Saved 6 files" tells nobody whether
             the thing they were asked for is in there. -->
      <ErrorAlert v-if="bundleError" :error="bundleError" class="mt-3" />
      <v-alert
        v-if="bundle"
        type="success"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
      >
        <div>{{ t('settings.saveBundleDone', { bytes: bytes(bundle.bytes) }) }}</div>
        <code class="text-caption log-path d-block mt-1">{{ bundle.path }}</code>
        <div class="mt-1">{{ bundle.entries.map((e) => e.name).join(', ') }}</div>
      </v-alert>

      <!-- The other direction: a bundle somebody sent, against this machine.
           "It works on mine" is the question this whole file exists around and
           nothing could answer it until there were two of them. -->
      <v-divider class="my-4" />
      <div class="d-flex align-center ga-2 flex-wrap">
        <v-btn
          size="small"
          variant="tonal"
          prepend-icon="mdi-compare-horizontal"
          :loading="comparing"
          @click="compareWithBundle"
        >
          {{ t('settings.compareBundle') }}
        </v-btn>
      </div>
      <div class="text-caption text-medium-emphasis mt-2">
        {{ t('settings.compareBundleHint') }}
      </div>

      <ErrorAlert v-if="comparisonError" :error="comparisonError" class="mt-3" />

      <!-- Two machines that agree is a result, not an empty state: it means
           the difference is somewhere this cannot see, which is worth being
           told rather than left to infer from a blank box. -->
      <v-alert
        v-if="comparison && !comparison.differences.length"
        type="success"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
      >
        {{ t('settings.compareSame', { count: comparison.same }) }}
      </v-alert>

      <template v-if="comparison?.differences.length">
        <div class="text-caption text-medium-emphasis mt-3">
          {{
            t('settings.compareResult', {
              count: comparison.differences.length,
              same: comparison.same,
            })
          }}
        </div>
        <v-table density="compact" class="mt-2 text-caption">
          <thead>
            <tr>
              <th>{{ t('settings.compareFact') }}</th>
              <th>{{ t('settings.compareHere') }}</th>
              <th>{{ t('settings.compareThere') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in comparison.differences" :key="row.key" data-test="diff-row">
              <td>
                <code class="text-caption">{{ row.key }}</code>
              </td>
              <!-- An absent side is said, not left blank: "you have this and
                   they do not" is the most actionable of the three answers. -->
              <td>{{ row.here ?? t('settings.compareAbsent') }}</td>
              <td>{{ row.there ?? t('settings.compareAbsent') }}</td>
            </tr>
          </tbody>
        </v-table>
      </template>
    </template>
  </SettingsGroup>

  <!-- ---- the images this app pulls ---------------------------------- -->
  <!-- Answering "what will this machine pull, and from where". The second half
       was already here as the policy's registry prefix; nothing answered the
       first, and the ten values lived as literals in four modules. -->
  <SettingsGroup
    help="settings-diagnostics-images"
    icon="mdi-cube-outline"
    :title="t('settings.images.title')"
    :description="t('settings.images.desc')"
  >
    <!-- The double standard, said plainly rather than left for somebody to
         notice: this repository forbids a third-party package from using a
         moving tag and then uses one itself. -->
    <v-alert v-if="moving.length" type="warning" variant="tonal" density="compact" class="mb-3">
      {{ t('settings.images.moving', { count: moving.length }) }}
    </v-alert>

    <v-list density="compact" class="pa-0">
      <v-list-item v-for="image in images" :key="image.repository" class="px-0">
        <v-list-item-title class="text-body-2">
          <code>{{ image.effective }}</code>
          <v-chip v-if="image.moving" size="x-small" color="warning" variant="tonal" class="ml-2">
            {{ t('settings.images.movingTag') }}
          </v-chip>
          <v-chip v-if="image.pinned" size="x-small" color="success" variant="tonal" class="ml-2">
            {{ t('settings.images.pinned') }}
          </v-chip>
        </v-list-item-title>
        <v-list-item-subtitle class="text-caption">{{ image.usedFor }}</v-list-item-subtitle>
      </v-list-item>
    </v-list>

    <p class="text-caption text-medium-emphasis mt-3">
      {{ t('settings.images.hint') }}
    </p>
  </SettingsGroup>

  <!-- ---- what can leave this machine -------------------------------- -->
  <!-- The other half of the images card above: that one says what this machine
       pulls and from where, this one says what its containers can reach. -->
  <SettingsGroup
    help="settings-diagnostics-egress"
    icon="mdi-lan-disconnect"
    :title="t('settings.egress.title')"
    :description="t('settings.egress.desc')"
  >
    <v-btn
      size="small"
      variant="tonal"
      prepend-icon="mdi-lan-disconnect"
      :loading="egressing"
      data-test="egress-run"
      @click="loadEgress"
    >
      {{ t('settings.egress.run') }}
    </v-btn>

    <ErrorAlert v-if="egressError" :error="egressError" class="mt-3" />

    <template v-if="egress">
      <!-- The line the person who set a mirror actually reads: on a machine
           where it holds, this list is one entry long. -->
      <div class="text-caption text-medium-emphasis mt-3 mb-2">
        {{
          t('settings.egress.summary', {
            registries: egress.registries.join(', '),
            contained: egress.contained,
            total: egress.rows.length,
          })
        }}
      </div>

      <v-alert
        v-if="egress.registryPrefix"
        type="info"
        variant="tonal"
        density="compact"
        class="mb-3 text-caption"
      >
        {{ t('settings.egress.mirror', { prefix: egress.registryPrefix }) }}
      </v-alert>

      <v-table density="compact" class="text-caption">
        <thead>
          <tr>
            <th>{{ t('settings.egress.container') }}</th>
            <th>{{ t('settings.egress.registry') }}</th>
            <th>{{ t('settings.egress.reach') }}</th>
            <th class="text-right">{{ t('settings.egress.sent') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in egress.rows" :key="row.container" data-test="egress-row">
            <td>
              <div>{{ row.container }}</div>
              <code class="text-caption text-medium-emphasis">{{ row.image }}</code>
            </td>
            <td>
              {{ row.registry }}
              <!-- Only ever shown when a mirror is in force. Without one there
                   is no rule to have bypassed. -->
              <v-chip
                v-if="egress.registryPrefix && !row.mirrored"
                size="x-small"
                color="warning"
                label
                class="ml-1"
              >
                {{ t('settings.egress.bypassed') }}
              </v-chip>
            </td>
            <td>
              <v-chip size="x-small" label :color="REACH_COLOUR[row.reach]">
                {{ t(`settings.egress.reachState.${row.reach}`) }}
              </v-chip>
            </td>
            <td class="text-right">{{ outgoing(row.sent) }}</td>
          </tr>
        </tbody>
      </v-table>
    </template>

    <!-- Said out loud rather than left as a gap, so nobody reads this table as
         a list of everywhere their containers have been. -->
    <p class="text-caption text-medium-emphasis mt-3">{{ t('settings.egress.noDestinations') }}</p>
  </SettingsGroup>

  <!-- ---- is the policy actually holding? ---------------------------- -->
  <!-- Only on a managed machine. Elsewhere the report would be a page of
       "the policy says nothing", which is what an absent pane already says. -->
  <SettingsGroup
    v-if="compliance"
    help="settings-diagnostics-compliance"
    icon="mdi-clipboard-check-outline"
    :title="t('settings.compliance.title')"
    :description="t('settings.compliance.desc')"
  >
    <template #append>
      <v-chip
        size="small"
        :color="compliance.attestable ? 'success' : 'warning'"
        data-test="attestable"
      >
        {{
          compliance.attestable
            ? t('settings.compliance.accountedFor')
            : t('settings.compliance.notAccountedFor')
        }}
      </v-chip>
    </template>

    <!-- The summary as four numbers, because the one number people want does
         not exist: `silent` is the policy saying nothing and must never be
         read as a pass. -->
    <div class="text-caption text-medium-emphasis mb-3">
      {{
        t('settings.compliance.summary', {
          holding: compliance.holding,
          bypassed: compliance.bypassed,
          unmeasured: compliance.unmeasured,
          silent: compliance.silent,
        })
      }}
    </div>

    <v-table density="compact" class="text-caption">
      <tbody>
        <tr
          v-for="(clause, i) in clauses"
          :key="`${clause.id}-${clause.subject}-${i}`"
          data-test="clause"
        >
          <td class="pl-0">
            <v-chip size="x-small" label :color="STATE_COLOUR[clause.state]">
              {{ t(`settings.compliance.state.${clause.state}`) }}
            </v-chip>
          </td>
          <td>
            <div>{{ t(labelKey(clause.id)) }}</div>
            <code class="text-caption text-medium-emphasis">{{ clause.subject }}</code>
          </td>
          <!-- Not translated: what was measured is a path, a key id, a
               reference, and it reads the same in every language. -->
          <td class="text-medium-emphasis">{{ clause.detail ?? '—' }}</td>
        </tr>
      </tbody>
    </v-table>

    <!-- Said here as well as in policy.rs, because this is the screen most
         likely to be read as a certificate. -->
    <p class="text-caption text-medium-emphasis mt-3">
      {{ t('settings.compliance.notACertificate') }}
    </p>
  </SettingsGroup>

  <!-- ---- certificates ---------------------------------------------- -->
  <!-- HTTPS worked before this pane existed and was invisible: the one
       question a browser warning raises — "is my domain in the
       certificate?" — had no answer anywhere in the app. -->
</template>
