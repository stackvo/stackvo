<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { useCopyTick } from '@/composables/useCopyTick';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * The editor itself, inside the container.
 *
 * `XdebugPane` wires an IDE on *this machine* to a debugger in the container.
 * This is the other half of that idea: the editor running in the image, with
 * the language server, the extensions, the terminal, `composer` and `artisan`
 * all in there and no PHP on the host at all.
 *
 * The whole feature is an address. VS Code has no attach-by-name command line
 * — it opens a running container through a remote authority — and every fact
 * that address is built from was already on this page: the container's name,
 * the directory the source is mounted at, and the mount itself.
 *
 * ## Why the address is on screen and not only behind the button
 *
 * The button needs a VS Code this app can start, which is a `code` on `PATH`
 * or the application in `/Applications`. Neither is a given, and a machine
 * with neither still has an address that works — pasted into VS Code's own
 * "Open Folder" or into a colleague's message. Hiding it would make a missing
 * launcher look like a missing feature.
 *
 * ## Why a refusal rather than a warning
 *
 * A container built with `COPY . .` holds a *copy* of the source. An editor
 * opened onto it works perfectly — the files are there, saving succeeds, the
 * language server is happy — and every line written is thrown away by the next
 * rebuild, with nothing on screen having said so. `editor.rs` reads the
 * container's own mount table and refuses, and this pane says which of the two
 * reasons it was.
 */
const props = defineProps({
  name: { type: String, required: true },
  /** Re-read when the container comes up or goes down. */
  running: { type: Boolean, default: false },
});

/** The container predates the server volume; recreating it is the fix. */
const emit = defineEmits(['apply']);

const { t } = useI18n();
const { copied, copy } = useCopyTick();

const status = ref(null);
const busy = ref(false);
const error = ref(null);

const readiness = computed(() => status.value?.readiness ?? null);
/**
 * The other editor, which needs a file rather than an address.
 *
 * PhpStorm has no attach-to-a-running-container connection type. What it has
 * is Dev Containers, and a dev container that names StackVo's own compose
 * files and this project's service is not a second container — it is the one
 * already running. So this half is a file StackVo writes and a path the user
 * points the IDE at.
 */
const jetbrains = computed(() => status.value?.jetbrains ?? null);
/** One reason, not a list: the first is the one to act on. */
const blocker = computed(() => readiness.value?.blockers?.[0] ?? null);
const caveats = computed(() => readiness.value?.caveats ?? []);
const attachable = computed(() => Boolean(readiness.value?.attachable));
const installed = computed(() => Boolean(status.value?.editorInstalled));

/**
 * Re-read the container.
 *
 * `keepError` is what stops the re-read from swallowing the refusal that
 * caused it. Pressing the button and being told no ends in a fresh read — the
 * container is not what the pane thought it was — and a plain "clear the error
 * first" would blank the sentence explaining why, half a frame after it
 * appeared.
 */
async function load({ keepError = false } = {}) {
  try {
    status.value = await api.editorStatus(props.name);
    if (!keepError) error.value = null;
  } catch (e) {
    // Not fatal to the page: this pane is one button, and a project whose
    // container cannot be read is a state the rest of the tab already shows.
    status.value = null;
    error.value = e;
  }
}

async function attach() {
  busy.value = true;
  error.value = null;
  try {
    await api.editorAttach(props.name);
  } catch (e) {
    error.value = e;
  } finally {
    // Re-read either way. A refusal here means the container changed under the
    // render, and the pane should now be showing what it changed to — with the
    // refusal still on screen, which is what `keepError` is for.
    busy.value = false;
    await load({ keepError: Boolean(error.value) });
  }
}

/** Write (or refresh) the file PhpStorm is pointed at. */
async function writeJetbrains() {
  busy.value = true;
  error.value = null;
  try {
    await api.editorJetbrainsWrite(props.name);
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
    await load({ keepError: Boolean(error.value) });
  }
}

watch(
  () => [props.name, props.running],
  () => load(),
  { immediate: true }
);
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-editor"
      icon="mdi-microsoft-visual-studio-code"
      :title="t('containerEditor.title')"
      :description="t('containerEditor.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" @close="error = null" />

    <template v-if="readiness">
      <!-- The two refusals, each with the thing that answers it. -->
      <v-alert v-if="blocker === 'notRunning'" type="info" variant="tonal" class="mb-4">
        <div class="text-caption">{{ t('containerEditor.notRunning') }}</div>
      </v-alert>
      <v-alert
        v-else-if="blocker === 'sourceIsASnapshot'"
        type="warning"
        variant="tonal"
        class="mb-4"
      >
        <div class="text-caption">
          {{ t('containerEditor.snapshot', { workdir: readiness.workdir }) }}
        </div>
      </v-alert>

      <div class="d-flex align-center ga-3 flex-wrap">
        <v-btn
          color="primary"
          variant="flat"
          prepend-icon="mdi-microsoft-visual-studio-code"
          :loading="busy"
          :disabled="!attachable || !installed || busy"
          @click="attach"
        >
          {{ t('containerEditor.open') }}
        </v-btn>
        <span v-if="attachable" class="text-caption text-medium-emphasis">
          {{
            t('containerEditor.opens', {
              container: readiness.container,
              workdir: readiness.workdir,
            })
          }}
        </span>
      </div>

      <!-- No VS Code on this machine. The address below still works. -->
      <v-alert v-if="!installed" type="info" variant="tonal" density="compact" class="mt-4">
        <div class="text-caption">{{ t('containerEditor.noEditor') }}</div>
      </v-alert>

      <!-- Worth saying, never a refusal. -->
      <v-alert
        v-if="caveats.includes('serverIsNotKept')"
        type="warning"
        variant="tonal"
        density="compact"
        class="mt-4"
      >
        <div class="text-caption">{{ t('containerEditor.serverNotKept') }}</div>
        <template #append>
          <v-btn size="small" variant="text" @click="emit('apply')">
            {{ t('containerEditor.recreate') }}
          </v-btn>
        </template>
      </v-alert>
      <div v-if="caveats.includes('musl')" class="text-caption text-medium-emphasis mt-3">
        {{ t('containerEditor.musl') }}
      </div>

      <!-- The address itself, because a missing launcher is not a missing
           feature: this string opens the same container from VS Code's own
           Open Folder dialog. -->
      <div class="section-head mt-5 mb-1">
        <v-icon size="18" class="mr-2">mdi-link-variant</v-icon>
        {{ t('containerEditor.address') }}
      </div>
      <p class="text-caption text-medium-emphasis mb-2">{{ t('containerEditor.addressWhy') }}</p>
      <div class="d-flex align-start ga-2">
        <pre class="snippet flex-grow-1">{{ readiness.folderUri }}</pre>
        <v-btn
          icon
          size="small"
          variant="text"
          :aria-label="t('a11y.copy')"
          @click="copy(readiness.folderUri, 'editorUri')"
        >
          <v-icon>{{ copied === 'editorUri' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
          <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
        </v-btn>
      </div>

      <div class="text-caption text-medium-emphasis mt-4">
        {{ t('containerEditor.serverNote', { dir: readiness.serverDir }) }}
      </div>

      <!-- The other editor. Not the same act and not the same shape: VS Code
           is handed an address, PhpStorm is handed a file — because JetBrains
           has no connection type that attaches to a container that is already
           running, and its Dev Containers are pointed at a compose service
           instead. -->
      <template v-if="jetbrains">
        <div class="section-head mt-6 mb-1">
          <v-icon size="18" class="mr-2">mdi-alpha-p-box-outline</v-icon>
          {{ t('containerEditor.jbTitle') }}
        </div>
        <p class="text-caption text-medium-emphasis mb-2">{{ t('containerEditor.jbWhy') }}</p>

        <!-- Alpine: for VS Code a note, here the end of the road. Stated
             before the file, because a file that is correct and a door that
             cannot open are two different things to be told. -->
        <v-alert
          v-if="jetbrains.musl"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          <div class="text-caption">{{ t('containerEditor.jbMusl') }}</div>
        </v-alert>

        <v-alert
          v-if="!jetbrains.installed"
          type="info"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          <div class="text-caption">{{ t('containerEditor.jbNotInstalled') }}</div>
        </v-alert>

        <!-- A file naming an older list of compose files is worse than no file:
             it opens a container assembled from fewer overlays than the one
             StackVo starts. -->
        <v-alert
          v-else-if="jetbrains.exists && !jetbrains.current"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          <div class="text-caption">{{ t('containerEditor.jbStale') }}</div>
        </v-alert>

        <div class="d-flex align-center ga-3 flex-wrap">
          <v-btn
            variant="tonal"
            prepend-icon="mdi-file-document-edit-outline"
            :loading="busy"
            :disabled="busy"
            @click="writeJetbrains"
          >
            {{ jetbrains.exists ? t('containerEditor.jbRewrite') : t('containerEditor.jbWrite') }}
          </v-btn>
          <span class="text-caption text-medium-emphasis">
            {{ t('containerEditor.jbService', { service: jetbrains.service }) }}
          </span>
        </div>

        <template v-if="jetbrains.exists">
          <div class="d-flex align-start ga-2 mt-3">
            <pre class="snippet flex-grow-1">{{ jetbrains.path }}</pre>
            <v-btn
              icon
              size="small"
              variant="text"
              :aria-label="t('a11y.copy')"
              @click="copy(jetbrains.path, 'editorJetbrains')"
            >
              <v-icon>{{ copied === 'editorJetbrains' ? 'mdi-check' : 'mdi-content-copy' }}</v-icon>
              <v-tooltip activator="parent">{{ t('a11y.copy') }}</v-tooltip>
            </v-btn>
          </div>
          <div class="text-caption text-medium-emphasis mt-2">
            {{ t('containerEditor.jbSteps') }}
          </div>
        </template>

        <!-- The cost this half cannot design away, so it says it. -->
        <div v-if="jetbrains.recreates" class="text-caption text-medium-emphasis mt-2">
          {{ t('containerEditor.jbRecreates') }}
        </div>
      </template>
    </template>
  </v-card>
</template>
