<script setup>
import { toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { useChecks } from '@/composables/useSupervisors';

/**
 * The probe that answers what `RUNNING` cannot, edited for one process.
 *
 * Its own component rather than markup inside the pane: the pane owns the rows
 * and this owns what a check is, and keeping them apart is what stops a check's
 * shape being described in two places.
 */
const props = defineProps({
  project: { type: String, default: null },
});
const emit = defineEmits(['saved']);

const { t } = useI18n();

const { checks, editing, trying, valid, load, checkFor, open, close, save, remove, tryIt } =
  useChecks(toRef(props, 'project'));

async function commit() {
  if (await save()) emit('saved');
}

async function drop(process) {
  await remove(process);
  emit('saved');
}

// The parents drive this: they own the rows, so they own which row is being
// asked about. `checks` and `checkFor` go back out so a row can show whether it
// has one without a second fetch.
defineExpose({ open, load, checks, checkFor });
</script>

<template>
  <v-dialog :model-value="Boolean(editing)" max-width="560" @update:model-value="close">
    <v-card v-if="editing">
      <v-card-title class="text-body-1">
        {{ t('supervisorCheck.title', { process: editing.process }) }}
      </v-card-title>

      <v-card-text>
        <p class="text-caption text-medium-emphasis mb-4">{{ t('supervisorCheck.explain') }}</p>

        <v-row dense>
          <v-col cols="12" md="4">
            <v-select
              v-model="editing.kind"
              :items="[
                { title: t('supervisorCheck.kinds.http'), value: 'http' },
                { title: t('supervisorCheck.kinds.tcp'), value: 'tcp' },
              ]"
              :label="t('supervisorCheck.kind')"
              density="compact"
              variant="outlined"
            />
          </v-col>
          <v-col cols="12" md="8">
            <v-text-field
              v-model="editing.target"
              :label="t(`supervisorCheck.target.${editing.kind}`)"
              :placeholder="editing.kind === 'http' ? 'https://shop.loc/up' : '127.0.0.1:9000'"
              persistent-placeholder
              density="compact"
              variant="outlined"
            />
          </v-col>
          <!-- A health endpoint behind auth answers 401 and is working, so the
               status that counts is a field rather than a constant. -->
          <v-col v-if="editing.kind === 'http'" cols="12" md="4">
            <v-text-field
              v-model="editing.expectStatus"
              :label="t('supervisorCheck.expect')"
              type="number"
              density="compact"
              variant="outlined"
            />
          </v-col>
        </v-row>

        <div v-if="trying" class="text-caption mt-2">
          <span v-if="trying === 'running'" class="text-medium-emphasis">
            {{ t('supervisorCheck.trying') }}
          </span>
          <span v-else :class="trying.ok ? 'text-success' : 'text-error'">
            {{ trying.detail }} · {{ trying.ms }}ms
          </span>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-btn
          v-if="checkFor(editing.process)"
          variant="text"
          color="error"
          @click="drop(editing.process)"
        >
          {{ t('supervisorCheck.remove') }}
        </v-btn>
        <v-spacer />
        <v-btn variant="text" :disabled="!valid" @click="tryIt">
          {{ t('supervisorCheck.try') }}
        </v-btn>
        <v-btn variant="text" @click="close">{{ t('supervisors.cancel') }}</v-btn>
        <v-btn color="primary" variant="flat" :disabled="!valid" @click="commit">
          {{ t('supervisors.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
