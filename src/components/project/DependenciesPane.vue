<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Z-1 — the same discipline `pkg.rs` applies to packages, applied to this
 * project's own.
 *
 * ## Two buttons, and the split is the design
 *
 * Everything the lock file already says is read **on this machine**: plain
 * `http://` sources, packages nothing verifies, which index each one came from.
 * That half needs no network and never asks for one.
 *
 * Asking whether any of them has a published advisory sends the names and
 * versions of the dependencies to `api.osv.dev`. That is a real disclosure, so
 * it is a second button with the sentence above it — not a flag on the first,
 * and not something that happens when the pane opens.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const { t } = useI18n();

const report = ref(null);
const error = ref(null);
const reading = ref(false);
const asking = ref(false);

const advisories = computed(() => report.value?.advisories ?? null);
const hosts = computed(() => Object.entries(report.value?.hosts ?? {}));

async function read() {
  reading.value = true;
  error.value = null;
  try {
    report.value = await api.depsReport(props.name);
  } catch (e) {
    report.value = null;
    error.value = e;
  } finally {
    reading.value = false;
  }
}

async function ask() {
  asking.value = true;
  error.value = null;
  try {
    report.value = await api.depsAdvisories(props.name);
  } catch (e) {
    // The local half stays on screen. A failed query must not read as a clean
    // result, and `advisories` staying null is exactly what says "nobody asked".
    error.value = e;
  } finally {
    asking.value = false;
  }
}

watch(
  () => props.name,
  () => {
    report.value = null;
    error.value = null;
  }
);
</script>

<template>
  <section class="pane">
    <PaneHeader
      help="project-dependencies"
      icon="mdi-package-variant-closed"
      :title="t('deps.title')"
      :description="t('deps.desc')"
    />

    <v-btn
      size="small"
      variant="tonal"
      prepend-icon="mdi-package-variant-closed"
      :loading="reading"
      data-test="deps-read"
      @click="read"
    >
      {{ t('deps.read') }}
    </v-btn>

    <ErrorAlert v-if="error" :error="error" class="mt-3" />

    <template v-if="report">
      <!-- "No findings" and "no lock file" are different answers, and a project
           this cannot see is not a clean project. -->
      <v-alert
        v-if="!report.locks.length"
        type="info"
        variant="tonal"
        density="compact"
        class="mt-3 text-caption"
        data-test="deps-nolock"
      >
        {{ t('deps.noLock') }}
      </v-alert>

      <template v-else>
        <div class="text-caption text-medium-emphasis mt-3 mb-2">
          {{
            t('deps.summary', {
              total: report.total,
              direct: report.direct,
              locks: report.locks.join(', '),
            })
          }}
        </div>

        <div v-if="hosts.length" class="text-caption text-medium-emphasis mb-3">
          {{ t('deps.hosts') }}
          <span v-for="([host, count], i) in hosts" :key="host">
            <template v-if="i">, </template><code>{{ host }}</code> ({{ count }})
          </span>
        </div>

        <!-- The local half. Findings first, because they are actionable
             without anything leaving the machine. -->
        <v-list v-if="report.findings.length" density="compact" class="bg-transparent pa-0 mb-3">
          <v-list-item
            v-for="(finding, i) in report.findings"
            :key="`${finding.id}-${finding.subject}-${i}`"
            class="px-0"
            data-test="deps-finding"
          >
            <template #prepend>
              <v-icon
                :color="finding.id === 'insecureSource' ? 'error' : 'warning'"
                size="18"
                class="mr-3"
              >
                {{
                  finding.id === 'insecureSource'
                    ? 'mdi-lock-open-alert-outline'
                    : 'mdi-alert-circle-outline'
                }}
              </v-icon>
            </template>
            <v-list-item-title class="text-body-2">
              {{ t(`deps.finding.${finding.id}`, { subject: finding.subject }) }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ t(`deps.fix.${finding.id}`, { detail: finding.detail ?? '' }) }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
        <v-alert v-else type="success" variant="tonal" density="compact" class="mb-3 text-caption">
          {{ t('deps.localClean') }}
        </v-alert>

        <!-- ---- the half that leaves the machine ------------------------ -->
        <v-divider class="my-3" />

        <!-- Said before the button, not after. A consent decision belongs
             where the decision is made. -->
        <p class="text-caption text-medium-emphasis mb-2">{{ t('deps.whatIsSent') }}</p>
        <v-btn
          size="small"
          variant="tonal"
          prepend-icon="mdi-shield-search"
          :loading="asking"
          :disabled="!report.total"
          data-test="deps-ask"
          @click="ask"
        >
          {{ t('deps.ask', { count: report.total }) }}
        </v-btn>

        <template v-if="advisories">
          <v-alert
            v-if="!advisories.length"
            type="success"
            variant="tonal"
            density="compact"
            class="mt-3 text-caption"
            data-test="deps-clean"
          >
            {{ t('deps.noAdvisories') }}
          </v-alert>
          <v-list v-else density="compact" class="bg-transparent pa-0 mt-2">
            <v-list-item
              v-for="row in advisories"
              :key="row.package"
              class="px-0"
              data-test="advisory"
            >
              <template #prepend>
                <v-icon :color="row.direct ? 'error' : 'warning'" size="18" class="mr-3">
                  mdi-shield-alert-outline
                </v-icon>
              </template>
              <v-list-item-title class="text-body-2">
                <code>{{ row.package }}</code>
                <!-- Named, because the two have different repairs: a direct
                     dependency is a version you choose. -->
                <v-chip size="x-small" label class="ml-2" :color="row.direct ? 'error' : undefined">
                  {{ row.direct ? t('deps.direct') : t('deps.transitive') }}
                </v-chip>
              </v-list-item-title>
              <v-list-item-subtitle class="text-caption">
                {{ row.ids.join(', ') }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </template>
      </template>
    </template>
  </section>
</template>
