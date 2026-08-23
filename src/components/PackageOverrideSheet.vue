<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import SideSheet from '@/components/SideSheet.vue';

/**
 * Taking over one file of a package somebody else published (decision 0031).
 *
 * ## Why this is a list of files and not an editor
 *
 * The same reasoning `PackageAuthorDialog` gives, and the same one `quickcmd`
 * gives for opening the user's terminal: a box in a settings pane is a worse
 * place to edit compose YAML than whatever they already use. So this hands back
 * a path and gets out of the way.
 *
 * ## The copy lives beside the package, not in it
 *
 * A manifest states the hash of every file it ships and StackVo checks them on
 * every read, so editing the fetched file in place produces a package that
 * refuses to load. The workspace's copy goes under `overrides/` instead: the
 * package stays intact, a reinstall is exactly as safe as before, and the
 * override survives it.
 *
 * ## Revert is destructive and says so
 *
 * The file being deleted is somebody's edit and nothing restores it. The
 * confirmation is inline — a second dialog over a side sheet is a stack of two
 * things covering the list the user is working from.
 */
const props = defineProps({
  service: { type: String, default: '' },
  version: { type: String, default: '' },
});

const emit = defineEmits(['changed']);

const model = defineModel({ type: Boolean, default: false });

const { t } = useI18n();

const files = ref([]);
const error = ref(null);
const busy = ref(false);
/** The path whose revert is waiting to be confirmed, or null. */
const confirming = ref(null);
/** Where the last take-over landed, so the next step is a path to copy. */
const landed = ref(null);

const named = computed(() => !!props.service && !!props.version);
const overriddenCount = computed(() => files.value.filter((f) => f.overridden).length);

async function load() {
  if (!named.value) return;
  busy.value = true;
  error.value = null;
  try {
    files.value = asList(await api.packageFiles(props.service, props.version));
  } catch (e) {
    error.value = e;
    files.value = [];
  } finally {
    busy.value = false;
  }
}

watch(
  () => [model.value, props.service, props.version],
  ([open]) => {
    if (!open) return;
    landed.value = null;
    confirming.value = null;
    load();
  },
  { immediate: true }
);

async function run(action) {
  busy.value = true;
  error.value = null;
  try {
    const result = await action();
    await load();
    // The catalogue row carries the count, so the page behind this sheet is
    // stale the moment anything here succeeds.
    emit('changed');
    return result;
  } catch (e) {
    error.value = e;
    return null;
  } finally {
    busy.value = false;
  }
}

async function take(path) {
  landed.value = await run(() => api.packageOverride(props.service, props.version, path));
}

async function revert(path) {
  confirming.value = null;
  landed.value = null;
  await run(() => api.packageOverrideRevert(props.service, props.version, path));
}

const kindIcon = (kind) =>
  ({
    compose: 'mdi-file-document-outline',
    config: 'mdi-cog-outline',
    companion: 'mdi-cube-outline',
  })[kind] ?? 'mdi-file-outline';
</script>

<template>
  <SideSheet
    v-model="model"
    icon="mdi-file-edit-outline"
    :title="t('overrides.title', { service, version })"
    :width="640"
  >
    <p class="text-caption text-medium-emphasis mb-4">{{ t('overrides.explain') }}</p>

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <v-alert
      v-if="overriddenCount"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-3"
      :text="t('overrides.inEffect', { count: overriddenCount })"
    />

    <v-list v-if="files.length" density="compact" class="pa-0">
      <template v-for="file in files" :key="file.path">
        <v-list-item :title="file.path">
          <template #prepend>
            <v-icon size="small" :color="file.overridden ? 'primary' : undefined">
              {{ kindIcon(file.kind) }}
            </v-icon>
          </template>

          <template #subtitle>
            <span>{{ t(`overrides.kind.${file.kind}`) }}</span>
            <span v-if="file.companion"> · {{ file.companion }}</span>
          </template>

          <template #append>
            <v-btn
              v-if="!file.overridden"
              size="small"
              variant="tonal"
              :loading="busy"
              @click="take(file.path)"
            >
              {{ t('overrides.take') }}
            </v-btn>
            <template v-else-if="confirming === file.path">
              <v-btn size="small" variant="text" @click="confirming = null">
                {{ t('app.cancel') }}
              </v-btn>
              <v-btn size="small" variant="flat" color="error" @click="revert(file.path)">
                {{ t('overrides.confirmRevert') }}
              </v-btn>
            </template>
            <v-btn v-else size="small" variant="text" @click="confirming = file.path">
              {{ t('overrides.revert') }}
            </v-btn>
          </template>
        </v-list-item>
      </template>
    </v-list>

    <p v-else-if="!busy" class="text-caption text-medium-emphasis">
      {{ t('overrides.none') }}
    </p>

    <!-- The path, because the next step is opening it in whatever they already
         use — and a regenerate, because nothing renders until one runs. -->
    <template v-if="landed">
      <v-divider class="my-4" />
      <div class="text-caption text-medium-emphasis mb-1">{{ t('overrides.landed') }}</div>
      <code class="text-caption">{{ landed }}</code>
      <div class="text-caption text-medium-emphasis mt-2">{{ t('overrides.thenRegenerate') }}</div>
    </template>

    <template #footer>
      <v-spacer />
      <v-btn variant="text" @click="model = false">{{ t('app.close') }}</v-btn>
    </template>
  </SideSheet>
</template>
