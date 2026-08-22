<script setup>
import HelpButton from '@/components/HelpButton.vue';

/**
 * The heading of one content card — a project-detail pane, the log page's
 * console, anything the page frames as its own card.
 *
 * The same header the Settings page gives every one of its group cards — a
 * filled icon, the name at body size, and a line underneath saying what the
 * card is for. The panes here were writing their own: an uppercase tracked
 * label with an inline icon, followed by a separate `<p>` for the sentence.
 * Two pages, two vocabularies for the same object, and the project page's was
 * the weaker one — uppercase tracking reads as a section divider inside a card,
 * not as the card's own name, which is why `SitePane` needed a second one two
 * lines below the first to say where its groups begin.
 *
 * `.section-head` stays for exactly that second use — a divider *within* a
 * pane. What changes is the top of the card, which is now the same thing
 * wherever a card is drawn.
 *
 * `append` is for a control acting on the whole pane — a save, a re-check, a
 * toggle — which belongs beside the name rather than lost under the contents.
 */
defineProps({
  icon: { type: String, default: '' },
  title: { type: String, default: '' },
  description: { type: String, default: '' },
  /** The topic its help button opens. See `lib/help.js`. */
  help: { type: String, default: '' },
});
</script>

<template>
  <div class="pane-head">
    <v-avatar rounded="lg" size="36" color="primary">
      <v-icon size="18">{{ icon }}</v-icon>
    </v-avatar>
    <div class="pane-head-text">
      <div class="text-body-2 font-weight-medium">{{ title }}</div>
      <div v-if="description" class="text-caption text-medium-emphasis">{{ description }}</div>
    </div>
    <v-spacer />
    <slot name="append" />
    <HelpButton v-if="help" :topic="help" />
  </div>
</template>

<style scoped>
/* `align-center` rather than `align-start`: most descriptions here are one
   line, and centring a 36px avatar against a two-line block still reads. */
.pane-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

/* A long domain or container path in the description must wrap rather than
   push the appended control off the row. */
.pane-head-text {
  min-width: 0;
}
</style>
