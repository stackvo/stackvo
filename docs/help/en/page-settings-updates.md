# Updates

Checks whether there is a new version and installs it.

## Controls

| Control | What it does |
| --- | --- |
| Check for updates | Asks the release server. |
| Install and restart | Downloads, installs and restarts the application. |
| Also receive beta releases | Adds pre-releases to what the check offers. Stable releases still arrive. |

## Worth knowing

- Updates are signed. The app verifies the package against a public key built into it; an unverified package is not installed.
- If the build has no public key in it, update checking is off and the card says so.
- Installing closes the application. Your running containers are unaffected.
- Beta means "stable, plus pre-releases", never a separate stream. A beta
  install still takes every stable release, and a beta that turns out badly is
  left behind by the next release, stable or beta. A stable install is never
  offered a pre-release.
- The beta switch takes effect the next time StackVo starts: the updater reads
  where to look once, at launch. Until then a beta install keeps checking the
  stable channel, which is safe in both directions.
- If no beta has been published yet, a beta install simply receives stable
  updates. Nothing about the switch can stop updates from arriving.
