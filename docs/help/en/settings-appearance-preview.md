# Preview and audit

Both themes at once, and the contrast each of them actually renders.

## What it shows

| Part | What it is |
| --- | --- |
| The two cards | The light and the dark theme, built from your current settings and drawn side by side. Neither one is the theme the app is using — they are rendered in place. |
| The table | The measured contrast ratio of each pair, in both themes, with its WCAG grade. |
| The tonal ramp | Material's tone ladder for the accent and the colour derived from it. Each block is one tone; hover for its code and hex. |

## The grades

- **AAA** — 7:1 or better. WCAG's enhanced level for body text.
- **AA** — 4.5:1 or better. The level this app is built to meet everywhere.
- **Low** — under 4.5:1. Nothing shipped with the app measures this; if you see it, a colour choice is at fault and the row names which one.

## Worth knowing

- The **secondary text** row is not the contrast between two theme colours. Captions, hints and field labels are drawn translucently, so the row measures the colour that is actually composited onto the surface — which is lower, and is the number that decides whether the app passes.
- **Button text** rows measure a colour the app does not store. Vuetify picks the text colour for a filled button from the fill itself, so those two rows are checking a decision made for you.
- Changing the contrast level moves the secondary-text row and the four status rows. It does not move body text or button text, which pass on their own at every level.
- The tonal ramp is drawn with Material's own colour engine, so its steps are evenly spaced to the eye rather than to a number. It covers the two accents only: the neutral palettes here are hand-picked, and an engine-derived ramp beside them would show colours the app never renders.
- The preview is a picture: it is skipped by keyboard navigation and by screen readers, because the table below says the same things in words.
