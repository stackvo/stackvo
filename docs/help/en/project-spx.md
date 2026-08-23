# Sampling profiler (php-spx)

The profiler you can leave on.

Xdebug's profiler records every call exactly and costs several times the request, which is right for "what does this one function actually do" and useless for "why is this page slow" — you cannot browse a site under it. php-spx samples instead, so the page still feels like the page and you profile the thing you were doing rather than a laboratory version of it.

Both are here. This one does not replace the Xdebug profiler beside it; they answer different questions.

## Three states, in the order they have to be satisfied

| State | What it means |
| --- | --- |
| Built | The extension has been compiled for this project's PHP version. |
| Mounted | The switch is on, so the compose overlay names this project. |
| In the container | Mounts are fixed when a container is created, so the switch reaches a running project only when it is recreated. |

### Why it has to be built

An extension has to match the exact PHP version, ABI and thread-safety of the binary that loads it, and php-spx is built from source — it is not on PECL. So it is compiled in a throwaway container of **this project's own image**, where the compiler, the headers and the target are the same ones php-fpm was built with.

It takes a few minutes, needs a network, and happens once per PHP version — every project on 8.4 shares one build. The running container is never used for it: that would mean installing a compiler inside your live php-fpm, which lasts until the next recreate and is a side effect nobody asked for.

The extension never enters `stackvo.json`, the Dockerfile or the image. It is mounted, like the debug bridge, so a project that never asks for it pays nothing.

## Recording

There are three ways in, and only one of them needs a browser.

### One request, from this pane

Type a path and press record. The app sends that one request to the project's own address with the profiler's trigger on it, and the recording appears in the list below with what it cost. No browser, no cookie, nothing to switch off afterwards — which also means this is the way an assistant or a terminal can profile a page (`stackvo spx-record <project> /checkout`).

The address is the project's, from its manifest. Only the path is yours, and a path naming another host is refused.

### One command

A migration, a queue worker, a test run. The slow thing is often not a page, and none of those can be profiled from a browser. Pick one of the project's own commands and it runs under the profiler, in the operation console, landing in the same list.

### The control panel, for a session

SPX's own panel is served by the extension from this site's own address — there is no port to publish and no second server to run. Open it, switch recording on there, and every page you then use is recorded. That is the one to use when what you want to profile is a *click*: a form, a checkout, a session with a logged-in user.

The panel has its own controls for sampling and built-ins; the Detail setting on this pane applies to recordings started **here**.

Loading the extension costs almost nothing on its own; nothing is recorded until you ask for it.

## Detail

php-spx records **every call** unless a sampling period is set — which is the cost this instrument exists to avoid. StackVo samples every 100 µs by default, which is what makes it safe to leave on: a function holding a tenth of a 30 ms request still has thirty samples behind it.

"Every call" is still a choice, and it is the right one for counting a fast function exactly rather than estimating it. Profiling PHP's own functions roughly doubles a trace, and is worth switching on when the answer turns out to be `preg_match` rather than anything in the project.

## Where the time went

Each recording opens into the functions that held it: the share of the run each spent in its own body, the share it held including everything it called, and how many times it was called. That is read from the recording itself, so it needs no browser either.

The flame graph, the call tree and the timeline are php-spx's own and better than anything worth rebuilding here — the second button on each row opens that recording in its viewer.

A very long trace is read up to a limit and says so when it hits one. What you are then looking at is the start of the run, honestly labelled, rather than a summary of all of it.

## Do not run both profilers at once

Two profilers hooking one engine is not supported by either project, and the symptom is numbers that are wrong rather than an error. The pane says so when Xdebug is also recording; switch the Xdebug mode back to step debugging.
