import { computed, ref } from 'vue';
import { api, asList } from '@/lib/ipc';

/**
 * The presets the frequency dropdown offers, and the expression each one is.
 *
 * A preset is a *spelling* of a cron expression and never a second way to
 * store one: the manifest holds the expression, and a preset that matched
 * nothing in the file would be a dropdown that forgets what the reader chose.
 * `describe` runs the mapping backwards for exactly that reason.
 */
export const PRESETS = [
  { key: 'everyMinute', cron: '* * * * *' },
  { key: 'every5', cron: '*/5 * * * *' },
  { key: 'every15', cron: '*/15 * * * *' },
  { key: 'every30', cron: '*/30 * * * *' },
  { key: 'hourly', cron: '0 * * * *' },
  { key: 'daily', cron: '0 0 * * *' },
  { key: 'nightly', cron: '0 3 * * *' },
  { key: 'weekly', cron: '0 0 * * 1' },
  { key: 'monthly', cron: '0 0 1 * *' },
];

/** The preset an expression is, or `null` when it is one somebody wrote. */
export function presetFor(cron) {
  return PRESETS.find((p) => p.cron === (cron ?? '').trim())?.key ?? null;
}

/**
 * The job kinds the form offers, and the argv each one builds.
 *
 * Not stored: the manifest holds an argv, and the kind is inferred back from
 * it. A kind in the file would be a second description of the same command,
 * and the two would eventually disagree.
 */
export const KINDS = ['laravel', 'artisan', 'custom'];

/** Split a typed command into argv. One word, one argument. */
export function words(text) {
  return (text ?? '').trim().split(/\s+/).filter(Boolean);
}

/** The argv a form produces, from the kind and what was typed. */
export function argvFor(kind, text) {
  if (kind === 'laravel') return ['php', 'artisan', 'schedule:run'];
  if (kind === 'artisan') return ['php', 'artisan', ...words(text)];
  return words(text);
}

/** Which kind an existing argv came from, so editing it opens the right form. */
export function kindOf(exec) {
  const argv = asList(exec);
  if (argv.length === 3 && argv[0] === 'php' && argv[1] === 'artisan' && argv[2] === 'schedule:run') {
    return 'laravel';
  }
  if (argv[0] === 'php' && argv[1] === 'artisan') return 'artisan';
  return 'custom';
}

/** What the form should show for an argv, given the kind it was read as. */
export function textOf(exec) {
  const argv = asList(exec);
  return kindOf(argv) === 'artisan' ? argv.slice(2).join(' ') : argv.join(' ');
}

/**
 * One project's scheduled jobs.
 *
 * The whole list is saved at once because that is how it is stored — one value
 * in `stackvo.json` and one generated directory. Editing a row therefore means
 * replacing it in a local copy and saving all of them, which is also what makes
 * "cancel" free: nothing has been written until save.
 */
export function useScheduler(name) {
  const jobs = ref([]);
  const running = ref(false);
  const restarts = ref(null);
  const buildable = ref(false);
  const busy = ref(null);
  const error = ref(null);

  /** A schedule with nothing running it is a list of intentions. */
  const scheduled = computed(() => jobs.value.some((job) => job.enabled) && running.value);

  async function load() {
    try {
      const view = await api.schedulerJobs(name.value);
      jobs.value = asList(view?.jobs);
      running.value = Boolean(view?.running);
      restarts.value = view?.restarts ?? null;
      buildable.value = Boolean(view?.buildable);
      error.value = null;
    } catch (e) {
      error.value = e;
    }
    return jobs.value;
  }

  /** The four fields the backend stores, and nothing the screen added. */
  function stored(list) {
    return list.map((job) => ({
      label: job.label,
      cron: job.cron,
      exec: asList(job.exec),
      enabled: job.enabled !== false,
    }));
  }

  async function save(list, marker = 'save') {
    busy.value = marker;
    error.value = null;
    try {
      const view = await api.schedulerSave(name.value, stored(list));
      jobs.value = asList(view?.jobs);
      running.value = Boolean(view?.running);
      restarts.value = view?.restarts ?? null;
      buildable.value = Boolean(view?.buildable);
      return true;
    } catch (e) {
      error.value = e;
      return false;
    } finally {
      busy.value = null;
    }
  }

  /** Add or replace one job, keyed by the id the backend derived from a label. */
  function upsert(job, replacing = null) {
    const next = jobs.value.slice();
    const at = replacing ? next.findIndex((j) => j.id === replacing) : -1;
    if (at >= 0) next.splice(at, 1, job);
    else next.push(job);
    return save(next, replacing ?? 'new');
  }

  function remove(id) {
    return save(
      jobs.value.filter((job) => job.id !== id),
      id,
    );
  }

  /** Pausing keeps the command, which is the part that took effort to write. */
  function toggleJob(id) {
    return save(
      jobs.value.map((job) => (job.id === id ? { ...job, enabled: !job.enabled } : job)),
      id,
    );
  }

  /** One button for the sidecar, because there is one sidecar. */
  async function toggleScheduler() {
    busy.value = 'scheduler';
    error.value = null;
    try {
      if (running.value) await api.schedulerStop(name.value);
      else await api.schedulerStart(name.value);
      await load();
      return true;
    } catch (e) {
      error.value = e;
      return false;
    } finally {
      busy.value = null;
    }
  }

  async function runNow(id) {
    busy.value = id;
    error.value = null;
    try {
      await api.schedulerRun(name.value, id);
      await load();
      return true;
    } catch (e) {
      error.value = e;
      return false;
    } finally {
      busy.value = null;
    }
  }

  function log(id, lines = 500) {
    return api.schedulerLog(name.value, id, lines);
  }

  return {
    jobs,
    running,
    restarts,
    buildable,
    busy,
    error,
    scheduled,
    load,
    save,
    upsert,
    remove,
    toggleJob,
    toggleScheduler,
    runNow,
    log,
  };
}
