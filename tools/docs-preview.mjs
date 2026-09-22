#!/usr/bin/env node
/**
 * Serves the built docs site — both locales, one process — under /stackvo/,
 * the way GitHub Pages does. Run with `npm run docs:preview`.
 *
 * ## Why this exists
 *
 * `npm run docs:serve` / `docs:serve:tr` each start a separate mkdocs dev
 * server, on separate ports, because mkdocs only ever builds one `docs_dir`
 * per process (`docs/README.md` explains the split). That's the right tool
 * for editing pages with live reload, but the header's language switcher
 * links to `/stackvo/tr/...` — a path that exists on neither server, because
 * neither one serves the other locale. No amount of running both side by
 * side fixes that: the switcher's link is to a *combined* path that only
 * exists once something publishes both builds under the same origin.
 * Production doesn't hit this because `mkdocs.yml` renders into `site/` and
 * `mkdocs.tr.yml` into `site/tr/`, and GitHub Pages publishes that one tree.
 *
 * This script does the same: `mkdocs build --strict` for both configs (what
 * CI runs), then a small static server for the merged `site/`, prefixed with
 * `/stackvo/` so every absolute link the theme writes — the switcher, the
 * 404 page, the sitemap — resolves exactly as it will once published. There
 * is no live reload; re-run after editing content.
 */
import { execFileSync } from "node:child_process";
import { existsSync, statSync, createReadStream } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const siteDir = join(root, "site");
const prefix = "/stackvo";
const port = Number(process.argv[2]) || 8000;

// The dev setup a fresh checkout follows (docs/README.md) activates a venv
// before calling `mkdocs`; falling back to a project-local `.venv` means
// this also works right after `python3 -m venv .venv && pip install -r
// docs/requirements.txt`, with nothing to activate.
const venvMkdocs = join(root, ".venv", "bin", "mkdocs");
const mkdocs = existsSync(venvMkdocs) ? venvMkdocs : "mkdocs";

for (const config of ["docs/mkdocs.yml", "docs/mkdocs.tr.yml"]) {
  console.log(`Building ${config}...`);
  execFileSync(mkdocs, ["build", "--strict", "-f", config], {
    cwd: root,
    stdio: "inherit",
  });
}

const types = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".webp": "image/webp",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
  ".txt": "text/plain; charset=utf-8",
};

createServer((req, res) => {
  const url = decodeURIComponent((req.url ?? "/").split("?")[0]);
  if (!url.startsWith(prefix)) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" }).end("Not found");
    return;
  }

  let rel = url.slice(prefix.length) || "/";
  if (rel.endsWith("/")) rel += "index.html";

  let filePath = normalize(join(siteDir, rel));
  let status = 200;
  if (!filePath.startsWith(siteDir) || !existsSync(filePath) || !statSync(filePath).isFile()) {
    filePath = join(siteDir, "404.html");
    status = 404;
  }

  res.writeHead(status, { "Content-Type": types[extname(filePath)] ?? "application/octet-stream" });
  createReadStream(filePath).pipe(res);
}).listen(port, "127.0.0.1", () => {
  console.log(`Serving both locales on http://127.0.0.1:${port}${prefix}/ (Ctrl+C to stop)`);
});
