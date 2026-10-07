# Releasing StackVo

The pipeline is [`.github/workflows/release.yml`](.github/workflows/release.yml).
The reasoning for each guard is written beside it in the workflow; this page is
the procedure and the things to expect.

## Cutting a release

1. Bump the version on a branch (`package.json`, `src-tauri/tauri.conf.json`,
   `src-tauri/Cargo.toml`), merge it, and wait for `main` to go green.
2. Tag the commit you mean — **the merge commit on `main`, after the merge** —
   and push the tag. A tag with a hyphen (`v0.4.0-beta.1`) is a pre-release.
3. The `Release` workflow builds six targets and leaves a **draft**. It never
   publishes by itself.
4. Open the draft. Check the run summary for the red/green of
   _The draft must be one commit's work, and complete_, then run
   `npm run updates:check -- --url <the latest.json asset on the draft>`.
5. Press **Publish**. The `channel` job then moves the beta pointer.

To try the pipeline without releasing: _Actions → Release → Run workflow_ on
any branch with **Rehearsal** left on. It builds everything and publishes
nothing.

## Never move a tag that has had a run

`tauri-action` finds a release by its tag name, so a second run for the same tag
uploads into the first run's draft. v0.3.1 was tagged, built, then re-tagged on
another commit: the draft ended up holding installers from two commits and a
`SHA256SUMS.txt` that matched ten of its twelve files to nothing.

The workflow now refuses this (the preflight, every build row, and the final
check each compare the tag with the run's commit). If it stops you:

```sh
gh release delete vX.Y.Z --yes                              # the draft
gh api -X DELETE repos/stackvo/stackvo/git/refs/tags/vX.Y.Z # the remote tag
```

Delete the local tag too (GitHub Desktop: History → right-click the tag →
Delete Tag), merge what you meant to ship, and tag again.

**Re-running failed jobs** is fine while the tag has not moved. If it has, the
build rows fail on purpose.

## macOS notarisation is slow, and sometimes the network drops

Apple's notary service took **33 to 72+ minutes** per submission on this
account's first runs, on top of a ~20 minute build. So:

- the two macOS rows are allowed 180 minutes (`timeout:` in the matrix); every
  other row 90;
- the macOS rows retry once (`retryAttempts`). The cause is nearly always
  `NSURLErrorDomain -1009 "The Internet connection appears to be offline"`
  while `notarytool` is polling — the runner, not the build;
- a cancelled macOS row is a timeout, not a failure of the code. Re-run the
  failed jobs.

Expect the whole run to take about an hour while notarisation is slow.

## Apple credentials

Signing needs `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD` and
`APPLE_SIGNING_IDENTITY`. Notarisation takes either of:

| Method                                    | Secrets                                                                                |
| ----------------------------------------- | -------------------------------------------------------------------------------------- |
| App Store Connect API key (**preferred**) | `APPLE_API_ISSUER`, `APPLE_API_KEY` (key id), `APPLE_API_KEY_CONTENT` (the `.p8` text) |
| Apple ID                                  | `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), `APPLE_TEAM_ID`                  |

With all three API-key secrets present the Apple ID pair is left out of the
build on purpose, so a stale app-specific password cannot take precedence. Set
the key from the file, never from a terminal:

```sh
gh secret set APPLE_API_KEY_CONTENT < AuthKey_XXXXXXXXXX.p8
```

Create the key in App Store Connect → Users and Access → Integrations → Team
Keys (role: Developer is enough for notarisation).

## What the final check proves

After the checksums are attached, the draft is read as a whole. It fails the run
unless:

- the tag still points at the commit that was built;
- `latest.json` names this version;
- `latest.json` has a URL **and** a signature for each of `linux-x86_64`,
  `linux-aarch64`, `windows-x86_64`, `windows-aarch64`, `darwin-x86_64`,
  `darwin-aarch64`;
- every URL in it is a file that is on the release.

A red check means: do not press Publish.

## Dependabot and Tauri

Tauri crates and `@tauri-apps/*` packages must move together; the CLI stops with
`Found version mismatched Tauri packages` otherwise. Dependabot therefore only
raises **patch** releases of the Tauri crates and nothing at all on the npm
side. A Tauri minor is a deliberate commit that moves both halves and runs
`npm run notice`.
