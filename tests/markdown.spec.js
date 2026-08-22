import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { renderMarkdown } from '../src/lib/markdown.js';

/**
 * The renderer, against the documents it was written for.
 *
 * It is a hundred lines instead of a hundred kilobytes of `markdown-it`, and
 * the trade only holds while it renders what the help documents actually
 * contain. So the last test here reads a real document rather than a fixture:
 * a syntax somebody starts using that this cannot parse shows up as a page with
 * the source text in it, which is exactly what a fixture would never catch.
 */
describe('the markdown renderer', () => {
  it('renders the blocks the documents are written in', () => {
    expect(renderMarkdown('# Title')).toBe('<h1>Title</h1>');
    expect(renderMarkdown('## Controls')).toBe('<h2>Controls</h2>');
    expect(renderMarkdown('one\ntwo')).toBe('<p>one two</p>');
    expect(renderMarkdown('- a\n- b')).toBe('<ul><li>a</li><li>b</li></ul>');
    expect(renderMarkdown('**bold** and `code`')).toBe(
      '<p><strong>bold</strong> and <code>code</code></p>'
    );
  });

  it('renders a table of fields', () => {
    const html = renderMarkdown('| Field | Means |\n| --- | --- |\n| **Name** | the name |');
    expect(html).toContain('<th>Field</th>');
    expect(html).toContain('<td><strong>Name</strong></td>');
    expect(html).toContain('<td>the name</td>');
  });

  /**
   * Markup in a document must never become markup in the page. The documents
   * are files on disk, and files on disk get edited.
   */
  it('escapes everything before it writes a single tag', () => {
    expect(renderMarkdown('<script>alert(1)</script>')).toBe(
      '<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>'
    );
    expect(renderMarkdown('| a |\n| --- |\n| <img onerror=x> |')).toContain(
      '&lt;img onerror=x&gt;'
    );
    expect(renderMarkdown('`<b>`')).toBe('<p><code>&lt;b&gt;</code></p>');
  });

  /** A link that runs code is not a link. */
  it('links only http and https', () => {
    expect(renderMarkdown('[x](https://a.b)')).toContain('href="https://a.b"');
    expect(renderMarkdown('[x](javascript:alert(1))')).not.toContain('<a ');
    expect(renderMarkdown('[x](file:///etc/passwd)')).not.toContain('<a ');
  });

  it('reads a document this repository actually ships', () => {
    const html = renderMarkdown(readFileSync('docs/help/en/project-tunnel.md', 'utf8'));

    expect(html).toContain('<h1>Share</h1>');
    expect(html).toContain('<table>');
    expect(html).toContain('<li>');
    // Nothing was left as raw source: a `|` or a `##` in the output means a
    // block this renderer walked past.
    expect(html).not.toMatch(/<p>[^<]*\|/);
    expect(html).not.toMatch(/<p>#/);
  });
});
