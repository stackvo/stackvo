<script setup>
import { useI18n } from 'vue-i18n';
import LogView from '@/components/LogView.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * One project's logs.
 *
 * A thin wrapper over `LogView`, and the point of it is the `built` guard: the
 * container stream carries stdout and nothing an application logs goes there,
 * so the file sources are what make this pane useful — and they only exist once
 * the project has been built.
 */
defineProps({
  project: { type: Object, required: true },
  name: { type: String, required: true },
  active: { type: Boolean, default: false },
});

const { t } = useI18n();
</script>

<template>
  <!-- In the card every other tab's content sits in, and the height of what is
       left of the window rather than of the log file. It used to run to the raw
       edges of the page, which made the one tab that reads like a different
       application out of the one whose content never ends. -->
  <v-card variant="flat" class="pane logs-pane">
    <!-- Named like every other pane on the page. The viewer under it brings a
         toolbar of its own — a source picker, a search box, the controls — and
         a toolbar is not a title: it says what you can do to the output, never
         what the card is. -->
    <div class="logs-head">
      <PaneHeader
        help="project-logs"
        icon="mdi-text-box-outline"
        :title="t('logs.title')"
        :description="t('logs.explain')"
      />
    </div>

    <LogView
      v-if="project.built"
      :container="project.containerName"
      :project="name"
      :active="active"
    />
    <div v-else class="text-caption text-medium-emphasis py-8 text-center">
      {{ t('detail.notBuilt') }}
    </div>
  </v-card>
</template>

<style scoped>
/* The viewer brings its own toolbar and its own scrolling viewport, so the
   card is a frame and nothing else: no padding of its own, and a column that
   hands its whole height to the one child inside it. */
.pane.logs-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 0;
  overflow: hidden;
}

.pane.logs-pane > *:not(.logs-head) {
  flex: 1 1 auto;
  min-height: 0;
}

/* The one part of this card that is padded: the viewer under it runs to the
   card's edges by design, and a heading that did the same would sit against
   the border. */
.pane.logs-pane > .logs-head {
  flex: 0 0 auto;
  padding: 16px 16px 0;
}
</style>
