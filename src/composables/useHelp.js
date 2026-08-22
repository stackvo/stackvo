import { readonly, ref } from 'vue';

/**
 * Which help topic the user has asked for, if any.
 *
 * Module-level rather than per-component: the button that asks is inside a card
 * halfway down a tab, and whatever ends up showing the answer is not its child.
 * One ref both can see is the whole of the wiring — the alternative is an event
 * threaded up through every pane between them.
 *
 * Nothing reads `topic` yet. The buttons and their topics land first so that
 * every card has one and the map is complete; the viewer is the next step, and
 * it needs exactly this ref.
 */
const topic = ref(null);

export function useHelp() {
  return {
    topic: readonly(topic),
    openHelp: (next) => {
      topic.value = next;
    },
    closeHelp: () => {
      topic.value = null;
    },
  };
}
