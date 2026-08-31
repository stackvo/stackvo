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

### The same request again

Every recording of a GET carries a **send it again** button. It re-issues exactly that request with the profiler on and shows both numbers with the difference — which is the commonest loop in performance work, and one that otherwise takes four steps: change the code, open the site, find the page, come back and hunt for the new recording among twenty.

There is deliberately no verdict on the screen. One run against one run is not a benchmark: a cold opcache, a cold query cache and whatever else the machine was doing at that second are all inside the difference. The two numbers are shown so you can read them, not so the app can tell you what they mean.

**Only a GET can be sent again**, and the refusal says why rather than hiding the button. A recording names the request *line* — `GET /checkout` — and nothing else: not its headers, not its body, not the session it ran under, because nothing records those. A POST re-sent without them is a different request, and against any framework with CSRF it answers 419 rather than the page. A result that looks like an answer and is not would be worse than a refusal.

### Capturing sessions, so a POST can be sent again

The refusal above is only true while nothing records the session. **Capture sessions** changes that — and what it changes it into is worth reading before you press it.

**It writes this project's request cookies and form input to disk.** A session cookie *is* the credential; there is no useful redacted version of it, because the value is the entire point of keeping it. So this is not a setting. It is a permission, and it is built like one:

| Rule | Why |
| --- | --- |
| **Off until you press it** | The bridge writes nothing without a second flag, separate from the one that shows your dumps. "Show me my dumps" and "record my session token" are two different permissions. |
| **Minutes, never indefinitely** | Five to sixty. The window is stored as the moment it ends, not as a length — a window that restarted its clock when the app opened would be a window that never closes. |
| **It ends by itself** | Even if the app was closed for the whole hour. Expiry is checked when anything asks, not by a timer that only runs while you are looking. |
| **Stopping deletes** | Not just "no new captures" — the ones already taken are removed, and the button tells you how many. A permission that ends leaving its harvest behind is one you only *believe* has ended. |

**The debug bridge has to be on first, and the button says so rather than letting you arm a window that records nothing.** The two flags are separate permissions on purpose, but they are not independent: the bridge's prepend file is the only thing that reads the capture flag, so arming with the bridge off would grant the permission, write the entry in the audit trail, and capture nothing at all — leaving you to conclude your POST simply cannot be replayed. Switch it on in the **Dumps** pane, then arm the window.

**No screen ever shows what was captured.** The list under the alert gives the request line, a *count* of cookies and a *size* of body — enough to tell you there is something here to replay, and not a second place your session token exists. It is not in the audit trail either: that records the window being opened and how long for, which is the part somebody has to be able to date.

A captured session is attached to a recording by the request line **and** the clock together, within two seconds. Either alone would be wrong: the line by itself would attach one visitor's basket to another visitor's recording of the same page. A recording with no match keeps the old refusal, word for word, because for that recording it is still true.

### Starting the replay from a snapshot

Once a session is captured, a POST can be sent again — and that means it **does the thing again**. A second order. A second row. A second charge.

That is not a reason to refuse it: you are pressing replay on a POST on purpose. It is a reason to offer the one thing that makes pressing it twice safe, which StackVo already has — a **named database snapshot**.

Pick one under **Start the replay from a snapshot** and it is restored before the second run. Four rules, and each is a refusal rather than a convenience:

| Rule | Why |
| --- | --- |
| Restored **before** the replay, never after | Restoring afterwards would discard what the replay did, which is the thing you replayed a POST to look at. |
| A **safety copy** of what is there now is taken first | The same net any restore gets. Pressing a button on a profiling screen must not be the irreversible act. |
| **StackVo never picks one** | It cannot know which snapshot holds the state the original ran under, and choosing one would be answering a question you did not ask with data it does not have. |
| A failure **stops the replay** | You asked for the second run to start from that snapshot. Sending the request anyway would run it from a state you did not choose and print a number under a premise that is not true. |

**What it buys, stated plainly: repeatability, not comparability.** The first recording ran against whatever the database held at the time, and nothing wrote that down. A snapshot means you can run the replay again from a stated point — not that the two numbers are a controlled experiment. The name of the snapshot is shown beside the result so the second run's premise is on screen rather than in your memory.

The rows that would write are marked, and the mark comes from the recording rather than from the screen guessing at a string: anything whose request line is not a `GET`. A GET can write too and nothing here can know that, which is why the marking says what it measured rather than promising more.

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
