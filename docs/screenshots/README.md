# Screenshots

Thirty-nine pictures of the application, all taken by `npm run screenshots` —
thirty-eight of them at 1600x1000@2x in the light theme, and one of a terminal
at 80x24 cells. Re-running it reshoots every one of them, so they age with the
tree rather than with somebody's afternoon.

**What these are pictures of.** The webview half. The tool boots the built front end
in Chromium and replaces `window.__TAURI_INTERNALS__.invoke` with
[`tests/e2e/stage.js`](../../tests/e2e/stage.js) — the same boundary the Playwright
suite replaces, answering in the shapes [`contracts/ipc.json`](../../contracts/ipc.json)
declares. The layout, the components, the theme and the type are the real ones; the two
projects and two services on screen are staged, and there is no native title bar around
them because there is no Tauri window here.

## The pages

One picture per top-level page, in the order the rail lists them.

<table>
  <tr><td width="25%" valign="top"><a href="dashboard.png"><img src="dashboard.png" alt="Dashboard"></a><br><sub><b>Dashboard</b><br>Health, cost, machine</sub></td><td width="25%" valign="top"><a href="projects.png"><img src="projects.png" alt="Projects"></a><br><sub><b>Projects</b><br>Every project and its state</sub></td><td width="25%" valign="top"><a href="market.png"><img src="market.png" alt="Catalogue"></a><br><sub><b>Catalogue</b><br>Packages and versions</sub></td><td width="25%" valign="top"><a href="logs.png"><img src="logs.png" alt="Logs"></a><br><sub><b>Logs</b><br>Application and server logs</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="dumps.png"><img src="dumps.png" alt="Dumps"></a><br><sub><b>Dumps</b><br>The debug bridge</sub></td><td width="25%" valign="top"><a href="mail.png"><img src="mail.png" alt="Mail"></a><br><sub><b>Mail</b><br>What the projects sent</sub></td><td width="25%"></td><td width="25%"></td></tr>
</table>

## Project detail, section by section

Ten sections and not one of them is a URL — this is each, opened.

<table>
  <tr><td width="25%" valign="top"><a href="project-detail.png"><img src="project-detail.png" alt="Indicator"></a><br><sub><b>Indicator</b><br>Live CPU, memory, disk, network</sub></td><td width="25%" valign="top"><a href="project-detail-configuration.png"><img src="project-detail-configuration.png" alt="Configuration"></a><br><sub><b>Configuration</b><br>Settings, manifest, Dockerfile</sub></td><td width="25%" valign="top"><a href="project-detail-container.png"><img src="project-detail-container.png" alt="Container"></a><br><sub><b>Container</b><br>Facts, editor, addresses</sub></td><td width="25%" valign="top"><a href="project-detail-jobs.png"><img src="project-detail-jobs.png" alt="Jobs"></a><br><sub><b>Jobs</b><br>Workers, schedule, supervisord</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="project-detail-terminal.png"><img src="project-detail-terminal.png" alt="Terminal"></a><br><sub><b>Terminal</b><br>A shell and a REPL</sub></td><td width="25%" valign="top"><a href="project-detail-logs.png"><img src="project-detail-logs.png" alt="Logs"></a><br><sub><b>Logs</b><br>This project's logs</sub></td><td width="25%" valign="top"><a href="project-detail-debugging.png"><img src="project-detail-debugging.png" alt="Debugging"></a><br><sub><b>Debugging</b><br>Xdebug, profiler, dumps</sub></td><td width="25%" valign="top"><a href="project-detail-runtime.png"><img src="project-detail-runtime.png" alt="Runtime settings"></a><br><sub><b>Runtime settings</b><br>php.ini or the dev server</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="project-detail-release.png"><img src="project-detail-release.png" alt="Production image"></a><br><sub><b>Production image</b><br>An image and a devcontainer</sub></td><td width="25%" valign="top"><a href="project-detail-agent.png"><img src="project-detail-agent.png" alt="AI"></a><br><sub><b>AI</b><br>What an assistant is told</sub></td><td width="25%"></td><td width="25%"></td></tr>
</table>

## Settings, pane by pane

Seventeen panes, in the page's own order.

<table>
  <tr><td width="25%" valign="top"><a href="settings.png"><img src="settings.png" alt="Appearance"></a><br><sub><b>Appearance</b><br>Theme, radius, density</sub></td><td width="25%" valign="top"><a href="settings-localisation.png"><img src="settings-localisation.png" alt="Localisation"></a><br><sub><b>Localisation</b><br>Interface and console language</sub></td><td width="25%" valign="top"><a href="settings-preferences.png"><img src="settings-preferences.png" alt="Preferences"></a><br><sub><b>Preferences</b><br>Terminal, editor, browser</sub></td><td width="25%" valign="top"><a href="settings-workspace.png"><img src="settings-workspace.png" alt="Directory and control"></a><br><sub><b>Directory and control</b><br>Where the workspace is</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="settings-domain.png"><img src="settings-domain.png" alt="Domain and network"></a><br><sub><b>Domain and network</b><br>Suffix, hosts, resolver, routes</sub></td><td width="25%" valign="top"><a href="settings-certificates.png"><img src="settings-certificates.png" alt="Certificates"></a><br><sub><b>Certificates</b><br>What the certificate covers</sub></td><td width="25%" valign="top"><a href="settings-servers.png"><img src="settings-servers.png" alt="Web servers"></a><br><sub><b>Web servers</b><br>The server configuration</sub></td><td width="25%" valign="top"><a href="settings-catalogue.png"><img src="settings-catalogue.png" alt="Catalogue"></a><br><sub><b>Catalogue</b><br>Source, and whether it verified</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="settings-php.png"><img src="settings-php.png" alt="Defaults"></a><br><sub><b>Defaults</b><br>Versions a new project starts from</sub></td><td width="25%" valign="top"><a href="settings-secrets.png"><img src="settings-secrets.png" alt="Credentials"></a><br><sub><b>Credentials</b><br>What moved to the keystore</sub></td><td width="25%" valign="top"><a href="settings-agents.png"><img src="settings-agents.png" alt="AI assistants"></a><br><sub><b>AI assistants</b><br>Clients, and whether rules are current</sub></td><td width="25%" valign="top"><a href="settings-localApi.png"><img src="settings-localApi.png" alt="Local API"></a><br><sub><b>Local API</b><br>What other tools can drive</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="settings-tooling.png"><img src="settings-tooling.png" alt="Tooling"></a><br><sub><b>Tooling</b><br>CLI, completions, external tools</sub></td><td width="25%" valign="top"><a href="settings-machineCommands.png"><img src="settings-machineCommands.png" alt="Machine-wide commands"></a><br><sub><b>Machine-wide commands</b><br>Declared once for every project</sub></td><td width="25%" valign="top"><a href="settings-doctor.png"><img src="settings-doctor.png" alt="Doctor"></a><br><sub><b>Doctor</b><br>What is wrong, by name</sub></td><td width="25%" valign="top"><a href="settings-audit.png"><img src="settings-audit.png" alt="Audit"></a><br><sub><b>Audit</b><br>What it changed outside the workspace</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="settings-about.png"><img src="settings-about.png" alt="About"></a><br><sub><b>About</b><br>Versions, machine, workspace</sub></td><td width="25%"></td><td width="25%"></td><td width="25%"></td></tr>
</table>

## The screens that are not pages

Sheets and drawers: no address, opened by the control that opens them.

<table>
  <tr><td width="25%" valign="top"><a href="market-service-detail.png"><img src="market-service-detail.png" alt="Service detail"></a><br><sub><b>Service detail</b><br>Runtime, network, connection strings</sub></td><td width="25%" valign="top"><a href="market-instance-settings.png"><img src="market-instance-settings.png" alt="Service settings"></a><br><sub><b>Service settings</b><br>The package's own settings</sub></td><td width="25%" valign="top"><a href="market-add-instance.png"><img src="market-add-instance.png" alt="New service instance"></a><br><sub><b>New service instance</b><br>Defaults, credentials, port</sub></td><td width="25%" valign="top"><a href="project-new.png"><img src="project-new.png" alt="New project"></a><br><sub><b>New project</b><br>Name, runtime, configuration</sub></td></tr>
</table>

## The two the browser could not take

The original list had two screens this tool had no way to reach, and it now
reaches both — by a different route each, because they are missing for
different reasons.

<table>
  <tr><td width="25%" valign="top"><a href="project-detail-worktrees.png"><img src="project-detail-worktrees.png" alt="Worktrees"></a><br><sub><b>Worktrees</b><br>A branch with an environment of its own</sub></td><td width="25%" valign="top"><a href="tui.png"><img src="tui.png" alt="stackvo tui"></a><br><sub><b><code>stackvo tui</code></b><br>The stack, from a terminal</sub></td><td width="25%"></td><td width="25%"></td></tr>
</table>

**Worktrees** is a place on the Configuration section rather than a section of
its own, below the fold of that picture. The screen "wants a working git tree
behind it", and it has one — on the Rust side of `worktree_support`, the one
call the pane makes. The browser never reads the tree, so the tree is staged
the way everything else on these pages is:
[`tools/screenshots/worktree-stage.mjs`](../../tools/screenshots/worktree-stage.mjs)
answers that call with two branch environments of `shop`, and the tool scrolls
to the pane, opens the form and types a branch so the preview is in the
picture.

**`stackvo tui`** is a terminal program, and there is no window to shoot. Its
`draw` builds each frame as a string so a test can read it;
[`examples/tui_frame.rs`](../../src-tauri/examples/tui_frame.rs) prints one
frame for the same staged stack, and
[`tools/screenshots/ansi-frame.mjs`](../../tools/screenshots/ansi-frame.mjs)
reads the escapes back into cells that the tool draws in Chromium like any
other page. `npm run screenshots -- --page tui` needs `cargo`; the first run
builds the library.
