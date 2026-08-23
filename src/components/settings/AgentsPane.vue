<script setup>
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import SettingsGroup from '@/components/SettingsGroup.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';
import { useCopyTick } from '@/composables/useCopyTick';

/**
 * Registering the MCP server with the assistants on this machine.
 *
 * The pane's shape follows from what it edits. Every row writes to a file
 * belonging to another application, so each one names that file on screen —
 * not as decoration, but because the two outcomes this cannot rule out (a
 * refusal to touch a file with comments in it, and a registration somebody
 * wants to undo by hand) both end with the reader opening that exact path.
 *
 * The write switch is above the list rather than inside a row, and off. It is
 * the only decision here with a consequence: `--allow-writes` hands the
 * assistant `stack_down` and `project_stop`, not just the read tools people
 * come for. A per-row toggle would ask the same question six times and get a
 * less considered answer each time.
 */
const { t } = useI18n();
const { copied, copy } = useCopyTick();

const status = ref({ binary: null, source: null, root: null, clients: [] });
const error = ref(null);
const busy = ref(null);
const loading = ref(false);
const allowWrites = ref(false);

/**
 * The rules half.
 *
 * Registering the server makes the tools reachable; it does not make them used
 * — which is why every rival ships a rules file beside its MCP server and this
 * pane now has two halves rather than one. The state lives here rather than in
 * a pane of its own because the two are one decision: somebody who registers
 * Cursor wants Cursor told what the server is for, and a second Settings page
 * would be a second place to forget.
 */
const rules = ref([]);
const projects = ref([]);
/** `null` is the workspace root, which is the sensible default and the only
 *  choice that works before any project exists. */
const project = ref(null);
const rulesBusy = ref(null);

const ready = computed(() => Boolean(status.value.binary));

/** Clients this machine has. The rest are listed dimmed rather than hidden —
 *  "Cursor is not here" is an answer, and an absent row reads as a bug. */
const rows = computed(() =>
  [...status.value.clients].sort((a, b) => Number(b.present) - Number(a.present))
);

/**
 * The block to paste when this cannot write the file itself.
 *
 * Built from the same values the installer would use, so what is shown and
 * what would be written cannot drift apart.
 */
function snippet(client) {
  const entry = { command: status.value.binary ?? '/path/to/stackvo-mcp' };
  if (client.id === 'vscode') entry.type = 'stdio';
  if (allowWrites.value) entry.args = ['--allow-writes'];
  if (status.value.root) entry.env = { STACKVO_ROOT: status.value.root };

  const key = client.id === 'vscode' ? 'servers' : 'mcpServers';
  return JSON.stringify({ [key]: { stackvo: entry } }, null, 2);
}

function state(client) {
  if (!client.parseable) return 'unparseable';
  if (!client.present) return 'absent';
  if (client.command && client.current) return 'registered';
  if (client.command) return 'stale';
  return 'available';
}

/** Absent, installed, or installed from an older release. */
function rulesState(row) {
  if (!row.installed) return 'absent';
  return row.current ? 'installed' : 'stale';
}

/** The two scopes, kept apart in the list because they answer different
 *  questions: one travels with the repository, the other with the machine. */
const workspaceRules = computed(() => rules.value.filter((r) => r.scope === 'workspace'));
const globalRules = computed(() => rules.value.filter((r) => r.scope === 'global'));

async function load() {
  loading.value = true;
  error.value = null;
  try {
    status.value = await api.agentsStatus();
    // Not fatal, and deliberately not awaited together: the client list is
    // about the machine and answers with no workspace open, while the rules
    // list needs one for its workspace half. A failure in the second must not
    // blank the first.
    // `asList`, not the value: a command that answers with nothing must not
    // leave the two computeds below iterating `undefined`.
    rules.value = asList(await api.rulesStatus(project.value ?? undefined));
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }

  try {
    projects.value = asList(await api.projectsList()).map((p) => p.name);
  } catch {
    // No workspace, or an unreadable one. The picker simply offers the root.
    projects.value = [];
  }
}

async function applyRules(row) {
  rulesBusy.value = `${row.id}:${row.scope}`;
  error.value = null;
  try {
    await api.rulesApply(row.id, row.scope, project.value ?? undefined);
    await load();
  } catch (e) {
    // Re-read first, then report — the same order the installer above uses,
    // and for the same reason: a failed write may still have changed the file.
    await load();
    error.value = e;
  } finally {
    rulesBusy.value = null;
  }
}

async function removeRules(row) {
  rulesBusy.value = `${row.id}:${row.scope}`;
  error.value = null;
  try {
    await api.rulesRemove(row.id, row.scope, project.value ?? undefined);
    await load();
  } catch (e) {
    await load();
    error.value = e;
  } finally {
    rulesBusy.value = null;
  }
}

async function install(client) {
  busy.value = client.id;
  error.value = null;
  try {
    await api.agentsInstall(client.id, allowWrites.value);
    await load();
  } catch (e) {
    // Re-read first, then report — the same order SecretsPane settled on, for
    // the same reason: a failed write may still have changed the file, and a
    // row showing the old state would be a claim nobody checked. `load()`
    // clears the error on the way in, so it is set afterwards.
    await load();
    error.value = e;
  } finally {
    busy.value = null;
  }
}

async function remove(client) {
  busy.value = client.id;
  error.value = null;
  try {
    await api.agentsRemove(client.id);
    await load();
  } catch (e) {
    await load();
    error.value = e;
  } finally {
    busy.value = null;
  }
}

onMounted(load);

defineExpose({ snippet, state, rulesState });
</script>

<template>
  <ErrorAlert v-if="error" :error="error" class="mb-4" />

  <SettingsGroup
    help="settings-agents"
    icon="mdi-robot-outline"
    :title="t('settings.agents.title')"
    :description="t('settings.agents.description')"
  >
    <v-alert type="info" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.agents.whatItDoes') }}</div>
      <div class="text-caption mt-2 text-medium-emphasis">
        {{ t('settings.agents.neverClobbers') }}
      </div>
    </v-alert>

    <!-- Not a warning about a failure: the binary is a separate build and
         always has been. What would be a failure is registering a path that is
         not there, which is why every button below is withheld until it is. -->
    <v-alert
      v-if="!ready && !loading"
      type="warning"
      variant="tonal"
      density="comfortable"
      class="mb-4"
    >
      <div class="text-body-2">{{ t('settings.agents.noBinary') }}</div>
      <code class="d-block mt-2 text-caption">{{ t('settings.agents.buildCommand') }}</code>
      <v-btn
        size="small"
        variant="text"
        class="mt-1"
        :prepend-icon="copied === 'build' ? 'mdi-check' : 'mdi-content-copy'"
        @click="copy(t('settings.agents.buildCommand'), 'build')"
      >
        {{ t('app.copy') }}
      </v-btn>
    </v-alert>

    <div v-if="ready" class="mb-4">
      <div class="text-caption text-medium-emphasis">{{ t('settings.agents.serverBinary') }}</div>
      <code class="text-caption">{{ status.binary }}</code>
    </div>

    <!-- The accent, like every other toggle in the application. Orange here was
         meant to mark the risk, and the sentence underneath already names it —
         a colour nothing else on the page uses reads as a warning *state* the
         switch is in rather than as the switch being on. -->
    <v-switch
      v-model="allowWrites"
      color="primary"
      density="compact"
      hide-details
      class="mb-1"
      :label="t('settings.agents.allowWrites')"
    />
    <div class="text-caption text-medium-emphasis mb-4">
      {{ t('settings.agents.allowWritesDetail') }}
    </div>

    <v-progress-linear v-if="loading" indeterminate class="mb-2" />

    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="client in rows" :key="client.id" class="px-0">
        <template #prepend>
          <v-icon
            :icon="state(client) === 'registered' ? 'mdi-check-circle-outline' : 'mdi-application'"
            :color="
              { registered: 'success', stale: 'warning', unparseable: 'warning' }[state(client)]
            "
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2">{{ client.label }}</v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ t(`settings.agents.state.${state(client)}`) }} — <code>{{ client.path }}</code>
        </v-list-item-subtitle>

        <template #append>
          <v-btn
            v-if="state(client) === 'registered'"
            size="small"
            variant="tonal"
            :loading="busy === client.id"
            :disabled="busy !== null && busy !== client.id"
            @click="remove(client)"
          >
            {{ t('settings.agents.remove') }}
          </v-btn>
          <v-btn
            v-else-if="state(client) !== 'unparseable'"
            size="small"
            variant="tonal"
            color="primary"
            :loading="busy === client.id"
            :disabled="!ready || (busy !== null && busy !== client.id)"
            @click="install(client)"
          >
            {{ state(client) === 'stale' ? t('settings.agents.update') : t('settings.agents.add') }}
          </v-btn>
          <v-btn
            v-else
            size="small"
            variant="text"
            :prepend-icon="copied === client.id ? 'mdi-check' : 'mdi-content-copy'"
            @click="copy(snippet(client), client.id)"
          >
            {{ t('settings.agents.copyBlock') }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <div class="text-caption text-medium-emphasis mt-4">
      {{ t('settings.agents.notListed') }}
    </div>
  </SettingsGroup>

  <!-- The second half. Registering the server makes the tools reachable;
       this is what makes them used. Same help topic on purpose — a reader
       here has the same question as a reader above, one step later. -->
  <SettingsGroup
    help="settings-agents"
    icon="mdi-script-text-outline"
    :title="t('settings.agents.rules.title')"
    :description="t('settings.agents.rules.description')"
    class="mt-6"
  >
    <v-alert type="info" variant="tonal" density="comfortable" class="mb-4">
      <div class="text-body-2">{{ t('settings.agents.rules.whatItDoes') }}</div>
      <div class="text-caption mt-2 text-medium-emphasis">
        {{ t('settings.agents.rules.markers') }}
      </div>
    </v-alert>

    <!-- The workspace root is the default and the only choice that works
         before a project exists, so it is an entry in the list rather than a
         cleared state of it. -->
    <v-select
      v-model="project"
      :items="[{ title: t('settings.agents.rules.workspaceRoot'), value: null }, ...projects]"
      :label="t('settings.agents.rules.writeInto')"
      density="compact"
      variant="outlined"
      hide-details
      class="mb-1"
      style="max-width: 22rem"
      @update:model-value="load"
    />
    <div class="text-caption text-medium-emphasis mb-4">
      {{ t('settings.agents.rules.writeIntoDetail') }}
    </div>

    <div class="text-overline text-medium-emphasis">
      {{ t('settings.agents.rules.scopeWorkspace') }}
    </div>
    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="row in workspaceRules" :key="`${row.id}:${row.scope}`" class="px-0">
        <template #prepend>
          <v-icon
            :icon="row.installed ? 'mdi-check-circle-outline' : 'mdi-file-document-outline'"
            :color="{ installed: 'success', stale: 'warning' }[rulesState(row)]"
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2">{{ row.label }}</v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ t(`settings.agents.rules.state.${rulesState(row)}`) }} — <code>{{ row.path }}</code>
        </v-list-item-subtitle>

        <template #append>
          <v-btn
            v-if="row.installed"
            size="small"
            variant="text"
            :loading="rulesBusy === `${row.id}:${row.scope}`"
            :disabled="rulesBusy !== null"
            @click="removeRules(row)"
          >
            {{ t('settings.agents.remove') }}
          </v-btn>
          <v-btn
            v-if="!row.current"
            size="small"
            variant="tonal"
            color="primary"
            class="ml-2"
            :loading="rulesBusy === `${row.id}:${row.scope}`"
            :disabled="rulesBusy !== null"
            @click="applyRules(row)"
          >
            {{
              rulesState(row) === 'stale'
                ? t('settings.agents.update')
                : t('settings.agents.rules.add')
            }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <div class="text-overline text-medium-emphasis mt-4">
      {{ t('settings.agents.rules.scopeGlobal') }}
    </div>
    <div class="text-caption text-medium-emphasis mb-1">
      {{ t('settings.agents.rules.globalDetail') }}
    </div>
    <v-list density="compact" class="bg-transparent">
      <v-list-item v-for="row in globalRules" :key="`${row.id}:${row.scope}`" class="px-0">
        <template #prepend>
          <v-icon
            :icon="row.installed ? 'mdi-check-circle-outline' : 'mdi-earth'"
            :color="{ installed: 'success', stale: 'warning' }[rulesState(row)]"
            class="mr-3"
          />
        </template>

        <v-list-item-title class="text-body-2">{{ row.label }}</v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          {{ t(`settings.agents.rules.state.${rulesState(row)}`) }} — <code>{{ row.path }}</code>
        </v-list-item-subtitle>

        <template #append>
          <v-btn
            v-if="row.installed"
            size="small"
            variant="text"
            :loading="rulesBusy === `${row.id}:${row.scope}`"
            :disabled="rulesBusy !== null"
            @click="removeRules(row)"
          >
            {{ t('settings.agents.remove') }}
          </v-btn>
          <v-btn
            v-if="!row.current"
            size="small"
            variant="tonal"
            color="primary"
            class="ml-2"
            :loading="rulesBusy === `${row.id}:${row.scope}`"
            :disabled="rulesBusy !== null"
            @click="applyRules(row)"
          >
            {{
              rulesState(row) === 'stale'
                ? t('settings.agents.update')
                : t('settings.agents.rules.add')
            }}
          </v-btn>
        </template>
      </v-list-item>
    </v-list>
  </SettingsGroup>
</template>
