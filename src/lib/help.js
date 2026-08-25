/**
 * Which card explains itself with which document.
 *
 * Every card on the project and settings pages carries a help button, and every
 * one of those buttons names a *topic* — a stable slug that maps to one file
 * under `docs/help/`. The slug is written at the call site rather than derived
 * from a title, because a title is a translated string: deriving the filename
 * from it would rename the document when somebody rewords a heading, and
 * differently in each locale.
 *
 * This module is only the map. What a topic's document says, and how it is
 * shown, is not decided here — the viewer reads `helpDoc()` and goes looking.
 *
 * The list is exhaustive on purpose: `help-topics.spec.js` reads every
 * `help="…"` in the sources and fails on one this file does not know, which is
 * what stops a card shipping a button that opens nothing.
 */

export const HELP_TOPICS = [
  'page-dashboard',
  'page-dashboard-cpu',
  'page-dashboard-cpu-history',
  'page-dashboard-disk-io',
  'page-dashboard-health',
  'page-dashboard-images',
  'page-dashboard-landing',
  'page-dashboard-memory',
  'page-dashboard-network',
  'page-dashboard-projects',
  'page-dashboard-services',
  'page-dashboard-storage',
  'page-dumps',
  'page-dumps-all-projects',
  'page-logs',
  'page-logs-all-projects',
  'page-mail',
  'page-mail-inbox',
  'page-market',
  'page-market-available',
  'page-market-instances',
  'page-project-detail',
  'page-projects',
  'page-settings',
  'page-settings-resources',
  'page-settings-system',
  'page-settings-updates',
  'panel-new-project',
  'project-agent',
  'project-container',
  'project-dev-server',
  'project-devcontainer',
  'project-dockerfile',
  'project-dumps',
  'project-editor',
  'project-hooks',
  'project-indicator-composition',
  'project-indicator-cpu-activity',
  'project-lan',
  'project-local-override',
  'project-logs',
  'project-manifest',
  'project-oauth',
  'project-overview',
  'project-perf',
  'project-php-ini',
  'project-profiler',
  'project-providers',
  'project-query-log',
  'project-release',
  'project-repl',
  'project-requirements',
  'project-scheduler',
  'project-supervisor',
  'project-sidecars',
  'project-site',
  'project-spx',
  'project-stripe',
  'project-terminal',
  'project-timeline',
  'project-tunnel',
  'project-workers',
  'project-why-slow',
  'project-worktree',
  'project-xdebug',
  'settings-agents',
  'settings-appearance-presets',
  'settings-appearance-status-colors',
  'settings-appearance-theme-colors',
  'settings-appearance-typography',
  'settings-catalogue-bundle',
  'settings-catalogue-source',
  'settings-certificates',
  'settings-diagnostics',
  'settings-diagnostics-engine',
  'settings-dns',
  'settings-domain-address',
  'settings-domain-hosts',
  'settings-domain-network',
  'settings-domain-proxy',
  'settings-idle',
  'settings-local-api',
  'settings-localisation-console-language',
  'settings-localisation-direction',
  'settings-localisation-language',
  'settings-php',
  'settings-php-runtimes',
  'settings-php-tools',
  'settings-preferences-backups',
  'settings-preferences-external-apps',
  'settings-preferences-startup',
  'settings-routes',
  'settings-secrets',
  'settings-server-directives',
  'settings-server-limits',
  'settings-server-limits-applies',
  'settings-template-overrides',
  'settings-tooling',
  'settings-workspace-compose',
  'settings-workspace-export',
  'settings-workspace-generator',
  'settings-workspace-group',
  'settings-workspace-import',
];

/**
 * Where a topic's document lives, for one locale.
 *
 * Per-locale directories rather than one file holding both languages: the
 * viewer reads a document and shows it, and a reader who set the interface to
 * Turkish should not be handed a page that opens in English and switches
 * halfway down. `en` is the fallback — a topic written in one language is still
 * readable by somebody who set the other, and an empty pane is not.
 */
export const HELP_LOCALES = ['en', 'tr'];

export const HELP_FALLBACK_LOCALE = 'en';

export const helpDoc = (topic, locale = HELP_FALLBACK_LOCALE) =>
  `docs/help/${HELP_LOCALES.includes(locale) ? locale : HELP_FALLBACK_LOCALE}/${topic}.md`;

export const isHelpTopic = (topic) => HELP_TOPICS.includes(topic);
