<script setup>
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAppearanceStore } from '@/stores/appearance';
import { i18n, loadLocalePacks, setLocale } from '@/i18n';
import { api } from '@/lib/ipc';
import SettingsGroup from '@/components/SettingsGroup.vue';

/**
 * Language, and the two settings that are about language without being it.
 *
 * Eighth pane out of `Settings.vue` under §14.16 and the smallest. Three
 * controls, three different owners, which is the whole reason it is worth
 * mounting: the app locale goes through `setLocale` because it also persists
 * and relabels the tray; the console locale and the RTL flag are appearance
 * state and go straight to the store.
 */
const { t, locale } = useI18n();
const appearance = useAppearanceStore();

/**
 * Language packs (M-7).
 *
 * The two built-in languages are compiled in; anything else is a JSON file in
 * the app's config directory, and this is where somebody finds out that it is
 * there, how much of it is translated, and how to start one.
 *
 * The share is worked out here rather than in Rust because this is the side
 * that holds the English catalogue — Rust can count a pack's strings but has
 * nothing to compare them against without a second copy of every string in the
 * app, which is exactly the duplication the tray catalogue exists to undo.
 */
const packs = ref([]);
const newTag = ref('');
const busy = ref(false);
const packError = ref(null);

/**
 * The English catalogue a pack is measured against.
 *
 * `$vuetify` is dropped, exactly as `startPack` drops it when it seeds a file:
 * those are the library's own labels, they ship translated for dozens of
 * languages already, and a pack has no reason to carry them. Leaving them in
 * put 89 strings in the denominator that no pack could ever fill, so every
 * finished translation would have stopped short of 100%.
 */
const english = computed(() => {
  const { $vuetify, ...rest } = i18n.global.getLocaleMessage('en');
  void $vuetify;
  return rest;
});

const englishStrings = computed(() => count(english.value));

function count(value) {
  if (typeof value === 'string') return 1;
  if (value && typeof value === 'object')
    return Object.values(value).reduce((n, v) => n + count(v), 0);
  return 0;
}

/**
 * How much of a pack is actually **translated**.
 *
 * Counting the strings a pack *holds* was the obvious arithmetic and it was
 * wrong in the way that matters: `startPack` seeds the file with every English
 * string — which is what a translation file is, and is the right thing to hand
 * a translator — so an untouched pack reported `2000 of 2000 (100%)` the moment
 * it was created. A progress figure that is full before the work begins, on a
 * language that is entirely English.
 *
 * `locale.rs` states the rule that broke, in its own doc comment: *a missing
 * string that falls back to English is honest; a fabricated one is a sentence
 * somebody has to find and disbelieve.* Two thousand of them, with a number
 * saying the job was done.
 *
 * A leaf counts when it **differs from the English one**, which is how every
 * translation tool decides the same question. It handles both shapes of pack at
 * once — a seeded file full of English and a sparse file missing keys both fall
 * back to English at runtime, and both read as untranslated here.
 *
 * It understates: `Docker`, `PHP` and `OK` are the same word in most languages
 * and are counted as untranslated. That is the safe direction. A translator who
 * sees 98% on a finished pack goes looking for the last few; one who sees 100%
 * on an untouched file learns that the number is a lie.
 *
 * The pack's messages come from vue-i18n rather than from a second read over
 * IPC: `loadLocalePacks` already registered every pack as English with the
 * pack's own strings merged on top, so the merged catalogue is exactly what a
 * reader sees, and comparing it against English answers the question directly.
 */
function translatedCount(messages, source) {
  if (typeof source === 'string') {
    return typeof messages === 'string' && messages !== source ? 1 : 0;
  }
  if (!source || typeof source !== 'object') return 0;
  return Object.entries(source).reduce(
    (n, [key, value]) => n + translatedCount(messages?.[key], value),
    0
  );
}

/**
 * Walked over the English tree, not the pack's.
 *
 * The denominator is the app. A pack carrying keys the app no longer has would
 * otherwise inflate its own score with strings nothing renders.
 */
function progress(pack) {
  const done = translatedCount(i18n.global.getLocaleMessage(pack.tag), english.value);
  const total = englishStrings.value || 1;
  return { done, total, percent: Math.min(100, Math.round((done / total) * 100)) };
}

const options = computed(() => [
  { value: 'tr', title: 'Türkçe' },
  { value: 'en', title: 'English' },
  ...packs.value.filter((p) => !p.broken).map((p) => ({ value: p.tag, title: p.label })),
]);

async function loadPacks() {
  try {
    packs.value = (await api.localePacks()) ?? [];
  } catch {
    packs.value = [];
  }
}

/**
 * Start a pack from the English catalogue.
 *
 * English, not a machine translation of it: a missing string that falls back
 * to English is honest, and a fabricated one is a sentence somebody has to
 * find and disbelieve. What the translator gets is every key with the original
 * beside it, which is what a translation file is.
 */
async function startPack() {
  busy.value = true;
  packError.value = null;
  try {
    const tag = newTag.value.trim();
    const messages = {
      ...i18n.global.getLocaleMessage('en'),
      // The pack names itself: this app cannot hold a label for a language it
      // has never heard of, so the picker reads it out of the file.
      language: { label: tag },
    };
    // `$vuetify` is the library's own catalogue and is not this app's to hand
    // to a translator; it ships in Vuetify for dozens of languages already.
    delete messages.$vuetify;
    await api.localePackWrite(tag, messages);
    newTag.value = '';
    await loadPacks();
    await loadLocalePacks();
  } catch (e) {
    packError.value = e;
  } finally {
    busy.value = false;
  }
}

async function removePack(tag) {
  busy.value = true;
  try {
    await api.localePackDelete(tag);
    if (locale.value === tag) await setLocale('en');
    await loadPacks();
  } catch (e) {
    packError.value = e;
  } finally {
    busy.value = false;
  }
}

onMounted(loadPacks);
</script>

<template>
  <SettingsGroup
    help="settings-localisation-language"
    icon="mdi-web"
    :title="t('settings.language')"
    :description="t('settings.languageDesc')"
  >
    <v-select
      :model-value="locale"
      :items="options"
      :label="t('settings.language')"
      @update:model-value="setLocale"
    />

    <!-- The packs, under the picker they feed. A pack that did not parse is
         listed with its error rather than quietly missing from the list
         above — a hand-edited file with a trailing comma that simply vanishes
         is the worst failure this could have. -->
    <div v-for="pack in packs" :key="pack.tag" class="pack" data-test="locale-pack">
      <v-icon size="16" :color="pack.broken ? 'error' : 'success'" class="mr-2">
        {{ pack.broken ? 'mdi-alert-circle-outline' : 'mdi-translate' }}
      </v-icon>
      <div class="grow">
        <div>
          <span class="font-weight-medium mr-2">{{ pack.label }}</span>
          <!-- A language that reads the other way says so here. Otherwise the
               only way to find out whether the pack's declaration took effect
               is to switch to it and look at the window. -->
          <v-chip v-if="pack.direction === 'rtl'" size="x-small" variant="tonal" class="mr-2">
            {{ t('settings.packRtl') }}
          </v-chip>
          <span class="text-caption text-medium-emphasis">
            {{ pack.broken ? pack.broken : t('settings.packProgress', progress(pack)) }}
          </span>
        </div>
        <!-- The file. "Drop a JSON file in the config directory" is only a
             mechanism somebody can use if they can find the file the button
             just made. -->
        <div class="text-caption text-disabled path">{{ pack.path }}</div>
      </div>
      <v-spacer />
      <v-btn size="x-small" variant="text" :loading="busy" @click="removePack(pack.tag)">
        {{ t('settings.packRemove') }}
      </v-btn>
    </div>

    <div class="d-flex ga-2 align-center mt-2">
      <v-text-field
        v-model="newTag"
        :label="t('settings.packTag')"
        :hint="t('settings.packHint')"
        persistent-hint
        density="compact"
        variant="outlined"
        style="max-width: 220px"
      />
      <v-btn
        size="small"
        variant="tonal"
        :disabled="!newTag.trim()"
        :loading="busy"
        @click="startPack"
      >
        {{ t('settings.packStart') }}
      </v-btn>
    </div>
    <div v-if="packError" class="text-caption text-error mt-2">{{ packError.message }}</div>
  </SettingsGroup>

  <SettingsGroup
    help="settings-localisation-console-language"
    icon="mdi-console"
    :title="t('settings.consoleLanguage')"
    :description="t('settings.consoleLanguageDesc')"
  >
    <v-select
      :model-value="appearance.value.consoleLocale"
      :items="[
        { value: 'app', title: t('settings.consoleFollowsApp') },
        { value: 'tr', title: 'Türkçe' },
        { value: 'en', title: 'English' },
      ]"
      :label="t('settings.consoleLanguage')"
      :hint="t('settings.consoleLanguageHint')"
      persistent-hint
      @update:model-value="(v) => appearance.set({ consoleLocale: v })"
    />
  </SettingsGroup>

  <SettingsGroup
    help="settings-localisation-direction"
    icon="mdi-format-textdirection-r-to-l"
    :title="t('settings.direction')"
    :description="t('settings.directionDesc')"
  >
    <v-switch
      :model-value="appearance.value.rtl"
      :label="t('settings.rtl')"
      color="primary"
      hide-details
      @update:model-value="(v) => appearance.set({ rtl: v })"
    />
    <div class="text-caption text-medium-emphasis">{{ t('settings.rtlHint') }}</div>
  </SettingsGroup>

  <!-- ---- workspace ------------------------------------------------ -->

  <!-- ---- preferences ---------------------------------------------- -->
</template>

<style scoped>
.grow {
  min-width: 0;
}

.path {
  overflow-wrap: anywhere;
}

.pack {
  display: flex;
  align-items: center;
  padding: 6px 0;
}
</style>
