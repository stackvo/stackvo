<script setup>
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { useCopyTick } from '@/composables/useCopyTick';
import SettingsGroup from '@/components/SettingsGroup.vue';

/**
 * The commands this machine adds to every project.
 *
 * Four rivals sell "add your own command" on the front page — DDEV drops a file
 * in `.ddev/commands/`, Lando has a `tooling:` block, dde and Laragon each have
 * their own — and this application had exactly one way to do it: edit a
 * repository somebody else owns. `<root>/commands.json` is the layer above the
 * project, and this is where a person finds out it exists.
 *
 * ## Read-only, and that is the design rather than a stage
 *
 * The file is the interface, the way it is in every product named above: you
 * drop a file and the command is there. A form here would be a second way to
 * write the same JSON, and the two would disagree the first time somebody used
 * an editor. What this pane owes is the other half — **where the file goes,
 * what it found, and what it refused** — because a file-shaped interface with
 * no feedback is one you edit twice and give up on.
 */
const { t } = useI18n();
const { copied, copy } = useCopyTick();

const state = ref(null);
const loading = ref(true);

/** The rows, as `[id, command]`, in the order the file declared them. */
const rows = () => Object.entries(state.value?.commands ?? {});

async function load() {
  loading.value = true;
  try {
    // Best effort: a workspace that cannot be read is an empty pane with its
    // path line still showing, not a page that refuses to open.
    state.value = await api.machineCommands().catch(() => null);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <SettingsGroup
    help="settings-machine-commands"
    icon="mdi-console-line"
    :title="t('settings.machineCommands.title')"
    :description="t('settings.machineCommands.desc')"
  >
    <div class="d-flex align-center ga-2 mb-3">
      <code class="mono flex-grow-1">{{ state?.path ?? '—' }}</code>
      <v-btn
        size="x-small"
        variant="text"
        :icon="copied === 'path' ? 'mdi-check' : 'mdi-content-copy'"
        :aria-label="t('app.copy')"
        :disabled="!state?.path"
        @click="copy(state.path, 'path')"
      />
      <v-btn
        size="x-small"
        variant="text"
        icon="mdi-refresh"
        :aria-label="t('app.refresh')"
        @click="load"
      />
    </div>

    <!-- An absent file is the ordinary case, not an error. It gets the
         instructions; an empty one that exists gets a different sentence,
         because "you have not written one" and "yours declares nothing" are
         different situations to be in. -->
    <v-alert
      v-if="!loading && !state?.exists"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-3"
    >
      {{ t('settings.machineCommands.absent') }}
    </v-alert>

    <v-alert
      v-else-if="!loading && !rows().length && !state?.problems?.length"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-3"
    >
      {{ t('settings.machineCommands.empty') }}
    </v-alert>

    <v-list v-if="rows().length" density="compact" class="pa-0">
      <v-list-item v-for="[id, command] in rows()" :key="id" class="px-0">
        <v-list-item-title class="text-body-2">
          <code>{{ id }}</code>
          <v-chip v-if="command.interactive" size="x-small" variant="tonal" class="ml-2">
            {{ t('settings.machineCommands.interactive') }}
          </v-chip>
        </v-list-item-title>
        <v-list-item-subtitle class="text-caption">
          <!-- `lang=""` — this is what somebody typed into their own file, and
               nothing here knows what language they wrote it in. Same rule as
               a sidecar's description; see `language-of-parts.spec.js`. -->
          <code class="mono">{{ command.exec.join(' ') }}</code>
          <span v-if="command.about" lang="" class="ml-2">{{ command.about }}</span>
        </v-list-item-subtitle>
      </v-list-item>
    </v-list>

    <!-- Refused rows, in the shape and under the code a manifest's bad block
         produces — it is the same parser reporting the same rule. -->
    <v-alert
      v-for="(problem, i) in state?.problems ?? []"
      :key="i"
      type="warning"
      variant="tonal"
      density="compact"
      class="mt-2"
    >
      <code>{{ problem.path }}</code>
      <span lang="" class="ml-2">{{ problem.message }}</span>
    </v-alert>

    <p class="text-caption text-medium-emphasis mt-3">
      {{ t('settings.machineCommands.hint') }}
    </p>
  </SettingsGroup>
</template>
