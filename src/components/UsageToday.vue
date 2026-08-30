<script setup>
import { computed, onMounted, onBeforeUnmount, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import HelpButton from '@/components/HelpButton.vue';
import { api, asList } from '@/lib/ipc';

/**
 * What Docker actually cost on this machine today.
 *
 * The rest of this page draws what is happening *now*, which is the wrong tense
 * for the question people ask about Docker — the one they ask is what it cost
 * them over an afternoon. Every product in this category has the same cost and
 * none of them measures it, so this is the card that turns the honest answer in
 * the README ("here is what Docker costs you") into a measurement rather than
 * an apology.
 *
 * Read once a minute, because that is the rate the totals change at: the
 * sampler behind them runs on a sixty-second timer, and a faster poll would ask
 * a question whose answer cannot have moved.
 */
const { t } = useI18n();

const report = ref(null);
const error = ref(null);
const budgets = ref({});
let timer = null;

async function load() {
  try {
    report.value = await api.usageReport();
    error.value = null;
  } catch (e) {
    // Kept on screen rather than blanking the card: "I could not read the
    // record" and "nothing has run today" are different answers.
    error.value = e;
  }
}

async function loadBudgets() {
  try {
    budgets.value = (await api.prefsGet())?.usageBudgets ?? {};
  } catch {
    // A preferences file that cannot be read is a card with no budgets on it,
    // which is what a machine with none set looks like anyway.
    budgets.value = {};
  }
}

onMounted(() => {
  load();
  loadBudgets();
  timer = setInterval(load, 60_000);
});
onBeforeUnmount(() => clearInterval(timer));

// ---- the budget, set where the cost is shown -------------------------------
//
// Here rather than on the project's own page, because this is the screen that
// makes somebody want one: a budget is a reaction to a number, and putting the
// field two clicks away from the number would be putting it where nobody is
// looking when they decide they want it.

const editing = ref(null);

function edit(row) {
  const current = budgets.value[row.name] ?? {};
  editing.value = {
    name: row.name,
    cpuMinutes: current.cpuMinutes ?? null,
    gbHours: current.gbHours ?? null,
  };
}

async function saveBudget() {
  const { name, cpuMinutes, gbHours } = editing.value;
  // `prefs_set` merges shallowly, so the whole map is sent — reading it back
  // first is what stops one project's budget from clearing everybody else's.
  const next = { ...budgets.value };
  const cpu = Number(cpuMinutes) || 0;
  const gb = Number(gbHours) || 0;

  // Cleared is removed rather than stored as zero. A zero already means "no
  // budget" everywhere else here, and two spellings of one state is one more
  // thing to keep in step.
  if (cpu > 0 || gb > 0) {
    next[name] = { cpuMinutes: cpu || undefined, gbHours: gb || undefined };
  } else {
    delete next[name];
  }

  await api.prefsSet({ usageBudgets: next });
  budgets.value = next;
  editing.value = null;
  await load();
}

const rows = computed(() => asList(report.value?.rows));

/** `38 min` — CPU seconds are the unit `time` reports; minutes are the unit
 *  somebody has a feel for. */
const minutes = (seconds) => `${(seconds / 60).toFixed(seconds < 600 ? 1 : 0)} ${t('usage.min')}`;

/** Two decimals under ten, none above: `0.42 GB·h`, `12 GB·h`. */
const gbh = (value) => `${value < 10 ? value.toFixed(2) : value.toFixed(0)} ${t('usage.gbh')}`;

const KINDS = { project: 'primary', service: 'info', stack: 'surface-variant' };
</script>

<template>
  <v-card elevation="1" class="pa-4">
    <div class="d-flex align-start mb-2">
      <div class="flex-grow-1">
        <div class="text-subtitle-1 font-weight-medium">{{ t('usage.title') }}</div>
        <div class="text-caption text-medium-emphasis">{{ t('usage.sub') }}</div>
      </div>
      <div v-if="report" class="d-flex ga-6 mr-2">
        <div class="text-center">
          <div class="text-caption text-medium-emphasis">{{ t('usage.cpu') }}</div>
          <div class="text-h6">{{ minutes(report.cpuSeconds) }}</div>
        </div>
        <div class="text-center">
          <div class="text-caption text-medium-emphasis">{{ t('usage.memory') }}</div>
          <div class="text-h6">{{ gbh(report.gbHours) }}</div>
        </div>
      </div>
      <HelpButton
        topic="page-dashboard-usage"
        :subject="t('usage.title')"
        class="align-self-start"
      />
    </div>

    <!-- Nothing yet is the ordinary state of a machine that just started, and
         is a sentence rather than an empty table. -->
    <v-alert v-if="!rows.length" type="info" variant="tonal" density="compact" class="text-caption">
      {{ error ? t('usage.unreadable') : t('usage.nothingYet') }}
    </v-alert>

    <v-table v-else density="compact" class="text-caption">
      <thead>
        <tr>
          <th>{{ t('usage.what') }}</th>
          <th class="text-right">{{ t('usage.cpu') }}</th>
          <th class="text-right">{{ t('usage.memory') }}</th>
          <th class="text-right">{{ t('usage.budget') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in rows" :key="row.name" data-test="usage-row">
          <td>
            <span class="mr-2">{{ row.name }}</span>
            <!-- Named, not colour alone: a shared service is not any one
                 project's cost and the row has to say so in words. -->
            <v-chip size="x-small" label :color="KINDS[row.kind]">
              {{ t(`usage.kind.${row.kind}`) }}
            </v-chip>
          </td>
          <td class="text-right">{{ minutes(row.cpuSeconds) }}</td>
          <td class="text-right">{{ gbh(row.gbHours) }}</td>
          <td class="text-right">
            <v-chip v-if="row.overBudget" size="x-small" color="warning" label class="mr-1">
              {{ t('usage.over') }}
            </v-chip>
            <span
              v-else-if="row.budgetCpuMinutes || row.budgetGbHours"
              class="text-medium-emphasis mr-1"
            >
              {{
                [
                  row.budgetCpuMinutes ? `${row.budgetCpuMinutes} ${t('usage.min')}` : null,
                  row.budgetGbHours ? `${row.budgetGbHours} ${t('usage.gbh')}` : null,
                ]
                  .filter(Boolean)
                  .join(' · ')
              }}
            </span>
            <span v-else-if="row.kind !== 'project'" class="text-disabled">—</span>
            <!-- Only on a project. A shared service is nobody's to be over on,
                 and offering the field would be offering a setting that does
                 nothing. -->
            <v-btn
              v-if="row.kind === 'project'"
              icon
              size="x-small"
              variant="text"
              :aria-label="t('usage.setBudget', { name: row.name })"
              @click="edit(row)"
            >
              <v-icon size="16">mdi-pencil-outline</v-icon>
            </v-btn>
          </td>
        </tr>
      </tbody>
    </v-table>

    <div class="text-caption text-medium-emphasis mt-2">{{ t('usage.sharedNote') }}</div>

    <v-dialog v-model="editing" max-width="420" @update:model-value="(v) => v || (editing = null)">
      <v-card v-if="editing" class="pa-4">
        <div class="text-subtitle-1 mb-1">{{ t('usage.budgetFor', { name: editing.name }) }}</div>
        <div class="text-caption text-medium-emphasis mb-3">{{ t('usage.budgetExplain') }}</div>

        <v-text-field
          v-model="editing.cpuMinutes"
          type="number"
          min="0"
          :label="t('usage.budgetCpu')"
          :placeholder="t('usage.noBudget')"
          persistent-placeholder
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />
        <v-text-field
          v-model="editing.gbHours"
          type="number"
          min="0"
          :label="t('usage.budgetMemory')"
          :placeholder="t('usage.noBudget')"
          persistent-placeholder
          density="compact"
          variant="outlined"
          hide-details
        />

        <div class="d-flex ga-2 mt-4">
          <v-spacer />
          <v-btn size="small" variant="text" @click="editing = null">{{ t('app.cancel') }}</v-btn>
          <v-btn size="small" variant="flat" color="primary" @click="saveBudget">
            {{ t('usage.saveBudget') }}
          </v-btn>
        </div>
      </v-card>
    </v-dialog>
  </v-card>
</template>
