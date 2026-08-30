<script setup>
import { computed, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useWorktrees } from '@/composables/useWorktrees';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';
import { useCopyTick } from '@/composables/useCopyTick';

/**
 * N — a branch with an environment of its own.
 *
 * ## One pane, two roles
 *
 * A project either *has* worktrees or *is* one, never both — creating a
 * worktree of a worktree is refused, so the branch structure stays one step
 * deep and readable. Two panes would have meant two entries in the rail, one of
 * which is always empty, and an empty tab is a promise the page cannot keep.
 *
 * ## The plan is what the form shows
 *
 * Every derived string on screen — the project name, the hostname, the database
 * name — comes back from `worktreePlan` rather than being assembled here. The
 * derivations are not obvious (a slug folds punctuation to hyphens, a database
 * name loses its stem rather than its branch when it will not fit), and a
 * second implementation in JavaScript would be a preview that shows one name
 * while the backend creates another.
 *
 * The same call is what refuses. A refusal is not an error: it belongs on the
 * form beside the field that caused it, so it is drawn as an inline alert and
 * the Create button is simply disabled.
 */
const props = defineProps({
  name: { type: String, required: true },
});

const emit = defineEmits(['changed', 'removed']);

const { t } = useI18n();
const { copied, copy } = useCopyTick();

const {
  support,
  plan,
  record,
  isWorktree,
  worktrees,
  branches,
  instances,
  available,
  reason,
  loading,
  planning,
  busy,
  error,
  load,
  preview,
  create,
  remove,
  saveEnv,
} = useWorktrees(computed(() => props.name));

// ---------------------------------------------------------------- creating

const creating = ref(false);
const form = reactive({
  branch: '',
  newBranch: false,
  name: '',
  database: 'none',
  instance: null,
  /** Minutes, or null for "until somebody says otherwise". */
  minutes: null,
});

/**
 * The branches a worktree can be made from.
 *
 * The ones already checked out stay in the list and are disabled: removing them
 * would leave somebody looking for a branch they can see in their terminal,
 * with nothing on screen saying why it is absent.
 */
const branchItems = computed(() =>
  branches.value.map((branch) => ({
    title: branch.name,
    value: branch.name,
    props: {
      disabled: branch.checkedOut,
      subtitle: branch.checkedOut ? t('worktree.branchTaken') : undefined,
    },
  }))
);

const dbInstances = computed(() =>
  instances.value.map((instance) => ({
    title: instance.running ? instance.id : `${instance.id} — ${t('worktree.stopped')}`,
    value: instance.id,
    props: { disabled: !instance.running },
  }))
);

/** The database modes, with copy withheld where there is nothing to copy. */
const databaseModes = computed(() => [
  { title: t('worktree.dbNone'), value: 'none' },
  { title: t('worktree.dbCreate'), value: 'create' },
  { title: t('worktree.dbCopy'), value: 'copy' },
]);

const options = computed(() => ({
  newBranch: form.newBranch,
  name: form.name.trim() || null,
  database: form.database,
  instance: form.instance,
  minutes: form.minutes,
}));

/**
 * How long the environment is wanted for.
 *
 * Empty is the default and means a worktree in the ordinary sense — somebody's
 * branch, theirs until they say otherwise. Choosing a duration is what makes it
 * a sandbox: something built for one task, by somebody who is not going to
 * remember it exists.
 */
const ttlOptions = computed(() => [
  { title: t('worktree.ttlNone'), value: null },
  { title: t('worktree.ttlHours', { count: 2 }), value: 120 },
  { title: t('worktree.ttlHours', { count: 8 }), value: 480 },
  { title: t('worktree.ttlDays', { count: 1 }), value: 1440 },
  { title: t('worktree.ttlDays', { count: 7 }), value: 10080 },
]);

/** `2 h 15 m` left, in the units somebody reads rather than in minutes. */
function left(minutes) {
  if (minutes === null || minutes === undefined) return null;
  if (minutes <= 0) return t('worktree.expired');
  if (minutes < 60) return t('worktree.leftMinutes', { count: minutes });
  if (minutes < 60 * 24) return t('worktree.leftHours', { count: Math.floor(minutes / 60) });
  return t('worktree.leftDays', { count: Math.floor(minutes / (60 * 24)) });
}

/**
 * Re-plan whenever anything the plan depends on changes.
 *
 * Debounced by nothing, deliberately: the call reads two files and asks git,
 * and `newBranch` typing is the only field that fires per keystroke. Adding a
 * timer would mean a form that shows a stale name for 300ms after the last
 * character, which is exactly when somebody reads it.
 */
watch(
  () => [form.branch, form.newBranch, form.name, form.database, form.instance, form.minutes],
  () => preview(form.branch.trim(), options.value)
);

function openForm() {
  creating.value = true;
  form.branch = '';
  form.newBranch = false;
  form.name = '';
  form.database = 'none';
  form.instance = instances.value.find((i) => i.running)?.id ?? null;
  form.minutes = null;
  plan.value = null;
}

async function submit() {
  if (await create(form.branch.trim(), options.value)) {
    creating.value = false;
    emit('changed');
  }
}

// ---------------------------------------------------------------- removing

const removing = ref(null);
const removal = reactive({ force: false, dropDatabase: false, deleteBranch: false });

function askRemove(row) {
  removing.value = row;
  removal.force = false;
  removal.dropDatabase = false;
  removal.deleteBranch = false;
}

async function confirmRemove() {
  const target = removing.value.name;
  const self = target === props.name;
  if (await remove(target, { ...removal })) {
    removing.value = null;
    emit(self ? 'removed' : 'changed');
  }
}

// ------------------------------------------------------- this worktree's env

const rows = ref([]);
const savedEnv = ref(null);

/** A list rather than an object, so a half-typed key does not vanish on
 *  re-render — the same reason `SitePane` keeps one. */
function toRows(env) {
  return Object.entries(env || {}).map(([key, value]) => ({ key, value }));
}

/**
 * What the container is given, minus what was typed.
 *
 * Shown separately because the two are different kinds of thing: one is a
 * setting somebody made, the other is what this app derived from the instance
 * and cannot be edited here — editing `DB_PASSWORD` in a text field would be
 * editing a copy of a value that is read live on every render.
 */
const derived = computed(() => {
  const typed = new Set(rows.value.map((row) => row.key.trim()));
  const all = savedEnv.value ?? support.value?.effectiveEnv ?? {};
  return Object.entries(all).filter(([key]) => !typed.has(key));
});

async function persistEnv() {
  const env = {};
  for (const row of rows.value) {
    const key = row.key.trim();
    if (key) env[key] = row.value;
  }
  const saved = await saveEnv(env);
  if (saved) {
    rows.value = toRows(saved.env);
    savedEnv.value = saved.effective;
    emit('changed');
  }
}

watch(
  () => props.name,
  async () => {
    creating.value = false;
    removing.value = null;
    savedEnv.value = null;
    await load();
    rows.value = toRows(record.value?.env);
  },
  { immediate: true }
);
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-worktree"
      icon="mdi-source-branch"
      :title="t('worktree.title')"
      :description="isWorktree ? t('worktree.explainSelf') : t('worktree.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-3" />

    <!-- Why it cannot be done here. A sentence from the boundary, so the screen
         and the command cannot disagree about the reason. -->
    <v-alert
      v-if="!isWorktree && !loading && reason"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-3"
      data-test="worktree-reason"
    >
      <div class="text-caption">{{ reason }}</div>
    </v-alert>

    <!-- ============================================ this project IS a worktree -->
    <template v-if="isWorktree && record">
      <div class="field">
        <span class="field-key">{{ t('worktree.parent') }}</span>
        <router-link class="field-val field-link" :to="`/projects/${record.parent}`">
          {{ record.parent }}
        </router-link>
      </div>
      <div class="field">
        <span class="field-key">{{ t('worktree.branch') }}</span>
        <span class="field-val field-mono">{{ record.branch }}</span>
      </div>
      <div class="field">
        <span class="field-key">{{ t('worktree.domain') }}</span>
        <span class="field-val field-mono">{{ record.domain }}</span>
      </div>
      <div class="field">
        <span class="field-key">{{ t('worktree.database') }}</span>
        <span class="field-val field-mono">
          {{
            record.database
              ? `${record.database.name} — ${record.database.instance}`
              : t('worktree.noDatabase')
          }}
        </span>
      </div>
      <!-- The fact that decides whether "its own database" also means "not the
           parent's". Stated either way: on PostgreSQL and MongoDB the shared
           login is the only answer there is, and a field that appeared only in
           the good case would read as a feature that sometimes forgets. -->
      <div v-if="record.database" class="field">
        <span class="field-key">{{ t('worktree.login') }}</span>
        <span class="field-val">
          {{ support?.isolated ? t('worktree.ownLogin') : t('worktree.sharedLogin') }}
        </span>
      </div>
      <p v-if="record.database && !support?.isolated" class="text-caption text-medium-emphasis">
        {{ t('worktree.sharedLoginExplain') }}
      </p>
      <!-- Whether the branch started from a copy of the data or an empty
           schema is the question somebody asks three weeks later. -->
      <div v-if="record.database?.seededFrom" class="field">
        <span class="field-key">{{ t('worktree.seededFrom') }}</span>
        <span class="field-val field-mono">{{ record.database.seededFrom }}</span>
      </div>
      <div v-if="record.expiresAt" class="field">
        <span class="field-key">{{ t('worktree.expiresAt') }}</span>
        <span class="field-val field-mono">
          {{ new Date(record.expiresAt).toLocaleString() }}
          <template v-if="support?.expired"> — {{ t('worktree.expired') }}</template>
        </span>
      </div>

      <v-divider class="my-3" />

      <!-- The sentence this whole arrangement is for: give an assistant this
           branch and nothing else. Rendered by the backend from the same grant
           the server enforces, so what is copied here is what is applied
           there — a second spelling on this screen would be a second thing to
           get wrong. -->
      <template v-if="support?.grantArgs?.length">
        <div class="section-head mb-2">
          <v-icon size="16" class="mr-2">mdi-robot-outline</v-icon>{{ t('worktree.forAgent') }}
        </div>
        <p class="text-caption text-medium-emphasis mb-2">{{ t('worktree.forAgentExplain') }}</p>
        <div class="d-flex align-center ga-2 mb-3">
          <code class="text-caption flex-grow-1" style="word-break: break-all">{{
            support.grantArgs.join(' ')
          }}</code>
          <v-btn
            size="small"
            variant="text"
            :prepend-icon="copied === 'grant' ? 'mdi-check' : 'mdi-content-copy'"
            @click="copy(support.grantArgs.join(' '), 'grant')"
          >
            {{ t('app.copy') }}
          </v-btn>
        </div>
      </template>

      <v-divider class="my-3" />

      <div class="section-head mb-2">
        <v-icon size="16" class="mr-2">mdi-code-braces</v-icon>{{ t('worktree.envTitle') }}
      </div>
      <p class="text-caption text-medium-emphasis mb-2">{{ t('worktree.envExplain') }}</p>

      <div
        v-for="(row, index) in rows"
        :key="index"
        class="d-flex ga-2 mb-2"
        data-test="wt-env-row"
      >
        <v-text-field
          v-model="row.key"
          :label="t('worktree.key')"
          density="compact"
          variant="outlined"
          hide-details
          class="flex-grow-0"
          style="max-width: 220px"
        />
        <v-text-field
          v-model="row.value"
          :label="t('worktree.value')"
          density="compact"
          variant="outlined"
          hide-details
        />
        <v-btn
          icon
          size="small"
          variant="text"
          :aria-label="t('worktree.removeRow')"
          @click="rows.splice(index, 1)"
        >
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </div>

      <div class="d-flex ga-2 mb-3">
        <v-btn
          size="small"
          variant="text"
          prepend-icon="mdi-plus"
          @click="rows.push({ key: '', value: '' })"
        >
          {{ t('worktree.addRow') }}
        </v-btn>
        <v-spacer />
        <v-btn
          size="small"
          color="primary"
          variant="tonal"
          :loading="busy === 'env'"
          @click="persistEnv"
        >
          {{ t('worktree.saveEnv') }}
        </v-btn>
      </div>

      <!-- Read-only, and said to be: these are read from the instance on every
           render, so a copy edited here would be a copy that goes stale. -->
      <template v-if="derived.length">
        <p class="text-caption text-medium-emphasis mb-2">{{ t('worktree.derivedExplain') }}</p>
        <div v-for="[key, value] in derived" :key="key" class="field" data-test="wt-derived">
          <span class="field-key field-mono">{{ key }}</span>
          <span class="field-val field-mono">{{ value }}</span>
        </div>
      </template>

      <v-divider class="my-3" />

      <v-btn
        size="small"
        variant="text"
        color="error"
        prepend-icon="mdi-delete-outline"
        @click="askRemove(record)"
      >
        {{ t('worktree.remove') }}
      </v-btn>
    </template>

    <!-- ================================================ this project HAS worktrees -->
    <template v-else-if="!loading">
      <div
        v-for="row in worktrees"
        :key="row.name"
        class="wt-row d-flex align-center ga-3"
        data-test="worktree-row"
      >
        <div class="min-width-0 flex-grow-1">
          <router-link class="field-link" :to="`/projects/${row.name}`">{{ row.name }}</router-link>
          <div class="text-caption text-medium-emphasis field-mono">
            {{ row.branch }} → {{ row.domain }}
          </div>
        </div>
        <v-chip v-if="row.database" size="x-small" label class="field-mono">
          {{ row.database.name }}
        </v-chip>
        <!-- Only the state worth acting on. A chip on every isolated row would
             be a badge on the normal case, which is how a list stops being
             read. -->
        <v-chip v-if="row.database && !row.isolated" size="x-small" color="warning" label>
          {{ t('worktree.sharedLogin') }}
        </v-chip>
        <!-- A sandbox says how long it has. Nothing removes it on a timer: an
             app that deleted a directory by itself would eventually delete one
             with a morning's uncommitted work in it. -->
        <v-chip
          v-if="row.expired"
          size="x-small"
          color="warning"
          label
          data-test="worktree-expired"
        >
          {{ t('worktree.expired') }}
        </v-chip>
        <v-chip v-else-if="row.remainingMinutes !== undefined" size="x-small" label>
          {{ left(row.remainingMinutes) }}
        </v-chip>
        <!-- Three states about the checkout, and only two of them are trouble. -->
        <v-chip v-if="row.orphaned" size="x-small" color="warning" label>
          {{ t('worktree.orphaned') }}
        </v-chip>
        <v-chip v-else-if="row.dirty" size="x-small" color="info" label>
          {{ t('worktree.dirty') }}
        </v-chip>
        <v-btn
          icon
          size="small"
          variant="text"
          :aria-label="t('worktree.remove')"
          :loading="busy === row.name"
          @click="askRemove(row)"
        >
          <v-icon>mdi-delete-outline</v-icon>
        </v-btn>
      </div>

      <p
        v-if="!worktrees.length && available && !creating"
        class="text-caption text-medium-emphasis mb-0"
      >
        {{ t('worktree.none') }}
      </p>

      <!-- ------------------------------------------------------- the form -->
      <template v-if="creating">
        <v-divider class="my-3" />

        <v-switch
          v-model="form.newBranch"
          color="primary"
          density="compact"
          hide-details
          class="mb-2"
          :label="t('worktree.createBranch')"
        />

        <v-combobox
          v-if="!form.newBranch"
          v-model="form.branch"
          :items="branchItems"
          :label="t('worktree.branch')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />
        <v-text-field
          v-else
          v-model="form.branch"
          :label="t('worktree.newBranchName')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />

        <v-text-field
          v-model="form.name"
          :label="t('worktree.nameOverride')"
          :placeholder="plan?.name"
          persistent-placeholder
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />

        <v-select
          v-model="form.database"
          :items="databaseModes"
          :label="t('worktree.databaseMode')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />
        <v-select
          v-if="form.database !== 'none'"
          v-model="form.instance"
          :items="dbInstances"
          :label="t('worktree.instance')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />

        <!-- The field that turns a worktree into a sandbox. Empty is the
             default, because a branch somebody made for themselves is theirs
             until they say otherwise. -->
        <v-select
          v-model="form.minutes"
          :items="ttlOptions"
          :label="t('worktree.ttl')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-1"
        />
        <p class="text-caption text-medium-emphasis mb-3">{{ t('worktree.ttlExplain') }}</p>

        <!-- What would actually be made. Every string here is the backend's
             own derivation, not a second one. -->
        <template v-if="plan">
          <div class="field">
            <span class="field-key">{{ t('worktree.willBeCalled') }}</span>
            <span class="field-val field-mono">{{ plan.name }}</span>
          </div>
          <div class="field">
            <span class="field-key">{{ t('worktree.willAnswerAt') }}</span>
            <span class="field-val field-mono">{{ plan.domain }}</span>
          </div>
          <div v-if="plan.database" class="field">
            <span class="field-key">{{ t('worktree.database') }}</span>
            <span class="field-val field-mono">
              {{ plan.database.name }}
              <template v-if="plan.database.seed">
                — {{ t('worktree.copiedFrom', { source: plan.database.source }) }}
              </template>
            </span>
          </div>

          <div v-if="plan.expiresAt" class="field">
            <span class="field-key">{{ t('worktree.expiresAt') }}</span>
            <span class="field-val field-mono">{{
              new Date(plan.expiresAt).toLocaleString()
            }}</span>
          </div>

          <v-alert
            v-for="warning in plan.warnings"
            :key="warning"
            type="warning"
            variant="tonal"
            density="compact"
            class="mt-3"
            data-test="worktree-warning"
          >
            <div class="text-caption">{{ warning }}</div>
          </v-alert>

          <v-alert
            v-if="plan.refused"
            type="error"
            variant="tonal"
            density="compact"
            class="mt-3"
            data-test="worktree-refused"
          >
            <div class="text-caption">{{ plan.refused }}</div>
          </v-alert>
        </template>

        <div class="d-flex ga-2 mt-3">
          <v-spacer />
          <v-btn size="small" variant="text" @click="creating = false">
            {{ t('worktree.cancel') }}
          </v-btn>
          <v-btn
            size="small"
            color="primary"
            variant="flat"
            :disabled="!plan?.possible || planning"
            :loading="busy === 'create'"
            @click="submit"
          >
            {{ t('worktree.create') }}
          </v-btn>
        </div>
      </template>
    </template>

    <!-- The one control that acts on the pane rather than on a row, so it sits
         under the list it adds to. Hidden while the form is open: the form is
         already the thing this button opens. -->
    <div v-if="!isWorktree && available && !creating" class="pane-foot">
      <v-spacer />
      <v-btn
        size="small"
        color="primary"
        variant="flat"
        prepend-icon="mdi-plus"
        :loading="busy === 'create'"
        @click="openForm"
      >
        {{ t('worktree.new') }}
      </v-btn>
    </div>

    <!-- ------------------------------------------------------- removal -->
    <!-- Each thing that can be destroyed is its own switch, off by default.
         A single "delete everything" would make the branch and the database
         casualties of pressing the only button on offer. -->
    <!-- `:model-value` rather than `v-model`: the dialog's model is a boolean
         and `removing` is the row being removed, so binding it directly makes
         the row the open flag and Vuetify warns about the type on every open. -->
    <v-dialog
      :model-value="removing !== null"
      :max-width="520"
      @update:model-value="(open) => !open && (removing = null)"
    >
      <v-card v-if="removing">
        <v-card-title class="text-body-1">
          {{ t('worktree.removeTitle', { name: removing.name }) }}
        </v-card-title>
        <v-card-text>
          <p class="text-caption text-medium-emphasis mb-3">
            {{ t('worktree.removeExplain', { branch: removing.branch }) }}
          </p>

          <v-switch
            v-if="removing.dirty"
            v-model="removal.force"
            color="error"
            density="compact"
            hide-details
            :label="t('worktree.removeForce')"
          />
          <v-switch
            v-if="removing.database"
            v-model="removal.dropDatabase"
            color="error"
            density="compact"
            hide-details
            :label="t('worktree.removeDatabase', { name: removing.database.name })"
          />
          <v-switch
            v-model="removal.deleteBranch"
            color="error"
            density="compact"
            hide-details
            :label="t('worktree.removeBranch', { branch: removing.branch })"
          />

          <ErrorAlert v-if="error" :error="error" class="mt-3" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="removing = null">{{ t('worktree.cancel') }}</v-btn>
          <v-btn
            color="error"
            variant="flat"
            :loading="busy === removing.name"
            @click="confirmRemove"
          >
            {{ t('worktree.remove') }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-card>
</template>

<style scoped>
/* One row per worktree, separated rather than boxed: the list is short and a
   card each would make three branches look like three pages of settings. */
.wt-row {
  padding: 8px 0;
}

.wt-row + .wt-row {
  border-top: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
</style>
