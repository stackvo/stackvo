<script setup>
import { ref, useId } from 'vue';
import HelpButton from '@/components/HelpButton.vue';

/**
 * A pane whose body folds away behind its own heading.
 *
 * For the two panes on the configuration tab whose content has no natural
 * size: the manifest editor is a 24-row textarea whatever the manifest says,
 * and the Dockerfile preview is however many lines the generator emitted —
 * around 120 for a PHP project with extensions. Between them they made a tab
 * you scroll for several screens to reach the panes underneath.
 *
 * Closed on arrival, which is the whole point: both answer questions that are
 * asked occasionally — "what exactly is in the manifest", "what would this
 * build from" — and neither is what the tab is for.
 *
 * The heading is the control. A separate chevron button beside a title that
 * does nothing gives the same action two sizes, and the small one is the only
 * one that works; here the whole heading is the button, and anything that acts
 * on the pane rather than opening it goes in `actions`, outside the fold.
 *
 * The heading itself is `PaneHeader`'s — the same icon, name and sentence every
 * other pane on the page now carries — rendered inside the button rather than
 * beside it, so what you click is still the whole thing. Hence props rather
 * than a title slot: the caller names the pane, the pane draws it the one way.
 */
defineProps({
  icon: { type: String, default: '' },
  title: { type: String, default: '' },
  description: { type: String, default: '' },
  /** The topic its help button opens. See `lib/help.js`. */
  help: { type: String, default: '' },
});
const expanded = ref(false);

/**
 * `aria-controls` needs the body to have an id, and two of these render on one
 * page — so it cannot be a constant. `useId` is per-instance and stable across
 * the server/client boundary, unlike a module counter.
 */
const bodyId = useId();
</script>

<template>
  <v-card variant="flat" class="pane">
    <div class="d-flex align-center ga-3">
      <button
        type="button"
        class="pane-toggle"
        :aria-expanded="expanded"
        :aria-controls="bodyId"
        @click="expanded = !expanded"
      >
        <v-icon size="18">{{ expanded ? 'mdi-chevron-down' : 'mdi-chevron-right' }}</v-icon>
        <v-avatar rounded="lg" size="36" color="primary">
          <v-icon size="18">{{ icon }}</v-icon>
        </v-avatar>
        <span class="min-width-0">
          <span class="d-block text-body-2 font-weight-medium">{{ title }}</span>
          <span v-if="description" class="d-block text-caption text-medium-emphasis">
            {{ description }}
          </span>
        </span>
      </button>

      <!-- Beside the title rather than inside the fold: a pane says what it is
           worth opening for while it is shut, or it is a row of headings. -->
      <slot name="meta" />

      <v-spacer />

      <HelpButton v-if="help" :topic="help" :subject="title" />
    </div>

    <!-- Outside the fold and inside the card. A failure the pane is reporting
         about itself has to be readable while the body is shut, and it belongs
         within the card's own border rather than floating above it, where it
         reads as a page-level failure stuck to the card above. -->
    <div v-if="$slots.alert" class="pane-alert">
      <slot name="alert" />
    </div>

    <!-- `v-show`, not `v-if`. The body holds a draft the user may have typed
         and a preview that costs a backend render to produce; unmounting it
         would make the fold a way to lose both. Hidden content takes no
         height, which is what was being asked for. -->
    <v-expand-transition>
      <div v-show="expanded" :id="bodyId" class="pane-body">
        <slot />
      </div>
    </v-expand-transition>

    <!-- Outside the fold, at the foot. These act on the project rather than on
         the view of the file, so they must stay reachable while the pane is
         shut — and under the contents rather than level with the heading, which
         is where the rest of the page now puts a control that commits. -->
    <div v-if="$slots.actions" class="pane-foot">
      <v-spacer />
      <slot name="actions" />
    </div>
  </v-card>
</template>

<style scoped>
/* The reset a button needs to read as the heading it replaced, plus the cursor
   and the focus ring it needs to read as a button. */
.pane-toggle {
  appearance: none;
  background: none;
  border: 0;
  padding: 0;
  font: inherit;
  text-align: inherit;
  color: inherit;

  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  border-radius: 4px;
}

.pane-toggle:focus-visible {
  outline: 2px solid rgb(var(--v-theme-primary));
  outline-offset: 2px;
}

/* Under the heading, whatever the fold is doing. */
.pane-alert {
  padding-top: 16px;
}

/* Only when open. A margin above a hidden body is a gap under the heading of
   every closed pane. */
.pane-body {
  padding-top: 16px;
}

/* A long description must wrap inside the button rather than push whatever the
   caller put in `meta` off the row. */
.pane-toggle .min-width-0 {
  min-width: 0;
}
</style>
