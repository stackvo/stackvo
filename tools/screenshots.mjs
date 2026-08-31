/**
 * The screenshots the README wants, taken by the machine rather than by hand.
 *
 * ```sh
 * npm run screenshots              # build, then shoot every page
 * npm run screenshots -- --no-build   # reuse dist/ as it stands
 * npm run screenshots -- --page settings --page dashboard
 * ```
 *
 * ## Why this exists
 *
 * A screenshot taken by hand is taken at whatever size the window happened to
 * be, on whatever machine happened to be free, with whatever projects that
 * person happened to have — and the vertical is the part that gets cut, because
 * a person capturing a window captures the screen it fits on. None of that is
 * repeatable, and the next one never matches the last one.
 *
 * This takes them at one fixed, wide viewport with the same staged machine every
 * time, in light, and writes them under `docs/screenshots/` with stable names.
 * Re-running it after a UI change reshoots all of them, so the pictures age
 * with the tree rather than with somebody's afternoon.
 *
 * ## What it is a picture of, exactly
 *
 * The webview half, and the honest caption is that word. It boots the built
 * front end in Chromium and replaces `window.__TAURI_INTERNALS__.invoke` with
 * `tests/e2e/stage.js` — the same boundary the Playwright suite uses, answering
 * in the shapes `contracts/ipc.json` declares. So the layout, the components,
 * the theme and the type are the real ones; the two projects and two services
 * on screen are staged, and there is no native title bar around them because
 * there is no Tauri window here.
 *
 * Sharing the stage with the suite is the point rather than a convenience: a
 * screenshot generator with its own fixture drifts into showing a product that
 * no test has ever rendered. This one can only show what `shell.e2e.js` boots.
 */

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, readdir, stat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import { chromium } from '@playwright/test';

import { DEFAULT_STAGE, callsOf, stage } from '../tests/e2e/stage.js';

/**
 * What the staged machine answers on top of `DEFAULT_STAGE`.
 *
 * The one entry that is NOT here any more is `prefs_get`: this file added
 * `tourSeen` to it, because `App.vue` renders `WelcomeTour` *instead of*
 * `<router-view>` while it is unset and the first run produced ten identical
 * pictures of step 1 of 6. It belongs in the shared stage, and it is there now
 * — the Playwright suite was failing nine of nineteen tests for exactly the
 * same reason, against pages the tour was covering.
 */
const STAGE = {
  // Everything below is a page's own question, in the shape `contracts/ipc.json`
  // declares for it. Unanswered they are all `null`, which is a legal reply and
  // renders a spinner or an empty state — true to a machine that has just been
  // installed, and useless as a picture of what the page is for.
  host_stats: {
    cpu: {
      percent: 34.2,
      cores: [41.1, 22.8, 37.4, 29.6, 55.2, 18.3, 31.7, 26.9],
      coreCount: 8,
      loadAverage: [2.14, 1.87, 1.62],
      breakdown: { user: 22.4, nice: 0.3, system: 11.5, idle: 65.8 },
    },
    memory: {
      total: 34_359_738_368,
      used: 19_327_352_832,
      free: 15_032_385_536,
      available: 12_884_901_888,
      percent: 56.3,
      swapTotal: 4_294_967_296,
      swapUsed: 268_435_456,
    },
    storage: {
      total: 994_662_584_320,
      used: 611_248_640_000,
      available: 383_413_944_320,
      percent: 61.5,
      mountPoint: '/',
    },
    network: { rxTotal: 8_912_345_678, txTotal: 2_314_567_890, rxRate: 184_320, txRate: 61_440 },
    disk: {
      readTotal: 4_812_345_678,
      writeTotal: 1_912_345_678,
      readRate: 262_144,
      writeRate: 98_304,
    },
    timestamp: 1_756_684_800,
  },

  docker_system_resources: {
    images: { total: 14, inUse: 9, unused: 5, size: 7_516_192_768 },
    volumes: { total: 6, inUse: 4, unused: 2, size: 2_147_483_648 },
  },

  usage_report: {
    date: '2026-09-01',
    rows: [
      {
        name: 'shop',
        kind: 'project',
        cpuSeconds: 412.7,
        gbHours: 1.84,
        samples: 96,
        overBudget: false,
      },
      {
        name: 'mysql',
        kind: 'service',
        cpuSeconds: 188.3,
        gbHours: 2.61,
        samples: 96,
        overBudget: false,
      },
      {
        name: 'redis',
        kind: 'service',
        cpuSeconds: 21.9,
        gbHours: 0.18,
        samples: 96,
        overBudget: false,
      },
    ],
    cpuSeconds: 622.9,
    gbHours: 4.63,
  },

  landing_status: {
    running: true,
    container: 'stackvo-landing',
    url: 'https://stackvo.loc',
    rendered: null,
    projects: 2,
    services: 2,
  },
  hosts_missing: [],

  // The detail page asks for one project by name, and `shop` is the one the
  // page list points at — the same object, so the two screens agree.
  project_get: DEFAULT_STAGE.projects_list[0],

  container_inspect: {
    name: 'stackvo-shop',
    id: 'b41d7c9e2f18',
    image: 'stackvo/shop:latest',
    state: 'running',
    running: true,
    startedAt: '2026-09-01T06:12:44Z',
    created: '2026-08-27T19:03:10Z',
    restartCount: 0,
    restartPolicy: 'unless-stopped',
    health: 'healthy',
    exitCode: null,
    ports: [{ container: 80, host: 80, protocol: 'tcp' }],
    networks: ['stackvo'],
    gateway: '172.19.0.1',
    mounts: [],
    ipAddress: '172.19.0.7',
    env: [],
    imageSize: 486_539_264,
  },

  mail_status: {
    available: true,
    kind: 'mailpit',
    service: 'mailpit',
    enabled: true,
    running: true,
    uiUrl: 'http://localhost:8025',
    total: 12,
    unread: 3,
    error: null,
  },

  project_manifest_text: JSON.stringify(DEFAULT_STAGE.projects_list[0].manifest, null, 2),

  debug_bridge_overview: [
    { project: 'shop', enabled: true, mounted: true, running: true, events: 24 },
    { project: 'storefront', enabled: false, mounted: false, running: false, events: 0 },
  ],

  app_logs_all: [
    {
      project: 'shop',
      id: 'app:storage/logs/laravel.log',
      label: 'storage/logs/laravel.log',
      group: 'application',
      bytes: 184_320,
      modified: 1_756_681_200,
    },
    {
      project: 'shop',
      id: 'server:nginx/error.log',
      label: 'nginx/error.log',
      group: 'server',
      bytes: 20_480,
      modified: 1_756_677_600,
    },
    {
      project: 'storefront',
      id: 'server:nginx/access.log',
      label: 'nginx/access.log',
      group: 'server',
      bytes: 962_560,
      modified: 1_756_684_200,
    },
  ],

  // The catalogue page gates on `status.fetched === true` and nothing else:
  // `DEFAULT_STAGE`'s answer has no such field, so every shot of this page was
  // the "No catalogue yet" screen — the right screen for a fresh install, and
  // the wrong one for a picture of the market. The source is the real default,
  // `market.rs`'s `stackvo-service-packages`.
  market_status: {
    fetched: true,
    sequence: 41,
    generatedAt: '2026-08-30T04:00:00Z',
    expires: null,
    sourceKind: 'https',
    sourceLocation: 'https://github.com/stackvo/stackvo-service-packages',
    packages: 4,
    installed: 3,
    signed: true,
    signatureRequired: false,
    verifiedBy: 'stackvo-official',
    offlineBundle: null,
    constrained: false,
  },

  market_catalog: [
    marketPackage(
      'mysql',
      'database',
      'MySQL',
      'The relational database most PHP projects assume.',
      [
        { version: '8.4', installed: true, inUse: true },
        { version: '8.0', installed: false, inUse: false },
      ]
    ),
    marketPackage('redis', 'cache', 'Redis', 'Cache, queue and session store in one process.', [
      { version: '8.0', installed: true, inUse: true },
      { version: '7.4', installed: false, inUse: false },
    ]),
    marketPackage(
      'mailpit',
      'mail',
      'Mailpit',
      'Catches every mail the stack sends, and shows it.',
      [{ version: '1.20', installed: true, inUse: false }]
    ),
    marketPackage(
      'meilisearch',
      'search',
      'Meilisearch',
      'Search that answers before the keystroke lands.',
      [{ version: '1.10', installed: false, inUse: false }]
    ),
  ],

  // `Service.id` is the INSTANCE id on the market model — `contracts/ipc.json`
  // says so in as many words — and `Market.vue` joins the two lists on it:
  // `serviceFor(instance)` is `services.find((s) => s.id === instance.id)`.
  // The shared stage still answers the pre-market `mysql`, so the join found
  // nothing and every instance row's Detail button rendered disabled.
  services_list: [
    {
      ...DEFAULT_STAGE.services_list[0],
      id: 'mysql-8-4',
      category: 'database',
      version: '8.4',
      containerName: 'stackvo-mysql-8-4',
      url: null,
    },
    {
      ...DEFAULT_STAGE.services_list[1],
      id: 'redis-8-0',
      category: 'cache',
      version: '8.0',
      containerName: 'stackvo-redis-8-0',
      running: true,
      health: 'healthy',
      url: null,
    },
  ],

  // The service detail sheet reads `dbTargets.value.find(...)` in a computed, so
  // a null answer here threw before the sheet could draw — the overlay never
  // opened and the picture was the page behind it, with the tooltip showing.
  // The create dialog is a title and two buttons until this answers: the form
  // it draws IS the plan — the settings the manifest declares and the port the
  // allocator would hand out.
  instance_plan: {
    id: 'mysql-8-0',
    refused: null,
    settings: [
      {
        key: 'DATABASE',
        kind: 'string',
        value: 'stackvo',
        secret: false,
        isDefault: true,
        defaultValue: 'stackvo',
        required: true,
        options: [],
        label: 'Database name',
      },
      {
        key: 'USER',
        kind: 'string',
        value: 'stackvo',
        secret: false,
        isDefault: true,
        defaultValue: 'stackvo',
        required: true,
        options: [],
        label: 'User',
      },
      {
        key: 'PASSWORD',
        kind: 'secret',
        value: '••••••••',
        secret: true,
        isDefault: false,
        defaultValue: null,
        required: true,
        options: [],
        label: 'Password',
      },
    ],
    ports: [{ name: 'main', container: 3306, host: 3307, protocol: 'tcp' }],
  },

  // The settings sheet reads this one, not the plan above — the plan is what a
  // NEW instance would take, this is what an existing one has. Unanswered, the
  // sheet says "this package has nothing to configure", which is a sentence
  // about a package that declares nothing rather than about a missing reply.
  instance_settings: [
    {
      key: 'DATABASE',
      kind: 'string',
      value: 'stackvo',
      secret: false,
      isDefault: true,
      defaultValue: 'stackvo',
      required: true,
      options: [],
      label: 'Database name',
    },
    {
      key: 'USER',
      kind: 'string',
      value: 'stackvo',
      secret: false,
      isDefault: true,
      defaultValue: 'stackvo',
      required: true,
      options: [],
      label: 'User',
    },
    {
      key: 'PASSWORD',
      kind: 'secret',
      value: '••••••••',
      secret: true,
      isDefault: false,
      defaultValue: null,
      required: true,
      options: [],
      label: 'Password',
    },
    {
      key: 'SLOW_QUERY_LOG',
      kind: 'bool',
      value: 'true',
      secret: false,
      isDefault: false,
      defaultValue: 'false',
      required: false,
      options: [],
      label: 'Slow query log',
    },
  ],

  // The detail sheet's headline: how a project reaches this service, from the
  // host and from inside the network.
  service_connection: {
    service: 'mysql-8-4',
    kind: 'mysql',
    fromHost: { uri: 'mysql://stackvo@127.0.0.1:3306/stackvo', host: '127.0.0.1', port: 3306 },
    fromContainer: {
      uri: 'mysql://stackvo@stackvo-mysql-8-4:3306/stackvo',
      host: 'stackvo-mysql-8-4',
      port: 3306,
    },
    masked: true,
    passwordKey: 'MYSQL_PASSWORD',
  },

  db_targets: [],
  db_snapshots: [],
  db_instances: [],
  service_db_clients: [],

  instance_list: [
    {
      id: 'mysql-8-4',
      service: 'mysql',
      version: '8.4',
      enabled: true,
      primary: true,
      container: 'stackvo-mysql-8-4',
      aliases: ['mysql', 'stackvo-mysql-8-4'],
      ports: { server: 3306 },
      packagePresent: true,
    },
    {
      id: 'redis-8-0',
      service: 'redis',
      version: '8.0',
      enabled: true,
      primary: true,
      container: 'stackvo-redis-8-0',
      aliases: ['redis', 'stackvo-redis-8-0'],
      ports: { server: 6379 },
      packagePresent: true,
    },
  ],

  container_stats: {
    cpuPercent: 18.4,
    memoryUsed: 412_090_368,
    memoryLimit: 2_147_483_648,
    memoryPercent: 19.2,
    netRx: 48_234_496,
    netTx: 12_058_624,
  },
  // `t` is epoch seconds and the page draws them in order, so a flat list of
  // identical samples would be a flat line — this walks.
  container_stats_history: Array.from({ length: 48 }, (_, i) => ({
    t: 1_756_598_400 + i * 1800,
    cpu: 12 + Math.round(Math.sin(i / 3.1) * 8 + i / 9) / 1,
    memory: 17 + Math.round(Math.cos(i / 4.7) * 4 + i / 14),
  })),

  quick_commands: [
    {
      id: 'artisan',
      display: 'php artisan',
      about: 'Laravel’s own command line, inside the project container.',
      interactive: true,
      because: 'artisan',
      declared: false,
    },
    {
      id: 'composer',
      display: 'composer',
      about: 'Dependency manager for PHP.',
      interactive: false,
      because: 'composer.json',
      declared: false,
    },
    {
      id: 'mysql',
      display: 'mysql',
      about: 'A client shell in the database this project is wired to.',
      interactive: true,
      because: 'mysql@8.4',
      declared: false,
      instance: 'mysql-8-4',
    },
  ],

  xdebug_status: {
    supported: true,
    enabled: false,
    active: false,
    needsRebuild: false,
    running: true,
    port: 9003,
    mode: 'debug',
    ideKey: 'STACKVO',
    serverName: 'shop',
    hostPath: '/Users/dev/StackVo/projects/shop',
    containerPath: '/var/www/html',
    phpVersion: '8.4',
    peclVersion: '3.4.1',
    overlayPath: 'docker/php/xdebug.ini',
  },

  mail_messages: [
    {
      id: 'm-3',
      from: 'orders@shop.loc',
      to: ['dev@stackvo.loc'],
      cc: [],
      bcc: [],
      replyTo: [],
      subject: 'Order #10428 confirmed',
      date: '2026-09-01T09:14:00Z',
      snippet: 'Thanks — your order is on its way. Track it any time from your account.',
      read: false,
    },
    {
      id: 'm-2',
      from: 'no-reply@shop.loc',
      to: ['dev@stackvo.loc'],
      cc: [],
      bcc: [],
      replyTo: [],
      subject: 'Password reset requested',
      date: '2026-09-01T08:02:00Z',
      snippet: 'Somebody asked to reset this password. The link expires in an hour.',
      read: false,
    },
    {
      id: 'm-1',
      from: 'queue@shop.loc',
      to: ['ops@stackvo.loc'],
      cc: [],
      bcc: [],
      replyTo: [],
      subject: 'Nightly export finished',
      date: '2026-08-31T23:41:00Z',
      snippet: '4,812 rows written in 38 seconds.',
      read: true,
    },
  ],
  mail_relay_get: {
    enabled: false,
    host: '',
    port: 587,
    username: '',
    security: 'starttls',
    from: '',
    allowedRecipients: [],
    hasPassword: false,
    keystore: false,
  },

  debug_bridge_events: { total: 0, events: [] },

  // Empty is the true answer on a machine with nothing to adopt, and an empty
  // array is a different screen from `null`: one says "nothing found", the
  // other never stops loading.
  project_adoptable: [],
  imports_scan: [],
  worktree_list: [],
  handover_preview: {
    pending: false,
    migrated: true,
    instances: [],
    notes: [],
    blockers: [],
    backup: false,
    missing: [],
  },

  // ---- the settings panes ------------------------------------------------
  // Six of these are not decoration: `null` reached a component that read a
  // field off it, and the pane threw rather than drew. Preferences wanted
  // `terminals`, certificates `length`, secrets `available`, the agents pane
  // `binary` and `clients`, the local API `running`.
  apps_available: {
    terminals: [
      { id: 'terminal', name: 'Terminal', icon: 'mdi-console', available: true, default: true },
      { id: 'iterm2', name: 'iTerm2', icon: 'mdi-console-line', available: true, default: false },
    ],
    editors: [
      {
        id: 'vscode',
        name: 'Visual Studio Code',
        icon: 'mdi-microsoft-visual-studio-code',
        available: true,
        default: true,
      },
      {
        id: 'phpstorm',
        name: 'PhpStorm',
        icon: 'mdi-language-php',
        available: true,
        default: false,
      },
    ],
    browsers: [
      { id: 'default', name: 'System default', icon: 'mdi-web', available: true, default: true },
      {
        id: 'chrome',
        name: 'Google Chrome',
        icon: 'mdi-google-chrome',
        available: true,
        default: false,
      },
    ],
  },

  cert_plan: {
    add: [],
    remove: [],
    domains: ['stackvo.loc', '*.stackvo.loc', 'shop.loc', 'storefront.loc'],
    covered: ['stackvo.loc', '*.stackvo.loc', 'shop.loc', 'storefront.loc'],
    rejected: [],
    changed: false,
    certPath: '/Users/dev/Library/Application Support/StackVo/certs/stackvo.pem',
    installCa: false,
    reloaded: false,
  },

  secrets_status: {
    available: true,
    keys: [
      { key: 'MYSQL_PASSWORD', moved: true, resolvable: true, set: true },
      { key: 'MAIL_PASSWORD', moved: false, resolvable: true, set: true },
    ],
  },

  agents_status: {
    binary: '/usr/local/bin/stackvo-agent',
    source: 'path',
    root: '/Users/dev/StackVo',
    clients: [
      {
        id: 'claude',
        label: 'Claude Code',
        path: '~/.claude/settings.json',
        present: true,
        exists: true,
        parseable: true,
        command: 'stackvo agent',
        current: true,
      },
      {
        id: 'cursor',
        label: 'Cursor',
        path: '~/.cursor/mcp.json',
        present: false,
        exists: false,
        parseable: true,
        command: null,
        current: false,
      },
    ],
  },
  rules_status: [
    {
      id: 'workspace',
      label: 'Workspace rules',
      scope: 'workspace',
      path: 'AGENTS.md',
      exists: true,
      installed: true,
      current: true,
    },
    {
      id: 'global',
      label: 'Global rules',
      scope: 'global',
      path: '~/.stackvo/AGENTS.md',
      exists: false,
      installed: false,
      current: false,
    },
  ],

  websurface_status: { running: false, address: null, tools: [] },

  // `DEFAULT_STAGE`'s certificate answer is short two fields the pane reads —
  // and reads without a guard: `certs.rejected.length` threw on every shot of
  // this pane. The field is real (`certs.rs` returns it); what is missing is its
  // line in `contracts/ipc.json`, which is why the stage was written without it.
  cert_status: {
    ...DEFAULT_STAGE.cert_status,
    stale: false,
    rejected: [],
    error: null,
    trust: [
      { id: 'system', trusted: true },
      { id: 'firefox', trusted: true },
    ],
    certPath: '/Users/dev/Library/Application Support/StackVo/certs/stackvo.pem',
    keyPath: '/Users/dev/Library/Application Support/StackVo/certs/stackvo-key.pem',
    notAfter: 1_788_220_800,
    daysRemaining: 364,
    expired: false,
    required: ['stackvo.loc', '*.stackvo.loc', 'shop.loc', 'storefront.loc'],
  },

  // Richer than `DEFAULT_STAGE`'s empty list, and `ready` stays true: a false
  // one opens `RequirementsGate` over the whole app and every picture becomes
  // that screen.
  preflight: {
    os: 'macos',
    ready: true,
    requirements: [
      { id: 'workspace', state: 'ok', detail: '/Users/dev/StackVo', fixable: false },
      { id: 'engine', state: 'ok', detail: 'Docker 29.7.2', fixable: false },
      { id: 'compose', state: 'ok', detail: 'v2.34.0', fixable: false },
      { id: 'network', state: 'ok', detail: 'stackvo', fixable: false },
      { id: 'mkcert', state: 'ok', detail: 'v1.4.4', fixable: false },
    ],
  },

  doctor: {
    preflight: {
      os: 'macos',
      ready: true,
      requirements: [
        { id: 'workspace', state: 'ok', detail: '/Users/dev/StackVo', fixable: false },
        { id: 'engine', state: 'ok', detail: 'Docker 29.7.2', fixable: false },
        { id: 'compose', state: 'ok', detail: 'v2.34.0', fixable: false },
        { id: 'network', state: 'ok', detail: 'stackvo', fixable: false },
        { id: 'mkcert', state: 'ok', detail: 'v1.4.4', fixable: false },
      ],
    },
    ports: [
      {
        port: 80,
        requiredBy: 'proxy',
        state: 'ok',
        process: 'stackvo-proxy',
        pid: 4821,
        ours: true,
      },
      {
        port: 443,
        requiredBy: 'proxy',
        state: 'ok',
        process: 'stackvo-proxy',
        pid: 4821,
        ours: true,
      },
      {
        port: 3306,
        requiredBy: 'mysql',
        state: 'ok',
        process: 'stackvo-mysql-8-4',
        pid: 4907,
        ours: true,
      },
    ],
    hostsMissing: [],
    dns: null,
    generated: { state: 'ok', detail: null },
    space: {
      images: { total: 14, inUse: 9, unused: 5, size: 7_516_192_768 },
      volumes: { total: 6, inUse: 4, unused: 2, size: 2_147_483_648 },
    },
    extensions: [],
  },

  hosts_overview: {
    entries: [
      { ip: '127.0.0.1', domain: 'stackvo.loc', configured: true, managedByStackvo: true },
      { ip: '127.0.0.1', domain: 'shop.loc', configured: true, managedByStackvo: true },
      { ip: '127.0.0.1', domain: 'storefront.loc', configured: true, managedByStackvo: true },
    ],
    stale: [],
  },
  dns_status: {
    mechanism: 'resolver',
    writable: true,
    suffix: 'loc',
    tld: 'loc',
    port: 5354,
    listening: false,
    tcp: false,
    file: '/etc/resolver/loc',
  },
  routes_list: [],
  projects_idle: [],
  templates_list: [],
  server_config_get: '',

  machine_commands: {
    path: '/Users/dev/StackVo/commands.json',
    exists: false,
    commands: {},
    problems: [],
  },
  audit_trail: { entries: [], total: 0, unreadable: 0 },
  logs_info: {
    directory: '/Users/dev/Library/Logs/StackVo',
    newestFile: 'stackvo-2026-09-01.log',
    totalBytes: 1_048_576,
  },

  tooling_status: {
    binDir: '/Users/dev/.stackvo/bin',
    onPath: true,
    currentShell: 'zsh',
    own: [
      {
        id: 'stackvo',
        about: 'The command line this app drives.',
        built: '0.1.0',
        linked: '/Users/dev/.stackvo/bin/stackvo',
      },
    ],
    shells: [
      {
        id: 'zsh',
        label: 'zsh',
        path: '/bin/zsh',
        exists: true,
        installed: true,
        current: true,
        line: 'source ~/.stackvo/completions/stackvo.zsh',
      },
      {
        id: 'bash',
        label: 'bash',
        path: '/bin/bash',
        exists: true,
        installed: false,
        current: false,
        line: null,
      },
    ],
    tools: [
      {
        id: 'docker',
        label: 'Docker',
        program: 'docker',
        why: 'Runs every container.',
        source: 'system',
        version: '29.7.2',
        path: '/usr/local/bin/docker',
        offers: null,
        publisher: null,
        availableHere: true,
      },
      {
        id: 'mkcert',
        label: 'mkcert',
        program: 'mkcert',
        why: 'Issues the local certificate.',
        source: 'managed',
        version: 'v1.4.4',
        path: '/Users/dev/.stackvo/bin/mkcert',
        offers: null,
        publisher: null,
        availableHere: true,
      },
    ],
  },

  catalog_get: {
    runtimes: [
      { id: 'php', versions: ['8.4', '8.3', '8.2'], default: '8.4', available: true },
      { id: 'node', versions: ['22', '20'], default: '22', available: true },
    ],
    servers: ['nginx', 'apache'],
    defaultServer: 'nginx',
    phpExtensions: [
      { name: 'pdo_mysql', install: 'docker-php-ext-install pdo_mysql', inDefaultSet: true },
      { name: 'redis', install: 'pecl install redis', inDefaultSet: true },
      { name: 'intl', install: 'docker-php-ext-install intl', inDefaultSet: false },
    ],
  },
  env_defaults: {},
};

/** One catalogue row, in `MarketPackage` shape, so the market page has a market. */
function marketPackage(service, category, name, summary, versions) {
  return {
    service,
    category,
    name: { en: name, tr: name },
    summary: { en: summary, tr: summary },
    capabilities: [],
    keywords: [],
    multiple: false,
    maintainer: 'StackVo',
    versions: versions.map((v) => ({
      version: v.version,
      recommended: v.version === versions[0].version,
      support: 'supported',
      eolDate: null,
      sizeBytes: 268_435_456,
      installed: v.installed,
      inUse: v.inUse,
      overridden: 0,
    })),
  };
}

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const OUT = join(ROOT, 'docs', 'screenshots');
const PORT = 4183; // Not 4173: the Playwright suite owns that one, and a run of
// each at the same time would otherwise shoot the other's server.
const ORIGIN = `http://localhost:${PORT}`;

// Wider and taller than the 1280x820 `tauri.conf.json` opens with, deliberately:
// that default is the smallest window anybody uses, and a picture taken in it
// shows every pane at its most cramped. This is the size a maximised window on
// a laptop gets, and the vertical is the half a person capturing by hand loses.
const VIEWPORT = { width: 1600, height: 1000 };

// One theme, and it is light. Two themes is two files per page to maintain and
// one of them is always the stale one.
const THEME = 'light';

// Commands every page fires and no picture depends on: the shell's own
// housekeeping. Listed so the "nobody answered this" report below is about the
// page's own questions rather than about the shell's.
const IGNORED = new Set(['locale_packs', 'tray_relabel', 'crash_reports']);

/**
 * Every page worth a picture, in the order a person meets them.
 *
 * `ready` is a locator that only resolves once the page has its own content on
 * it — not the shell's. Shooting on `load` gives a rail and an empty pane,
 * because every view here fetches before it draws.
 */
/**
 * The detail page's own rail, one picture per entry.
 *
 * Ten sections and none of them is in the URL: `section` is a ref inside
 * `ProjectDetail.vue`, so the only way to a pane is to click its rail entry —
 * which is also the only honest way to shoot it, because clicking is what a
 * person does. The labels are the English ones the rail renders; a locale
 * change here would break the click rather than take a wrong picture, which is
 * the failure worth having.
 *
 * `.nav-item` and not the text alone: "Logs", "Container" and "Terminal" are
 * all words the panes themselves use, and a bare text locator picks whichever
 * came first in the tree.
 */
const SECTIONS = [
  ['indicator', null], // The default pane; no click, and the file every other name hangs off.
  ['configuration', 'Configuration'],
  ['container', 'Container'],
  ['jobs', 'Jobs'],
  ['terminal', 'Terminal'],
  ['logs', 'Logs'],
  ['debugging', 'Debugging'],
  ['runtime', 'Runtime settings'],
  ['release', 'Production image'],
  ['agent', 'AI'],
];

function projectSections() {
  return SECTIONS.map(([key, label]) => ({
    name: key === 'indicator' ? 'project-detail' : `project-detail-${key}`,
    path: '/#/projects/shop',
    // The detail page titles itself with the project's name, not its domain —
    // waiting on `shop.loc` here timed out on every run and shot the page mid-load.
    ready: (page) => page.getByRole('main').getByText('shop', { exact: true }).first(),
    act: label
      ? async (page) => {
          await page
            .getByRole('main')
            .locator('.nav-item')
            // Anchored, and that is not defensiveness: `hasText` matches a
            // case-insensitive SUBSTRING, so 'AI' selected "Cont(ai)ner" and the
            // AI tab's picture was the container pane's, with the rail to prove it.
            .filter({ hasText: new RegExp(`^\\s*${label}\\s*$`) })
            .first()
            .click();
          // The panes fetch on activation, and several draw a chart once they
          // have. Long enough for that, short enough to keep ten of these cheap.
          await page.waitForTimeout(900);
        }
      : null,
  }));
}

/**
 * The settings page's own rail, one picture per pane.
 *
 * Unlike the detail page, this one is addressable: `Settings.vue` reads
 * `route.query.tab` on mount and selects the pane from it, so these are
 * navigations rather than clicks — no rail locator, and nothing to go wrong in
 * the way the AI label did. The keys are `SECTIONS` in that file, in its order;
 * `appearance` is the default and keeps the plain `settings` name.
 */
const SETTINGS_TABS = [
  'appearance',
  'localisation',
  'preferences',
  'workspace',
  'domain',
  'certificates',
  'servers',
  'catalogue',
  'php',
  'secrets',
  'agents',
  'localApi',
  'tooling',
  'machineCommands',
  'doctor',
  'audit',
  'about',
];

function settingsTabs() {
  return SETTINGS_TABS.map((key) => ({
    name: key === 'appearance' ? 'settings' : `settings-${key}`,
    path: `/#/settings?tab=${key}`,
    ready: (page) => page.getByRole('main'),
    // The pane is chosen before anything is fetched, but it still fetches: a
    // shot taken on `ready` alone catches half of them mid-load.
    act: (page) => page.waitForTimeout(700),
  }));
}

const PAGES = [
  {
    name: 'dashboard',
    path: '/#/',
    ready: (page) => page.getByRole('main').getByText('Dashboard'),
  },
  {
    name: 'projects',
    path: '/#/projects',
    ready: (page) => page.getByRole('main').getByRole('button', { name: 'shop.loc' }),
  },
  ...projectSections(),
  { name: 'market', path: '/#/market', ready: (page) => page.getByRole('main') },
  { name: 'logs', path: '/#/logs', ready: (page) => page.getByRole('main') },
  { name: 'dumps', path: '/#/dumps', ready: (page) => page.getByRole('main') },
  { name: 'mail', path: '/#/mail', ready: (page) => page.getByRole('main') },
  ...settingsTabs(),

  // ---- the screens that are not pages -------------------------------------
  // Sheets and drawers, opened by the control that opens them. Each is a place
  // a person spends real time and none of them has an address: they live on a
  // page and are a picture of that page mid-use.
  {
    name: 'market-service-detail',
    path: '/#/market',
    ready: (page) => page.getByRole('main').getByText('Instances'),
    act: (page) => openOverlay(page, 'Detail'),
  },
  {
    name: 'market-instance-settings',
    path: '/#/market',
    ready: (page) => page.getByRole('main').getByText('Instances'),
    act: (page) => openOverlay(page, 'Settings'),
  },
  {
    name: 'market-add-instance',
    path: '/#/market',
    ready: (page) => page.getByRole('main').getByText('Instances'),
    act: async (page) => {
      // Two panels deep: the category, then the service. `Add instance` is on
      // the version row underneath and does not exist until both are open.
      const main = page.getByRole('main');
      await main.getByText('database', { exact: true }).first().click();
      await page.waitForTimeout(400);
      await main.getByText('MySQL', { exact: true }).first().click();
      await page.waitForTimeout(400);
      await openOverlay(page, 'Add instance');
    },
  },
  {
    name: 'project-new',
    path: '/#/projects',
    ready: (page) => page.getByRole('main').getByRole('button', { name: 'shop.loc' }),
    act: (page) => openOverlay(page, 'New project'),
  },
];

/**
 * Click the control that opens an overlay, and wait for the overlay itself.
 *
 * Waiting on `.v-overlay--active` rather than on a fixed pause: these all fetch
 * when they open, and a sheet caught at 300ms is a picture of its own skeleton.
 */
async function openOverlay(page, name) {
  // The first match is not always the one to click: the project rail and the
  // page header both carry "New project", and the rail's is disabled while the
  // rail is collapsed. Clicked by enabledness rather than by order.
  const candidates = page.getByRole('button', { name, exact: true });
  const count = await candidates.count();
  for (let i = 0; i < count; i += 1) {
    const button = candidates.nth(i);
    if (!(await button.isEnabled())) continue;
    await button.click();
    // The sheets are overlays; the new-project drawer is a navigation drawer and
    // never matches one. Waited for where it applies and timed out quietly where
    // it does not — the pause below is what both of them actually need.
    await page
      .locator('.v-overlay-container .v-overlay')
      .first()
      .waitFor({ state: 'visible', timeout: 6_000 })
      .catch(() => {});
    await page.waitForTimeout(1_000);
    return;
  }
  throw new Error(`no enabled "${name}" button to open (${count} matched)`);
}

const argv = process.argv.slice(2);
const wanted = argv.reduce((acc, value, index) => {
  if (argv[index - 1] === '--page') acc.push(value);
  return acc;
}, []);
const shouldBuild = !argv.includes('--no-build');

/** Run a command to completion, inheriting stdio so its failure is readable. */
function run(command, args) {
  return new Promise((ok, fail) => {
    const child = spawn(command, args, { cwd: ROOT, stdio: 'inherit', shell: false });
    child.on('exit', (code) =>
      code === 0 ? ok() : fail(new Error(`${command} ${args.join(' ')} exited ${code}`))
    );
  });
}

/** The preview server, and a promise that resolves when it answers. */
async function serve() {
  const child = spawn(
    'npx',
    ['vite', 'preview', '--port', String(PORT), '--strictPort', '--host', 'localhost'],
    { cwd: ROOT, stdio: ['ignore', 'pipe', 'inherit'], shell: false }
  );

  const deadline = Date.now() + 60_000;
  for (;;) {
    if (Date.now() > deadline) {
      child.kill();
      throw new Error(`vite preview did not answer on ${ORIGIN} within 60s`);
    }
    try {
      const response = await fetch(ORIGIN, { redirect: 'manual' });
      if (response.ok || response.status === 304) break;
    } catch {
      // Not up yet. The loop's deadline is the failure, not this.
    }
    await sleep(250);
  }

  return child;
}

/**
 * One page, one theme, one file.
 *
 * The context is new per shot rather than reused: `colorScheme` is a context
 * option, and a page that changes theme after boot animates the change — which
 * is a screenshot of a transition, taken at whatever moment the shutter fell.
 */
async function shoot(browser, spec) {
  const context = await browser.newContext({
    viewport: VIEWPORT,
    deviceScaleFactor: 2, // Retina. A 1x screenshot of a 2x interface reads as blurry type.
    colorScheme: THEME,
    reducedMotion: 'reduce', // Otherwise the shutter catches a card mid-fade.
  });

  const page = await context.newPage();
  const noise = [];
  page.on('console', (message) => {
    if (message.type() === 'error') noise.push(message.text());
  });
  page.on('pageerror', (error) => noise.push(`threw: ${error.message}`));

  await stage(page, STAGE);
  await page.goto(`${ORIGIN}${spec.path}`, { waitUntil: 'load' });

  try {
    await spec.ready(page).first().waitFor({ state: 'visible', timeout: 15_000 });
  } catch {
    noise.push('the page never showed what it was waited on for');
  }

  // A section of the page rather than a page of its own: clicked, because the
  // rail's selection lives in a ref rather than in the route. A click that
  // cannot be made is reported and shot anyway — the picture then shows what
  // the page really did, which is the point of taking it.
  if (spec.act) {
    try {
      await spec.act(page);
    } catch (error) {
      noise.push(`the shot could not be set up: ${error.message}`);
    }
  }

  // Boot fires a dozen optional calls and the last of them still paints. A
  // fixed pause here rather than `networkidle`: nothing on this page talks to
  // the network at all, so idle is true immediately and means nothing.
  await page.waitForTimeout(600);

  const file = join(OUT, `${spec.name}.png`);
  await page.screenshot({ path: file, animations: 'disabled' });

  // What the page asked that nobody answered. `null` is a legal reply, so an
  // unstaged command does not fail — it draws a spinner that never resolves, or
  // an empty state, in a picture meant to show the page doing its work. Named
  // here rather than found by looking at the file afterwards.
  const answered = new Set([...Object.keys(DEFAULT_STAGE), ...Object.keys(STAGE)]);
  const unanswered = [...new Set((await callsOf(page)).map((c) => c.cmd))].filter(
    (cmd) => !answered.has(cmd) && !IGNORED.has(cmd) && !cmd.startsWith('plugin:')
  );
  await context.close();

  return { file, noise, unanswered };
}

const specs = wanted.length ? PAGES.filter((p) => wanted.includes(p.name)) : PAGES;
if (!specs.length) {
  console.error(`No such page. Known: ${PAGES.map((p) => p.name).join(', ')}`);
  process.exit(2);
}

if (shouldBuild) await run('npm', ['run', 'build']);
if (!existsSync(join(ROOT, 'dist', 'index.html'))) {
  console.error('dist/index.html is missing — run without --no-build.');
  process.exit(2);
}

await mkdir(OUT, { recursive: true });

const server = await serve();
const browser = await chromium.launch();
let complained = false;

try {
  for (const spec of specs) {
    const { file, noise, unanswered } = await shoot(browser, spec);
    const size = (await stat(file)).size;
    console.log(`${file.replace(`${ROOT}/`, '')}  ${(size / 1024).toFixed(0)} KB`);
    if (unanswered.length) console.log(`  ? unstaged: ${unanswered.join(', ')}`);
    for (const line of noise) {
      complained = true;
      console.log(`  ! ${line}`);
    }
  }
} finally {
  await browser.close();
  server.kill();
}

const written = (await readdir(OUT)).filter((f) => f.endsWith('.png'));
console.log(
  `\n${written.length} file(s) in docs/screenshots/ at ${VIEWPORT.width}×${VIEWPORT.height}@2x, ${THEME}.`
);

// A console error on a page is not a reason to refuse the picture — it is a
// reason for the person looking at it to know the page complained while it was
// taken, which is why the lines are printed rather than swallowed.
if (complained) console.log('Some pages complained while being shot; see the ! lines above.');
