import { computed, ref } from 'vue';
import { api } from '@/lib/ipc';

/**
 * What the app can build: which runtime versions exist, and which servers.
 *
 * Read from the binary rather than typed into a list here, so a PHP release
 * added to the catalog shows up without a second edit. These are *not*
 * settings — editing one could only ever select something the app cannot
 * build, because a generator either exists for a runtime or it does not.
 *
 * ## Module-scoped, like `useCertificates`
 *
 * Two panes read it — the runtime defaults and the server picker — and they
 * need the same answer from one fetch. A per-call instance would send the same
 * request twice and let the two panes disagree while one of them was still in
 * flight.
 */

const catalog = ref(null);
const loading = ref(false);

export function useCatalog() {
  /**
   * Best effort. A catalog that cannot be read is a narrower list of choices,
   * not a pane that refuses to open — see `itemsFor`, which keeps the current
   * value visible either way.
   */
  async function load() {
    if (loading.value) return;
    loading.value = true;
    try {
      catalog.value = await api.catalogGet().catch(() => null);
    } finally {
      loading.value = false;
    }
  }

  const servers = computed(() => catalog.value?.servers ?? []);

  const versionsFor = (id) => catalog.value?.runtimes?.find((r) => r.id === id)?.versions ?? [];

  /** For tests, which must not inherit another test's fetch. */
  function reset() {
    catalog.value = null;
    loading.value = false;
  }

  return { catalog, servers, versionsFor, load, reset };
}

/** The version a new project of each runtime starts on. */
export const RUNTIME_DEFAULTS = [
  { id: 'python', key: 'SUPPORTED_LANGUAGES_PYTHON_DEFAULT', icon: 'mdi-language-python' },
  { id: 'go', key: 'SUPPORTED_LANGUAGES_GO_DEFAULT', icon: 'mdi-language-go' },
  { id: 'ruby', key: 'SUPPORTED_LANGUAGES_RUBY_DEFAULT', icon: 'mdi-language-ruby' },
  { id: 'rust', key: 'SUPPORTED_LANGUAGES_RUST_DEFAULT', icon: 'mdi-language-rust' },
  { id: 'node', key: 'SUPPORTED_LANGUAGES_NODEJS_DEFAULT', icon: 'mdi-nodejs' },
];

/**
 * The `.env` list each version picker draws from, and why it is editable.
 *
 * The pickers above read the catalog the binary was built with, and that list
 * goes stale the moment the world ships a release: `SUPPORTED_LANGUAGES_GO_VERSIONS`
 * stopped at 1.23, Ruby at 3.3, Node at 23, Rust at 1.84. Every one of those is
 * a version somebody could build against today and could not select here, and
 * the only way out was **waiting for a release of this application** — for a
 * number in a list.
 *
 * So the list is a setting, and this is the shape of it. `.env` overrides the
 * embedded default (`config::SETTINGS`), `build_catalog` reads it back through
 * `SUPPORTED_LANGUAGES_{KEY}_VERSIONS`, and the picker offers what came out.
 * Adding a version the registry does not have fails at build time with the
 * engine's own message, which is the same bargain `PHP_DEFAULT_TOOLS` makes.
 *
 * Separate from `RUNTIME_DEFAULTS` because the two answer different questions —
 * *which* versions exist against *which one* a new project starts on — and
 * because PHP is in this list and not in that one: its default has a picker of
 * its own, one group down.
 *
 * `nodejs` and not `node`: `.env` spells it the long way and the manifest
 * spells it the short way. See C-01, and `build_catalog`, which translates.
 */
export const VERSION_LISTS = [
  { id: 'php', key: 'SUPPORTED_LANGUAGES_PHP_VERSIONS', icon: 'mdi-language-php' },
  { id: 'python', key: 'SUPPORTED_LANGUAGES_PYTHON_VERSIONS', icon: 'mdi-language-python' },
  { id: 'go', key: 'SUPPORTED_LANGUAGES_GO_VERSIONS', icon: 'mdi-language-go' },
  { id: 'ruby', key: 'SUPPORTED_LANGUAGES_RUBY_VERSIONS', icon: 'mdi-language-ruby' },
  { id: 'rust', key: 'SUPPORTED_LANGUAGES_RUST_VERSIONS', icon: 'mdi-language-rust' },
  { id: 'nodejs', key: 'SUPPORTED_LANGUAGES_NODEJS_VERSIONS', icon: 'mdi-nodejs' },
];

/**
 * The choices a select offers for one `.env` key.
 *
 * **The current value is always in the list**, even when the catalog does not
 * know it. A select whose only item is missing renders blank, which reads as
 * data loss rather than as "this version is no longer shipped" — and the two
 * ways to get there are an unreadable catalog and a value written by an older
 * build.
 */
export function useVersionChoices(envEditor) {
  const { effective } = envEditor;
  const { servers, versionsFor, load, catalog } = useCatalog();

  const itemsFor = (key, versions) => {
    const current = effective(key);
    const list = versions?.length ? [...versions] : [];
    if (current && !list.includes(current)) list.unshift(current);
    return list;
  };

  const phpVersions = computed(() =>
    itemsFor('SUPPORTED_LANGUAGES_PHP_DEFAULT', versionsFor('php'))
  );
  const nodeVersions = computed(() => itemsFor('PHP_TOOL_NODEJS_VERSION', versionsFor('node')));
  const serverChoices = computed(() => itemsFor('SUPPORTED_SERVERS_DEFAULT', servers.value));

  const runtimeItems = (runtime) => itemsFor(runtime.key, versionsFor(runtime.id));

  return {
    catalog,
    servers,
    itemsFor,
    phpVersions,
    nodeVersions,
    serverChoices,
    runtimeItems,
    loadCatalog: load,
  };
}
