/**
 * A terminal frame, turned into something a browser can draw.
 *
 * `stackvo tui` is a terminal program: there is no window to shoot, only the
 * bytes it writes to a terminal, and those are text with ANSI escapes in it.
 * `tui::draw` builds one frame as a string — that is why it returns rather
 * than prints, and why `examples/tui_frame.rs` can hand the frame to this file
 * without a pty, a Docker socket or a terminal in raw mode.
 *
 * What is here is the reading of that string. It is deliberately not a
 * terminal emulator: `draw` uses six SGR codes, `\r\n`, home, clear-to-end and
 * clear-below, and this reads exactly those. Anything it does not know is
 * dropped rather than guessed at, and the test says which sequences it knows.
 *
 * The picture itself is then `tools/screenshots.mjs`'s job, the same way the
 * other pictures are: HTML into Chromium, at 2x, in the light theme. A PNG
 * encoder written here would be a dependency or a second renderer; the one
 * the tool already has draws type better than either.
 */

/** The eight ANSI colours, by their SGR foreground code. */
const COLOURS = {
  30: 'black',
  31: 'red',
  32: 'green',
  33: 'yellow',
  34: 'blue',
  35: 'magenta',
  36: 'cyan',
  37: 'white',
};

const PLAIN = Object.freeze({ bold: false, dim: false, colour: null });

/**
 * One CSI sequence: `ESC [`, parameters, one letter. Built rather than
 * written as a literal because `no-control-regex` refuses an escape byte in
 * one — and it is right that a control character in a pattern is usually a
 * mistake. Here it is the subject.
 */
const ESC = String.fromCharCode(0x1b);
const CSI = new RegExp(`(${ESC}\\[[0-9;]*[A-Za-z])`);

/**
 * Apply one `m` sequence's parameters to a style.
 *
 * `0` and an empty parameter both reset — `\x1b[m` is the short form — and
 * `22` takes bold and dim away together, because that is what it means: "normal
 * intensity", which neither of them is.
 */
export function sgr(style, params) {
  let next = { ...style };
  for (const code of params.split(';')) {
    if (code === '' || code === '0') next = { ...PLAIN };
    else if (code === '1') next.bold = true;
    else if (code === '2') next.dim = true;
    else if (code === '22') next = { ...next, bold: false, dim: false };
    else if (code === '39') next.colour = null;
    else if (COLOURS[code]) next.colour = COLOURS[code];
    // Anything else — backgrounds, 256-colour, italics — is not something
    // `draw` writes, and a renderer that half-supported it would show a
    // frame the screen never showed.
  }
  return next;
}

/**
 * The frame as lines of styled runs: `[[{ text, bold, dim, colour }, …], …]`.
 *
 * A run is the longest stretch of text drawn in one style, which is what
 * becomes one `<span>`. Cursor movement and erasing are dropped: the frame
 * homes the cursor once, clears to the end of every line and clears below the
 * last, and on a blank surface all three draw nothing.
 */
export function linesOf(frame) {
  const lines = [];
  let runs = [];
  let style = { ...PLAIN };

  // Escapes are one token each; the text between two of them is another.
  // Splitting on a capturing group keeps the escapes in the result, so this
  // is a walk over alternating text and control rather than a byte parser —
  // and a `▸` or `─` is one character to `split`, never a half of one.
  for (const chunk of frame.split(CSI)) {
    if (chunk === '') continue;
    if (chunk.startsWith(`${ESC}[`)) {
      if (chunk.endsWith('m')) style = sgr(style, chunk.slice(2, -1));
      continue;
    }
    // Raw mode wants `\r\n`; a bare `\n` is read the same way so a frame
    // captured through a cooked terminal reads too. A bare `\r` is a return
    // to column zero with nothing drawn after it, which is nothing.
    chunk.split(/\r?\n/).forEach((part, index) => {
      if (index > 0) {
        lines.push(runs);
        runs = [];
      }
      const text = part.replace(/\r/g, '');
      if (text) runs.push({ text, ...style });
    });
  }
  if (runs.length) lines.push(runs);
  return lines;
}

/** How many cells one line takes: characters, not bytes. */
export function columnsOf(line) {
  return line.reduce((sum, run) => sum + [...run.text].length, 0);
}

/** The widest line, which is what the terminal would have to be. */
export function widthOf(lines) {
  return lines.reduce((widest, line) => Math.max(widest, columnsOf(line)), 0);
}

function escapeHtml(text) {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/**
 * The light palette, and why it is light: the other thirty-eight pictures
 * are, and `screenshots.mjs` says a second theme is a second file per page
 * that is always the stale one. A terminal's own colours are whatever the
 * person's terminal has; these are the GitHub light ones, chosen so `ok` and
 * `dim` read at the size the README shows them.
 */
export const PALETTE = {
  background: '#ffffff',
  foreground: '#1f2328',
  dim: '#6e7781',
  green: '#1a7f37',
  yellow: '#9a6700',
  red: '#cf222e',
  blue: '#0969da',
  magenta: '#8250df',
  cyan: '#1b7c83',
  black: '#1f2328',
  white: '#6e7781',
};

/**
 * A page that is only the terminal.
 *
 * `columns` × `rows` cells, in `ch` units so the width is the font's own cell
 * and not a guess at it. The frame is shorter than the terminal, and the rest
 * of the screen is left blank below it rather than cropped: that blank is what
 * the screen looks like at 80×24, and the key hints sit above it, not at the
 * bottom of the window, because `draw` puts them after the list.
 */
export function htmlOf(lines, { columns = 80, rows = 24, title = 'stackvo tui' } = {}) {
  const body = lines
    .map((line) =>
      line
        .map((run) => {
          const classes = [run.bold && 'b', run.dim && 'd', run.colour && `c-${run.colour}`]
            .filter(Boolean)
            .join(' ');
          const text = escapeHtml(run.text);
          return classes ? `<span class="${classes}">${text}</span>` : text;
        })
        .join('')
    )
    .join('\n');

  const colourRules = Object.entries(COLOURS)
    .map(([, name]) => `.c-${name}{color:${PALETTE[name]}}`)
    .join('');

  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>${escapeHtml(title)}</title>
<style>
html,body{margin:0;background:${PALETTE.background}}
#terminal{display:inline-block;padding:14px 18px;background:${PALETTE.background};color:${PALETTE.foreground}}
pre{margin:0;width:${columns}ch;height:${rows * 22}px;overflow:hidden;
font:15px/22px ui-monospace,"SF Mono",Menlo,Consolas,"DejaVu Sans Mono",monospace}
.b{font-weight:600}.d{color:${PALETTE.dim}}${colourRules}
</style></head>
<body><div id="terminal"><pre>${body}</pre></div></body></html>
`;
}
