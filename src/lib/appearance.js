import vuetify from '@/plugins/vuetify';
import { i18n, packDirections } from '@/i18n';
import {
  AA_TEXT,
  AAA_TEXT,
  contrast as ratioOf,
  luminance,
  over,
  parse,
  readable,
  toHex,
} from '@/lib/contrast';

/**
 * The look of the app, as data.
 *
 * Vuetify can be re-themed at runtime — `theme.themes` is a writable ref and
 * the stylesheet it generates is recomputed from it — so everything here is a
 * value the user picks rather than a build-time constant. What Vuetify cannot
 * do at runtime is the SASS half: corner radius and typography are compiled to
 * fixed pixels, so those two ride on CSS custom properties instead, with the
 * override layer in `styles/global.css`.
 *
 * Persisted under `appearance` in preferences.json, applied by the store in
 * `stores/appearance.js` at boot — not by the settings page, which is opened
 * long after the first paint.
 */
/**
 * The surface a console keeps when `darkConsoles` is on.
 *
 * One constant because it was two literals, and they were in the same file:
 * `TerminalPane` set xterm's background in JavaScript and the host element's in
 * CSS, both spelled `#12121a`, with nothing holding them together. The failure
 * they invite is quiet and specific — the two drift, and a frame appears around
 * the terminal in the colour of whichever one moved.
 *
 * Darker than any theme's `surface`, and deliberately so: this is the colour
 * that says "this panel is a console", which is the whole of what `darkConsoles`
 * asks for. `LogView` and `OperationConsole` get the same effect by switching
 * to Vuetify's dark theme, which they can because they are made of components;
 * xterm paints its own canvas and has to be told.
 */
export const CONSOLE_BACKGROUND = '#12121a';

export const DEFAULT_APPEARANCE = {
  /** 'system' is Vuetify's own: it follows prefers-color-scheme and updates live. */
  theme: 'system',
  primary: '#1976D2',
  /** How `secondary` is derived from the accent. See `harmonise`. */
  harmony: 'analog',
  neutral: 'graphite',
  radius: 24,
  fontFamily: 'system',
  /** Root px. Vuetify's type scale is in rem, so this scales the whole UI. */
  fontSize: 15,
  /** 'standard' | 'medium' | 'high'. See `CONTRAST_LEVELS`. */
  contrast: 'standard',
  rtl: false,
  /** Which set of colours carries running / stopped / failed. */
  statusPalette: 'default',
  /**
   * How tight the controls are. Every `density` prop in the app was removed in
   * favour of this: with them in place the setting moved nothing, because a
   * prop written on the component outranks any default.
   */
  density: 'default',
  /** Terminals and log views keep a dark surface whatever the app theme is. */
  darkConsoles: true,
  reduceMotion: false,
  /** Take the accent from the desktop instead of the swatch grid. */
  useSystemAccent: false,
  /**
   * Language for the log and terminal panels: 'app' follows the interface, a
   * locale code pins them. Pinning to English is what makes an error message
   * pasted into an issue readable by someone who does not share your UI
   * language.
   */
  consoleLocale: 'app',
};

/**
 * The two numeric settings and the ranges they are allowed to take.
 *
 * Here rather than on the sliders because there are now three readers and
 * `min="0" max="24"` written in a template is not somewhere the other two can
 * look: the pane draws the sliders, and the import validator has to reject a
 * radius of 9,000 that arrived in somebody's pasted JSON. Two copies of a
 * bound is a settings file that the UI cannot represent.
 */
export const RANGES = {
  radius: { min: 0, max: 24 },
  fontSize: { min: 12, max: 20 },
};

/**
 * A saved settings object, brought up to the current schema.
 *
 * Called on everything that comes off disk — the live preferences and every
 * saved preset — because both were written by whichever version of the app the
 * user happened to be running, and a preset is the sneakier of the two: it can
 * sit unopened for a year and then be applied.
 *
 * Merged over `DEFAULT_APPEARANCE` by its callers, so a field that simply did
 * not exist yet needs nothing here. What needs this is a field that *changed
 * shape*, and there has been one:
 *
 *   `highContrast: true` → `contrast: 'high'`. A switch became three stops. The
 *   old `true` meant "as much help as this application can give", so it maps to
 *   the top rather than to the middle — a reader who had asked for the most and
 *   silently received less would have no way to notice.
 *
 * The stale key is deleted rather than ignored: it would otherwise ride along
 * in every future write, and be copied into every new preset, forever.
 */
export function migrate(values) {
  const next = { ...values };

  if ('highContrast' in next) {
    // A newer `contrast` wins. Both keys are present in exactly one situation —
    // a preset saved before the change, applied after it — and there the
    // explicit value is the one the user last chose.
    if (next.highContrast === true && !values.contrast) next.contrast = 'high';
    delete next.highContrast;
  }

  return next;
}

/**
 * The colours that mean "running", "stopped", "failed".
 *
 * This is the one palette in the app that is not decoration. A dashboard whose
 * primary signal is a red or green dot is unreadable to the ~8% of men with a
 * red-green deficiency, and no amount of theming fixes that if the only two
 * states differ by hue alone.
 *
 * `colorblind` is Okabe-Ito, the palette designed to stay distinguishable under
 * every common form of colour vision deficiency: blue-green against vermillion
 * rather than green against red.
 */
export const STATUS_PALETTES = [
  {
    id: 'default',
    colors: { success: '#4CAF50', warning: '#FB8C00', error: '#FF5252', info: '#2196F3' },
  },
  {
    id: 'colorblind',
    colors: { success: '#009E73', warning: '#E69F00', error: '#D55E00', info: '#56B4E9' },
  },
  {
    id: 'muted',
    colors: { success: '#5E8C61', warning: '#B58B34', error: '#B4544E', info: '#4F7CAC' },
  },
];

/**
 * How hard the interface pushes for legibility.
 *
 * This was a switch — `highContrast`, on or off — and Vuetify Studio's version
 * of the same control is three stops rather than two, which is the right shape
 * for a reason the switch hid: the gap between them is large. Standard is WCAG
 * AA on body text and high is AAA, and a reader who needs *some* help had to
 * take all of it, including the heavier dividers that come with it.
 *
 * Studio reaches its three stops by recomputing every role's tone through
 * Material's contrast curves, which needs Material's engine. The three
 * numbers below reach the same place with the levers this application already
 * has, and — unlike the curves — each one can be *measured*: `target` is handed
 * to `readable`, and `tests/contrast.spec.js` asserts the ratio that comes back.
 *
 *   `emphasis`  Vuetify's `medium-emphasis-opacity`. Every caption, hint and
 *               field label in the application is drawn at this, composited
 *               with the text colour's own 0.87 alpha.
 *   `border`    `border-opacity`, which is every divider and outlined field.
 *   `target`    The ratio `readable` lifts the status text colours to.
 *
 * `standard` is not a fallback: 0.76 was arrived at by measuring, and a caption
 * at that emphasis renders 5.29:1 on white and 5.15:1 on the search field's
 * `#f7f7f7` — comfortably past AA either way. The two levels above it are
 * enhancements on a baseline that passes on its own, which is the whole reason
 * a middle stop is worth offering rather than one big jump.
 */
export const CONTRAST_LEVELS = {
  standard: { emphasis: 0.76, border: 0.12, target: 4.5 },
  medium: { emphasis: 0.84, border: 0.2, target: 5.5 },
  high: { emphasis: 0.9, border: 0.3, target: 7 },
};

/**
 * The accent, offered as swatches rather than a colour picker.
 *
 * A free picker lets you choose a primary that white text cannot sit on, and
 * every one of these has been checked against both surface families. Vuetify
 * derives `on-primary` from the value automatically, so the text on a button
 * stays legible without a second choice being asked for.
 */
export const PRIMARY_SWATCHES = [
  '#5E35B1',
  '#7C4DFF',
  '#3F51B5',
  '#1E5FD8',
  '#1976D2',
  '#0288D1',
  '#00ACC1',
  '#009688',
  '#00A870',
  '#2E7D32',
  '#7CB342',
  '#F5A623',
  '#FB8C00',
  '#F4511E',
  '#E53935',
  '#D81B60',
  '#EC407A',
  '#D500F9',
  '#8E24AA',
  '#546E7A',
];

/**
 * Where `secondary` comes from.
 *
 * It used to come from nowhere. `plugins/vuetify.js` declared it once as
 * `#5C6BC0` and `applyAppearance` never touched it, so choosing an accent moved
 * `primary` and left the indigo behind — on the timeline rule, which reads
 * `--v-theme-secondary` directly; on the replay button in `ProjectDetail`; and
 * on **every checkbox in the application**, because the md3 blueprint ships
 * `VCheckbox: { color: 'secondary' }`. A purple accent had blue tick boxes, and
 * nothing on screen said why.
 *
 * The fix is Vuetify Studio's, which offers four relationships between the
 * accent and the colours drawn from it. Three are a rotation of the hue by a
 * fixed angle; `mono` is no rotation at all and a third of the saturation taken
 * out, which is how you get a partner that reads as the same colour, quieter.
 *
 * ## Why HSL rather than HCT, and what that costs
 *
 * Studio does this in HCT, Google's perceptual space, because it is deriving a
 * whole tonal ramp and the tones have to look evenly spaced. Here the
 * derivation produces exactly one colour, used as a fill behind text Vuetify
 * writes for us — `on-secondary` is generated from whatever this returns, so
 * legibility is not this function's problem. HCT would put the 42 KB of
 * `@material/material-color-utilities` on the boot path, because the theme is
 * applied before the first paint and nothing here is lazy.
 *
 * A plain hue rotation is nine lines, and it is **wrong** in a way worth
 * writing down, because the first version of this shipped it. HSL's `l` is not
 * perceived lightness: rotating `#1976D2` by −30° at the same `s` and `l` gives
 * `#19D2D1`, and cyan at half lightness is far brighter to the eye than blue at
 * half lightness. The accent was a calm mid-blue and its partner was a highway
 * sign. What HCT gives for free — a constant *tone* — is recovered here by
 * measuring: rotate the hue, then move `l` until the relative luminance is back
 * where it started. `luminance` is already in `contrast.js` and already tested,
 * and it is monotonic in `l`, so a bisection finds the answer in 24 steps.
 */
export const HARMONIES = ['analog', 'triadic', 'split', 'mono'];

/** Degrees around the wheel. `mono` moves saturation instead, so it is absent. */
const HARMONY_ANGLES = { analog: 30, triadic: 120, split: 150 };

/**
 * `[r, g, b]` in 0–255 to `[h, s, l]` with the hue in degrees.
 *
 * @param {number[]} channels
 * @returns {number[]}
 *
 * Annotated for the same reason `toHex` is, and caught the same way — by
 * `npm run types:tsc` rather than by reading. Destructuring in the parameter
 * has TypeScript infer a three-tuple, and the only caller passes `parse`'s
 * return, which is a list: a `number[]` "may have fewer" than three elements as
 * far as the checker is concerned, so the call site was refused.
 */
function toHsl([r, g, b]) {
  const [rn, gn, bn] = [r / 255, g / 255, b / 255];
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  const span = max - min;
  // Black, white and every grey in between: no hue to rotate, and the
  // saturation expression below would divide by zero reaching that conclusion.
  if (span === 0) return [0, 0, l];

  const s = span / (1 - Math.abs(2 * l - 1));
  const h =
    max === rn
      ? ((gn - bn) / span + (gn < bn ? 6 : 0)) * 60
      : max === gn
        ? ((bn - rn) / span + 2) * 60
        : ((rn - gn) / span + 4) * 60;
  return [h, s, l];
}

/** And back, to the `[r, g, b]` triple `toHex` takes. */
function fromHsl(h, s, l) {
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  const [r, g, b] = [
    [c, x, 0],
    [x, c, 0],
    [0, c, x],
    [0, x, c],
    [x, 0, c],
    [c, 0, x],
  ][Math.floor((h % 360) / 60)];
  return [(r + m) * 255, (g + m) * 255, (b + m) * 255];
}

/**
 * The lightness at which `(h, s)` has the luminance asked for.
 *
 * Bisection rather than algebra: luminance is a piecewise expression in three
 * channels that are themselves piecewise in `l`, and it is monotonic, which is
 * the only property a bisection needs. Twenty-four halvings put the answer
 * inside 1/16,000,000 of the interval — far below the rounding to eight bits
 * that happens next.
 */
function lightnessFor(h, s, target) {
  let low = 0;
  let high = 1;
  for (let step = 0; step < 24; step += 1) {
    const mid = (low + high) / 2;
    if (luminance(fromHsl(h, s, mid)) < target) low = mid;
    else high = mid;
  }
  return (low + high) / 2;
}

/**
 * The accent's partner: the same weight, a different hue.
 *
 * Studio derives *two* colours per harmony — `secondary` at `hue - angle` and
 * `tertiary` at `hue + angle`. Only the first is returned here: nothing in this
 * application draws a third accent, and a theme role no component reads is a
 * value somebody has to keep right for no reason.
 *
 * `mono` keeps the hue and takes a third of the saturation out, and it is put
 * through the same luminance match as the rotations — dropping saturation at a
 * fixed `l` lightens a colour too, by less, and a rule that holds for three of
 * four cases is a rule nobody can predict.
 *
 * Returns its input unchanged when handed something that is not a colour, the
 * same contract `readable` has, so a corrupted preference degrades to the
 * accent rather than to `null` — which Vuetify would write into the stylesheet
 * verbatim.
 */
export function harmonise(color, harmony = 'analog') {
  const rgb = parse(color);
  if (!rgb) return typeof color === 'string' ? color : null;

  const [h, s] = toHsl(rgb);
  const weight = luminance(rgb);

  if (harmony === 'mono') {
    const muted = (s * 2) / 3;
    return toHex(fromHsl(h, muted, lightnessFor(h, muted, weight)));
  }

  const angle = HARMONY_ANGLES[harmony] ?? HARMONY_ANGLES.analog;
  const turned = (h - angle + 360) % 360;
  return toHex(fromHsl(turned, s, lightnessFor(turned, s, weight)));
}

/**
 * Surface families. Only the greys move — the accent is chosen separately, and
 * a palette that changed both at once would make the two choices inseparable.
 *
 * `on-surface-variant` travels with them because it is the colour of secondary
 * text on those surfaces; leaving it fixed makes the warm palettes look muddy.
 */
export const NEUTRALS = [
  {
    id: 'graphite',
    dark: {
      background: '#0E1116',
      surface: '#161B22',
      'surface-bright': '#1E2530',
      'surface-light': '#21262D',
      'surface-variant': '#2A313C',
      'on-surface-variant': '#C9D1D9',
    },
    light: {
      background: '#F5F7FA',
      surface: '#FFFFFF',
      'surface-bright': '#FFFFFF',
      'surface-light': '#EEF1F6',
      'surface-variant': '#E7EAF0',
      'on-surface-variant': '#3B4252',
    },
  },
  {
    id: 'carbon',
    dark: {
      background: '#0B0B0C',
      surface: '#151516',
      'surface-bright': '#1F1F21',
      'surface-light': '#1A1A1C',
      'surface-variant': '#2B2B2E',
      'on-surface-variant': '#CFCFD2',
    },
    light: {
      background: '#F4F4F5',
      surface: '#FFFFFF',
      'surface-bright': '#FFFFFF',
      'surface-light': '#ECECEE',
      'surface-variant': '#E3E3E6',
      'on-surface-variant': '#3F3F46',
    },
  },
  {
    id: 'midnight',
    dark: {
      background: '#080D1A',
      surface: '#101728',
      'surface-bright': '#182238',
      'surface-light': '#141C30',
      'surface-variant': '#233048',
      'on-surface-variant': '#C3CEE4',
    },
    light: {
      background: '#F2F5FC',
      surface: '#FFFFFF',
      'surface-bright': '#FFFFFF',
      'surface-light': '#E9EFFA',
      'surface-variant': '#DFE7F6',
      'on-surface-variant': '#33415C',
    },
  },
  {
    id: 'forest',
    dark: {
      background: '#0A1210',
      surface: '#111C19',
      'surface-bright': '#182722',
      'surface-light': '#14211D',
      'surface-variant': '#22352E',
      'on-surface-variant': '#C4D5CD',
    },
    light: {
      background: '#F2F7F4',
      surface: '#FFFFFF',
      'surface-bright': '#FFFFFF',
      'surface-light': '#E9F2ED',
      'surface-variant': '#DEEBE4',
      'on-surface-variant': '#33473E',
    },
  },
  {
    id: 'warm',
    dark: {
      background: '#12100E',
      surface: '#1B1815',
      'surface-bright': '#26221E',
      'surface-light': '#201C18',
      'surface-variant': '#332D27',
      'on-surface-variant': '#D8CEC4',
    },
    light: {
      background: '#FAF7F3',
      surface: '#FFFFFF',
      'surface-bright': '#FFFFFF',
      'surface-light': '#F3EEE8',
      'surface-variant': '#EBE4DB',
      'on-surface-variant': '#4A4239',
    },
  },
];

/**
 * Font stacks, not fonts.
 *
 * The app bundles one webfont — the icon set — and nothing else, so offering
 * "Roboto" would name a face that is simply absent on most machines and fall
 * back silently to something else. Every stack here starts with a face the
 * platform ships.
 */
export const FONT_FAMILIES = [
  {
    id: 'system',
    stack:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, "Helvetica Neue", Arial, sans-serif',
  },
  { id: 'grotesk', stack: '"Helvetica Neue", Helvetica, Arial, "Segoe UI", sans-serif' },
  { id: 'serif', stack: 'Georgia, "Times New Roman", "Noto Serif", serif' },
  { id: 'mono', stack: '"SF Mono", Menlo, Consolas, "DejaVu Sans Mono", monospace' },
];

const byId = (list, id) => list.find((item) => item.id === id) ?? list[0];

/**
 * The alpha a medium-emphasis text style carries before the element opacity.
 *
 * Vuetify's own number, and the multiplier half of the composite `over`
 * describes: the rendered alpha of a caption is this times
 * `medium-emphasis-opacity`, which at the standard level is 0.87 × 0.76.
 */
const HIGH_EMPHASIS_ALPHA = 0.87;

/**
 * The pairs worth measuring, and what each one is in the interface.
 *
 * Not every colour against every other — that is a matrix nobody reads. These
 * eight are the pairs a person actually looks at, and between them they cover
 * every choice this pane offers: the accent (`onPrimary`), the colour derived
 * from it (`onSecondary`), the neutral family (`body`), the contrast level
 * (`caption`) and the status palette (the last four).
 *
 * `caption` is the one that earns the table. It is the only row whose
 * foreground is not a theme colour but a composite, it is the row that has
 * failed WCAG twice in this codebase's history, and it is invisible to anyone
 * reading the palette.
 */
export const AUDIT_PAIRS = [
  { id: 'body', fg: 'on-surface', bg: 'surface' },
  { id: 'caption', fg: 'on-surface', bg: 'surface', emphasised: true },
  { id: 'onPrimary', fg: 'on-primary', bg: 'primary' },
  { id: 'onSecondary', fg: 'on-secondary', bg: 'secondary' },
  { id: 'success', fg: 'success-text', bg: 'surface' },
  { id: 'warning', fg: 'warning-text', bg: 'surface' },
  { id: 'error', fg: 'error-text', bg: 'surface' },
  { id: 'info', fg: 'info-text', bg: 'surface' },
];

/**
 * What the chosen colours actually measure, pair by pair.
 *
 * Vuetify Studio has no equivalent, and the omission is not an oversight so
 * much as a different bet: it derives its palette through Material's contrast
 * curves and trusts them. This application derives half its palette by
 * *measuring* — `readable` exists for exactly that — so the ratios are already
 * being computed, and putting them on screen costs nothing and turns a
 * settings page into something a person can check rather than trust.
 *
 * Takes resolved colours rather than a settings object because the `on-*`
 * half of every pair is generated by Vuetify, not by this file: the caller
 * hands over `theme.computedThemes.value[name].colors`, which is the only
 * place `on-primary` exists.
 */
export function auditTheme(colors, contrast = 'standard') {
  const level = CONTRAST_LEVELS[contrast] ?? CONTRAST_LEVELS.standard;

  return AUDIT_PAIRS.map(({ id, fg, bg, emphasised }) => {
    const background = colors?.[bg];
    const foreground = emphasised
      ? over(colors?.[fg], background, HIGH_EMPHASIS_ALPHA * level.emphasis)
      : colors?.[fg];

    const ratio = ratioOf(foreground, background);
    return {
      id,
      ratio,
      // `null` rather than a failing grade for a pair that could not be read
      // at all: a theme missing a role is a different problem from a theme
      // whose roles are too close together, and grading it "fail" would file
      // the two under one heading.
      grade: ratio === null ? null : ratio >= AAA_TEXT ? 'aaa' : ratio >= AA_TEXT ? 'aa' : 'fail',
    };
  });
}

/**
 * The theme a settings object describes, as a value rather than an effect.
 *
 * Split out of `applyAppearance` so one derivation can answer two questions.
 * `applyAppearance` asks for the theme to *install*; the settings page asks for
 * a theme to *show* — registered under its own name and rendered inside a
 * `<v-theme-provider>`, which is how the light theme can be put on screen
 * without the dark one being taken off it. Before this the only way to see the
 * other variant was to switch to it, which meant a user on dark chose their
 * light palette blind.
 *
 * Reads the live theme as its base rather than a stored copy, and that is safe
 * for the reason `applyAppearance` is idempotent and total: every field the
 * appearance controls is written on every call, so a base that has already been
 * themed carries nothing forward.
 */
export function buildTheme(appearance, name, systemAccent = null) {
  const base = vuetify.theme.themes.value[name];
  if (!base) return null;

  const a = { ...DEFAULT_APPEARANCE, ...appearance };
  const primary = (a.useSystemAccent && systemAccent) || a.primary;
  const neutral = byId(NEUTRALS, a.neutral);
  const status = byId(STATUS_PALETTES, a.statusPalette);
  const level = CONTRAST_LEVELS[a.contrast] ?? CONTRAST_LEVELS.standard;

  const colors = {
    ...base.colors,
    primary,
    // Derived rather than declared — see `harmonise`. On both themes, for the
    // same reason `primary` is: the accent is one choice, not two.
    secondary: harmonise(primary, a.harmony),
    ...neutral[name],
    ...status.colors,
  };

  // A second, readable copy of each status colour, for the times it is used as
  // *text* rather than as a fill.
  //
  // The palette is chosen for what a dot and a chip have to do, and it is good
  // at that: `#4CAF50` reads as running at a glance. As small text on a card it
  // is 2.77:1, against WCAG AA's 4.5 — a failure axe found ten of on the
  // project page. Darkening the palette itself would be the wrong fix twice
  // over: a darker dot is a worse dot, and `colorblind` is Okabe-Ito, whose
  // values are the entire reason somebody picks it.
  //
  // So the fill keeps its colour and the text gets a variant, derived against
  // this theme's own surface — which the neutral palette above may just have
  // changed — and against the target the contrast level asks for.
  // `global.css` is where `.text-success` is pointed at it.
  for (const [role, value] of Object.entries(status.colors)) {
    colors[`${role}-text`] = readable(value, colors.surface, level.target);
  }

  return {
    dark: base.dark,
    colors,
    variables: {
      ...base.variables,
      // Contrast is a pair of opacities rather than a second palette: Vuetify
      // renders secondary text and every divider through these two, so raising
      // them lifts the whole interface without inventing new colours.
      //
      // The *standard* stop is the number that matters and it was 0.68, which
      // does not meet WCAG AA. A field label composites its own colour alpha
      // (0.87) with this, so `rgba(27,32,38,.87)` at 0.68 lands on `#787b7f` —
      // 4.25:1 on white and 4.15:1 on the search field's `#f7f7f7`, both under
      // 4.5. Measured by axe in a real engine once the run stopped being scoped
      // to `#app`, which is where the label lives, and reproducible now with
      // `over` in `contrast.js`.
      //
      // 0.76 is 5.29:1 and 5.15:1 respectively, which is why it is the standard
      // stop rather than the accessible one: the two levels above it are an
      // enhancement on a baseline that already passes. See `CONTRAST_LEVELS`.
      //
      // These two assignments are the ones that decide it. `plugins/vuetify.js`
      // declares the same variables and is overwritten here on every apply — a
      // value set in two places where one of them always wins is how the first
      // fix of this missed.
      'medium-emphasis-opacity': level.emphasis,
      'border-opacity': level.border,
    },
  };
}

/**
 * What each setting is allowed to be.
 *
 * Every rule is written against the list that *offers* the value rather than
 * against a copy of it — `NEUTRALS.some(...)`, not a literal set of five ids —
 * so a palette added to this file is importable the moment it exists, and a
 * palette removed stops being accepted without anybody remembering to come
 * here.
 *
 * The gate exists because `parseAppearance` takes text a person pasted. Without
 * it, "import a look" is a write of arbitrary JSON into preferences.json: a
 * `radius` of 9,000, a `density` string Vuetify has never heard of, a `primary`
 * that is not a colour and lands in the stylesheet verbatim. None of those is
 * an attack — they are what a truncated copy-paste looks like — and all of them
 * produce an application that cannot be got back to normal from its own
 * settings page.
 *
 * `tests/appearance-harmony.spec.js` holds this against `DEFAULT_APPEARANCE`
 * key for key, which is what stops the next setting being added without one.
 */
const inRange = (key) => (v) => Number.isInteger(v) && v >= RANGES[key].min && v <= RANGES[key].max;

const isBool = (v) => typeof v === 'boolean';

const oneOf = (list) => (v) => list.includes(v);

const hasId = (list) => (v) => list.some((item) => item.id === v);

export const APPEARANCE_RULES = {
  theme: oneOf(['system', 'light', 'dark']),
  // Held to the grid the pane offers rather than to "is a hex". A colour from
  // outside it is one no swatch can show as selected, so importing it would
  // leave the page unable to say what it had just done.
  primary: oneOf(PRIMARY_SWATCHES),
  harmony: oneOf(HARMONIES),
  neutral: hasId(NEUTRALS),
  radius: inRange('radius'),
  fontFamily: hasId(FONT_FAMILIES),
  fontSize: inRange('fontSize'),
  contrast: (v) => typeof v === 'string' && v in CONTRAST_LEVELS,
  rtl: isBool,
  statusPalette: hasId(STATUS_PALETTES),
  density: oneOf(['compact', 'comfortable', 'default']),
  darkConsoles: isBool,
  reduceMotion: isBool,
  useSystemAccent: isBool,
  // 'app' or a BCP 47 tag. Matched loosely on purpose: a locale pack names its
  // own tag, and this file cannot hold a list of languages it has never heard
  // of — the same reason `locale_pack_write` validates the shape rather than
  // the language.
  consoleLocale: (v) =>
    typeof v === 'string' && /^(app|[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*)$/.test(v),
};

/**
 * A pasted look, read back into settings.
 *
 * Returns `{ values, ignored }` rather than a bare object or `null`, because
 * the two failure modes are different and the page has to be able to say which
 * happened. A file that is not JSON at all, or is JSON but holds nothing this
 * application recognises, is a paste that went wrong — `values` is `null` and
 * the page says so. A file with fifteen good fields and one from a newer
 * release is a *usable* look, and dropping that field silently is how somebody
 * ends up wondering why their radius did not come across.
 *
 * Migrated before it is filtered, so a look exported by a build that still had
 * `highContrast` arrives as `contrast` and passes the gate rather than being
 * reported as an ignored field.
 */
export function parseAppearance(text) {
  let raw;
  try {
    raw = JSON.parse(String(text ?? ''));
  } catch {
    return { values: null, ignored: [] };
  }
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    return { values: null, ignored: [] };
  }

  const candidate = migrate(raw);
  const values = {};
  const ignored = [];

  for (const [key, value] of Object.entries(candidate)) {
    const rule = APPEARANCE_RULES[key];
    if (rule && rule(value)) values[key] = value;
    else ignored.push(key);
  }

  // Nothing recognised is not a partial import, it is the wrong file. Merging
  // it over the defaults would silently reset the look somebody was trying to
  // add to.
  if (Object.keys(values).length === 0) return { values: null, ignored };

  return { values: { ...DEFAULT_APPEARANCE, ...values }, ignored };
}

/**
 * The current look as a `createVuetify` call, for a project that is not this
 * one.
 *
 * The one thing Vuetify Studio does that this application had no answer to. A
 * look tuned here — the accent, its derived partner, a neutral family, the
 * status palette lifted to the contrast level chosen — is a theme somebody may
 * well want in the web app beside the desktop one, and retyping thirty hex
 * values out of a screenshot is the alternative this replaces.
 *
 * Both themes, because `buildTheme` produces both and a snippet carrying only
 * the one that happens to be current is a snippet that breaks the moment its
 * reader toggles.
 */
export function themeSnippet(appearance, systemAccent = null) {
  const a = { ...DEFAULT_APPEARANCE, ...appearance };

  const themes = {};
  for (const name of ['light', 'dark']) {
    const built = buildTheme(a, name, systemAccent);
    if (built) themes[name] = built;
  }

  // Indented to sit inside the call rather than left flush: this is copied
  // straight into a file, and the alternative is asking the reader to reformat
  // eighty lines by hand.
  const body = JSON.stringify(themes, null, 2).split('\n').join('\n    ');

  return [
    "import { createVuetify } from 'vuetify';",
    '',
    'export default createVuetify({',
    '  theme: {',
    `    defaultTheme: '${a.theme}',`,
    `    themes: ${body},`,
    '  },',
    '});',
    '',
  ].join('\n');
}

/**
 * Push a settings object into the running app.
 *
 * Idempotent and total: every field is written on every call, so applying a
 * partial change and applying the whole object land in the same place. That is
 * what lets the settings page call this on every keystroke of a slider without
 * tracking what actually changed.
 */
export function applyAppearance(appearance, systemAccent = null) {
  const a = { ...DEFAULT_APPEARANCE, ...appearance };

  vuetify.theme.change(a.theme);

  // Assigned onto the live objects rather than replaced: `theme.themes` holds
  // reactive theme records that Vuetify's own computed reads through, and
  // swapping the record wholesale is a different identity for every watcher
  // downstream of it. `buildTheme` returns the values; this writes them.
  for (const name of ['light', 'dark']) {
    const built = buildTheme(a, name, systemAccent);
    if (!built) continue;

    Object.assign(vuetify.theme.themes.value[name].colors, built.colors);
    Object.assign(vuetify.theme.themes.value[name].variables, built.variables);
  }

  // `global`, not per-component: Vuetify reads defaults through a proxy on
  // props, so only components that actually declare `density` ever see it —
  // there are no stray attributes on the ones that do not.
  vuetify.defaults.value = {
    ...vuetify.defaults.value,
    global: { ...vuetify.defaults.value.global, density: a.density },
  };

  // An attribute rather than a class: the stylesheet keys off it, and it is one
  // value to read back when debugging why nothing on screen is animating.
  document.documentElement.dataset.reduceMotion = String(!!a.reduceMotion);

  const root = document.documentElement.style;
  root.setProperty('--app-radius', `${a.radius}px`);
  root.setProperty('--app-font-family', byId(FONT_FAMILIES, a.fontFamily).stack);
  root.setProperty('--app-font-size', `${a.fontSize}px`);

  // Layout direction is Vuetify's, not a stylesheet of ours: every component
  // mirrors itself from it.
  //
  // Written into the per-locale map rather than onto `isRtl`, which is a
  // computed derived from that map — assigning to it is silently discarded.
  //
  // A locale whose pack **declared** a direction keeps it; the switch decides
  // for everything else, so the preference still survives switching between the
  // two built-in languages. See `packDirections` for why a language outranks a
  // preference here and only here: Arabic reads right to left whether or not
  // anybody chose it, and before this an Arabic pack rendered left to right
  // until its reader found a switch that then mirrored English as well.
  for (const name of i18n.global.availableLocales) {
    vuetify.locale.rtl.value[name] = rtlFor(name, a.rtl);
  }

  // And on the document, which is not the same element and not a duplicate of
  // the line above.
  //
  // Vuetify's flag sets `direction: rtl` on `.v-application` — everything drawn
  // *inside* the app root mirrors, and that is what the existing test measures.
  // What it does not reach is everything drawn outside it, and in this
  // application that is not an edge: Vuetify's overlay container is a sibling
  // of `#app`, so every dialog, side sheet, menu and tooltip was still laid out
  // left-to-right with the rest of the window mirrored. Measured in a real
  // engine — `direction: ltr` on the overlay container while the app root said
  // `rtl`.
  //
  // `dir` on `<html>` is inherited by the whole document, so it settles the
  // overlays, the scrollbar side and the paragraph direction of any text that
  // is not inside a Vuetify component. Written explicitly in both directions
  // rather than removed when off: an attribute that is sometimes absent is one
  // a user stylesheet or a screen reader has to guess about.
  //
  // The **active** locale's direction, not the switch's. They were the same
  // value while one flag decided for everything; now a pack can disagree with
  // it, and an Arabic window whose overlays laid out left to right would be the
  // exact failure the paragraph above describes, reintroduced one line down.
  if (typeof document !== 'undefined') {
    const rtl = rtlFor(i18n.global.locale.value, a.rtl);
    document.documentElement.setAttribute('dir', rtl ? 'rtl' : 'ltr');

    // And the language, on the same element and for the same reason `dir` is
    // here — except that this one was not merely incomplete, it was **wrong**.
    //
    // `index.html` ships `lang="en"` and nothing ever changed it, so a Turkish
    // window announced itself as English for its whole life. That is WCAG
    // 3.1.1, and it is the criterion everything else about language rests on: a
    // screen reader picks its voice and its pronunciation rules from this
    // attribute, so a Turkish interface was being read out with English
    // phonetics. `ACCESSIBILITY.md` said the interface language "is
    // announced on the document", which was the claim this line makes true.
    //
    // The **active** locale, like `dir` above — including a pack's tag, which
    // is what makes a third language announce itself as itself rather than as
    // one of the two this build was born with.
    document.documentElement.setAttribute('lang', i18n.global.locale.value);
  }
}

/** What a locale's direction is: what its pack said, or what was chosen. */
function rtlFor(locale, chosen) {
  const declared = packDirections[locale];
  return declared ? declared === 'rtl' : chosen;
}
