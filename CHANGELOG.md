# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning is [semver](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **`strict` is off in the `main` ruleset, because it needs a queue and there
  is none.** What it bought is worth naming: every merged pull request had been
  tested against `main` exactly as it stood, so two changes that were each
  green alone could not land and break together.

  What it cost was measured on 2 September 2026. A merge to `main` marks every
  other open pull request out-of-date at once; each then needs `Update branch`
  and a fresh eight-minute run of ten required checks, and any merge inside
  that window marks them all again. Three pull requests — #78, #79 and #81 —
  went BEHIND the moment #83 landed. None could be merged. No two of them
  touched the same file.

  What is actually given up is smaller than it looks: CI runs on `push` to
  `main` as well as on pull requests, so a pair that only breaks together still
  fails — on the merge rather than before it. The right replacement is a merge
  queue, which tests each pull request against the `main` it will land on and
  merges it with nobody clicking; ten required checks make for a slow queue, so
  that is its own decision to take.

  Changed in [`.github/rulesets/main.json`](.github/rulesets/main.json) **and**
  in the repository settings, which is the rule that file exists to enforce: a
  drift between the two is the invisible state it was written to replace.

### Fixed

- **`vitest` reported 1437 passed and exited 1.** Three times over two days,
  with a clean run either side of each — the shape that gets a suite re-run
  rather than read. The failure was not in an assertion; it arrived after every
  one of them had passed, as three of:

      TypeError: Cannot read properties of null (reading 'insertBefore')
        ❯ insert @vue/runtime-dom … ❯ ReactiveEffect.componentUpdateFn

  `tests/service-connection.spec.js` clears `document.body` in `afterEach` and
  did not unmount first, which takes the DOM away from a component that is still
  mounted and still scheduling. That file's overlay tests mount with `attachTo:
  document.body` and no teleport stub on purpose — the stub does not relocate an
  overlay's content, it drops it, so a menu inside the sheet becomes unreadable
  — which means Vuetify's real overlays are in play, and an overlay's activation
  runs through a transition and a `setTimeout`. A render could leave the queue
  after the test that asked for it had ended, and insert into a parent that no
  longer existed. Whether it flushed before or after teardown was the race.

  `afterEach` unmounts what it mounted, and then clears. The clear stays: a
  stray overlay node left by a component this file did not mount would otherwise
  be read by `overlays()`.

- **The `driver` job passed its five tests and then hung for 55 minutes.** It
  had to be cancelled by hand, and it would have run for six hours if nobody
  had been watching: no job in `ci.yml` carried a `timeout-minutes`, so every
  one of them ran under GitHub's default, which is not a limit so much as the
  absence of one.

  The hang was in teardown, and the log said nothing because there was nothing
  to say — five `ok` lines, then no TAP summary at all. `after` calls
  `session.close()`, which is `DELETE /session`; the driver answers that by
  closing the window; and `lib.rs` answers a `CloseRequested` with
  `api.prevent_close()` on **every** path. Under the default `closeBehaviour`
  of `"ask"` the close is forwarded to the front end as `app:close_requested`,
  a dialog opens, and it waits for a click. There is nobody at a driven run to
  click it, so the application never exited, the driver never replied, and
  `fetch` — which has no timeout of its own — waited for a response that was
  never going to be written.

  Three things were wrong and all three are fixed. The suite now writes
  `closeBehaviour: "quit"` into the temporary config directory before the
  application starts, so the close it is asked to perform is one it can
  perform. Every WebDriver request has a deadline, and teardown has a shorter
  one, because by then the answer changes nothing. And `stop` signals the
  driver's whole **process group** — `tauri-driver` → `WebKitWebDriver` → the
  application — rather than the one pid it spawned, which is why the runner
  used to report six orphan processes it had to terminate on its way out.

- **`STACKVO_CONFIG_DIR` did nothing.** `tests/driver/launch.js` set it, its
  own comment said it moved the settings, and no line of Rust read the
  variable: `appdir::config()` went to the OS config directory regardless. So
  the driven application read and wrote the real `~/.config/stackvo` of
  whatever machine it ran on — which is how it found `closeBehaviour: "ask"` in
  the first place, and what made the isolation the suite claimed a claim only.
  `config()` reads it now, the same seam `STACKVO_ROOT` and
  `STACKVO_POLICY_FILE` already were, and `migrate_config` declines to run
  under an override rather than moving a real preferences file into a directory
  a test deletes.

- **The driver suite ran twice per job, and a failure in it could not fail the
  job.** Two steps ran the same five tests — one for the exit code, one for the
  TAP — on the slowest job in CI. They are one step now, and it sets
  `set -o pipefail`: the run is piped into `tee`, `bash -e` reports the status
  of the last command in a pipeline, and `tee` succeeds whatever `node` did.
  The check that follows reads the pass count as well as the skip count,
  because a TAP stream with no summary is the signature of the hang above and
  an exit code cannot tell it apart from a clean kill.

## [0.2.0] - 2026-09-01

The first release with a changelog behind it. Everything below had been sitting
under `Unreleased` since the repository started, which meant the file recorded
the work honestly and dated none of it — a reader could not tell what shipped
from what was merely written down. It is dated now, and what follows this
heading is the engineering log; the short version a user reads is
[`docs/RELEASE-NOTES-0.2.0.md`](docs/RELEASE-NOTES-0.2.0.md).

### Added

- **The screenshots exist, and producing them is no longer somebody's afternoon.**
  `npm run screenshots` boots the built front end in Chromium behind
  `tests/e2e/stage.js` — the same boundary the Playwright suite replaces — and
  writes thirty-seven pictures into `docs/screenshots/` at 1600x1000@2x: six
  pages, the project detail page's ten sections, the settings page's seventeen,
  and four screens that have no address at all — the service detail sheet, the
  instance settings sheet, the new-instance dialog and the new-project drawer,
  each opened by the control that opens it. Both READMEs show twelve of them in a
  four-column grid and link to [`docs/screenshots/README.md`](docs/screenshots/README.md),
  where all thirty-seven are laid out by group. The detail sections are clicked, because `section` is a ref inside
  `ProjectDetail.vue` and no URL names it; the settings panes are navigated to,
  because `Settings.vue` reads `route.query.tab` and one of them is addressable
  where the other is not. Sharing the suite's stage is the point rather than a
  convenience: a generator with its own fixture drifts into photographing a
  product no test has ever rendered.

  It reports, per page, every command that page asked and nobody staged. `null`
  is a legal reply over this boundary, so an unstaged command does not fail — it
  draws a spinner that never resolves, in a picture meant to show the page doing
  its work. The first run found the sharper version of the same class:
  `WelcomeTour` renders **instead of** `<router-view>`, so ten files came out
  byte-identical and every one of them was step 1 of 6. The rail clicks found
  the other kind: Playwright's `hasText` matches a case-insensitive substring,
  so the AI section's label selected **Cont(ai)ner** and that tab's picture was
  the container pane — with the rail highlighted to say so, in a file nobody
  would have doubted.

  **And the settings panes found two things in the tree rather than in the
  tool.** `tests/e2e/stage.js` answered `engine_status.platform` as
  `dockerDesktop`, where `Platform` is `#[serde(rename_all = "kebab-case")]` —
  so the doctor pane rendered the raw key `engine.platform.dockerDesktop` where
  a person would read "Docker Desktop", and no assertion had ever looked. The
  same file's `cert_status` was missing `rejected`, which `CertificatesPane`
  reads as `certs.rejected.length` without a guard: the pane threw instead of
  drawing. `certs.rs` returns that field; `contracts/ipc.json`'s `CertStatus`
  does not declare it, which is why the stage was written without it — the
  contract is behind the struct, and `notAfter` is typed `string?` there against
  an `Option<i64>` as well.

  **The catalogue page photographed as "No catalogue yet" for the same class of
  reason.** It gates on `status.fetched === true`; the stage answered a
  `market_status` without that field, so every picture of the market was the
  screen a fresh install sees. And `Service.id` is the INSTANCE id on the market
  model — the contract says so — while the stage still answered the pre-market
  `mysql`, so `serviceFor(instance)` found nothing and every row's Detail button
  rendered disabled. Neither is a bug in the app; both are a fixture that had
  never been looked at.

- **Nine of the Playwright suite's nineteen tests were failing, and the reason
  was the same welcome tour.** `tests/e2e/stage.js` never answered `tourSeen`,
  so every page assertion in `shell.e2e.js` and `rtl.e2e.js` was made against a
  screen that renders instead of the router — "Dashboard is not visible" was
  true, and about the wrong page. The stage answers it now, and `rtl.e2e.js`
  answers it in its own override too: an override REPLACES `prefs_get` rather
  than merging into it, which is what put the tour back over that one test after
  the shared fix. Nineteen of nineteen pass. The same file's `engine_status` and
  `cert_status` were corrected in the same pass, above.

- **The Laravel ecosystem block, and the one pattern behind eight of its nine
  items: the piece was already here and its precondition was never said.**

  `php artisan boost:install` writes a `.mcp.json` holding `php artisan
  boost:mcp` — a command line that assumes a php on the host. There is none, by
  decision, so Laravel's own installer leaves a configuration that cannot start
  and the failure surfaces in somebody's assistant rather than in the tool that
  wrote it. `boost.rs` writes the passage this application already owns
  instead: `docker exec -i stackvo-<project> php artisan boost:mcp`. Not
  `stackvo artisan`, and the reason is the working directory — the CLI resolves
  which project it means from where it was started and an assistant starts its
  servers from wherever it happens to be, so the passage that names the
  container is the one that cannot pick the wrong project. The registration is
  matched by **what it runs** rather than by what it is called, and an entry
  that is already there keeps its own name: renaming it would leave two servers
  in somebody's client instead of one working one. A `Mcp::web()` server needs
  nothing at all — it is an ordinary route inside the application, already
  served on the project's own domain over a certificate the browser trusts, so
  that row shows the URL and offers no button.

  **Telescope, Horizon and Pulse all opened at `https://<domain>/…` the whole
  time** and nothing said so. A link would have been the cheap half. Each of
  the three goes quietly empty for a reason the developer cannot see, and every
  container is green while it happens: Horizon wants a `redis` queue connection
  and a five-minute `horizon:snapshot`; Telescope wants its migrations and a
  daily prune; Pulse refuses SQLite storage, wants a Redis connection separate
  from the queue's, and needs a long-running `pulse:check` — which is why it is
  a **worker** (`worker.rs` gained `Kind::Pulse`, and `Kind::PulseWork` where
  the ingest asks for it) rather than a schedule entry that would start a second
  copy every time it fired. The limit is stated where it applies rather than at
  the top of a pane: this reads `.env` and `composer.lock`, not `config/*.php`,
  so every observation carries **the key it read and the value it found** and
  the sentence about a compiled configuration sits beside that row.

  **A worktree's own hostname is the one Sanctum does not know**, which is a bug
  this feature produced by working. Two variables are now written, and each only
  where it changes something: `SANCTUM_STATEFUL_DOMAINS` only where the project
  pins a list of its own — and the branch host is *appended* to it, because
  where the project pins nothing Sanctum's default already follows `APP_URL` and
  writing a list there could only take hostnames away; `SESSION_DOMAIN` only
  where the project pins a domain that does not cover the branch host, because
  Laravel's own default is the current host and is already right. Both go in
  before the record's own variables, so a value somebody typed still wins.
  Passport's signing key is **generated** in the branch's container and the
  parent's is never copied — a branch able to mint the tokens it branched from
  is the opposite of what its own database is for — and a key that is already
  there is refused rather than overwritten.

  **Dusk needed two pieces and neither works without the other.** The sidecar
  was the easy half; the half no `docker-compose.yml` fixes is that the browser
  has to open `https://` from inside a container that has never heard of this
  machine's certificate authority, and a test that dies on a certificate
  warning reads as an application bug. The CA now goes into two places that
  fail separately — the system bundle and Chromium's NSS database — reported as
  two results rather than one tick, because a step that says `certutil: not
  found` is a sentence somebody can act on. It writes into the container's
  writable layer, so recreating the container loses it; that is said beside the
  button. The image follows the architecture (`standalone-chromium` on arm64,
  because no arm64 Chrome exists and a browser under emulation is a suite that
  times out for a reason nobody finds), and `.env.dusk.local` is shown before it
  is written and never overwritten.

  **Octane holds an edited file in memory**, so a route somebody adds does not
  exist until the workers are replaced — and they go looking for the mistake in
  their own code. Laravel's answer is `octane:start --watch` and its price is
  Node and chokidar *inside the image*, polling a bind mount this application
  already watches from the host. So the answer here is `octane:reload` attached
  to the watcher that is already running: nothing is added to the image, which
  makes it strictly better than the documented route rather than merely
  different. Off by default, because a reload arriving mid-request kills that
  request; debounced by seconds rather than milliseconds, because a `composer
  install` touches thousands of files.

  **`verify` gained its Laravel half with no new measurement.**
  `detect::first_version` has turned `^8.3` into `8.3` all along and has used it
  to *choose* a PHP line at adoption; the two were simply never compared again.
  `composer.json` against the manifest catches a frequent, expensive failure —
  the image builds, then `composer install` dies inside the container naming PHP
  and not naming the file that has to change — and the `ext-*` half is one row
  per missing extension, because the repair is per extension. `require-dev` is
  not read, and a constraint with no `major.minor` in it is `unknown` rather
  than a guess.

  **The command catalogue had no check in it.** `pint --test` (the reporting
  shape, never the bare command, because a formatter that rewrites the tree
  behind a button is a diff nobody asked for), `artisan test`,
  `wayfinder:generate` and `boost:add-skill`. Three of the four are offered on
  the **package** that provides them rather than on `artisan`, because every
  Laravel project has artisan and most have none of these. Wayfinder is the one
  row with a pre-step — `route:clear` first, as two argv arrays in sequence
  rather than one `sh -c "a && b"` — because it reads the route list and a
  cached one makes it generate against yesterday's routes, which produces
  *wrong* output under a green console. `pennant:purge` is absent and a test
  holds it there.

  **`sk-ant-` was the last common provider the leak scanner could not see.**
  OpenAI's and Google's were already in the table, and `laravel/ai` puts one of
  the three in the `.env` of every application that installs it. It is not a
  worse finding than the others; what is different is that having one says the
  application sends every request it makes outside this machine, so one sentence
  beside it points at the egress report — which asks Docker which containers can
  reach the internet rather than inferring it.

  **Scout is the purest form of the pattern**: the service is a Market click and
  an empty index returns nothing for every search while every container is
  green. It gets a sentence and not a button, because `scout:import` takes a
  model class name this application cannot know — and it points at the mechanism
  that does exist, a project's own `commands` block.

- **K-5: a replay can be bound to a database snapshot**, which is what makes
  replaying a POST safe to press twice.

  Capturing a session lifted the old refusal, and what took its place is a
  request that **does the thing again** — a second order, a second row, a second
  charge. That is not a reason to refuse it; somebody replaying a POST is doing
  it on purpose. It is a reason to offer the one thing that makes pressing it
  twice safe, which this application already had: a named snapshot.

  Four rules, each a refusal rather than a convenience. It is restored **before**
  the second run and never after, because restoring afterwards would discard
  what the replay did — the thing a POST was replayed to look at. The restore
  takes its own safety copy first, the same net `db_restore` puts under every
  restore: pressing a button on a profiling screen must not be the irreversible
  act. It is **never chosen automatically**, because this app cannot know which
  snapshot holds the state the original ran under and picking one would be
  answering an unasked question with data it does not have. And a failure at any
  step **stops the replay**, because sending the request from a state the caller
  did not choose would print a number under a premise that is not true.

  What it buys is stated rather than implied: **repeatability, not
  comparability.** The first recording ran against whatever the database held at
  the time and nothing wrote that down, so a snapshot means the replay can be
  run again from a stated point — not that the two numbers are a controlled
  experiment. The snapshot's name comes back with the result so the second run's
  premise sits beside its numbers.

  A recording now carries `mutates` — whether sending it again would write.
  Everything that is not a `GET`, and never a CLI run. It is computed where the
  request line is parsed, so the window, the pane and an assistant all read one
  answer instead of three re-derivations of a string, and the marking says what
  it measured rather than promising more: a GET can write too, and nothing here
  can know that.

- **A capture window can no longer be armed while the bridge that would do the
  capturing is off**, which was a permission that granted nothing.

  The two flags are separate on purpose — *"show me my dumps"* and *"record my
  session token"* are two permissions, and that separation is the point of the
  design. They are not independent, though: the debug bridge's prepend file is
  the only thing that reads the capture flag, and it returns at its first line
  when the bridge is off. So arming without it wrote the flag into a directory
  nothing was reading, took the sharpest entry in the audit trail — *credentials
  are now being written* — and captured nothing at all.

  What somebody concludes from an empty window is not "the bridge is off". It is
  that their POST simply cannot be replayed, which is the wrong lesson to teach
  with a button. This is the failure `channel.rs` names in one sentence — a
  setting that silently does nothing — so it is now refused with the reason
  (`capture::arm`, `Conflict`, the hint pointing at the switch), and
  `capture_status` carries `bridge` so the pane says it **before** the press
  rather than after.

- **The README says how to open a build that is not code-signed**, which is the
  one installation step this project's own distribution decision creates.

  Releases come from GitHub Releases and nowhere else, and there is no code
  signing — an Apple Developer Program membership and an Authenticode
  certificate are not store requirements, they are needed for a file downloaded
  from a release page too, and dropping them dropped the last external
  dependency in the chain. The cost lands once, at first launch, on the person
  who downloaded it.

  It is written out because **macOS does not say "unidentified developer" for an
  unsigned bundle — it says "is damaged and can't be opened. You should move it
  to the Trash."** That message is about the quarantine attribute and not about
  the file, and nothing on the dialog says so; without a paragraph naming it,
  every macOS download is a support ticket where somebody cannot tell an
  installation step from a corrupt file. Both routes out are given (right-click
  → Open, or `xattr -dr com.apple.quarantine`), Windows gets its own step (More
  info → Run anyway, which is a warning with an extra click rather than a
  refusal), and Linux is named as having nothing to do rather than left out.

  And the section says what *can* still be verified, so "unsigned" does not read
  as "nothing is checked": the per-target `SHA256SUMS-…` list published beside
  the artifacts, and the updater's own minisign signature over the update
  manifest, whose key is separate from platform code signing and is in place.

  `readme_claims.rs` holds the section against `release.yml`'s own warnings —
  the ones the build prints when the signing secrets are absent. That is the
  direction the pair goes wrong in: somebody adds signing, the workflow stops
  warning, and the README keeps telling people to right-click past a dialog they
  will never see.

- **A package can bring its own commands**, which closes the second half of the
  machine-wide commands item.

  Installing the Redis package can also give you `redis-cli`, the way installing
  `ddev-redis` gives you `ddev redis-cli` — with the one thing DDEV does not
  have: every byte of it was verified before it was read, and re-verified on
  every read rather than only at install.

  A package may name a command for the reason a project may name one in its own
  container and a **sidecar** may not: the package already chose the image,
  already wrote the compose fragment and already decides what that container
  runs. What is still *built* rather than inherited is the containment — only an
  enabled instance is offered, the command never reaches the host, and the
  container name is derived from the instance rather than declared, so a package
  can no more name somebody else's container than it can name a host port. Ids
  carry the instance (`redis-7-2:redis-cli`), so two installed versions of one
  service are two rows rather than one collision and a project cannot shadow a
  package's command by declaring the same string. The row is tagged with the
  instance it runs in, because *"this does not touch your project"* is what to
  say before somebody presses a button, and the package author's own sentence
  carries `lang=""` rather than claiming this interface's language.

  No shell-metacharacter rule is applied to the argv, and a test holds that
  open: there is no shell, the argv reaches `docker exec` as an array, and a
  refusal there would read as security while only breaking legitimate arguments
  such as `redis-cli --scan --pattern 'a*'`.

  The failure mode this feature could have had is the one the first half already
  taught: *the layers were never joined up*, invisible to six tests that built
  both halves by hand. Three tests now write a real package tree — manifest,
  compose fragment, matching digest, instance table — and one of them corrupts
  the compose file and asserts that nothing is offered, so the last link of the
  chain is tested rather than assumed.

- **Four things reported from the screen, and one of them was a bug in the
  build.**

  **An icon added while the dev server was running never appeared**, and the
  cause was neither the icon nor the subsetter. `vite.config.js`'s
  `mdi-used-icons-only` plugin scans the source trees inside `transform`, so it
  reads them once — the first time Vite asks for `materialdesignicons.css` — and
  Vite caches the result. Nothing in the module graph connects a `.vue` file to
  that stylesheet, so adding `mdi-source-branch-check` to a new pane left the
  cached, already-subsetted stylesheet being served for the rest of the session.
  The pane drew a blank square while the name was a real icon and the scan did
  collect it, which is why every check anybody would run said it was fine.

  The plugin is now `apply: 'build'`. Subsetting exists to shrink the *shipped*
  bundle; in dev nobody is measuring bundle size and correctness is what
  matters. Nothing is lost: a name that is not an icon is caught by
  `tests/mdi-icons.spec.js`, and a rule wrongly stripped by
  `tools/check-bundle.mjs` after the build — two gates that fail loudly, where
  the dev-server square failed silently.

  **The catalogue's own sentence moved into the info tooltip** rather than
  getting a tooltip of its own. A tooltip on plain text is reachable by no
  keyboard and opened by no touch; the service row already carries a focusable
  info button whose `aria-label` joins its facts into one string, so the summary
  became the first of those facts — set apart from the counts, with no clamp,
  because the tooltip is finally somewhere the whole sentence fits. The
  two-line-under-the-name rendering and its `-webkit-line-clamp` are gone with
  it; a style rule for a class nothing uses is one the next reader has to prove
  is dead.

  **The OAuth chips name the action instead of the lack.** "Needs public" reads
  as a fault in the provider or in the project and is neither — three of the
  seven accept only a public hostname, which is a fact about their console. The
  chips now say *paste the local one* / *paste the public one*, and a row that
  cannot proceed says so with the repair beside it rather than leaving somebody
  to connect a warning at the top of the pane to a row at the bottom.

  **A rebuild-everything button joins start, stop and restart in the app bar.**
  `project_build` was per project, and changing a PHP version or a generator
  template changes every project's Dockerfile. It is deliberately **not** one
  `docker compose build`: that is a single process, and one broken Dockerfile
  ends it leaving the projects behind it unbuilt with nothing saying which those
  were — the answer `generator::render` already gave for a broken manifest.
  So the builds are sequential and the result **names** every project that
  failed rather than counting them.

  `noCache` defaults to false — "rebuild" and "pull every layer again" are
  different jobs and the second takes minutes per project.

- **All four stack actions now ask first.** The app bar's start-all, stop-all,
  restart-all and rebuild-all act on **every container on the machine**, and
  none of them is undone by pressing another: stopping takes every site down
  now, and starting them again is a different act with a different cost. A
  single mis-aimed click on a toolbar is the cheapest way to lose an
  afternoon's running state, and a toolbar is exactly where mis-aimed clicks
  happen.

  All four, not the slowest one. The first instinct was to guard only the
  rebuild because it takes minutes — but stopping is the *abrupt* one, and a
  dialog on only the slow one would have been in front of the wrong button.

  `ConfirmDialog` is the application's first, and it does not ask "are you
  sure?" — a question nobody can answer, because it does not say what about.
  Each caller passes the sentence that names what will happen and to how many
  things, which is `hints.rs`' rule applied to a dialog. The confirm button
  carries the **same verb as the toolbar button that opened it** rather than
  "OK", so the second press reads as the same act as the first, and it is
  deliberately **not autofocused**: a dialog that opens with the destructive
  action under the return key turns one stray keypress into two, which is the
  opposite of what a second, deliberate act is for.

- **Replaying the request that actually failed.** `explain::replayable` refused a
  POST by name, with the reason: *"only the request line was recorded — not its
  body, its headers or its session — and a POST replayed without them is a
  different request, which is usually answered with a redirect or a 419 rather
  than the page"*. That refusal was not a gap in the code. It was what the code
  knew.

  But the roadmap item's own example is the one that needs the other half:
  *"this bug only happened in that basket."* A basket is a session, and
  replaying it means recording the request's cookies and body — **somebody's
  session token and their form input, written to disk.** So the feature is the
  asking, not the storing.

  **A permission, built like one, with four rules that are each a refusal.**
  *Off, always, until armed* — the bridge writes nothing without a second flag
  file, separate from the one that turns the bridge on, so "show me my dumps"
  and "record my session token" are two permissions rather than one. *Minutes,
  never indefinitely* — sixty at most, stored as an absolute expiry rather than
  a duration, which is `stats_store.rs`'s lesson about a series reloaded
  verbatim applied to a permission: a window that restarts its clock on launch
  is a window that never closes. *It ends by itself*, swept when anything next
  asks, because a timer that only runs while the app is open would leave one
  open across a night the app spent closed — `worktree` expiry's arrangement.
  *Disarming deletes* — not merely no new captures, but the ones already taken,
  with the count reported: a permission that ends leaving its harvest behind is
  one somebody only believes has ended.

  **The value never reaches a report.** Redaction is unavailable here, because
  the value is the entire point of keeping it — so what is done instead is time,
  deletion, and never quoting. `capture_status` answers with a *count* of cookie
  names and a *size* of body; the audit trail records the window being opened
  and how long for, which is the fact somebody has to be able to date, and never
  a captured value; and the diagnostics bundle reads the app's log directory
  only, so it cannot pick these up.

  **The join is the request line and the clock together**, within two seconds,
  nearest wins. Either alone is wrong: the line by itself would attach one
  visitor's basket to another visitor's recording of the same page, and the
  clock by itself would attach whatever happened next. A recording with no match
  keeps the old refusal **word for word**, because for that recording it is
  still true — and only the "not a GET" refusal is ever lifted, since a CLI run
  is still not a request and a path this app will not send is still not one.

  `spx::send` grew one optional argument rather than a second implementation,
  for the reason the replay already turns on: a replay differing from the
  original in the profiler cookie, the redirect policy or the certificate root
  would not be comparable with it. The captured `Cookie` is **appended to** the
  profiler's trigger rather than replacing it — the trigger is what makes the
  request produce a recording at all. A body over 64 KB is dropped by the bridge
  rather than truncated: half a body is not a request.

- **A monorepo is one project.** A repository with `api/` in Go, `web/` in
  Next.js and `worker/` in Python is now one entry, one start, one certificate
  and one set of hostnames.

  Every other tool in this category makes it three, and the reason is
  structural: their unit is a **site** — one directory, one runtime, one
  hostname — so a monorepo becomes three entries somebody has to remember are
  related, and nothing in the tool knows they are one thing. A local binary
  cannot do otherwise, because a directory has one runtime when the binary
  serving it has one. Containers are what make the other answer possible, and
  every piece of it was already here: eight runtimes, a Dockerfile renderer for
  each, Traefik routers generated per hostname. What was missing was a manifest
  that could say so.

  **Third of three, and the distinctions are the design.** `services` names a
  *need* the machine satisfies from a package and shares between every project
  that asked; a `sidecar` is *somebody else's image*, project-scoped, not built
  and not routed; a **component** is *this repository's own code* in a
  subdirectory, built here and routed at its own hostname. Folding any two
  together would make "how many of these exist" a question with two answers.

  **The containment is `sidecar.rs`'s and carries over exactly.** No host port —
  a published one would be two clones of one repository fighting over 8080, and
  a `ports` key is refused *by name* on the manifest with that reason. The
  `path` stays inside the project: `..`, an absolute path and `.` itself are all
  refused, because a build context is what Docker reads everything under. The
  container is named from the project by `sidecar::container_name` **itself**
  rather than a copy of it — they share one namespace, so an id used by both is
  a manifest warning instead of two declarations quietly producing one
  container. And it takes the project's own compose profile, so
  `--profile project-shop` brings the whole repository up.

  **A component's `domain` is deliberately not an alias.** An alias is another
  name for the project's *own* container and reaches its Traefik rule; a
  component domain names a *different* container, and merging them would route
  `api.shop.loc` to the PHP app. They meet in `Manifest::extra_hostnames`, which
  is the shape `/etc/hosts` and the certificate already wanted — so both get an
  entry and a SAN with no change to either module.

  **PHP is not a component runtime**, and is refused by name with the reason: a
  PHP part needs a web server, a document root and a `php.ini` overlay, which
  the project's own `runtime` already renders, which does not generalise to
  several inside one project, and over which differential fixtures are frozen.
  A component with no `domain` is not misconfigured either — it is reachable
  from the project's other containers and from nowhere outside, which is what a
  worker wants, and inventing a hostname for one would put a name in the
  certificate that nothing ever asks for.

  Everything is gated on the block being non-empty, so a single-runtime project
  renders byte for byte what it always did.

- **The same discipline `pkg.rs` applies to packages, applied to the project's
  own.** `pkg.rs` verifies every file of every service package against a
  manifest digest before it runs, refuses a moving tag, and checks a signature
  over the index it came from. Meanwhile the project sitting beside it pulls
  four hundred libraries out of `composer.lock` and `package-lock.json` and
  nothing here had ever looked at them — which is the wrong way round: the
  service packages are a catalogue this project publishes and can vouch for, and
  the dependencies are somebody else's code, in far greater quantity, running
  with the developer's own permissions.

  **The local half reads what the lock file already says, and needs no
  network.** A dependency fetched over plain `http://` is named package by
  package, because whoever is on the path chooses what arrives — `market.rs`
  refuses exactly that for this application's *own* catalogue, and a project
  doing it for four hundred libraries is the same hole at scale. Packages the
  lock carries no hash over are **counted** rather than listed, because on a
  lock written by an older tool that is every package and four hundred identical
  rows is a screen nobody reads. Every index other than the ecosystem's own is
  named with a count: a private mirror is not a fault, but a supply chain nobody
  wrote down is one nobody is watching.

  Direct and transitive are kept apart — the distinction the roadmap's own
  example turns on — and it comes from `composer.json` / `package.json`, because
  `composer.lock` does not record it. Dev dependencies are included and not
  marked apart: they are installed on this machine, they run in the same
  container, and *"it is only a dev dependency"* is a sentence that has
  introduced real incidents.

  **The advisory query is a separate command, and the reason is the
  disclosure.** It sends the **names and versions** of the project's
  dependencies to `api.osv.dev` — no identifier, no project name, no path, no
  file contents — and that is a real thing to say out loud, so it is written in
  `PRIVACY.md` in those words rather than described as "checking for updates",
  with `privacy_claims.rs` holding the document to it. The sentence sits above
  the button rather than after it, because a consent decision belongs where the
  decision is made. A failure is an **error**, never an empty list: "nothing was
  found" and "I could not ask" must not look the same on a security screen, and
  a null `advisories` is what the report already means by the second. Only OSV's
  ids come back — an id is what somebody searches for, and a severity word this
  build derived itself would be a judgement it is in no position to make.

  `locks` names the files that were read, so "no findings" can be told apart
  from "no lock file was found" — a project this cannot see is not a clean
  project. Only `composer.lock` and `package-lock.json` are read: the two this
  application's own runtimes reach for most, and the two that are JSON.
  `yarn.lock`, `pnpm-lock.yaml`, `go.sum` and `Cargo.lock` are named as absent
  rather than guessed at, the same call `images.rs` made about picking version
  pins — a parser written from memory against a format nobody measured is how a
  report starts quietly missing half a project.

- **What can leave this machine, as far as Docker can actually say.** An
  administrator who pointed every pull at the organisation's mirror with
  `registryPrefix` has a follow-up — *who bypassed it* — and the same person
  wants to know which containers on the laptop can reach the internet at all.
  Neither question had an answer, and neither has one anywhere else in this
  category: a local binary has no containers, so it has no network namespaces to
  separate one program's traffic from another's.

  **Whether a container can route out is a fact, not a heuristic.** A Docker
  network created with `internal: true` gets no gateway installed, so a
  container whose every network is internal cannot route out — asked of the
  daemon rather than inferred from behaviour. The column is deliberately
  asymmetric: one non-internal network is enough for *yes*, because one gateway
  is all a packet needs, and nothing short of **every** network being *known*
  internal earns a *no*. A network the daemon would not describe leaves the row
  at "cannot tell", because a containment claim resting on a failed lookup is
  the one wrong answer a report like this must not give.

  **Where each image came from** is the mirror-bypass answer, exact. A reference
  naming no host is reported as `docker.io` rather than as none — that is where
  it is pulled from, and "none" would omit the one host most worth naming. The
  rule that decides it moved into `images::registry_of`, which
  `policy::names_a_registry` now asks as well: two implementations of one rule
  is how a mirror and a report come to disagree about which references are
  already qualified.

  **It says out loud what it cannot see.** Docker keeps no connection log, so
  there are no destinations here — answering *"which host did this container
  talk to"* needs a packet capture inside the container's network namespace or a
  proxy in front of it, and this app will not install either quietly to fill in
  a report. The refusal is on the screen rather than left as a gap, for the
  reason `compliance.rs` gives about its own `unmeasured`. The byte counters are
  reported with the same honesty: they are per interface and include the StackVo
  network between the user's own containers, so they are a floor on *"did
  anything leave"* and never a measure of internet traffic. They are read with
  `one_shot` — bytes are counters, not a percentage, so the two-reading shape
  `container_stats` needs for CPU would turn a thirty-container report from one
  second into a minute — and a stopped container reports nothing rather than
  zero.

- **`git bisect`, with the environment the commit was written against.**
  `git bisect` moves the code and nothing else. Three months ago a project
  declared PHP 8.3 and locked `redis` at 7.0; the container on the machine today
  is 8.4 and 7.2 — so every step through that range runs **old code against a
  new environment**, and the commit the search finally accuses may be innocent,
  because the behaviour changed with the runtime rather than with the diff.

  Nothing in this category does anything about that, because nothing in it knows
  what environment a commit wanted. This does: `stackvo.json` has always
  travelled with the repository and, since the lock file above, `stackvo.lock`
  does too — and both are readable **at a revision**, with `git show
  <rev>:<path>`, without touching the working tree. So every step lists what
  that commit expected and how this machine differs. "Nothing differs" is a
  result rather than an empty space: it means the environment is not in the
  bisect, so whatever the search accuses is the code.

  **It reports the difference and does not install it.** The obvious next move
  is to make each step bring the environment along, and it is the wrong one:
  matching an old service version means replacing a container whose volume holds
  the developer's data, and a ten-step search would do that twenty times.
  Destroying a database to answer a question about a diff is not a trade this
  app makes on somebody's behalf. The mechanism to act is one click away on the
  Market page, and it asks first.

  Three refusals rather than fallbacks. **A dirty tree is refused before
  anything moves**, by name and with the repair — git refuses most of these
  itself, and a git error arriving in a desktop alert is a sentence written for
  a terminal. **A revision is validated against an allowlist**, the same hazard
  `git.rs` wrote its URL allowlist for: this is webview text reaching a
  subprocess argument, and a value beginning with `-` is read by git as an
  option, of which git has several that name a program to run. **Every move is
  audited**, because starting a bisect and marking a step both put the user's
  checkout on somebody else's commit, and *"my files are not what they were"* is
  the loudest question a developer can have about their own machine.

  `skip` is offered beside good and bad because git offers it, and leaving it
  out is worse than it looks: a commit that will not build has to be marked
  something, and without a third answer people mark it `good` — which poisons
  the search in a way nothing downstream can detect. The step estimate and the
  culprit are read back from git's own `BISECT_LOG` rather than recomputed, and
  the culprit is recognised on git's exact sentence and nothing looser: a bisect
  still running must never read as finished, because the finished screen is the
  one somebody copies a hash out of.

- **`stackvo.lock` — the versions a project was actually built against.**
  `stackvo.json` declares what a project needs *around* it — `"services":
  ["redis", "mysql"]` — and names no versions, which is right: a repository
  should not have to guess what the person cloning it will have installed.

  `verify.rs` had written the cost of that down in as many words: two machines
  can both satisfy `redis` while running 7.0 and 7.2, so *"a version the
  declaration does not pin is reported as `Ok` with the version it found written
  beside it, rather than being called a match it has not checked … saying which
  of those is right needs a lock file, which is a separate item"*. This is that
  file, and it is the division every ecosystem settled on for the same reason:
  **the manifest is intent, the lock is fact**, and both belong in the
  repository because the second is what makes the first reproducible.

  **`sha256` is what makes it a lock rather than a version list.** It is the
  digest of the package *manifest* as the registry stated it at install time —
  already recorded per instance in `instances::PackageRef`, already the thing
  `pkg::Tree::load` verifies every file against. With it, "redis 7.2" out of
  somebody else's catalogue is a different answer from "redis 7.2" out of the
  official one, which is exactly the substitution a version list cannot see. It
  is copied rather than recomputed: a second answer to a question that already
  has one is how the two come to disagree.

  **Written when somebody asks, never on its own.** A lock the app refreshed
  quietly would record whatever the machine drifted to, would therefore always
  agree with the machine, and could never disagree with it — and a check that
  cannot fail is worse than no check. `project_verify` only ever reports against
  the file; nothing in this path installs, switches on, or changes a version.

  `verify` gains three answers it could not previously give: `lockedVersion`
  (the wrong version is running, with both numbers, because one alone is not
  actionable), `lockedPackage` (the right version out of a different package)
  and `lockExtra` (the lock names a service the manifest has dropped, so the
  repair is to re-lock rather than to install anything). Without a lock the
  behaviour is unchanged, and a declared service the lock does not name keeps
  the old answer — `project_lock` already reported what it could not lock, at
  the moment it could not lock it.

  Deliberately **not** locked: the runtime and the web server, which
  `stackvo.json` already carries; and the images this application itself runs,
  which belong to the machine rather than to any one project — one `cloudflared`
  serves ten of them — and already have their pinning in the policy file's
  `imagePins`. The output is sorted and newline-terminated, so re-locking an
  unchanged workspace is byte-identical: a lock whose diff is noise is one
  people stop reading in a pull request, which is the only place it does its
  job. Also `stackvo lock <project>`, which is the form that belongs in CI.

- **Which clause of the policy is actually holding.** `policy_status` answered
  what an administrator's file *says*. Nothing answered whether any of it was in
  force on the machine reading it, which is the question the person who deployed
  the file has — and the two come apart for one reason that is not misconduct.

  **A policy arrives on a machine that was already set up.** The registry mirror
  rewrites image references as files are *generated*, so a project nobody
  regenerated since Tuesday still pulls from Docker Hub — on a network where
  Docker Hub is not reachable that is not a compliance detail, it is why the
  stack will not start. `allowedPackages` is checked as something is *installed*,
  so a service installed last month stays installed when the list that would have
  refused it lands today. `requireSignature` decides what the **next** refresh
  accepts and says nothing about the index already in the cache.
  `allowOverrides: false` stops a new override being made and does nothing about
  the files already on disk, which keep being read in front of the published
  package. Every one of those has a repair — regenerate, uninstall, refresh,
  delete — and until now nothing told anybody which, or where.

  **Four states, and the third is what makes it honest.** `holding`, `bypassed`,
  `silent`, `unmeasured`. Every list in the `market` block means *no opinion*
  when empty, never "none", so a report that folded silence into a green tick
  would score a machine with **no policy at all** as fully compliant — the most
  misleading thing a compliance report can do. `unmeasured` covers a fact the app
  cannot see *and* a clause with nothing to apply to (an `imagePins` entry naming
  a repository this build never runs is a line that does nothing, the same
  failure `policy.rs` already names for a locked key it does not also set), and
  neither is evidence of compliance.

  **`attestable`, not `compliant`.** `verify.rs` has the same shape and makes the
  opposite call — its `unknown` does not hold `ready` back, because a check it
  declined to make is not evidence a project is broken. That is right for *can I
  work?*. This asks *can somebody sign their name to this?*, where an unasked
  question is exactly what must not be swallowed, so anything `bypassed` **or**
  `unmeasured` holds the verdict back. The name is deliberate: the layer being
  reported on is not a security boundary — `policy.rs` says so in its first
  paragraph — and a certificate is not this app's to issue.

  Nothing new is measured and the engine is never asked: the `.env` as written
  (`Env::parse`, not `load` — load applies the policy last, so asking the loaded
  environment whether the policy took effect is a question whose answer is `true`
  by construction), the generated tree, the package directory, the remembered
  catalogue source, the override files, the project manifests. A compliance
  report you cannot run with Docker stopped is one you cannot run when you most
  need it. The mirror clause is answered by the mirror itself — if
  `policy::rewrite` over a file's own bytes would change them, it never reached
  that file — rather than by a second scanner that would be a second opinion
  about build stages and already-qualified registries.

- **The direction nothing went.** `secrets.rs` moves a password out of `.env`
  and into the OS keystore — the step somebody takes **after** they know there
  is a problem. The other one, *"there is an AWS key in your `.env` that is not
  in the keystore"*, and harder, *"that key is in a file git is tracking"*, did
  not exist, so nobody found out until the repository was already public.

  It matches the **value**, not only the key's name, and that is the whole
  design. `Env::is_secret` matches names by suffix, which is right for masking
  and not enough here — `preset.rs` had already written the reason down: *"a key
  added upstream tomorrow called `SERVICE_FOO_APIKEY` would sail straight
  through"*. So every rule is a shape its issuer publishes, and the name rule is
  kept as a second, independent net. Two nets that fail differently catch more
  than one net that is twice as clever.

  **There is deliberately no entropy heuristic**, and that is the decision that
  keeps the feature usable. A "long random-looking string" rule fires on
  minified JavaScript, on a hash in a lockfile and on a base64 image; a scanner
  people learn to ignore is worse than no scanner, because a miss costs one
  finding and a false positive costs the whole thing. A PEM header only counts
  for the block types that are actually private keys, so the certificates this
  repository ships are not findings.

  Two more decisions. **Nothing is put on a command line**: the obvious way to
  ask git whether a value was ever committed is `git log -S<value>`, which makes
  the secret an argument and therefore visible in `ps` to every process on the
  machine — so tracked files are read and matched in-process, and the only thing
  reaching git's argument list is `ls-files`. And **a finding never carries the
  value**: a report that quotes the secret is a second copy of it, on a screen
  people photograph and paste.

  `.env` being tracked outranks every other finding and is reported on its own
  terms — it means every value in that file is in the history whatever its
  shape, and no rule has to match for that to be true. The scan is bounded at
  2,000 files and half a megabyte each, and says how many it skipped: one that
  passed over four hundred files and said nothing would read as a clean
  repository.

  **Three refusals turned into answers.** A feature that says what it cannot do
  has not done the part it can:

  * *The value is never carried* — still true, and a finding now carries what
    every scanner in this field carries instead: a **fingerprint** (twelve hex
    characters of the value's sha256) and a **masked preview** (`AKIA…MPLE`,
    and only for values of at least sixteen characters — below that it is
    masked whole, because four characters at each end of a short password is
    the password). It identifies without revealing: two rows with one
    fingerprint are one secret in two places, which is the difference between
    rotating one key and rotating two. The fingerprint is taken from the
    matched run rather than the line, so the same key written `AWS_KEY=…` in
    one file and `key: "…"` in another is one secret and not two.
  * *Nothing goes on a command line* — still true, and the history question is
    now asked **by path**: `git log --all -- <path>`. A path is not a secret,
    and the answer is stronger than a value search — a file committed and later
    deleted is still in the history with everything that was in it, which
    `-S<value>` would miss the moment somebody rotated half of it. "Tracked
    now" and "committed at some point" are reported separately, because the
    second is the one people get wrong: untracking a file today does not take
    it out of the history.
  * *`.env` being tracked outranks everything* — and is now a **repair** rather
    than only a finding. `git rm --cached` (not `git rm`, which would delete
    the configuration the stack is running on), `git check-ignore` asked rather
    than `.gitignore` parsed, and `.env.example` written when there is none —
    the same keys, no values, comments and grouping kept, because an example
    file is documentation. An existing one is never overwritten.

  And the two halves the repair does **not** do are printed where it happened:
  the removal is staged rather than committed, so nothing has left the machine
  until somebody pushes it; and history is not rewritten, so if the file was
  ever committed, every value in it is still there and has to be rotated. A fix
  somebody thinks they made is worse than no fix.

- **"Trusted" was one word, and there is more than one store.** The category's
  leader has published that it is *exploring real certificates instead of the
  mkcert CA*, and it is worth being precise about what problem that is aimed
  at. It is not that a local authority is technically inferior — a
  locally-issued certificate is as valid as a public one to anything that
  trusts the root. It is that **the root is not trusted everywhere, and this
  app could not say where.**

  Firefox is the case that costs somebody an afternoon. It does not use the
  operating system's trust store; it carries its own, per profile. mkcert
  installs into it *if* `certutil` is on the machine, and prints a warning and
  carries on when it is not. So the ordinary state on a fresh laptop was:
  trusted by the system, working in Safari and Chrome, refused by Firefox — and
  one green `caTrusted: true` on the screen with nothing to act on.

  The certificate card now names each store and, where the answer is no, what
  to do about it. Three failing states are kept apart because their repairs
  differ: `certutil` missing (install nss, trust again), the CA absent from a
  store that read fine (trust again), and a store that could not be read at
  all. A store that answers neither yes nor no means that browser is not
  installed here, which is not a problem anybody has to fix and is not coloured
  like one.

  **ACME is not being built, and that is a decision with the measurement behind
  it.** A public authority validates control of a name in *public DNS*, and
  `shop.loc` is not in public DNS and never will be. HTTP-01 needs port 80 on
  this machine reachable from the internet; a laptop behind a router is not.
  DNS-01 avoids that and needs a real domain plus a provider API token, which
  is a real setup and not one a local development environment can assume. What
  a public certificate would actually buy is *other devices* trusting it
  without installing the CA — and for that the app already has nine tunnel
  providers, each terminating TLS with a certificate every device trusts. The
  help document says all of this where somebody would look for it.

- **The half of onboarding nobody builds.** Every tool in this category does
  the setting-up half — DDEV has its config and hooks, this app has presets.
  None of them does the **checking** half, which is the question somebody
  actually has an hour after cloning: not "how do I set this up" but *"I did
  set it up; why does it still not work?"* The repository already declares what
  it needs; what was missing was the sentence back. `stackvo verify <project>`,
  and a button on the project's page, answer it line by line.

  Nothing new is measured, which is why it is a pure function rather than a
  probe: four of the five facts are ones `projects_list` already computes —
  whether the manifest validated, whether the image was ever built here,
  whether the generated tree is older than `stackvo.json`, whether the domain
  is in the hosts file — and the fifth is the instance table. A second
  derivation of any of them could disagree with the first.

  **A declared service fails in three ways and they are three sentences.** In
  the catalogue and not installed: install it. Installed and switched off: turn
  it on — and the versions that *are* there are named, because "install it"
  would be the wrong instruction. A name this build has never heard of: that is
  a typo or a catalogue newer than the app, and it is reported as **unknown**
  rather than missing. Unknown deliberately does not fail the project: a check
  the app declined to make is not evidence that something is wrong, and a
  verifier that says "not ready" for a question it did not ask is one people
  learn to override.

  Every line is returned, not only the failing ones. A verifier that answered
  with nothing when everything matched would leave somebody unable to tell "it
  checked and I am fine" from "it did not check". And a version the declaration
  does not pin is reported with the version found beside it rather than judged
  — saying which one *should* be there needs a lock file, which is its own
  item, so this does not pretend to have checked it.

- **The verb the instruments were already waiting for.** `explain.rs` opens by
  saying that no new measurement is needed, and it was right about more than
  its own view: the recording that names a request, the sender that can issue
  one with the profiler on, and the observation window a query log is joined
  against all existed. What was missing was the act. A recording of a GET now
  carries a **send it again** button, which re-issues that exact request and
  answers with **both** reports and the difference.

  The loop it closes is the commonest one in performance work and it took four
  steps: change the code, open the site, find the page, come back and hunt for
  the new recording among twenty.

  Two decisions are worth stating. It runs *the same* request
  `spx_record_request` does — one function, not two implementations, because
  they would drift on exactly the details that make the replay comparable to
  the original: the profiler cookie, the redirect policy, the observation
  window. And there is deliberately **no verdict**: no `faster` field, no green
  tick. One run against one run is not a benchmark — a cold opcache and
  whatever else the machine was doing are inside the difference — and a boolean
  would invite a conclusion the measurement cannot carry.

  **Only a GET, and the refusal says why rather than hiding the button.** A
  recording names the request *line* and nothing else: not its headers, not its
  body, not the session it ran under, because nothing records those. A POST
  re-sent without them is a different request, and against any framework with
  CSRF it answers 419 rather than the page. A result that looks like an answer
  and is not is worse than a refusal — so the boundary of what a replay can
  honestly be here is stated on screen instead of being discovered.

- **What Docker cost you today, measured rather than argued about.** This
  README has a table saying what Docker costs and does not pretend otherwise —
  and until now that was an admission with no number behind it. Every rival
  built on containers carries the same cost and none of them measures it, which
  leaves "why is my laptop hot" to Activity Monitor and its nine processes
  called `com.docker.backend`. Being the one product that can say *"`shop` has
  held 4.2 GB·hours and used 38 minutes of CPU today"* is a better place to
  stand than denying it.

  **No new measurement.** `sample_container_stats` has read CPU and memory once
  a minute since the port, for the sparkline, and threw the readings away after
  two hours because a sparkline was all they were for. The same readings are
  now added up instead of discarded.

  **The interval is the part that is easy to get wrong.** A total is a rate
  times an interval, and the interval is the thing nobody has: the sampler runs
  on a sixty-second timer, so sixty is the tempting constant and it is wrong on
  every machine that has ever been shut. A laptop closed on Friday and opened
  on Monday would be billed for the weekend at its Friday rate. So the gap is
  measured per container, and a gap longer than five minutes contributes
  **nothing** — the reading still counts and the clock still moves forward,
  only the time is refused. The worst case is a total a few minutes short
  rather than one with a weekend in it, which is the same trade `stats_store`
  made for the same reason.

  **Shared services are never divided between projects.** `shop` and `blog`
  both use the same MySQL and any split of its memory between them would be
  invented; a number somebody could act on has to be one they can check. It
  gets its own row and says what it is, and for the same reason only a project
  can carry a budget. The stack's own containers are listed too — a total that
  quietly left out the router would understate what Docker costs here.

  A budget lives in preferences and not in `stackvo.json`, because it is a
  **machine's** decision: the same repository on a colleague's laptop has
  different room to spare, and a threshold committed to git would be one of
  them arguing with the other in a pull request. Passing one is announced
  **once per project per day** — the sampler runs every minute and a project
  that is over at two o'clock is still over at half past, so a notice per round
  would be four hundred by the evening, and the feature somebody switches off
  within the hour is the one that would have told them tomorrow. A budget of
  zero is no budget, because a cleared field arrives as a zero and an alert
  that fires the moment somebody clears the box is one they will never leave
  on.

- **"It works on my machine", answered by measurement.** It is the oldest
  complaint in this category and every product in it says the container solves
  the problem — which it does not: the same compose file on two Docker versions
  is two different things. This app was already the only one that packages the
  state of a machine. What it could not do was put two of them side by side.

  It could not because the bundle is written for a **person**: `about.txt` is
  prose and `doctor.json` is shaped for reading, so diffing them produces noise
  — a socket path, a pid, a byte count that moved — and a comparison whose
  output is mostly noise is one nobody reads twice. So the bundle now carries
  `environment.json` as well: one line per fact, a key somebody can say out
  loud, and a value that is the same string on two machines when the two
  machines agree. Versions, the engine, every service instance and version and
  whether it is on, and what each project declares — its runtime, its version,
  its server, whether Xdebug is on.

  Two things are deliberately **not** in it. No paths: a home directory differs
  on every machine and would report two identical setups as different in five
  places. No credentials and no `.env` values, which is the same reason the
  bundle it sits in is safe to attach at all — and a test asserts both rather
  than trusting the list.

  Settings takes a bundle a colleague sent and lists only what the two machines
  disagree about, counting the rest: a report that prints two hundred identical
  lines buries the four that matter. A fact only one side states is a
  difference with a missing half, because *"you have redis-7-2 and they do
  not"* is a different sentence from *"you are on 7.2 and they are on 7.0"* and
  an intersection would have dropped the more useful one. Their file is read
  first — a file that cannot be read is the answer, and there is no reason to
  walk this machine to find that out — and it is compared against what this
  machine is **now** rather than a stored copy of itself, because the question
  is always what is different now. A whole zip or the `environment.json`
  somebody pasted out of one, tried in that order rather than guessed by
  extension, so renaming a correct file does not make it unreadable. A bundle
  older than the file says so by name instead of comparing nothing.

- **A worktree with an expiry, and the registration that scopes an assistant to
  it.** The three pieces were all in the tree and nothing joined them: a branch
  could have its own directory, hostname, database and environment; the MCP
  server could be bounded to one project for a fixed time; and neither knew
  about the other. Creating a worktree now takes a duration, and a branch that
  has one is a *sandbox* — something built for one task by somebody who is not
  going to remember it exists.

  The card for such a branch prints the flags that give an assistant that
  branch and nothing else, rendered from the same `Grant` the server enforces
  rather than assembled on screen. Under that scope the twelve writing tools
  are the four a project can bound, and stopping the whole stack is not one of
  them. Once the expiry has passed the registration is **empty** rather than a
  grant with no duration on it — dropping a spent `--for` would silently hand
  out an unlimited one.

  **Nothing is deleted on a timer, and nothing ever will be.** An app that
  removed a directory by the clock would eventually remove one with a morning's
  uncommitted work in it, and no expiry policy is worth that. What the date does
  is let the list say the time has passed, so removing it stays one click and a
  decision. The output survives either way: a sandbox's product is the
  **branch**, and the database is scaffolding.

  Two details worth stating. The expiry is recomputed at creation rather than
  taken from the plan — a plan made minutes ago on a screen somebody was reading
  should not spend part of the time it was asked for. And "has it passed" is a
  string comparison, which is correct only because these timestamps are
  fixed-width UTC; the one place that needed real arithmetic — how many minutes
  are *left*, which becomes the grant's `--for` — got `days_from_civil` beside
  the `civil_from_days` this app already had, and the pair is tested against
  each other rather than against a table somebody typed.

- **"Its own database" and "cannot reach the parent's" were two promises and
  only the first one was kept.** A worktree gets a database on the instance
  that is already running — and it also got the *instance's* login, the same
  account the parent project uses. So the parent's data was one `USE shop;`
  away from the branch. That was a small thing while a worktree was somebody's
  second branch. It is the whole claim once the thing working in there is an
  assistant that was told to fix a failing test and decided a migration would
  do it.

  A worktree now gets a database account granted on its own schema and nothing
  else. On MySQL and MariaDB that is the complete answer: the account cannot
  read another schema, `SHOW DATABASES` does not list one it has no grant on,
  and the grant covers tables the application creates later — which matters,
  because the data arrives after the account does.

  **PostgreSQL and MongoDB are refused rather than approximated.** Postgres's
  model is not that shape: `GRANT ALL ON DATABASE` grants connect, create and
  temp, not one row, and the tables in a copied database are owned by the
  superuser that restored them — so the grants have to be applied to the
  objects inside that database afterwards, plus default privileges for what
  comes later. Half of that produces a branch whose application cannot read its
  own tables. Mongo publishes no database name to scope to, which is the same
  reason `copy_database` refuses it. On both, the branch keeps the shared login
  and **the app says so**: `worktree_list` reports `isolated`, the pane prints
  the sentence, and the help document names the two engines. An isolation that
  is claimed and not arranged is worse than one that was never offered — it is
  the one somebody stops checking.

  The password lives in `worktree-logins.json`, `0600`, and **not** on the
  worktree record. `Table` is serialised by one derive to two places — the file
  and, through `worktree_list`, a webview — so a credential on `Record` would
  ship to the browser with it, and the only defence would be remembering to
  strip it at every call site. A field that must never reach the webview does
  not live in a struct that goes there.

  Three smaller decisions. The account name is the database name, truncated to
  MySQL's 32 characters **with seven hex characters of the full name's digest**
  — two branch databases of one project share a long prefix by construction, so
  truncation alone would have handed the second worktree the first one's login,
  silently. A server that will not create the account does not fail the
  worktree: that would take a working feature away from anybody whose database
  user cannot `GRANT`, and the honest alternative is to say which login the
  branch got. And the account is dropped on removal **whatever was decided
  about the data** — keeping `dropDatabase` off means "leave my data alone",
  not "leave behind an account that can reach it".

- **The trail was written from eighteen places and the assistant surface was
  not one of them.** `audit.rs` exists for "whoever has to account for the
  machine", and *"`stackvo_stack_down` was called at 14:32"* was a sentence
  this app could not produce — about the only caller that is not a person. Now
  every writing call over MCP is recorded, refusals included, and a refusal is
  usually the line worth having: an assistant that tried to stop the stack and
  was told it may not is exactly what somebody reviewing a grant is looking for.

  That widens a bar the module set deliberately — `project_start` is excluded
  there because pressing the button again undoes it. The exclusion was never
  about the act; it was about the actor. Starting a container from the window
  is done by the person reading the trail, who watched it happen. The same act
  asked for by an assistant is the one nobody saw. The recording therefore
  lives where the act *has* an actor — the function the server binary runs —
  and not in the dispatch that the loopback surface and an undo also go through.

  **And they can be put back.** Each recorded act carries a compensation
  computed **before** the tool ran: what `stack_down` stopped exists only
  before it stopped it, so a plan worked out when somebody presses Undo would
  be worked out against a machine that has since changed. Stopping the whole
  stack is undone by starting what was running — services first, then projects,
  because a project that comes up without its database comes up broken.

  **Most acts have no compensation, and saying which is the feature.** A
  restart has already gone through the state an undo would return to; generate
  overwrote output that was not kept, so the repair is the input; a reissued
  certificate replaced one that was not kept either; a snapshot added a file
  and changed nothing. Each line carries its own sentence, and the pane shows
  that sentence rather than a button that would not keep its promise.
  `stack_up` is refused for a subtler reason: which containers it actually
  started is not knowable before the call, and stopping everything that was
  down would name containers it never touched.

  Two properties of the file did the rest of the work. It is append-only, so an
  undone act is **not edited** to say so — the undo names the line it reversed
  and the reader joins the two, which keeps both halves of the record: that it
  happened, and that somebody put it back. And an undo is a sequence rather
  than a transaction: if the fourth of six calls fails, the first three stay
  done and the trail says where it stopped.

- **`--allow-writes` was one switch over twelve tools, and one of them stops
  everything.** The README warned about it in a paragraph — *"read that list
  before passing the flag"* — which is what a warning is for when the code
  cannot express the limit. A person asked what they actually want to allow says
  a sentence with three parts in it: *this assistant may restart `shop`, for the
  next half hour.* The flag could express none of them.

  There are three limits now, and each is there because the other two do not
  cover it. `--allow=project_restart,project_start` names the tools instead of
  counting them. `--project=shop` bounds the server to a project. `--for=30m`
  ends the writing half that long after the server starts, because an
  assistant's session outlives the task it was given. The Settings pane writes
  the same flags, `stackvo mcp-install` takes them too, and the preview under
  the controls is exactly what lands in the client's configuration file — that
  line is the record of what an assistant was allowed to do.

  **A scope removes what it cannot bound.** `--project=shop` cannot make
  `stackvo_stack_down` safe: the tool takes no project and stops every container
  there is. So a project scope does not narrow such a tool, it removes it —
  twelve writing tools become the four a project bounds (`xdebug_set`,
  `project_start`, `project_stop`, `project_restart`), and the eight that no
  project bounds are not offered at all. A surface that accepted `--project` and
  still served `stack_down` would be reporting a limit it was not applying,
  which is worse than having no limit: the person who set it stops watching.

  **Reads are bounded too**, and that was a decision rather than an oversight.
  The tempting rule is "the scope bounds writes; reading is harmless", and it is
  wrong here — `stackvo_explain_request` returns another project's request
  traces and queries, and `stackvo_log_read` returns its log files. Somebody who
  said "this assistant works on `shop`" did not say "and may read what the other
  eleven projects did". The project listings report what is in scope rather than
  naming what is not, since a list of what may not be touched is an invitation
  to try. `stackvo_logs` and `stackvo_container_stats` stay open: their argument
  is a container, as often a service as a project, and refusing `redis-7-2` for
  not being `shop` would be a limit nobody could act on.

  **And it is not information isolation**, which matters more than the feature
  because describing it as one would be the dangerous half. Every machine-wide
  instrument still answers — the doctor, the hosts table, the mail catcher, a
  database service's query log, a container's log by id — and some of those
  carry another project's name or content. Bounding them would leave a scoped
  assistant unable to diagnose the project it *was* given, which is what the
  surface is for. What the scope buys is that no per-project instrument answers
  for a project it was not given, and that twelve writing tools become four.

  Two smaller decisions. The grant filters the advertised **list**, not only the
  call — a tool advertised and then refused is one an assistant retries, reports
  as broken and works around. And a flag this server does not recognise **stops
  it from starting**: a mistyped `--project=shpo` would otherwise be a server
  that quietly grants nothing while its configuration file says otherwise.

- **This app forbade moving image tags and then used six of them.** `pkg.rs`
  refuses to install a package whose version is `latest`, `stable`, `edge`,
  `main` or `master`, and says why: *"an image that changes under a fixed
  manifest has no digest the manifest can pin"*. Six of the ten images StackVo
  runs and did not build are on `latest` — the tunnel providers, the SSH client.
  The rule applied to everybody except this application, and the day one of
  those publishers ships a broken build it arrives on every machine at once,
  with no version to go back to.

  The tags are **not** pinned in the source, and that is deliberate: choosing a
  pin means naming a version that exists, and a source file cannot verify that a
  tag or digest is real. Writing one in would replace a known-moving reference
  with an invented fixed one, which is worse. Picking the pins is a release-time
  act against a registry.

  What is fixed is that **there was nowhere to pin.** The ten lived as literals
  in four modules, appeared in no interface, no `.env` and no policy file, and
  nothing could say which of them moved. They are one table now, every one is
  overridable through a policy `imagePins` block keyed by repository, and
  Diagnostics shows what this machine will actually pull with the moving ones
  marked.

  Three decisions in it are worth stating. **The pin is applied before the
  registry mirror** — the other order prefixes the reference first, so the pin
  would then be looked up under a repository with the organisation's host in
  front of it and would stop applying on exactly the managed machines that set
  one. **A pin naming a different repository is refused and reported**:
  `"nginx": "alpine:3"` is a typo, and silently running something else is worse
  than the moving tag it was meant to fix. And **"moving" is computed after the
  pin**, so a row an administrator has fixed stops being flagged — one that
  stayed flagged would be a screen that lies about a setting that worked.

- **The app tells you it crashed.** It could crash, write a perfectly good
  report, and never mention it — so "I would like to report this" never started,
  not because reporting is hard but because nobody knew there was anything to
  report. There is a line on the next launch now, once, with a button that opens
  the diagnostic bundle.

  Nothing leaves the machine, and that is not an omission: `PRIVACY.md` promises
  no telemetry, no crash reporting service and no server behind the app, and
  says anything future would be opt-in, off by default, and described there
  before it ships. The reports already travelled in a diagnostic bundle; what
  was missing was knowing one existed to send.

  Reports are compared by name rather than by count — ten are kept, so a pruned
  one would make a count go down and a newer crash then go unmentioned — and the
  "seen" marker is written when somebody dismisses or follows the line, not when
  it is read. Marking on read dismisses a notice nobody looked at, which is the
  same as not having one.

- **An introduction, once, after everything that is an obstacle.** There were
  four full-window screens and every one of them was a gate: something missing
  or unassembled, and the app unusable until it was dealt with. None of them
  introduced anything, and being stopped four times on the way in is not an
  introduction — on a surface of 26 panes and 317 commands, discovery is harder
  here than in a menu-bar app, not easier.

  Six cards, and the selection is the point: the things **nobody would find on
  their own**. Importing from seven other tools, a full environment per git
  branch, explaining a slow request, building the production image, exporting a
  devcontainer, and the audit record. All six are written and tested, and four
  of them appear in no user-facing document at all. A tour of the dashboard
  would introduce the part that introduces itself.

  It arrives after the stack is up, can be left at any point, and leaving it is
  not a failure state. `tourSeen` is a preference rather than a session flag,
  because "once" has to survive a restart or it is not once — and Settings can
  show it again, because a one-shot screen somebody skipped in their first
  minute is one they can never get back.

- **RoadRunner, which is Laravel Octane's other driver.** Octane has exactly two
  — Swoole and RoadRunner — and this application shipped one. For a project
  using Octane that is not a missing feature, it is a coin toss it can lose:
  half of them chose the half that was not here.

  The shape is Swoole's: the `php-cli` image, its own HTTP server on 8000, no
  FPM, no directory listing, and Traefik pointed at 8000 rather than 80. The
  difference is the cost. Swoole is a PHP extension that has to be compiled into
  the interpreter — a PECL module and two dev libraries — while RoadRunner is a
  Go binary that speaks to PHP over a pipe, so nothing about the PHP build
  changes for it. The binary is resolved by Octane's own installer script rather
  than pinned to a release asset, because the image is built on whatever the
  developer's machine is.

  The fallback matters more here than it does for Swoole. Without `artisan`
  there is no Octane, and RoadRunner without Octane wants a `.rr.yaml` and a
  PSR-7 worker this application cannot write for somebody — so a non-Laravel
  project gets PHP's own development server on the same port: a working site
  rather than a container that exits. Swoole can fall back to a twelve-line
  script because the extension *is* an HTTP server.

  It generates no configuration file of its own, and not because it needs none:
  Octane publishes a `.rr.yaml` that belongs to the project, and writing a second
  would be this application arguing with the framework's own file.

- **Podman is recognised, rootless first.** It already spoke the Docker API on a
  unix socket, so everything this application does already worked — the only
  thing missing was the path. `$XDG_RUNTIME_DIR/podman/podman.sock` is searched
  before the system sockets, because rootless is the case somebody runs Podman
  for and the one Lerd sells its whole product on; `/run/podman/podman.sock`
  covers a rootful daemon.

  `XDG_RUNTIME_DIR` is read rather than assumed. `/run/user/<uid>` is what it is
  on nearly every Linux, and "nearly" is the failure: the variable is what
  defines the directory, so a session that puts it elsewhere would get a path
  that does not exist while the socket sat somewhere nothing looked.

  The engine name is a label and never a branch — nothing here does anything
  different because of which of the four answered — and that is also what makes
  a non-Docker engine work at all: no version string is compared against a
  minimum anywhere. Podman answers `Version: 5.1.1`, which is Podman's own, and
  `ApiVersion: 1.41`, the Docker level it emulates; a check for "Docker >= 20"
  would have refused an engine that speaks the API perfectly well on the
  strength of a version string from another product. A fixture of Podman's real
  `/version` answer holds that.

  One limitation is written down rather than closed: `podman-docker` installs
  Podman's socket under Docker's name, so it reads as Docker Engine. It works,
  and reporting it correctly would mean asking the daemon what it is — an answer
  that would change nothing this application then does.

- **A preset now has a place to live, so a clone brings the stack as well as the
  project.** `preset.rs` had solved the right problem — a teammate clones and
  gets `stackvo.json`, but not which of the twenty services are on and at which
  versions, because that is in `.env`, the one file nobody commits, since it is
  also where every password is. What was missing was that the file had nowhere
  to *be*: the export wrote one, the import read one, and between them somebody
  had to say out loud where they had put it. That sentence is what the feature
  exists to remove.

  It is `<project>/stackvo.preset.json` now, beside the manifest, in the
  repository. The requirements card sees one and says what applying it would
  change; the button goes through `preset_apply`, the same plan-then-apply
  command the Settings import uses, because a file that arrived with somebody
  else's clone must not rewrite your stack because you opened a page. The line
  disappears once your stack matches — a permanent banner is one nobody reads by
  the third visit, and the moment it matters is the first open after a clone.

  A divergence turned up on the way: the export suggested
  `<name>.stackvo-preset.json` while nothing looked for that name. Two spellings
  of one idea, only one of them read — an export writing a file no reader looks
  for is the feature quietly not working. One name now, held level from both
  sides.

- **A file at the root of your workspace adds commands to every project in it.**
  Four rivals sell "add your own command" on the front page — DDEV drops a file
  in `.ddev/commands/`, Lando has a `tooling:` block, dde and Laragon each have
  their own — and this application had exactly one way to do it: edit a
  repository somebody else owns. A command you run in all of them is exactly the
  one no single project should have to carry.

  `commands.json` is the same schema as a project's `commands` block, the same
  argv rule and the same container boundary, read by the **same parser** —
  deliberately not a second shape and deliberately not a second threat model. It
  is the union of two decisions already taken: a file on disk may declare a
  command, and a declared command runs in the project's container and nowhere
  else. There is no `host` form here either; a step that has to run on this
  machine is a hook, approved against a digest first.

  Between the two declaring layers **the project wins, and the machine file is
  told.** Refusing a committed, shared file because of a personal one its author
  has never seen would be hostile; overriding it silently is the
  button-that-lies failure the catalogue rule already refuses. So every row
  carries the file it came from, and the pane names it.

  The pane is read-only, on purpose: the file is the interface, the way it is in
  every product named above, and a form would be a second way to write the same
  JSON that disagreed with your editor the first time you used one. What it owes
  instead is the other half — where the file goes, what it found, and what it
  refused — because a file-shaped interface with no feedback is one you edit
  twice and give up on. `stackvo commands` and `stackvo run` pick it up without
  a change: both already went through the one resolver.

- **The four application pickers have a row that says "Other…", and it is a
  command line you type.** `apps.rs` detects terminals, editors, browsers and
  database clients, which is a real improvement on the free-text box it
  replaced — that box asked you to know a launcher's name. But the box was
  removed without anything going in its place, so the lists were closed sets,
  and somebody running Helix, Emacs or one of the eight unlisted JetBrains IDEs
  opened the preferences pane and found nothing they could select. Not a worse
  choice: no choice.

  (Two of the three examples the roadmap gave were wrong, and checking came
  first: Neovim and Vim have been in the editor list since it was written.)

  The row is in every list, always selectable, and its command lives beside the
  choice in `preferences.json` — `editorCustom`, `terminalCustom`,
  `browserCustom`, `dbClientCustom`. Two keys and not one: `editorCommand` says
  *which*, `editorCustom` says *what to run*, and a single string would make
  "go back to VS Code" mean losing the command you typed.

  Two properties it keeps deliberately. **It is never the automatic fallback** —
  the row is appended after the default is marked, and the walk of installed
  editors skips it — because falling back to a command nobody chose is worse
  than not falling back. **And there is no shell**: the line is split on quotes
  and handed to the OS unexpanded, so `$HOME`, `&&`, a pipe and a redirect are
  literal text. A backslash is not an escape either, because the commonest thing
  anyone will type on Windows is a quoted absolute path.

  The terminal gets its own type, because a terminal is handed a *command line*
  rather than a path and every emulator takes one its own way. One rule: your
  words first, the command appended as a single trailing argument. The flag goes
  in the box with the launcher rather than being guessed — `-e`, `--`,
  `start --` and `/K` are four different answers, and a wrong guess opens a
  terminal running the wrong thing.

- **The version lists every picker draws from are a setting now, not a
  release.** Six `.env` keys — `SUPPORTED_LANGUAGES_{PHP,PYTHON,GO,RUBY,RUST,NODEJS}_VERSIONS`
  — decide what the runtime and PHP pickers offer, and nothing in the interface
  could edit one. They ship compiled in and go out of date the moment the world
  ships a release: Go stopped at 1.23, Ruby at 3.3, Node at 23, Rust at 1.84.
  Selecting a version this application had never heard of meant editing `.env`
  by hand, or waiting for a release of the application. For a number in a list.

  The runtimes group now carries a disclosure with one chip editor per list, in
  the shape `PHP_DEFAULT_TOOLS` already uses. A version the registry does not
  have fails when the image is built, with the engine's own message — the same
  bargain that control already makes.

  **The roadmap's own diagnosis needed correcting first.** It read "seven of the
  36 settings keys never appear in `src/`" as "nothing reads them", and that is
  half a fact: `build_catalog` composes the key with
  `format!("SUPPORTED_LANGUAGES_{key}_VERSIONS")`, so a key-by-key search finds
  nothing while the chain — `.env` to catalog to picker — works perfectly. The
  lists were not dead. They were load-bearing and unreachable.

  A test holds the two lists level, because the bug is one edit away from
  returning: a seventh runtime added to `config::SETTINGS` is read by
  `build_catalog` and offered by the picker immediately, and would be editable
  by nobody, since the pane iterates a list written by hand in JavaScript.

- **Twelve hundred lines of domain logic left the command layer for the
  modules whose subject they are.** `ARCHITECTURE.md` had already weighed
  splitting `commands.rs` and declined: the file's size is uncomfortable rather
  than harmful, and moving three hundred thin functions into a directory
  touches every command at once. Measuring it again — it had grown from 12.7k
  to 16.1k since that was written — confirmed the decline and found something
  else.

  The file's longest items are not commands. Five private helpers of a hundred
  to three hundred lines each take a path in and return a plain value, with no
  Tauri type between them, which makes them domain logic living one band above
  its subject — and the cost is the band rule's own promise, that everything
  below the command layer "can be called from a test, from the `diagnose`
  example, or from the MCP surface". Three of the five had callers in `mcp.rs`,
  `cli.rs` and `examples/diagnose.rs` reaching *up* into `commands` to get at
  them, which is the dependency arrow drawn pointing the other way.

  So the whole generated tree (render, verify, write) is now `generator`'s, the
  migration preview is `handover`'s, the worktree plan is `worktree`'s, and "a
  project's directory and its manifest" is `workspace`'s. `commands.rs` is
  14,850 lines from 16,070, all 308 commands are still in it, and not one moved
  line takes Tauri state — the gate that holds that rule reads the same and now
  holds in both directions.

  Each move arrived with the tests it had and left with more, because the code
  is reachable now. Three of those tests could not have been written a commit
  earlier: that a migrated workspace is not planned a second time, that a
  refused worktree plan still carries the name and hostname it would have had,
  and the two that were reaching across a band boundary for `scope_includes`
  and `service_source`. The first was written twice — its first version passed
  with the refusal deleted, because an empty package tree makes the planner
  produce nothing either way, so it was thrown away and rewritten where a real
  catalogue exists.

- **The build ships the icon rules this application can reach, and two of the
  ones it named did not exist.** Material Design Icons declares 7,447 glyph
  rules — 408 KB of stylesheet — and this application names 313 of them. The
  eager set was 1547 KB against a 1700 ceiling and every asset together was
  3001.5 against 3000, which is to say the bundle budget was not tight, it was
  breached. It is now 1246 and 2701, and the eager ceiling has come *down* from
  1700 to 1400: 36% of slack over a measurement is where a budget stops being
  an alarm.

  The trim is the one `totalKb`'s own comment named as the thing it exists to
  catch — "a dependency arriving by accident: a full icon font" — and it needed
  no new dependency, because the stylesheet is text. The font itself is still
  whole at 394 KB; subsetting a woff2 wants a font toolchain, so it is written
  down as the next piece rather than done badly.

  **And reading the icon set against this tree found two names that are not
  icons.** `mdi-discord` has been gone for two major versions — the set dropped
  its brand marks — and `mdi-format-textdirection-r-to-l` was renamed, so the
  Discord row of the community links and the heading of the Localisation
  panel's direction group had both been rendering a blank square. A missing
  glyph is the quietest failure a screen can have: nothing warns, and the gap
  reads as a spacing decision.

  Which icons count as used is now one list with three readers — the plugin
  that strips, the bundle gate that holds the built stylesheet against it after
  the build, and a test that holds it against the icon set. Vuetify renders 26
  icons this repository never writes down (the checkbox marks, the sort arrows,
  the pagination chevrons); a list built from `src/` alone ships an application
  whose checkboxes are empty, which is what its first version did.

- **A nightly asks the one question CI cannot: does a project actually come
  up.** `tests/docker_smoke.rs` lays out a throwaway workspace, writes a
  manifest, renders it through the application's own `write_generated`, runs
  `docker compose up --build`, and asks for the page twice — from inside the
  container, which proves nginx, php-fpm, the bind mount and `document_root`,
  and through Traefik over HTTPS, which proves the label the generator wrote
  became a router. It prints `PHP_SAPI`, so a container serving the file as
  static text fails instead of passing with the source on screen.

  **It refuses to run beside a live stack rather than skipping.** The generated
  compose project is called `stackvo` on every machine, its containers are
  `stackvo-*` and its network is `stackvo-net`, so a second stack is not a
  second stack — `--remove-orphans` against a developer's own installation
  removes the containers it did not put there. That refusal is why this could
  not be run on the machine it was written on, and it is why it is a check
  rather than a note.

  The half that needs no engine — the workspace renders a service, at the right
  domain, with a Dockerfile in the build context, and every file
  `compose_file_list` would hand compose exists — runs in the ordinary suite,
  so a rename fails at the commit rather than at three in the morning. Two
  gates in `workflow_parity.rs` keep the nightly from drifting: its Linux
  packages must be a superset of CI's, and the sentence its job greps for must
  be the sentence the test prints — otherwise green and never-ran look the
  same, which is the failure the `driver` job already carries a comment about.

- **The debug bridge writes three kinds of signal, not one, and a queued job's
  `dump()` finally reaches it at all.** `Event::kind` was written before there
  was a second value for it, so that a second signal would need no second file
  and no second reader. Two arrived and it did not: a `request` — one row per
  execution, with the HTTP status and how long it took — and a `job`.

  The request row is raised from `register_shutdown_function`, not from any
  framework's "request handled" event, and that is the point rather than a
  fallback: PHP itself guarantees the hook, it runs for a fatal and for an
  `exit()` as well as for a clean return, and it needs nothing loaded. So the
  row appears for Laravel, for Symfony, for WordPress and for a hand-written
  `index.php` alike. It is registered at prepend time, which puts it first on
  the shutdown stack — measured to matter, because an `exit()` inside a
  shutdown function skips every handler behind it.

  **The obvious way to add jobs does not exist, and that was measured rather
  than assumed.** Four listeners on Laravel's own queue events cannot be
  attached from an `auto_prepend_file`: the container does not exist yet, and
  watching the autoloader for the queue's classes fails because **Composer
  registers its own loader with `$prepend = true`** and lands in front of
  anything the prepend file registered. Against a real Laravel 12 checkout,
  `spl_autoload_functions()` comes back `[Composer…, <the bridge>]` in that
  order. So jobs are read the way queries are read — out of the producer's own
  record. The new `queuelog` parses `queue:work`'s own two-column output and
  folds it into the same events file, on the **engine's** clock rather than the
  timestamp the worker printed in whatever timezone its image was built with.

  **And the worker never carried the bridge.** It reaches the web container
  through a compose overlay, and a queue sidecar is started with `docker run`,
  which was giving it one bind mount. A `dump()` inside a queued job was
  therefore written by nothing and read by nobody — while the module's own
  header and `debug_bridge_set`'s contract note both said it worked. Measured
  by running the same job against a container with the mounts and one without;
  `worker::run_args` now passes all three, the ini as a *file* mount so it does
  not shadow every other ini in `conf.d`.

  Two smaller things the running found. `spl_autoload_register`'s `$do_throw`
  makes PHP 8 print a notice when it is `false` — which a prepend file would
  put into everybody's response. And the first job run after switching capture
  on was swallowed: with no cursor the seed took "the worker's newest line" as
  history to skip, and that line was the job. A worker that has printed nothing
  now seeds at zero, and the seed happens at the switch rather than a poll
  later.

- **Every platform's bundler is given an icon in the format it demands (§3 #22).**
  Both Windows rows of the first rehearsal — x86_64 and ARM64 — died on
  `Couldn't find a .ico icon`, after the application had finished compiling.
  `icons/icon.ico` and `icons/icon.icns` were in the repository the whole time;
  `bundle.icon` named only `icons/icon.png`, so the bundler was never told about
  them.

  It stayed invisible because the failure is asymmetric: macOS *generates* an
  `.icns` from a PNG when it has to and Linux uses the PNG directly, so two
  platforms were quietly fine and the third was a hard error — and the third is
  the one nobody runs. Read off the run page, `windows-11-arm` looked like an
  ARM problem. It had nothing to do with ARM, and `windows-latest` was failing
  in exactly the same place. A test holds both halves now: the list must name a
  `.ico` and an `.icns`, and every name in it must be a file — a list naming an
  icon it does not ship fails the same build one step later.

- **The bundler is answerable on this machine, and a release run stops being a
  test environment (§3 #22).** Proven on its first day: two rounds of
  `tools/linux/run.sh --bundle` took down two walls in a row and the second was
  one no workflow could have declared. `xdg-utils` was the runner's find;
  `appimagetool` then asked for `file`, which is in no workflow and does not
  need to be, because a GitHub runner image ships it. So the image has to be a
  superset in two directions at once — every package the release declares, plus
  everything the runner gives the release for free. The run ends with
  `aarch64-unknown-linux-gnu: deb, rpm, appimage — each present and named for
  aarch64`: all three formats, right architecture, no runner spent. The reason a screenshot of a log became the way
  this repository learned about `xdg-utils` was narrow and entirely fixable:
  every mode in `tools/linux/run.sh` compiled or tested, and **none of them
  bundled**. `ci.yml` never bundles either, and the container was mirroring
  `ci.yml` — so the one list nothing held the image against was the only list
  describing the job it exists to rehearse.

  Three things follow. `tools/linux/run.sh --bundle` builds the installers the
  container's own architecture ships and runs `check-installers.mjs` over them —
  and on an Apple Silicon machine that container **is**
  `aarch64-unknown-linux-gnu`, the row that failed, so that failure was always
  reproducible here and no command existed to reproduce it.
  `--windows-bundle` cross-builds the NSIS installer through `cargo-xwin`, for
  `x86_64-pc-windows-msvc` or `aarch64-pc-windows-msvc`; the MSI cannot be built
  off Windows and that is `tauri-bundler`'s own division — its `msi` module is
  `#[cfg(target_os = "windows")]` while its `nsis` module says outright that it
  builds on Linux and macOS. NSIS is the half worth having anyway: the updater
  downloads the `-setup.exe`, never the `.msi`. Both are wired into
  `tools/before-push.sh --all`, whose first line claims everything CI asks is
  asked here first — a claim that is false is worse than absent, which is that
  file's own argument for existing.

  And the rule that keeps it true: `workflow_parity.rs` now holds
  `tools/linux/Dockerfile` against **release.yml**'s package list rather than
  CI's. The direction is the one that file already argues, one link further
  along — the container must be at least the release environment, never less.

- **Cross-building to Windows works here, and the one row that cannot was
  measured rather than assumed.** `--windows-bundle` produces the NSIS installer
  through `cargo-xwin`, and two missing *names* stood in the way: Debian ships
  clang's cl-compatible driver and not the name `clang-cl` — the driver picks
  its mode from `argv[0]`, so a symlink is the whole fix — and `llvm-ml`, which
  reads the MASM assembly `ring` and `aws-lc-sys` ship on x86_64. Without them
  `cc-rs` falls back to plain `clang` and every C file dies on
  `no such file or directory: '/imsvc'`, an error that names a flag as if it
  were a path. The MSVC CRT `cargo-xwin` downloads now lives in a volume instead
  of being fetched on every run.

  The last name in the way was NSIS itself: on Windows the bundler downloads its
  own and runs `<toolset>/makensis.exe`, everywhere else it runs plain
  `makensis` off the PATH and does not carry it. The error still says
  `makensis.exe`, so an `apt-get install` away reads like a cross-compilation
  dead end. With it, `x86_64-pc-windows-msvc` produces
  `StackVo_0.1.0_x64-setup.exe` here. Five findings of one shape in a day —
  `xdg-utils`, `file`, `clang-cl`, `llvm-ml`, `nsis` — all of them the
  *bundler's* dependencies rather than the compiler's, and none of them in any
  workflow, because a runner image gives them to the release for free.

  `check-installers.mjs` gained `--only` for the same run: it asks tauri for
  `--bundles nsis`, and without it the checker failed the build for not
  producing an `.msi` nobody had asked for. It narrows and never widens — a name
  that is not a format of that platform is refused rather than ignored — and a
  narrowed run says out loud what it did not check, so it can never read as a
  full one.

  `aarch64-pc-windows-msvc` cannot be built here, and that is upstream rather
  than unconfigured: `ring` compiles its ARM64 Windows assembly with plain
  `clang` because MSVC's assembler cannot read that syntax, and plain `clang`
  cannot compile Microsoft's ARM64 headers — `winnt.h` stops at
  `#define __MACHINEARM_ARM64 __MACHINE`, which `clang-cl` handles without
  complaint. Both requirements live in one `cc::Build`, and there is no flag
  that satisfies both: measured on this machine, `clang` accepts only
  `-isystem` and `clang-cl` only `-imsvc`/`/imsvc`. That mode now says so in a
  second instead of failing after ten minutes of compiling.

- **`tools/sidecars.mjs` honours `CARGO_TARGET_DIR`, and takes a `--runner`.**
  The runner is `tauri build`'s own flag by the same name and for the same
  reason: cross-building the two sidecars to `*-pc-windows-msvc` needs
  `cargo-xwin`, and without it they fail before the bundler is ever reached.

  It also composed the path `src-tauri/target/<triple>/<profile>` by hand and
  ignored `CARGO_TARGET_DIR`, so inside the container — which
  sets the variable to `target-linux/` on purpose, to keep a Linux object file
  out of the macOS target directory — cargo succeeded and the copy failed on a
  path nobody had chosen, with "cargo reported success but ... does not exist".
  That is the one message this file was written to prevent.

- **The AppImage bundler is given the binary it copies off the runner (§3 #22).**
  The first rehearsal put `StackVo_0.1.0_arm64.deb` and
  `StackVo-0.1.0-1.aarch64.rpm` on disk from `ubuntu-24.04-arm` — the two
  packages this item had been waiting three rounds to see, named for the right
  architecture, built natively rather than cross-compiled — and then failed on
  the third format with `xdg-open binary not found /usr/bin/xdg-open`.

  `tauri-bundler` copies `/usr/bin/xdg-open` into the AppDir when the
  application can open a link, and this one can, through `tauri-plugin-opener`.
  It copies it from the **build machine**. `ubuntu-latest` ships `xdg-utils`;
  `ubuntu-24.04-arm` does not. So the same commit produced an AppImage on one
  Linux runner and not the other, and the question was never "does ARM compile"
  but "are these two images the same machine". `xdg-utils` is on the apt line,
  and a test holds it there — it is not a `-dev` header and nothing else needs
  it, so it reads like something that wandered in.

- **The check on what was bundled no longer waits for the bundler to succeed.**
  It was gated on `steps.tauri.outcome == 'success'`, and the run above is what
  disproved that: two correct ARM packages were on disk and the step skipped
  itself and said nothing about them. A check that speaks only when nothing went
  wrong is a check that is silent exactly when it matters.

- **A rehearsal can now answer the question it was added for (§3 #22).** The two
  ARM rows — `aarch64-unknown-linux-gnu` on `ubuntu-24.04-arm`,
  `aarch64-pc-windows-msvc` on `windows-11-arm` — had one open half: whether the
  bundler produces a package there. The line said the remaining work was another
  run, and this time it would reach the packaging step. It would not have, for
  two reasons, and both were measurable without a runner.

  **The packaging step sat behind the suite.** The first real run died at
  `cargo test` on all six targets, so eighteen minutes on `windows-11-arm`
  taught #22 nothing about the only thing it asks. The two questions are
  independent — does the suite pass there, can a package be produced there — and
  a rehearsal publishes nothing, so a red suite has no release to protect. The
  suite step is `continue-on-error` in a rehearsal and the bundle steps run
  anyway; the last step in the job reads that step's outcome and fails on it, so
  the run still ends red. `continue-on-error` alone would finish green, which is
  a worse thing to own than an unanswered question.

  **And the rehearsal would have died at the end of packaging, having packaged.**
  `bundle.createUpdaterArtifacts` is true and `plugins.updater.pubkey` is set,
  so `tauri build` signs the updater-enabled bundles unconditionally, from
  `TAURI_SIGNING_PRIVATE_KEY`. A repository that has not decided where a private
  key lives does not get tauri's clean *"a public key has been found, but no
  private key"* — Actions sets an absent secret to the **empty string**, which
  is set, so the guard passes and the run dies decoding a zero-length key. That
  happens in `sign_updaters`, which runs after `bundle_project`: every installer
  is already on disk. And the artifact upload was `if: ${{ inputs.rehearsal }}`,
  which carries an implied `success()` — so the rehearsal would have built
  exactly what #22 asks about and kept none of it. It now builds with
  `--no-sign`, and the upload runs on `always()`.

- **`npm run installers:check` gives the answer a verdict instead of a zip
  file.** "The bundle directory exists" is not what #22 asks. `bundle.targets`
  is `"all"`, so Linux owes a `.deb`, an `.rpm` **and** an `.AppImage` from three
  separate bundlers — one of which downloads `linuxdeploy-aarch64.AppImage` and
  executes it, the step with no equivalent on the x86 rows — and Windows owes a
  `.msi` and an NSIS `-setup.exe`. The checker demands each of them, and then
  demands the harder thing: that what came out is named for **this** target's
  architecture. Every bundler writes the architecture into the file name and
  each writes it in its own vocabulary (`arm64` to dpkg, `aarch64` to rpm and
  AppImage, `arm64` to WiX and NSIS, `aarch64` to the dmg); the table is read
  out of `tauri-bundler` rather than guessed, because collapsing it into one
  word would fail every real ARM release. A green ARM job holding an x86
  installer is the failure an artifact listing cannot show, and it is now the
  one this catches. Judgement half tested without a bundler in
  `tests/installer-formats.spec.js` (15 tests); that the workflow runs it, in
  `src-tauri/tests/release_rehearsal.rs` (8 tests).

- **The matrix target reaches the toolchain that actually builds.**
  `dtolnay/rust-toolchain@stable` installs the target into `stable`, and nothing
  in the release job builds with stable: `src-tauri/rust-toolchain.toml` pins
  1.96.1 and rustup resolves that pin from the working directory.
  `stable-<host>` and `1.96.1-<host>` are separate installations even on the day
  they are the same compiler. Free on the four rows where the target is the
  host; the whole build on `x86_64-apple-darwin`, which cross-compiles from an
  arm64 runner. The sidecar steps moved into `src-tauri` for the same reason —
  started at the repository root they built the **shipped** sidecars with
  whatever `stable` meant that morning, a different compiler from the one that
  builds the application they are bundled into.

- **A workspace's service vocabulary is read from the catalogue, and seventy-two
  keys left with the answer (ADR 0037).** ADR 0016 moved services into packages
  and made `env.schema.json`'s `services` a vocabulary. A second answer stayed
  behind: whether a key was *present* in `.env`. The only place the two met was
  small, which is exactly why nobody had looked — `mail.rs`'s `detect` decided
  which catcher a workspace knows about by asking whether
  `SERVICE_MAILPIT_ENABLE` was declared, and on an untouched workspace the only
  thing declaring it was `config::LEGACY_SERVICES`, a table named for the
  migration.

  `detect` now asks `contracts::env_schema().knows_service(...)`. `.env` answers
  what is *enabled*; it no longer answers what *exists*.

  The deletion is a consequence rather than a tidy-up: with the vocabulary
  coming from the catalogue, the catalogue's shadow in `.env` has no reader.
  Every `SERVICE_*_ENABLE`, `_VERSION` and `_VERSIONS` — **72 keys** — is gone.
  `LEGACY_SERVICES` 150 → 78, `EMBEDDED` 186 → 114. Nothing was lost with them:
  all twenty-five enables defaulted to `"false"` and `Env::bool` reads a missing
  key as false, so as values they were the identity, and `handover::plan`
  already asks the catalogue for a version the `.env` does not state.

  One test broke, for a reason unrelated to its subject:
  `an_unlisted_version_is_folded_into_the_options` borrowed its fixture from
  `SERVICE_MONGO_VERSIONS`, so a test about what `service_versions` does with a
  list never said what the list was. It says so now.

  What this does **not** close, stated correctly on the second attempt. The
  first answer split the remaining 78 into "live" and "migration" and that was
  wrong: all 78 are the same old `.env` family, and the package manifest took
  over every one of their jobs — `url` for `_URL`, `ports` for `_HOST_PORT`,
  `settings` for the passwords. The schema says it outright: a setting's `key`
  is "the bare key, without the `SERVICE_<ID>_` prefix the old .env family
  carried". Every reader is a correctly gated pre-migration branch.

  What actually keeps them alive is one line in `db.rs`. `Value::or_env`
  resolves `stored → .env → manifest default`, with `.env` in the middle on
  purpose — a handover leaves the real password there while the manifest
  declares a placeholder, so preferring the manifest would hand a dump the
  wrong password with no readable error. But the `.env` it consults is a merged
  `Env`, and `Env::parse` lays these defaults **under** the file. On a
  workspace installed from packages, with no `.env` at all,
  `SERVICE_MYSQL_ROOT_PASSWORD` answers `"root"` out of the binary and beats
  what the package declares — the reverse of ADR 0016.

  So the fix was one change to `Env`, and it is in: `Env::stated` answers only
  what the file wrote, and `db.rs`'s middle slot reads that instead of `get`. A
  workspace installed from packages now takes its password from the package.
  `parse` already kept this distinction for the alias chain — the comment there
  records what learning it cost, a checkout asking for Apache quietly served
  nginx — so this is that distinction kept rather than discarded at the end.

  With it, `config::LEGACY_SERVICES` is what its name says for the first time:
  every reader is the migration or a pre-migration branch, and the only thing
  left is deleting it at the cutoff.

- **`db.rs`'s three-source resolution order had no test.** It is described
  carefully and relied on by every dump, restore and connection string, and
  nothing asserted it — the same state `mail::detect` was in.
  `stored_beats_env_beats_the_packages_default` pins all three, including the
  case that is not a preference but a defect: whatever reaches the middle slot
  outranks the package, and today `config::EMBEDDED` can reach it. The
  assertion carries the sentence saying what a change to it would mean.

- **`mail::detect` had no test, and it is the function the rest of that module
  hangs off.** `status` returns an empty panel without it and `open` refuses. It
  has one now — written because the deletion rehearsal went looking for what was
  holding the constant up, and pinning today's answers before the source of
  those answers moved. The assertions did not change across ADR 0037; only where
  the answer comes from did. Without it that move would have been
  indistinguishable from a regression, because the failure has no red of its
  own: the panel would just stop naming a catcher.

- **`npm run legacy:rehearse`** performs the deletion §3 #36 is waiting for,
  runs the suite, restores the tree, and compares the damage against a written
  list of sites and the sentence saying why each is on it. It reports a row that
  stops failing as loudly as one that starts — which is how two rows were caught
  going stale on the commit that made them stale.

### Changed

- **Three appearance defaults moved: corner radius 12 -> 24, UI scale 16 -> 15,
  density compact -> spacious.** The defaults are the whole of what a first
  launch looks like, and nobody opens the appearance pane before they have seen
  one. Nothing is migrated and nothing is overwritten: `DEFAULT_APPEARANCE` is
  what a preference falls back to when it was never set, so anybody who has
  already chosen keeps their choice and this changes only the untouched install.
  All 1,437 front-end specs pass unchanged, which is the fact worth recording —
  no test had pinned the old three.

- **The coverage floors were four points under the measurement and are now
  three; the measurement was four points stale and is now current.** The
  recorded pair had stood at 2026-08-07 while the tree gained four points, so
  the Rust floor of 60 sat eight points under a real 68.05 — a seven-point
  regression would have passed in silence. Rust is now 65, and the front end's
  85/72 are 90/78 against a measured 92.20/80.99.

  The distance is arithmetic rather than habit: the measurement, less one point
  of platform (where this repository's history puts macOS against Ubuntu) and
  two for a module written before its tests, which is its uncovered lines over
  the tree's 59.5k. And the gate now says when the recorded measurement has
  fallen behind the tree — a gain, so it is said rather than failed on, but an
  unsaid one is exactly how the pair went stale.

- **The accessibility statement moved to the root, and the transcript moved out
  of the source tree.** `docs/` in this repository is not a documentation dump:
  `tauri.conf.json` bundles `docs/help` into the application as a resource, so
  what lives there is shipped product content, and beside it there was only a
  worklog. A conformance statement is neither. It is a public claim about the
  product in the same register as `PRIVACY.md` and `SECURITY.md`, which is a
  class this repository already has a home for — so `docs/accessibility.md` is
  now `ACCESSIBILITY.md`, at the root with the rest of them, one click from the
  landing page instead of two.

  `docs/accessibility-transcript.md` had a second problem: it is generated,
  gitignored, and rewritten by every `npm run test:js` — a build product living
  inside a directory otherwise fully tracked and partly shipped. It writes to
  `reports/` now, which is ignored whole. Not `test-results/`, which is
  Playwright's output directory and is emptied at the start of every browser
  run; not `coverage/`, which is vitest's. The spec creates the directory
  itself, so a fresh clone's first run does not fail on a missing path.

  Nothing about the statement's content changed, and nothing stopped being
  checked: `accessibility-claims.spec.js` still reads it and holds it against
  the routes and the axe suite, and `native_window_claims.rs` still fails if §4
  stops saying the native audit is owed.

### Fixed

- **The contract said which files read each setting, and 39 of its 45 paths
  named files that do not exist.** All thirty-nine were under `core/` — the Bash
  and Node implementation the Rust rewrite replaced. Six pointed at the live
  tree. A contract field naming a deleted directory is worse than no field: it
  reads as evidence, and somebody deciding whether a setting is safe to change
  reads it.

  The tool that maintains the field pointed at `core/` too, so it exited 2 on
  every run and told nobody — the thing written to stop this measurement rotting
  died in exactly the way it was written to prevent. It reads this repository
  now, and `tests/env-consumers.spec.js` fails when the lists go stale: the
  paths must exist, the lists must match a fresh scan, and the `active`/`dead`
  labels must match what that scan finds.

  Two rules in it are worth stating. **Declaration is not consumption** — every
  key is written in `config.rs`'s settings tables, so counting that file would
  report all 72 as active and `dead` would become unreachable. And **comments
  are stripped**, which is load-bearing rather than tidy: this repository writes
  down that a key is dead in a sentence naming the key, and an unstripped scan
  read `DOCKER_REMOVE_ORPHANS` and `HOST_PORT_ADMINER` as live off the sentence
  saying they were not.

  Two more claims in the schema's own header were fossils of the same removal:
  it named `.env.example` as its source — there is no such file — and claimed
  159 keys against the 72 it describes. Both are corrected, and the count is
  checked rather than asserted.

- **A `.env` line with nothing after it gave every project a domain ending in a
  bare dot.** `DEFAULT_TLD_SUFFIX` had two derivations that disagreed:
  `certs::suffix` trimmed it, lower-cased it and treated an empty value as
  absent, while ten call sites did a bare `unwrap_or("stackvo.loc")`. `Env::parse`
  lets a file's empty value win over the embedded default, so
  `DEFAULT_TLD_SUFFIX=` produced `shop.` everywhere while the certificate was
  still issued for `stackvo.loc`. A suffix written `Shop.LOC` split the same way.

  Three values had this shape — the suffix, the Docker network, and the default
  web server — so each is now one named constant and one derivation:
  `config::DEFAULT_TLD_SUFFIX`, `DEFAULT_NETWORK`, `DEFAULT_SERVER`, with
  `Env::tld_suffix`, `Env::docker_network` and `Env::default_server` beside them.
  The network is deliberately not case-folded: Docker network names are
  case-sensitive, so lowering one would name a network that is not there.

- **One setting, three answers: `SUPPORTED_SERVERS_DEFAULT` moved the wizard and
  nothing else.** It was read in exactly one place in production — the value the
  new-project wizard shows preselected — while five render sites said
  `unwrap_or("nginx")`. So with `=caddy`, a project opened from the wizard ran
  Caddy, an adopted project ran nginx, and a manifest with its `server` line
  deleted ran nginx. The setting reaches the render path now, and
  `Manifest::server_or` is the one place a project that did not choose gets an
  answer.

- **A DDEV project configured for Apache was imported as something else.**
  `Detected.server` was the string `"nginx"` at all four places detection set
  it, which read as a finding and was a default. `imports.rs` overwrites it with
  real evidence — DDEV's `webserver_type` — but because the literal made the two
  indistinguishable, `detected_spec` resolved the workspace setting *first* and
  the evidence it had recorded was ignored. It is an `Option` now, and the order
  is the one every other field in that function already used: what the folder
  declares wins, and the setting answers for a folder that declares nothing.

- **Forty-seven English sentences were printed into a Turkish window.** Five
  Rust catalogues carry prose to the screen — what each quick command does, what
  each identity provider accepts as a callback URL, why each required tool is
  required, what this repository's own two binaries are for, and what has to be
  changed before a shipped provider recipe will run — and every one of them was
  an English literal rendered raw. A Turkish user got a translated interface
  with English inside it.

  `hints.rs` had already written down why this class is the worst one to leave
  alone: *"It is the sentence that tells someone what to do."* "Run pending
  migrations." sits under `php artisan migrate`, above a button that runs it in
  somebody's container.

  Keyed by the id each catalogue already carries — `Spec.id`, `Provider.id`,
  `Tool.id`, `Recipe.name` — so there is no second name to typo and no mapping
  to keep level. The English stays in Rust and is still what the CLI prints,
  what an MCP client reads and what the log records; the window shows the
  translation and falls back to the English. `hint_translations.rs` holds the
  two sides equal: a row nobody translated fails, a translation nothing offers
  any more fails, and English that has drifted from Rust fails.

  A recipe's edit list is the one exception and carries its own key, because it
  has no ids and **a position is not a name**: both database recipes need the
  same instruction, so keying by position would have meant two translations of
  one sentence. It is `{ key, english }` now, which is the shape `hints.rs`
  settled on.

  The roadmap counted 39 across three catalogues. Measured, those three carry 37
  — `oauth.rs` has 7 notes, not 9 — and two catalogues it had not looked at
  carry eleven more.

  One passage went the other way. A sidecar's description comes out of the
  project's own `stackvo.json`, so nothing here knows what language it is in; it
  is marked `lang=""` — undetermined — beside the container output and captured
  dumps that already were.

- **The charts ignored the theme, and one of them ignored a colour-blindness
  setting.** Four colours were written into the Indicator pane and its stats
  composable as hex, and every one was a copy of a theme value: `#1976D2` for
  `primary` three times over, `#2A313C` for one theme's `surface-variant`,
  `#4CAF50` for `success`. Move the accent to purple and the application turned
  purple with three blue pie charts and a blue sparkline in it; switch to the
  light theme and the second slice of every pie was dark charcoal on a white
  card — a colour with no relationship to anything on screen.

  The sparkline is the one that mattered most. `success` is one of the three
  colours the status palette rewrites for red-green deficiency, so a hardcoded
  green ignored that choice on a chart made of nothing but colour.

  The pie colours left the composable rather than being replaced there — not
  because a composable must not know colours, but because the colour that
  belongs there is the *current theme's*, and the theme is only readable from
  inside a component. The pane paints them now, by position: every pie here is
  measured-against-remainder, so the first slice is `primary` and the rest is
  `surface-variant`, which is the ground in every theme this app ships.

- **The CPU activity grid was five fixed greens, and two of them were near-black
  squares on the light theme.** `.heat-cell.l0` through `.l4` were tied to
  nothing — not the theme, not the accent, not the status palette — so somebody
  who had chosen Okabe-Ito for colour blindness had it everywhere in the
  application except on the one card that is nothing but a field of colour.

  It is the theme's `success` now, with intensity carried by opacity rather than
  by five hand-picked shades. That is what makes it work in both themes: the
  cell sits over the card, so the ramp runs from the card's own surface up to
  the full colour, whatever that surface is. Five hues cannot do that — they are
  picked against one background.

- **A frame around the terminal, in a colour nothing agreed on.** `#12121a` was
  written twice in one file: once as xterm's background in JavaScript, once as
  the host element's in CSS, eight pixels of padding apart. Only the first was
  conditional, so with "keep consoles dark" switched off xterm fell back to its
  own default and the CSS frame stayed `#12121a` — the drift the two literals
  invited was already there. One constant now, read by both halves, and the
  frame is transparent when the setting is off.

- **Every icon in the terminal, editor and browser pickers was a blank square,
  and one of them had been blank since it was written.** The icon subsetter
  added last release strips the rules nothing names, and it read `src/` only —
  but `apps.rs` is a catalogue, and eighteen of its icons are named nowhere
  else: `mdi-apple`, `mdi-firefox`, `mdi-powershell` and fifteen more appear in
  no `.vue` file at all. All eighteen were being stripped. The scan reads
  `src-tauri/src` now, which costs 2.4 KB of the eager set, measured.

  Its first run then found a nineteenth of a different kind. `mdi-vim`, the icon
  on the Neovim and Vim rows, **is not an icon** — Material Design Icons has
  never had one — so those two rows have been drawing an empty square since the
  day they were added. It could not have been caught before, because the file it
  was written in was not being read.

  Rust is scanned narrowly, and that is measured too: in `.rs` an icon name is
  always a string literal, while the prose around it is thick with names that
  are not icons. The first wide run collected `mdi-vim` written down as the bug
  it was, and `mdi-icons.mjs` naming the file doing the scanning.

### Added

- **PhpStorm opens the same container, not a second one (§2 R-2, decision
  0036).** The question this closes had three answers and all three cost
  something: **sshd** wants a host port that ADR 0023 does not give side
  containers — and this time the thing listening is a shell; a **join link**
  asks who puts a gigabyte of JetBrains backend inside the project image; and
  **Dev Containers** builds a second container beside the one already running.

  What closed it was not a preference but a measurement, and it was taken from
  the IDE rather than from a page. PhpStorm 2026.2 bundles
  `clouds-docker-gateway`, and the devcontainer schema inside that plugin
  carries `dockerComposeFile`, `service`, `runServices`, `workspaceFolder`,
  `shutdownAction` and `overrideCommand`. So "Dev Containers builds a second
  container" is true only of the **image/Dockerfile** flavour. In the compose
  flavour, what the dev container *is* is decided by whoever writes the file.

  StackVo writes it: the compose files are this workspace's own — read out of
  `runner::compose_base_args`, so overlays included and no second list to go
  stale — and the service is the project's. The container it opens is the one
  already running. No sshd, no host port (ADR 0023 untouched), no foreign
  binary in the project image: JetBrains' own plugin downloads its backend,
  which is its job.

  Three fields each turn off a default that would have been wrong here, and
  each of them fails silently rather than loudly: `shutdownAction: "none"` (the
  compose default takes the workspace down when the IDE closes),
  `overrideCommand: false` (the default replaces the service's command, which
  in a PHP project is the thing serving the site) and `runServices: [service]`
  (unspecified means every service in every file listed).

  The file goes under `generated/devcontainer/<project>/`, never into the
  repository — it names absolute paths under this user's home, so a committed
  copy resolves to nothing on anybody else's machine. `devcontainer.rs` still
  writes the one that *is* meant to be committed, and it answers the opposite
  question: how a machine **without** StackVo runs this project.

  Two things are said on screen rather than discovered: attaching recreates the
  project's container (the plugin's own setting says the main service always
  is), and an Alpine image has no JetBrains backend at all — VS Code publishes
  a musl server and JetBrains does not, so the one fact that was a footnote in
  the VS Code half is a wall in this one.

  And one measured property carries the whole design: the generated compose
  files interpolate nothing (`${` appears zero times), which is what makes
  handing them to a tool that runs compose without this app's `--env-file`
  honest. `editor_claims.rs` now fails if that ever stops being true.

  **The claim was then run rather than argued.** The command the plugin itself
  would issue was put to the live daemon with `--dry-run`, against the six
  files the written file names: `docker compose -f … up -d --no-recreate
  parser.ajans` → `Container stackvo-parser.ajans Running`. So the container it
  opens is the one already up, and it resolves with no `--env-file`. The same
  run closed a question nobody had asked yet: every generated project service
  sits behind a compose **profile**, and a profile activates only when
  something names it or names its service — which is the second job
  `runServices` is doing, and why an empty list there would send the IDE
  looking for a service compose does not consider to exist.

- **The editor itself, inside the container (§2 R-1, and §2 R-3 with it).**
  `ide.rs` wired an IDE on the *host* to a debugger in the container. This is
  the other half: VS Code running **in** the image — language server,
  extensions, terminal, `composer` and `artisan` all in there, and no PHP on
  the machine at all.

  The whole feature is an address. VS Code has no attach-by-name command line;
  it opens a running container through a remote authority, and that authority
  is derivable from three facts this tree already had — the container's name
  (`engine::container_name`), the directory the source sits at
  (`editor::workdir_of`), and the bind mount itself. `editor::attach_authority`
  builds `attached-container+<hex>`, where `<hex>` is
  `{"containerName":"/stackvo-shop"}` in hex, and two forms come out of it
  because two are needed: `folder_uri` for `code --folder-uri`, and
  `handler_url` for the OS. A machine can easily have VS Code and not its
  `code` launcher — the application registers its own URL handler, so that is
  the form that opens anything there.

  **The hex is held against a hand-written constant**, not against a second
  run of the same arithmetic. A test that hexed the string again would agree
  with any encoding both halves shared, including one VS Code cannot read. The
  spelling was then checked against the Dev Containers extension's own
  construction, which builds the same string from Docker's `Name` — the one
  that carries a leading slash, which is why `shop` and `/shop` here produce
  one address and not two.

  Nothing is stored. The address is re-derived on every read, so a recreated
  container or a renamed project cannot leave a stale one behind, and
  `editor_claims.rs` now fails if anything outside Rust assembles it.

  **It was then opened, and that is the half a test cannot make.** Against a
  running project on a developer's machine: `/root/.vscode-server` appeared in
  the container — 956 MB, `bin/<commit>`, its own extension host — eleven
  server processes came up inside it, and the git extension's log there says
  `Opened repository (path): /var/www/html`. The address reached the right
  folder in the right container, from a string this app derived and never
  wrote down.

  The screen is `EditorPane.vue`, on the Container tab beside the tunnel and
  the LAN name — the same kind of thing as those, an address that reaches this
  container, pointing inward instead of out. Both refusals are named
  separately, because "cannot open an editor" over a stopped container and
  over a container holding a *copy* of the source are two sentences with two
  different answers. And the address is on screen even when the button cannot
  be pressed: it is a string that works on a machine this one is not, so
  hiding it would turn a missing launcher into a missing feature.

- **A refusal that was wrong for every container there has ever been.**
  `editor.rs`'s judgement — is the workdir really a bind mount — compares a
  mount's `kind` to `bind`. `engine::inspect` produced that field with
  `format!("{t:?}")`, and `MountPointType` is a **String** in bollard, not an
  enum: `Debug` on a string puts the quotes in. Every container reported
  `"\"bind\""`, so the comparison was false everywhere, and a PHP project
  with its source mounted was refused with a sentence about a snapshot.

  A refusal that is always wrong is indistinguishable from a refusal that is
  working, which is why thirteen unit tests did not see it: both sides of the
  comparison are written by hand in a test. `examples/editor_attach_probe.rs`
  asked a live daemon instead, and the mount table answered in quotes.
  `engine::mount_kind` is the fix, two tests hold the word, and
  `editor_claims.rs` fails if the mapping goes back to `Debug`. Against the
  same running container afterwards: `source live true`, `ATTACHABLE true`.

- **§2 C is closed and its row is gone.** Third-party package distribution had
  three things left, and the last round called all three process. Measuring
  them said otherwise, and the last of them was a hole this repository had just
  opened for itself.

  `maintainer` is a free string in the schema, and the app had just started
  **showing it on the market card**. Those two facts side by side are an
  impersonation hole with nothing in between: a pull request could name
  `stackvo` as the publisher of anything, and every user would read a name
  nobody had checked. Showing the field did not create the hole — the string
  was always free — it created the cost, so it is closed in the same round.

  `publishers.json` in the packages repository is the list of who may be named,
  and `validate.mjs` refuses any maintainer that is not in it, or none at all.
  The check lives there rather than in the client, and that is the whole trust
  argument: the index is signed with the registry key **after** those gates
  run, so the signature carries *the registry vouches for this* and the
  maintainer strings are inside what it covers. A client cannot verify an
  identity claim on its own; it can only verify that the registry made it.
  Adding a publisher is a separate pull request from adding a package, because
  bundled, the decision that matters arrives as one line in a diff about YAML.

  `CONTRIBUTING.md` is the moderation process, and most of it is not judgement:
  the gates already decide, the same way every time, most of what a reviewer
  would otherwise be trusted to notice. The three things a gate cannot hold are
  named — vouching for a publisher, whether a service belongs in the catalogue,
  and whether an image is one to hand to a stranger's daemon. A review whose
  rules are not written down is a review that changes with whoever does it.

  Takedown was already there (§6, decision 0014). So the row is deleted per §8,
  its decisions live in §6 (0021, 0033, 0034, 0035) and its record here, and
  the seven comments that pointed at it now point at what actually owns the
  behaviour. Two of those had been dangling already: `C-1` names a row that has
  not existed for some time.

- **The catalogue can say whose a package is (§2 C).** `maintainer` has been in
  `package.schema.json` all along and every one of the 31 published packages
  fills it in — and the index dropped every one of them. `grep` found no screen
  reading it. So publisher identity existed as data and reached nobody, which
  is indistinguishable from a field nobody ever filled in.

  It is exactly what happened to `keywords`, with a sharper edge. A catalogue
  that means to carry third-party packages is asking somebody to run somebody
  else's compose fragment, and who wrote it is the one fact they weigh before
  saying yes. Leaving it out was not a missing nicety; it was the catalogue
  being unable to answer the question third-party distribution is *about*.

  The whole chain, because a field is only carried if every link carries it:
  `registry.schema.json` → `build-registry.mjs` → `market::PackageRow` →
  `MarketPackage` → the ⓘ facts on the service row, in both locales. It sits
  with the other facts rather than on a badge of its own — a row with two
  information buttons invites the question of which one holds the information.

  Optional at every link. An index built before the field was carried is a
  perfectly good index, and inventing a publisher for one would be worse than
  the gap: a name on a card reads as something somebody checked.

  This is also the correction to a claim made two entries above — that what
  remained for §2 C was entirely process. Half of it was code, and it was
  measurable in one `grep`.

- **The hole exactly one refresh wide, closed — and the rule that makes it safe,
  held by a build.** `Trust::WhenSigned` learns that a source signs from
  `market/source.json`, and a machine that has never refreshed has no
  `source.json`. So every refresh was protected except the **first** — the one
  that decides what a new machine installs, and the one somebody on the path
  would target by serving a 404 for the signature.

  `market::known_to_sign` compiles in the fact that the official catalogue
  signs. One address, compared against `resolve_location`'s output so every
  spelling of it — web page, trailing slash, clone URL, `www.` — is one
  equality and nothing else is matched by a prefix. It could only be written
  once it was true: before the index was signed, a build claiming this would
  have refused the catalogue outright.

  What that costs is that a regeneration published without a fresh signature is
  now refused on a first refresh as well as a hundredth. So the rule is held
  where it can be held rather than remembered: the packages repository's CI
  verifies the signature over the index it ships (`tools/verify-signature.mjs`,
  zero dependencies, no network, in the `gates` job). Ed25519 and BLAKE2b are
  in Node; `minisign` is not on a runner, and an apt install in the one job
  whose purpose is to be trustworthy is a poor trade.

- **The chain is closed, and it was measured from outside.** The last link was
  never code: a pinned key signs nothing, and `registry.json` had to be signed
  with the private half by hand, on the machine that holds it. That happened.
  `raw.githubusercontent.com/stackvo/stackvo-service-packages/HEAD/registry.json`
  (44638 bytes) and its `registry.json.minisig` (400 bytes) were fetched and
  verified against `signing::PINNED`'s `256219FF1F9A0F1B`. Pinned key → index →
  manifest → file, end to end, against the address the app suggests.

  What that leaves for §2 C is not engineering. A moderation process and a
  publisher identity registry are two faces of one thing — a third party being
  able to *submit* — and what exists today is verifiable **first-party**
  distribution. The gap is not a missing mechanism; it is that nobody has
  written down who accepts a submitted package, and on what grounds.

  It also creates a standing operational rule, and it is load-bearing: the index
  is generated, so every regeneration needs signing again. A stale signature is
  a refusal rather than a downgrade — correct, and the reason the rule cannot be
  left to memory. The gate that would make forgetting impossible belongs in the
  packages repository's CI and does not exist yet.

- **The ceremony reads the file you named, from the directory you are standing
  in.** `tools/keys.sh sign` and `verify` handed their path straight on to two
  helpers that deliberately run somewhere else — `tauri()` in the repository
  root, `verifier()` in `src-tauri` — so a relative path was resolved against a
  directory the person had never seen.

  Relative is not an edge case here: the whole ceremony is `cd` into the
  packages repository and name the index in front of you. `keys.sh verify
  registry.json` from there answered `reading registry.json: No such file or
  directory`, which is a true sentence about the wrong directory, at the one
  moment somebody is closing the chain for the first time. Signing had the same
  fault and predates the verifier.

  The regression test runs the script with a `cargo` on `PATH` that only prints
  its arguments, and asserts which path came out. The real tool would mean a
  nested cargo inside `cargo test`, which deadlocks on the build lock; what is
  under test is the path, and a stub settles it.

- **The signature is checked because the publisher signed, not because someone
  found a setting (§2 C, ADR 0034).** The chain had three links, a pinned key
  and fifteen tests, and on a stock machine **none of it ran**: `market_refresh`
  passed `Trust::Unsigned` unless an administrator had written
  `requireSignature`. Publishing a signed index would have changed nothing any
  user could see.

  `Unsigned`/`Signed` describes a flag day — everything unsigned until the
  publisher signs, everything mandatory after — and in between is the moment
  every machine that has not been told breaks. That is exactly why the default
  stayed where it was, and the reason was sound.

  `Trust::WhenSigned` is the third answer and is now the default. A signature
  that is present is checked, and a check that fails is a refusal — never a
  fall-through to "unsigned, then", because a signature that verifies against
  nothing is the loudest evidence a refresh can produce. A signature that is
  absent is accepted only from a source that has never given one; the memory
  was already on disk (`SourceRef.verified_by`). Without that second half the
  mode is defeated by deleting a file: anyone who can serve a tampered index
  can serve a 404 for its signature. It is the same shape as the rule that an
  index may not go backwards, for the same reason.

  So there is no flag day. The day the index is signed, every machine starts
  verifying on its next refresh with nobody editing anything, and no machine
  breaks the day before. ADR 0009 is intact — `requireSignature` still only
  tightens, refusing a missing signature too.

  The two refusals are worded apart because they are different events: "no
  signature here" and "this source signed for you before and is serving none
  now". The second carries its own hint, since "no such file" reads the one
  thing an attacker on the path would arrange as a filing error.

  Four integration tests and two unit tests, each shown to bite by breaking
  what it guards: the strip attack, a bad signature read as no signature, and
  the default reverted.

- **The ceremony now proves its own signature before publishing it (§2 C).**
  `tools/keys.sh sign` produced a `registry.json.minisig` and said "publish it",
  and between that sentence and a user's machine nothing ever asked the question
  that decides it: is this a signature a shipped build accepts?

  The failure that leaves is quiet and total. The content key and the updater
  key live in one directory and their file names differ by one word; an index
  signed with the wrong one — or with a key `PINNED` has since rotated away
  from — signs without complaint, uploads cleanly, and is refused by every
  installed copy of the app at once, with the publishing side holding no
  evidence at all.

  So the signature takes the published name only after the app has accepted it,
  which is the shape `market::install` already uses for a package: verified
  whole, then moved. A rejected signature stays `.sig` and is named; whatever
  `.minisig` was already beside it is untouched, so nothing that was working is
  replaced by something that is not. The judge is the app itself —
  `examples/verify_index.rs` links `signing::Keys::pinned()` and `verify`, the
  set and the function a release actually uses. A `minisign -V` here would have
  been a second opinion, and the round the two disagree is the round this prints
  a tick for a file every machine refuses. An organisation signing its own
  mirror names its key with `--key`, because its question is not whether *this*
  build trusts the index but whether the machines it configured will.

  `keys.sh check` gained the same question for the content key that the updater
  key has been asked since the script existed: is the private half on this
  machine the pair of what the build pins? Getting the updater pair wrong is
  caught by a release that will not sign; getting the content pair wrong was
  caught by nobody. Measured on the ceremony machine: it is the private half of
  `256219FF1F9A0F1B`, so both ends of the chain now look at one key.

  Four gates, each shown to bite by breaking the thing it guards.
  `key_ceremony.rs` runs the script against a fixture key directory rather than
  grepping it for a sentence — an inverted comparison passes a grep — and runs
  `verify_index` for real, because `sign` decides whether to publish from one
  exit status. `signed_refresh.rs` gained the one that was missing: `refresh`
  checks the signature **before** it parses, four lines apart, and swapping them
  broke no test. Bytes that are neither an index nor signed by anything now have
  to be refused for the *key*, which only one ordering can produce.

  And the stale claim was closed as a class, not as two edits. Filling `PINNED`
  turned honest prose about an empty list into confident, specific, wrong
  claims, and two files were still telling readers the chain's first link was
  open. No `.rs` file may now describe the other state of `PINNED`, in either
  direction — so a future retirement fails the same way round.

- **A container can now say whether it could carry an editor (§3 R-3).** The
  half of "run the editor inside the container" that has to be settled before
  the button is worth building, and it is not one question. `editor.rs` answers
  four.

  The one it exists for is the source. A PHP project bind-mounts
  `/var/www/html`, so an editor in there edits the repository. A `runtime: node`
  project does not — its Dockerfile is `COPY . .`, and the container holds a
  **snapshot** taken when the image was built. An editor opened against that
  works perfectly, saves without complaint, and nothing written in it ever
  reaches the host; the session is lost on the next rebuild, with nothing
  anywhere saying so. That is a refusal here, not a warning.

  It is read from the container's own mount table rather than from the
  manifest, because the two come apart: turning the dev server on writes an
  overlay, and the overlay does nothing until the container is recreated. A
  bind mount at the wrong path does not count (`/var/log` is on every PHP
  container), a *named volume* at the workdir does not count either (that is
  `perf.rs`, which is precisely the case where the host copy is no longer what
  the container reads), and a stopped container reports only that it is
  stopped — reading a refusal out of an empty mount list is inventing one.

  Persistence is a `docker-compose.editor.yml` overlay: one named volume per
  project at `/root/.vscode-server`, so a rebuild does not throw away a hundred
  megabytes that then download again. The mount goes in whether or not anybody
  attaches, which is the argument `debugbridge` already makes for its three — a
  volume is the part that needs the container recreated, so adding it on first
  use would mean the button restarts the application it was asked to open.

  libc is read from the image rather than from a table of runtimes, and
  `node:X-alpine` now has one source (`generator::node_base_image`) instead of
  two that would agree until node's tags changed. Alpine is recorded, not
  refused: VS Code publishes a musl server, JetBrains does not, and that is §2
  R-2's problem to state. The lang runtimes are excluded from the volume
  entirely — they build with `COPY . .` and have no equivalent of the dev
  server's overlay, so a volume for an editor that must be refused anyway is a
  hundred megabytes of nothing.

  `editor_claims.rs` holds the two assumptions this makes about files it does
  not own: that no generated Dockerfile carries a `USER` directive (a perfectly
  ordinary hardening change that would move the server out from under the
  volume, and fail nothing — the download would simply repeat for ever), and
  that PHP still mounts its source while the snapshot runtimes still do not.
  Both were checked by making the change and watching them fail.

- **`npm run updates:check` — the one command that asks the update endpoint.**
  §3 #2 spent three rounds on a sentence that could not be improved: the keys
  are in place, the workflow ran, and `latest.json` is still 404. Nothing in
  this repository could say more, because nothing here had ever made the
  request. The Rust tests check that the URL is spelled the way the workflow
  publishes and that the flag writing the file is still set — both true today,
  and both true while the endpoint answers 404.

  The gap between those two claims is one HTTP request wide. This closes it: the
  tool reads the manifest the updater will read and refuses to call it working
  on a 200 alone — a missing platform (six jobs write that one file, and a
  platform absent from it tells those users they are current for ever), an empty
  signature (installable by hand, invisible to the updater — the failure that
  most resembles success), a version that is not ahead of the running one. Its
  judgement is exercised without a network in `tests/updater-manifest.spec.js`,
  and the platforms it expects are derived from the release matrix so a seventh
  target widens the check on its own.

- **`npm run a11y:transcript` — what a screen reader announces, written down.**
  Y-1 has said for a long time that a person has to decide whether a label makes
  sense and that nobody had done it. The reason was never unwillingness: the
  job, as it stood, meant installing a screen reader, learning its rotor, and
  driving it blind across thirty screens in two languages — then repeating that
  after every change.

  A machine can do all of that except the deciding, so it does.
  `docs/accessibility-transcript.md` lists every page's headings and controls in
  the order the markup puts them, under the name a screen reader announces, in
  Turkish and English, with a section at the top saying what to look for. The
  remaining task is an hour of reading, and it needs no screen reader installed.

  It found something on its first read, and it was the harness rather than the
  application: a bare `createVuetify()` has no locale adapter, so every string
  Vuetify names itself came out in English and the Turkish transcript reported
  `Clear Proje ara...`. Both this and `accessible-names.spec.js` use the
  application's own instance now. What survived the correction is one real gap,
  recorded in the transcript rather than changed quietly: a search field's clear
  button announces as "temizle" with nothing after it, because Vuetify builds
  that name from the field's `label` prop alone and these fields carry a
  placeholder instead — which is a visual decision, not an oversight.

- **One screen for "why was this request slow" (B-1).** Three panes on the Debug
  tab already answered a third of the question each — php-spx says where the
  code's time went, the query log says what the database was asked, the axis
  says what else happened — and putting them around *one* request meant opening
  all three and comparing clocks by eye. No new measurement was needed; a common
  key was.

  **The key is a recording.** `spx::Report` is the only artefact here that names
  a request, says when it started and says how long it took, so everything else
  is placed against the stretch of wall clock it claims.

  **The join is by time, and the screen keeps saying so.** `timeline.rs`'s
  refusal to attribute a statement to a request is not reversed — a query moment
  still carries no request. What replaces the reader's eye is a stated window
  plus `overlaps`, which names any other recording claiming part of the same
  stretch. Attributing a statement for certain needs the *application* to say
  so, which is the thing this feature exists to avoid needing.

  **The window is watched rather than worked out, wherever it can be.** The
  arithmetic version rests on `exec_ts` being the start of the run, which is
  php-spx's field and was reasoned rather than measured — and if it is the
  moment the file was written instead, a window sits one whole duration late and
  quietly. So a recording StackVo starts itself carries the host clock from both
  sides of the request (`spx::record_observed`, kept beside the reports and
  pruned against them), which brackets the run whatever the field means. That
  covers the pane's own button and `stackvo spx-record`. The fallback is still
  there for a recording made in a browser, and the pane says which of the two it
  is showing. `cargo run --example explain_probe -- <project> <slow-path>` asks a
  live container the remaining question and prints one of three answers.

  **Two findings no single pane could make.** The N+1 is counted over *this
  request's* slice rather than the whole session, at `querylog::N_PLUS_ONE` so
  two panes cannot call three repeats an N+1 and four. And when statements land
  in the window while the trace names no driver frame at all, the profile cannot
  answer the question it appears to be answering — php-spx's `builtins` switch
  is off and the wait is charged to whichever userland function called the
  driver — which is reported rather than rendered as a fast-looking page.

  The database share is the sum of the *exclusive* time of driver frames (`PDO`,
  `mysqli`, `pg_*`, `SQLite3`, the Mongo driver), computed over the whole hotspot
  list rather than a top-25: a request whose driver frames each sit below the cut
  is exactly the one the split exists for. A framework's query layer counts as
  PHP, because the wait happens underneath it.

  24 unit tests, 19 screen tests, `request_explain` on the contract, and the
  command is on `websurface::REACHES_THE_KEYSTORE` — it hands back statement
  text, which for a development database is the data.

- **A shared tunnel can ask for a password, and can keep its address (B-7).**
  The Share pane could hand out a public URL and then only *warn* that it was a
  public, unauthenticated door into an application running on this laptop; and
  a quick tunnel's address changes on every start, which is fine for "did the
  webhook arrive" and useless for an OAuth redirect URI, a Stripe endpoint or a
  QR code on a slide.

  **Authentication is StackVo's, not the provider's.** Four of the providers can
  do basic auth themselves, three cannot, and the four that can spell it four
  different ways — one of them through a YAML policy file rather than a flag.
  So when a credential is stored, the sidecar is pointed at an nginx container
  of StackVo's own on the stack network, and that container is what asks. Every
  provider reaches it identically — including Tailscale, whose sidecar joins the
  guard's network namespace instead of the project's — and the check is
  measurable here rather than in nine vendors' documentation:
  `cargo run --example tunnel_guard_probe` starts it against real containers and
  measures no credentials → 401 with a challenge, wrong credentials → 401, right
  credentials → the application's own bytes. It also measures the failure that
  would matter most: a guard that reaches no credential refuses to start rather
  than coming up open.

  The password is generated (twenty characters, without the ones misread on a
  phone), kept in the OS keystore one entry per project, and reaches the guard
  as an environment variable — never a file in the workspace, never an argument.
  It is also the one secret in this app that can be read back, because unlike a
  token it has to be typed into a browser on somebody else's device.

  **A reserved name is sent through the flag the chosen client actually has**
  — `--subdomain`, `--url=`, `--hostname=`, or zrok's reserve-then-share — and
  then **checked**. Measured with localtunnel, which needs no account: the same
  subdomain came back twice, ninety seconds apart. Started immediately after the
  previous tunnel closed, the same request came back as
  `bitter-bulldog-88.loca.lt` with no error at all — a tunnel that works and a
  dashboard entry that points nowhere. The pane now says when the name asked for
  was not the name granted.

  A ninth provider came with it, as a row rather than a branch:
  `cloudflare_named` is the same client as the quick tunnel, running a tunnel
  somebody already created in Cloudflare — measured to read `TUNNEL_TOKEN` from
  the environment on its own, so the credential never becomes an argument, and
  its refusal (`Provided Tunnel token is not valid.`) needed a needle
  `find_failure` did not have.

  Two findings from the probe changed the code rather than the notes: an
  unquoted heredoc performs command substitution, so a backtick in an nginx
  *comment* became `sh: always: not found` and the guard would not start; and a
  credential keyed into an nginx `map` needs a hash bucket bigger than the
  credential, which nginx refuses with `could not build map_hash` on a password
  no longer than one somebody might actually choose. The configuration compares
  the header directly instead.

### Fixed

- **§3 #36 said "only for the migration" for three rounds, and the migration
  was never what was holding it up.** The way to find out was to do the
  deletion: `npm run legacy:rehearse` empties `config::LEGACY_SERVICES`, runs
  the suite, and puts the tree back. With the whole legacy half gone,
  `handover_equivalence.rs` passes **13 of 13** — every image, port and volume
  preserved, every refusal still refusing. Eight tests fail and not one of them
  is the migration.

  It never needed a default, because it does not read one as a default:
  `handover::plan` takes what the `.env` states, and where it states no version
  it asks the catalogue (`catalogue.recommended`). That is what ADR 0016 made
  services dynamic for, and the row was still describing the world before it.

  What actually holds the constant up, measured rather than assumed:
  **presence, not value** — all twenty-five `_ENABLE` defaults are `"false"`
  and `Env::bool` reads a missing key as false, so as values they are the
  identity; as keys they are not, because `mail.rs`'s `detect` names the
  catcher a workspace would enable by asking which one is *declared*, and on an
  untouched workspace the only thing declaring it is this constant. **Live
  credentials** — `db.rs` reads the passwords below for instances that have
  already migrated, and `skeleton.rs` scans them to keep a real credential out
  of the binary. **An invariant** — `ENABLE`, `VERSION` and `VERSIONS` travel
  together per service, so removing the forty-seven version-shaped keys while
  the enables stay is not available; it was tried, measured and reverted.

  `mail::detect` had no test at all, and it is the function every other thing
  in that module hangs off. It has one now, written because the rehearsal went
  looking — pinning that an untouched workspace still names Mailpit, and saying
  in its failure message what a silence there would mean.

  Two more sentences in the constant's own doc comment were false and had
  survived because nothing read them: that without a `VERSION` default there is
  no tag to migrate, and that credentials are deliberately absent — they start
  at `SERVICE_MYSQL_ROOT_PASSWORD`, ten of them.

  The date stays at 0.4.0 and now forces the right question. What deletes the
  constant is not the end of migration support; it is the app answering "which
  services does this workspace know about" from the catalogue rather than from
  `.env` presence, which is a product decision and is now §5's rather than an
  assumption inside a comment.

- **Deleting the constant opened with a compile error rather than a red test.**
  `config::tests::the_merge_keeps_every_entry_and_invents_none` asserted
  `EMBEDDED[SETTINGS.len()] == LEGACY_SERVICES[0]`, and `deny(unconditional_panic)`
  rejects a constant index into an empty array — so emptying the legacy half
  stopped the crate's tests from building instead of failing one of them. The
  test now lays the two halves end to end and compares elementwise, which
  checks every entry rather than four corners and survives one half going away.

- **`npm run contracts:check` reported an error about a directory the product
  stopped having.** The script passed `--root ../stackvo`, a sibling-checkout
  layout that no longer exists; run from inside the repository it resolves to
  the repository, which is the mistake the validator's own comment calls "one
  keystroke away". It now runs the way CI runs it.

  Suite A was looking in a stale place besides. `workspace.rs` removed
  `<root>/projects` as a default on purpose — a hidden directory nobody chose
  would satisfy the very requirement the setup gate exists to hold — and the
  tree now lives wherever `projects.path` points. The validator resolves it the
  same way the app does: the pointer, then `STACKVO_PROJECTS`, then the old
  layout as a last resort, so a real workspace with a dozen projects in it no
  longer reports `NO_MANIFESTS`.

- **The gate on §3 #36 was green while missing a third of what it counts.**
  `legacy_env_claims.rs` is the checklist for the day
  `config::LEGACY_SERVICES` is deleted: it names every module that reads a
  `SERVICE_*` default, and it fails in both directions so the list cannot
  drift. It recognised exactly one spelling of "reads one" — a call to one of
  seven `Env` accessors — and two modules never use it.

  `db.rs` keeps the key names in its own per-engine table
  (`password: "SERVICE_MYSQL_ROOT_PASSWORD"`) and hands them to `Env::get` and
  `Env::bool`, because the handover deliberately leaves a migrated instance's
  credentials in `.env`. `mail.rs` builds `SERVICE_MAILPIT_ENABLE` out of a
  prefix constant and a suffix, which is how `detect` decides which of the two
  catchers an unmigrated workspace has. Both read a legacy default on every
  call. The checklist said four modules, the tree held six, and the document
  repeated the four — a third of the deletion missing from the plan for it,
  with a passing test on top.

  Naming the key is now the second spelling, and it is the one that cannot be
  avoided: a module that reads a `SERVICE_*` default has to say which one
  somewhere. Only the bare family prefix `"SERVICE_"` is excluded, because it
  names no service — counting it would make a reader of `template.rs`, whose
  `PREFIXES` list decides which *variable names* the renderer substitutes. The
  walk also became recursive, so `src/bin/` counts; the CLI and the MCP server
  read nothing today, and that is now established rather than assumed.

- **The deletion date was a gate in one file and prose in another, with
  nothing between them.** §3 #36 argues that 0.4.0 is "not prose but a gate",
  and the gate — `LEGACY_SERVICES_GO_AT` — was a constant no document was held
  against. Moving it to `(0, 9)` left §3, §4 and §5 all saying 0.4.0 and every
  test green: the deletion would have stopped being due, silently, which is
  the one outcome naming a version was meant to make impossible. The three
  places and the constant are now one fact.

  The count in §3's row is held the same way. §8's rule stands — a *status* in
  §2–§4 cannot be gated, because "not done" is not a property of the code —
  but a number in a row is, and this one had already gone stale.

- **`--no-fail-fast` paid for itself on its first run.** With it, one Windows
  job listed four remaining failures at once instead of the one it would have
  reported before — and three of the four were tests asserting the platform
  they were written on, which is exactly the class §3 #35 has been working
  through.

  `independence.rs` built its expectation from a raw host path and the compose
  file it was reading says `/c/Users/...`, because `paths::to_docker_mount`
  already does that job — so a correct mount failed with "the source mount did
  not follow the project tree". `worktree_flow.rs` created its own git
  repository and let it inherit the machine's `core.autocrlf`, so `git worktree
  add` handed back a manifest with `\r\n` in it and the assertion that the
  branch's committed manifest is *untouched* named this application as having
  rewritten a file it never opened. `foreign_import.rs` gated its symlink setup
  to Unix and left its assertions ungated, so on Windows it measured a tree
  where the setup had quietly not happened and reported the reader as broken; a
  Valet site *is* a symlink — `imports::linked` uses `read_link` — so the link
  has to be real wherever this runs, and on Windows that needs Developer Mode
  or elevation, which is not this test's to demand, so a refusal skips out loud.

- **Generated file labels were half one path convention and half the other.**
  `configs/mysql-8-0\my.cnf.tpl` — a `/`-written prefix joined onto a
  `Path::display()`. A label is an identifier: it appears in the generated-files
  list, it is what somebody searches for, and it is compared against in tests,
  so it has to be one string rather than one per platform. `paths::to_label`
  is that line given a name, and `applog.rs` — which had already reached the
  same conclusion on its own — now calls it instead of repeating it.

- **A Windows checkout was CRLF, and the tests that read this repository's own
  source had never seen one.** Git for Windows ships with `core.autocrlf=true`
  and the Actions runner inherits it, so `cfg_regions.rs` — which finds out
  which attribute belongs to which function by splitting on a blank line —
  searched for `"\n\n"` in a file that only had `"\r\n\r\n"`, never split, and
  read the window back into the function above. It reported the keystore's real
  backend and its in-memory fake as carrying the same `cfg` gate: a
  security-shaped assertion failing for a reason with nothing to do with
  security. `.gitattributes` pins `eol=lf` and closes the whole class;
  `workflow_parity.rs` fails if it is deleted.

  It had been hiding behind another failure for a full round. `cargo test`
  stops at the first test binary that fails and `agent_install` sorts first, so
  fixing that one is what let this one be seen — which is also why "nineteen
  failures" was never a finished count.

- **The screen-reader transcript timed out under coverage and took the floors
  gate down with it.** Twelve page mounts against a five-second default is fine
  until the coverage job instruments every module it loads, and then `test:js`
  is green on the same commit where `test:js:coverage` dies, no front-end report
  is written, and the gate two steps later fails naming itself. The generator
  gets a timeout that matches what it does.

- **`agents` resolved the home directory differently on Windows, and the
  difference was invisible from a Mac.** `dirs::home_dir()` reads `$HOME` on
  Unix and ignores `%USERPROFILE%` on Windows, asking the shell for
  `FOLDERID_Profile` instead — so this module honoured a relocated home on two
  platforms out of three, and the third is the one nobody here runs. It
  surfaced as the last failing test on Windows: `agent_install.rs` points both
  variables at a scratch directory, which is the only way it can exercise the
  write half without editing the profile of whoever is running it. Skipping the
  test there was the other option and it is the trade §3 #35 already caught
  once, when `runner.rs` looked thoroughly tested on Windows while having no
  coverage at all.

- **Six red release jobs were read as one Windows problem for three rounds,
  because nobody opened the logs.** They had three different causes: the two
  macOS rows failed on `key_ceremony`, the two Linux rows on `elevate_probe`,
  the two Windows rows on §3 #35's platform assumptions. Three of the three
  were fixed the same evening, in a commit that landed *after* the tag — so the
  release sat waiting on Windows while two thirds of its failures were already
  repaired. `src-tauri/tests/workflow_parity.rs` now holds the release job's
  environment to at least CI's, and the release keeps its test output (below),
  because a release log that exists only as six live browser tabs is a log
  nobody reads.

- **A green release run would still have left the endpoint at 404.** The
  updater asks `releases/latest/download/latest.json`; the workflow creates the
  release with `releaseDraft: true`. GitHub resolves `releases/latest` to the
  latest *published* release and never to a draft, so the two settings
  contradicted each other silently and the resulting 404 read as a failed build.
  The draft stays — `fail-fast: false` makes a partial matrix ordinary, and a
  `latest.json` naming four of six platforms tells the other two they are
  current for ever — but the run now says on its own summary page that a person
  still has to publish it, and `updater_endpoint.rs` fails if those two settings
  ever drift apart again without that being said.

- **A failed release left nothing behind to read.** What §3 #2 recorded after
  the first real run was that how many failures remained had not been read from
  the log, because reading six red targets meant opening six live logs in a
  browser. The suite runs with `--no-fail-fast` so the log reaches the end
  instead of stopping at the first crate, and the output is kept as an artifact
  when the step fails. `PIPESTATUS` is what stops the pipe to `tee` from
  reporting success on a failing suite.

- **Two buttons on one page both announced as "Temizle" (Y-1).** On Logs and on
  Dumps, the search field's clear icon and the button that empties the view were
  read out identically, for two different actions — a listener could not choose
  between them. Each says what it clears now: "Görünümü temizle" and "Dump
  listesini temizle". Mail's search field, which carried a placeholder and no
  name of its own, got one.

  Found by reading `docs/accessibility-transcript.md` — which is the workflow
  that transcript was added for, working on its first pass. The i18n suite then
  caught the first attempt at fixing it: adding a `clearAria` key left
  `logs.clear` and `dumps.clear` translated and unreachable. The button is an
  icon, so those keys were only ever its accessible name; widening their values
  is the fix, and it leaves nothing dead behind.

  **Y-2 is folded into Y-1 and its row is gone.** What was left of both was one
  task, and it is not a machine: nobody has used this application with a screen
  reader. The transcript says what is announced; it does not say what using it
  is like — whether a flow can be finished by ear, whether focus lands where it
  should after a dialog closes, where it becomes tiring. The accessibility
  statement does not claim otherwise.

- **Eleven buttons on the Dashboard were all called "what this card is for"
  (Y-1).** The help button repeats on some forty cards and announced the same
  sentence on every one, so a screen reader user on the Dashboard heard it
  eleven times with no way to choose. Each one had a name, so every automated
  name check passed — the failure only exists at page scale, across controls
  that are individually fine. It carries the name of the card it belongs to now,
  from the wrapper that already knew it.

  `tests/accessible-names.spec.js` mounts the six pages and keeps three things
  no per-component check can see: no control announced by its role alone, no
  control named with a word that says nothing on its own, and a floor under the
  share of *distinct* names on a page. A threshold rather than "no duplicates":
  a Delete button on every row of a table is fine, because the row names itself.

  This was the last part of Y-1 that was written off as needing a person. It
  was not — "is this label meaningful" has a mechanical floor and nothing was
  standing on it. What genuinely needs a person is the top of that floor:
  whether a particular word is the right word.

- **A screen reader reached the template chooser after the form it decides the
  meaning of (Y-1).** The new project drawer put the form first in the markup
  and pulled the chooser above it with `order: -1` on a narrow window. The eye
  saw chooser-then-form; the markup said form-then-chooser, so a screen reader
  and a keyboard were handed the sequence backwards — you filled in fields whose
  meaning depends on a template you had not been offered yet. The chooser is
  first in the markup now and `order` moves the box rather than the sequence.

  Y-1 listed three things as needing a human: whether a label makes sense,
  whether an error says what to do, whether the reading order matches the visual
  one. **Two of the three are not judgements.** Reading order against visual
  order is a fact, and `tests/reading-order.spec.js` now keeps it — every
  `order:`, every `*-reverse` and every positive `tabindex` in the tree, with
  the rule being a sentence rather than a ban: re-ordering is legitimate, doing
  it *silently* is not, because a divergence nobody wrote down is one no
  reviewer can find. That is exactly how this one survived. Whether an error
  says what to do is countable: 178 of 629 error constructions carry a
  suggestion, which is a number the row never had.

  What is left of Y-1 is the first question — whether a label *makes sense* —
  and it is the same person Y-2 is waiting for. Two rows, one remaining task.

- **The tray icon had no name until the first engine check landed (Y-2).** A
  tooltip is a status item's accessible name on macOS, and it was only ever set
  by `refresh` — so between the icon appearing and the first check returning, a
  screen reader was handed an unnamed control in the menu bar, and a check that
  hangs left it that way for good. It is set when the icon is built now, to the
  product name alone: there is no summary yet at that moment, and a tooltip
  claiming a state nothing has checked would be worse than a short one.

  This is the third defect of the same family the accessibility probe found, and
  the family is the point: a control nobody could see was unnamed, because
  nothing on screen looks wrong when it is. The tray was also the surface the
  row named as out of reach — macOS puts a status item on its own menu bar, and
  the probe reads it there.

  The probe then corrected its own author. Its first version asked for the
  item's `name`, which is `AXTitle`, found it empty and reported the status item
  as unnamed — while the tooltip was sitting in `AXHelp` the whole time. An
  icon-only status item has no `AXTitle` on macOS by construction; giving it one
  means visible text in the menu bar, which is a product decision and not an
  accessibility fix. It reads all three name attributes now and prints which one
  carries the name.

- **The app menu offered "Quit stackvo-desktop" (Y-2).** Tauri's predefined
  Hide and Quit items interpolate an application name, and a `None` label fills
  that hole with the **crate** name. `menu.rs` had already rebuilt that submenu
  once for exactly this reason — its own comment says `Menu::default` "titles it
  with the crate name, so a `stackvo-desktop` sat in the menu bar of an app
  called StackVo" — and fixed the submenu's title while leaving the two items
  inside it. Both now take their text from the label catalogue with the product
  name substituted, which also puts them in the interface's language.

  It stayed there because it could not be seen from anywhere a test looked, and
  a person reads a menu item by its verb. A screen reader says the whole string.

  **The blocker on this row was named wrong, and that is what kept it
  unstarted.** It said the native surfaces need `tauri-driver`, which does not
  run on macOS — but **WebDriver does not reach a native menu on any platform**;
  it drives the web view. A Linux runner with the driver installed could no more
  enumerate a menu bar than this machine can, so the thing being waited for was
  never going to answer the question. What answers it is the accessibility API,
  the layer a screen reader itself reads.
  `src-tauri/examples/native_ax_probe.rs` reads the running application's tree
  through it — no new crate, no driver, no CI runner, just the app up and one
  permission granted — and found both this and the unnamed About window on its
  first run.

  What is left of Y-2 is the judgement half, and it is Y-1's problem in the
  native surfaces: whether the reading order makes sense, whether a label is
  meaningful, whether the tray menu is usable under VoiceOver rather than merely
  present. That needs a person.

- **The window that says which version is installed had no name (Y-2).**
  `menu::open_about` built the About window with `.title("")`, and a window's
  title *is* its accessible name — what the window list announces, what the
  Window menu shows, and what a screen reader reads when focus lands inside. So
  the one window whose whole job is to answer a question was handing the answer
  over unnamed. The title now comes from the same catalogue as the menu item
  that opens it, which puts it in the interface's language rather than the
  build's; `tray::relabel` renames it when the language changes, and
  `open_about` renames it on the way back up as well — a window hidden across a
  language change never saw `relabel` at all.

  This was found by rejecting the reading that made Y-2 unstartable. The row
  said the native window cannot be audited without `tauri-driver`, which does
  not run on macOS, and that was taken to mean nothing about it is checkable. A
  driver is needed to **operate** those surfaces; it is not needed to know
  whether they have names. `src-tauri/tests/native_window_claims.rs` now keeps
  six facts a driver was never required for — every declared window has a title,
  no new window can be built without one, the About window's name comes from the
  catalogue, both re-title paths exist, the main window can be resized to the
  scale `docs/accessibility.md` offers as its reflow answer, and the statement
  still says the audit is owed. The audit itself still is: keyboard operation of
  the native chrome, focus order through it, and the tray menu under a screen
  reader.

- **The window said it was English whatever language it was speaking (Y-3).**
  `index.html` ships `lang="en"` and nothing ever changed it, so a Turkish
  interface announced itself as English for its whole life — WCAG 3.1.1, and the
  criterion everything else about language rests on, because a screen reader
  picks its voice and its pronunciation rules from that attribute.
  `docs/accessibility.md` claimed the interface language "is announced on the
  document"; the sentence was true about the attribute existing and false about
  what it said. It is now set from the active locale beside `dir`, on the same
  element and from the same value, including a locale pack's own tag.

  **And passages in another language are marked (3.1.2).** Two kinds, and they
  take different values. The message Rust wrote is the app's own English and is
  marked `en`. Everything a container produced — a log line, a captured dump,
  docker's output, a statement's literals — carries `lang=""`, HTML's
  "undetermined": nothing here knows what language somebody else's application
  writes in, and marking it `en` would be a guess stated as a fact, wrong for
  exactly the projects this app's second language exists for.

  `tests/language-of-parts.spec.js` holds both. The passages are scanned from
  the sources rather than rendered, for the reason `a11y.spec.js` is: a mount
  test asserts on text and roles and would pass with every attribute missing,
  which is also how this regresses — an attribute is invisible on screen.

- **Saving a project's settings no longer deletes the half of its manifest the
  form has no fields for.** Found while writing a test project that declares
  every block the contract has: `providers` was read straight off the file by
  `provider::parse` and never reached `manifest::to_json`, so the writer — which
  re-renders the whole document on every save — wrote a file without it. The
  button that fetches staging simply stopped being there.

  Measuring the round trip rather than fixing the one report found four more,
  all of the same shape and all silent:

  - `hooks` serialised as `{"kind": "exec", "argv": [...]}` and `commands` as
    `argv`, neither of which the reader accepts. `ipc.json` already said these
    are "keyed maps whose values are defined by project.schema.json" — the
    contract was right and the implementation was not.
  - a sidecar serialised its empty defaults, and `command: []` is a step list
    with no first word, so the reader refused the sidecar it had just written.
  - `"node": null` is what a PHP project's payload carries for the runtime it
    does not have. Posted back, C-02 read it as a node block and refused the
    manifest.
  - the settings sheet builds the whole file out of a form, and the form kept
    only the fields it draws. `lan_share`, `php.xdebug`, `hooks`, `schedule`,
    `commands`, `sidecars` and `providers` were dropped on every save —
    `blankForm`'s own comment says "a field it forgets is a field that Save
    deletes", and seven of them were being forgotten.

  Each half now serialises the way the file spells it, `provider::parse` keeps
  the file's order instead of alphabetising (the reordering `sidecar::Declared`
  and `quickcmd::Declared` each keep an order list to avoid), and the form
  carries what it does not edit.

- **The manifest editor edits the file.** It was showing the reader's view of
  it, which is spelled for the IPC boundary — `documentRoot`, `lanShare` —
  while the file spells them `document_root` and `lan_share`. Saving what it
  displayed turned a project's document root into `public` and switched LAN
  sharing off, with nothing on screen saying so. `project_manifest_text` returns
  the committed bytes, the way the machine-local override editor has always been
  given them, and refuses with the reader's error when the file does not parse
  rather than offering to save over something this app never understood.

- **Three red CI jobs, and none of them was the product.** Each test was
  asserting the machine it was written on:

  - `supervisor::a_docker_command_with_input_is_given_a_standard_input` ran
    `docker exec <container> false` for real. macos-latest has no `docker`
    binary, so it failed for a reason that says nothing about this code — and
    on a runner that *has* one it could not fail the way it was written for:
    `false` exits non-zero with or without `-i`. The argv is built by
    `docker_argv` now and asserted as a string, which is the flag in the
    position that matters, everywhere.
  - `dns::a_file_that_is_not_ours_is_summarised_rather_than_hidden` wrote a
    fixture saying `port 53` and called it somebody else's file. On Windows
    `PORT` **is** 53, so the "foreign" file was byte-for-byte the one this app
    writes — the test failed on the single platform it was making a claim
    about. The fixture uses a port that is nobody's on any platform and asserts
    that it is not ours, so the next person to pick a number cannot pick this
    one.
  - `architecture_claims::the_counts_match_the_tree` was the module count, and
    is covered by the claim refresh below.

  Both habits are now refused where they happened, in `cfg_regions.rs`, beside
  the rule that keeps a POSIX shell out of `runner.rs`'s tests — and scoped the
  same way, to the one file that had them. A test in `supervisor.rs` may not
  call `.exec()`; a test in `dns.rs` may not write `port 53` or `port 15353` as
  a literal. Neither gate needs a Windows machine to fire, which is the point:
  `tools/before-push.sh --all` type-checks the Windows branch but cannot *run*
  its tests, so the class of bug that only fails there has to be caught by
  reading rather than by running.

- **`tools/linux/run.sh` builds what it needs, the way the Windows branch
  already did.** Both Linux steps of `before-push.sh --all` failed on this
  machine at `resource path binaries/stackvo-aarch64-unknown-linux-gnu doesn't
  exist` — `tauri-build` checks every `externalBin` exists on any cargo build of
  the package, and it looks for the *container's* triple, so the host's sidecars
  are not ones. The `--windows` branch has written stubs for exactly this since
  the day it was added and says why; the Linux side never did. The probe run
  writes stubs (nothing executes them) and `--driver`, which builds the
  application the way it ships, builds the real ones — `beforeBuildCommand`
  refuses a placeholder, correctly.

- **A service the catalogue publishes is no longer reported as unknown.**
  `dragonfly`, `soketi`, `prometheus` and `graylog` are published packages that
  this machine can install from Market — and every project declaring one was
  told on its own page that "this version of StackVo has no template for it".
  The check was reading the list compiled into the binary, whose own note in
  `env.schema.json` calls it "the vocabulary, not an inventory" and says it has
  to grow when the catalogue does. It had not, twice: Solr and ClickHouse were
  added late, these four not at all.

  A list inside a binary cannot follow a repository that ships on its own
  schedule, so the answer no longer comes from one. `market::is_known_service`
  asks the **catalogue this machine has fetched** first — the file the Refresh
  button in Settings writes — and falls back to the compiled-in vocabulary,
  which is what a machine that has never fetched still knows and what keeps a
  typo a typo. It is re-read when the index changes rather than once per
  process, so a refresh is visible without restarting the app.

  The three places that judged an id separately — the manifest reader, the
  requirements card and `.env` detection — now ask that one function, because a
  Market that installs a service beside a card that calls it unknown is the
  disagreement worth removing. The four ids are added to the vocabulary as well,
  so a machine with no catalogue yet is right about them too.

- **`project.schema.json` describes `schedule`.** The reader has accepted the
  block since the Scheduled jobs panel shipped and the writer emits it, but the
  one document both halves are meant to share never mentioned it — so the
  contract described a manifest the app does not have. Labels, the portable cron
  subset, the argv rule and what `enabled` means are now written down where the
  other blocks are.

- **The counted claims in `README.md`, `ARCHITECTURE.md` and `docs/durum.md`
  are current again** — modules, commands, events, front-end files and the
  `ipc.js` wrapper count had drifted, and their gates had been red for it.

### Added

- **The suite runs on Windows, and the two product bugs it found are fixed.**
  §3 #35's open half was whether the branches *run*, not whether they compile.
  They run now, and the first real Windows run failed nineteen tests. Sorting
  them apart mattered more than the count: **two were the product**, the rest
  were tests asserting the platform they were written on.

  `dns.rs` built `/etc/resolver` paths with `PathBuf::join`, which uses the
  **host's** separator — so the same plan rendered `/etc/resolver\test` on
  Windows and every command built from it named a file that cannot exist
  anywhere. A path belonging to another operating system is a string; only a
  path on this machine is a `Path`.

  `runner.rs` had no Windows coverage at all while looking thoroughly tested:
  nine of its tests drove `sh -c`, and Windows has no `sh`. That is the module
  which spawns `docker compose`. They run `node` now — already required to build
  this repository, writes the bytes it is given on every platform, one payload
  instead of a per-platform pair. `cmd` was the obvious alternative and can
  neither write a bare carriage return nor omit a trailing newline, which are
  the two cases those tests exist for.

  The rest were assertions about separators. `imports.rs` wrote paths into JSON
  unescaped, so on Windows the fixture was invalid JSON, the parser found no
  sites, and five tests failed on a claim about Valet rather than about a file
  that never loaded; it also declined to create the symlinks a Valet layout
  *is*, which is a fixture quietly building less than it says. `agents.rs`
  compared a rendered path against a POSIX literal. `stats.rs` asserted a
  tighter tolerance than `breakdown_is_credible` enforces, so a breakdown the
  code calls fine failed the test that checks it — one constant now.

  `cfg_regions.rs` refuses a POSIX shell in `runner.rs`'s tests, scoped to that
  file on purpose: `quickcmd.rs` names `sh` throughout and is right to, because
  that `sh` runs inside a container. The first version flagged it, and a gate
  that fails correct code is one people work around.

  **Three other jobs went green with it.** `driver` closes §3 #12 — the suite
  had run 5/5 in a container here and the last open question was CI, which is
  now answered. `coverage` and `ubuntu-latest` were red for reasons the previous
  round fixed. Seven jobs, six green; `windows-latest` is the one left.

- **`tools/before-push.sh` now asks what CI asks, which it had been claiming
  since the day it was written.** Its opening line is "everything CI will ask,
  asked here first" and ADR 0030 is the decision behind it. It was not true, and
  the first release found out the expensive way: three separate red jobs, each
  from a gate the script had never run.

  **The bundle budget** had been over its ceiling since the in-app help round —
  three merges of red CI on a number nobody was reading. `measured` still said
  1344.7 KB, so "since measured: +170 KB" was reporting drift from a figure last
  checked several features ago, which the file's own comment calls a formality
  rather than a budget. Re-measured, and the ceiling raised to 1700 as a stated
  decision: trimming 15 KB out of `index.js` is the better answer and is not a
  release-day job.

  **The coverage floors** were failing on an empty report, and the cause was two
  steps upstream: the `coverage` job never wrote the placeholder sidecars, so
  `cargo llvm-cov` could not build at all. Both measuring steps are
  `continue-on-error: true`, so the build error went by as a green tick and the
  gate two steps later blamed itself. A step that goes green with its work not
  done is worse than one that fails.

  **And `cargo deny` was never run locally at all.**

  ## The toolchain pin was on disk doing nothing

  `src-tauri/rust-toolchain.toml` pins 1.96.1, and its comment says exactly why:
  clippy gains lints with every release, this repo runs it with `-D warnings`,
  so `stable` means the build depends on the day it runs. rustup resolves that
  file from the **working directory** — and every cargo step in `ci.yml` ran
  from the repository root with `--manifest-path`, where it is invisible.

  So CI compiled with whatever `stable` was that morning while
  `tools/before-push.sh`, which runs from inside `src-tauri`, compiled with
  1.96.1. Clippy 1.98 then failed a release on a `useless_format` lint that
  could not be reproduced locally — the exact "green here, red there" the pin
  was written to prevent, and it had been true of one job in the file all along.
  The `coverage` job had already found the trap and worked around it alone.

  Every workflow now runs cargo from `src-tauri`, and `cfg_regions.rs` refuses a
  workflow that goes back to the root. `cargo cyclonedx` is exempt and named
  rather than pattern-matched: it reads a dependency graph, and which compiler
  resolves it does not change the answer.

  The coverage floors are behind `--all` in the local script, because
  `cargo llvm-cov` re-instruments and re-runs the whole suite — five minutes
  against seconds for everything else. What that buys is a place to run it
  before a release rather than reading about it afterwards.

- **The signing ceremony is written down, and it is a script.** §3 #2 was never
  an engineering problem — the endpoint has been correct since ADR 0025 and the
  updater carries a public key. What was missing is that nobody had performed
  the ceremony, and the reason nobody had is that there was no ceremony to
  perform: the updater key had one sentence in a workflow comment, the content
  key (ADR 0015) had nothing at all, and the two were reached by different
  tools.

  `tools/keys.sh` is the procedure now — `generate`, `check`, `sign`. A script
  rather than a page, for the reason this release deletes a design document: a
  ceremony written as prose drifts, and prose that has gone stale about *keys*
  is discovered after somebody has already generated one and put it somewhere.

  **Writing it found a real defect, and it was in the load-bearing part.** ADR
  0015 pays for two keys with two places to leak from, and what buys that back
  is that the *procedure* is shared — same tool, same storage, same rotation.
  `tauri signer` is the tool the updater ceremony already uses, and a signature
  it produced was one the app **refused**: it wraps the whole minisign file in
  base64 (its updater manifest carries a signature as one JSON string) and
  `signing::verify` read only the plain form. The refusal said "invalid encoding
  in minisign data" — a message about bytes, for a problem about an envelope,
  raised at the one moment somebody is closing the chain for the first time. So
  the registry key would have needed a second tool, which is the second
  procedure ADR 0015 says goes unmaintained.

  `verify` peels either envelope now. It widens what can be *read*, not what is
  accepted: a wrapper around bytes no trusted key signed still fails one line
  later, and three tests say so. The passing one uses a signature the real tool
  really produced, against a throwaway key whose private half never left the
  scratch directory — the same reasoning the existing vector has for coming from
  `minisign-verify`'s own tests rather than from this repository.

  **The chain's first link had never been shown to work.** Every test of the
  signed path was a refusal — no key pinned, key checked before the signature
  file, index going backwards — and a chain with no passing case is one whose
  first success is somebody's first release. `tests/signed_refresh.rs` is that
  case: a signed index, a machine that trusts the key through
  `policy.market.additionalKeys`, and a refresh that completes. Its own binary,
  because `policy::current()` is a `OnceLock` and a test that points
  `STACKVO_POLICY_FILE` somewhere has to own its process.

  `tests/key_ceremony.rs` holds four rules a script cannot hold on its own,
  because they are about the tree rather than about a run. **No private key is
  committed** — the one failure that is unrecoverable rather than inconvenient,
  since rotating the updater key means every machine already running StackVo can
  never be updated again. **The updater key and the registry key are not the
  same key**, which is vacuous today and is exactly why it is worth writing: the
  moment somebody fills `PINNED` is the moment reusing the working pair saves an
  afternoon and costs the property the separation was for. **No key is both
  pinned and retired**, a state `verify` skips silently, so the retirement would
  look done. And **the ceremony never puts a password on a command line**, where
  the process table and the shell history both keep it.

  `tools/before-push.sh` runs `keys.sh check`. Not something CI asks — CI cannot
  see the keys — but the same instinct, and the last place an unrecoverable
  mistake is visible before a push.

  **The ceremony was then performed** (ADR 0033). Both key pairs exist, the
  updater's public half is in `tauri.conf.json` and the registry's is in
  `signing::PINNED` — separate keys, which `key_ceremony.rs` now enforces from
  both sides.

  Pinning a key turned two tests over rather than breaking them. The pair that
  asserted `PINNED` was empty and that a signed refresh failed on the *missing
  key* could not survive a build that has one; what a build with a key does is
  get past the key check and be refused for the **signature** it cannot find,
  which is a different sentence and the honest one.

  It also found a field that was right by accident. `market_status.signed` was a
  hard-coded `false`, true only for as long as nothing could verify. It reads
  `market/source.json` now and answers the question that belongs on a screen —
  *was the index this machine holds verified* — rather than *can this build
  verify*, which a machine that pins the official key and last refreshed from a
  folder would have answered wrongly. `verifiedBy` names the key, because
  "verified" and "verified by whose key" are different answers on a machine with
  a mirror.

  What is left of #2 is two acts and neither is code: the repository secrets,
  and a `v*` tag. And one more for the chain's far end — a pinned key signs
  nothing, so the packages repository still has to sign and publish its own
  `registry.json` with the private half, which is why that key is deliberately
  not a CI secret.

### Removed

- **`docs/servis-market-mimarisi.md`**, and its own closing section is what asked
  for this: _a design document, once the thing it describes exists, is a second
  source of truth and it drifts._ It said what the package system should become,
  in what order, at what risk and with what exit criterion. All eight phases
  shipped. Everything it described now has a code path, a contract file or a
  decision, and a report that describes those from the outside is a fourth place
  for them to disagree.

  **What kept it alive past that point was not its content but its citations.**
  Thirteen Rust modules, its tests and three contract files pointed at it by
  section number (`§4.4`, `§9`, `Faz 2`), and deleting it would have left every
  one of them addressing nothing — worse than a stale document, because a reader
  who cannot find a reference does not learn that the reference was wrong, they
  learn that this repository's comments cannot be followed.

  So each citation went to whatever is now the source rather than being dropped:
  the version manifest to `contracts/package-version.schema.json`, the compose
  allowlist to `contracts/compose-policy.json`, the index and its hash chain to
  `contracts/registry.schema.json`, and the **threat model to `SECURITY.md`** —
  which had no row for packages at all, despite the app having started
  downloading service definitions and handing them to Docker. T-1 to T-8 are
  there now with the thing that answers each, and that is a gap closed rather
  than a paragraph moved.

  Two decisions in it were nowhere else and are now ADRs. **0032** records why a
  version is a directory and not a template with a version variable — the
  differences between series are real and measured (MySQL 5.7's authentication
  plugin, Elasticsearch 8's security default, RabbitMQ's `management` tag,
  MongoDB's config keys), and a template carrying them with conditionals would be
  a *program* fetched from a repository and run, which is the one thing the whole
  design exists to prevent. **0013** gained the four transports it rejected and
  why: a git submodule, a full clone, an npm package, and one large
  `services.json`.

  **The phase numbers went with it.** `Faz 1`…`Faz 7` were a delivery plan, and a
  delivery plan stops being true by succeeding; a module doc opening with "Faz 2
  of <deleted file>" tells a reader the order the work happened in — which git
  already carries — instead of telling them what the module does. Eleven module
  docs were rewritten to say the second thing, and three of them were describing
  a tree that no longer exists ("nothing in the tree today decides that", "a
  fixed array of twenty-five compiled-in templates").

  `src-tauri/tests/no_dangling_docs.rs` holds all of it: the file is gone,
  nothing cites it, and nothing dates itself by a phase. `docs/durum.md` §1
  carries the tombstone — the sentence a reader who half-remembers the document
  needs — and `CHANGELOG.md` is deliberately outside the scan, because an entry
  that described the file while it existed is still a true account of that
  release and rewriting it would be editing history to keep a test quiet.

### Added

- **A workspace can take over one file of a package it did not write.** P, and
  the last of the package system's three extension points — the other two,
  authoring a package and the enterprise half of a third-party source, were
  already here.

  **The obstacle was a hash, not a missing editor.** Before packages, somebody
  who needed one line of the Redis configuration different edited
  `core/templates/services/redis/…` and `skeleton.rs` made that edit win. ADR
  0016 deleted that directory and the replacement is a _verified_ tree: a
  manifest states the sha256 of every file it ships and `pkg::verify` checks
  them on every read. So the same edit now produces a package that refuses to
  load, complaining about bytes rather than about the line just typed — the
  exact obstacle `authoring.rs` was written for, arriving from the other side.

  **Re-sealing the fetched package was the obvious answer and is the wrong
  one.** Sealing is right for a package you are writing; applied to one you
  fetched it rewrites somebody else's manifest so it describes your edit, and
  the next `market_install` undoes it with no record that it ever existed. An
  override has to survive a reinstall and it has to be visible.

  **So the copy lives beside the package rather than inside it** —
  `<root>/overrides/<service>/<version>/<the file's own path>`. Nothing under
  `market/packages/` is touched, so the hash chain is exactly as it was and a
  reinstall is exactly as safe. This is `skeleton.rs`'s mechanism with the one
  change that matters: skeleton's overrides live _at_ the path they replace, and
  these live next to it, because that path is now covered by a hash.

  **Templates, never the manifest, and that is the load-bearing rule.** The
  manifest declares the image, the ports, the volumes and the settings, and the
  render context is built from it; a workspace that could override it could run
  one thing while the catalogue reported the published one, and every statement
  this app makes about what is installed would become a statement about what
  _was_ installed. A template cannot do that — whatever it says is substituted
  from a context the manifest defines and then passed through `compose_policy`,
  the same allowlist a downloaded fragment goes through, on the same code path,
  after substitution.

  **One rule needed a gate, and writing it found a real inconsistency.** The
  layering is a single line in `pkg::Tree::file` and works only for a tree
  opened _with_ the overrides attached, which `market::catalogue` is the one
  place that does. An override only some screens honoured would be worse than
  none: compose would render from the workspace's fragment while the connection
  string, the settings sheet and the doctor described a different one, and the
  symptom is a service that behaves unlike everything written about it.
  `overrides_claims.rs` holds that, and on its first run named `doctor.rs`,
  which was still opening a bare tree.

  Two more rules are held the same way: the overridable list is built from
  manifest _fields_ rather than from whatever is on disk — the kind of thing a
  later refactor generalises into "every file in the directory" without noticing
  what it has opened — and `overrides.rs` never writes into the package tree.

  Reverting deletes and does not restore. Writing the published bytes back into
  the override would leave a file on disk that means nothing, which is the state
  `skeleton.rs` documents at length as the one it exists to stop producing.

  `policy.market.allowOverrides` is the organisation's half. A note rather than a
  lock (ADR 0009), and when it is off the files stay on disk and `doctor` names
  them — bytes that are being ignored are exactly the thing somebody needs told.
  Two policy keys that refuse an action, `allowedSources` and this one, were also
  missing from `policy_status`, whose own comment says the pane exists so that a
  person who has just been refused can read _which_ rule refused them; both are
  on the wire now.

  Recorded as **ADR 0031**. It was the last open item in the package
  architecture's plan, and closing it closed the plan — see Removed.

- **A project can fetch its data from where it really runs, and send it back.**
  A-1, the largest gap the competitor review found and the only one that is a
  whole category rather than a feature: DDEV ships `ddev pull`/`ddev push` with
  recipes for Upsun, Acquia, Lagoon and Pantheon, Lando and Herd have their own,
  and the concept did not exist here at all.

  **Everything dangerous about it was already answered next door.** A provider is
  a command a *repository* declares that reaches the network with the
  *developer's* credentials — which is `hooks.rs`'s threat model word for word,
  so this borrows its answers rather than inventing worse ones. A step is an
  argv array and there is no shell. It runs in a container, never on the machine,
  and unlike a hook there is no host variant and there will not be one. Consent
  is per project, keyed on a digest, so editing the recipe asks again. An
  administrator can forbid it and cannot approve it.

  **The credentials are the asset, and they are not in the repository.** DDEV
  mounts the developer's ssh agent into the container that runs the pull —
  coherent for a tool with curated recipes, and wrong here, because this
  application's rule is that a repository-declared container gets no host path
  (ADR 0023) and an ssh agent is a host path that signs things. So a recipe
  **names** what it needs and never carries it: the values come out of the
  keystore (ADR 0010), scoped per project *and* per provider — two projects
  wanting `SSH_KEY` are two credentials — and arrive as `-e NAME` with no value,
  which tells Docker to copy each from this process. No secret is ever an
  argument, so none of it is in `ps`, a shell history or a crash report.

  **With no shell there is no pipe**, so the contract is a path instead: a pull
  writes `/stackvo/dump.sql`, a push reads it, and the path is fixed rather than
  configurable. What comes back is checked with `symlink_metadata` and refused
  unless it is a regular, non-empty file — a container that can write into a
  mounted directory can write `dump.sql -> /etc/passwd`, which would turn a data
  import into an arbitrary read of the host; and an empty dump restored over a
  database is the same act as dropping it, which is exactly the shape a failed
  remote command leaves behind.

  **A pull imports nothing itself.** It produces a file and hands it to
  `db_restore`, which takes a copy of what it is about to replace — so pulling
  staging over the wrong database is recoverable for the same reason restoring
  the wrong file is. Reusing that path rather than writing a second importer is
  what makes that true rather than claimed.

  **Push is the same shape and not the same act.** DDEV's own documentation
  warns about it in the loudest terms it uses anywhere. Everything asymmetric
  here is deliberate: a recipe has to declare `push` explicitly, consent is
  granted per direction so a pull agreement cannot be spent on a send, it is
  audited where a pull is not, and there is **no scheduler** — nothing in this
  application may push on a timer.

  Two gates caught real things on the way in. `websurface_claims.rs` refused
  `project_providers` until it was declared as reaching the keystore: it hands
  back no value, and *presence is information* — "this machine holds a
  production key for this project" is not a sentence the loopback surface should
  answer either. `hint_translations.rs` refused three untranslated hints.

### Changed

- **`docs/` is one backlog and two records.** Work to be done lives in
  `docs/durum.md` and nowhere else: §2 the product side, §3 the engineering
  side, §4 the two in one order. `docs/rakip-analizi-2026-08.md` is gone — its
  findings were delivered or refused one by one and are in this file, its open
  rows moved into §2, and the two things in it that existed nowhere else (the
  defensible-territory list, and the one item the round *removed* from "fights
  not to reopen") moved with them.

  The other two stay, and could not have gone: `docs/accessibility.md` is a
  published EN 301 549 statement that `tests/accessibility-claims.spec.js`
  reads, and `docs/servis-market-mimarisi.md` is cited **by section number**
  from thirteen Rust modules, its tests and the contract (`§4.4`, `§9`,
  `Faz 2`). Deleting either would have left those citations pointing at nothing.
  Both now hand their open items to §2 and say so, so nobody goes looking for a
  backlog in a record.

  Closed items are out of §2 and §3 as well. What was done and why is in this
  file; the choices that cannot be taken back are in §6, where the code's
  `ADR 0005`-style references still find them.

### Fixed

- **`docs/durum.md` contradicted itself about Windows, and the wrong half was
  the one somebody planning the work would read.** §4 is the suggested order —
  a sentence per remaining item — and its bullet for #35 said the Windows branch
  "does not even compile here (`aws-lc-sys`'s Windows SDK)". §3's row for the
  same item, marked 🟢, records that `cargo-xwin` downloads Microsoft's SDK,
  points clang at it and removes exactly that obstacle, and that
  `tools/linux/run.sh --windows` is how it is run. Anyone starting from §4 would
  have begun at a blocker that had already been taken away — worse than an
  absent note, because it reads as a measurement.

  `tools/linux/Dockerfile` names this shape in its own opening comment: *when
  two places state one fact, the second one is the one that goes stale.* It is
  the second one that did.

  `durum_sections_agree.rs` holds them together now: every `#N` in §4 is a row
  in §3, nothing §3 marks ✅ is still being asked for in §4, and the specific
  claim that went stale is held **against the tree** — while the image installs
  `cargo-xwin` and the script wires the mode, no bullet may say the branch
  cannot be built here. Whether two paragraphs of Turkish prose agree is not
  checkable and the file says so rather than pretending, which is the line
  `platform_matrix_claims.rs` already draws around the counts it refuses to
  judge.

  §4 now says what is actually left of #35: the tests **running** on Windows.
  Type-checking is not running, and a test keeps the document distinguishing the
  two.

### Added

- **A language pack can say which way it reads.** Layout direction was one
  appearance switch applied to every locale at once. That is right for the two
  languages this app ships — both left to right — and wrong the moment a pack is
  Arabic or Farsi, which is not hypothetical: of the five languages the nearest
  competitor ships, **two are right-to-left**. A translator had no way to state
  it, so their window laid out left to right until they found a switch in
  Settings, and that switch then mirrored English as well.

  `"language": { "label": "العربية", "direction": "rtl" }`, and the pack wins for
  its own locale. Direction is a **fact about a language** — Arabic reads right
  to left whether or not anybody prefers it — while the switch is a
  **preference**, and it still decides for every locale that has not stated a
  fact. The `dir` attribute on the document now follows the active locale rather
  than the switch: they were the same value while one flag decided everything,
  and an Arabic window whose dialogs and menus laid out left to right would be
  exactly the failure that attribute was added to fix.

  The key is **seeded** into a new pack as `"direction": "ltr"`, which is what
  the app does for it anyway. It is the one key whose absence was invisible: a
  translator cannot ask for a key they have never seen, and the symptom reads as
  the app being broken rather than as a line missing from their file. Anything
  other than the two words reads as absent rather than as an error — a pack is a
  hand-edited file, and one typo in one optional key should cost the direction,
  not the language.

  The card also shows **where the pack lives**. "Adding a language is a JSON file
  you drop in the config directory" is only a mechanism somebody can use if they
  can find the file the button just made, and the path was in the data the pane
  already had and on screen nowhere.

### Fixed

- **A new language pack said it was 100% translated before a word of it was.**
  "Start a translation" seeds the file with **every English string** — which is
  what a translation file is, and is the right thing to hand a translator. The
  progress figure then counted the strings the file *held*, so an untouched pack
  reported `2000 of 2000 (100%)` the moment it was created: a progress bar full
  before the work starts, on a language that is entirely English.

  `locale.rs` states the rule this broke, in its own doc comment — *a missing
  string that falls back to English is honest; a fabricated one is a sentence
  somebody has to find and disbelieve.* Two thousand of them, with a number
  saying the job was done.

  A string counts when it **stops being the English one**, which is how every
  translation tool decides the same question and which handles both shapes of
  pack at once: a seeded file full of English and a sparse file missing keys
  both fall back to English at runtime and both read as untranslated. It
  understates — `Docker`, `PHP` and `OK` are the same word in most languages —
  and that is the safe direction: a translator who sees 98% on a finished pack
  goes looking for the last few, one who sees 100% on an untouched file learns
  that the number is a lie.

  The denominator was wrong too, in the other direction: it counted Vuetify's own
  `$vuetify` strings, which "Start a translation" deliberately leaves out of the
  file, so 89 strings sat in the total that no pack could ever fill and every
  finished translation would have stopped short of 100%.

### Changed

- **The performance number is on the row it was measured for, not averaged
  across the card.** The competitor row said the measurement never reached the
  screen. It did — in four places. What was wrong is *which* number reached it:
  the card's header sentence read "2–3× … 3.8× on a framework boot and 2.8× on
  the writes", as though both applied to everything, while the contract, the
  module and the pane's own code comment all say the opposite. `vendor` in a
  volume buys the **boot** and does nothing at all for writes;
  `storage/framework` is the one that buys the **writes**. Averaging them hides
  the exact fact the feature is shaped around — it is a list of directories
  rather than one "make it fast" switch *because* they disagree — and the row
  somebody actually toggles carried no figure at all.

  It is `perf::GAINS` now: data, per directory, with the workload named, on the
  `PerfLayer` the backend already returns. The chip on the row says "3.8×
  faster on a framework boot"; `bootstrap/cache` and `node_modules` say **not
  measured**, because they have never been through the bench and lending them a
  neighbour's number would be the average all over again.

  **The number left the translated strings, and that is the part worth keeping.**
  A measurement in `perf.explain` is a measurement a translator has to restate
  correctly with no way to know it is one — and one locale could drift from the
  benchmark without anything anywhere disagreeing. `perf_claims.rs` now fails
  the build if a figure grows back into either locale.

  The benchmark also **records what it printed**. `examples/perf_layer_bench.rs`
  produced these multiples and kept no record of them: the only written copy was
  a table in `PerfPane.vue`'s doc comment, unreachable from the program that
  made it. Its doc comment carries a `MEASURED` block now, and `perf_claims.rs`
  holds `perf::GAINS`, that block and both help documents against each other. It
  reads the sources rather than re-running anything — the bench measures the
  machine it runs on, so a test that re-measured would fail on a faster laptop,
  which is the opposite of what it is for.

  Both help documents now say **which machine** produced the table, and that on
  Linux there is no filesystem boundary to cross at all.

### Added

- **A project can be exported as a devcontainer.** The competitor row read "the
  generator already renders compose, and `.devcontainer/devcontainer.json` is a
  small sibling of it". It was written without reading
  `render_compose_service`, and what that function produces is bound to *this
  machine* in five separate ways, each load-bearing: absolute paths under the
  user's home; a `context:` relative to `generated/`, a directory no repository
  contains; the shared `stackvo-net`, created by a different compose file;
  Traefik labels in a file with no Traefik in it, routed by a certificate
  authority installed in this machine's trust stores; and the backing services
  in *another compose file entirely*, rendered with values pulled from the OS
  keystore. Copied into a repository it cannot start anywhere. So this is a
  **second rendering of the same manifest**, and the whole design was deciding
  which facts survive the trip.

  **The Dockerfile is the generator's own** for PHP, not a second renderer —
  the value of the export is that the container is the one StackVo builds, and
  a copy would drift from it in the week nobody is looking. It needs no
  adjustment either: the PHP image never copies the source, because here it
  arrives through a bind mount and there through the workspace mount. The
  runtimes are the exception, and it is `release.rs`'s asymmetry from the other
  side — a node or Python Dockerfile does `COPY . .`, installs, builds and ends
  in a `CMD`. That is a snapshot of the application, which is right for what
  StackVo runs and wrong for a container you open a terminal in: the source
  would be a stale copy of the one on screen, and the container would exit
  whenever the application did. Those get the toolchain, `sleep infinity`, and
  the install moved to `postCreateCommand` where it runs against the mount.

  **The services are their own packages' fragments**, through the same strict
  substituter the workspace uses. Not a table of image names written here: ADR
  0011 is that this application carries no service definitions, and "except in
  the exporter" is how that decision gets lost. It also could not have been
  done by hand and be right — the mapping from StackVo's `settings.ROOT_PASSWORD`
  to MySQL's `MYSQL_ROOT_PASSWORD` exists only in that package's template, and
  a compose file that starts `mysql` with no root password set does not start
  at all.

  Six variables are answered differently and they are exactly the ones that
  name this laptop: `file.*` becomes a relative path, `instance.logs` a named
  volume, a secret setting a `${DEV_…}` placeholder, `network` the implicit
  `default`, `instance.domain` localhost — and `port.*` keeps **the host number
  this workspace allocated**, deliberately, because it is the one already in the
  author's database client. Container names are kept exactly:
  `stackvo-mysql-8-4` looks absurd in a repository that has nothing to do with
  StackVo and is the only answer that works, because the project's own `.env`
  names that host.

  **Passwords leave as names.** `DEV_` is chosen rather than anything shorter:
  `template.rs` lists the eight prefixes the workspace renderer substitutes, and
  a placeholder starting with `STACKVO_` would have been eaten on the way out —
  silently, leaving a compose file in a repository with an empty password and no
  error anywhere. A test asserts the prefix is one that renderer leaves alone. A
  `.gitignore` holding `.env` is written beside them, because the file the
  reader is about to create is the file that would undo the whole arrangement.
  A rendered *config* file holding a placeholder is refused rather than written
  wrong: `${…}` is expanded by Compose in a compose file and is five literal
  characters in a `my.cnf`.

  Read before written, and never on generate. `agentctx` writes into the project
  on every generation and says in the file that `.stackvo/` is not meant to be
  committed; this is the opposite — `.devcontainer/` exists *to* be committed,
  and a file that turns up in somebody's `git status` because they pressed Start
  is a file they learn to `git checkout`.

  **Two of these were wrong until a probe was pointed at them.** The first:
  `command:` in column zero in every export for a non-PHP project. A `\`
  continuation in a Rust string literal eats the *leading whitespace* of the
  next line as well as the newline, so a four-space-indented constant came out
  with three of its four lines flush left — making `command:` a top-level
  compose key and the file one Docker refuses outright.

  Found by `examples/devcontainer_probe.rs` on its first run, against a document
  fourteen unit tests had just called correct. Those tests ask "is this string
  in that string", which cannot answer the question the export exists for:
  whether Docker accepts the file. The probe renders this machine's real
  projects against its real packages and hands each result to
  `docker compose config` — the parser that will actually read it.

  It found a second one in the same run: **Traefik labels were surviving into
  the export**, naming a host that resolves nowhere and, on a team that does run
  Traefik, quietly binding a route nobody asked for. The unit test that should
  have caught it asserted the absence of `traefik` from a fragment written *for
  that test*; the shipped packages have labels and the fixture did not. The
  fixture now carries them, copied from the phpmyadmin package — `import_probe`
  states the same trap from the other side: a parser that reads a fixture its
  own author wrote agrees with its author.

- **The projects directory is a park, and now it behaves like one.** Herd, Valet,
  Yerd and Laragon all sell the same thing: point at a folder and every child of
  it becomes a site. StackVo already had the folder — `projects_root` *is* that
  directory, and `project_adoptable` already read every child of it and said what
  each one was. Two halves of the verb were missing.

  **`project_adopt_many` adopts the whole list in one press.** Not a `for` loop
  in the UI, and the difference is measurable in three places. `generate`
  rewrites the *whole* projects scope under one global lock, so eleven adoptions
  as eleven calls are eleven full passes over eleven projects, serialised — the
  batch writes every manifest first and generates once. `sync_project_host`
  raises the system password prompt when a name is missing from `/etc/hosts`;
  its own comment argues that three subdomains must not ask three times, and
  eleven projects asking eleven times is that same argument one level up, so the
  hosts write moved into a helper that takes every name at once and the batch
  asks at most once. And a loop that stops on the first error leaves "some of
  them worked": every folder gets an outcome here.

  **Skipped is not failed.** A folder that already carries a `stackvo.json` —
  the list can be a minute old — and a folder holding nothing but dotfiles are
  ordinary outcomes, not red ones. They carry a stable `code` beside the English
  `reason`, because the UI has to say them in the reader's language and matching
  the prose in JavaScript would have turned a sentence somebody could improve
  into a wire format nobody could see was one. Everything else fails that folder
  alone. If the *generator* fails, every manifest the call wrote is removed: the
  state it would otherwise leave — projects listed and none of them generated —
  is exactly what `project_register` had to be invented to repair, and somebody
  who pressed one button should not have to know that.

  **`folder:appeared` makes the count live.** Clone a repository into the parked
  folder and the badge goes up on its own; the answer to "why is my new site not
  listed" used to be "reopen this dialog", which is the part a park exists to
  remove. The watcher was already receiving those events and dropping them — its
  manifest reader accepts `<projects>/<name>/stackvo.json` and nothing else,
  which is right for its own question and misses every `git clone`. A folder is
  announced on becoming *new to the watcher* rather than on a timer: a clone
  writes for as long as the repository takes and every second of it is a create
  event, so a window would either repeat the same folder a dozen times or be
  long enough to miss the next one. A removed folder is forgotten, so cloning
  over the same name announces it again.

  The assumption under all of it is driven against the real thing: a test starts
  a platform watcher, creates a directory it has never seen and asserts that a
  path *inside* that directory arrives. On macOS that is FSEvents, and a
  coalescing backend reporting only the parent would have made the whole feature
  silent on the platform it was written on — with every other test still
  passing.

- **A restore takes a copy of what it is about to replace.** DevTent sells
  "backup before shutdown"; copying that literally would have been theatre here,
  because `compose_down` leaves the volumes alone and `instance_remove` is
  documented as leaving them alone (ADR 0012) — nothing is lost when this stack
  goes down. Measured against the paths that actually can lose data, the answer
  was a different one: **restore** is the single operation here with nothing
  behind it. The rows that were there are gone the moment it succeeds.

  Both restore paths take one first — `db_restore` from a file the user picked,
  and `db_snapshot_restore`, which is the one people actually use. Leaving it
  off the second would have put the safety on the rarer of the two. The snapshot
  path takes it _after_ the named file is known to exist: a copy taken and then
  a failure because the thing being restored was never there is a net for
  nothing.

  **It gets its own reserved prefix, `before-restore-`, not the scheduler's
  `auto-`,** and two things would have broken otherwise. `last_automatic`
  matches on `auto-`, so a restore would have pushed the next scheduled backup
  out by a full interval — hours or a day of no backups, with nothing on screen
  connecting the two. And `expired` prunes automatic copies against one
  retention window, so a run of restores would have evicted the scheduled ones
  that window exists to keep. Both are held by tests, one of them against a real
  directory because `last_automatic` reads one.

  Its window is **the last 3 per service, pruned when one is taken** rather than
  by the scheduler. Not a style choice: the schedule defaults to `off`, so a net
  that only tidied up when a feature nobody switched on runs is a disk that
  fills. Three, because "I restored the wrong file" is realised in minutes.

  **A failure to take it stops the restore** — the caller asked for a net, so
  doing the irreversible thing without one would answer a different question.
  That would trap one person, though: a database too broken to dump is exactly
  the one somebody is restoring over. So it is a flag, and the UI asks the
  second question **only when the copy actually fails**, which is where it
  belongs — nobody should have to answer it on the way past a working one.

  One duplication went with it. The reserved-prefix character check existed
  twice, in `snapshot.rs` and in `commands.rs`, and only one of the two would
  have learned about a second prefix. There is one `reserved_checked` now, and
  adding the prefix is what found the copy.

### Fixed

- **A test could hang the whole suite on a password dialog.**
  `hosts::apply` falls back to an elevated copy when it cannot write the file,
  and on macOS that is `osascript`'s administrator prompt — a window behind
  every other window with nobody to answer it. `cargo test` sat at 0% CPU until
  it was killed. It is §3 #37's failure one seam over: a hanging suite looks
  like a slow one.

  The comment above `write_in_place` already claimed the unelevated branch "is
  the one that runs, on every platform, without a prompt nobody could answer in
  CI". It was a claim and nothing held it. Elevation is now **refused outright
  while `STACKVO_HOSTS_PATH` is set**, with an error naming the file — which is
  the honest answer as well as the safe one, because the seam exists precisely
  to point this at a file we may write, and being unable to write it is a broken
  test rather than a reason to ask a human for a password.

  `tests/hosts_no_prompt.rs` holds it, on a real read-only file rather than a
  stub, and in a binary of its own: the seam is an environment variable, so a
  test that sets it needs the process to itself. Putting it beside
  `hosts_roundtrip` made both fail, which is the cheapest possible demonstration
  of the rule that file already states at the top.

- **Declared containers have a screen.** `sidecars` — a repository declaring a
  container of its own (ADR 0023) — was parsed, validated, refused when it asked
  for the host, and rendered into the project's compose file. Nothing showed it.
  `hooks`, the sibling block in the same manifest, has had a pane since it was
  written.

  That gap is what made ADR 0027's answer hollow. It rejects Ollama and Qdrant
  as catalogue services and says "somebody who wants either writes a `sidecars`
  block" — a true sentence about a feature nobody reading the app could find out
  existed, which is the same as not having it. The decision is unchanged; what
  changed is that the alternative it names is now reachable.

  `project_sidecars` is a command rather than a field read off the manifest the
  view already has, and the two derived names are why: `container` is the
  hostname the application connects to and `volume` is what `docker volume ls`
  calls the state. Deriving them in JavaScript would be a second copy of a
  naming rule whose entire purpose is that two clones of one repository cannot
  collide.

  The pane leads with the hostname, because it is the only thing a reader cannot
  work out and the only thing they have to put in their own config. It says once,
  at the bottom, that a declared container has no host port and no host path —
  as a reason rather than a limitation to work around. A project that declares
  none gets no card at all.

  One gate was missing under all of it and is not tautological: **the sidecar
  and the project must be on the same network**, which is what makes the
  container name a hostname the application can resolve. The pane's headline
  sentence is true only while that holds, and nothing in the generator's tests
  would have noticed it stopping. Confirmed by changing the network and watching
  it fall over.

  Help in both languages, and a worked example: what to declare, why `image`
  needs a tag, why `env` is not for secrets, and when to write one instead of
  asking for a package.

- **Laravel Reverb runs as a worker, and is the first one a browser can
  reach.** Herd Pro sells Reverb as a service and EnvKit advertises "proxies
  WebSocket while keeping trusted HTTPS routing"; this had neither.

  **It is not a service, and the competitive review's estimate was wrong about
  that.** Reverb has never been an image — it is `php artisan reverb:start`
  inside the application — so the service catalogue was the wrong place and
  `worker.rs` was the right one. It joins queue, scheduler and Horizon as a
  fourth kind, detected the same way Horizon is: `laravel/reverb` in
  `composer.json`'s `require`. That read now excludes `require-dev` for both,
  because a package there is a tool for the test suite rather than a process to
  run beside the site — and Horizon under `require-dev` while somebody is
  evaluating it is a real thing people do.

  **What makes it different is that a browser has to open a socket to it, and a
  published host port cannot do that.** This app serves projects over HTTPS, and
  no browser will open `ws://localhost:8080` from an `https://` page — it is
  blocked as mixed content. So it is routed, and routed on the project's **own
  domain under Reverb's own path prefixes** rather than at a hostname of its
  own. That choice costs nothing and buys everything: no certificate to extend,
  no hosts entry to write, no `*.` alias to require, and
  `wss://shop.loc/app/<key>` is same-origin with a certificate the browser
  already trusts. `/app` and `/apps` are fixed by the Pusher protocol, not
  chosen — they are what Laravel's own deployment notes put in front of an
  nginx.

  Two numbers are held together by tests because they are written twice and
  their coming apart is silent: the `--port=8080` Reverb is started with and the
  Traefik service port it is routed to. A mismatch is a socket that connects to
  Traefik and closes, which reads as a Reverb bug. `--host=0.0.0.0` is likewise
  asserted — Reverb's own default binds loopback, and inside a container that
  means Traefik one hop away cannot reach it.

  The router priority is **set rather than inherited**. Traefik orders routers
  by rule length by default and this rule is longer than the project's bare
  `Host()`, so it would usually win; usually is what the number replaces. And a
  project with no domain is refused rather than routed, because the alternative
  is a `Host(``)` rule that loads and matches nothing.

### Changed

- **Ollama and Qdrant stay out, and the softest half of the reason is now the
  hardest.** ADR 0027 rejected both with "wants a GPU it may not find", written
  as a risk. On the platform most of these users are on it is a certainty:
  Docker Desktop on macOS cannot pass the Apple GPU into a container at all —
  Apple's virtualisation framework exposes no GPU API for it — so a
  containerised Ollama on Apple Silicon is CPU-only and runs **3–5× slower**
  than the native application it would replace. That held from M1 through the M5
  line, and Ollama's own answer for macOS is "run it natively". The package
  people ask for would be measurably worse than doing nothing, with no way to
  fix it from here. Recorded in `vector_capability.rs`, where the decision
  already lives.

- **Twelve more programs can be run in the project's container.** DDEV has
  fifteen of these rows; this had four, which made "run it in the container"
  read as a PHP feature. The new ones are `wp`, `console`, `rails`, `bundle`,
  `yarn`, `pnpm`, `python`, `ruby`, `go`, `cargo`, `bun` and `deno`.

  **The rule for every row is that this app already declares the program**, and
  that is what keeps this from being breadth for its own sake. Three sources and
  nothing outside them: `quickcmd::CATALOGUE`, which records what each
  framework's container actually runs and was verified against real images when
  those rows were written; `manifest::LANG_RUNTIMES`, the runtimes this app
  generates a container for; and `manifest::NODE_PACKAGE_MANAGERS`, the three
  Corepack can pin.

  The runtimes were the real gap. `php` and `node` had a row and the six others
  did not, so a project this app can build had no way to open a `python -V` in
  it. Checked before it was written rather than assumed: `generator.rs` builds
  each of them in **one** stage — `FROM golang:1.23`, `FROM rust:1` — so the
  toolchain is still in the running container and `stackvo cargo test` reaches a
  cargo that exists.

  **The rule is a test, not a comment.** `cli_surface.rs` now fails the build if
  a runtime in `LANG_RUNTIMES` or a manager in `NODE_PACKAGE_MANAGERS` has no
  way to be run, and if two rows run the same program. Confirmed by deleting the
  `deno` row and watching it fall over, rather than by reading it.

  `wp` carries `--allow-root` in its prefix for the same reason `quickcmd`'s two
  wp rows do: the container runs as root and wp-cli refuses outright without it,
  so every call would fail. wp-cli takes a global flag anywhere on the line,
  which is what makes putting it in the prefix safe.

  **`drush` is deliberately absent.** `detect.rs` recognises `drupal/core`, but
  nothing in this app says how Drupal is driven — no catalogue row, no generator
  step — so a `drush` row would be inventing a path and finding out from a bug
  report. It is one `stackvo exec drush` away.

  All twelve are tab-completable with no further work, which is the payoff of
  generating the completion from the same table.

- **A program that is not in the container says which project it is not in.**
  Twelve new rows make `stackvo python -V` in a PHP project a mistake somebody
  will make. Docker's own message is accurate — `"python": executable file not
found in $PATH` — and is left exactly as it arrived; one line is added after
  it with the fact Docker cannot know. Only on exit 127, only for a command that
  names a fixed program, and never under `--quiet`, because that is narration.
  The exit code is still passed straight through.

### Fixed

- **A test asserted a property of the author's machine.**
  `tooling::resolve_falls_back_to_the_bare_name` read the real
  application-support directory and asserted the fallback branch on the
  reasoning that "nothing is installed in a test run" — which stopped being true
  the moment somebody installed mkcert with the app they were building, and then
  a full suite failed on one machine and passed on every other. It is the same
  flaw as a test waiting on the real keychain (§3 #37), one turn quieter: it
  does not hang, it accuses the wrong change. `resolve` now has a pure
  `resolve_in(dir, program)` under it and the test owns its directory, so
  **both** branches are covered where only one was.

- **Tab completion, in all four shells, generated from the command table.**
  DDEV ships completions and `dde` installs them as part of `system:install`;
  this had none. The reason it needed a module rather than four files is that a
  hand-written completion script is a **second copy of the command list**, in a
  language no test reads, that silently stops matching the first. `cli.rs`
  already refuses to let the CLI drift from `contracts/ipc.json`; letting it
  drift from a `.bash` file instead would be the same mistake with the gate
  removed.

  **The shell side is four lines and knows nothing.** It collects what has been
  typed and asks `stackvo complete --word <partial> -- <the words before it>`,
  then prints what comes back. `completions::candidates` is the whole of the
  logic, it is pure, and it is tested. Adding a shell is a stub; adding a
  command is nothing at all. The current word is passed **separately** because
  every shell disagrees about whether the word under the cursor is in the word
  list — bash puts an empty string there, fish does not — so inferring it from
  the last element would behave differently in each.

  It completes commands, flags — global and the command's own — and the
  positionals whose placeholder already names a list this app keeps:
  `<project>` from the workspace, `<client>`, `<target>`, `<tool>`, `[shell]`,
  and a literal `on|off`. Everything else yields nothing, which is not a
  failure: the stubs leave the shell's own file completion on, so a `<path>`
  falls through to filenames. Service containers are **not** offered for
  `logs <container>`, and that cap is named rather than silent — listing them
  needs the engine, and a completion that waits on Docker is a shell that hangs
  when Docker is down, which is exactly when somebody types `stackvo logs`.

  **`stackvo path-install` writes it into the same marked block as the `PATH`
  line**, after it rather than before — a completion registered for a command
  that is not yet on `PATH` does not fail, it simply never fires. One region,
  so one `path-remove` takes both back out. `stackvo completions <shell>`
  prints a stub on its own for a package manager.

  Two things had to be right that no unit test could see.

  **The zsh stub is guarded on `compdef` existing.** `compdef` comes from
  `compinit`, which many people never run and oh-my-zsh runs from the middle of
  their file — and `merge` appends our block. Unguarded, it prints
  `command not found` into every new terminal: the exact failure this module
  exists to prevent, delivered by the fix for it.

  **The bash stub builds the word list before it narrows `IFS`.** The obvious
  spelling puts `local IFS=$'\n'` at the top and expands
  `"${COMP_WORDS[@]:1:COMP_CWORD-1}"` inside the command substitution — and on
  **bash 3.2, the bash macOS ships**, that collapses the slice into a single
  argument. `artisan migrate` arrived as one word, no command was recognised in
  it, and `stackvo artisan migrate --<TAB>` offered this binary's own global
  flags. Every Rust test passed the whole time, because they call `candidates`
  directly and it was right. `examples/completion_probe.rs` is what found it and
  is what keeps it found: it writes each stub, sources it in a real bash and a
  real zsh, sets the variables the line editor would have set, and reads back
  what the function put in the reply array. Reintroducing the bug fails 2 of its
  14 checks — including one symptom nobody had noticed by hand, `xdebug shop
<TAB>` answering with the entire command list.

  The probe prints **which binary it read and whether it is older than
  `completions.rs`**, because the first run of it reported every check green
  against the very bug it was written for: `cargo run --example` builds the
  example and not the `stackvo` bin.

  `Backing::Local` is new and is the second exception to "every CLI command
  names a contract command". These two answer from the table in `cli.rs` rather
  than from the stack, so naming one would be an invention, and `Surface` means
  "a screen over several", which they are not. It is held to a boundary by
  `cli_surface.rs` exactly as the container commands are: a `Local` command
  reaches no contract command and never writes. They also get their own
  `--help` heading — listing `complete`, which no person ever types, between
  `doctor` and `logs` would put noise in the one list people read before typing.

- **Herd and DDEV can be imported.** The importer read five rivals and neither
  of the two that matter most: Herd is the paid leader of this category, and
  DDEV's project file is the most machine-readable thing any of them writes.
  Both were added, and neither needed a reader of its own.

  **Herd is Valet's shape with another root.** That is the finding rather than
  an assumption — Herd is built on Valet and keeps the same `config.json` with
  the same `paths` and `tld`, the same `Sites/` directory of symlinks and a
  `Nginx/` directory of per-site configs, all under `~/Library/Application
Support/Herd/config/valet`. So the reader that already existed was pointed at
  another directory instead of a second one being written, and `scan_valet`
  became `scan_parked`.

  Two things are Herd's and not Valet's, and both are read. **`~/Herd` is parked
  whether or not the config says so** — Herd parks it on install and does not
  write it into `paths`, so a reader that trusted the config alone would miss
  the directory most Herd users keep everything in. And **the PHP version is
  written down**: Herd runs a pool per version and points each site's
  `fastcgi_pass` at that pool's socket, which makes Herd the only source of the
  seven whose sites arrive with a version somebody _chose_ rather than one
  inferred from a `composer.json` constraint. `^8.1` is what a framework needs;
  `8.3` is what the site was being served with.

  Reading digits out of a socket name needed two guards and both are earned.
  The line has to name a **`.sock`**, because `fastcgi_pass 127.0.0.1:9000` also
  has digits in it and reading those gives `1.270`. And the run has to be
  **exactly two digits**, which is every PHP version there has ever been a build
  of. Failing to match costs an override that does not happen, not a version
  that is wrong.

  **DDEV is a fourth shape: a registry plus a declaration.** `~/.ddev` lists
  every project's `approot`, and each of those holds a `.ddev/config.yaml` that
  states the PHP version, the document root, the web server, the database engine
  and the extra hostnames. Every other source here makes this app _infer_ those
  from the code; DDEV's file declares them, so an imported DDEV project is the
  one case where nothing is guessed. The declaration wins over detection and
  only where it speaks — gated on the detected runtime, so a `php_version`
  cannot be laid over a project detection called Node (W-02) — and detection
  keeps what it reads from the code, which is the framework.

  The registry is read **by its leaf key**, `approot`, from wherever it appears.
  That is what makes it survive being reorganised: the list lives in
  `global_config.yaml` under `project_info`, DDEV has an open proposal to move it
  to its own `project_list.yaml`, and both files nest the same key under the
  project name. Both are read, because a machine mid-upgrade has both.

  The trap in the config file is not a detail. **DDEV writes its whole annotated
  template into every project**, so the file carries `#php_version: "8.4"` — no
  space after the hash — as an example. A scanner that took the first line
  mentioning the key would read the example instead of the answer, on every DDEV
  project in existence. There is no YAML crate in this tree to reach for
  (`serde_yaml` is archived and `deny.toml` fails the build on an unmaintained
  direct dependency), so this is a line scan for the third time in the module,
  with the same reasoning `sail_services` wrote down.

  **Two gaps closed while they were visible.** `Source::from_id` kept its own
  copy of the source list, which is how a source can be readable by `scan_at`
  and refused by the command that calls it — the state three of the five were in
  once. There is now one list, `imports::ALL`, and everything walks it. And
  nothing held the front end's `IMPORT_SOURCES` against the backend's:
  `foreign_import.rs` now checks both directions, because an id the backend
  refuses is a button that errors and a source with no id is a tool nobody can
  point at.

  One existing test asserted `from_id("herd") == None`. That is the wrong shape
  for a claim — it froze one absence rather than checking the list, so it would
  have kept passing while the list grew. It iterates `ALL` now, which is the
  check its name always described.

- **`stackvo` can be put on your PATH, from the app.** Every rival ships a
  Tooling page; the measured one here is Yerd's, which fetches `composer`,
  `node`, `bun`, the Laravel installer and `wp-cli` onto the host and shims them
  onto `PATH`. Half of that page has no place here and half of it was missing
  entirely — and the missing half was the load-bearing one.

  **Not copied:** the tool downloads. Those five run in the project's container
  at the version the project declared; `stackvo composer install` and the quick
  commands already reach them. A host copy would be a second answer to "which
  composer runs" and it would be the wrong one — it knows nothing about the
  project's PHP. `cli.rs` had already written that argument down about `php`.

  **What was missing:** `stackvo` and `stackvo-mcp` are programs this repository
  builds, the README documents them, `agents_install` registers one of them with
  six assistants — and nothing anywhere put either where a shell would find it.
  The instruction was "build it and remember the path", which is not something
  you can tell the person who downloaded a `.dmg`. **Both ship inside the app
  now** — `externalBin`, built by `tools/sidecars.mjs` under the target-triple
  name the bundler looks for, landing beside the main binary in
  `Contents/MacOS/` (measured on a real bundle, not assumed: that is exactly
  where `agents::binary` and `tooling::shipped` already looked, so neither
  needed a line changed). The app is 27 MB instead of 16, and the `.dmg` 12 MB.

  Declaring `externalBin` has a cost that only shows up when you run it:
  `tauri-build` checks the files exist on **every** cargo build of the package,
  including the one that produces the sidecars, because they are `[[bin]]`
  targets of the crate that carries the build script. Building them requires
  them. `tools/sidecars.mjs` writes a text placeholder first and copies the real
  binary over it, and `beforeBuildCommand` runs `--verify`, so no `tauri build`
  on any path can bundle one. The placeholder is a script that exits 1 rather
  than an empty file: if one ever escaped it fails loudly instead of looking
  like a truncated download.

  Settings → Tooling links
  both into a directory the app owns and writes one line into one shell's
  startup file: zsh, bash (`.bash_profile` on macOS, `.bashrc` elsewhere), fish
  or PowerShell. Between markers, after a backup, leaving every other byte alone
  — `rules.rs`'s rules, for the same reason. `stackvo path-install [shell]`,
  `path-remove` and `tools` are the same thing from a terminal.

  The line quotes the directory and puts `$PATH` **inside** the quotes: the
  default on macOS is `~/Library/Application Support/StackVo/bin`, and
  `export PATH="/a b":$PATH` is valid and wrong — the unquoted expansion is
  word-split, so a `PATH` that already holds a space reaches `export` as several
  arguments.

- **mkcert can be installed by the app, against a digest it was built with.**
  It was the one host requirement this app could report and never obtain:
  without it the stack runs and every browser warns. It is also the only tool in
  the catalogue where fetching is the right answer rather than a second copy of
  `docker pull` — one static binary its author publishes.

  The SHA-256 is **compiled into the build**, one per platform, and is not
  fetched: a checksum served beside the file it describes is not a check,
  because whoever can replace one can replace the other. Nothing is written
  until the bytes match. There is deliberately no update verb — an idempotent
  install that follows upstream is how a pin stops being a pin.

  The requirements gate offers it too, and that is where it matters most: the
  gate's own comment says a row that reports a problem and offers nothing to do
  about it is worse than absent, and named the mkcert row as the one that had
  been like that since it was added.

- **Recording a profile no longer needs a browser.** php-spx's control panel is
  the documented way in and it needs a person: a page opened at the site's own
  address, a checkbox, a cookie. Everything this app could offer stopped at
  opening that page. Two doors are open now and neither goes through a browser.

  **A request** is recorded by sending it with the profiler's cookie on it,
  which is php-spx's own documented trigger — its README profiles a page with
  `curl --cookie "SPX_ENABLED=1; SPX_KEY=…"`. Type a path, get a profile.
  **A command** is recorded by running it with `SPX_REPORT=full` in its
  environment, through the same fixed catalogue the quick-command buttons use,
  so the frontend still names an _id_ and never a program. Both land in the same
  list. `stackvo spx-record <project> [path]` is the same thing from a terminal.

  The host is always the project's own domain, from its manifest. What crosses
  is the **path**, and one opening `//` is refused: that is a protocol-relative
  URL, and accepting it would have made a text field on a pane a way of sending
  this app, with a credential attached, to somebody else's host. Redirects are
  not followed either — a framework answering `/` with `/login` would otherwise
  write two recordings for one button and show the wrong one.

  Lerd reaches the browser case from its own window by injecting the cookie in
  the web server's configuration. That is deliberately not done here: the server
  config is generated under a byte-for-byte contract with the Bash CLI, and
  reaching into it to hold a piece of UI state would put this app inside a file
  another program owns.

- **Where the time went, without leaving the app.** A report row could say a
  request took 900 ms and nothing about which function held it; that answer was
  in the trace half of the pair, readable only in SPX's own web UI. It is read
  here now — `spx_report`, the `stackvo_hotspots` tool, `stackvo spx-top` — as
  the functions that held the run, ranked, with the share each held in its own
  body and the share it held including everything it called.

  The format was established by recording against a real image, and two
  properties of it decide the whole implementation: the metric values are
  **cumulative totals**, so time is attributed by the gap between consecutive
  events to whatever was on top of the stack, and the function table is written
  **after** the events that index into it, so names are applied after the replay
  rather than during it. Recursion is counted once — a function that calls
  itself adds its inclusive time only when its outermost frame leaves, or it
  reports having held 166% of the run. A very long trace is replayed up to a
  limit and **says so** rather than presenting its first half as the whole.

  There is also a **view** button per row now, which opens that recording in
  SPX's own viewer rather than its index. The flame graph and the call tree are
  that project's work and there was no reason to rebuild them; what was missing
  was a way to reach _this_ report.

- **A sampling period, so the pane's own first sentence is true.** php-spx's
  default period is `0` — every call — which makes it a tracing profiler with
  the cost this whole feature exists to avoid. Recordings started from StackVo
  now sample every 100 µs by default, and "every call" is still one choice away
  for counting a fast function exactly. Built-in functions are an option beside
  it.

  Measured, and not where it looked like it should go: php-spx has
  `spx.http_profiling_sampling_period` and siblings that read exactly like a
  place to put these, and they are never consulted for a recording. Its
  `PHP_RINIT` reads the ini source **only when access was not granted** — a
  request carrying no key, which is a request it is not profiling. Wiring the
  pane to them would have produced controls that appeared to work and did
  nothing. They ride in the request and in the environment instead.

### Fixed

- **A recorded run's time was shown a thousand times too long.** php-spx's
  metadata calls the field `wall_time_ms` and it holds **microseconds**, so a
  183 ms request was listed as "182837 ms". Measured rather than reasoned about:
  a script written to burn a known 180 ms produced `"wall_time_ms": 182837`
  while the trace's own cumulative total for the same run said `182837191`,
  which is nanoseconds. The value now crosses as `wallTimeUs` and is formatted
  by one function per surface, because a number that reads `736 µs` in the
  window and `0 ms` in the terminal is the same bug twice.

- **The profiler tool's redaction was tested against a copy of itself.** The
  test that proved no key leaves the assistant surface re-implemented the
  removal instead of calling it. It agreed with the code and neither matched
  what ran — so when a second field carrying the key was added, the test would
  have kept passing while the field walked straight out. The redaction is one
  function now and the test goes through it.

- **The profiler and the IDE setup reach the surfaces that get asked about
  them.** `stackvo_profiler` and `stackvo spx` answer "why is _this page_ slow"
  — the sampling profiler's three states and everything it has recorded — and
  the AI rules name both it and `stackvo_ide_debug` in the table that says
  which tool answers which question, which a test keeps honest by checking that
  every tool the rules name exists. `stackvo spx-build` compiles the extension
  from a terminal, the same throwaway-container build the pane runs.

  **The tool redacts the control URL**, and that is the part worth writing down:
  the URL carries the profiler's key, a key is a credential however cheap it is,
  and this surface returns none — the loopback HTTP surface serves whatever the
  tool dispatch returns. The pane keeps the URL because a person is going to
  click it; a model cannot open a browser, so it is told where to find it
  instead. Thirty-three tools now, sixteen of them served over loopback.

  The CLI renderer also said "not mounted — recreate it" for a project with the
  profiler switched **off**, which is telling somebody to recreate a container
  to apply a setting they never asked for. The pane had always asked the second
  question; the renderer had not.

- **php-spx: the profiler you can leave on** (§3, the profiler). Herd and Lerd
  both ship it and both sell it on one property — it samples, so it can be left
  on during a real page load, where Xdebug's profiler costs several times the
  request. That is not a nicer version of what this app had; it is the case
  Xdebug's profiler cannot cover, because you cannot browse a site under it.

  `profile.rs` had ruled it out, and the reasoning was right about the contract
  and wrong about the conclusion: it assumed the only way to get an extension
  into a container is to put it in the manifest. `php-extensions.json` is the
  data half of the Bash generator's own install matrix, so adding `spx` would
  claim the Bash CLI knows how to install something it has never heard of — and
  it could not be honoured anyway, because **SPX is not on PECL** and the
  contract's `special` install method is documented as v1-MUST-REJECT.

  So the extension never enters the manifest, the Dockerfile or that contract.
  It is installed the way the debug bridge is: compiled into a directory this
  app owns, mounted, and switched on by an ini in `conf.d`. The build runs in a
  **throwaway container of the project's own image**, because an extension has
  to match the ABI of the php-fpm that loads it — and not against the running
  container, which would mean `apt-get install` inside somebody's live php-fpm.
  Output is keyed by PHP version, so every project on 8.4 shares one build.

  Four things in this module were measured rather than read, and three
  contradict what the documentation implies. **It is `extension=`, not
  `zend_extension=`** — loading it the other way fails outright, which is the
  error the first version of this was written against. A report is a **pair**
  of files, `<key>.json` and `<key>.txt.gz`, and the JSON carries wall time,
  peak memory, call counts and the request — enough that the list here needs
  none of SPX's own UI to say what was recorded. And `spx_utils_ip_match`
  accepts `*` and IPv4 CIDR and nothing else, which is what decides the
  whitelist: behind this stack's own proxy the address SPX sees is the proxy's
  container address, so the private ranges are what have to be allowed and `*`
  is not needed to do it.

  The whole path was driven for real before it shipped: the module's own build
  script, run against this repository's project image, produced a loadable
  `spx.so` and the web UI assets; the ini it renders, mounted at the paths the
  overlay uses, loads the extension, applies every setting and records a report
  into the host-visible directory.

  Recording itself is SPX's own control panel, served by the extension from
  inside the project's vhost — no port to publish and no second server. The
  pane warns when Xdebug is recording too: two profilers hooking one engine is
  unsupported by both projects and the symptom is wrong numbers rather than an
  error, so it is said rather than prevented — which one to turn off is not
  this app's decision.

- **Two Xdebug modes that were missing, and one of them is not a mode.**
  `coverage` is the fourth: DDEV exposes it, this did not, and without it
  `--coverage-html` produces an empty report and a warning most people never
  read. It is the only mode that records nothing of its own — PHPUnit writes
  the report — so it mounts no ini, claims no recording directory, and the pane
  says so instead of leaving somebody watching an empty list.

  `develop` is the one that is **not** a mode, and modelling it as one would
  have been the mistake worth avoiding: `xdebug.mode` is a _list_, and `develop`
  is what makes `var_dump` readable and puts a stack trace on a warning. Herd's
  own documented configuration is `debug,develop` — so a fifth radio button
  would have made "step debugging with readable dumps" unreachable. It is a
  switch beside the picker, `XDEBUG_MODE` becomes `debug,develop`, and moving
  either control leaves the other exactly as it was.

  That list is also why the front end's mismatch check had to change. It
  compared the container's `XDEBUG_MODE` against the configured _mode_, so a
  project with `develop` on ran `debug,develop` against a picker still reading
  `debug` — a "recreate the container" warning for a container that was already
  correct. It compares the rendered value now, which the backend computes so the
  screen and the overlay cannot disagree about what was applied.

- **`stackvo ide` and `stackvo_ide_debug`.** The IDE setup arrived on the
  Settings screen and nowhere else, which left the two surfaces that are asked
  the question unable to answer it: an assistant asked "why is my breakpoint not
  hit" had no tool for it, and neither had the terminal. Both now read the same
  status — the port, the mapping, each IDE's state, and whether anything is
  listening. `stackvo ide-install <project> <ide>` does the same write the pane
  does, audited the same way, because the trail's question is "did something
  write into a repository" and it must not have a different answer depending on
  which surface did it.

- **The IDE setup for step debugging is filled in, not described.** Every
  local-environment tool's step-debugging page is the same page — here is the
  port, here is the host, here is the path mapping, now type them into your IDE
  — and DDEV, Laradock, ServBay and Herd all then name **the path mapping** as
  the usual reason a breakpoint never hits. `xdebug.rs` already computed both
  halves of that mapping and this app had been printing them on screen as two
  strings to copy into a dialog by hand.

  `ide.rs` writes them. VS Code gets a `Listen for StackVo: <project>` entry in
  the project's `.vscode/launch.json`, mapping written remote-to-local with
  `${workspaceFolder}` on the local side rather than this machine's path —
  `launch.json` is committed by roughly everybody, and an absolute path in it is
  a configuration that works for exactly one person. The entry is replaced where
  it stands, because that list is the IDE's dropdown and its order is somebody's
  preference, and everything else in the file comes back unchanged with a
  `.stackvo-backup` beside it. A `launch.json` with comments in it — which is
  what VS Code itself creates — is reported rather than rewritten, the same
  decision `agents.rs` made about the same editor's `mcp.json`.

  **PhpStorm is deliberately not written.** Its equivalent lives in
  `.idea/php.xml` and `.idea/workspace.xml`, which the IDE holds in memory and
  rewrites on exit, so an edit made underneath a running PhpStorm is an edit
  PhpStorm overwrites — leaving a tool that says it configured something and an
  IDE that disagrees. Its server entry, with the name and both roots already
  filled in, is offered to paste. Refusing to write is not a smaller feature
  than writing badly.

  And the half that is in no file: **is anything listening?** An IDE that is not
  listening is the other reason a breakpoint never hits, and nothing in an IDE
  says so out loud — DDEV is the only one of the five with a tool for it, and
  it is a separate command. This reads the operating system's own table of
  listening sockets, the one `doctor` already uses to say who holds port 80, and
  names the process holding 9003 or says nothing does. A read, never a
  connection: dialling a DBGp port to see whether anything answers would appear
  in the user's IDE as a debug session that immediately dropped, which is noise
  this app has no business generating on somebody's screen.

- **The MCP surface goes from 17 tools to 31, and service control arrives on
  it.** The gap against the five rivals with an MCP server was not the count.
  ServBay exposes service start/stop/restart, system metrics, hosts and domains,
  packages and backups; FlyEnv and EnvKit both expose service control as the
  first thing they mention. This one exposed neither service control nor a
  single metric, and the reason was structural: `instance_start` and its pair
  took an `AppHandle`, so a stdio subprocess could not call them.

  `progress::Null` is what let them off the window — the same split that made
  `stack_up` reachable a release ago — so `stackvo_service_start`,
  `_stop` and `_restart` now drive the exact function the window drives, with
  the events dropped. They take an **instance** id rather than a service name:
  a workspace running MySQL 8.0 and 8.4 has two answers to "restart MySQL", and
  a tool that took the service name would work on the machine that has one and
  be a coin toss on the machine that made instancing worth building.

  The reads that came with it are the ones a question actually needs and this
  surface could not answer: `stackvo_system` (host CPU, memory, disks, network,
  the engine's totals, and which stack member holds the image bytes — sampled
  twice because a single reading has no CPU delta), `stackvo_container_stats`,
  `stackvo_hosts`, `stackvo_log_read` (the other half of `log_files`: that one
  says which file changed a minute ago, this one reads it),
  `stackvo_service_instances`, `stackvo_service_connection`,
  `stackvo_packages`, `stackvo_snapshots` and `stackvo_mail_message`. Plus
  `stackvo_project_restart` and `stackvo_snapshot_take` behind
  `--allow-writes`.

  **Restoring a snapshot is deliberately not a tool.** Taking one is: it adds a
  file, changes nothing, and is the call to make before asking for a migration.
  Putting data back over live rows is a decision for the app's own confirmation.
  And no tool returns a credential — `service_connection` is hard-coded to the
  unrevealed form, and a test now asserts that no schema on this surface has a
  `reveal`, `password`, `secret` or `token` property, because the way that comes
  back is somebody adding the parameter for symmetry with the IPC command.

  Every one of the fourteen goes through the same three cross-checks the table
  already had: it names a real `contracts/ipc.json` command, a read-only tool
  cannot be backed by a declared mutation, and a write-gated tool cannot be
  backed by a mere query.

- **The server negotiates its protocol revision instead of asserting one.**
  `initialize` answered with the constant `2024-11-05` whatever the client
  asked for. The spec's rule is to echo the client's revision when it can be
  supported, and a client that gets a different one back is entitled to hang up
  — which reads to the user as "the server does not work", with the reason in a
  log they never see. It now speaks `2025-06-18`, `2025-03-26` and
  `2024-11-05`, answers with the one it was asked for, and falls back to its
  own only for a revision it does not know.

- **The loopback surface intersects over a tool's whole reach, not its
  headline command** (§34). `websurface::tools()` asked `exposable` about the
  one command a tool names, and that is the whole answer only while a tool
  reads nothing else — which several do, correctly: `stackvo_project` reads the
  certificate and the PHP limits along with the manifest because that is the
  answer somebody wanted. Undeclared, that made the check a gap the width of
  whatever the dispatch touched.

  `stackvo_log_read` walked through it. It names `app_logs`, a `query` that
  lists files, and returns the tail of one — which is `app_log_open`, a
  `mutation`. Container logs were kept off that surface by their `stream` kind
  and application log _contents_ would have been served beside them. Each tool
  now declares the other commands it reaches in `mcp::Tool::also`, the surface
  intersects over all of them, and a test names the regression rather than
  implying it. Fourteen of the thirty-one tools are served, and `log_read` is
  not one of them.

- **`tunnel_providers` joins the keystore denial list.** It arrived with the
  eight providers, hands back a `hasToken` boolean and no token, and computes
  that boolean by calling `secrets::read` — which is exactly the argument this
  list already refuses to accept from `service_db_clients`. Named for what is
  proved rather than for what is suspected: the fixpoint in
  `websurface_claims.rs` found it, nothing else had.

- **Four tool calls that answered a mistake with a fact.** The names on this
  surface come from a model rather than from a list somebody clicked, so a
  misspelling is a case rather than an edge case — and four readers underneath
  answered one with an empty result, which reads as a fact about the subject
  instead of about the name. `stackvo_log_read` on a project that does not
  exist returned no files, which is what a project that has never logged
  anything returns; `stackvo_service_connection` returned `null`, which is what
  a real service with no connection string returns; `stackvo_logs` returned no
  lines for a container that is not there; and `stackvo_mail_message` failed
  with a transport error naming an unreachable `127.0.0.1:8025` rather than
  saying the catcher was off. All four now say which it is, and the project
  lifecycle tools name the project before they ask the engine, so "no such
  project" stops arriving as "no such container" — a different problem with a
  different fix.

- **AI rules — Settings → AI rules, and `stackvo rules-install`** (competitive
  review K-3). Registering the server makes the tools reachable. It does not
  make them used: an assistant that has never seen this stack reads the source,
  guesses at nginx, and suggests editing a generated file, because nothing told
  it that `stackvo_doctor` answers that question in one call. ServBay files
  "AI Rule" beside its MCP documentation as a first-class feature, EnvKit
  installs a skill, Lerd's `mcp:enable-global` writes context files. This
  repository had no answer to that half at all.

  `rules.rs` writes a marked section into the instructions file the assistant
  already reads: `CLAUDE.md`, `AGENTS.md` (Codex and Zed), Cursor's
  `.cursor/rules/stackvo.mdc`, VS Code's
  `.github/instructions/stackvo.instructions.md`, `.windsurf/rules/stackvo.md`
  and `GEMINI.md` — in the project, or in the home directory for the three that
  read a global file. A row is a **file** rather than a product, because Codex
  and Zed share one and two rows writing one path would disagree about whether
  the rules are installed.

  It follows the same three rules `agents.rs` follows, because it edits the same
  class of file — somebody's own `CLAUDE.md`, not ours. Only the region between
  `<!-- stackvo:rules:begin -->` and `<!-- stackvo:rules:end -->` is ever
  written; a file with no markers is appended to, never replaced; everything
  else comes back byte for byte; and a `.stackvo-backup` copy is left beside it
  first. HTML comments as markers, so a Markdown preview shows the rules and not
  the plumbing. The front matter Cursor and VS Code need to apply the file at
  all is written when the file is created and never again — a user who narrowed
  `applyTo` to their PHP directories meant it, and a test drives exactly that.

  Audited, like `agent_install` and for the same reason: this writes
  instructions into a file the user owns and usually commits. And a test asserts
  that **every tool the rules name is a tool that exists** — rules that send an
  assistant at a tool the server would refuse are worse than no rules.

  **Reachable from the project as well as from Settings.** The rules are per
  project, so the project page is where somebody looking for them looks first;
  asking them to leave it, find the project again in a dropdown and press a
  button there is asking them to hold a name in their head for no reason. The
  detail page gains an **AI** tab over the same three commands, scoped to the
  project it is on — the global rows stay in Settings, because "on this
  machine" is not a fact about one project. The same tab names
  `.stackvo/context.json` and explains it rather than offering it: the
  generator writes that file for every project on every run, and a switch would
  imply it could be off.

- **Eight tunnel providers in the Share pane, not one** (§3, the tunnel).
  `tunnel.rs` was cloudflared and the shape of "one provider" had leaked into
  every part of it: the image was a constant, the URL was recognised by
  `.trycloudflare.com`, and the pane said "no account needed" as though that
  were a fact about tunnels rather than about Cloudflare. The choice is a real
  one — a quick tunnel's address changes on every start, which is right for
  "did the webhook arrive" and useless for a redirect URI somebody registers in
  a dashboard once — and the providers that keep an address are exactly the
  ones that want an account.

  A provider is now data: image, arguments, the shape of the URL it prints, and
  whether it needs a token. `cloudflare`, `localhost.run`, `pinggy` and
  `localtunnel` need no account; `ngrok`, Tailscale Funnel, `zrok` and
  LocalXpose take a token, which goes in the OS keystore beside the Stripe key
  and reaches the container as an environment variable — never as an argument,
  which `docker inspect` and this app's own operation console both print. A
  provider that needs a token and has none is refused **before** the image is
  pulled rather than after minutes of download.

  **Every one of the eight is run for real** by
  `cargo run --example tunnel_probe`, against a throwaway nginx on a throwaway
  network, using the same `run_args` the app uses. That establishes for all of
  them that the image runs, the client is inside it, and the arguments built
  here are arguments it accepts; for the four anonymous ones that a public URL
  comes back and is picked out of the client's own banner; and for the four
  that need an account that an invalid token is refused in words the pane can
  show. What is left untested for those four is a single step — what the
  provider does with a _valid_ token — and the pane says exactly that instead
  of a blanket "unverified".

  Five findings came out of watching the clients rather than reading about
  them, and each one changed the code:

  - `localhost.run` and Pinggy both link their own dashboard directly above the
    tunnel they just opened, so suffix lists written from documentation would
    have handed out `admin.localhost.run` and `dashboard.pinggy.io` as the
    public address of somebody's application;
  - `tailscale funnel` serves "a service running on the local machine", so its
    sidecar joins the **project container's network namespace** and the target
    is a port number — rather than a remote URL the documentation never
    promises to accept;
  - localtunnel's `--host` names the _tunnel server_, not the target: pointed
    at the project container it produced a client that sat in silence;
  - LocalXpose can present the local domain after all, through the
    `--request-header host:` plugin in its own help text;
  - ngrok's `--log` defaults to `false` — without `--log=stdout` the agent
    works perfectly and prints the URL nowhere at all.

  The sidecar is no longer `--rm`, for the reason `stripe.rs` already learned:
  a rejected token makes the client print its complaint and exit, and `--rm`
  takes the log away with the container — leaving the likeliest failure the
  feature has looking like a tunnel that is merely slow. `tunnel_status` now
  reads that complaint out and the pane shows it in the client's own words,
  with the provider read from a `stackvo.tunnel.provider` label rather than
  guessed from an image two of them share.

- **Two more release targets: Linux aarch64 and Windows ARM64** (§3 #22). Six
  in the matrix instead of four, on **native ARM runners** rather than by
  cross-compilation — a Tauri bundle is a platform installer, a `.deb`, an
  AppImage, an MSI, and cross-building one means a second toolchain and a
  second set of bundler quirks to keep working for a machine GitHub already
  rents by the minute. Every `matrix.os == 'ubuntu-latest'` style condition
  became a family test, or the new rows would have skipped the steps that make
  their artefacts installable; the SBOM stays pinned to one target rather than
  to the Linux family, because two attachments with one name is a collision
  rather than a second document.

  **Unverified until a tag runs it**, and the row in `docs/durum.md` says so
  rather than moving to done. Nothing on a developer's machine can prove that a
  runner label exists or that the bundler is happy on it. `fail-fast: false`,
  which was already there, is what keeps a wrong guess from taking the other
  four builds down with it.

- **An accessibility statement, and the measurement that makes it true**
  (§3 #25). [`docs/accessibility.md`](docs/accessibility.md), in the shape
  EN 301 549 asks for: conformance status, what was measured and how, the
  features that exist for it, and — at the same length — what it cannot claim.

  The row said the prerequisite was already there: "axe runs on four pages in a
  real engine, zero serious or critical". Both halves of that turned out to be
  weaker than they read, and the statement could not honestly be written on top
  of them:

  - The run was **scoped to `#app`**, and Vuetify's overlay container is a
    _sibling_ of `#app` rather than a child. Every tooltip, menu, dialog and
    side sheet in the application was outside the measurement.
  - Those overlays were **covering the page**. A closed overlay keeps a
    full-viewport box at `z-index: 2000`, so axe could not resolve a background
    for anything underneath and skipped the contrast rule almost everywhere.
    The zero was partly the page being hidden from the checker.
  - Four of the nine routes were covered, and the four were the ones somebody
    thought of.

  All three are closed: the run is over the whole document and every route the
  router declares, and `tests/accessibility-claims.spec.js` fails if the
  statement, the suite and the router come apart — including if the `#app`
  scope ever comes back. The result is **zero violations at every severity on
  all nine routes**, which is a different sentence from the one it replaced.

- **A workbench: a snippet, the application it runs inside, and what came
  back** (F-5, ADR 0022). The row called `tinker` over the PTY "honest 90%, but
  not a workbench", and §5.5 held the remaining tenth as a **decision** rather
  than a task — because `quickcmd.rs` had refused an in-app REPL in writing, and
  reversing a refusal is not something a commit does quietly.

  **The refusal was right and it still stands.** A _line_ REPL in a pane is
  exactly what it described: a worse `tinker` with no readline, no history file,
  none of the colours somebody configured, and a terminal one click away that
  does all of it better. `tinker` still opens the user's own terminal from the
  command list. What was accepted is the tool a line REPL cannot be: a snippet
  is **twenty lines you edit** — write a query, run it, change line three, run
  it again — which in a REPL is retyping. The two are not ranked, they are split
  by what the person is doing, and on screen they sit one above the other: the
  terminal pane, then this.

  **The security model is extended one level down, not broken.** The webview
  sends a runner **id** and a body of code; the id is what picks the program, so
  `laravel` means `php artisan tinker --execute` and nothing else. The snippet
  is **one argv element and always the last**, so `; rm -rf ~` is a string
  rather than a second command. No consent gate, for two reasons that are the
  whole argument: it runs in the project's **own container**, which already runs
  that repository's code, and the code comes from the **person at the keyboard**
  rather than from a file in a repository somebody cloned. There is no `host`
  runner and there will not be one — that is `hooks`' `host` step, approved
  against a digest first.

  Six runners in two tiers, and every row says which it is: `php artisan tinker
--execute`, `wp eval`, `python manage.py shell -c` and `bin/rails runner` boot
  the application; `php -r` and `node -e` are the language on its own. A pane
  that showed them alike would let somebody debug for ten minutes before finding
  out their models were never loaded. Laravel is offered only where
  `composer.json` also declares `laravel/tinker`, because `--execute` is
  Tinker's flag and a `--no-dev` install does not have it.

  **Every argv was measured before it was written**, on Laravel 13.25, Django on
  Python 3.14, Rails 8.1.3, `wordpress:cli`, and this workspace's own project
  container. Two of those measurements changed the design. Laravel's
  `--execute` does **not** echo the value of the last expression the way the
  interactive REPL does — `2+3;` produces nothing — so the pane says to
  `dump()`. And a PHP fatal is written to **stdout**, not stderr, so success is
  read from the exit code; a pane that decided by looking at stderr would have
  drawn a green chip over an uncaught exception, and a test now holds that.

  **The limit runs inside the container.** Killing a `docker exec` client does
  not stop what it started, so this app's own clock alone would have left a
  snippet with a loop in it burning a CPU in somebody's container after the pane
  said "timed out". The command is wrapped in `timeout 30` — measured present in
  the php, node, python, ruby and wordpress-cli images and in this workspace's
  project container, where `timeout 1 sleep 3` exits 124. "Every image I
  checked" is not "every image", so there is a fallback and the result carries
  `limited`, which the pane shows as a warning rather than as silence.

  History keeps **the code, never the output**: a snippet is what the person
  wrote and is the thing they want back, while the output is the application's
  data — the rule `querylog` states. It lives in the app's own config directory
  beside `hook-consent.json` rather than in the project, because a file written
  into a checkout turns up in somebody's `git status`.

  `examples/repl_probe.rs` runs the whole thing against the projects on the
  machine, and it distinguishes a broken runner from a project whose
  dependencies were never installed — on this workspace it reports the Laravel
  runner as skipped, with `vendor/` missing as the reason, and measures the
  other two.

- **`stackvo`, a command-line interface** (A-1). Eight of the ten tools this app
  is measured against ship one. It sat unbuilt because it is a **third surface**
  — the window reaches the core one way, an assistant another — and a third
  consumer is a third thing that can drift away from `contracts/ipc.json` while
  every existing test still passes.

  It is accepted on the condition the MCP server was accepted on: **every
  command names the contract command it implements**, and `cli_surface.rs`
  cross-checks the pair. A command naming something the contract does not
  declare fails the build, and so does a command listed under `--help`'s
  "Reads" heading whose contract command is a mutation — because that heading
  is what somebody reads before typing into a machine they care about.

  One part is tighter than the MCP table. A tool there dispatches on its _name_,
  so a table entry with no matching arm compiles and fails when called; the
  module says so and keeps a fallback for it. Here the table carries an
  `Action`, dispatch matches on the enum, and the compiler refuses a variant
  with no arm. There is no "listed but not implemented" state to test for
  because there is no way to reach one.

  Twenty commands: ten that read (`status`, `doctor`, `projects`, `project`,
  `services`, `logs`, `certs`, `db`, `mail`, `mcp`) and ten that change the
  stack (`up`, `down`, `start`, `stop`, `restart`, `generate`, `xdebug`,
  `certs-renew`, `mcp-install`, `mcp-remove`). Every one has `--json`, so the
  same value the table is rendered from is available to a script — the human
  output is built _from_ that value rather than from a second query, which is
  how the two cannot come to describe different things.

  **stdout is the answer, stderr is the narration.** The progress writer ADR
  0005 left room for is the fourth sink, and it prints to stderr — so
  `stackvo doctor --json | jq` works while a compose build is still scrolling
  past, and a failure leaves an empty file and a non-zero status rather than a
  file with an error message in it. The exit code distinguishes the two
  failures a wrapper script wants to handle rather than report: 3 is "nothing is
  set up on this machine", 4 is "Docker is not running".

  **No argument-parsing dependency.** What is needed is long and short flags,
  `--flag=x` and `--flag x`, and `--` to stop; what matters far more than the
  shape of that code is that an unrecognised flag is an **error**. A CLI that
  shrugs at `--tial 50` and quietly uses the default has told you it did
  something it did not do.

  **The last Tauri binding in the lifecycle path went with it.** `run_hooks`
  needed an `AppHandle` for one reason — building the sink — so its body moved
  to `hooks::run_for_project`, which takes the sink instead. `stackvo stop` and
  the stop button now run the same hooks rather than being two operations
  wearing one name. Writes land in the same audit trail, under `cli_` names:
  the log answers "what happened to this machine", and "somebody ran this in a
  terminal" is part of that answer.

- **The package index can be verified, withdrawn versions are refused** (C).
  `market.rs` described a chain of three links and said the first one — _a
  pinned key → registry.json_ — was missing. `Trust::Signed` was a shape with
  no implementation, so a third-party source could be fetched but never
  believed, and the claim that the architecture was "ready" for third-party
  distribution was not true of a door that could not be shut.

  It is now. `signing.rs` verifies a minisign signature over the index against
  keys the machine already trusts, and `refresh` runs it **before the index is
  parsed** — the same ordering this module already applied to a manifest, for
  the same reason.

  **Zero new packages.** minisign is ed25519 with a file format around it and
  is what `tauri-plugin-updater` already uses, so `minisign-verify` was in
  `Cargo.lock` already — measured, not assumed. It also means ADR 0015's key
  ceremony has a tool that exists (`minisign -G`); a scheme needing a bespoke
  tool is one whose ceremony never happens.

  **No official key is shipped, and a test keeps it that way.** Inventing a
  placeholder would be worse than the gap, because every later reader would
  believe the chain was closed. A build with no key refuses a signed refresh
  and names _that_ as the missing half. An organisation running its own mirror
  is not waiting on any of it: it signs its own index and pins its own key
  through `policy.market.additionalKeys` — a field written for exactly this
  and, until now, read by nothing.

  **Rotation is designed in, because it cannot be added afterwards.** A machine
  holds a _set_, and a new key arrives in a `known-keys.json` signed by one
  already trusted. What that deliberately cannot do is remove a compromised
  key on its own say-so — a leaked key can sign a document naming only itself —
  so retirement is a property of a build, and a retired key cannot be brought
  back by any document or by policy.

  **Takedown has both halves.** A withdrawn version is refused at install
  rather than warned about: ADR 0014 keeps it in the index so a machine can
  find out what happened to one it already has, and whether a _new_ install may
  proceed is a different question. The other half is `doctor`, which lists
  installed versions the publisher has since withdrawn — without it the
  container keeps running, the stack looks healthy, and the withdrawal is a
  line in an index nobody re-reads.

  Two decisions changed while this was being written, both because a test
  disagreed. Legacy minisign signatures are accepted after all — the reasoning
  for refusing them (that the two modes sign different things) does not survive
  contact with how the mode is declared and checked, and refusing bought
  nothing while costing an organisation whose mirror was signed by an older
  tool. And the pinned-key check now happens _before_ the signature file is
  fetched: fetching first told a machine with no key `registry.json.minisig: No
such file`, sending somebody to ask their publisher for a signature when the
  missing half was on this side.

- **A project may declare its own commands** (B-4). The catalogue is eleven
  commands most projects have; what it cannot know is the one _this_ project
  runs every day — `artisan app:reindex`, `npm run codegen`, a `bin/` script
  somebody wrote last week. Now `stackvo.json` can say so:

  ```json
  "commands": {
    "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Rebuild the search index" }
  }
  ```

  They appear in the project pane beside the built-in ones, marked as coming
  from the project, and in the terminal as `stackvo commands` and
  `stackvo run <id>`.

  **The security rule the catalogue existed for is intact.** The webview still
  only ever sends an **id** — it cannot name a program, and `quickcmd::resolve`
  is the one place either kind becomes an argv. What changed is where a command
  may be _declared_, and `docs/durum.md` §5 had been holding exactly that
  distinction open: the argument against a webview naming a program is about a
  surface that runs code it did not choose, and a file committed to the
  repository is not that surface.

  **It stops at the container line, and that is why it needed no approval
  flow.** A declared command may only be `exec` — inside the project's own
  container. There is no `host` form. `hooks.rs` already makes the argument: a
  container runs the repository's code anyway, so a repository able to run a
  command in it has gained nothing, whereas a host step is what turns `git
clone` plus a button into arbitrary code execution — and that one has a
  consent record keyed to a digest. Declaring `host` here is refused by name,
  so an author is told where the real feature lives.

  Three rules make a quiet mistake impossible. `exec` is an **argv array**, and
  a command string is refused rather than split — splitting is how
  `sh -c "a && b"` becomes four arguments. An id already in the catalogue is
  **refused** rather than allowed to win or lose silently, because either ends
  with somebody pressing a button labelled `migrate` that is not
  `php artisan migrate`. And an id is lower-case letters, digits and dashes,
  because it travels to the webview and back.

  The manifest serialiser learned the block too, for the reason the hooks block
  is written out: that text is rewritten on every form save, so a field the
  serialiser does not know about disappears the first time somebody changes an
  unrelated setting.

- **`stackvo tui`, a terminal screen you can work in** (M-8). M-8 is
  "alternative surfaces", and its own record already said what makes something
  one: the tray stopped being a shortcut the day it could **act** without
  raising the window. The same test applies here, so this is not a report —
  it lists every project and service live, moves a cursor through them, and
  starts and stops the row it is on. `l` shows a container's last lines, `r`
  refreshes, `q` leaves.

  **No TUI library.** Measured before deciding, the way `keyring` and
  `toml_edit` were: `ratatui` puts **25 new packages** in `Cargo.lock`
  (649 → 674) — a layout solver, a widget set, two `unicode-width` crates, an
  LRU cache, `strum`, `darling`, a second `rustix` — for a list, a detail line
  and a status bar. Drawing is `cli::Style` and the column arithmetic the CLI
  tables already use; the cursor, the alternate screen and colour are escape
  sequences, which are text. Raw mode is the only part that needs an operating
  system and both halves were already in the lock file — `libc` through
  `portable-pty`, `windows-sys` through Tauri. **Zero new packages**, measured
  rather than claimed.

  **The risk is the terminal, and it is paid for.** One left in raw mode has no
  echo, no line editing and no working `Ctrl-C`, and the way out is typing
  `reset` blind. All four exits are covered: `Drop` for returning and for `?`,
  a panic hook because release builds are `panic = "abort"` and `Drop` does not
  run, and `Ctrl-C` read as a key because raw mode stops the terminal turning
  it into a signal. The restore goes through one function that _takes_ the
  saved settings, so a hook and a `Drop` firing together still put the terminal
  back exactly once.

  And it is held by running rather than by reading. `examples/tui_probe.rs`
  opens a real pty, runs the real binary inside it, sends `j` and `q`, and
  reads the terminal's own settings back to confirm echo and line mode
  returned — eleven checks, because a screen that agreed with its own
  expectations is the QR encoder's mistake in a new place.

  `cli::Backing` gained a third value for it. A screen implements no single
  contract command; it drives several, and every name is still checked, with a
  test that a screen names more than one — a row that named exactly one should
  have been a `Contract`.

- **`stackvo php …`, `stackvo artisan …` — the project's container from the
  working directory** (A-3). `cd` into a project and `stackvo php -v` reports
  the PHP that project declares, with its extensions and its `php.ini`, from a
  machine with no PHP installed on it. `artisan`, `composer`, `npm`, `node`, a
  general `exec` and an interactive `shell` alongside it.

  Which project comes from the working directory, matched against the real
  project list rather than a folder name — a worktree's name lives in
  `stackvo.local.json`, not in its directory. The **deepest** enclosing project
  wins, because a worktree sits inside the project tree and answering with the
  parent would migrate the wrong database while the caller stood in the feature
  branch. `--project` names another one from anywhere.

  **The subdirectory maps through.** `stackvo artisan` from `app/Http` runs in
  `/var/www/html/app/Http`, so it behaves the way `artisan` on the host would.
  Only where the source is mounted: `generator.rs` writes no source mount for
  Node and the rest, because a bind mount over `/app` would shadow the built
  output — so there is no counterpart directory to map onto, and the one line
  that says anything written there stays in the container goes to stderr.

  **Flag parsing stops at the command name.** `stackvo artisan migrate --force`
  reaches artisan whole; a parser that kept reading would eat `--force` and then
  complain about it, which would break the most common artisan call there is.
  The cost is that StackVo's own flags come first — `--project` included, which
  is why it is global — and that `stackvo artisan --help` goes to artisan.
  `stackvo --help artisan` prints this app's, and the main help says so.

  **The exit code is passed through**, because `stackvo artisan test` in a CI
  script is worth nothing if a failing suite comes back as 0. A TTY is requested
  only when there is one to inherit, so `echo … | stackvo php` works in a
  pipeline and `stackvo shell` still gets a terminal.

  These are the first commands with no `contracts/ipc.json` command behind them,
  and that is a decision rather than an omission — ADR 0018. The webview may
  never name a program to execute, so the contract has none that takes one; a
  terminal is the opposite case, and `stackvo artisan migrate` is strictly safer
  than the `docker exec -it stackvo-shop php artisan migrate` it replaces
  because it cannot get the container name wrong. The exception is not a gap in
  the gate: `Backing::HostShell` carries four assertions of its own, including
  that every one of them runs through `docker exec` and none on the host.

- **An environment per worktree** (N). `git worktree add` gives a branch its own
  directory; this gives that directory its own hostname, its own database and
  its own environment, so `shop.loc` and `feature-x.shop.loc` are two branches
  of one application running at the same time. Of the ten tools this app is
  measured against, one does this.

  **A worktree is a project, and nothing below the module knows the word.** The
  directory lands in the project tree beside its parent, and `list_projects`,
  the renderer, the hosts writer and the certificate pick it up as they pick up
  anything else. A parallel lifecycle for something that is already a project
  would have been a second copy of every bug.

  **Nothing is written into the checkout that git tracks**, and that constraint
  is what the design turns on: the files in a worktree directory belong to the
  branch, so a derived `stackvo.json` would appear as a change to whoever is
  working on it. Identity — the name and the hostname — goes into
  `stackvo.local.json`, the machine-local overlay that exists to be the file
  that is never committed, and it is added to the repository's
  `.git/info/exclude`, which is git's own per-clone ignore list rather than the
  `.gitignore` the team shares. Everything else — the database credentials,
  `APP_URL`, the branch name — is kept outside every checkout and reaches the
  container through the compose overlay that per-project variables already use.
  A branch that carries no manifest at all gets a full one derived from its
  parent's, with the parent's extra hostnames dropped so it cannot claim them.

  That cost one change in the manifest layer, and it narrows the rule as it
  widens it: a local overlay may set `name` **only to the directory it is
  sitting in**. Every other value is refused exactly as the whole key used to
  be, because the hazard never changed — a project renamed locally builds one
  image and looks for another. Restating the directory is the one thing a local
  file can say about identity that cannot be wrong.

  **The database is a database, not an instance.** A branch is not a different
  engine, and a MySQL container per branch would spend a gigabyte of memory
  holding a copy of one schema. So a worktree gets a database on the instance
  that is already running — empty, or copied from the workspace's own. The copy
  reads the source's character set on MySQL rather than taking the server
  default: a `utf8mb4` parent copied into a server-default database produces
  tables that compare differently, with no error anywhere and a join that
  returns nothing as the only symptom.

  **Three bugs came out of running it rather than reading it.** `git worktree
remove` refuses while the tree contains **untracked** files, not only modified
  ones — and the untracked file was this app's own overlay, so a worktree with
  no user changes could not be removed, and git's message sent you looking for
  work you never did. The fix moves the overlay out of the way first, only when
  git is not tracking it, and **puts it back** if git refuses anyway: without
  that, a failed removal would strip the checkout of its name and hostname and
  leave a project the app then reports as broken.

  The second had been there for as long as the Mongo branch of `run_sql`: it
  never passed `--authenticationDatabase=admin`, and the root account lives in
  `admin` while mongosh authenticates against whatever database it connected to.
  That branch could never authenticate at all. `mongodump` and `mongorestore`
  have always carried the flag; nothing had asked Mongo a question until a
  worktree needed to know which databases it had.

  The third was an ordering one, and the sort that stays wrong for years because
  nothing visible breaks: the compose overlay is built only for projects that
  have a **compose service**, and a worktree gains one in the file the generate
  writes — so rendering the overlay first produced one with the branch's
  database credentials missing. The next `docker compose` re-rendered it and
  quietly corrected itself.

  **The database half was run against live engines**, because a `CREATE
DATABASE` can be right in every unit test and wrong at the server — the lesson
  `mariadb-dump` and the QR encoder each cost this repository once. Two
  `#[ignore]`d tests create, list and drop on MySQL 9.7, MariaDB 12.3 and
  MongoDB 8.0, and check that the workspace's own database is refused. They do
  not run in CI: a test that fails for the absence of Docker is one people learn
  to ignore for real reasons too.

  `APP_URL` is supplied deliberately. The branch's own `.env` names the parent's
  hostname, and a framework generating links from it would send everybody back
  to `shop.loc` mid-flow — the exact bug this feature would be sold as fixing.
  The container's environment wins over the application's `.env` file, which is
  what lets this work without M-5's rule bending: the framework's own file is
  still never touched.

- **The eight "small items" that were not small** (M-2, M-3, M-4, M-7, M-8, M-9,
  M-11, M-12). Each carried a dependency, a decision or a product surface, and
  the previous round said so and stopped. This round pays the costs.

  **Releasing one caught message** (M-2). The catcher goes on catching
  everything; a message you pick is sent on to a real address through a relay
  you configure. Deliberately not the other shape — pointing the application at
  a real SMTP server sends the forty password resets a test suite generates in
  an hour to whatever addresses the fixtures contain. The service package is
  untouched: Mailpit reads its relay settings from its own environment, and this
  app reaches that through a compose overlay, the mechanism `site.rs` and
  `perf.rs` already use. The catcher's compose service is found by its **image**,
  because the key is `mailpit` on one workspace and `mailpit-1-30` on another.
  The password is in the OS keystore.

  **A QR code for every address meant for another device** (M-3). The LAN name
  and the tunnel URL both exist to be opened on a phone, and both were things
  you typed by hand. The encoder is written here — a QR code is a closed
  specification with published vectors, so the usual argument for a maintained
  dependency does not apply — and it is verified against **macOS's own decoder**
  rather than against its author's expectations. That measurement earned its
  keep immediately: every unit test passed while no decoder on earth could read
  the output, because the first copy of the format information was written along
  row 8 instead of down column 8. The format names the mask, so everything else
  being correct bought nothing.

  **A landing page listing every site** (M-4). It needed no new name: the app
  already writes the bare suffix into `/etc/hosts` and already issues a
  certificate for it, and opening `https://stackvo.loc` got Traefik's own 404.
  So this is a small sidecar answering on a name the stack had claimed and left
  empty. `nginx:alpine` rather than the `alpine:3` already pulled elsewhere,
  because Alpine's busybox is built without the `httpd` applet — found by
  running it, not by reading about it.

  **Language packs** (M-7). Adding a language was a source change in three
  files and a rebuild, which is why "the app speaks N languages" was never
  going to be answered by translating 2,000 strings — the strings were not the
  blocker. A pack is now one JSON file in the config directory, discovered at
  startup and listed beside the built-in two, with the share of the app it
  covers stated. No machine translation and no English relabelled as another
  language: a string that falls back to English is honest, a fabricated one is a
  sentence somebody has to find and disbelieve.

  **A tray that can act** (M-8). Every tray entry raised the window, which made
  it a launcher rather than a surface — and "tray-only" was therefore a mode you
  could not work in. Starting and stopping a project now happens without the
  window coming forward, with a notification as the acknowledgement. The
  coloured dot stays at the top level: "is my stack up" is a glance.

  **Framework commands** (M-9). Symfony, Django, Rails and plain Ruby, each on a
  marker file only that framework writes. Rails is found by `bin/rails` rather
  than by a Gemfile — which is Sinatra and Jekyll as often as it is Rails — and
  runs through `bundle exec`, because a binstub only works if its permission bit
  survived the checkout. This is **not** the B-4 unlock: every row is still
  compiled in and the webview still sends an id.

  **A Stripe webhook listener** (M-11). `stripe listen` connects outward, so
  nothing has to be reachable and the signing secret is stable for the session —
  unlike a tunnel, whose URL changes on every start. The API key is in the OS
  keystore and reaches the container through the environment, never as an
  argument that `docker inspect` and the operation console would both print.

  **The OAuth callback** (M-12). Defined by reading what the providers require:
  a redirect URI is a **browser redirect, not a fetch**, so `https://shop.loc/...`
  works for the flow. What varies is whether a provider accepts the string at
  registration, which is a per-provider rule that is invisible at their console
  and is now written down beside both addresses.

- **A local DNS responder** (E-1). Every new project needed a line in
  `/etc/hosts` and an administrator password to put it there. Now a responder
  answers for the workspace's whole suffix, and `*.shop.loc` works — which
  `/etc/hosts` cannot express at all, and which is the only reason E-2 was left
  half-done.

  **Not dnsmasq**, which is what every comparable tool ships. It would have
  worked, and it means a second binary packaged for three platforms, a config
  file generated from this app's state, a process supervised by something, and
  a failure mode where the machine's name resolution depends on a container
  being up. For a responder whose entire job is "say 127.0.0.1 to anything
  under one suffix", that is a lot of moving parts around a function that fits
  on a page.

  **It is not a resolver, and that is the security property.** It answers for
  one suffix and refuses everything else. It never forwards, has no upstream
  and holds no cache — an open forwarder listening on a machine is a thing that
  can be pointed at, and a development tool has no business becoming the
  resolver for anything it did not create. `shop.loc` is answered, `google.com`
  is REFUSED, and there is no code path that opens a socket to anywhere. It
  binds loopback, so nothing off the machine can reach it either.

  A high port, because 53 needs root at every start and mostly does not need
  to: the resolver files on macOS and the dnsmasq drop-ins on Linux both take a
  port, so the responder runs as the user. Windows is the exception and has to
  be — its NRPT rule names a server and has nowhere to put a port — and it is
  also the platform with no privileged-port rule, so binding 53 on loopback
  there costs nothing. The one privileged act is pointing the machine at us,
  and it is a separate button from starting the responder — the same separation
  `hosts_plan`/`hosts_apply` has, because folding them together would make a
  password prompt appear from something that reads like turning a feature on.

  **It answers over TCP as well as UDP.** A stub resolver picks its own
  transport — after a truncated answer, on a retry, or because that is simply
  what it does — and a name server that only listens on UDP answers those with
  a connection refused.

  **Every reply echoes the question it answers.** This is the bug the first
  round shipped and the probe missed: a REFUSED or a NODATA carried a header
  claiming one question over a body with none, and `dig` said so on a line
  above the one the probe was reading — _"Message parser reports malformed
  message packet"_. A lenient tool reads it anyway; a stub resolver drops what
  it cannot match against the query it sent, and a dropped reply is not a fast
  failure, it is a five-second timeout. The NODATA path is not an exotic one:
  every Chrome and Safari page load asks for an HTTPS record (type 65) before
  it asks for an address, so this was on the way to every page. An EDNS query
  now gets an EDNS answer back, too.

  **Three platforms, three mechanisms, and a detection step in front of each.**
  macOS writes `/etc/resolver/<tld>`. Linux is not one answer but a question
  about the machine, so it is asked: NetworkManager's dnsmasq, a standalone
  dnsmasq, or systemd-resolved, in that order, and nothing is written when none
  of them is what sits in front of `resolv.conf` — a guessed path is worse than
  no feature, because it rearranges name resolution on a machine this app was
  wrong about. Windows gets an NRPT rule, which takes a namespace and a server,
  applies to that suffix and nothing else. The previous round said Windows had
  no per-suffix mechanism at all; that was a statement about what had been
  looked for.

  **Nothing is trusted to have worked.** Reading back the file this app just
  wrote proves a write happened and nothing else, so applying the change is
  followed by measuring it _through the machine's own resolver_: a name under
  the suffix has to come back as loopback, and a public name that resolved
  before the change has to still resolve after it. If either fails, the change
  is undone. That is what makes writing a resolver file on a Linux distribution
  nobody here runs an acceptable thing to do — the failure mode being guarded
  against is a laptop that cannot resolve anything and a user with no idea this
  app is why. The same four measurements are a **Test it** button on the pane,
  reported separately, because the repair differs: a responder that does not
  answer is this app's fault, a machine that does not ask it is the resolver
  file's, and public names failing matters more than the feature does.

  **A file that is already there is not overwritten.** `/etc/resolver/test` is
  a path dnsmasq, Valet or a colleague's script may own, and taking a suffix
  away from another tool with no way back is not a thing to discover
  afterwards. The pane says what is there and what it says, the file is copied
  aside under the same password, and turning this off **puts it back**.

  **The file a suffix change leaves behind is removed.** macOS's mechanism is
  one file per suffix, so moving a workspace from `.loc` to `.test` used to
  leave `/etc/resolver/loc` pointing at a responder that now answers for
  `.test` and **refuses** `.loc` — worse than never having written it, because
  before that those names went upstream and failed honestly. They are found,
  named on the pane, and removed by the next apply or by turning the feature
  off.

  **The doctor knows the one failure nothing else reports.** The machine's
  resolver names a port, something else takes that port, and every project
  domain stops resolving while the app, the containers and the proxy all look
  healthy. It is a row on the doctor page now, and `null` in every other state
  — including the feature being off — because a doctor that lists what is fine
  is one people stop reading.

  **The responder comes back with the app.** Turning this on and quitting used
  to leave the machine pointed at a port nothing was bound to, and every project
  domain stopped resolving until somebody found the switch again. The condition
  for starting at launch is read off the machine — a resolver file that names us
  _is_ the record that this was turned on — rather than out of a preference that
  can disagree with what the machine actually does.

  **A name answered by DNS is no longer counted as missing from the hosts
  file.** It was, which meant setting this up left the same per-project nagging
  in place and could hold the first-run gate shut over two names that resolve.

  **The four plans this machine cannot build are tested anyway.** On a macOS
  laptop the mechanism is always the resolver file, so the NetworkManager,
  dnsmasq, systemd-resolved and NRPT plans would otherwise ship without
  anything having read them; the plan is a pure function of (mechanism, TLD)
  and every one of them is checked for where it writes and whether its text
  names this machine and this port **in that file's own syntax**. So is the
  elevated command line itself — that the backup is taken before the write that
  would destroy it, that the steps are joined with `&&` so a failed one stops
  the rest, and that the reload comes last.

  The unit tests build the query with the same code that reads it, which proves
  self-consistency and nothing else. So `examples/dns_probe.rs` asks the running
  responder with the system's own `dig` — ten queries across UDP, TCP and EDNS,
  covering the wildcard, the REFUSED and the type-65 NODATA, with `dig`'s
  malformed-message warning counted as a failure rather than printed and
  ignored — and then runs the app's own self-test beside them.

  The suffix reaches a path this app writes as root, and it comes from a line
  in a file the user edits. `DEFAULT_TLD_SUFFIX=loc/../../etc` used to build
  `/etc/resolver/../../etc` — reachable only by editing your own `.env`, so
  barely an escalation, and still not something a function that runs `cp` as
  root should be capable of. One label, letters, digits and hyphens; everything
  else is refused before it reaches a path or a shell.

- **A real flame graph, and the four things running F found broken** (F-3, F-1).

  **F-3 could not be closed by drawing the same numbers better.** A flame graph
  is built from stacks — each measurement carrying its whole path, so a function
  called from two places is two boxes with their own widths — and cachegrind
  holds _edges_: the summed cost of "A called B" over every place A called B.
  `profile::call_tree` said so in its own comment and the screen honestly called
  itself a call tree. No arrangement of those edges recovers what the file does
  not contain, so the input had to change.

  Xdebug already writes the other kind. `xdebug.mode=trace` with
  `trace_format=1` records one line per function **entry and exit**, each with
  its depth and a timestamp; folding the gaps onto whatever is on the stack
  gives, per distinct path, the time that path was on it. That is a flame graph,
  and its widths are measured rather than sampled. **Trace** is now a third
  Xdebug mode beside stepping and profiling — a third mode and not a checkbox,
  because it writes a different file that a different parser reads and it costs
  far more to record.

  Measured on the running stack, not reasoned about
  (`examples/trace_probe.rs`): a PHP program calling `slow()` from two parents,
  60ms under one and 10ms under the other, comes back as **62,089µs and
  11,167µs in two boxes**. That is the sentence cachegrind cannot say.

  **Profiling had never written a file.** `xdebug.output_dir` named
  `/var/log/xdebug` from the day it shipped, and nothing on either side of the
  mount ever created that directory. Xdebug does not create it and does not
  complain — it writes nothing, silently — so switching profiling on, reloading
  with the trigger and finding an empty list was the _normal_ outcome, with no
  error anywhere to say why. It is created before every compose command now.

  **MariaDB 12 had no client to talk to.** MariaDB 11 removed the `mysql*`
  symlinks and 12 ships without them, so a `mariadb:12` container has `mariadb`
  and `mariadb-dump` and no `mysql` at all — and every database feature in this
  app asked it for `mysql`. Dumps, restores, snapshots, moves and the query log,
  all of them, on a service that is in the catalogue. The unit tests passed
  throughout, because they assert the argument _list_ and the list was right for
  the program it named. The container picks now, by asking itself which of the
  two it has.

  **The Mongo query log recorded nothing on a fresh database, and was
  unreadable when it did.** Profiling in Mongo is per database and was applied
  to the databases that existed when the button was pressed — of which a
  freshly started container has none, so the switch turned on nothing and
  honestly reported itself as off, and the everyday case (an application
  creating its database on the first write) was missed entirely. `admin` now
  carries the session, so "on" is a fact the server holds, and every read
  switches profiling on for any database that has appeared since. What it
  recorded was then shown raw: five hundred characters of `$clusterTime`, a
  signature, a session id and a read preference per row, with the `find` and the
  `filter` somewhere in the middle. The driver's envelope is stripped from what
  is displayed as well as from what is counted as a shape — one list, so a key
  that is noise in one cannot still be noise in the other.

  `examples/querylog_probe.rs` is how the last three were found and is what
  keeps them found: it switches recording on against every database that is up,
  asks a question it can recognise coming back — including the N+1 shape the
  feature exists for — reads the session, and puts each database back the way it
  found it.

- **A performance layer for the directories a bind mount is slowest at** (I-1).

  Bind-mounted source on macOS and Windows is the single most common reason
  people leave a Docker-based workflow, and this section's own note said the
  remaining work was "a sync layer". It is not what was built, because measuring
  the question the _feature_ has to answer changed the answer.

  `mount_bench` had established the general number — `:cached` and `:delegated`
  are inert, and bind→volume is 2–3× on metadata and writes.
  `examples/perf_layer_bench.rs` asks the narrower one, on an 8,000-file
  `vendor/`, with the source left bind-mounted where an editor can see it:

  ```text
                  bind    vendor in a volume    + storage/framework
    boot         1.47s    0.39s   (3.8x)        0.40s
    stat         0.42s    0.39s                 0.34s
    write        1.14s    1.21s   (nothing)     0.41s   (2.8x)
  ```

  Two of those rows decided the design. `vendor` buys the framework boot and
  does **nothing** for writes; `storage/framework` is what buys the writes. A
  single "make it faster" switch would hide that, and the directories it moved
  would be a guess about somebody's project — so the feature is a list of
  directories, each with its own switch, suggested from what the project
  actually has.

  **Mutagen is not bundled and no bidirectional sync was written.** The first is
  a second binary to package, sign and update for three platforms — the call
  this repository already made against dnsmasq, for the same reasons. The second
  is a hard problem with a long tail whose half-correct version does not fail
  loudly, it loses a file somebody wrote. Neither is needed here: nothing on the
  host writes these directories, so there is nothing to reconcile.

  The price is real and is on the row that charges it — an editor can no longer
  see `vendor/` — with a button that copies a snapshot back to the host for the
  index. The two cliffs are closed: a fresh named volume is **empty**, so
  enabling seeds it from the host copy first and refuses to save the setting if
  that fails; and deleting the volume is its own act, never the switch's side
  effect, because what is in there may be the only copy.

  Verified against the running engine rather than assumed: compose **appends** a
  second file's `volumes:` rather than replacing the first's, and a volume
  mounted at a sub-path shadows the bind — the container reads the volume, the
  host copy stays where it is, and what the container writes never reaches the
  host filesystem. That last one is the whole mechanism, and it is also why the
  seeding step exists.

- **Codex and Zed register the MCP server too — and every file now comes back
  the way it went in** (K-1).

  Two clients were missing and the module header said why. Both reasons were
  answered rather than worked around.

  **Codex** keeps its configuration in TOML, and editing TOML while preserving
  comments, key order and quoting style needs `toml_edit` — a dependency, which
  in this repository is a measured decision. Measured: `toml_edit` and
  `toml_writer` are **already in `Cargo.lock` and `NOTICE.md`** through Tauri's
  own graph, so the lock file gains two dependency edges and zero packages. The
  shape was not remembered either: a real `~/.codex/config.toml` holds
  `[mcp_servers.<name>]` with `command`, `args` and a nested `env` table, and
  OpenAI's own diagnostics reference documents the same block.

  **Zed** could not be verified against a running copy — it still cannot, so the
  shape comes from Zed's current published documentation: the flat
  `"context_servers": { "<name>": { "command": …, "args": [], "env": {} } }`,
  with no `source` key. Zed does not document _where_ that file is and keeps
  some things under `~/.config/zed` and others under `~/Library/Application
Support/Zed`, so both are looked for and whichever exists is written. Picking
  one would have been silently wrong on half the machines.

  **And running it against real files found an older fault.**
  `examples/agent_config_probe.rs` copies each installed client's actual
  configuration, registers the server, removes it again, and compares the result
  with the original **byte for byte**. The new TOML path was exact on the first
  run. Four of the JSON ones were not: `serde_json::Map` is a BTreeMap without
  `preserve_order`, so every file this app rewrote came back **alphabetised**,
  with its indentation replaced by two spaces. Nothing was lost and a 58 KB
  `~/.claude.json` still moved from end to end for an edit that added one entry
  — which is exactly what this module's first rule says it will not do.

  Both are closed. Key order is preserved (again with zero new packages —
  `indexmap` was already there) and a file is written back with its own
  indentation, tabs included, and its own trailing newline or lack of one. Five
  of five real files now round-trip byte for byte; the only difference left is
  the empty `mcpServers` map that removing deliberately leaves behind.

  One existing test had to change, and it is the interesting one: it asserted
  that a spec serialised through `Value` _tripped_ the W-01 rule, because the
  sorted keys put `extensions` before `version` and broke the Bash parser. That
  ordering can no longer be produced. The rule is still tested — against the
  literal it used to produce — and the test now records that the path which
  caused it is gone.

- **Sail is importable, and MAMP and Valet finally are too** (L).

  The line in the plan read "MAMP, Sail, Valet" and was half wrong before any
  of this: MAMP and Valet were already written, and the **point-at-it-yourself
  path refused them**. `imports_scan_at` knew two of the five sources and the
  menu offered the same two, so somebody whose MAMP is not in `/Applications`
  was told it "is not a tool this app can read" — and for Valet and Sail, which
  no scan can ever find, that path is the only one there is.

  **Laravel Sail is a third shape.** XAMPP, Laragon and MAMP keep one directory
  of sites; Valet keeps none and is read from its own config; Sail is not an
  installation at all, but a composer package inside each project. What
  identifies one is a `docker-compose.yml` that names `laravel/sail` — a compose
  file alone is not evidence, since every second PHP project has one. So Sail is
  never offered as a well-known path (`~/Code` is a convention, not a fact about
  a machine) and is always "point at the folder", which may be the project or a
  directory holding several.

  It is also the only source that says **what a site needs**: its compose file
  lists the services, and those are mapped onto this app's own catalogue —
  `pgsql` to `postgres`, `mongodb` to `mongo`. Anything with no counterpart is
  left out rather than substituted, so an import can say what to switch on
  without describing something nobody wrote.

  `examples/import_probe.rs` builds each tool's real layout in a temp directory
  and runs the shipped scanner over it. It found the bug this class of work
  keeps producing: Sail's template is indented with **four** spaces, the service
  reader was written against a two-space rule copied from this app's own
  generated compose, and it read no services at all. The indentation is taken
  from the file now.

- **Four of the twelve small items, and eight that are not small** (M).

  **Per-project environment variables** (M-5), a **directory listing switch**
  (M-6) and **SSH agent forwarding** (M-10) share one file — `.stackvo/site.json`
  — because they are one kind of thing: a setting the generator cannot read from
  the manifest, since `project.schema.json` is `additionalProperties: false` and
  frozen. They do not share a destination, which is the part worth knowing: the
  variables and the agent are a compose overlay, the listing is a _generated
  server config_, so one save runs both paths.

  The variables are set on the container and are never written into the
  application's own `.env` — that file belongs to Laravel, Symfony and everything
  since. A value carrying a newline is refused rather than escaped: the overlay
  is YAML, where a newline ends a scalar and everything after it is read as
  configuration somebody else wrote.

  Agent forwarding is the one with a real edge. **The socket is not
  `$SSH_AUTH_SOCK`**: on macOS and Windows the daemon runs in a VM where the
  host's path means nothing, and Docker Desktop publishes the agent at a fixed
  path instead. Measured rather than trusted — a container with that path
  mounted answers `ssh-add -l` with _"The agent has no identities"_, which is
  the agent **replying**, where an unforwarded one says it could not open a
  connection at all. It is off by default and per project, because anything
  running in that container can sign with every key in the agent for as long as
  it is up, and that is the right trade for a `composer install` and the wrong
  one as a permanent setting.

  **Favourites** (M-1) pin a project to the top of the list. A preference, not a
  manifest key: a favourite is about the person, and `stackvo.json` is committed
  — writing one there would put "Ali likes this project" in a teammate's diff.
  It sorts rather than filters, so it cannot become a mode somebody is stuck in.

  The other eight were measured against the code and **the "cheap" label did not
  hold**, so they are recorded with what each actually costs rather than left
  looking like an afternoon: mail relay is SMTP credentials and a decision about
  where they live; a QR code needs an encoder, and hand-rolling Reed-Solomon
  with no decoder on this machine to check it against is the exact move this
  repository refuses; a landing page needs something that serves static files,
  which Traefik does not, so it is a new container in the stack; a third UI
  language is two thousand translated strings and not a code change; alternative
  surfaces are a product surface; framework passthrough commands wait on the
  B-4 decision `quickcmd.rs` already took deliberately; the Stripe listener
  needs an account and its CLI; and the OAuth item is not yet specified enough
  to build.

- **Custom routes** (E-4). A name can be pointed at something StackVo did not
  start — a dev server started by hand, a service in another tool, a staging
  host. Traefik and the certificate were already here; the only addresses
  either would serve were the ones this app generated.

  The whole difficulty is one string. `http://localhost:3000` is read _inside
  Traefik's container_, where `localhost` is Traefik: the config loads, the
  browser gets a 502, and nothing anywhere says why. So `localhost` and
  `127.0.0.1` are rewritten to `host.docker.internal` and the row says they
  were. Refusing them would be defensible and useless — it is the address
  people have, and correcting it is something this app knows how to do. The
  base compose file gains `host.docker.internal:host-gateway` on Traefik, since
  that name resolves by itself only on Docker Desktop.

  Everything the check changes is reported rather than quietly applied: a path
  is dropped, because a proxy target is an origin and Traefik ignores the rest;
  a name outside the suffix is flagged, because the wildcard certificate does
  not cover it. Each of those is otherwise a silent failure.

  Routers are named from the domain rather than a position — a router keyed by
  index changes identity when one above it is deleted, and Traefik reads that
  as one router removed and another added, dropping a live connection for a
  route nobody touched. Saving replaces the whole list and checks every row
  before writing any, so one bad row fails the save instead of writing half of
  it, and a duplicate domain is refused: two routers on one name resolve by
  whichever Traefik read last, which is a coin toss the user cannot see.

- **Writing a service package, not only installing one** (C-1). A Market
  toolbar button creates a package, checks one, and re-seals one after an edit.

  The obstacle was never the JSON. A manifest states the sha256 of every file
  it ships and the app verifies them on every read — deliberately, because the
  point of writing a hash down is to catch the change nobody announced. Which
  also means opening `compose.yml`, changing one line and saving leaves a
  package that refuses to load, complaining about bytes rather than about the
  line just typed. A person can compute those by hand once; nobody does it
  twice. So the surface is create, check, seal, and the files themselves are
  edited in whatever the author already uses — the same reasoning `quickcmd`
  gives for opening their terminal instead of shipping a worse one.

  Sealing is not a way past the validator. It recomputes hashes and _then_
  parses the manifest, runs the manifest's own checks and puts the fragment
  through the compose policy — and writes nothing if any of those fail, so the
  manifest keeps describing the old bytes and nothing downstream believes a
  broken package is intact. A tool that sealed a fragment the machine would
  refuse to run would be producing packages that install and cannot start.

  The policy check runs on the template with its `{{ … }}` stubbed, and that is
  half a check rather than a whole one, said plainly: the key rules —
  `privileged`, `userns_mode`, and every key nobody has considered — are caught
  at the moment somebody writes them, while the value rules ask whether a mount
  source is one the _renderer_ produced, and those values do not exist until
  there is an instance. `render.rs` remains the check that decides whether this
  machine runs the thing; this one decides whether the author finds out from a
  user.

  The webview names a service and a version, never a directory: the path is
  built from the workspace root and checked components, the same
  handle-not-a-path rule `applog` and `quickcmd` state as their security model.

- **`policy.market.allowedSources`** (C-2). An organisation that runs its own
  package mirror can name it and the machine will fetch from nothing else.
  `docs/servis-market-mimarisi.md` §4.6 asks for exactly this as the enterprise
  half of the third-party gate.

  Two spellings because a source is two different things. An `https://` entry
  matches on the **host**, so one line naming a mirror allows every path on it
  — matching the whole string would refuse the same mirror over a trailing
  slash. A local path matches as a **directory prefix on a boundary**, so
  `/opt/stackvo` allows `/opt/stackvo/packages` and refuses `/opt/stackvo-evil`;
  a bare `starts_with` would let the second through, which is the same class of
  bug as reading `mysql:8.0`'s tag as a port.

  Enforced in `market::open`, the one place a source becomes something that can
  read bytes — including for a _remembered_ source, so a policy that arrives
  after somebody has already fetched takes effect on the next refresh rather
  than on the next fresh install. Not a security boundary; ADR 0009's sentence
  holds here as everywhere in that file.

  A test caught a real bug in the first version: splitting on the first `://`
  read `/tmp/https://packages.example` as the host `packages.example`, so a
  directory anybody can create under `/tmp` would have satisfied a policy
  naming a mirror. The scheme has to actually be a scheme.

- **Lifecycle hooks** (B-3). `stackvo.json` may declare commands to run on
  `post-build`, `post-start` and `pre-stop`, and the project detail page shows
  them beside the manifest that declares them.

  This is the most dangerous feature in the application and the design is
  mostly about that, so it is stated plainly: a hook lives in a repository, so
  the sequence is clone, open, press Start, and commands somebody else wrote
  run. That is the shape of a malicious `postinstall`.

  Two kinds of step, because they are not the same risk. A step that runs
  **inside the project's container** needs no approval — that container already
  runs the repository's code, its entrypoint and its dependencies, so a
  repository able to run a command in it has gained nothing it did not already
  have. A step that runs **on the machine** is gated, and the gate is consent
  to a _digest of the exact commands_: approving means "I read these", and a
  hook that changes — or a commit that changes one — asks again. A per-project
  checkbox would have meant reviewing a repository once and then trusting
  whatever it grew afterwards, which is the property that makes supply-chain
  attacks work. The approval sends the digest back to the backend, which
  refuses it if the manifest moved in between, so it is a receipt for the list
  that was on screen.

  A step is an argv array and never a command string. Everything here spawns
  argv and never a shell — that rule _is_ the security model in `runner.rs` and
  `quickcmd.rs` — and a hook taking shell text would be the one place a shell
  came back, holding text from a cloned repository. The cost is real: no `&&`,
  no pipes, no globs. A step that needs them is a script, and a script is one
  element — `["sh", "scripts/seed.sh"]` — which is the user choosing a shell
  and naming the file, not this app deciding every hook is shell.

  `pre-start` is deliberately not an event. There is no container to run in
  before a start, so its only possible occupant is a host step, and a slot that
  can only hold the dangerous kind is an invitation rather than a feature.

  A malformed hook is a warning, not an error: a typo in an optional
  convenience must not be why a project cannot be opened. Steps run in order
  and stop at the first failure, and a hook failure does **not** fail the start
  it hangs off — a container that started is started, and reporting otherwise
  would leave nobody able to tell which half broke.

  `policy.hooks` gives an administrator `enabled` and `allowHost`. Like
  `market.requireSignature`, neither can loosen anything: `allowHost` false
  stops host steps and true is already the default. In particular neither
  replaces consent — an administrator can forbid host steps fleet-wide and
  cannot approve them on a user's behalf, because approval is something a
  person does after reading a list and a file pushed to three hundred laptops
  has read nothing.

### Fixed

- **A warning that named work and offered no way to do it, and a button that
  did the work and then said it had not.** Two halves of the same gap in the
  Debug section, and the second is the one that reads as a broken app.

  Switching Xdebug on for the first time compiles the extension into the image,
  so nothing happens until the project is regenerated and rebuilt — and the pane
  said exactly that and stopped. Switching it _off_ was worse: the running
  container keeps `XDEBUG_MODE` until it is recreated, and the pane said nothing
  at all, so debugging carried on under a switch that read as off. Each state
  now carries the button that answers it, and they are deliberately different
  buttons: a first switch-on is a rebuild and takes minutes, a container that
  merely predates the overlay is a recreate and takes seconds, and turning it
  off never needs a rebuild because the extension stays in the image on purpose.
  None of them runs on its own — a switch that quietly started a rebuild is a
  surprise nobody asked for, so the warning asks.

  The second half: **pressing the button fixed the container and the screen went
  on saying it had not.** Every one of these commands returns an operation id as
  soon as the work _starts_ — that is what the operation console is for — so the
  caller's `await` resolved while docker was still recreating, and the panes
  re-read nothing at all afterwards. "The container is in debug, the setting is
  profile" survived the recreate that fixed it. All three surfaces now re-read on
  the **falling edge of the busy flag**, which is set by the operation's own
  finished event rather than by the call returning, and is therefore the first
  instant at which the container on disk is the one being described.

- **Every help document fetch was answering 404.** The repository moved to
  `stackvo/stackvo` and `help.rs`'s remote base still named
  `fahrettinaksoy/stackvo-tauri`, so the pull that exists to get a _corrected_
  help page to somebody on last month's build fetched nothing, ever. Nothing
  showed it and nothing could: a failed help fetch is silent by design and the
  panel falls back to the copy the app shipped with, which is the right
  behaviour on a slow connection and indistinguishable from a URL that can
  never work. On the machine it was written on — where the bundled documents
  _are_ the current ones — it looked perfect. The updater endpoint in
  `tauri.conf.json` was stale in the same way and from the same move.

  The interesting part is that one of the two was already guarded.
  `updater_endpoint.rs` derives its expected URL from `.git/config` and argues
  the case at length: a constant is a second copy of a fact, and the copy is
  the one that goes stale. The argument was right and it was applied to exactly
  one constant. `published_urls.rs` now applies it to the class — it scans the
  crate and `tauri.conf.json` for every GitHub repository URL and requires each
  to be the repository this checkout came from or one of a declared list with
  a reason attached, currently just `stackvo/stackvo-service-packages`. Test
  regions are excluded, so `market.rs`'s `github.com/o/r` parser fixture stays
  a fixture. The gate was verified by breaking the URL and watching it name the
  file and line.

  The help base now answers 200. The updater endpoint still answers 404 and
  that is no longer a code problem: the repository has zero releases, and
  publishing one is the work §2 has been holding all along.

### Changed

- **A finished build offers the hosts entry it needs.** `project_build`
  regenerates, builds and brings the container up — and then the project answers
  on a name the machine may not resolve, with nothing on screen at that moment
  to say so beyond a warning icon on a row somebody has to notice. The build now
  ends by opening the same review dialog every other hosts write goes through.

  It offers, it does not write. `/etc/hosts` is the app's only elevation prompt
  and the rule around it is unchanged: the diff is shown, and nothing reaches
  the file until somebody has read it and pressed apply — a build that raised an
  auth dialog in the middle of `docker compose build` is exactly what the
  separation exists to prevent.

  Three cases it stays quiet for. `build:success` is the name of the finished
  event rather than a claim, so the `success` flag decides. The DNS responder
  (E-1) answers for the whole suffix, which is what makes a per-project line
  unnecessary — where it is listening and the machine is asking it, there is
  nothing to offer; where it is configured and _down_, the line is the repair
  and the offer stands. And the project is re-read rather than taken from the
  page's list, which the same event is refreshing — reading that would be a race
  whose loser is a modal over somebody's work.

- **The tray's verbs moved under the project they belong to.** Starting and
  stopping lived in one `Start / stop` submenu that listed every project a
  second time, so acting on a row meant reading the project list twice and then
  picking out of rows labelled `shop: durdur` — the name had to be repeated out
  there to say which project was meant. Each project is a submenu of its own
  now: `Open` and, below a separator, the one verb that applies (`Start` or
  `Stop`). The name is the row above, so the verb is a word.

  The coloured dot stays at the top level, which was the reason the shared
  submenu existed — a submenu takes an icon exactly as a menu item does, so the
  glance is unchanged. `Open` needs a row of its own because a submenu title
  fires no click event on any platform; it keeps the `project:` id, so the
  handler and both front-end listeners are untouched. `tray.control`,
  `tray.startProject` and `tray.stopProject` are gone from the locales — the two
  verbs are the projects table's own (`projectsView.menu.start` / `.stop`),
  which pick between themselves the same way.

### Fixed

- **The hosts dialog was blank and its Apply button dead when a page mounted it
  open.** Everything in it comes from one `hosts_plan` call — the path, the
  diff, and `plan.changed`, which is what enables Apply — and the watcher that
  makes the call was not `immediate`. Two of the five callers render it behind a
  `v-if`, so the component was created with the flag already true and the
  watcher had nothing to fire on: a shield icon, two paragraphs, no error, and a
  disabled button with nothing on screen to explain it. The three that keep it
  mounted and flip the flag were fine, which is why it survived. The domains are
  watched now too — joined rather than compared by identity, because
  `:add="[hostsFixFor]"` is a fresh array on every parent render — so a dialog
  left open while a second build finishes shows that project's diff rather than
  the first one's under the wrong name.

- **Seventeen WCAG failures, none of which any previous run could have
  reported** (§3 #25). The corrected axe pass above found them; each is a
  success criterion rather than a piece of advice. Closed tooltips exposed as
  unnamed `role="tooltip"` nodes; three `<nav>` landmarks with no names; no
  `<h1>` on any page; a table column header with no text; and nine contrast
  failures — the page subtitle at 3.62:1, secondary text at 2.67:1, every form
  label at 4.25:1, tile labels at 4.49:1 and footers at 3.42:1.

  The last of them is the interesting one. A status colour has two jobs — a
  fill (a dot, a chip) and text — and the palette is chosen for the first:
  `#4CAF50` reads as running at a glance and is 2.77:1 as a sentence.
  Darkening the palette would be wrong twice over, because a darker dot is a
  worse dot and because `colorblind` is Okabe-Ito, whose values are the entire
  reason somebody picks it. So `src/lib/contrast.js` derives a **text variant**
  of each status colour against the theme's own surface, moved only far enough
  to meet 4.5:1 and with the hue kept, and `global.css` points the text
  utilities at it — every existing `text-success` in the application became
  readable without one call site changing.

  Derived rather than hand-picked because three palettes × four roles × two
  themes is twenty-four values to keep right by hand, and the surface under all
  of them is a setting. Being derived, it is also testable, and the test earned
  its place immediately: `readable` mixed a colour and then asked for the
  contrast of the mixture, and `parse` took strings only — so every one of
  those questions answered `null`, which compares false against everything. The
  loop ran its hundred steps, improved nothing, and returned its input. The
  only symptom was axe reporting exactly the ratio it had before the fix.

- **The hosts file asked for a password it did not need** (§3 #35). `apply`
  elevated unconditionally — a polkit dialog or a UAC prompt even where the
  file was already this process's to write: a root shell, a CI runner, a
  machine whose administrator made `/etc/hosts` group-writable on purpose. A
  password prompt that cannot change the outcome is one that teaches people to
  type their password at anything that asks, which is the opposite of what a
  single elevation point is for. It now writes directly when it can and
  elevates only when it cannot.

  Written in place rather than through `atomic::write`, which is the one place
  in this application that rule is wrong: an atomic write replaces the inode,
  so `/etc/hosts` would come back carrying the mode and owner of whatever this
  process created. A hosts file that lands as `0600 developer:staff` is one the
  next tool cannot read, and it does not announce itself.

  That is also what made the row testable. `hosts_path()` now honours
  `STACKVO_HOSTS_PATH` — the seam `STACKVO_ROOT` already is — and
  `tests/hosts_roundtrip.rs` runs plan, write, read-back, idempotence and
  removal against a temporary file, in its own process so no parallel test can
  be handed the fixture. CI already runs `cargo test` on three operating
  systems, so this closes the hosts half of "the privilege paths never ran on
  Windows or Linux". The half that remains is the elevation dialog itself,
  which needs a human, and the row says so rather than claiming the item.

- **`list_projects` has no cache, and now there is a measurement saying it
  should not** (§3 #27). The row stood at half done with "no cache" as the gap,
  which is the kind of gap that is obviously worth closing until somebody looks
  at the numbers. `examples/list_bench.rs` looks at them, and splits the call
  into the half that grows with the workspace and the half that does not:

  |                         | 1 project | 50 projects |
  | ----------------------- | --------- | ----------- |
  | the whole call          | 26.7 ms   | 38.1 ms     |
  | of which the engine     | 24.6 ms   | 34.4 ms     |
  | the tree, by difference | 2.1 ms    | 3.7 ms      |
  | per project             | 2.09 ms   | **0.07 ms** |

  The half a cache would help is free: fifty projects cost under four
  milliseconds of directory scanning and manifest reading. Everything else is
  one `stackvo_containers()` call — a fixed cost — and what that call produces
  is `running`, the one field on the row that must never be stale. It is what
  the start, stop, rebuild and terminal buttons are enabled by. So the only
  cache worth having is the one that would be wrong, and the reasoning now
  lives on the function with the numbers beside it rather than in a queue.

- **Right-to-left turned the app root round and left the rest of the window
  behind** (§3 #24). The row read "no `rtl` configuration in `vuetify.js`",
  which was wrong twice: the configuration is in `appearance.js`, where it
  belongs, and the real gap was somewhere else again — three places, none of
  them visible without an engine to lay the window out.

  `<html>` carried no `dir`, so the document's own direction never changed. The
  overlay container is a sibling of `#app`, so **every dialog, side sheet, menu
  and tooltip stayed left-to-right inside a mirrored window** — the same
  structural fact that had been hiding half the axe run. And both navigation
  drawers were pinned to `location="left"`, a physical side, so the primary
  navigation sat on the wrong edge for a right-to-left reader while its own
  contents mirrored.

  Setting `dir` on the document settles the first two at once, because
  `direction` inherits and the overlay container is a child of `body`. The
  drawers now say `start` and the side sheet `end`. Twenty-two physical CSS
  properties across fourteen files became logical ones —
  `margin-left: auto` is the "push the rest to the far side" idiom, and in a
  mirrored window the far side is the other one. `tests/e2e/rtl.e2e.js`
  measures the result with boxes; the jsdom test that already held the wiring
  now also holds the attribute.

- **The query log's Postgres half had never run, and three separate things had
  to be true before it could** (F-1). §2 carried the row as half-done with the
  reason "this workspace has no Postgres installed; the probe skips it and says
  so". A Postgres was installed. The probe stopped skipping, and what it found
  was not one gap but three, each of which alone was enough to make the pane
  show "recording" over an empty list — the worst shape a diagnostic can fail
  in, because it looks exactly like a page that ran no queries.

  **One: nothing could log in.** `db::settings` resolved the user and password
  out of `.env`, and after ADR 0016 a workspace is installed from packages — a
  package stores `USER` on the instance and renders it into the compose file, so
  `.env` holds nothing for it. Every caller fell through to a hardcoded default,
  which for Postgres is `postgres`: an account the container does not have,
  because the image created the one the manifest named. `psql -U postgres`
  against this workspace answers `FATAL: role "postgres" does not exist`, and
  that was the reply the query log, dump, restore and snapshot all got. It now
  resolves through the manifest's `connection` block — `userSetting`,
  `passwordSetting`, `databaseSetting` — which is the package contract saying
  which setting is the login, then the keystore, then `.env`, then the manifest
  default. `.env` sits in the **middle** deliberately: putting the package first
  would hand a migrated workspace's dump the manifest's default password instead
  of the real one in `.env`, which is the same bug facing the other way.

  `settings_for_instance` was reading the instance's settings map with the
  `.env` key — `instance.settings.get("SERVICE_POSTGRES_USER")` for a map whose
  key is `USER` — so it matched nothing on every workspace and was `settings`
  with a different container name. Same fix, same place.

  **Two: it was reading the wrong file.** `read` pulled the container's log
  stream, which is correct for a stock `postgres` image and wrong for every
  Postgres this app installs: the packaged `postgresql.conf` sets
  `logging_collector = on`, and a collector takes stderr out of the stream and
  writes it to a file under the data directory. `docker logs` on a four-hour-old
  container held the startup banner and one line saying the log went elsewhere.
  It now asks the server — `pg_current_logfile('stderr')` — and reads the file
  from inside the container, falling back to the stream when there is no
  collector. That answers log rotation too, which a hardcoded path would not.

  **Three: the format was pinned by half.** `log_statement` and
  `log_line_prefix` were pinned, `log_destination` was not — so a workspace
  configured for `csvlog` or `jsonlog` writes the same statements in a shape the
  parser cannot read, and unlike a wrong prefix that is invisible in everything
  the pane shows. Now pinned with the other two and reset with them.

  The `%n` epoch escape was the one thing here that looked like a version risk,
  so it was measured rather than assumed: `postgres:12` — the oldest version the
  catalogue offers — and `postgres:14` both write it. An escape a server did not
  understand would be dropped silently, which is the same empty-list failure
  again.

  Every statement this module sends Postgres now carries a
  `/* stackvo:querylog */` comment and is filtered on that, rather than on a
  keyword list that had to grow with each new statement. That list also matched
  every `SHOW` — including the reader's own, hidden in order to conceal the
  tool's one.

  Verified by `examples/querylog_probe.rs` against the live stack rather than
  against fixtures: `mysql:9.7`, `mariadb:12.3`, `postgres:14` and `mongo:8.0`
  each switched on, asked an N+1 of five, and reported the group.

- **"Start again" did nothing on Postgres, and was hidden so it would not
  show.** The log belongs to the server and this app must not rewrite it — true,
  and the wrong conclusion was drawn from it. "Cannot delete" is not "cannot
  start again". `clear` now writes a **watermark** statement, and the reader
  drops everything above it on the next read: the person pressing the button
  gets what they meant, and the server's log is left exactly as it was. Stopping
  writes it too, so a session no longer opens on the previous one's statements.

  The probe checks it, because this is the one operation whose failure is
  invisible from the caller's side — a `clear` that returns `Ok(())` having done
  nothing looks identical to one that worked.

  What the watermark cannot do is take the statements off the server's disk, and
  recording on Postgres puts every one of them there. The pane says so, beside
  the switch, while it is recording — the moment somebody decides whether to
  record against a database holding real rows.

- **Every database reported itself stopped while it was running.**
  `db::targets` asked the engine about the **service** name — `stackvo-mysql` —
  but the container comes from the instance table and is called
  `stackvo-mysql-9-7`. There has been no `stackvo-mysql` since instances landed,
  so the lookup missed every time and `running` was false for all four engines
  with all four up. That field is what the dump, restore, snapshot and query-log
  controls are disabled by, so the feature was unreachable on a working stack.
  `db::instances`, immediately above it, had always asked by container name.

  Found by running the new CLI rather than by reading: `stackvo db` printed four
  engines as down while `docker ps` listed them up. Nothing in the unit tests
  could have caught it — they check the argument list, and the argument list was
  correct for the container it named.

- **The contract gate's service suite checked a directory ADR 0016 deleted.**
  Suite C was built entirely around `skeleton/core/templates/services/` — one
  directory per service — and asked three things of it. That directory is gone
  and services are packages fetched at run time, so the checks reported
  twenty-five errors about files somebody deliberately removed, on every run.
  A gate that is always red is a gate nobody reads.

  What replaces it runs in the other direction and is the check that still has
  a subject: every `SERVICE_*_ENABLE` switch must name a service the catalogue
  knows. The obvious direction — every declared service has a switch — is wrong
  by `env.schema.json`'s own account of itself, which says the list is "the
  vocabulary, not an inventory" and names solr and clickhouse as entries with
  nothing behind them. Read this way it still catches a real defect: a
  `SERVICE_MYQSL_ENABLE` is a switch that will never turn anything on and
  nothing else in the toolchain would notice.

- **`stackvo.local.json`, this machine's overrides** (B-2). A committed
  manifest is the whole of what makes a checkout reproducible, and is exactly
  why there was nowhere to say "on _this_ machine, PHP 8.3, because I am
  chasing a bug in it". Now there is a file beside it that is not committed,
  and the project detail page has an editor for it below the manifest editor.

  Merged as JSON before validation rather than as fields afterwards. That is
  what lets an override be _checked_: a local file saying `"aliases": ["not a
hostname"]` is reported by the same rule the committed file would be, because
  validation happens on the way in and a post-merge value would arrive after
  it. The merge nests one level, so a file setting only `php.version` keeps
  `php.extensions` — a whole-value overlay would leave the project building
  fine and unable to reach its database. Arrays are replaced whole, because
  "also this" and "only this" are both defensible readings of a one-item list
  and neither should be guessed at.

  `name` and `runtime` are refused, and named when they are. `name` keys the
  container and the image; `runtime` is not a property of a machine — "PHP
  here, Go on my laptop" describes two different programs. A refused key is a
  warning when the file is read, so a project whose local file predates a
  change still runs, and an error when it is written, because at that moment
  somebody is typing it and can fix it.

  The guard is at the write, not at each caller. `manifest::read` now returns
  the effective manifest, because that is what all twenty-odd readers that run,
  render or inspect a project want, and making the overlay opt-in would have
  been twenty-odd chances to forget it. The five callers that read in order to
  write back ask for `read_committed` — and forgetting _that_ is not silent:
  `manifest::write` refuses a manifest carrying overrides. So the mistake that
  would land one developer's settings in everybody's clone fails loudly, and
  the mistake that costs nothing is the default.

  Whether the file stays out of a commit is git's business and this app does
  not write ignore rules, so it asks `git check-ignore` and reports the answer
  — three states, not two. "git had no answer" (no git, not a repository) is
  not a warning: a directory that is not under version control has nothing to
  leak into anybody's clone.

- **A command palette** (A-2). ⌘K on a Mac, Ctrl+K elsewhere, and a button in
  the toolbar with those keys printed on it — a shortcut nobody is told about
  is not a second way into the app, which was the whole of what the gap said.
  It goes to any of the seven pages, opens any project, and starts, stops,
  restarts or builds one; it drives the three stack-wide actions and the theme,
  the language and the new-project drawer.

  The list is built from the stores every time it opens rather than registered
  by pages as they mount. A registry sounds tidier and is wrong here: a command
  a page registered outlives the mount that can run it, so leaving the page
  leaves an entry whose handler closes over a component that is gone — and
  nothing about that failure is visible, because the row is listed and the
  click does nothing.

  Nothing is offered that would fail. A stopped project has no Stop, one that
  was never built has only Build, and a project whose domain has no hosts entry
  has no "open in the browser" — the same rules the project rail's menu already
  applies. The one exception is deliberate: the stack-wide actions stay in the
  list, greyed, when the engine is down, because absent would read as a missing
  feature where greyed reads as the state it is.

  The matcher is substring, not fuzzy. A subsequence matcher — the kind where
  `sts` finds `SeTtingS` — would also return "Start all containers" and "Stop
  all containers" for that query, and a score nobody can predict would then
  decide which came first. Ranking is by where the hit lands: a label starting
  with the query, then one containing it, then a match that was only in the
  section name or the hint.

  Not registered as an operating-system shortcut, though Tauri can. That would
  take ⌘K away from every other application on the machine for a palette that
  can only act on the window in front of you. It is a `keydown` on `window`,
  and it does fire from inside text fields, because a shortcut that stopped
  working because the cursor was in the search box would be the one case a user
  reaches for it hardest. It is off entirely while a first-run gate is up, for
  the reason the toolbar and both rails are hidden there: every command acts on
  a workspace or a daemon that is the thing missing.

  Keyboard-driven, so focus never leaves the input and the rows are named to a
  screen reader through `aria-activedescendant` on a `role="listbox"` — the
  pattern that exists for exactly this. The rows are still buttons: they are
  clickable too, and a div with a click handler is not reachable, not announced
  and not a control.

- **A call tree for a profile** (F-3). The cost table answers where the time
  went; it cannot answer what called that, and the parser had been reading
  caller→callee edges all along — it needs them to attribute an inclusive cost
  — and discarding the caller. It is deliberately called a call tree and not a
  flame graph: a flame graph is built from sampled stacks, so a box's width is
  how often that exact path appeared, and cachegrind holds no stacks. A branch
  here means "reaching B through A cost this much in total", which is the
  question people bring to a profile and is answerable from the file; "this
  path was taken this often" is not, and no arrangement of the edges recovers
  it. Recursion stops at the first repeat on the path and the frame is kept and
  marked, because a recursive call is a fact about the program rather than an
  edge to hide. Drawn as buttons rather than a canvas, so the rows take focus,
  carry `aria-expanded`, and can be found with the browser's own search.

- The query log covers all four databases the stack runs: Mongo joined by the
  same route Postgres did. Its note said the profiler is per-database and
  writes a capped collection — both true, and neither a reason it cannot be
  done; it is a loop. A Mongo query is a document, so the shape is the
  command's keys with the values thrown away (`{find:"users",filter:{_id:3}}`
  becomes `find users filter{_id}`), keys sorted because a driver may
  serialise a document in any order, and per-connection bookkeeping dropped
  because leaving it in would make every statement its own shape. One limit is
  measured and stated: profiling is set on the databases that exist when the
  switch is pressed, so a server with none reports itself off rather than
  pretending.
- The query log now covers Postgres as well. The first version left it out and
  wrote the reason down — its log is a stream whose format `log_line_prefix`
  changes — which was true and incomplete: this app can set that prefix itself.
  `ALTER SYSTEM` plus `pg_reload_conf()` takes effect without a restart, `%n`
  stamps each line with a Unix epoch so a Postgres statement lands on the same
  axis as a MySQL one and as a dump, and the official image writes to the
  container's stream rather than a file. Turning it off resets both settings,
  because `ALTER SYSTEM` writes a file that survives a restart and leaving a
  changed setting behind is no better than leaving the log on.
- The timeline carries mail as well, so a page load reads as what the code
  thought, what it asked the database, and what it sent. The two catchers
  disagree about the date format — Mailpit answers RFC 3339, MailHog hands back
  the message's own RFC 2822 header — and both are parsed. A date in neither
  spelling is left off the axis rather than placed at the epoch: on a timeline
  1970 is not a missing value, it is a wrong one that drags everything else into
  a corner.
- **A request timeline** (F-2), which puts what the code thought it had and what
  it actually asked the database for on one axis. Dumps carry the request they
  happened in, so several from one page load group together and the group is
  named. Queries do not, and that is stated rather than smoothed over: nothing
  in a database's general log says which HTTP request caused a statement, and
  inferring it from what sits either side would be wrong the first time two
  requests overlap — silently. Attributing a query to a request needs the
  application to say so, which is code inside somebody's project and the thing
  this was built to avoid needing.

- **Query log and N+1 detection** (F-1, which §2 calls the largest product gap).
  That row said it needed a collector inside the container; for MySQL and
  MariaDB it does not, and this is why. Both keep a general query log that can
  be pointed at a table and switched on with two `SET GLOBAL` statements at
  runtime — no agent, no image change, no restart, no code in the application.
  Statements are reduced to a _shape_ (`WHERE id = 1` and `WHERE id = 4711` are
  one question) and a shape seen three or more times is reported, which is the
  N+1 pattern. It is a session rather than a feed: the log is unsampled and
  costs write throughput, so you switch it on, reload the page you are
  investigating, look, and switch it off — and switching it off clears what was
  collected, because the log holds statement text. Postgres and Mongo answer
  "not this kind" rather than an error; their logs are a stream in a
  configurable format and a per-database profiler, which are real work of a
  different shape.

- The front end is now tested in a real browser engine (Playwright), not only in
  jsdom, and axe runs over four whole pages there. jsdom has no layout, so no
  test in this repository could establish that a control is visible, has a size,
  or can be reached by keyboard — which is why the two bugs of that kind that
  shipped were both found by a person. `tauri-driver` is deliberately not used:
  Tauri does not support it on macOS, and a suite its authors cannot run is one
  that rots until CI is the only thing that has seen it pass. This drives the
  webview and replaces one global, `window.__TAURI_INTERNALS__.invoke`, which is
  the process boundary at the same seam ADR 0001 draws it.
- Its first run found six real defects, all invisible to the existing suites:
  six anchors that carried a click handler and no `href` — so the domain in
  every row of the projects table was unreachable by keyboard and announced as
  plain text; seven action buttons in that table with no accessible name, now
  carrying the project's name because twenty rows all saying "Delete" tell a
  screen reader user nothing; two clickable icons that were both `role="button"`
  and `aria-hidden`; and a dashboard body that scrolled but could not take
  focus. A source-reading guard was added for the first class, verified by
  breaking it on purpose.
- Solr and ClickHouse in the service catalogue (27 services, 107 versions), and
  they are the first two that were never templates in this binary — every other
  entry arrived by migration, these arrived as packages. Both were measured
  rather than written from documentation: ClickHouse's healthcheck has to work
  whatever credentials are set, because a `health` block cannot read settings,
  and it does because the `default` user stays password-free even when
  `CLICKHOUSE_USER` is given. ClickHouse also refuses to start on a container's
  default 1024 file descriptors, so the fragment raises them.
- Bun and Deno as project runtimes, on the same shared template the other four
  lang runtimes use. They are not a flavour of `node`: both read the same
  `package.json`, but each is built from its own image with its own verbs, and
  folding them in would have meant one block whose meaning depended on a
  sibling key. Deno is pinned to a full patch version because `denoland/deno`
  publishes no major or minor tag — checked against the registry, and held by a
  test so a later tidy-up cannot shorten it into an image that does not exist.
  Detection treats them differently on purpose: `deno.json` is exclusive and
  decides, while a Bun lockfile beside an npm one decides nothing and falls
  through to the answer that was right before Bun was an option.
- `node.package_manager` — name npm, Yarn or pnpm and the image enables
  Corepack, which is what makes a `packageManager` field in `package.json` pin
  anything at all; without it that field is a comment. Absent is not `npm`: a
  project that names nothing builds the image it always has, byte for byte,
  which is what stops this from silently changing every existing Node project.
- A project can answer on a name other devices on the same network resolve, so
  the site opens on a real phone without editing a file on that phone. The name
  is `<project>.<address-with-dashes>.sslip.io`, which is resolved
  arithmetically out of the name itself — nothing is registered, nothing is
  published, and no traffic leaves the network. It is derived and never stored:
  a DHCP lease expires and a written-down name then points at whichever machine
  took the address, so what is stored is the intent and the app reports when
  what it rendered no longer matches where it is. Only a private address is
  offered; a public one is refused rather than publishing a development site
  under a name anybody can resolve. The visiting browser will warn about the
  certificate — it is the local CA's and that device has never seen it — and
  the pane says so beside the address, because on a phone that warning and a
  name that does not resolve look identical and only one of them is expected.
- A service's connection string can be opened in a desktop database client,
  not only copied. Which clients are offered is read from the applications
  themselves rather than from a list in this repository — on macOS from each
  bundle's own `CFBundleURLTypes`. That is not a detail: Redis Insight
  registers exactly one URL scheme, `redisinsight`, so a table written from the
  name would have offered it for `redis://` and produced a button that launches
  an application which then ignores the address it was handed. The host address
  is the one that goes across, always; the container address is a name on a
  Docker network that no client on this desktop can resolve.
- `examples/mount_bench.rs`, which measures what a bind mount costs by putting
  the same real Laravel project in front of the same PHP four ways — plain
  bind, `:cached`, `:delegated`, and a named volume — and timing an install, a
  stat pass over the tree, a small-file write loop and the framework's own test
  suite. The named volume is not a shipping option; it is the ceiling, because
  a synchronising layer can at best close the distance between it and a bind.
  The program also reports how long an empty container takes in each mode, and
  refuses to let the comparison be read when that constant moves — the machine
  being busy is otherwise indistinguishable from a mount being slow. On an
  Apple-silicon Docker Desktop the answer is that the consistency flags are
  inert — `:cached` and `:delegated` cannot be told apart from a plain bind —
  while the distance to a named volume is 2–3× on metadata and small writes.
  Nothing in the app changes yet; the number is what the decision needs.
- Project details as a page (`/projects/:name`) rather than a dialog, with
  indicator, configuration and container sections, plus the manifest editor and
  Dockerfile preview the dialog carried.
- A diagnostic log on disk, rotated daily and capped at seven files. Secret
  values in subprocess output are masked before they are written. Settings
  points at the folder.
- One-operation-per-subject locking at the IPC boundary, so the tray, a second
  view and a shortcut cannot start the same thing twice.
- Supply-chain gates: `cargo-deny`, Dependabot, and `npm audit` in CI.
- Front-end tests (vitest) and linting (ESLint + Prettier), both wired into CI.

### Removed

- `connect.rs`'s compiled-in table of twenty-five connection shapes, and the
  `.env` branch of `connect::of` that read it. A workspace with no instance
  table can no longer render a stack, so it has no running service to ask
  about — that branch was unreachable, and it carried a second copy of what
  every package manifest declares in its own `connection` block.
- `examples/build_packages.rs`, the one-shot tool that turned the embedded
  templates into packages. It still compiled and could no longer run: its input
  was the template directory removed with the migration gate, so every read
  would have returned nothing. A tool that cannot work is worse than no tool,
  because it looks like one.

### Changed

- **Turning Xdebug off no longer costs the next `on` a rebuild.** The extension
  being compiled in and step debugging being switched on were the same fact,
  so switching off removed the extension from the image — minutes for something
  that should take seconds. They are separate now (`php.xdebug` in the
  manifest): the extension goes in the first time and stays, and a toggle moves
  one environment variable. Measured before it was designed, because the answer
  depended on the number: an image carrying Xdebug at `mode=off` runs at the
  speed of one without it (0.009s against 0.009s), while `mode=debug` costs
  about 6.7×. The same measurement retired an assumption:
  `start_with_request=trigger` does _not_ reduce that cost — it is
  indistinguishable from `default`, because the hook is loaded whenever the mode
  is on and the trigger only decides whether to dial the debugger. The pane says
  which toggle rebuilds and which restarts, because otherwise the second one
  being far faster reads as a fault.

- **A workspace that still keeps its services in `.env` is migrated behind a
  gate, and the old path is gone.** StackVo rendered services two ways — from
  `.env` and twenty-five templates compiled into the binary, or from
  `instances.json` and the package catalogue — so that existing installs kept
  working while the second was built. Keeping both is what left the catalogue
  with two lists that disagreed: adding Solr and ClickHouse as packages made a
  project declaring `services: ["solr"]` get a correct declaration met with a
  warning that could not be fixed. `MigrationGate` is the fourth of the
  first-run screens; it shows the plan before writing anything, backs `.env` up
  to `.env.pre-market.bak`, and can be left — the app then opens without
  services, which is still a reverse proxy, a certificate authority and a
  project runner. What leaving it does not do is bring the old stack back.
  A workspace reaching the renderer without a table is refused by name rather
  than rendered into an empty stack.

- The unmanaged code on the Projects page — folders under `projects/` with no
  `stackvo.json`, and the XAMPP and Laragon folder pickers — moved off the page
  and into an overflow menu beside Refresh. The button carries a count, because
  those folders are invisible everywhere else in the app and that is exactly
  why they accumulate.
- `.env` and `stackvo.json` are written atomically, and `.env` patches are
  serialised so two callers cannot lose each other's change.
- The Rust toolchain is pinned rather than tracking `stable`.
- The bundle ships one font format instead of four: 5.2 MB of assets down to
  2.1 MB.

### Fixed

- Dump, restore, snapshot and the new query log all ran against a container
  that does not exist on a migrated machine. `db.rs` built the name as
  `stackvo-<service>`, which was right while every service was single-instance
  and named after itself and has not been right since the instance table
  arrived — an instance is `stackvo-mysql-9-7`, and after the migration gate
  there is no other kind of workspace. The same mistake `list_services` was
  fixed for, in a quieter and more expensive place.

- A container that passes or fails its own healthcheck now reaches the UI.
  Docker reports a health verdict as its own event rather than as a state
  change, so it was dropped with `exec_start` and friends — a service that had
  genuinely become healthy kept the hourglass it was given at boot until
  something else made the app refetch, which is why leaving the page and coming
  back "fixed" it.
- Project and service names are validated at every entry point. A name
  containing `..` previously produced a path outside the workspace, which
  mattered most in `project_delete`, where the next call is `remove_dir_all`.
- A service id the contract does not define is refused instead of writing a
  `SERVICE_<JUNK>_ENABLE` key into the user's `.env`.
- Per-container stats history no longer keeps series for containers that are
  gone.

### Security

- Releases cannot be published without an updater public key and signing
  secret; the workflow fails in preflight rather than shipping artifacts the
  updater will refuse.
