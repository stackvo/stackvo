<script setup>
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { api } from '@/lib/ipc';
import { renderMarkdown } from '@/lib/markdown';
import { useHelp } from '@/composables/useHelp';
import SideSheet from '@/components/SideSheet.vue';
import ErrorAlert from '@/components/ErrorAlert.vue';

/**
 * What a card is for, beside the card rather than over it.
 *
 * A side sheet and not a dialog, for the reason `SideSheet` itself gives: a
 * dialog covers what it is about. Half of what these documents say is "this
 * button writes X and restarts Y", and reading that with the button hidden
 * behind the panel explaining it is the wrong way round.
 *
 * Mounted once, in `App.vue`. Every help button in the application writes a
 * topic into one module-level ref, and this is what watches it — so a card
 * halfway down a tab needs no wiring to a panel that is not its child, and
 * moving from one card's help to another's swaps the content of an open panel
 * instead of opening a second one.
 *
 * ## Read on open, every time
 *
 * The documents are files on disk precisely so that a sentence can be corrected
 * without a release; caching them here would hand the reader yesterday's
 * sentence until they restarted the application. A markdown file is a few
 * kilobytes and the read is local.
 */
const { t, locale } = useI18n();
const { topic, closeHelp } = useHelp();

const source = ref('');
const error = ref(null);
const loading = ref(false);

const open = computed({
  get: () => !!topic.value,
  set: (next) => {
    if (!next) closeHelp();
  },
});

const html = computed(() => renderMarkdown(source.value));

/**
 * The document's own `# heading` is the panel's title.
 *
 * Taken from the file rather than from the card that opened it: the card's
 * title is a translated interface string and the document is a translated
 * document, and when they disagree the one the reader is looking at should win.
 */
const title = computed(() => {
  const first = /^#\s+(.+)$/m.exec(source.value);
  return first ? first[1].trim() : t('a11y.help');
});

watch(topic, async (next) => {
  if (!next) return;
  loading.value = true;
  error.value = null;
  source.value = '';
  try {
    source.value = await api.helpDoc(next, locale.value);
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <SideSheet v-model="open" icon="mdi-help-circle-outline" :title="title" :width="620" above>
    <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-4" />

    <!-- A topic whose document nobody has written yet is a normal state, not a
         fault: the documents land one tab at a time. It says which topic, so
         whoever sees it knows what to write. -->
    <ErrorAlert v-if="error" :error="error" type="info" class="mb-4" />
    <div v-if="error" class="text-caption text-medium-emphasis">
      {{ t('help.notWritten', { topic }) }}
    </div>

    <!-- eslint-disable-next-line vue/no-v-html -->
    <article v-else class="help-doc" v-html="html" />
  </SideSheet>
</template>

<style scoped>
/* Long-form prose, which nothing else in this application is: the line height
   and the space between blocks are what make eight hundred words readable in a
   620px column. */
.help-doc {
  font-size: 0.9rem;
  line-height: 1.65;
}

/* `:deep()` throughout — every element below is written by `renderMarkdown`
   into `v-html`, so none of them carries this component's scope attribute. */
.help-doc :deep(h1) {
  /* The panel's header already shows it. Kept in the document because the file
     has to open with one, and hidden here rather than stripped in the renderer,
     which would make the renderer know what it is rendering for. */
  display: none;
}

.help-doc :deep(h2) {
  font-size: 1rem;
  font-weight: 600;
  margin: 28px 0 8px;
}

.help-doc :deep(h2:first-child) {
  margin-top: 0;
}

.help-doc :deep(h3) {
  font-size: 0.9rem;
  font-weight: 600;
  margin: 20px 0 6px;
}

.help-doc :deep(p) {
  margin: 0 0 12px;
}

.help-doc :deep(ul) {
  margin: 0 0 12px;
  padding-inline-start: 20px;
}

.help-doc :deep(li) {
  margin-bottom: 6px;
}

.help-doc :deep(code) {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.82em;
  background: rgba(var(--v-border-color), 0.1);
  border-radius: 4px;
  padding: 1px 5px;
}

.help-doc :deep(pre) {
  background: rgba(var(--v-border-color), 0.08);
  border-radius: var(--app-radius);
  padding: 10px 12px;
  overflow-x: auto;
  margin: 0 0 12px;
}

.help-doc :deep(pre code) {
  background: none;
  padding: 0;
  font-size: 0.78rem;
  line-height: 1.5;
}

/* A table of fields is most of what these documents are. It scrolls inside its
   own box rather than widening the panel. */
.help-doc :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 0 0 16px;
  font-size: 0.84rem;
}

.help-doc :deep(th) {
  text-align: start;
  font-weight: 600;
  padding: 6px 10px 6px 0;
  border-bottom: thin solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.help-doc :deep(td) {
  padding: 8px 10px 8px 0;
  border-bottom: thin solid rgba(var(--v-border-color), calc(var(--v-border-opacity) / 2));
  vertical-align: top;
}

/* The first column of these tables is the name of a control, and a name that
   wraps to three words per line is a column of confetti. */
.help-doc :deep(td:first-child) {
  white-space: nowrap;
  padding-inline-end: 16px;
}

.help-doc :deep(a) {
  color: rgb(var(--v-theme-primary));
}
</style>
