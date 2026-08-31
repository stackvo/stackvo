<script setup>
import { useI18n } from 'vue-i18n';

/**
 * "Do this?" — the one dialog this application had none of.
 *
 * ## Why it exists rather than a `title` guard on each button
 *
 * The four stack actions in the app bar act on **everything**: they start,
 * stop, restart or rebuild every container on the machine at once. None of them
 * is undone by pressing another — stopping takes every site down now, and
 * starting them again is a different act with a different cost, not an undo.
 * A single mis-aimed click on a toolbar is the cheapest way to lose an
 * afternoon's running state, and a toolbar is exactly where mis-aimed clicks
 * happen.
 *
 * ## The question names the act, and never asks "are you sure"
 *
 * "Are you sure?" is a question nobody can answer, because it does not say what
 * about. Each caller passes the sentence that describes what will happen and to
 * how many things — `hints.rs`' rule, applied to a dialog: a refusal or a
 * confirmation is only worth reading when it names the thing.
 *
 * ## Cancel is the default
 *
 * The confirm button is not autofocused. A dialog that opens with the
 * destructive action under the return key is a dialog that turns one stray
 * keypress into two, and the whole point of this component is to require a
 * second, *deliberate* act rather than a second reflex.
 */
defineProps({
  modelValue: { type: Boolean, default: false },
  title: { type: String, default: '' },
  /** What will happen, in the caller's own words. */
  message: { type: String, default: '' },
  /** The verb on the button, so it reads as the act rather than as "OK". */
  confirmText: { type: String, default: '' },
  /** Matches the colour of the button that opened it. */
  color: { type: String, default: 'primary' },
});

const emit = defineEmits(['update:modelValue', 'confirm']);

const { t } = useI18n();

function confirm() {
  emit('update:modelValue', false);
  emit('confirm');
}
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    max-width="440"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <v-card class="pa-4" data-test="confirm-dialog">
      <div class="text-subtitle-1 font-weight-medium mb-2">{{ title }}</div>
      <!-- The sentence, not a warning icon and an ellipsis. It is the only
           thing on this card that carries information. -->
      <div class="text-body-2 text-medium-emphasis">{{ message }}</div>

      <div class="d-flex ga-2 mt-4">
        <v-spacer />
        <v-btn
          size="small"
          variant="text"
          data-test="confirm-no"
          @click="emit('update:modelValue', false)"
        >
          {{ t('app.no') }}
        </v-btn>
        <v-btn size="small" variant="flat" :color="color" data-test="confirm-yes" @click="confirm">
          {{ confirmText || t('app.yes') }}
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>
