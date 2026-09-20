# Creating a reproduction

A reproduction is the smallest setup that still shows the bug: a fresh
workspace, one project, the fewest settings that matter, and the steps that
get there. It is the difference between "cannot reproduce" and a fix,
because most of what goes wrong in StackVo is a particular manifest on a
particular machine, and a report without either leaves the maintainer
guessing at both.

## Guide

### Environment

StackVo keeps everything in one workspace — `~/.stackvo` by default — and
the `STACKVO_ROOT` variable moves it. A second workspace costs nothing and
keeps your real projects out of the picture. Set the variable and start the
app from that shell:

=== "macOS and Linux"

    ```bash
    export STACKVO_ROOT=~/stackvo-repro
    ```

=== "Windows"

    ```powershell
    $env:STACKVO_ROOT = "$HOME\stackvo-repro"
    ```

The app treats the empty folder as a first run — the one question, the
certificate, the stack — and writes everything there. Delete the folder
afterwards and nothing of it remains. (**Settings → Workspace** switches
between workspaces that already exist; it is for coming back, not for
creating one.)

!!! note "One stack at a time"

    The compose project is called `stackvo` on every machine, so the throwaway
    workspace and your real one cannot be up at the same time. Bring yours
    down first (`stackvo down`, or the tray), and bring it back up after.

### Minimal reproduction

1. **Update first.** Only the latest release is supported;
   **Settings → Updates** installs it. A bug that is gone afterwards was
   already fixed.

2. **One project, from a template.** **New project**, the closest template,
   default settings. If the bug shows already, stop here — that is the
   reproduction.

3. **Add one setting at a time** until the bug appears: the PHP version, a
   service, a domain, a hook, the `.env` line. The last thing you added is
   the thing to name in the report.

4. **Take away what does not matter.** Remove settings and services one by
   one and check the bug is still there after each. What remains is the
   reproduction; everything else was noise.

The result is usually a `stackvo.json` of a dozen lines. Paste it into the
report as it is — it is the manifest, and the manifest is most of what the
maintainer needs.

### Steps

Numbered, from the fresh workspace, ending at the thing that is wrong:

```text
1. STACKVO_ROOT=~/stackvo-repro, first run, defaults
2. New project, the Laravel template, named "repro"
3. Services → enable Redis
4. Restart the project
5. Open https://repro.loc — expected the app, got 502
```

"Sometimes" is not a step. If it only happens sometimes, say how often, and
what was different the times it did not.

### What to attach

| Attach | Where it comes from |
| --- | --- |
| The manifest | `stackvo.json` in the project folder |
| The changed `.env` lines | The workspace's `.env` — **Settings → Workspace directory → Open the folder** shows it. Only the lines you changed, values that are not secrets |
| The diagnostics bundle | **Settings → Application log → Save a diagnostic bundle** — log, preflight, Doctor report, crash reports, in one archive, secrets masked as it is written |
| Doctor's word | `stackvo doctor`, or **Settings → Doctor**; paste the lines it flags |
| Version and platform | **Settings → Updates**; the OS; the Docker runtime and its version |

A screenshot shows what you saw; the bundle shows why. Attach both if you
have both, the bundle if you have to choose.

### What to leave out

- Your real projects. That is what the second workspace is for.
- Anything with a password or a token in it. The bundle masks them; a
  pasted `.env` does not.
- A second bug. It gets its own issue, and its own reproduction.

## Then

Open the [bug report](https://github.com/stackvo/stackvo/issues/new/choose)
with the manifest, the steps and the bundle, and delete the throwaway
workspace. [Reporting a bug](../contributing/reporting-a-bug.md) has the
rest of the form.
