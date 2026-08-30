<script setup>
import { computed, nextTick, onBeforeUnmount, ref, shallowRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAppearanceStore } from '@/stores/appearance';
import { CONSOLE_BACKGROUND } from '@/lib/appearance';
import { useTerminal } from '@/composables/useTerminal';
import ErrorAlert from '@/components/ErrorAlert.vue';
import PaneHeader from '@/components/PaneHeader.vue';

/**
 * A shell inside this project's container, in the window.
 *
 * The app has offered "open a terminal" since the port, and it has always meant
 * the *system* terminal — `terminal_open_external`, which launches Terminal.app
 * or its equivalent. That still exists and is still the right answer when
 * somebody wants their own shell with their own profile. This is the other
 * half: a session that lives beside the project it belongs to, which is what
 * `pty.rs` was written for and what nothing had ever called.
 *
 * ## Loaded on demand
 *
 * xterm and its fit addon are imported inside `start()` rather than at the top
 * of the file. They are ~250 KB that only a user who asks for a shell needs,
 * and a static import would put them in the bundle every project page pays
 * for — including the ones with no container to attach to.
 *
 * ## Not opened automatically
 *
 * Mounting a pane must not spawn a process. Every other pane on this page reads
 * something; this one would start a shell in a container merely because the
 * page rendered it, and would keep doing so on every visit.
 *
 * That still holds now the pane has a tab of its own, and the tab is the
 * reason to say so again: arriving somewhere is not the same act as asking for
 * a shell, and a session that opened itself would also be one that reopened
 * itself every time somebody passed through. The button stays.
 */
const props = defineProps({
  containerName: { type: String, default: null },
  running: { type: Boolean, default: false },
});

const { t } = useI18n();
const appearance = useAppearanceStore();

/**
 * The console surface, or nothing — one value, read by both halves.
 *
 * `null` when `darkConsoles` is off, because that is what is handed to xterm
 * there: its own default theme. The host frame then falls back to `transparent`
 * and the card underneath shows through, which is the only answer that cannot
 * disagree with a canvas this pane does not paint.
 */
const consoleBackground = computed(() =>
  appearance.value.darkConsoles ? CONSOLE_BACKGROUND : null
);

const host = ref(null);
const term = shallowRef(null);
const fit = shallowRef(null);
let observer = null;

const { status, error, exitCode, open, write, resize, close } = useTerminal(
  (data) => term.value?.write(data),
  (code) => term.value?.writeln(`\r\n\x1b[90m${t('terminal.exited', { code })}\x1b[0m`)
);

async function start() {
  if (!props.containerName || term.value) return;

  const [{ Terminal }, { FitAddon }] = await Promise.all([
    import('@xterm/xterm'),
    import('@xterm/addon-fit'),
  ]);
  await import('@xterm/xterm/css/xterm.css');

  const instance = new Terminal({
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
    fontSize: 13,
    cursorBlink: true,
    // Consoles keep their own dark theme when the setting says so, the same
    // rule `OperationConsole` follows — a terminal on a white sheet is a
    // different product.
    theme: consoleBackground.value ? { background: consoleBackground.value } : undefined,
    // A shell that scrolls away its own output is worse than one that keeps
    // too much; this is bounded but generous.
    scrollback: 5000,
  });
  const fitAddon = new FitAddon();
  instance.loadAddon(fitAddon);

  // Visible before it is opened, and this order is the whole of it.
  //
  // The host is `v-show="term"`, so it stays `display: none` until the line
  // below runs — and an element with no layout box has no size to read.
  // `fit()` derives the row count by measuring the host, so opening first
  // measured a zero-height box: the count it settled on was whatever the
  // observer rescued a frame later, always about one row too many for the
  // space, and the bottom line of every session was drawn half outside the
  // box that `overflow: hidden` then cut in half.
  term.value = instance;
  fit.value = fitAddon;
  await nextTick();

  instance.open(host.value);
  fitAddon.fit();

  // Keystrokes go straight through. The shell, not this pane, decides what a
  // character means — echo, line editing and control sequences are all its.
  instance.onData((data) => write(data));

  // The PTY has to be told the size or every full-screen program in it draws
  // to the wrong width: `top`, `vim` and a wrapped prompt all read it.
  observer = new ResizeObserver(() => {
    fitAddon.fit();
    resize(instance.cols, instance.rows);
  });
  observer.observe(host.value);

  await open({ kind: 'container', name: props.containerName }, instance.cols, instance.rows);
}

async function stop() {
  await close();
  observer?.disconnect();
  observer = null;
  term.value?.dispose();
  term.value = null;
  fit.value = null;
}

// A stopped container has no shell to attach to, and the session it had is
// already gone with it — leaving the pane looking live would be a lie.
watch(
  () => props.running,
  (isRunning) => {
    if (!isRunning && term.value) stop();
  }
);

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
  term.value?.dispose();
  term.value = null;
});
</script>

<template>
  <v-card variant="flat" class="pane">
    <PaneHeader
      help="project-terminal"
      icon="mdi-console"
      :title="t('terminal.title')"
      :description="t('terminal.explain')"
    />

    <ErrorAlert v-if="error" :error="error" class="mb-4" />

    <v-alert v-if="!running" type="info" variant="tonal" class="mb-0">
      {{ t('terminal.needsRunning') }}
    </v-alert>

    <template v-else>
      <div class="d-flex align-center ga-2 mb-3">
        <v-btn
          v-if="!term"
          size="small"
          color="primary"
          variant="tonal"
          :loading="status === 'opening'"
          @click="start"
        >
          {{ t('terminal.start') }}
        </v-btn>
        <v-btn v-else size="small" color="error" variant="tonal" @click="stop">
          {{ t('terminal.stop') }}
        </v-btn>

        <span v-if="exitCode !== null" class="text-caption text-medium-emphasis">
          {{ t('terminal.exited', { code: exitCode }) }}
        </span>
      </div>

      <!-- `tabindex` and the label are what make this reachable without a
           mouse: xterm renders into a canvas, so a screen reader has nothing
           to announce and the keyboard has nothing to land on unless the host
           element provides both. -->
      <div
        v-show="term"
        ref="host"
        class="terminal-host"
        :style="{ '--console-bg': consoleBackground }"
        role="application"
        tabindex="0"
        :aria-label="t('terminal.title')"
      ></div>
    </template>
  </v-card>
</template>

<style scoped>
/* The host's background and xterm's are the same colour by construction now.
   They were the same colour by coincidence — `#12121a` written twice, once here
   and once in the `Terminal` options above — and the padding means both are
   visible at once: the host is the 8px frame around the canvas. Two literals
   that must agree, eight pixels wide, is a frame waiting to appear.

   Bound rather than fixed, because the JS half is conditional: with
   `darkConsoles` off, xterm keeps its own default and the frame has to stop
   claiming otherwise. */
.terminal-host {
  height: 340px;
  border-radius: var(--app-radius, 8px);
  overflow: hidden;
  background: var(--console-bg, transparent);
  padding: 8px;
}
</style>
