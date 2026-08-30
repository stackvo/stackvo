#!/usr/bin/env node
/**
 * Measure which `.env` keys anything in this repository actually reads, and
 * check the answer against the `status` labels and `consumers` lists in
 * `contracts/env.schema.json`.
 *
 * This exists because the first pass of that measurement was done by hand and
 * was wrong: it ran from the wrong working directory, so `grep core/` resolved
 * to a path that did not exist and reported zero consumers for every key. That
 * made 17 keys look dead. Two were live. Two keys labelled active were dead.
 *
 * A hand-run grep produces a number that looks like evidence and is not
 * checkable later. This is.
 *
 * ## It stopped being checkable, in the way it was written to prevent
 *
 * `core/` was the Bash and Node implementation, and it is gone. So this tool
 * exited 2 on every run — *"No core/ under … — is that a StackVo checkout?"* —
 * and the field it maintains went with it: **39 of the 45 `consumers` paths in
 * the schema pointed at files that no longer exist**, all of them `core/…`.
 * Six pointed at the live tree. A contract field naming a tree that was deleted
 * is worse than no field: it reads as evidence and is a fossil.
 *
 * It reads this repository now — `src-tauri/src`, `src`, `skeleton`, `tools` —
 * and the same tool that maintains the field is the one `env-consumers.spec.js`
 * runs to fail on it going stale.
 *
 * ## Two rules that are not obvious
 *
 * **Declaration is not consumption.** Every key is written in `config.rs`'s
 * `SETTINGS`/`LEGACY_SERVICES` tables, so counting that file would make every
 * key "active" and the measurement would say nothing. A key that appears only
 * where it is declared is a key nothing reads, which is exactly what `dead`
 * means.
 *
 * **Comments are stripped**, and it is load-bearing rather than tidy: this
 * repository writes down the fact a key is dead *in a comment naming the key*.
 * `skeleton.rs` names `DOCKER_REMOVE_ORPHANS` and `HOST_PORT_ADMINER` in a
 * sentence explaining that they are dead, and an unstripped scan read both as
 * live — the sentence saying so was the evidence against it.
 *
 *   node tools/measure-env-usage.mjs [--root .] [--fix]
 *
 * `npm run env:usage` reads; `npm run env:usage:fix` writes and then runs
 * Prettier over the result, because `JSON.stringify(…, 2)` and Prettier
 * disagree about short arrays and a `--fix` that left the contract unformatted
 * would put that noise in the next unrelated diff.
 */

import { readFileSync, writeFileSync, readdirSync, existsSync, statSync } from 'node:fs';
import { join, dirname, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const SCHEMA = join(HERE, '..', 'contracts', 'env.schema.json');

const argv = process.argv.slice(2);
const fix = argv.includes('--fix');
const rootFlag = argv.indexOf('--root');
const ROOT = resolve(
  rootFlag !== -1 ? argv[rootFlag + 1] : process.env.STACKVO_ROOT || join(HERE, '..')
);

/** The trees a setting can be read from. */
export const ROOTS = ['src-tauri/src', 'src', 'skeleton', 'tools'];

/**
 * Where the keys are declared, which is not where they are read.
 *
 * See the note above: counting this file would make every key active.
 */
export const DECLARATION = 'src-tauri/src/config.rs';

const SKIP_DIRS = new Set(['node_modules', '.git', 'dist', 'target', 'target-linux']);

/**
 * The UI builds these from a runtime name — `format!("SUPPORTED_LANGUAGES_{key}_VERSIONS")`
 * in `build_catalog` — so no literal appears in the source and a text search
 * cannot find them. Declared rather than measured, and the roadmap's diagnosis
 * of exactly this was wrong once: a key-by-key search found nothing while the
 * chain `.env` → `build_catalog` → `catalogGet` → the picker worked perfectly.
 */
const DYNAMIC = /^SUPPORTED_LANGUAGES_(PHP|PYTHON|GO|RUBY|RUST|NODEJS)_/;
const DYNAMIC_CONSUMER = 'src-tauri/src/commands.rs (composed by build_catalog)';

/** A file with its comments removed. See the note above for why. */
function code(text, path) {
  let out = text;
  if (/\.(rs|js|mjs|ts|vue|css)$/.test(path)) {
    out = out.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1');
  }
  if (/\.(vue|html|md)$/.test(path)) out = out.replace(/<!--[\s\S]*?-->/g, '');
  if (/\.(ya?ml|conf|sh)$/.test(path)) out = out.replace(/(^|\s)#.*$/gm, '$1');
  return out;
}

function walk(dir, acc = []) {
  if (!existsSync(dir)) return acc;
  for (const entry of readdirSync(dir)) {
    if (SKIP_DIRS.has(entry)) continue;
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) walk(full, acc);
    else acc.push(full);
  }
  return acc;
}

/** Every readable file under the roots, comments stripped, keyed by repo path. */
export function sources(root = ROOT) {
  const out = [];
  for (const dir of ROOTS) {
    for (const full of walk(join(root, dir))) {
      const path = relative(root, full).split('\\').join('/');
      if (path === DECLARATION) continue;
      let text = '';
      try {
        text = readFileSync(full, 'utf8');
      } catch {
        /* binary */
      }
      out.push({ path, text: code(text, path) });
    }
  }
  return out;
}

/** What reads `key`, as repo-relative paths. */
export function consumersOf(key, files) {
  if (DYNAMIC.test(key)) return [DYNAMIC_CONSUMER];
  return files.filter((f) => f.text.includes(key)).map((f) => f.path);
}

/**
 * The whole comparison, as data, so a test can assert on it without shelling out.
 *
 * `alias` and `derived` are excluded from the active/dead judgement rather than
 * forced into it: an alias is a spelling of another key and a derived value is
 * computed, so "does anything read this literal" is not the question either one
 * answers.
 */
export function audit(root = ROOT) {
  const schema = JSON.parse(readFileSync(SCHEMA, 'utf8'));
  const files = sources(root);
  const rows = [];

  for (const group of Object.values(schema.groups)) {
    for (const [key, spec] of Object.entries(group)) {
      if (key === '_note' || typeof spec !== 'object') continue;
      const found = consumersOf(key, files);
      const measured = found.length ? 'active' : 'dead';
      const labelled = spec.status;
      const judged = labelled === 'alias' || labelled === 'derived' ? null : measured;
      const labelledActive = labelled === 'active' || labelled === 'conflicting';
      rows.push({
        key,
        labelled,
        measured,
        found,
        agrees: judged === null || (judged === 'active') === labelledActive,
        spec,
      });
    }
  }

  return { schema, rows, files: files.length };
}

// ---------------------------------------------------------------- run

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  const { schema, rows, files } = audit();
  const mismatches = rows.filter((r) => !r.agrees);

  if (fix) {
    for (const row of rows) {
      if (!row.agrees && row.labelled !== 'alias' && row.labelled !== 'derived') {
        row.spec.status = row.measured;
      }
      if (row.found.length) row.spec.consumers = row.found;
      else delete row.spec.consumers;
    }
    writeFileSync(SCHEMA, JSON.stringify(schema, null, 2) + '\n');
  }

  console.log(`\nenv usage — ${rows.length} keys against ${files} files under ${ROOT}\n`);

  if (!mismatches.length) {
    console.log('  every status label matches what this repository actually reads\n');
    process.exit(0);
  }

  for (const m of mismatches) {
    console.log(
      `  ${m.key.padEnd(36)} labelled ${m.labelled}, measured ${m.measured} (${m.found.length} files)`
    );
  }
  console.log(
    `\n  ${mismatches.length} mismatch(es)${fix ? ' — schema updated' : ' — rerun with --fix to update the schema'}\n`
  );
  process.exit(fix ? 0 : 1);
}
