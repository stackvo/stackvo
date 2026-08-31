<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * F-1 — the MCP server that lives inside the container.
 *
 * ## What this card is for
 *
 * `php artisan boost:install` writes a `.mcp.json` into the project holding
 * `php artisan boost:mcp`, which assumes a php on the host. There is none, so
 * Laravel's own installer leaves a configuration that cannot start — and the
 * failure appears in somebody's assistant rather than in the tool that wrote
 * it.
 *
 * Everything on screen is read from the project's own files: `composer.lock`
 * for whether the packages are installed, `routes/ai.php` for the servers this
 * project registered by name, and the client configurations for what stands
 * today. Nothing is written until a button is pressed.
 *
 * ## The web row has no button, and that is the point
 *
 * `Mcp::web()` is an ordinary route inside the application. It is already
 * served on the project's own domain, over the certificate the browser already
 * trusts — there is no process to start and nothing to register, so the row
 * shows the URL and stops.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const report = ref(null);
const error = ref(null);
const loading = ref(false);
/** The `${server}:${file}` currently being written, so one spinner is enough. */
const writing = ref(null);
const wrote = ref(null);

const packages = computed(() => {
  const found = report.value?.packages ?? {};
  return [
    ['laravel/boost', found.boost],
    ['laravel/mcp', found.mcp],
    ['laravel/ai', found.ai],
  ];
});

const installed = computed(() => packages.value.some(([, version]) => !!version));

async function load() {
  loading.value = true;
  error.value = null;
  wrote.value = null;
  try {
    report.value = await api.boostStatus(props.name);
  } catch (e) {
    report.value = null;
    error.value = e;
  } finally {
    loading.value = false;
  }
}

async function register(server, file) {
  writing.value = `${server}:${file}`;
  error.value = null;
  try {
    wrote.value = await api.boostRegister(props.name, file, server);
    // Read back rather than patched in place: what the file now says is the
    // answer, and a row this card coloured itself would be this card's opinion
    // of what it just did.
    report.value = await api.boostStatus(props.name);
  } catch (e) {
    error.value = e;
  } finally {
    writing.value = null;
  }
}

/** The colour a state deserves — `hostPhp` is the one that is actually broken. */
function tone(state) {
  return (
    {
      container: 'success',
      hostPhp: 'error',
      other: 'warning',
      unparseable: 'warning',
    }[state] ?? undefined
  );
}

function icon(state) {
  return (
    {
      container: 'mdi-check-circle-outline',
      hostPhp: 'mdi-alert-circle-outline',
      other: 'mdi-help-circle-outline',
      unparseable: 'mdi-file-alert-outline',
    }[state] ?? 'mdi-circle-small'
  );
}

watch(
  () => props.name,
  () => {
    report.value = null;
    error.value = null;
    wrote.value = null;
  }
);
</script>

<template>
  <section class="pane">
    <PaneHeader
      help="project-boost"
      icon="mdi-robot-outline"
      :title="t('boost.title')"
      :description="t('boost.desc')"
    />

    <v-btn
      size="small"
      variant="tonal"
      prepend-icon="mdi-magnify"
      :loading="loading"
      data-test="boost-read"
      @click="load"
    >
      {{ t('boost.read') }}
    </v-btn>

    <ErrorAlert v-if="error" :error="error" class="mt-3" />

    <template v-if="report">
      <!-- The packages, first. Everything below only means something if one of
           them is installed. -->
      <div class="text-caption text-medium-emphasis mt-3 mb-2">
        <span v-for="([pkg, version], i) in packages" :key="pkg">
          <template v-if="i"> · </template>
          <code>{{ pkg }}</code>
          <template v-if="version"> {{ version }}</template>
          <template v-else> — {{ t('boost.notInstalled') }}</template>
        </span>
      </div>

      <v-alert
        v-if="!installed"
        type="info"
        variant="tonal"
        density="compact"
        class="text-caption"
        data-test="boost-none"
      >
        {{ t('boost.installHow', { container: report.container }) }}
      </v-alert>

      <template v-else>
        <!-- laravel/mcp with no routes file: the servers this would list are
             ones the project has not written yet. Said rather than shown as an
             empty list. -->
        <v-alert
          v-if="report.packages.mcp && !report.hasRoutes"
          type="info"
          variant="tonal"
          density="compact"
          class="text-caption mb-3"
          data-test="boost-noroutes"
        >
          {{ t('boost.noRoutes') }}
        </v-alert>

        <v-alert
          v-if="!report.servers.length"
          type="info"
          variant="tonal"
          density="compact"
          class="text-caption"
        >
          {{ t('boost.noServers') }}
        </v-alert>

        <div
          v-for="server in report.servers"
          :key="server.id"
          class="mb-4"
          data-test="boost-server"
        >
          <div class="text-body-2 mb-1">
            <v-icon size="18" class="mr-2">mdi-server-network-outline</v-icon>
            <template v-if="server.server.kind === 'boost'">{{ t('boost.serverBoost') }}</template>
            <template v-else-if="server.server.kind === 'local'">
              {{ t('boost.serverLocal', { handle: server.server.handle }) }}
            </template>
            <template v-else>{{ t('boost.serverWeb', { path: server.server.path }) }}</template>
          </div>

          <!-- A route: already served, nothing to register. -->
          <template v-if="server.server.kind === 'web'">
            <p class="text-caption text-medium-emphasis">
              {{ t('boost.webAlreadyServed') }}
              <code v-if="server.url">{{ server.url }}</code>
            </p>
          </template>

          <template v-else>
            <p class="text-caption text-medium-emphasis mb-2">
              {{ t('boost.willWrite') }} <code>{{ server.command }}</code>
            </p>

            <v-list density="compact" class="bg-transparent pa-0">
              <v-list-item
                v-for="file in server.files"
                :key="file.id"
                class="px-0"
                data-test="boost-file"
              >
                <template #prepend>
                  <v-icon :color="tone(file.state)" size="18" class="mr-3">
                    {{ icon(file.state) }}
                  </v-icon>
                </template>
                <v-list-item-title class="text-body-2">
                  {{ file.label }} — <code>{{ file.path }}</code>
                </v-list-item-title>
                <v-list-item-subtitle class="text-caption">
                  {{ t(`boost.state.${file.state}`) }}
                  <template v-if="file.command">
                    <br /><code>{{ file.command }}</code>
                  </template>
                </v-list-item-subtitle>
                <template #append>
                  <!-- Withheld on an unparseable file, and that is the rule
                       rather than a caution: a file this cannot read is one it
                       must not rewrite. -->
                  <v-btn
                    v-if="file.state !== 'container' && file.state !== 'unparseable'"
                    size="x-small"
                    variant="tonal"
                    :loading="writing === `${server.id}:${file.id}`"
                    data-test="boost-register"
                    @click="register(server.id, file.id)"
                  >
                    {{
                      file.state === 'unregistered' || file.state === 'absent'
                        ? t('boost.register')
                        : t('boost.repair')
                    }}
                  </v-btn>
                </template>
              </v-list-item>
            </v-list>
          </template>
        </div>

        <p v-if="wrote" class="text-caption text-success" data-test="boost-wrote">
          {{ t('boost.wrote', { path: wrote }) }}
        </p>
      </template>
    </template>
  </section>
</template>
