# Accessibility statement

**StackVo Desktop** — statement prepared 16 August 2026, against the code in
this repository at that date.

This is a statement about a desktop application, written to the shape EN 301 549
asks for (§9, which adopts WCAG 2.2 Level AA for non-web software). It is
published in the repository rather than on a website because that is where the
software is, and because every number in it is reproducible from a checkout with
one command.

---

## 1. Conformance status

**Partially conformant with WCAG 2.2 Level AA.** Partially, and the word is
doing real work: the automated criteria are met on every screen and the
criteria a machine cannot decide have not been audited by a person or with a
screen reader. §4 says exactly which those are.

Nothing here is a claim about a certification. No third party has audited this
application, and this statement is a self-assessment.

## 2. What was measured, and how

`axe-core` 4.x, run through Playwright against Chromium, over the **rendered
page in a real engine** — layout done, colours computed, overlays where they
actually end up in the document.

```sh
npm run test:e2e          # tests/e2e/a11y.e2e.js
```

**Result: zero violations, at every severity, on all nine routes.**

| Route | Violations |
| --- | --- |
| Dashboard `/` | 0 |
| Projects `/projects` | 0 |
| Project detail `/projects/:name` | 0 |
| Market `/market` | 0 |
| Logs `/logs` | 0 |
| Dumps `/dumps` | 0 |
| Mail `/mail` | 0 |
| Settings `/settings` | 0 |
| About `/about` | 0 |

The list is the router's own, and `tests/accessibility-claims.spec.js` fails if
this table and the routes come apart — a statement about "the application" that
quietly measured four of nine screens is the failure mode this table exists to
prevent.

Two more checks run in the same engine, in `tests/e2e/shell.e2e.js`: that every
visible control has a size a pointer can hit, and that the keyboard reaches
something and that what it reaches can be seen. Beside those, 800-odd unit tests
include an axe pass over individual components (`tests/a11y-axe.spec.js`) and a
source scan for icon-only buttons with no accessible name (`tests/a11y.spec.js`).

### What the measurement used to miss

Recorded because a conformance statement is worth exactly as much as the
measurement under it, and this one was weaker than it looked until August 2026.

* The axe run was **scoped to `#app`**. Vuetify's overlay container is a sibling
  of `#app`, not a child — so every tooltip, menu, dialog and side sheet in the
  application was outside the measurement, along with the two rules that only
  exist against `<html>`.
* Worse, those overlays were **covering the page**. A closed overlay kept a
  full-viewport box at `z-index: 2000`, so axe could not determine a background
  for anything underneath and skipped the contrast rule almost entirely. The run
  reported "zero serious violations" because the page was hidden from it.

Both are fixed. What the corrected run then found — and what §5 records as
fixed — was seventeen real failures across four screens, none of which any
previous run could have reported.

## 3. Accessibility features

These are settings, in **Settings → Appearance**, and they persist:

* **High contrast**, which raises the emphasis and border opacities the whole
  interface is drawn through.
* **Status palette**, including an **Okabe-Ito** option: the colours that mean
  running, degraded, failed and idle, chosen to stay distinguishable under every
  common form of colour-vision deficiency. Status is never carried by hue alone
  — every coloured dot has a label or an icon beside it.
* **Interface scale**: root font size, so the whole type scale grows with it
  rather than one label at a time.
* **Reduced motion**, honoured for transitions and animated meters.
* **Density**, three steps, applied globally rather than per component.
* **Right-to-left**, as a layout preference rather than a property of the
  chosen language.
* **Theme**: light, dark, or the desktop's own setting, followed live.

Beyond the settings: text is real text and never an image of text; every icon
button carries an accessible name (there is a test that fails the build
otherwise); the terminal and log views can be read at any interface scale; and
the application is fully operable from the keyboard, with a visible focus
indicator on what the keyboard reaches.

## 4. Known limitations

Stated plainly, because a statement without them is a marketing page.

Each of these is also **tracked as work** in `docs/durum.md` §2 (Y-1), which is this repository's single backlog. This document states what is true today; that one says what is going to be done about it.

* **No screen-reader audit.** Nothing here has been driven with VoiceOver, NVDA
  or Orca by a person. Automated tooling decides roughly a third of WCAG's
  success criteria; the rest — whether a label *makes sense*, whether an error
  message says what to do, whether the reading order matches the visual one —
  needs a human, and no human has done it.

  None of those three is wholly a judgement, and measuring them found two real
  defects. **Reading order against
  visual order is a fact**, and measuring it found one real defect — the
  template chooser in the new project drawer was pulled above the form it
  decides the meaning of, so a screen reader read the whole form first — now
  fixed, with `tests/reading-order.spec.js` requiring every re-ordering to say
  which of the two sequences is the meaningful one. **Whether an error says what
  to do is countable**: 178 of this application's 629 error constructions carry
  a suggestion. **Whether a label makes sense has a mechanical floor**, and this
  application was not standing on it: the Dashboard offered twelve controls
  under two distinct names, eleven of them "what this card is for". Every one
  had a name and every automated check passed — the failure only exists at page
  scale, which is where `tests/accessible-names.spec.js` now looks. Each help
  button carries the name of the card it belongs to.

  The wording pass has since been done from that transcript and its findings
  fixed — two buttons on one page announced as "Temizle" for different actions,
  and a search field with no name of its own. The reason it had not been done
  before was never unwillingness. It was that the job meant installing a screen reader, learning
  its rotor and driving it blind across thirty screens in two languages, then
  repeating that after every change. `npm run a11y:transcript` writes
  `docs/accessibility-transcript.md`: every page's headings and controls, in the
  order the markup puts them, under the name a screen reader announces, in both
  languages. What is left after it is narrower and cannot be closed from here:
  **nobody has used this application with a screen reader.** A transcript says
  what is announced; it does not say what using it is like — whether a flow can
  be completed by ear, whether focus lands where it should after a dialog, where
  it becomes tiring. This statement does not claim otherwise.
* **No audit of the native window.** The measurement drives the front end in a
  browser engine. The window chrome, the menu bar and the tray menu are the
  operating system's, reached through Tauri, and are not covered.
  `tauri-driver`, which would cover them, does not run on macOS — Tauri's own
  documentation says so — and this application is developed on macOS. That is
  item #12 in `docs/durum.md`.

  `tauri-driver` was named as the blocker and that was wrong in a way worth
  writing down, because it kept this unstarted: **WebDriver does not reach a
  native menu on any platform.** It drives the web view. What does reach one is
  the accessibility API — the layer a screen reader itself reads — and macOS
  exposes it to any granted process. `src-tauri/examples/native_ax_probe.rs`
  reads the running application's tree and reports what a screen reader would be
  handed. It found two defects on its first run, both now fixed and both in §5.

  So what is covered is: every window this application builds carries a title —
  the window's accessible name to the operating system and to a screen reader —
  every menu item is named from the same catalogue the interface is, in the
  interface's language rather than the build's, and all of it is re-labelled
  when the language changes. `src-tauri/tests/native_window_claims.rs` fails the
  build otherwise.

  The tray is not out of reach either: macOS puts a status item on its own menu
  bar and the probe reads it there. Its tooltip — the only name it carries, in
  the `AXHelp` attribute rather than `AXTitle`, which an icon-only status item
  does not have — is now set when the icon is created rather than when the first
  engine check lands.

  What is still owed is the judgement half, and it is the same gap as the
  screen-reader audit above rather than a second one: the reading order is now
  a list the probe prints and the presence of every name is a build failure, so
  what is left is whether that text is *good*. That needs a person, and it is
  the same person Y-1 needs.
* **Dialogs and drawers are measured as they load.** The axe pass opens each
  route and measures what is on it. A dialog somebody opens by clicking is in
  the overlay container and therefore in scope, but only if something opened it
  during the run.
* **No conformance claim for third-party content.** The Market lists packages
  whose descriptions come from a catalogue this application does not write.

## 5. What was fixed to make this statement true

Every one of these was found by the corrected measurement in §2, and each is a
WCAG failure rather than a piece of advice:

| What | Where it was | Criterion |
| --- | --- | --- |
| Closed tooltips exposed as unnamed `role="tooltip"` nodes | every page that has built one | 4.1.2 Name, Role, Value |
| `<html lang>` fixed at `en`, so a Turkish window announced itself as English | the whole application | 3.1.1 Language of Page |
| The About window built with an empty title, so the window that says which version is installed had no accessible name | the About window | 2.4.2 Page Titled, 4.1.2 Name, Role, Value |
| `Hide stackvo-desktop` and `Quit stackvo-desktop` in the app menu — Tauri's default label interpolates the crate name, not the product | the macOS menu bar | 4.1.2 Name, Role, Value |
| The tray icon created with no tooltip, so the status item carried no name at all until the first engine check landed — and none if it never did | the menu-bar status item | 4.1.2 Name, Role, Value |
| No marking on a passage in another language — a log line, a captured dump, docker's output, the message Rust wrote | Logs, Dumps, the operation console, every error alert | 3.1.2 Language of Parts |
| Eleven help buttons on one page all announced as "what this card is for", so a screen reader could not tell them apart | Dashboard, and every card that carries help | 2.4.6 Headings and Labels |
| The new project drawer read the form before the template chooser that decides what its fields mean | the new project drawer, under 720px | 1.3.2 Meaningful Sequence |
| Three `<nav>` landmarks, none of them named | the whole application shell | 1.3.1 Info and Relationships |
| No `<h1>` on any page | every page | 1.3.1, 2.4.6 Headings and Labels |
| A table column header with no text | Projects | 1.3.1 |
| Page subtitle at 3.62:1 on the primary bar | every page | 1.4.3 Contrast (Minimum) |
| Secondary text at 2.67:1 (`text-grey` on white) | Dashboard, Projects | 1.4.3 |
| Field labels at 4.25:1 and 3.97:1 | every form in the application | 1.4.3 |
| Tile labels at 4.49:1 and footers at 3.42:1 | project detail | 1.4.3 |
| Status colours as text: green 2.77:1, orange 2.37:1 | project detail, and every pane using them | 1.4.3 |

The last one is worth a note, because the fix is not "use a different palette".
A status colour has two jobs — a fill (a dot, a chip) and text — and the palette
is chosen for the first. `src/lib/contrast.js` derives a **text variant** of each
status colour against the theme's own surface, moved only far enough to meet
4.5:1 and with the hue kept, so the dot stays the colour somebody picked and the
sentence beside it can be read. It is derived rather than hand-picked because
three palettes × four roles × two themes is twenty-four values, and the user can
change the surface under all of them.

## 6. Feedback

If something here is wrong, or if you are blocked by something this statement
does not mention, open an issue in this repository. That is the only channel,
and saying so is better than naming one nobody watches.
