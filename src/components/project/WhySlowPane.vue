<script setup>
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { api, asList } from '@/lib/ipc';
import { micros, percent } from '@/lib/format';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * Why this request was slow — the three instruments on one request.
 *
 * B-1. SPX, the query log and the axis were three panes on this same tab, and
 * each of them answers a different third of one question: where the code's time
 * went, what the database was asked, and what else happened while it ran.
 * Answering the question they were all built for meant opening three of them
 * and comparing clocks by eye.
 *
 * ## The recording is the subject, and everything else is placed against it
 *
 * A php-spx report is the only thing in this app that names a request, says
 * when it started and says how long it took. So the pane opens on a recording —
 * not on a database, not on a time range — and every other number here is what
 * that stretch of wall clock held.
 *
 * That join is by **time**, and this pane's job is to keep saying so. The
 * `timeline` module's refusal to attribute a statement to a request has not
 * been reversed and is not being worked around: a statement still carries no
 * request. What the reader gets instead of a guess is a stated window and, when
 * something else was recorded across the same stretch, its name.
 *
 * ## The answer above the evidence
 *
 * The findings come first, the numbers second, and the three lists last and
 * closed. `QueryLogPane` learned the same shape — a page asking one query three
 * hundred times is the finding, and four hundred rows of SQL are the evidence
 * for it — and this pane is that idea applied to all three instruments at once.
 */
const props = defineProps({
  name: { type: String, required: true },
  runtime: { type: String, default: '' },
});

const { t } = useI18n();

const status = ref(null);
const targets = ref([]);
const service = ref(null);
const key = ref(null);
const explanation = ref(null);
const loading = ref(false);
const error = ref(null);

/**
 * The databases whose log can be read, asked for here rather than passed in.
 *
 * The same list and the same reasoning as `QueryLogPane`: `db_targets` is a
 * workspace-level answer, and threading it through the page would make this
 * pane's correctness depend on a parent remembering to fetch it.
 */
const usable = computed(() =>
  asList(targets.value).filter((target) =>
    ['mysql', 'mariadb', 'postgres', 'mongo'].includes(target.service)
  )
);

const reports = computed(() => asList(status.value?.reports));
const findings = computed(() => asList(explanation.value?.findings));
const split = computed(() => explanation.value?.split ?? null);
const hotspots = computed(() => asList(explanation.value?.hotspots));
const repeats = computed(() => asList(explanation.value?.repeats));
const queries = computed(() => asList(explanation.value?.queries));
const moments = computed(() => asList(explanation.value?.moments));

/** What a recording is called on screen: the request, the command, or its key. */
const label = (report) => report.request || report.command || report.key;

// The same rendering `SpxPane` gives a recording's timestamp, so the two
// panes name one recording the same way.
const when = (seconds) => new Date(seconds * 1000).toLocaleString();

/**
 * The recordings, as a picker.
 *
 * Newest first, which is `spx::list`'s own order — the recording somebody wants
 * to look at is nearly always the one they just made.
 */
const choices = computed(() =>
  reports.value.map((report) => ({
    value: report.key,
    title: label(report),
    subtitle: `${when(report.recordedAt)} · ${micros(report.wallTimeUs)}`,
  }))
);

/**
 * The sentence one finding renders as.
 *
 * Built here rather than in Rust, and that is the reason the payload carries a
 * kind and its numbers instead of a sentence: this window speaks two languages
 * and a string assembled behind the boundary would be English in both.
 */
function say(finding) {
  return t(`whySlow.finding.${finding.kind}`, {
    subject: finding.subject ?? '',
    count: finding.count ?? 0,
    percent: (finding.percent ?? 0).toFixed(0),
  });
}

/**
 * Is this finding an answer, or a qualification of one?
 *
 * The first three name something to change; the rest say what the evidence
 * could not cover. Colouring them the same would put "your page runs this query
 * 240 times" and "the query log was off" at one weight.
 */
const ANSWERS = new Set(['nPlusOne', 'databaseBound', 'hotspot']);
const tone = (finding) => (ANSWERS.has(finding.kind) ? 'warning' : 'info');
const icon = (finding) =>
  ({
    nPlusOne: 'mdi-repeat-variant',
    databaseBound: 'mdi-database-clock',
    hotspot: 'mdi-fire',
    noDriverFrames: 'mdi-help-rhombus-outline',
    queriesUnrecorded: 'mdi-database-off-outline',
    queriesOutsideWindow: 'mdi-timer-off-outline',
    overlaps: 'mdi-call-split',
    traceMissing: 'mdi-file-remove-outline',
    truncated: 'mdi-content-cut',
  })[finding.kind] ?? 'mdi-information-outline';

const SOURCE_ICON = {
  dump: 'mdi-code-braces',
  query: 'mdi-database',
  mail: 'mdi-email-outline',
};

/** Seconds into the request, so the axis reads as one page load rather than as clock time. */
const offset = (at) => {
  const from = explanation.value?.window?.from ?? at;
  return `+${(at - from).toFixed(3)}s`;
};

async function loadStatus() {
  error.value = null;
  try {
    status.value = await api.spxStatus(props.name);
  } catch (e) {
    error.value = e;
  }
}

async function load() {
  if (!key.value) {
    explanation.value = null;
    return;
  }
  loading.value = true;
  error.value = null;
  try {
    explanation.value = await api.requestExplain(props.name, key.value, service.value);
  } catch (e) {
    error.value = e;
    explanation.value = null;
  } finally {
    loading.value = false;
  }
}

/**
 * Re-read the list, and stay pointed at something that still exists.
 *
 * The recordings are made and deleted in the php-spx card on this same tab —
 * one of its buttons deletes all of them. Refreshing without this check leaves
 * the picker holding a key nothing answers to, and `request_explain` is
 * deliberately an error in that case rather than an empty explanation. So the
 * pinned recording is kept when it survived and dropped to the newest when it
 * did not, which is also what somebody who just recorded a second request
 * means by "refresh".
 */
async function refresh() {
  await loadStatus();
  const list = reports.value;
  if (!list.some((report) => report.key === key.value)) {
    key.value = list.length ? list[0].key : null;
  }
  await load();
}

/**
 * Both answers first, then the defaults, then one read.
 *
 * The order is the whole of it. The recordings and the databases arrive from
 * two commands that settle at different moments, and the first version of this
 * picked a default the instant either one landed — which meant the recording
 * was chosen while the database list was still empty, and the first
 * explanation on screen was fetched with no database at all. It corrected
 * itself a moment later when the second answer arrived, so the defect was one
 * frame of a pane reporting the database half as absent, and a wasted read of a
 * gzip trace behind it.
 *
 * Nothing watches `key` or `service` for that reason: the pickers call `load`
 * themselves. A watcher would have to be told which changes are the user's and
 * which are this function's own, and that is a flag nobody can keep true.
 */
onMounted(async () => {
  // PHP only: php-spx is a PHP extension, so a Node project has no recording
  // this could ever open on.
  if (props.runtime !== 'php') return;

  await loadStatus();
  try {
    targets.value = asList(await api.dbTargets());
  } catch {
    // A workspace whose databases cannot be listed still has recordings worth
    // reading, and the pane reports the database half as absent on its own.
    targets.value = [];
  }

  // Open on the newest recording rather than on an empty picker somebody has
  // to discover, and on the first readable database for the same reason.
  if (!service.value && usable.value.length) service.value = usable.value[0].service;
  if (!key.value && reports.value.length) key.value = reports.value[0].key;

  await load();
});
</script>

<template>
  <v-card v-if="runtime === 'php'" variant="flat" class="pane">
    <PaneHeader
      help="project-why-slow"
      icon="mdi-timer-alert-outline"
      :title="t('whySlow.title')"
      :description="t('whySlow.explain')"
    >
      <template #append>
        <v-btn
          v-if="reports.length"
          size="small"
          variant="text"
          prepend-icon="mdi-refresh"
          :loading="loading"
          @click="refresh"
        >
          {{ t('app.refresh') }}
        </v-btn>
      </template>
    </PaneHeader>

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <!-- Nothing recorded. Said as the one action that leads somewhere rather
         than as an empty picker: this pane cannot record, and the pane that
         can is on the same tab. -->
    <v-alert v-if="!reports.length" type="info" variant="tonal">
      <div class="text-caption">{{ t('whySlow.nothingRecorded') }}</div>
    </v-alert>

    <template v-else>
      <div class="d-flex ga-3 flex-wrap align-start">
        <v-select
          v-model="key"
          :items="choices"
          item-title="title"
          item-value="value"
          :label="t('whySlow.recording')"
          density="compact"
          variant="outlined"
          hide-details
          prepend-inner-icon="mdi-record-circle-outline"
          class="picker"
          @update:model-value="load"
        >
          <template #item="{ props: itemProps, item }">
            <v-list-item v-bind="itemProps" :subtitle="item.raw.subtitle" />
          </template>
        </v-select>

        <!-- Only when there is a choice to make. One database is not a
             decision, and a select with a single option is a control that
             invites a look and gives nothing back. -->
        <v-select
          v-if="usable.length > 1"
          v-model="service"
          :items="usable.map((target) => ({ value: target.service, title: target.service }))"
          :label="t('whySlow.database')"
          density="compact"
          variant="outlined"
          hide-details
          prepend-inner-icon="mdi-database"
          class="picker"
          @update:model-value="load"
        />
      </div>

      <v-progress-linear v-if="loading" indeterminate class="mt-3" />

      <template v-if="explanation">
        <!-- THE SUBJECT ------------------------------------------------- -->
        <div class="subject mt-4">
          <div class="text-body-2 font-weight-medium">
            {{ explanation.request || explanation.command || explanation.key }}
          </div>
          <div class="text-caption text-medium-emphasis">
            {{ when(explanation.recordedAt) }} ·
            {{ explanation.cli ? t('whySlow.cli') : t('whySlow.httpRequest') }} ·
            {{ t('whySlow.took', { took: micros(explanation.wallTimeUs) }) }}
          </div>
          <!-- The join, stated. Every number below is "what this stretch of
               wall clock held", and a reader who is not told that will read it
               as "what this request did". -->
          <div class="text-caption text-medium-emphasis mt-1">
            {{ t('whySlow.window') }}
          </div>
          <!-- And where the stretch itself came from. A window this app watched
               and a window it worked out are not equally trustworthy: the
               second rests on a reading of php-spx's `exec_ts` that nothing
               here has measured, and a reader comparing two recordings should
               be able to see which of the two they are looking at. -->
          <div class="text-caption text-medium-emphasis mt-1">
            <v-icon size="12" class="mr-1">
              {{ explanation.window.basis === 'observed' ? 'mdi-eye-outline' : 'mdi-calculator' }}
            </v-icon>
            {{
              explanation.window.basis === 'observed'
                ? t('whySlow.windowObserved')
                : t('whySlow.windowDerived')
            }}
          </div>
        </div>

        <!-- THE ANSWER --------------------------------------------------- -->
        <div class="section-head mt-4 mb-1">{{ t('whySlow.findings') }}</div>

        <div v-if="!findings.length" class="text-caption text-medium-emphasis">
          {{ t('whySlow.nothingToSay') }}
        </div>

        <div v-for="(finding, i) in findings" :key="i" class="finding">
          <v-icon size="18" :color="tone(finding)" class="mt-1">{{ icon(finding) }}</v-icon>
          <div class="min-width-0">
            <div class="text-body-2">{{ say(finding) }}</div>
            <div v-if="finding.subject" class="text-caption text-medium-emphasis finding-subject">
              <code>{{ finding.subject }}</code>
            </div>
          </div>
        </div>

        <!-- WHERE THE TIME WENT ------------------------------------------ -->
        <template v-if="split">
          <div class="section-head mt-4 mb-1">{{ t('whySlow.split') }}</div>
          <!-- One bar rather than two numbers: the question is a proportion,
               and a proportion drawn is read in one glance where a pair of
               percentages is read twice. -->
          <div
            class="bar"
            role="img"
            :aria-label="
              t('whySlow.splitLabel', {
                database: split.databasePercent.toFixed(0),
                php: split.phpPercent.toFixed(0),
              })
            "
          >
            <div class="bar-db" :style="{ width: `${split.databasePercent}%` }"></div>
            <div class="bar-php" :style="{ width: `${split.phpPercent}%` }"></div>
          </div>
          <div class="d-flex ga-4 text-caption mt-1 flex-wrap">
            <span>
              <v-icon size="12" color="warning">mdi-square</v-icon>
              {{ t('whySlow.inDatabase') }} — {{ micros(split.databaseUs) }} ({{
                percent(split.databasePercent, 0)
              }})
            </span>
            <span>
              <v-icon size="12" color="primary">mdi-square</v-icon>
              {{ t('whySlow.inPhp') }} — {{ micros(split.phpUs) }} ({{
                percent(split.phpPercent, 0)
              }})
            </span>
          </div>
          <div class="text-caption text-medium-emphasis mt-1">{{ t('whySlow.splitHint') }}</div>
        </template>

        <!-- THE EVIDENCE, CLOSED ----------------------------------------- -->
        <v-expansion-panels variant="accordion" multiple class="mt-4">
          <v-expansion-panel v-if="hotspots.length">
            <v-expansion-panel-title>
              {{ t('whySlow.hotspots', { n: explanation.functions }) }}
            </v-expansion-panel-title>
            <v-expansion-panel-text>
              <v-table density="compact">
                <tbody>
                  <tr v-for="spot in hotspots" :key="spot.function">
                    <td class="mono spot-name">{{ spot.function }}</td>
                    <td class="mono text-right">{{ micros(spot.exclusiveUs) }}</td>
                    <td class="mono text-right">{{ percent(spot.exclusivePercent) }}</td>
                    <td class="mono text-right">{{ spot.calls }}</td>
                  </tr>
                </tbody>
              </v-table>
            </v-expansion-panel-text>
          </v-expansion-panel>

          <v-expansion-panel>
            <v-expansion-panel-title>
              {{ t('whySlow.statements', { n: explanation.queryCount }) }}
            </v-expansion-panel-title>
            <v-expansion-panel-text>
              <!-- Absent, not empty. Without this line an unrecorded window and
                   a request that asked nothing are the same picture — which is
                   the distinction `queriesRecording` exists to carry. -->
              <div v-if="!explanation.queriesRecording" class="text-caption text-medium-emphasis">
                {{ t('whySlow.notRecording') }}
              </div>
              <div v-else-if="!queries.length" class="text-caption text-medium-emphasis">
                {{
                  explanation.queriesElsewhere
                    ? t('whySlow.noneInWindow', { n: explanation.queriesElsewhere })
                    : t('whySlow.noneAtAll')
                }}
              </div>
              <template v-else>
                <div v-for="repeat in repeats" :key="repeat.shape" class="repeat">
                  <div class="d-flex align-center ga-2">
                    <v-chip size="x-small" color="warning" variant="tonal">
                      ×{{ repeat.count }}
                    </v-chip>
                    <code class="repeat-shape">{{ repeat.shape }}</code>
                  </div>
                </div>
                <div class="entries" :class="{ 'mt-2': repeats.length }">
                  <div v-for="(entry, i) in queries" :key="i" class="entry">
                    <span class="entry-at mono">{{ offset(entry.at) }}</span>
                    <!-- WCAG 3.1.2, undetermined — see `QueryLogPane`. -->
                    <code class="entry-sql" lang="">{{ entry.sql }}</code>
                  </div>
                </div>
              </template>
            </v-expansion-panel-text>
          </v-expansion-panel>

          <v-expansion-panel v-if="moments.length">
            <v-expansion-panel-title>
              {{ t('whySlow.axis', { n: moments.length }) }}
            </v-expansion-panel-title>
            <v-expansion-panel-text>
              <div class="entries">
                <div v-for="(moment, i) in moments" :key="i" class="entry">
                  <span class="entry-at mono">{{ offset(moment.at) }}</span>
                  <v-icon size="14" class="entry-icon">{{ SOURCE_ICON[moment.source] }}</v-icon>
                  <span class="entry-summary" lang="">{{ moment.summary }}</span>
                </div>
              </div>
            </v-expansion-panel-text>
          </v-expansion-panel>
        </v-expansion-panels>
      </template>
    </template>
  </v-card>
</template>

<style scoped>
.picker {
  min-width: 240px;
  flex: 1 1 240px;
}

.subject {
  padding: 10px 12px;
  border-radius: 8px;
  background: rgb(var(--v-theme-surface-light));
}

.finding {
  display: flex;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid rgb(var(--v-border-color), var(--v-border-opacity));
}

.finding-subject code {
  word-break: break-all;
}

/* The proportion, drawn. Two blocks in a row rather than a stacked chart: this
   is one number and its complement, and anything richer would imply a third. */
.bar {
  display: flex;
  height: 10px;
  border-radius: 5px;
  overflow: hidden;
  background: rgb(var(--v-theme-surface-variant));
}

.bar-db {
  background: rgb(var(--v-theme-warning));
}

.bar-php {
  background: rgb(var(--v-theme-primary));
}

.spot-name {
  font-size: 0.75rem;
  word-break: break-all;
}

.repeat {
  padding: 6px 0;
}

.repeat-shape {
  font-size: 0.8rem;
  word-break: break-all;
}

/* Bounded and scrolled rather than let to run off the page: `PageLayout` is a
   fixed-height flex column, so an unbounded child is compressed instead —
   the same guard `page-scroll.spec.js` earned for `QueryLogPane`. */
.entries {
  max-height: 320px;
  overflow-y: auto;
}

.entry {
  display: flex;
  gap: 8px;
  padding: 3px 0;
  font-size: 0.75rem;
  align-items: baseline;
}

.entry-at {
  flex: 0 0 auto;
  color: rgb(var(--v-theme-on-surface-variant));
  opacity: 0.7;
}

.entry-icon {
  flex: 0 0 auto;
  opacity: 0.6;
}

.entry-sql,
.entry-summary {
  word-break: break-all;
}
</style>
