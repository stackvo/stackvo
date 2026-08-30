<script setup>
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { usePreferences, appChoice } from '@/composables/usePreferences';
import SettingsGroup from '@/components/SettingsGroup.vue';

/**
 * The app's own preferences: which editor, terminal and browser to launch, what
 * closing the window does, and whether StackVo starts with the machine.
 *
 * Eleventh pane out of `Settings.vue` in the pane split, and the last that owns
 * state. These are facts about this installation rather than about the stack,
 * so they live in `preferences.json` and never touch the `.env` editor.
 */
const { t } = useI18n();

const { prefs, autostart, load, set: setPref, toggleAutostart } = usePreferences();

/** The installed applications the pickers offer. */
const apps = ref({ terminals: [], editors: [], browsers: [] });

/**
 * The id every list ends with. `apps::CUSTOM`, and the reason it is repeated
 * here rather than read from the catalogue is that the box below has to appear
 * for a *choice*, before anything has been launched.
 */
const CUSTOM = 'custom';

/**
 * Is this picker on `Other…`?
 *
 * `appChoice` and not the raw preference, because the fallback entry is what
 * the picker is showing — and while `Other…` is never the fallback (it is
 * appended after the default is marked, and `open_in_editor` skips it), reading
 * anything else here would mean the box could disagree with the select above it.
 */
const isCustom = (stored, list) => appChoice(stored, list) === CUSTOM;

const appItemProps = (a) => ({
  prependIcon: a.icon,
  disabled: !a.available,
  subtitle: a.default ? t('settings.appDefault') : undefined,
});

onMounted(async () => {
  await load();
  // Every key the pickers read has to be present in the fallback too — a
  // missing `browsers` leaves that select bound to undefined instead of empty.
  apps.value = await api
    .appsAvailable()
    .catch(() => ({ terminals: [], editors: [], browsers: [] }));
});
</script>

<template>
  <SettingsGroup
    help="settings-preferences-external-apps"
    icon="mdi-application-cog-outline"
    :title="t('settings.externalApps')"
    :description="t('settings.externalAppsDesc')"
  >
    <div class="d-flex flex-column ga-3">
      <!-- Detected rather than typed. The old free-text box asked the
             user to know the launcher name; what is actually installed
             is something the app can find out. Missing apps stay in the
             list but disabled — omitting them would read as lack of
             support. -->
      <v-select
        :model-value="appChoice(prefs?.terminalApp, apps.terminals)"
        :items="apps.terminals"
        item-title="name"
        item-value="id"
        :item-props="appItemProps"
        :label="t('settings.terminalApp')"
        :hint="t('settings.appsHint')"
        persistent-hint
        clearable
        @update:model-value="(v) => setPref({ terminalApp: v || null })"
      />
      <!-- Detection is better than a free-text box; a free-text box is
             better than nothing, which is what somebody running Helix,
             Neovim or one of the eight unlisted JetBrains IDEs had. It
             appears only for the choice it belongs to, so the pane does
             not grow four command lines nobody asked for. -->
      <v-text-field
        v-if="isCustom(prefs?.terminalApp, apps.terminals)"
        class="ms-6"
        density="compact"
        :model-value="prefs?.terminalCustom ?? ''"
        :label="t('settings.appCustomTerminal')"
        :hint="t('settings.appCustomHint')"
        persistent-hint
        clearable
        @update:model-value="(v) => setPref({ terminalCustom: v || null })"
      />
      <v-select
        :model-value="appChoice(prefs?.editorCommand, apps.editors)"
        :items="apps.editors"
        item-title="name"
        item-value="id"
        :item-props="appItemProps"
        :label="t('settings.editorApp')"
        clearable
        @update:model-value="(v) => setPref({ editorCommand: v || null })"
      />
      <v-text-field
        v-if="isCustom(prefs?.editorCommand, apps.editors)"
        class="ms-6"
        density="compact"
        :model-value="prefs?.editorCustom ?? ''"
        :label="t('settings.appCustomEditor')"
        :hint="t('settings.appCustomHint')"
        persistent-hint
        clearable
        @update:model-value="(v) => setPref({ editorCustom: v || null })"
      />
      <!-- Every "visit" button in the app goes through this. Cleared
             means the system default, which is why the list carries an
             explicit entry for it rather than only an empty state. -->
      <v-select
        :model-value="appChoice(prefs?.browserCommand, apps.browsers)"
        :items="apps.browsers"
        item-title="name"
        item-value="id"
        :item-props="appItemProps"
        :label="t('settings.browserApp')"
        :hint="t('settings.browserAppHint')"
        persistent-hint
        clearable
        @update:model-value="(v) => setPref({ browserCommand: v || null })"
      />
      <v-text-field
        v-if="isCustom(prefs?.browserCommand, apps.browsers)"
        class="ms-6"
        density="compact"
        :model-value="prefs?.browserCustom ?? ''"
        :label="t('settings.appCustomBrowser')"
        :hint="t('settings.appCustomHint')"
        persistent-hint
        clearable
        @update:model-value="(v) => setPref({ browserCustom: v || null })"
      />
      <!-- No select beside it: the database client is chosen per service, in
             the menu on the endpoint, because which clients are offered depends
             on the scheme that service speaks. Its `Other…` row is there too,
             and this is the only place its command can live. -->
      <v-text-field
        density="compact"
        :model-value="prefs?.dbClientCustom ?? ''"
        :label="t('settings.appCustomDbClient')"
        :hint="t('settings.appCustomDbHint')"
        persistent-hint
        clearable
        @update:model-value="(v) => setPref({ dbClientCustom: v || null })"
      />
    </div>
  </SettingsGroup>

  <!-- Scheduled database snapshots. A preference rather than a stack setting:
       it is about what this installation does unattended, and it applies to
       whichever workspace is open. -->
  <SettingsGroup
    help="settings-preferences-backups"
    icon="mdi-database-clock-outline"
    :title="t('settings.backups')"
    :description="t('settings.backupsDesc')"
  >
    <v-select
      :model-value="prefs?.backupSchedule ?? 'off'"
      :items="[
        { value: 'off', title: t('settings.backupOff') },
        { value: 'hourly', title: t('settings.backupHourly') },
        { value: 'daily', title: t('settings.backupDaily') },
        { value: 'weekly', title: t('settings.backupWeekly') },
      ]"
      :label="t('settings.backupSchedule')"
      :hint="t('settings.backupScheduleHint')"
      persistent-hint
      @update:model-value="(v) => setPref({ backupSchedule: v })"
    />
    <v-text-field
      class="mt-3"
      type="number"
      min="1"
      max="100"
      :model-value="prefs?.backupKeep ?? 7"
      :label="t('settings.backupKeep')"
      :hint="t('settings.backupKeepHint')"
      persistent-hint
      @update:model-value="
        (v) => setPref({ backupKeep: Math.min(100, Math.max(1, Number(v) || 1)) })
      "
    />
  </SettingsGroup>

  <SettingsGroup
    help="settings-preferences-startup"
    icon="mdi-power"
    :title="t('settings.startup')"
    :description="t('settings.startupDesc')"
  >
    <v-switch
      :model-value="autostart"
      :label="t('settings.autostart')"
      color="primary"
      hide-details
      @update:model-value="toggleAutostart"
    />
    <v-switch
      :model-value="prefs?.startMinimized ?? false"
      :label="t('settings.startMinimized')"
      color="primary"
      hide-details
      @update:model-value="(v) => setPref({ startMinimized: v })"
    />

    <v-divider class="my-3" />

    <div class="text-body-2">{{ t('close.behaviour') }}</div>
    <div class="text-caption text-medium-emphasis">{{ t('close.behaviourHint') }}</div>
    <v-radio-group
      :model-value="prefs?.closeBehaviour ?? 'ask'"
      hide-details
      @update:model-value="(v) => setPref({ closeBehaviour: v })"
    >
      <v-radio value="ask" :label="t('close.ask')" />
      <v-radio value="tray" :label="t('close.tray')" />
      <v-radio value="quit" :label="t('close.quit')" />
      <v-radio value="stopAndQuit" :label="t('close.stopAndQuit')" />
    </v-radio-group>

    <v-divider class="my-3" />

    <!-- A one-shot screen somebody skipped in their first minute is one they
         can never get back, which is the failure mode of every welcome flow
         that has a single chance to land. This is the way back. -->
    <div class="d-flex align-center ga-3">
      <div class="flex-grow-1">
        <div class="text-body-2">{{ t('settings.showTour') }}</div>
        <div class="text-caption text-medium-emphasis">{{ t('settings.showTourHint') }}</div>
      </div>
      <v-btn size="small" variant="tonal" @click="setPref({ tourSeen: false })">
        {{ t('settings.showTourAction') }}
      </v-btn>
    </div>
  </SettingsGroup>

  <!-- ---- stack ----------------------------------------------------- -->
  <!-- These shape every generated file. They live in the binary as
       defaults, so a fresh .env has none of them; changing one here
       writes the key, which is what makes a line in that file mean
       something. -->
</template>
