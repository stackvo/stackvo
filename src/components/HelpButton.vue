<script setup>
import { useI18n } from 'vue-i18n';
import { useHelp } from '@/composables/useHelp';

/**
 * The one control every card carries on the opposite side from its name: what
 * is this, and what do the buttons in it do.
 *
 * An icon and not a line of text, because it repeats on some forty cards and a
 * word on each of them would read as forty different offers. Quiet by default —
 * this is a card's footnote, not one of its actions — and it comes to full
 * strength on hover, in whatever colour the surface underneath it is already
 * giving its text.
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
    size="small"
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

/* Opacity and nothing else.
 *
 * This used to take the accent colour on hover, which works on a card and
 * disappears on the page banner: `PageLayout`'s toolbar is `bg-primary`, so
 * hovering the one help button every page carries painted a primary glyph on a
 * primary field and the icon vanished under the cursor. `SideSheet`'s header is
 * the same surface and had the same hole. Inheriting the colour is what makes
 * this safe wherever a card puts it — white on the banner, on-surface in a
 * card — and the button's own overlay is what tints under the pointer. */
.help-btn:hover,
.help-btn:focus-visible {
  opacity: 1;
}

/* A floor under the box, because the size of this one is not this component's
   to decide. Appearance offers a density and `global.density` reaches every
   button: at compact, `size="small"` computes to 20px around an 18px glyph, and
   the previous `x-small` computed to 12px — an icon overhanging its own button
   by three pixels on every side, where the ring you can see is not the ring you
   can press. The floor holds the target at 28px through all three densities
   without touching what the rest of the interface does with the setting. */
.help-btn.v-btn--icon {
  min-width: 28px;
  min-height: 28px;
}
</style>
