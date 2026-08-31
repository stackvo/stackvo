<script setup>
import { onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * K-7 — the other runtimes this repository declared.
 *
 * ## Why it is a pane rather than a line on the overview
 *
 * The `SidecarsPane` beside it exists because a block that is parsed, validated
 * and rendered and that **nothing shows** is the same as not having the
 * feature. The same is true here and more so: a component is a directory of the
 * user's own code that this app builds and routes, and the two derived names —
 * the container and the hostname — exist nowhere in the repository. They are
 * computed here, so this is the only place anybody can read them.
 *
 * Renders nothing for a project with no components, which is nearly every one.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const components = ref([]);
const error = ref(null);

async function load() {
  error.value = null;
  try {
    components.value = asList(await api.projectComponents(props.name));
  } catch (e) {
    components.value = [];
    error.value = e;
  }
}

onMounted(load);
watch(() => props.name, load);
</script>

<template>
  <v-card v-if="components.length" variant="flat" class="pane">
    <PaneHeader
      help="project-components"
      icon="mdi-source-repository-multiple"
      :title="t('components.title')"
      :description="t('components.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <div v-for="part in components" :key="part.id" class="part" data-test="component">
      <div class="d-flex align-center ga-2 mb-1 flex-wrap">
        <span class="text-body-2 font-weight-medium">{{ part.id }}</span>
        <v-chip size="x-small" label>{{ part.runtime }} {{ part.version }}</v-chip>
        <code class="mono">{{ part.path }}/</code>
      </div>

      <!-- The two derived names, which exist nowhere in the repository: this
           is the only screen that can tell anybody what they are. -->
      <div v-if="part.domain" class="text-caption mb-1">
        {{ t('components.servedAt') }}
        <code class="mono">https://{{ part.domain }}</code>
      </div>
      <!-- Said, not left blank. A part with no domain is not misconfigured —
           it is a worker, and that is a different thing from a broken site. -->
      <div v-else class="text-caption text-medium-emphasis mb-1">
        {{ t('components.noDomain') }}
      </div>

      <div class="text-caption mb-1">
        {{ t('components.reachedAt') }}
        <code class="mono">{{ part.container }}:{{ part.port }}</code>
      </div>

      <div class="text-caption text-medium-emphasis">
        <code class="mono">{{ part.start }}</code>
      </div>
    </div>

    <!-- Said once at the bottom: a property of every row, and a reason rather
         than a limitation to work around. -->
    <div class="text-caption text-medium-emphasis mt-2">
      <v-icon size="14" class="mr-1">mdi-lan-disconnect</v-icon>{{ t('components.noHost') }}
    </div>
  </v-card>
</template>

<style scoped>
.part {
  padding: 6px 0;
}

.part + .part {
  border-top: 1px solid rgb(var(--v-border-color), var(--v-border-opacity));
}

.mono {
  min-width: 0;
  font-size: 0.75rem;
  word-break: break-all;
}
</style>
