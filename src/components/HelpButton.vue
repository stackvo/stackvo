<script setup>
import { useI18n } from 'vue-i18n';
import { useHelp } from '@/composables/useHelp';

/**
 * The one control every card carries on the opposite side from its name: what
 * is this, and what do the buttons in it do.
 *
 * An icon and not a line of text, because it repeats on some forty cards and a
 * word on each of them would read as forty different offers. Quiet by default —
 * this is a card's footnote, not one of its actions — and it takes the accent
 * colour on hover so it is findable once you are looking for it.
 */
defineProps({
  topic: { type: String, required: true },
});

const { t } = useI18n();
const { openHelp } = useHelp();
</script>

<template>
  <v-btn
    icon
    size="x-small"
    variant="text"
    class="help-btn"
    :aria-label="t('a11y.help')"
    @click.stop="openHelp(topic)"
  >
    <v-icon size="18">mdi-help-circle-outline</v-icon>
    <v-tooltip activator="parent" location="bottom">{{ t('a11y.help') }}</v-tooltip>
  </v-btn>
</template>

<style scoped>
/* Present but not competing: the card's name and its controls come first, and
   a full-strength icon in the corner of every card reads as a row of them. */
.help-btn {
  opacity: 0.6;
}

.help-btn:hover,
.help-btn:focus-visible {
  opacity: 1;
  color: rgb(var(--v-theme-primary));
}
</style>
