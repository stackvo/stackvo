# Dashboard

The current state of the stack and the machine. Nothing here is a setting; it is all measurement.

## Overview

| Card | What it shows |
| --- | --- |
| Projects | How many are running, how many are stopped. |
| Services | How many service instances are active. |
| Images | The number and size of Docker images on this machine. |
| Health | Anything that needs attention: a domain that does not resolve, a missing certificate. |

## Measurements

| Card | What it shows |
| --- | --- |
| CPU load | The machine's CPU use, split into system, user, nice and idle. |
| CPU history | A graph of recent samples. |
| Memory | Used and available memory. |
| Disk I/O | Current read and write rates, with history. |
| Network | Current download and upload, with history. |

## Worth knowing

- These are for the whole machine, not one project. For a single project's usage, see its Indicator tab.
- The CPU split is absent on the first sample. The counters are cumulative, so the first reading would describe the app's own start-up; it waits for a second sample.
- A warning on the Health card is usually one click to fix: for a missing hosts entry, the button offered adds it.
