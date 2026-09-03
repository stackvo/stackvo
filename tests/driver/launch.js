/**
 * Starting the real application under `tauri-driver`, and taking it down again.
 *
 * ## What this proves that nothing else in this repository does
 *
 * The Playwright suite drives the front end in a real engine and stubs the one
 * global underneath `ipc.js` → `call()`. That is honest and it is half. It does
 * not start the Rust binary, so it cannot tell anyone whether a command named
 * in `contracts/ipc.json` is actually registered, whether the built bundle
 * loads under the CSP `tauri.conf.json` declares, or whether a failure comes
 * back in the contract's error shape. Every one of those has exactly one way to be
 * checked: run the binary and ask it.
 *
 * ## Where it runs, and why the skip is loud
 *
 * Linux CI. Not macOS — Tauri's documentation is explicit that there is no
 * WKWebView driver — and that is a fact about the platform rather than a
 * preference, so [`whyNotHere`] returns prose and the tests skip with it
 * printed. On Linux there is no skip path at all: a missing `tauri-driver` or
 * a missing binary fails, because "the suite was green" and "the suite never
 * ran" must not look the same on the one platform that can tell them apart.
 *
 * ## The machine it runs on is not this machine
 *
 * The application is pointed at a temporary `STACKVO_ROOT` and
 * `STACKVO_CONFIG_DIR`, so a driver run reads and writes a directory that is
 * deleted afterwards. `hosts_roundtrip.rs` made the same argument for
 * `STACKVO_HOSTS_PATH`: a test that touches the developer's real
 * workspace is a test somebody eventually switches off.
 */

import { spawn } from 'node:child_process';
import { connect } from 'node:net';
import { mkdtemp, mkdir, rm, access, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Session, WebDriverError, CLOSE_TIMEOUT_MS } from './webdriver.js';

const HERE = dirname(fileURLToPath(import.meta.url));
export const REPO = resolve(HERE, '..', '..');

/** `tauri-driver` proxies WebDriver on this port and speaks to the native one on the next. */
export const PORT = Number(process.env.TAURI_DRIVER_PORT || 4444);
export const NATIVE_PORT = Number(process.env.TAURI_DRIVER_NATIVE_PORT || 4445);
export const BASE = `http://127.0.0.1:${PORT}`;

/**
 * The reason this suite cannot run here, or `null` when it can.
 *
 * Returned as prose rather than a boolean because it is printed: a skipped
 * test whose reason is `false` teaches the next reader nothing, and this
 * particular skip is one they will meet on their own machine.
 */
export function whyNotHere(platform = process.platform) {
  if (platform === 'darwin') {
    return 'tauri-driver does not support macOS — there is no WKWebView driver. This suite runs on Linux CI.';
  }
  // `false`, NOT `null`. `node:test` takes `{ skip: <string|boolean> }`, and it
  // treats a `null` as a skip DIRECTIVE with no reason: a test that throws is
  // then reported `not ok N # SKIP`, counted under `# skipped`, left out of
  // `# fail`, and the process exits 0.
  //
  // That is not a detail. The first run of this suite that anybody ever watched
  // had four real failures — the app mounting nothing, and every IPC call
  // refused with "Origin header is not a valid URL" — and it exited GREEN. The
  // CI job would have reported success while proving nothing, which is the one
  // outcome a gate must never have.
  return false;
}

/**
 * Where `cargo build` leaves the application.
 *
 * `debug` by default, and the binary that belongs there is the one
 * `tauri build --debug` writes — **not** the one `cargo build` writes. They
 * land at the same path and are not the same program: `tauri-build` emits
 * `cfg(dev)` for a plain cargo build, so that binary embeds `devUrl` and the
 * webview opens `http://localhost:1420`. With no dev server up it gets
 * "connection refused", sits on `about:blank`, and every test here reports an
 * empty `#app`.
 *
 * That is what the first Linux run of this suite actually found, after a
 * failing assertion was made to print what the page said. Until then the
 * message was "the app root never rendered any children", four times, with no
 * way to tell an unbuilt bundle from a refused script from a Vue that threw.
 *
 * `STACKVO_DRIVER_BINARY` overrides the path for the run that wants to ask a
 * packaged binary instead.
 */
export function binaryPath({
  repo = REPO,
  platform = process.platform,
  profile = process.env.STACKVO_DRIVER_PROFILE || 'debug',
  override = process.env.STACKVO_DRIVER_BINARY,
} = {}) {
  if (override) return override;
  const name = platform === 'win32' ? 'stackvo-desktop.exe' : 'stackvo-desktop';
  return join(repo, 'src-tauri', 'target', profile, name);
}

/**
 * Wait until something is listening, by connecting to it.
 *
 * Not by polling an HTTP endpoint: `tauri-driver` is a proxy that starts the
 * native driver lazily, so `/status` before the first session is a question
 * about a process that does not exist yet. A TCP connect asks the only thing
 * worth knowing here — is the proxy up — and nothing else.
 */
export async function waitForPort(port, { timeoutMs = 20_000, everyMs = 100 } = {}) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const open = await new Promise((done) => {
      const socket = connect({ port, host: '127.0.0.1' });
      const settle = (value) => {
        socket.destroy();
        done(value);
      };
      socket.once('connect', () => settle(true));
      socket.once('error', () => settle(false));
      socket.setTimeout(1_000, () => settle(false));
    });
    if (open) return;
    if (Date.now() > deadline) {
      throw new Error(
        `nothing listened on ${port} within ${timeoutMs}ms — is tauri-driver installed?`
      );
    }
    await new Promise((r) => setTimeout(r, everyMs));
  }
}

/**
 * Open the session, retrying while the native driver is still coming up.
 *
 * `tauri-driver` is a proxy: the port it listens on answers as soon as *it* is
 * up, and the thing it proxies to — `WebKitWebDriver` — is started separately.
 * So `waitForPort` succeeding says the proxy is ready and says nothing about
 * whether there is anything behind it. Asking too early comes back as
 *
 *     Error serving connection: hyper::Error(User(Service), client error
 *     (Connect) ... Connection refused (os error 111)
 *
 * which reads like the app failing to start and is neither the app nor a
 * failure — it is a race, and it lands differently depending on how warm the
 * build cache was.
 *
 * Bounded, and it re-raises the last error rather than a timeout of its own: a
 * driver that is genuinely absent must still say so in its own words.
 */
/**
 * Whether `error` is the shape of "nothing is listening behind the proxy yet".
 *
 * Decided by what kind of failure it is, not by what its message says. The
 * first version of this test matched the message against `connect|refused`,
 * which is what `tauri-driver` prints — but that text is on the driver's
 * *stderr*, which `launch` appends to the error only after it has already
 * been thrown past this point. The error `fetch` actually raises when the
 * proxy accepts the TCP connection and then drops it is a `TypeError` whose
 * message is the two words `fetch failed`, with the real reason (`other side
 * closed`, `ECONNRESET`, `ECONNREFUSED`) one or two levels down in `cause`.
 * So the retry never fired for the one case it was written for, and the
 * suite passed only on the runs where the first attempt happened to land
 * after `WebKitWebDriver` was up. On 3 September 2026 one did not.
 *
 * Three answers, in order:
 *
 * - A `WebDriverError` is an HTTP response from the driver — a bad
 *   capability, a missing binary. It answered; asking again gets the same
 *   answer, slower.
 * - A `TimeoutError` is the request's own deadline. The driver had its
 *   chance; twenty more of them is a job timeout.
 * - Anything else that failed at the transport — `fetch failed` at the top,
 *   or a connection-refused/reset anywhere down the `cause` chain — is the
 *   race, and the only thing worth waiting on.
 */
export function stillComingUp(error) {
  if (error instanceof WebDriverError) return false;
  if (error?.name === 'TimeoutError' || error?.name === 'AbortError') return false;
  const transport =
    /fetch failed|connect|refused|ECONNRESET|socket hang up|other side closed|UND_ERR_SOCKET/i;
  for (let cause = error, depth = 0; cause && depth < 5; cause = cause.cause, depth += 1) {
    if (transport.test(`${cause.message ?? ''} ${cause.code ?? ''}`)) return true;
  }
  return false;
}

/**
 * `open` is injectable so the loop itself can be watched from a machine that
 * cannot run `tauri-driver`; the default is the real thing.
 */
export async function openWithRetries({
  application,
  args,
  attempts = 20,
  gapMs = 500,
  open = (options) => Session.open(BASE, options),
}) {
  let last;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      return await open({ application, args });
    } catch (error) {
      last = error;
      if (!stillComingUp(error)) throw error;
      await new Promise((r) => setTimeout(r, gapMs));
    }
  }
  // Still the driver's own words, with how long they were given.
  last.message = `${last.message}\n(after ${attempts} attempts, ${attempts * gapMs} ms apart in total)`;
  throw last;
}

/**
 * Start `tauri-driver`, open a session against the application, and hand back
 * both the session and the one function that undoes all of it.
 *
 * The teardown closes the session *before* killing the driver, in that order
 * and with the session close guarded: a driver killed while it still owns a
 * session leaves the application it started running, and on a CI runner that
 * is a job that hangs at the end rather than a job that fails.
 */
export async function launch({ application = binaryPath(), args = [] } = {}) {
  await access(application).catch(() => {
    throw new Error(
      `no application at ${application} — build it first: cargo build --manifest-path src-tauri/Cargo.toml`
    );
  });

  const root = await mkdtemp(join(tmpdir(), 'stackvo-driver-'));
  const config = join(root, 'config');
  await seedPreferences(config);

  const driver = spawn(
    'tauri-driver',
    ['--port', String(PORT), '--native-port', String(NATIVE_PORT)],
    {
      stdio: ['ignore', 'pipe', 'pipe'],
      // Its own process group, so teardown can take down everything the driver
      // started rather than only the driver. `tauri-driver` spawns
      // `WebKitWebDriver`, which in turn spawns the application: killing the
      // pid alone leaves two processes holding the display, and the runner
      // reports them at the end as orphans it had to terminate.
      detached: process.platform !== 'win32',
      env: {
        ...process.env,
        // The application inherits these through the driver, which is what
        // keeps a run off the developer's real workspace. `STACKVO_ROOT` moves
        // the app root (`appdir.rs`) and `STACKVO_CONFIG_DIR` the settings.
        STACKVO_ROOT: join(root, 'app'),
        STACKVO_CONFIG_DIR: config,
        // A policy file that does not exist reads as "no policy" rather than
        // as whatever this machine has under /Library or /etc. The policy layer
        // is not a security boundary; it is still an input, and an
        // input a test should not inherit from the host it runs on.
        STACKVO_POLICY_FILE: join(root, 'policy.json'),
      },
    }
  );

  // NOT `unref`ed, and the reason is a run that failed for it.
  //
  // It was, on the argument that a `before` hook which times out leaves the
  // driver running and an `unref`ed child cannot hold the event loop open past
  // the last test. That argument was right about the hazard and wrong about the
  // cost. With the child and both its pipes `unref`ed, the only things keeping
  // this process alive during launch are the socket `waitForPort` opens and the
  // timer between its attempts — and there is a gap between the two, on every
  // attempt, where the loop holds nothing at all. Node is entitled to exit
  // there, and on 3 September 2026 it did: five tests came back
  // `cancelledByParent` with
  //
  //     Promise resolution is still pending but the event loop has already
  //     resolved
  //
  // 117 milliseconds after the suite started, on a commit that changed nothing
  // in this directory. The run before it, on the same code, was green — which
  // is what a race looks like from the outside and why it must not be left in.
  //
  // The hazard it was aimed at is covered twice over without it: `stop` kills
  // the process group, `remember` kills it on exit whatever the reason, and the
  // `Drive the application` step carries `timeout-minutes: 10`. A hook that
  // times out now costs ten minutes and a red job — not the fifty-five it used
  // to, and not a suite that fails while its subject is working.
  const forget = remember(driver);

  // Kept, not printed: `tauri-driver`'s own diagnostics are the only place a
  // "cannot find WebKitWebDriver" ends up, and a failed launch that reports
  // only "nothing listened on 4444" sends the reader to the wrong question.
  const noise = [];
  driver.stdout.on('data', (chunk) => noise.push(String(chunk)));
  driver.stderr.on('data', (chunk) => noise.push(String(chunk)));
  const died = new Promise((_, reject) => {
    driver.once('error', (error) =>
      reject(new Error(`tauri-driver would not start: ${error.message}`))
    );
    driver.once('exit', (code) =>
      reject(new Error(`tauri-driver exited with ${code}\n${noise.join('')}`))
    );
  });
  // The driver is *expected* to exit — `stop` kills it — and by then the two
  // races below have long settled. Without this, that rejection is the last
  // thing to happen in the process, and Node's default for an unhandled one is
  // to end the process non-zero: a suite whose tests all passed, failing on the
  // way out.
  died.catch(() => {});

  let session;
  try {
    await Promise.race([waitForPort(PORT), died]);
    session = await Promise.race([openWithRetries({ application, args }), died]);
    // Generous, and bounded. The first script runs while the app is still
    // booting its gates; 30 seconds is long enough for a cold CI runner and
    // short enough that a hang is a failure rather than a job timeout.
    await session.timeouts({ script: 30_000, pageLoad: 30_000, implicit: 0 });
  } catch (error) {
    await kill(driver);
    forget();
    await rm(root, { recursive: true, force: true });
    error.message = `${error.message}${noise.length ? `\n--- tauri-driver said ---\n${noise.join('')}` : ''}`;
    throw error;
  }

  const stop = async () => {
    // Bounded, and its failure is swallowed on purpose. `DELETE /session` asks
    // the driver to close the window and the window is entitled to refuse —
    // `lib.rs` calls `api.prevent_close()` on every path, and under the "ask"
    // preference the close is handed to a dialog nobody is here to answer.
    // Waiting on that answer is what hung this suite for 55 minutes.
    // `seedPreferences` removes the reason; the timeout removes the class.
    await session.close({ timeoutMs: CLOSE_TIMEOUT_MS }).catch(() => {});
    await kill(driver);
    forget();
    await rm(root, { recursive: true, force: true });
  };

  return { session, stop, root, driverOutput: () => noise.join('') };
}

/**
 * Write the preferences a driven run needs before the application reads them.
 *
 * One key, and it is the one that decides whether this suite can end. Closing
 * the window is `prevent_close()` on every path (`lib.rs`), and the default
 * preference is `"ask"`: the close is forwarded to the front end as
 * `app:close_requested`, a dialog opens, and the window stays open until
 * somebody clicks. Under a driver nobody clicks — so `DELETE /session` never
 * completes, the application never exits, and the runner ends up killing
 * orphans.
 *
 * `"quit"` is the behaviour a driven run wants and the only one it can have:
 * exit, and leave the containers — of which a CI runner has none — alone.
 * `stopAndQuit` would ask Docker to stop a stack that was never started.
 *
 * `schemaVersion` is written because `preferences.json` carries one
 * (`PREFS_SCHEMA_VERSION`), and a file without it is a file a later migration
 * has to guess about. Its presence also stops `appdir::migrate_config` from
 * looking at the real config directory: it never overwrites a preferences file
 * that already exists.
 */
async function seedPreferences(configDir) {
  await mkdir(configDir, { recursive: true });
  await writeFile(
    join(configDir, 'preferences.json'),
    `${JSON.stringify({ schemaVersion: 1, closeBehaviour: 'quit' }, null, 2)}\n`,
    'utf8'
  );
}

/**
 * End the driver and everything it started, without waiting forever for either.
 *
 * SIGTERM to the whole process group first, because the group is what has to
 * go: `tauri-driver` → `WebKitWebDriver` → the application. Then SIGKILL, for
 * the case where the polite signal reached a process that is already stuck in
 * the close it refused to perform.
 *
 * Every kill is guarded. Between the check and the signal the group can be
 * gone, and `ESRCH` from that race must not become the failure of a run whose
 * tests all passed.
 */
async function kill(child, { graceMs = 2_000 } = {}) {
  if (child.exitCode !== null || child.signalCode !== null) return;

  const ended = new Promise((done) => child.once('exit', done));
  signal(child, 'SIGTERM');

  const inTime = await Promise.race([
    ended.then(() => true),
    new Promise((done) => setTimeout(() => done(false), graceMs).unref()),
  ]);
  if (inTime) return;

  signal(child, 'SIGKILL');
  await Promise.race([ended, new Promise((done) => setTimeout(done, graceMs).unref())]);
}

/**
 * Signal the child's process group where the platform has one, and the child
 * itself where it does not.
 *
 * A negative pid is POSIX for "the group"; Windows has no equivalent and
 * `detached` there means something else entirely, so it gets the plain kill.
 */
function signal(child, name) {
  try {
    if (process.platform === 'win32' || child.pid === undefined) {
      child.kill(name);
      return;
    }
    process.kill(-child.pid, name);
  } catch {
    // Already gone, or never in a group of its own. Either way there is
    // nothing left to signal, and a second attempt on the pid covers the one
    // case worth covering.
    try {
      child.kill(name);
    } catch {
      /* gone */
    }
  }
}

/**
 * The last resort: kill the driver when this process exits, whatever the
 * reason.
 *
 * `stop` is the path that runs when the suite behaves. This one is for when it
 * does not — a `before` hook that times out, an assertion that throws before
 * `after` is reached, a `SIGINT` from somebody watching the log. `exit`
 * handlers must be synchronous, so this is a bare `process.kill` and no wait.
 */
function remember(child) {
  const onExit = () => signal(child, 'SIGKILL');
  process.once('exit', onExit);
  return () => process.off('exit', onExit);
}
