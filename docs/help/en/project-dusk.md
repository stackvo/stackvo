# Browser tests (Dusk)

Laravel Dusk drives a real browser against your application. Two things have to be true for that to work here, and only one of them is the part people expect.

## The easy half: a browser in a container

StackVo already knew how to express this. Importing a Sail project recognises its `selenium` service **by name**, and a project's `stackvo.json` has been able to declare its own containers since sidecars existed. So this card declares one:

* the image is **`selenium/standalone-chromium`** on Apple Silicon and `selenium/standalone-chrome` elsewhere. That is not a preference: Google publishes no arm64 build of Chrome, so the `chrome` image has no arm64 manifest, and a browser running under emulation is a test suite that times out for a reason nobody finds;
* it is **tagged**, never `latest` — an untagged image moves under whoever pulled it last month. The tag lives in your `stackvo.json`, so bumping it is your edit rather than a wait for a StackVo release;
* it gets **no host port**, because a declared container never does. Nothing outside this project's network needs to reach it, and two clones of one repository would otherwise fight over 4444.

`.env.dusk.local` is written beside it. Dusk loads that file in place of `.env` for the length of a run, which makes it the right place for a driver URL that only means anything while the container is up:

```
APP_URL=https://<your domain>
DUSK_DRIVER_URL=http://stackvo-<project>-chromium:4444/wd/hub
```

**It is never overwritten.** A file that replaces your `.env` for a test run is one whose contents you should have chosen, so if it is already there this card shows you what it *would* have written and leaves yours alone.

If your project uses an environment other than `local`, rename the file to match — Dusk looks for `.env.dusk.<environment>` and loads nothing if the name does not match.

## The hard half: the certificate

The browser has to open `https://<your domain>`. It is inside a container, and that container has never heard of the certificate authority StackVo installed on **this machine**.

So the test fails on a certificate warning — and a certificate warning inside a browser being driven by a test framework does not look like a certificate warning. It looks like your page did not load, and you go looking in your own code.

The trust button puts the CA in **two** places, and they are two because they fail separately:

| Step | What reads it |
| --- | --- |
| The system bundle (`update-ca-certificates`) | `curl`, the JVM, anything using OpenSSL |
| Chromium's NSS database (`certutil`) | **Chromium itself** — this is the one that decides whether your test passes |

The second needs a tool that is not in every image, so it is reported on its own rather than folded into "trust failed". A step that says `certutil: not found` is a sentence you can act on.

Both run as root inside that container, because the image runs as `seluser` and neither location is writable by it. This is `docker exec` against a container your own project declared, on your own machine.

**It has to be run again when the container is recreated.** It writes into the container's writable layer; that is how the approach works rather than a fault in it, and it is why the sentence is next to the button instead of in a footnote.

## The database

Dusk hits a **real database**. It is not a transaction that gets rolled back at the end of a test, which is why people who happily run their unit tests locally will not run Dusk.

StackVo's own answer to that is already on this page: a **worktree** gives a branch its own database, its own hostname and its own environment. This card says so and stops there — a card that moved your test suite onto a different database without being asked would have decided something for you.

## What this does not do

It does not run your tests. It makes an environment `stackvo artisan dusk` can run in.

And it does not touch your `DuskTestCase`. The driver, the window size and the Chrome flags in it are your code. If Chrome runs out of shared memory in a container — the classic `session not created` crash — the fix is `--disable-dev-shm-usage` in that file's Chrome options. StackVo cannot set it for you: a declared sidecar has no `shm_size` option, and a button that pretended to fix it would be a button that fixed nothing.
