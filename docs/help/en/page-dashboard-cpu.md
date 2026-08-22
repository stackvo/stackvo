# CPU load

This machine's CPU use. The percentage in the middle of the ring is the total share in use.

## The split

The list beside the ring says where it goes:

| Share | What it is |
| --- | --- |
| System | Kernel work. |
| User | Normal-priority applications. |
| Nice | Low-priority work. |
| Idle | The unused share. |

## Worth knowing

- This is the whole machine, not just the containers. For one project's use, see its Indicator tab.
- The split is not always shown. The counters are cumulative, so the first sample would describe the app's own start-up; until a second one arrives the ring shows only used against idle.
