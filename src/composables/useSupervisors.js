import { computed, ref } from 'vue';
import { api, asList } from '@/lib/ipc';

/** The states worth a colour of their own. */
export function stateColor(process) {
  if (process.flapping) return 'warning';
  switch (process.state) {
    case 20:
      return 'success';
    case 200:
      return 'error';
    case 0:
      return undefined;
    default:
      return 'info';
  }
}

/** `1:23:45` from a number of seconds, or the text the transport already had. */
export function uptimeOf(process) {
  if (process.uptime === null || process.uptime === undefined) return process.uptimeText || '';
  const total = Math.max(0, Number(process.uptime));
  const days = Math.floor(total / 86400);
  const hours = String(Math.floor((total % 86400) / 3600)).padStart(2, '0');
  const minutes = String(Math.floor((total % 3600) / 60)).padStart(2, '0');
  const seconds = String(total % 60).padStart(2, '0');
  return days ? `${days}d ${hours}:${minutes}:${seconds}` : `${hours}:${minutes}:${seconds}`;
}

/**
 * The health check on one process, edited in place.
 *
 * Kept out of the pane because the dialog and the rows both need it, and a
 * second copy of "what a check is" is how the two would come to disagree.
 */
export function useChecks(project) {
  const checks = ref([]);
  const editing = ref(null);
  const trying = ref(null);

  async function load() {
    if (!project.value) {
      checks.value = [];
      return checks.value;
    }
    checks.value = asList(await api.supervisorChecks(project.value));
    return checks.value;
  }

  const checkFor = (process) => checks.value.find((c) => c.process === process) ?? null;

  /** Open the form for a process, on the check it already has or a blank one. */
  function open(process) {
    const existing = checkFor(process);
    editing.value = existing
      ? { ...existing }
      : {
          project: project.value,
          process,
          kind: 'http',
          target: '',
          expectStatus: 200,
        };
    trying.value = null;
  }

  function close() {
    editing.value = null;
    trying.value = null;
  }

  const valid = computed(() => Boolean(editing.value?.target?.trim()));

  /** The record the backend stores — a blank status is the default, not a value. */
  function record() {
    const check = { ...editing.value, target: editing.value.target.trim() };
    if (check.kind === 'tcp' || !check.expectStatus) delete check.expectStatus;
    else check.expectStatus = Number(check.expectStatus);
    return check;
  }

  async function save() {
    if (!valid.value) return false;
    checks.value = asList(await api.supervisorCheckSave(record()));
    close();
    return true;
  }

  async function remove(process) {
    checks.value = asList(await api.supervisorCheckRemove(project.value, process));
    close();
    return true;
  }

  /** Try it before it is saved, which is the whole reason the form has a button. */
  async function tryIt() {
    if (!valid.value) return null;
    trying.value = 'running';
    try {
      trying.value = await api.supervisorCheckRun(record());
    } catch (e) {
      trying.value = { ok: false, detail: e?.message ?? String(e), ms: 0 };
    }
    return trying.value;
  }

  return { checks, editing, trying, valid, load, checkFor, open, close, save, remove, tryIt };
}
