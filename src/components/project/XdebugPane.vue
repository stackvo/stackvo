<script setup>
import { computed, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useXdebug } from '@/composables/useXdebug';
import { api, asList } from '@/lib/ipc';
import { useCopyTick } from '@/composables/useCopyTick';
import { useOperationsStore } from '@/stores/operations';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';
import RemedyAlert from '@/components/project/RemedyAlert.vue';

/**
 * Xdebug's three layers, and the switch that moves all of them.
 *
 * One of three panes out of the Debug section in the pane split. Toggling rewrites
 * the manifest on disk, which is also what the Configuration section is
 * showing — hence `changed`: the view re-reads the manifest rather than this
 * pane reaching across to an editor it does not own.
 *
 * ## Two warnings, two different fixes, and both are actionable
 *
 * The warnings used to end at a sentence. "The extension is compiled into the
 * image, so this has no effect until the project is regenerated and rebuilt"
 * is a true thing to say and a useless place to stop: the reader is left
 * holding a fact and a page to go hunting through for the button that answers
 * it.
 *
 * They are deliberately **two** remedies and not one. A first switch-on has to
 * rebuild the image, which is minutes; a container that merely predates the
 * overlay needs recreating, which is seconds. Offering the expensive one for
 * both would teach people to reach for it every time, and offering the cheap
 * one for both would produce a container that still has no extension in it.
 *
 * Both are `RemedyAlert` now rather than two hand-written alerts and two emits
 * the page turned back into calls. This pane is where that pattern was worked
 * out; three other cards were re-deriving it, one of them badly, so it became
 * a component. The re-read this pane already does on the falling edge of the
 * busy flag is what that component emits as `done`, so nothing here changed
 * except who owns the markup.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: '' },
  running: { type: Boolean, default: false },
});

const emit = defineEmits(['changed']);

const { t } = useI18n();
const ops = useOperationsStore();

const { status, busy, error, load, toggle } = useXdebug(toRef(props, 'name'));
const { copied, copy } = useCopyTick();

/**
 * The IDE half.
 *
 * The three values above are what an IDE needs and were already on this
 * screen — as strings to copy into a dialog by hand, which is exactly what
 * every competitor's documentation asks for and what all five of them then
 * name as the usual reason a breakpoint never hits. This writes them where it
 * safely can, and shows them where it cannot.
 */
const ide = ref(null);
const ideBusy = ref(null);
const ideError = ref(null);

/** The fact that is in no file: is the IDE actually listening? */
const listening = computed(() => Boolean(ide.value?.listener?.process || ide.value?.listener?.pid));

async function loadIde() {
  ideError.value = null;
  try {
    ide.value = await api.ideDebugStatus(props.name);
  } catch (e) {
    // Not fatal to the pane: the switch above works without any of this, and
    // a project whose directory has gone is a case the row cannot fix anyway.
    ide.value = null;
    ideError.value = e;
  }
}

/** Rows first for the editor this project is actually opened in. */
const ideTargets = computed(() =>
  [...asList(ide.value?.targets)].sort((a, b) => Number(b.detected) - Number(a.detected))
);

function ideState(target) {
  if (target.method === 'shown') return 'shown';
  if (!target.parseable) return 'unparseable';
  if (target.installed && target.current) return 'written';
  if (target.installed) return 'stale';
  return 'absent';
}

async function writeIde(target) {
  ideBusy.value = target.id;
  ideError.value = null;
  try {
    await api.ideDebugApply(props.name, target.id);
  } catch (e) {
    ideError.value = e;
  } finally {
    // Re-read either way: a failed write may still have changed the file, and
    // a row describing the old state would be a claim nobody checked.
    await loadIde();
    ideBusy.value = null;
  }
}

async function removeIde(target) {
  ideBusy.value = target.id;
  ideError.value = null;
  try {
    await api.ideDebugRemove(props.name, target.id);
  } catch (e) {
    ideError.value = e;
  } finally {
    await loadIde();
    ideBusy.value = null;
  }
}

watch(
  () => [props.name, props.runtime],
  () => {
    load(props.runtime);
    loadIde();
  },
  { immediate: true }
);

/**
 * Re-read when the operation this pane started finishes.
 *
 * Without this the buttons above were one-way. `compose_up_project` and
 * `project_build` return an **operation id as soon as the work starts**, not
 * when it ends — that is the whole point of the operation console — so the
 * caller's `await` resolves while docker is still recreating the container.
 * Anything that re-read at that moment read the old container, and anything
 * that did not re-read at all (which is what this pane did) went on showing
 * "the container is in debug, the setting is profile" over a container that
 * had already been fixed. The user pressed the button, watched it work, and
 * was told it had not.
 *
 * The falling edge of the busy flag is the signal, because that flag is set by
 * the operation's own `finished` event rather than by the call returning.
 */
watch(
  () => ops.isBusy(props.name),
  (busyNow, wasBusy) => {
    if (wasBusy && !busyNow) {
      load(props.runtime);
      loadIde();
    }
  }
);

async function set(enabled) {
  if (await toggle(enabled)) emit('changed');
  // The port and the mapping do not move, but "is it written" can: a project
  // switched off and on again is the moment somebody looks at this list.
  await loadIde();
}

defineExpose({ ideState });
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-xdebug"
      icon="mdi-bug-outline"
      :title="t('xdebug.title')"
      :description="t('xdebug.subtitle')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <template v-if="status">
      <v-switch
        :model-value="status.enabled"
        :loading="busy"
        :disabled="busy"
        color="primary"
        hide-details
        density="comfortable"
        :label="status.enabled ? t('xdebug.on') : t('xdebug.off')"
        @update:model-value="set($event)"
      />

      <!-- The extension is compiled in, so the manifest can be ahead of
           the image. Saying nothing here is how a toggle becomes a lie. -->
      <!-- Switching on for the first time puts the extension in the
           image and costs a rebuild; every time after that it moves one
           environment variable and costs a container recreate. Without saying
           so, the second toggle looks identical to the first and being much
           faster reads as a fault rather than as the point. -->
      <div v-if="!status.compiledIn" class="text-caption text-medium-emphasis mt-2">
        {{ t('xdebug.firstTime') }}
      </div>
      <div v-else-if="!status.enabled" class="text-caption text-medium-emphasis mt-2">
        {{ t('xdebug.staysInstalled') }}
      </div>

      <!-- The warning used to end here, which left the reader holding a
           sentence and no way to act on it. The work it names is exactly what
           the header's Rebuild button does, so it is offered where the problem
           is stated rather than somewhere else on the page. Not done
           automatically: a rebuild is minutes and recreates the container, and
           a switch that quietly started one would be a surprise the user did
           not ask for. -->
      <RemedyAlert
        v-if="status.needsRebuild"
        :name="name"
        remedy="rebuild"
        :text="t('xdebug.needsRebuild')"
        :disabled="busy"
        class="mt-3"
      />
      <!-- A different fault with a different fix: the image has the extension,
           the container was created before the overlay. That is a recreate,
           which is seconds, not a rebuild. Offering the expensive one here
           would teach people to reach for it every time. -->
      <RemedyAlert
        v-else-if="status.enabled && status.running && status.active === false"
        :name="name"
        remedy="recreate"
        :text="t('xdebug.notActive')"
        :disabled="busy"
        class="mt-3"
      />
      <v-alert
        v-else-if="status.enabled && status.active === true"
        type="success"
        variant="tonal"
        class="mt-3"
      >
        <div class="text-caption">{{ t('xdebug.active') }}</div>
      </v-alert>
      <!-- Switched off and still running with it. The overlay no longer names
           this project, but a container's environment is fixed when it is
           created — so it keeps debugging until it is recreated, and saying
           nothing here is the same lie as saying nothing when it is switched
           on. Not a rebuild: the extension stays in the image on purpose, so
           there is nothing to build and a rebuild would cost minutes for
           nothing. -->
      <RemedyAlert
        v-else-if="!status.enabled && status.active === true"
        :name="name"
        remedy="recreate"
        :text="t('xdebug.stillActive')"
        :disabled="busy"
        class="mt-3"
      />

      <!-- The path mapping is the step people get wrong, and both halves
           are already known here. -->
      <template v-if="status.enabled">
        <div class="section-head mt-5 mb-2">
          <v-icon size="18" class="mr-2">mdi-tune</v-icon>{{ t('xdebug.ideSettings') }}
        </div>
        <v-table density="compact">
          <tbody>
            <tr>
              <td class="text-medium-emphasis">{{ t('xdebug.port') }}</td>
              <td class="mono">{{ status.port }}</td>
            </tr>
            <tr>
              <td class="text-medium-emphasis">{{ t('xdebug.ideKey') }}</td>
              <td class="mono">{{ status.ideKey }}</td>
            </tr>
            <tr v-if="status.serverName">
              <td class="text-medium-emphasis">{{ t('xdebug.serverName') }}</td>
              <td class="mono">{{ status.serverName }}</td>
            </tr>
            <tr>
              <td class="text-medium-emphasis">{{ t('xdebug.pathMapping') }}</td>
              <td class="mono">{{ status.hostPath }} → {{ status.containerPath }}</td>
            </tr>
            <tr v-if="status.peclVersion">
              <td class="text-medium-emphasis">{{ t('xdebug.version') }}</td>
              <td class="mono">{{ status.peclVersion }} (PHP {{ status.phpVersion }})</td>
            </tr>
          </tbody>
        </v-table>

        <!-- The one thing this design cannot fix, said where it will be
             read rather than left for someone to discover. -->
        <div class="text-caption text-medium-emphasis mt-3">
          {{ t('xdebug.cliCaveat') }}
        </div>

        <!-- Filling the values in, rather than leaving them to be typed.
             The listener comes first because it is the half that is in no
             file and the half an IDE never says out loud. -->
        <div class="section-head mt-5 mb-2">
          <v-icon size="18" class="mr-2">mdi-application-braces-outline</v-icon>
          {{ t('xdebug.ide.title') }}
        </div>

        <ErrorAlert v-if="ideError" :error="ideError" class="mb-3" />

        <v-alert
          v-if="ide && !ide.listener.unknown"
          :type="listening ? 'success' : 'warning'"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          <div class="text-caption">
            {{
              listening
                ? t('xdebug.ide.listening', {
                    process: ide.listener.process || t('xdebug.ide.someProcess'),
                    port: ide.listener.port,
                  })
                : t('xdebug.ide.notListening', { port: ide.port })
            }}
          </div>
        </v-alert>

        <v-list v-if="ide" density="compact" class="bg-transparent">
          <v-list-item v-for="target in ideTargets" :key="target.id" class="px-0">
            <template #prepend>
              <v-icon
                :icon="
                  ideState(target) === 'written'
                    ? 'mdi-check-circle-outline'
                    : 'mdi-application-braces-outline'
                "
                :color="
                  { written: 'success', stale: 'warning', unparseable: 'warning' }[ideState(target)]
                "
                class="mr-3"
              />
            </template>

            <v-list-item-title class="text-body-2">
              {{ target.label }}
              <span v-if="target.detected" class="text-caption text-medium-emphasis ml-2">
                {{ t('xdebug.ide.detected') }}
              </span>
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ t(`xdebug.ide.state.${ideState(target)}`) }} — <code>{{ target.path }}</code>
            </v-list-item-subtitle>

            <template #append>
              <v-btn
                v-if="ideState(target) === 'written' || ideState(target) === 'stale'"
                size="small"
                variant="text"
                :loading="ideBusy === target.id"
                :disabled="ideBusy !== null"
                @click="removeIde(target)"
              >
                {{ t('settings.agents.remove') }}
              </v-btn>
              <v-btn
                v-if="target.method === 'written' && target.parseable && !target.current"
                size="small"
                variant="tonal"
                color="primary"
                class="ml-2"
                :loading="ideBusy === target.id"
                :disabled="ideBusy !== null"
                @click="writeIde(target)"
              >
                {{
                  ideState(target) === 'stale' ? t('settings.agents.update') : t('xdebug.ide.write')
                }}
              </v-btn>
              <v-btn
                v-if="target.method === 'shown' || !target.parseable"
                size="small"
                variant="text"
                :prepend-icon="copied === target.id ? 'mdi-check' : 'mdi-content-copy'"
                @click="copy(target.snippet, target.id)"
              >
                {{ t('settings.agents.copyBlock') }}
              </v-btn>
            </template>
          </v-list-item>
        </v-list>

        <div class="text-caption text-medium-emphasis mt-2">
          {{ t('xdebug.ide.neverClobbers') }}
        </div>
      </template>
    </template>
  </v-card>
</template>
