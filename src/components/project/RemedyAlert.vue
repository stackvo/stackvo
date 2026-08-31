<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { useOperationsStore } from '@/stores/operations';
import ErrorAlert from '@/components/ErrorAlert.vue';

/**
 * A pane's warning, with the one button that answers it.
 *
 * ## Why this is a component and not four copies of a `v-alert`
 *
 * Half the panes on the project page can end up in a state whose only fix is
 * one of exactly two acts: **rebuild the project**, because the image is
 * behind what the manifest now says, or **recreate the container**, because
 * the container is behind what the compose overlay now says. Xdebug wrote both
 * by hand, the container editor wrote the second by hand, and the supervisord
 * pane wrote neither — it stated the problem, named the fix in prose
 * ("rebuild the project"), and left the reader to go and find the button.
 *
 * That is three different renderings of one idea, which is how a fourth pane
 * ends up with a fourth. It also made the same sentence actionable in one card
 * and not in the next, with nothing on screen explaining the difference. So
 * the alert, the icon, the label, the busy state and the re-read are one
 * object now, and a pane that discovers a fifth of these states adds a line.
 *
 * ## It runs the act rather than emitting it
 *
 * The earlier shape was an emit the page turned into a call. That worked and
 * cost every adopter three edits in two files — a `defineEmits`, a handler on
 * the page, and a prop wired through — which is exactly the friction that left
 * the supervisord pane with prose instead of a button.
 *
 * Nothing is lost by calling directly. `ProjectDetail` re-reads the project on
 * the falling edge of the same busy flag this sets, so the page still refreshes
 * itself; the flag is what disables every other action on the project while the
 * work runs; and the refusal is shown here, against the sentence that prompted
 * it, rather than in the page's banner at the top of a scrolled tab.
 *
 * ## Done means finished, not returned
 *
 * `project_build` and `compose_up_project` both resolve with an operation id as
 * soon as the work *starts* — that is what the operation console is for — so
 * awaiting them says nothing about the container. `done` is emitted on the
 * falling edge of the busy flag instead, which is set by the operation's own
 * finished event: the first instant at which a pane re-reading itself would
 * read the container this alert asked for. This is the "works by itself" half —
 * an adopter passes `@done="load"` and the card corrects itself.
 *
 * ## And it asks
 *
 * Neither act is done automatically on discovering the state. A rebuild is
 * minutes and recreates the container; a recreate is seconds but still drops
 * whatever was unsaved inside. A card that quietly started one would be a
 * surprise nobody asked for — which is what a warning with a button in it
 * exists to avoid.
 */
const props = defineProps({
  /** The project both remedies act on. */
  name: { type: String, required: true },
  /**
   * Which of the two.
   *
   * `rebuild` is the expensive one: the image does not contain what the
   * manifest declares, so it has to be built. `recreate` is the cheap one: the
   * image is right and the running container was created before the overlay
   * that configures it. Offering the expensive one for both teaches people to
   * reach for it every time; offering the cheap one for both produces a
   * container that still has nothing in it.
   */
  remedy: {
    type: String,
    required: true,
    validator: (value) => value === 'rebuild' || value === 'recreate',
  },
  /** The sentence saying what is wrong. The button says what will be done. */
  text: { type: String, default: '' },
  type: { type: String, default: 'warning' },
  density: { type: String, default: 'default' },
  /** For a pane with its own in-flight work that must not overlap this. */
  disabled: { type: Boolean, default: false },
});

const emit = defineEmits(['done']);

const { t } = useI18n();
const ops = useOperationsStore();

const REMEDIES = {
  rebuild: { icon: 'mdi-hammer-wrench', run: (name) => api.projectBuild(name) },
  recreate: { icon: 'mdi-autorenew', run: (name) => api.composeUpProject(name) },
};

const error = ref(null);

/**
 * Busy on the *project*, not on this button.
 *
 * Deliberately shared: two panes can both be showing a remedy for the same
 * cause, and a rebuild started from one of them is a rebuild the other must
 * not offer to start again.
 */
const running = computed(() => ops.isBusy(props.name));

async function run() {
  error.value = null;
  // Set here rather than waiting for the operation's `start` event: the event
  // arrives a round trip later, and a button that stays live for that long is
  // a button somebody presses twice.
  ops.markBusy(props.name, true);
  try {
    await REMEDIES[props.remedy].run(props.name);
  } catch (e) {
    error.value = e;
    // Refused before it began, so no finished event is coming to clear it.
    ops.markBusy(props.name, false);
  }
}

watch(running, (now, was) => {
  if (was && !now) emit('done');
});
</script>

<template>
  <v-alert :type="type" variant="tonal" :density="density">
    <div class="text-caption">{{ text }}</div>

    <!-- The refusal against the sentence that prompted it. -->
    <ErrorAlert v-if="error" :error="error" closable class="mt-2" @close="error = null" />

    <v-btn
      size="small"
      :color="type"
      variant="tonal"
      class="mt-2"
      :prepend-icon="REMEDIES[remedy].icon"
      :loading="running"
      :disabled="disabled || running"
      :data-test="`remedy-${remedy}`"
      @click="run"
    >
      {{ t(`remedy.${remedy}`) }}
    </v-btn>
  </v-alert>
</template>
