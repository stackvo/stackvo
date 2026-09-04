/**
 * The worktree screen's half of the staged machine.
 *
 * Issue #101 asked for "a fixture that prepares a real git tree" behind the
 * worktree picture, and the honest answer is that the screen never sees one.
 * `WorktreePane.vue` asks the boundary one question — `worktree_support` — and
 * draws the answer; git, the worktrees file and the checkouts are all on the
 * Rust side of that call. A throwaway repository on disk would be a tree that
 * `git worktree list` could read and Chromium could not, because the browser
 * the tool shoots in has no `invoke` to reach it with. So the fixture is the
 * answer itself, in the shape `contracts/ipc.json` declares for it, on the
 * same seam every other picture is staged through.
 *
 * ## Why it is a file of its own
 *
 * `tools/screenshots.mjs` runs on import — it builds, serves and shoots — so
 * nothing in it can be tested without taking thirty-nine pictures. What is
 * here is the part a test can hold still: two worktrees, the branches they
 * took, and the two commands that have to agree about them. `worktree_list`
 * is what the projects page reads to say "branch of shop" on a row, and
 * `worktree_support.worktrees` is what the detail page lists; built from one
 * array so the two pages cannot show different branches of the same project.
 */

/** The parent, which is the first project on every other picture. */
export const PARENT = { name: 'shop', domain: 'shop.loc', branch: 'main' };

/** The instance a branch's database is made on — `STAGE.instance_list[0]`. */
const MYSQL = {
  id: 'mysql-8-4',
  service: 'mysql',
  version: '8.4',
  kind: 'mysql',
  container: 'stackvo-mysql-8-4',
  enabled: true,
  running: true,
};

/**
 * One `WorktreeRow`, field for field from `commands::WorktreeRow` — the
 * record's own fields flattened in, then what is true right now beside them.
 *
 * The derivations are spelled out rather than computed, on purpose: `worktree.rs`
 * owns them (a slug folds `/` to `-`, a domain is a subdomain of the parent's),
 * and a second implementation here would be the drift the pane's own comment
 * warns about. The test checks the spelled-out values obey the rules instead.
 */
function worktree(branch, extra) {
  const slug = branch.replace(/[^a-z0-9]+/gi, '-').toLowerCase();
  return {
    name: `${PARENT.name}-${slug}`,
    parent: PARENT.name,
    branch,
    domain: `${slug}.${PARENT.domain}`,
    path: `/Users/dev/StackVo/projects/${PARENT.name}-${slug}`,
    database: null,
    env: {},
    createdAt: '2026-08-31T14:20:00Z',
    exists: true,
    dirty: false,
    orphaned: false,
    isolated: false,
    expired: false,
    ...extra,
  };
}

/**
 * Two rows, and they are the two states the pane draws differently.
 *
 * The first is a branch somebody keeps: its own database copied from the
 * workspace's, a login of its own, and work in it that is not committed — the
 * `dirty` chip is the one that matters on a removal. The second is a sandbox:
 * no database, a duration, and the chip saying how long it has. Nothing here
 * is orphaned or expired, because a picture of the feature is not a picture
 * of it having gone wrong.
 */
export const WORKTREES = [
  worktree('feature/checkout', {
    database: { instance: MYSQL.id, name: 'stackvo_feature_checkout', seededFrom: 'stackvo' },
    env: { APP_ENV: 'branch' },
    dirty: true,
    isolated: true,
  }),
  worktree('fix/1042-cart-total', {
    createdAt: '2026-09-01T08:05:00Z',
    expiresAt: '2026-09-01T16:05:00Z',
    remainingMinutes: 285,
  }),
];

/**
 * The branches the create form offers, newest commit first as `git` lists them.
 *
 * The two above stay in the list and are marked: the pane disables them rather
 * than hiding them, so the picture shows the disabled rows too.
 */
export const BRANCHES = [
  { name: 'fix/1042-cart-total', checkedOut: true, current: false },
  { name: 'feature/checkout', checkedOut: true, current: false },
  { name: PARENT.branch, checkedOut: true, current: true },
  { name: 'feature/search', checkedOut: false, current: false },
  { name: 'chore/php-8-5', checkedOut: false, current: false },
];

/** What the form previews for the branch the shot types into it. */
export const PLANNED_BRANCH = 'feature/search';

/**
 * The commands the worktree screen asks, answered.
 *
 * `worktree_plan` is the form's preview and it has no side effects, so the
 * stage's one static answer is right for every keystroke — the shot types
 * `PLANNED_BRANCH` and the plan is for that branch.
 */
export const WORKTREE_STAGE = {
  worktree_list: WORKTREES,
  worktree_support: {
    gitAvailable: true,
    repository: true,
    linked: false,
    record: null,
    isolated: false,
    grantArgs: [],
    effectiveEnv: null,
    domain: PARENT.domain,
    currentBranch: PARENT.branch,
    branches: BRANCHES,
    instances: [MYSQL],
    reason: null,
    worktrees: WORKTREES,
    auth: null,
  },
  worktree_plan: {
    parent: PARENT.name,
    branch: PLANNED_BRANCH,
    newBranch: false,
    name: 'shop-feature-search',
    path: '/Users/dev/StackVo/projects/shop-feature-search',
    domain: 'feature-search.shop.loc',
    database: null,
    warnings: [],
    refused: null,
    possible: true,
  },
};
