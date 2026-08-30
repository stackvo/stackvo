<script setup>
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { usePreferences } from '@/composables/usePreferences';

/**
 * The one screen that introduces the surface, shown once.
 *
 * ## Why this is not a gate
 *
 * There are four already — requirements, catalogue, migration, bootstrap — and
 * every one of them is an **obstacle**: something is missing or unassembled and
 * the app is not usable until it is dealt with. None of them introduces
 * anything, and being stopped four times on the way in is not an introduction.
 *
 * This is the opposite kind of screen and is built to behave like one. It
 * appears **after** the stack is up, it can be left at any point, and leaving
 * it is not a failure state. Herd, ServBay and EnvKit all show a welcome flow;
 * the reason to want one here is stronger than theirs, because the surface is
 * wider: 26 panes and 317 commands is harder to discover than a menu bar icon.
 *
 * ## What is on it, and why these six
 *
 * Not a feature list — a list of the things **nobody would find on their own**.
 * Each of these is written, tested and shipping, and four of the six appear in
 * no user-facing document at all: building the production image, a full
 * environment per git branch, explaining a slow request, importing from seven
 * other tools, the audit record, and exporting a devcontainer. A tour of the
 * dashboard would introduce the part that introduces itself.
 *
 * Each card goes somewhere. A tour that only describes is a tour people close;
 * the button is what makes it worth a screen rather than a paragraph in a
 * README nobody opened.
 *
 * ## Once, and recoverable
 *
 * Kept in preferences rather than in a session flag: "once" has to survive a
 * restart or it is not once. And it is re-openable from Settings, because a
 * one-shot screen somebody skipped on their first minute is a screen they can
 * never get back — which is the failure mode of every welcome flow that has one
 * chance to land.
 */
const { t } = useI18n();
const router = useRouter();
const { set: setPref } = usePreferences();

const emit = defineEmits(['done']);

const step = ref(0);

/**
 * The six, in the order somebody meets them: what to do with a project first,
 * then what this does that nothing else in the category does.
 */
const CARDS = [
  { key: 'import', icon: 'mdi-import', to: { name: 'Projects' } },
  { key: 'branch', icon: 'mdi-source-branch', to: { name: 'Projects' } },
  { key: 'explain', icon: 'mdi-magnify-scan', to: { name: 'Projects' } },
  { key: 'release', icon: 'mdi-package-variant-closed', to: { name: 'Projects' } },
  { key: 'devcontainer', icon: 'mdi-microsoft-visual-studio-code', to: { name: 'Projects' } },
  {
    key: 'audit',
    icon: 'mdi-clipboard-text-clock',
    to: { name: 'Settings', query: { tab: 'audit' } },
  },
];

const card = computed(() => CARDS[step.value]);
const last = computed(() => step.value === CARDS.length - 1);

/** Remember it happened, then get out of the way. */
async function finish() {
  await setPref({ tourSeen: true }).catch(() => {});
  emit('done');
}

async function go() {
  const target = card.value.to;
  await finish();
  router.push(target).catch(() => {});
}
</script>

<template>
  <v-container class="fill-height justify-center">
    <v-card class="pa-6 tour" variant="flat" max-width="640">
      <div class="d-flex align-center mb-4">
        <v-icon :icon="card.icon" size="32" color="primary" class="mr-4" />
        <div>
          <div class="text-h6">{{ t(`tour.${card.key}.title`) }}</div>
          <div class="text-caption text-medium-emphasis">
            {{ t('tour.step', { n: step + 1, of: CARDS.length }) }}
          </div>
        </div>
      </div>

      <p class="text-body-2 mb-6">{{ t(`tour.${card.key}.body`) }}</p>

      <div class="d-flex align-center ga-2">
        <!-- Leaving is a first-class action, not a link in the corner: a tour
             somebody cannot obviously escape is one they resent. -->
        <v-btn variant="text" size="small" @click="finish">{{ t('tour.skip') }}</v-btn>
        <v-spacer />
        <v-btn v-if="step > 0" variant="text" size="small" @click="step -= 1">
          {{ t('tour.back') }}
        </v-btn>
        <v-btn variant="tonal" size="small" @click="go">{{ t('tour.show') }}</v-btn>
        <v-btn v-if="!last" color="primary" variant="flat" size="small" @click="step += 1">
          {{ t('tour.next') }}
        </v-btn>
        <v-btn v-else color="primary" variant="flat" size="small" @click="finish">
          {{ t('tour.done') }}
        </v-btn>
      </div>
    </v-card>
  </v-container>
</template>

<style scoped>
.tour {
  width: 100%;
}
</style>
