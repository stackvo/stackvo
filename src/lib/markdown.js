/**
 * The subset of markdown the help documents are written in, rendered to HTML.
 *
 * ## Why not a library
 *
 * `markdown-it` and the plugins these documents would need are around 100 KB,
 * and the bundle budget is a gate in this repository (`tools/check-bundle.mjs`).
 * What is being parsed is not arbitrary markdown from the internet: it is a
 * directory of files written in this repository, in a syntax listed below, and
 * `tests/help-topics.spec.js` reads them all. A hundred lines that handle
 * exactly that is a better trade than a parser that handles footnotes.
 *
 * ## What it handles
 *
 * `#`–`###` headings, paragraphs, `-` lists, `|` tables, fenced code, `**bold**`,
 * `` `code` `` and `[text](url)`. Anything else is passed through as text.
 *
 * ## Escaping
 *
 * Every character of the source is escaped before any tag is written, and the
 * tags are then produced by this file rather than copied from the source. So a
 * document containing `<script>` renders the four characters — there is no path
 * by which markup in a document becomes markup in the page, which matters
 * because the documents are files on disk and files on disk get edited.
 */

const escape = (text) =>
  text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** Inline spans, applied to already-escaped text. */
function inline(text) {
  return (
    escape(text)
      // Code first: what is inside a span of code is not markup.
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      // Only http(s), and only as a whole href — a `javascript:` URL in a
      // document must not become a link that runs it. Anything else is left as
      // the characters somebody typed.
      .replace(
        /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g,
        (_, text, href) => `<a href="${href}" rel="noreferrer noopener" target="_blank">${text}</a>`
      )
  );
}

/** One `| a | b |` row, split on the pipes that separate cells. */
const cells = (line) =>
  line
    .replace(/^\s*\|/, '')
    .replace(/\|\s*$/, '')
    .split('|')
    .map((cell) => cell.trim());

const isDivider = (line) => /^\s*\|?[\s:-]*-[\s:|-]*$/.test(line) && line.includes('-');

export function renderMarkdown(source) {
  const lines = String(source ?? '').split(/\r?\n/);
  const out = [];
  let i = 0;

  const paragraph = [];
  const flush = () => {
    if (paragraph.length) {
      out.push(`<p>${inline(paragraph.join(' '))}</p>`);
      paragraph.length = 0;
    }
  };

  while (i < lines.length) {
    const line = lines[i];

    // ---- fenced code ----------------------------------------------------
    if (/^\s*```/.test(line)) {
      flush();
      const body = [];
      i += 1;
      while (i < lines.length && !/^\s*```/.test(lines[i])) {
        body.push(lines[i]);
        i += 1;
      }
      i += 1;
      out.push(`<pre><code>${escape(body.join('\n'))}</code></pre>`);
      continue;
    }

    // ---- an indented block, kept as written ------------------------------
    if (/^ {4}\S/.test(line)) {
      flush();
      const body = [];
      while (i < lines.length && (/^ {4}/.test(lines[i]) || !lines[i].trim())) {
        body.push(lines[i].slice(4));
        i += 1;
      }
      out.push(`<pre><code>${escape(body.join('\n').replace(/\s+$/, ''))}</code></pre>`);
      continue;
    }

    // ---- heading ---------------------------------------------------------
    const heading = /^(#{1,3})\s+(.*)$/.exec(line);
    if (heading) {
      flush();
      const level = heading[1].length;
      out.push(`<h${level}>${inline(heading[2])}</h${level}>`);
      i += 1;
      continue;
    }

    // ---- table -----------------------------------------------------------
    if (line.trim().startsWith('|') && isDivider(lines[i + 1] ?? '')) {
      flush();
      const head = cells(line);
      i += 2;
      const body = [];
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        body.push(cells(lines[i]));
        i += 1;
      }
      out.push(
        '<table><thead><tr>' +
          head.map((c) => `<th>${inline(c)}</th>`).join('') +
          '</tr></thead><tbody>' +
          body
            .map((row) => '<tr>' + row.map((c) => `<td>${inline(c)}</td>`).join('') + '</tr>')
            .join('') +
          '</tbody></table>'
      );
      continue;
    }

    // ---- list ------------------------------------------------------------
    if (/^\s*-\s+/.test(line)) {
      flush();
      const items = [];
      while (i < lines.length && (/^\s*-\s+/.test(lines[i]) || /^\s{2,}\S/.test(lines[i]))) {
        if (/^\s*-\s+/.test(lines[i])) items.push(lines[i].replace(/^\s*-\s+/, ''));
        // A wrapped continuation line belongs to the item above it.
        else items[items.length - 1] += ` ${lines[i].trim()}`;
        i += 1;
      }
      out.push('<ul>' + items.map((item) => `<li>${inline(item)}</li>`).join('') + '</ul>');
      continue;
    }

    // ---- blank ends a paragraph -----------------------------------------
    if (!line.trim()) {
      flush();
      i += 1;
      continue;
    }

    paragraph.push(line.trim());
    i += 1;
  }

  flush();
  return out.join('\n');
}
