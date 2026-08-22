<script setup>
import { useI18n } from 'vue-i18n';
import PageLayout from '@/components/PageLayout.vue';
import LogView from '@/components/LogView.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * One live tail across every project.
 *
 * The per-project viewer answers "what did this project write". It cannot
 * answer "which of my eight projects just errored" — the question you have
 * before you know which project to open, and the one that made the per-project
 * viewer a place you went eight times in a row. Herd sells this as a Pro
 * feature.
 *
 * A page rather than a tab: it belongs to no project, and it is somewhere you
 * leave open while you work in another window.
 *
 * Everything on screen is `LogView` in `scope="all"`. Nothing about search,
 * level filtering, following or click-through is reimplemented here — a second
 * console renderer is a second place for the two to drift, and the ways they
 * would drift (level inheritance, the `/var/www/html` substitution) are exactly
 * the details that took the longest to get right the first time.
 */
const { t } = useI18n();
</script>

<template>
  <PageLayout
    help="page-logs"
    top-icon="mdi-text-box-multiple-outline"
    :top-title="t('logs.title')"
    :top-subtitle="t('logs.allDescription')"
    hide-bar
  >
    <!-- Named like the card it is, the way the project page names its panes.
         The blue band above says what the *page* is; this says what the card
         under it holds — and the toolbar below is not a title, it is what you
         can do to the stream. -->
    <div class="logs-head">
      <PaneHeader
        help="page-logs-all-projects"
        icon="mdi-text-box-multiple-outline"
        :title="t('logs.allProjects')"
        :description="t('logs.allExplain')"
      />
    </div>

    <LogView scope="all" class="flex-grow-1" />
  </PageLayout>
</template>

<style scoped>
/* The one padded part of the card: the console under it runs to the card's
   edges by design, so the heading is what needs the inset. */
.logs-head {
  flex: 0 0 auto;
  padding: 16px 16px 0;
}
</style>
