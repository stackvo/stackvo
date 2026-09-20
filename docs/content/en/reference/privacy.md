# What leaves the machine

No telemetry, no reporting. This page is the complete list of network calls,
in two parts: the two the app makes on its own, and the ones that happen
because you pressed something. The full text is
[PRIVACY.md](https://github.com/stackvo/stackvo/blob/main/PRIVACY.md).

## On the app's own initiative

| Host | When | What is sent |
| --- | --- | --- |
| `github.com` | When the Settings screen is opened, and when you press **Check for updates**. Not at launch. | One GET for a static `latest.json`. No identifier, no version parameter. |
| `127.0.0.1` | While the mail screen is open | Loopback only; it never reaches a network interface. |

That is the complete list.

## Because you asked for it

| What you do | Where it goes |
| --- | --- |
| Create a project from a Git URL | The remote you typed, with your own `git` credentials. |
| Start the stack | Whatever registry the images name — Docker Hub by default. Your Docker daemon does this. |
| Open a share tunnel | The provider you picked, and only that one. |
| Fetch the service catalogue | The address you chose. Nothing fetches until you press the button. |
| Install a service package | The same address. The image itself is pulled later by Docker. |
| Open a help panel | `raw.githubusercontent.com`, for that card's document. **The request names the topic you opened**, and your language, and nothing else. Cached after the first fetch. |
| Build the sampling profiler | The container's package mirror and `github.com`, inside a throwaway container. Nothing is uploaded. |
| Record a profile | Your own project, on your own machine. The host is never taken from what you type. |
| Install a host tool | `github.com`, for one release asset of `mkcert`, checked against a digest compiled into the app. |
| Check dependencies for advisories | `api.osv.dev` — **the names and versions of that project's dependencies**. A real disclosure, behind its own button, with the sentence saying so above it. |
| Send a diagnostics bundle | Wherever you send it. Passwords and tokens are redacted as the log is written; the archive is plain text — look before you send. |

## What it writes, and never sends

`preferences.json`, `stats-history.json`, the application log, crash reports,
`audit.jsonl` — all under the app's own directory, listed in
[Workspace and files](workspace.md). Capture sessions for replay write a
project's request cookies and form bodies to `generated/debug/<project>/`
while a capture window is armed; they never leave the machine.

## This site

These pages are static files served by GitHub Pages. They set no cookies,
load no analytics and no fonts from third parties, and the search runs in
your browser. The one request a page makes on its own is to the GitHub API
from the front page, to name the latest release. What GitHub logs as the host
is GitHub's to say: [GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
