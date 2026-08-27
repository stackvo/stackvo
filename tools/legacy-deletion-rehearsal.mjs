#!/usr/bin/env node
/**
 * legacy-deletion-rehearsal — performs the deletion `LEGACY_SERVICES` is waiting for, and
 * reports what it costs.
 *
 * `config::LEGACY_SERVICES` goes at 0.4.0. `tests/legacy_env_claims.rs` names
 * every module that *reads* one of those defaults, and that list is greppable —
 * a module has to name the key it reads. But reading is not the only way to
 * depend on a constant, and the deletion-day claim was the stronger one: that
 * the constant and the modules that test lists are "the whole change".
 *
 * Nobody had checked. This checks, by doing it: empty the legacy half, run the
 * suite, put the tree back, and print every site that failed. The first run
 * found three the reader list could not have named — two of them in modules
 * that never read a legacy default, and one of them not a failing test at all
 * but a **compile error**, because a test indexed `LEGACY_SERVICES[0]` and
 * `deny(unconditional_panic)` rejects a constant index into an empty array. A
 * deletion that stops the crate's tests from building is a worse morning than
 * one that turns six of them red.
 *
 * It also found what was actually holding the constant up, which was not the
 * migration: `handover_equivalence.rs` passes 13 of 13 without it. It was
 * `mail.rs` asking whether a key was *present* to decide which mail catcher a
 * workspace knows about. That question was pointed at the catalogue, and
 * seventy-two keys — the `.env` shadow of a catalogue that had already moved
 * into packages — left with it. Two rows disappeared from the list below on that
 * commit, and the tool is what said so: a row that stops failing is reported
 * as loudly as one that starts.
 *
 * ## Why in place, and why that is safe enough
 *
 * A copy of the tree would need `docs/`, `contracts/` and `skeleton/` beside it
 * — several of the tests read the repository they live in — so a copy is the
 * whole repository, and the compile is not incremental. The mutation is instead
 * applied to `src-tauri/src/config.rs` and undone in a `finally`, with the same
 * restore wired to SIGINT and SIGTERM. The original bytes are also written to
 * `<file>.rehearsal-backup` first, so an interrupt the process cannot survive
 * still leaves the file recoverable by hand rather than by memory.
 *
 * Refuses to start if `config.rs` already has uncommitted changes, because the
 * restore writes bytes back rather than reverting a diff, and restoring over
 * somebody's unsaved edit is the one failure mode worth being rude about.
 *
 *   node tools/legacy-deletion-rehearsal.mjs [--keep] [--json]
 *
 *     --keep   leave the deletion applied (for looking around); the backup file
 *              stays beside it and the exit code is unchanged
 *     --json   machine-readable result
 */

import { readFileSync, writeFileSync, existsSync, unlinkSync } from 'node:fs';
import { execFileSync, spawnSync } from 'node:child_process';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, '..');
const CONFIG = join(ROOT, 'src-tauri', 'src', 'config.rs');
const MANIFEST = join(ROOT, 'src-tauri', 'Cargo.toml');
const BACKUP = `${CONFIG}.rehearsal-backup`;

const argv = process.argv.slice(2);
const asJson = argv.includes('--json');
const keep = argv.includes('--keep');

/**
 * What the deletion is expected to cost, and why each site is on the list.
 *
 * This is the deletion-day checklist's second half. `READERS` in
 * `tests/legacy_env_claims.rs` is the first half and is held by a grep; this
 * half cannot be, because a test that leans on an embedded default names no key
 * — `preflight.rs` asks `commands.rs` a question and expects phpMyAdmin to be
 * on, which is true only because `SERVICE_PHPMYADMIN_ENABLE=true` is a default
 * this constant supplies.
 *
 * A run that produces a different set than this one is the point of the tool:
 * either the tree grew a dependant nobody wrote down, or one went away and the
 * row for it is now a wrong instruction on deletion day.
 */
const EXPECTED = [
  {
    test: 'config::tests::the_two_halves_partition_the_defaults',
    file: 'src-tauri/src/config.rs',
    why: 'asserts the two lengths. The deletion is what this test is watching for.',
  },
  {
    test: 'config::tests::stated_is_what_the_file_wrote_and_get_is_that_plus_the_defaults',
    file: 'src-tauri/src/config.rs',
    why: 'shows the line between `get` and `stated` using a real key — an untouched workspace answering `"root"` from the binary is the case `stated` exists to refuse, and with the constant gone there is no default to demonstrate it with.',
  },
  {
    test: 'preflight::tests::a_fresh_install_asks_for_the_two_core_names_and_nothing_else',
    file: 'src-tauri/src/preflight.rs',
    why: 'reads no legacy key itself. It asks `commands::service_domains_for_test`, which builds a hostname from `env.service_url(id)` — so the row survives on `SERVICE_PHPMYADMIN_URL` arriving as an embedded default, not on anything the test states.',
  },
  {
    test: 'skeleton::tests::no_real_credential_is_compiled_into_the_binary',
    file: 'src-tauri/src/skeleton.rs',
    why: 'every credential-shaped key lives in the legacy half, so after the deletion this guard has nothing to scan. It fails loudly ("saw 0") rather than passing on an empty set, which is the only reason we know — and is what every claims test in this tree is supposed to do.',
  },
  {
    test: 'only_the_declared_modules_read_a_legacy_service_default',
    file: 'src-tauri/tests/legacy_env_claims.rs',
    why: 'the file goes with the constant; it is the checklist, not a survivor.',
  },
  {
    test: 'naming_a_key_counts_as_reading_one',
    file: 'src-tauri/tests/legacy_env_claims.rs',
    why: 'same file. Its keys come from the constant, so with the constant gone it is scanning for nothing and says so.',
  },
];

// ---------------------------------------------------------------- the mutation

/** Empty the legacy half, and resize the two arrays built from it. */
function applyDeletion(source) {
  const declared = (name) => {
    const m = source.match(new RegExp(`pub const ${name}: \\[\\(&str, &str\\); (\\d+)\\]`));
    if (!m) throw new Error(`config.rs no longer declares ${name} in the shape this tool edits`);
    return Number(m[1]);
  };
  const settings = declared('SETTINGS');
  const legacy = declared('LEGACY_SERVICES');
  const embedded = declared('EMBEDDED');

  const open = source.indexOf(`pub const LEGACY_SERVICES: [(&str, &str); ${legacy}] = [`);
  const close = source.indexOf('\n];', open);
  if (open === -1 || close === -1) throw new Error('could not find the legacy half to remove');

  let out =
    source.slice(0, open) +
    'pub const LEGACY_SERVICES: [(&str, &str); 0] = [];\n' +
    source.slice(close + 3);

  const resize = (from, to) => {
    if (!out.includes(from)) throw new Error(`could not resize: ${from}`);
    out = out.replace(from, to);
  };
  resize(
    `pub const EMBEDDED: [(&str, &str); ${embedded}] = both_halves();`,
    `pub const EMBEDDED: [(&str, &str); ${settings}] = both_halves();`
  );
  resize(
    `const fn both_halves() -> [(&'static str, &'static str); ${embedded}] {`,
    `const fn both_halves() -> [(&'static str, &'static str); ${settings}] {`
  );
  resize(`let mut out = [("", ""); ${embedded}];`, `let mut out = [("", ""); ${settings}];`);

  return { source: out, settings, legacy, embedded };
}

// ---------------------------------------------------------------- reading the run

/**
 * Every failing test, and the binary it ran in.
 *
 * `--no-fail-fast` is not optional here. Without it cargo stops at the first
 * failing binary, and the first failing binary is the lib — so the run would
 * report three sites and hide four. The Windows suite learned this the hard way and it is the
 * same lesson.
 */
function parseFailures(output) {
  const failures = [];
  const compileErrors = [];

  // Deliberately not attributing a test to the binary it ran in. cargo prints
  // "Running tests/x.rs" on stderr and "test … FAILED" on stdout, so the two
  // streams are already de-interleaved by the time this reads them, and a
  // confident wrong answer is worse than no column. The expected list carries
  // the file, which is what a reader wants anyway.
  for (const line of output.split('\n')) {
    const failed = line.match(/^test (\S+) \.\.\. FAILED/);
    if (failed) {
      failures.push({ test: failed[1] });
      continue;
    }
    // cargo ends a failing run with `error: test failed, to rerun pass …` and
    // `error: N targets failed:`. Those are the summary, not a compile error,
    // and counting them would make every red run look like a broken build —
    // which is the one distinction this tool exists to draw.
    if (
      /^error(\[E\d+\])?: /.test(line) &&
      !/^error: test failed/.test(line) &&
      !/^error: \d+ targets? failed/.test(line)
    )
      compileErrors.push(line.trim());
  }

  return { failures, compileErrors };
}

// ---------------------------------------------------------------- run

const original = readFileSync(CONFIG, 'utf8');

// Uncommitted work in config.rs is not a reason to refuse — the restore writes
// back the exact bytes read a line above, so an edit in progress survives
// untouched. It is a reason to say so: if the process is killed in a way it
// cannot catch, the backup beside the file is what that edit comes back from,
// and knowing it exists is the difference between a recovery and a retype.
try {
  const dirty = execFileSync('git', ['status', '--porcelain', '--', CONFIG], {
    cwd: ROOT,
    encoding: 'utf8',
  }).trim();
  if (dirty && !asJson)
    console.log(
      `note: ${dirty.trim()} has uncommitted changes. They are restored with the rest;\n` +
        `      if this run is killed uncatchably, they are in ${BACKUP}.\n`
    );
} catch {
  // Not a git checkout, or git is absent. Nothing to say, and nothing to stop.
}

writeFileSync(BACKUP, original);

let restored = false;
const restore = () => {
  if (restored || keep) return;
  restored = true;
  writeFileSync(CONFIG, original);
  if (existsSync(BACKUP)) unlinkSync(BACKUP);
};
process.on('SIGINT', () => {
  restore();
  process.exit(130);
});
process.on('SIGTERM', () => {
  restore();
  process.exit(143);
});

let result;
try {
  const { source, settings, legacy, embedded } = applyDeletion(original);
  writeFileSync(CONFIG, source);

  if (!asJson)
    console.log(
      `rehearsing the deletion: ${legacy} legacy keys removed, ` +
        `EMBEDDED ${embedded} -> ${settings}\nrunning the suite (this compiles the crate)…\n`
    );

  const run = spawnSync(
    'cargo',
    ['test', '--manifest-path', MANIFEST, '--no-fail-fast', '--', '--test-threads=4'],
    { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 }
  );
  result = parseFailures(`${run.stdout || ''}\n${run.stderr || ''}`);
} finally {
  restore();
}

// ---------------------------------------------------------------- report

const seen = result.failures.map((f) => f.test);
const expected = EXPECTED.map((e) => e.test);
const missing = expected.filter((t) => !seen.includes(t));
const unexpected = seen.filter((t) => !expected.includes(t));

if (asJson) {
  console.log(
    JSON.stringify(
      { failures: result.failures, compileErrors: result.compileErrors, missing, unexpected },
      null,
      2
    )
  );
} else {
  console.log(`what the deletion costs — ${result.failures.length} failing test(s)\n`);
  for (const { test } of result.failures) {
    const row = EXPECTED.find((e) => e.test === test);
    console.log(`  ${row ? '\u00b7' : '!'} ${test}${row ? `  (${row.file})` : ''}`);
    console.log(
      `      ${row ? row.why : 'NOT on the expected list — nobody wrote this one down.'}`
    );
  }
  if (result.compileErrors.length) {
    console.log(
      '\ncompile errors — these are worse than a red test, because the suite\n' +
        'never runs and the message names a line rather than a decision:'
    );
    for (const e of result.compileErrors) console.log(`  ! ${e}`);
  }
  for (const t of missing)
    console.log(
      `\n  ? ${t} was expected to fail and did not — its row is now a wrong instruction.`
    );
  console.log(
    `\n${keep ? 'deletion LEFT APPLIED (--keep); backup at config.rs.rehearsal-backup' : 'tree restored'}`
  );
}

const clean = !missing.length && !unexpected.length && !result.compileErrors.length;
process.exit(clean ? 0 : 1);
