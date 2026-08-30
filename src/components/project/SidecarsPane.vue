<script setup>
import { onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Containers this project's own repository brought with it.
 *
 * ## Why this screen had to exist before the feature really did
 *
 * The `sidecars` block was parsed, validated, refused when it asked for the
 * host, and rendered into the project's compose file — and nothing anywhere
 * showed it. `hooks`, the sibling block in the same manifest, has had a pane
 * since it was written. So the app could answer "can I have Qdrant or Ollama"
 * with "write a `sidecars` block" while nobody reading the app could find out
 * that the block exists, which is the same as not having it.
 *
 * ## The one thing a reader cannot work out
 *
 * The hostname. A sidecar is reachable only from inside the project's own
 * network — no host port, no host path, by construction — so the application
 * connects to it by container name, and that name is derived from the project
 * rather than declared. It comes from the backend already derived, for the
 * reason `useWorkers` gets its container name the same way: a second copy of a
 * naming rule in JavaScript is a second thing to be wrong about, and the whole
 * purpose of this rule is that two clones of one repository cannot collide.
 *
 * The card is absent rather than empty when a project declares none, which is
 * every project until somebody writes one — an empty pane reading "no
 * sidecars" answers a question nobody asked.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const sidecars = ref([]);
const error = ref(null);
const loading = ref(false);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    sidecars.value = asList(await api.projectSidecars(props.name));
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch(() => props.name, load);
</script>

<template>
  <v-card v-if="sidecars.length" variant="flat" class="pane">
    <PaneHeader
      help="project-sidecars"
      icon="mdi-cube-outline"
      :title="t('sidecars.title')"
      :description="t('sidecars.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <div v-for="sidecar in sidecars" :key="sidecar.id" class="sidecar">
      <div class="d-flex align-center ga-2 mb-1">
        <span class="text-body-2 font-weight-medium">{{ sidecar.id }}</span>
        <code class="image">{{ sidecar.image }}</code>
      </div>

      <!-- `lang=""` — undetermined, not `en`. This sentence is not the app's:
           it is whatever the project wrote under `sidecars.<id>.about` in its
           own `stackvo.json`, so nothing here knows what language it is in.
           Claiming English would be a guess stated as a fact, which for a
           screen reader is worse than saying nothing is known. Same rule as
           `LogView` and `DumpValue`; see `language-of-parts.spec.js`. -->
      <div v-if="sidecar.about" class="text-caption text-medium-emphasis mb-1" lang="">
        {{ sidecar.about }}
      </div>

      <!-- The load-bearing line: what the application puts in its own config.
           Shown before the details, because it is the only one anybody needs
           on the first read. -->
      <div class="text-caption mb-1">
        {{ t('sidecars.reachedAt') }}
        <code class="host">{{ sidecar.container }}</code>
      </div>

      <div v-if="sidecar.command?.length" class="text-caption text-medium-emphasis">
        <code>{{ sidecar.command.join(' ') }}</code>
      </div>

      <div
        v-for="volume in sidecar.volumes"
        :key="volume.name"
        class="text-caption text-medium-emphasis"
      >
        <v-icon size="12" class="mr-1">mdi-database-outline</v-icon>
        <code>{{ volume.volume }}</code> → {{ volume.path }}
      </div>
    </div>

    <!-- Said once, at the bottom: it is a property of every row and a reason
         rather than a limitation somebody should try to work around. -->
    <div class="text-caption text-medium-emphasis mt-2">
      <v-icon size="14" class="mr-1">mdi-lan-disconnect</v-icon>{{ t('sidecars.noHost') }}
    </div>
  </v-card>
</template>

<style scoped>
.sidecar {
  padding: 6px 0;
}

.sidecar + .sidecar {
  border-top: 1px solid rgb(var(--v-border-color), var(--v-border-opacity));
}

.image,
.host {
  min-width: 0;
  font-size: 0.75rem;
  word-break: break-all;
}
</style>
