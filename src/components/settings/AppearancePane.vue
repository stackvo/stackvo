<script setup>
import { computed, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTheme } from 'vuetify';
import { useAppearanceStore } from '@/stores/appearance';
import {
  AUDIT_PAIRS,
  DEFAULT_APPEARANCE,
  PRIMARY_SWATCHES,
  HARMONIES,
  NEUTRALS,
  FONT_FAMILIES,
  RANGES,
  STATUS_PALETTES,
  auditTheme,
  buildTheme,
  parseAppearance,
  themeSnippet,
} from '@/lib/appearance';
import SettingsGroup from '@/components/SettingsGroup.vue';
import { ramp } from '@/lib/tones';
import { useCopyTick } from '@/composables/useCopyTick';
import { toastError, toastSuccess } from '@/lib/toast';

/**
 * How the app looks: theme, accent, density, font and the saved presets.
 *
 * Sixth pane out of `Settings.vue` in the pane split and the cleanest seam so far —
 * it is the only one that touches neither the `.env` editor nor the operation
 * console. Everything it changes lives in `useAppearanceStore`, which persists
 * and applies on its own, so the pane is markup over a store and nothing else.
 */
const { t } = useI18n();
const appearance = useAppearanceStore();
const theme = useTheme();

const isDark = computed(() => theme.global.current.value.dark);

/** Name being typed for a new preset. Empty disables the save button. */
const presetName = ref('');

async function savePreset() {
  await appearance.savePreset(presetName.value);
  presetName.value = '';
}

const { copied, copy } = useCopyTick();

/** What is being pasted in. Empty disables the import button. */
const importText = ref('');

/**
 * A look, as the two things somebody might want to do with one.
 *
 * The settings object is for another copy of this application — it is exactly
 * what a preset holds, so it round-trips through `parseAppearance` with nothing
 * lost. The snippet is for a *different* application: a `createVuetify` call
 * carrying both themes, which is the thing Vuetify Studio exists to produce and
 * the one part of it this pane could not answer.
 */
const lookJson = computed(() => JSON.stringify(appearance.value, null, 2));

const lookSnippet = computed(() => themeSnippet(appearance.value, appearance.systemAccent));

/**
 * Applied, not just parsed.
 *
 * A partial import is reported rather than swallowed: a look from a newer
 * release can carry a field this build has no rule for, and fifteen good
 * fields are worth taking — but somebody who pasted sixteen and got fifteen
 * should be told which one did not survive.
 */
async function importLook() {
  const { values, ignored } = parseAppearance(importText.value);

  if (!values) {
    toastError(t('settings.importFailed'));
    return;
  }

  await appearance.set(values);
  importText.value = '';

  if (ignored.length) toastSuccess(t('settings.importPartial', { fields: ignored.join(', ') }));
  else toastSuccess(t('settings.importDone'));
}

const statusItems = computed(() =>
  STATUS_PALETTES.map((p) => ({ value: p.id, title: t(`settings.statusPalettes.${p.id}`) }))
);

const fontItems = computed(() =>
  FONT_FAMILIES.map((f) => ({ value: f.id, title: t(`settings.fonts.${f.id}`) }))
);

/** Derived from the library rather than transcribed, like the two above it. */
const harmonyItems = computed(() =>
  HARMONIES.map((id) => ({ value: id, title: t(`settings.harmonies.${id}`) }))
);

/**
 * The two themes the preview draws, registered under names nothing else uses.
 *
 * Vuetify's theme registry is a plain reactive map, so a theme that is never
 * made current can still be built and handed to `<v-theme-provider>`. That is
 * the whole trick: the light palette can be put on screen without the dark one
 * being taken off it, which is the question this pane could not answer before
 * — a user on dark chose their light colours blind, and found out what they had
 * picked the next time the sun came up.
 */
const PREVIEW_THEMES = [
  { base: 'light', alias: 'preview-light', label: 'settings.themeLight' },
  { base: 'dark', alias: 'preview-dark', label: 'settings.themeDark' },
];

/**
 * Kept in step with the settings, on an explicit watch rather than
 * `watchEffect`: this writes into the same reactive map `buildTheme` reads its
 * base out of, and an effect that tracks its own writes is a loop waiting for
 * one more key to be added.
 */
watch(
  () => [appearance.value, appearance.systemAccent],
  () => {
    for (const { base, alias } of PREVIEW_THEMES) {
      const built = buildTheme(appearance.value, base, appearance.systemAccent);
      if (built) theme.themes.value[alias] = built;
    }
  },
  { deep: true, immediate: true }
);

/**
 * Taken back out on the way off the page.
 *
 * Vuetify regenerates one stylesheet from *every* registered theme, so a pair
 * left behind is a permanent 50% on the theme CSS of a session that opened
 * settings once.
 */
onUnmounted(() => {
  for (const { alias } of PREVIEW_THEMES) delete theme.themes.value[alias];
});

/**
 * What each preview actually measures.
 *
 * Read out of `computedThemes` rather than the themes map, because half of
 * every pair is an `on-*` colour and those do not exist until Vuetify has
 * derived them — `theme.themes` holds what was written, `computedThemes` holds
 * what will be rendered.
 */
const audits = computed(() =>
  PREVIEW_THEMES.map(({ alias }) =>
    auditTheme(theme.computedThemes.value[alias]?.colors, appearance.value.contrast)
  )
);

/**
 * The tonal ramps behind the two accents.
 *
 * Both are derived — `primary` from the swatch grid, `secondary` from it in
 * turn — so this is the one part of the palette that has a *structure* worth
 * showing rather than a value. Material's shorthand is kept (`P-40`, `S-70`)
 * because it is what somebody reads out to a designer.
 *
 * The neutral families are deliberately absent. Studio derives its neutrals
 * from the accent's hue and can therefore ramp them honestly; this application
 * has five hand-authored families, and drawing an engine-derived ramp beside a
 * palette that does not come from it would be showing a colour the app never
 * renders.
 */
const ramps = computed(() => [
  { code: 'P', label: t('settings.primaryColor'), steps: ramp(currentPrimary.value, 'P') },
  { code: 'S', label: t('settings.harmony'), steps: ramp(currentSecondary.value, 'S') },
]);

/** What is actually on screen, which is the desktop accent when that is on. */
const currentPrimary = computed(
  () => (appearance.value.useSystemAccent && appearance.systemAccent) || appearance.value.primary
);

const currentSecondary = computed(
  () => theme.themes.value['preview-dark']?.colors?.secondary ?? currentPrimary.value
);

/**
 * Dark text on the light half of a ramp, light text on the dark half.
 *
 * Tone *is* perceptual lightness, which is the whole reason this ramp is drawn
 * with Material's engine — so the threshold is the tone number itself, and no
 * second contrast calculation is needed to decide it.
 */
const stepInk = (tone) => (tone >= 50 ? '#000' : '#fff');

/** `4.53:1`, or an em dash for a pair that could not be read at all. */
const ratioText = (ratio) =>
  ratio === null || ratio === undefined ? '—' : `${ratio.toFixed(2)}:1`;

/**
 * AA and AAA are both passes and are both drawn as one, in the status palette
 * the user chose — which is the palette every other "this is fine" in the
 * application is drawn in. The grade itself is the label, so the distinction
 * is read rather than decoded from a colour.
 */
const gradeColour = (grade) => (grade === 'fail' ? 'error' : 'success');

const gradeLabel = (grade) =>
  grade === 'fail' ? t('settings.contrastAuditFail') : grade?.toUpperCase();

/** Shown next to the reset button so "back to defaults" is not a leap of faith. */
const isDefaultAppearance = computed(() =>
  Object.keys(DEFAULT_APPEARANCE).every((k) => appearance.value[k] === DEFAULT_APPEARANCE[k])
);
</script>

<template>
  <SettingsGroup
    help="settings-appearance-theme-colors"
    icon="mdi-palette"
    :title="t('settings.themeColors')"
    :description="t('settings.themeColorsDesc')"
  >
    <template #append>
      <v-btn
        size="small"
        variant="text"
        prepend-icon="mdi-backup-restore"
        :disabled="isDefaultAppearance"
        @click="appearance.reset()"
      >
        {{ t('settings.resetAppearance') }}
      </v-btn>
    </template>

    <div class="field-label">{{ t('settings.theme') }}</div>
    <!-- Three buttons rather than a dropdown: the choice is small,
           fixed and worth showing all of at once. `system` is Vuetify's
           own theme name, so it tracks prefers-color-scheme live rather
           than being read once at launch. -->
    <v-btn-toggle
      :model-value="appearance.value.theme"
      mandatory
      divided
      color="primary"
      variant="flat"
      class="mb-5 bg-surface-light"
      @update:model-value="(v) => appearance.set({ theme: v })"
    >
      <v-btn value="system" size="small" prepend-icon="mdi-theme-light-dark">
        {{ t('settings.themeSystem') }}
      </v-btn>
      <v-btn value="light" size="small" prepend-icon="mdi-white-balance-sunny">
        {{ t('settings.themeLight') }}
      </v-btn>
      <v-btn value="dark" size="small" prepend-icon="mdi-weather-night">
        {{ t('settings.themeDark') }}
      </v-btn>
    </v-btn-toggle>

    <div class="d-flex align-center ga-2 mb-1">
      <div class="field-label mb-0">{{ t('settings.primaryColor') }}</div>
      <v-spacer />
      <!-- Offered only where it can be answered: on Linux there is no
             one accent colour to read. -->
      <v-switch
        v-if="appearance.systemAccent"
        :model-value="appearance.value.useSystemAccent"
        :label="t('settings.systemAccent')"
        color="primary"
        hide-details
        class="flex-grow-0"
        @update:model-value="(v) => appearance.set({ useSystemAccent: v })"
      />
    </div>
    <div class="swatches mb-5" :class="{ 'is-disabled': appearance.value.useSystemAccent }">
      <button
        v-for="c in PRIMARY_SWATCHES"
        :key="c"
        type="button"
        class="swatch"
        :class="{ 'is-active': appearance.value.primary === c }"
        :style="{ background: c }"
        :title="c"
        :aria-label="c"
        :disabled="appearance.value.useSystemAccent"
        @click="appearance.set({ primary: c })"
      >
        <v-icon v-if="appearance.value.primary === c" size="15" color="white"> mdi-check </v-icon>
      </button>
    </div>

    <!-- Under the accent because it is a function of it: `secondary` is
           the accent's hue turned by a fixed angle, at the same measured
           lightness. Before this it was a constant nothing ever moved. -->
    <v-select
      :model-value="appearance.value.harmony"
      :items="harmonyItems"
      :label="t('settings.harmony')"
      :hint="t('settings.harmonyHint')"
      persistent-hint
      class="mb-5"
      @update:model-value="(v) => appearance.set({ harmony: v })"
    />

    <div class="field-label">{{ t('settings.neutralPalette') }}</div>
    <div class="swatches mb-5">
      <button
        v-for="n in NEUTRALS"
        :key="n.id"
        type="button"
        class="swatch swatch--neutral"
        :class="{ 'is-active': appearance.value.neutral === n.id }"
        :style="{ background: isDark ? n.dark.surface : n.light['surface-variant'] }"
        :title="t(`settings.neutrals.${n.id}`)"
        :aria-label="t(`settings.neutrals.${n.id}`)"
        @click="appearance.set({ neutral: n.id })"
      >
        <v-icon v-if="appearance.value.neutral === n.id" size="15">mdi-check</v-icon>
      </button>
    </div>

    <div class="field-label">
      {{ t('settings.radius', { px: appearance.value.radius }) }}
    </div>
    <!-- Previewed while dragging, written when the handle is let go:
           a slider emits on every pixel, and preferences.json is a file
           on disk. -->
    <!-- The label above is a plain `div`, so nothing connects it to the
           control: a screen reader announced "slider, 12" with no indication of
           what was at 12. Found by the axe pass once this pane could be
           mounted. -->
    <v-slider
      :aria-label="t('settings.radius', { px: appearance.value.radius })"
      :model-value="appearance.value.radius"
      :min="RANGES.radius.min"
      :max="RANGES.radius.max"
      :step="1"
      hide-details
      @update:model-value="(v) => appearance.preview({ radius: v })"
      @end="appearance.commit()"
    />
  </SettingsGroup>

  <SettingsGroup
    help="settings-appearance-typography"
    icon="mdi-format-font"
    :title="t('settings.typography')"
    :description="t('settings.typographyDesc')"
  >
    <v-select
      :model-value="appearance.value.fontFamily"
      :items="fontItems"
      :label="t('settings.fontFamily')"
      :hint="t('settings.fontFamilyHint')"
      persistent-hint
      class="mb-5"
      @update:model-value="(v) => appearance.set({ fontFamily: v })"
    />

    <div class="field-label">{{ t('settings.density') }}</div>
    <!-- One knob for the whole app: every `density` prop written on a
           component was removed, because a prop outranks a default and
           would have made this setting a no-op wherever one existed. -->
    <v-btn-toggle
      :model-value="appearance.value.density"
      mandatory
      divided
      color="primary"
      variant="flat"
      class="mb-5 bg-surface-light"
      @update:model-value="(v) => appearance.set({ density: v })"
    >
      <v-btn value="compact" size="small">{{ t('settings.densityCompact') }}</v-btn>
      <v-btn value="comfortable" size="small">
        {{ t('settings.densityComfortable') }}
      </v-btn>
      <v-btn value="default" size="small">{{ t('settings.densitySpacious') }}</v-btn>
    </v-btn-toggle>

    <div class="field-label">
      {{ t('settings.uiScale', { px: appearance.value.fontSize }) }}
    </div>
    <!-- Vuetify's type scale is in rem throughout, so the root size
           scales every label, table row and dialog with it — this is a
           UI scale control, not just a font size. -->
    <v-slider
      :aria-label="t('settings.uiScale', { px: appearance.value.fontSize })"
      :model-value="appearance.value.fontSize"
      :min="RANGES.fontSize.min"
      :max="RANGES.fontSize.max"
      :step="1"
      hide-details
      class="mb-4"
      @update:model-value="(v) => appearance.preview({ fontSize: v })"
      @end="appearance.commit()"
    />

    <div class="field-label">{{ t('settings.contrast') }}</div>
    <!-- Three stops rather than a switch. The gap the switch hid is large:
           standard is WCAG AA and high is AAA, so a reader who needed some
           help had to take all of it, heavier dividers included. -->
    <v-btn-toggle
      :model-value="appearance.value.contrast"
      mandatory
      divided
      color="primary"
      variant="flat"
      class="bg-surface-light"
      @update:model-value="(v) => appearance.set({ contrast: v })"
    >
      <v-btn value="standard" size="small">{{ t('settings.contrastStandard') }}</v-btn>
      <v-btn value="medium" size="small">{{ t('settings.contrastMedium') }}</v-btn>
      <v-btn value="high" size="small">{{ t('settings.contrastHigh') }}</v-btn>
    </v-btn-toggle>
    <div class="text-caption text-medium-emphasis mt-2">
      {{ t('settings.contrastHint') }}
    </div>

    <v-switch
      :model-value="appearance.value.reduceMotion"
      :label="t('settings.reduceMotion')"
      color="primary"
      hide-details
      class="mt-2"
      @update:model-value="(v) => appearance.set({ reduceMotion: v })"
    />
    <div class="text-caption text-medium-emphasis">
      {{ t('settings.reduceMotionHint') }}
    </div>
  </SettingsGroup>

  <!-- The one palette in the app that is not decoration: these four
         colours are how a container reports what it is doing. -->
  <SettingsGroup
    help="settings-appearance-status-colors"
    icon="mdi-traffic-light-outline"
    :title="t('settings.statusColors')"
    :description="t('settings.statusColorsDesc')"
  >
    <v-select
      :model-value="appearance.value.statusPalette"
      :items="statusItems"
      :label="t('settings.statusPalette')"
      class="mb-3"
      @update:model-value="(v) => appearance.set({ statusPalette: v })"
    />

    <!-- Shown, not described: whether two states are tellable apart is
           a question about your eyes, not about the palette's name. -->
    <div class="d-flex ga-2 flex-wrap">
      <v-chip size="small" color="success" prepend-icon="mdi-check-circle">
        {{ t('system.running') }}
      </v-chip>
      <v-chip size="small" color="error" prepend-icon="mdi-alert-circle">
        {{ t('system.stopped') }}
      </v-chip>
      <v-chip size="small" color="warning" prepend-icon="mdi-alert">
        {{ t('settings.generatorDiffers') }}
      </v-chip>
      <v-chip size="small" color="info" prepend-icon="mdi-information">
        {{ t('settings.about') }}
      </v-chip>
    </div>

    <v-switch
      :model-value="appearance.value.darkConsoles"
      :label="t('settings.darkConsoles')"
      color="primary"
      hide-details
      class="mt-3"
      @update:model-value="(v) => appearance.set({ darkConsoles: v })"
    />
    <div class="text-caption text-medium-emphasis">
      {{ t('settings.darkConsolesHint') }}
    </div>
  </SettingsGroup>

  <!-- Vuetify Studio previews the theme it is building and never says what
       it measures; this application measures everything it derives and never
       showed any of it. Both halves, in one group. -->
  <SettingsGroup
    help="settings-appearance-preview"
    icon="mdi-format-color-highlight"
    :title="t('settings.preview')"
    :description="t('settings.previewDesc')"
  >
    <!-- `inert` rather than `aria-hidden`: this is a picture of the interface,
         and its buttons are focusable. Everything it shows is also in the
         table below it, in words, so nothing is lost by taking it out of the
         tab order and the accessibility tree — and a screen reader that walked
         into two duplicate button labels would be worse than silence. -->
    <div class="theme-preview ga-3" inert>
      <v-theme-provider
        v-for="p in PREVIEW_THEMES"
        :key="p.alias"
        :theme="p.alias"
        with-background
        class="d-flex flex-column ga-2 pa-3 rounded-lg border"
      >
        <div class="text-caption text-medium-emphasis">{{ t(p.label) }}</div>
        <div class="text-body-2">{{ t('settings.previewBody') }}</div>
        <div class="text-caption text-medium-emphasis">{{ t('settings.previewCaption') }}</div>

        <div class="d-flex ga-2 flex-wrap">
          <v-btn size="small" color="primary" variant="flat">
            {{ t('settings.previewPrimary') }}
          </v-btn>
          <!-- The colour that had no source until it was derived. A checkbox
               anywhere in the app is drawn in it, and so is this. -->
          <v-btn size="small" color="secondary" variant="flat">
            {{ t('settings.previewSecondary') }}
          </v-btn>
        </div>

        <div class="d-flex ga-1 flex-wrap">
          <v-chip size="x-small" color="success">{{ t('system.running') }}</v-chip>
          <v-chip size="x-small" color="error">{{ t('system.stopped') }}</v-chip>
          <v-chip size="x-small" color="warning">{{ t('settings.generatorDiffers') }}</v-chip>
        </div>
      </v-theme-provider>
    </div>

    <!-- Material's own ramp, drawn with Material's own engine. Everything
         else here derives colour with arithmetic this repository owns; a ramp
         cannot be faked that way, because its whole claim is that the steps are
         perceptually even. See `lib/tones.js` for why the 42 KB that costs is
         allowed to exist only inside this chunk. -->
    <div class="field-label mt-4">{{ t('settings.tones') }}</div>
    <div v-for="r in ramps" :key="r.code" class="theme-ramp rounded-lg mb-2" :aria-label="r.label">
      <div
        v-for="step in r.steps"
        :key="step.code"
        class="theme-ramp-step"
        :style="{ background: step.hex, color: stepInk(step.tone) }"
        :title="`${step.code} · ${step.hex}`"
      >
        {{ step.tone }}
      </div>
    </div>
    <div class="text-caption text-medium-emphasis mb-2">{{ t('settings.tonesHint') }}</div>

    <!-- The half Studio does not have. Every ratio here is the one the app
         will actually render, including the composite in the `caption` row —
         which is the pair that has failed WCAG twice in this codebase and is
         invisible to anyone reading the palette. -->
    <div class="d-flex flex-column mt-4">
      <div class="theme-audit-row theme-audit-head ga-2">
        <span>{{ t('settings.contrastAuditPair') }}</span>
        <span v-for="p in PREVIEW_THEMES" :key="p.alias">{{ t(p.label) }}</span>
      </div>
      <div v-for="(pair, i) in AUDIT_PAIRS" :key="pair.id" class="theme-audit-row ga-2">
        <span class="text-medium-emphasis">{{ t(`settings.contrastAudit.${pair.id}`) }}</span>
        <span v-for="(rows, n) in audits" :key="n" class="theme-audit-cell">
          <span class="theme-audit-ratio">{{ ratioText(rows[i].ratio) }}</span>
          <v-chip v-if="rows[i].grade" size="x-small" label :color="gradeColour(rows[i].grade)">
            {{ gradeLabel(rows[i].grade) }}
          </v-chip>
        </span>
      </div>
    </div>
  </SettingsGroup>

  <SettingsGroup
    help="settings-appearance-presets"
    icon="mdi-bookmark-multiple-outline"
    :title="t('settings.presets')"
    :description="t('settings.presetsDesc')"
  >
    <div class="d-flex ga-2 align-start">
      <v-text-field
        v-model="presetName"
        :label="t('settings.presetName')"
        hide-details
        @keyup.enter="savePreset"
      />
      <v-btn color="primary" variant="flat" :disabled="!presetName.trim()" @click="savePreset">
        {{ t('settings.savePreset') }}
      </v-btn>
    </div>

    <div v-if="appearance.presets.length" class="d-flex ga-2 flex-wrap mt-3">
      <v-chip
        v-for="p in appearance.presets"
        :key="p.name"
        closable
        prepend-icon="mdi-palette-swatch"
        @click="appearance.applyPreset(p.name)"
        @click:close="appearance.deletePreset(p.name)"
      >
        {{ p.name }}
      </v-chip>
    </div>
    <div v-else class="text-caption text-medium-emphasis mt-3">
      {{ t('settings.noPresets') }}
    </div>

    <v-divider class="my-4" />

    <!-- A look that cannot leave the machine is a look one person has. Two
         destinations, because they are two different questions: the settings
         object goes to another copy of this app and round-trips exactly; the
         snippet goes to a project that is not this app at all, which is the
         thing Vuetify Studio is for. -->
    <div class="field-label">{{ t('settings.share') }}</div>
    <div class="d-flex ga-2 flex-wrap mb-4">
      <v-btn
        size="small"
        variant="tonal"
        :prepend-icon="copied === 'look-json' ? 'mdi-check' : 'mdi-content-copy'"
        @click="copy(lookJson, 'look-json')"
      >
        {{ t('settings.copyLook') }}
      </v-btn>
      <v-btn
        size="small"
        variant="tonal"
        :prepend-icon="copied === 'look-snippet' ? 'mdi-check' : 'mdi-code-braces'"
        @click="copy(lookSnippet, 'look-snippet')"
      >
        {{ t('settings.copySnippet') }}
      </v-btn>
    </div>

    <!-- Paste rather than a file picker: writing a file needs an IPC command
         and a Rust handler, and the clipboard is how a look actually travels
         between two people anyway. -->
    <div class="d-flex ga-2 align-start">
      <v-textarea
        v-model="importText"
        :label="t('settings.importLook')"
        :hint="t('settings.importLookHint')"
        persistent-hint
        rows="2"
        auto-grow
        max-rows="6"
      />
      <v-btn color="primary" variant="flat" :disabled="!importText.trim()" @click="importLook">
        {{ t('settings.importAction') }}
      </v-btn>
    </div>
  </SettingsGroup>

  <!-- ---- localisation ---------------------------------------------- -->
</template>
