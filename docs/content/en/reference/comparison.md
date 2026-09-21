# How it compares

This table is about **what each tool chose**, not about better or worse.

| | **StackVo** | Herd | ServBay | Laragon | DDEV | Laradock | Devilbox | FlyEnv |
|---|---|---|---|---|---|---|---|---|
| Approach | Docker + desktop | Native binaries | Native binaries | Native binaries | Docker + CLI | Docker + compose | Docker + compose | Native binaries |
| Interface | Desktop + CLI + TUI + MCP | Desktop | Desktop | Desktop | CLI | none | Web intranet | Desktop |
| Platforms | mac · Win · Linux | mac · Win | mac · Win | Win | mac · Win · Linux | all | all | mac · Win · Linux |
| Project isolation | Container | Site | Site | Site | Container | Shared stack | Shared stack | Site |
| Automatic HTTPS | Yes (mkcert) | Yes | Yes | Yes | Yes | manual | Yes | Yes |
| Per-branch env + **own DB** | Yes | No | No | No | partial | No | No | No |
| Request-level "why slow" | Yes (profile+queries+dumps) | partial (Pro) | No | No | No | No | No | No |
| Replay a recorded request | Yes | No | No | No | No | No | No | No |
| In-app mail inbox | Yes | Yes (Pro) | Yes (Pro) | Yes | web UI | web UI | web UI | Yes |
| Named DB snapshots | Yes (+ scheduled) | No | scheduled | scheduled | Yes | No | No | No |
| Monorepo as one project | Yes | No | No | No | No | No | No | No |
| MCP / AI integration | Yes — 38 tools, scoped | No | Yes | No | No | No | No | Yes |
| Builds the production image | Yes | No | No | No | No | Yes | No | No |
| Devcontainer export | Yes | No | No | No | No | No | No | No |
| Import sources | **7** | 1 | a few | No | a few | No | No | a few |
| Measures resource cost | Yes | No | No | No | No | No | No | No |
| Admin policy (MDM) | Yes | No | team plan | No | No | No | No | No |
| Portable install | No (by architecture) | No | No | Yes | No | No | No | Yes |
| Runs in Codespaces/Gitpod | No | No | No | No | Yes | Yes | Yes | No |
| Price | Free, MIT | Free + Pro $99/yr | Free + Pro | Free | Free, Apache-2 | Free, MIT | Free, MIT | Free + Pro $10 |

*Compiled from each project's own documentation, September 2026. If a row is
wrong, please [open an issue](https://github.com/stackvo/stackvo/issues/new/choose)
and it will be corrected.*

## Being honest: what Docker costs you

| | StackVo | A tool that installs PHP on the host |
|---|---|---|
| First install | the app (~27 MB) **plus** Docker and its images (GB) | one installer, ~100 MB |
| A project's first start | an image build — minutes | seconds |
| **Changing PHP version** | rewrite the manifest, rebuild the image | immediate |
| Idle memory | the Docker VM, Traefik and whatever services are on | the language runtime alone |

What you get for it is the thing none of them can offer: every project's
environment is a container, so what runs on your machine is what a Dockerfile
says rather than what your `brew` history says.

**Two limits, both decided rather than missing:**

- **There is no portable install, and there cannot be one.** Images and
  volumes live in Docker's own store. `STACKVO_ROOT` is half an answer — your
  workspace moves with you, the engine does not.
- **It does not run inside Codespaces or Gitpod.** This is a desktop
  application. What it does instead is **export a devcontainer**, so a project
  set up here can be opened in a cloud environment.
