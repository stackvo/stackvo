# Profiler

Xdebug's own profiler. It writes into files this app reads. No account and no extra extension.

## The modes

| Mode | What it does |
| --- | --- |
| Step debugging | Connects to your IDE on every request. |
| Profiling | Waits for a trigger, then writes that request's profile to a file. |
| Trace | Writes every function entry and exit. |

One or the other. Leaving two on breaks one of them, so the card makes you pick.

## How to record

Nothing is recorded until a request asks for it. Add `?XDEBUG_TRIGGER=1` to the address you want to look at, or set the same name as a cookie. The card shows the exact trigger to use.

## Controls

| Control | What it does |
| --- | --- |
| Mode | Applies one of the three modes above. |
| The recording list | The profile files that have been written. Clicking one opens its flame graph. |
| Delete | Clears the recordings. |
| Apply to container | Recreates the container when the running one disagrees with the selected mode. |

## What a trace costs

A trace is far heavier than a profile. A single request can run to hundreds of megabytes. Record one page, then set the mode back.

The flame graph cannot draw the whole of a very long trace. When that happens the card says that what you are seeing is only the start of the request.

## Worth knowing

- Xdebug has to be on first. Profiling is a mode of the same extension.
- A mode change does not take effect until the container is recreated. The card says when they disagree.

## Coverage

The fourth mode, and the only one that records nothing of its own: it switches on the API PHPUnit calls, and PHPUnit writes the report. Run your tests with a coverage flag once it is applied — nothing will ever appear in the recorded list for it, and the pane says so rather than leaving you waiting.

Without it, `--coverage-html` produces an empty report and a warning most people never read.

## Readable dumps and stack traces

Xdebug's `develop` is not a fifth mode. `xdebug.mode` is a **list**, and `develop` rides alongside whichever mode is chosen — so `XDEBUG_MODE` becomes `debug,develop` rather than replacing your choice. It makes `var_dump` readable and puts a stack trace on a warning.

It is off until asked for, because it changes what your own code prints, and a debugging tool that alters the output of the code being debugged should be something you chose.

The switch and the mode buttons are two controls over one file: moving one leaves the other exactly as it was.
