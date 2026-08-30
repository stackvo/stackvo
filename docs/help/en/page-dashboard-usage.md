# What it cost today

CPU used and memory held since midnight UTC, one row per container.

Everything else on this page is *now*: what is running, how hot the machine is at this second. This is the only card that answers the question people actually ask about a container-based setup — what it cost them over an afternoon.

## The two numbers

| Column | What it is |
| --- | --- |
| CPU | Seconds of one core, shown in minutes. The same thing `time` reports, so ten minutes here is ten minutes of one core — or five minutes of two. |
| Memory held | Gigabyte-hours. Memory is not spent, it is **occupied**, so the unit has time in it: one gigabyte held for an hour, or two held for half of one. |

## Where the numbers come from

The app has read CPU and memory once a minute since it was written, for the sparkline on a project's page, and threw the readings away after two hours. Nothing new is measured; the same readings are added up instead of discarded.

The interval between two readings is **measured**, not assumed to be the sixty seconds the timer is set to. A gap longer than five minutes contributes nothing at all — a laptop closed on Friday and opened on Monday must not be billed for the weekend at Friday's rate. The reading still counts and the clock still moves; only the time is refused. So a total can be a few minutes short after a sleep, and it can never be three days long.

Thirty days are kept. Anything older is gone rather than summarised, because a summary of a summary is a number nobody can check.

## Shared services are not divided up

`shop` and `blog` both use the same MySQL. Splitting its memory between them would be a made-up number, and a number you cannot check is one you cannot act on — so a service gets its own row and says what it is. The stack's own containers, the router and the mail catcher, are listed for the same reason: leaving them out would understate what Docker costs on this machine.

## Budgets

Only a project can carry one, for the same reason: a shared service is not any one project's to be over on. A budget is set per project and is a **machine's** decision — it lives in this app's preferences and not in `stackvo.json`, because the same repository on a colleague's laptop has different room to spare, and a threshold committed to git would be one of you arguing with the other in a pull request.

When a project passes its budget you are told **once that day**. The sampler runs every minute and a project that is over at two o'clock is still over at half past; a notice per reading would be four hundred by the evening, and the feature you switch off within the hour is the one that would have told you tomorrow. The first breach the next day is a notice again.

A budget of zero is no budget. A cleared field arrives as a zero, and an alert that fires the moment you clear the box is one nobody leaves switched on.
