# Updates

Checks whether there is a new version and installs it.

## Controls

| Control | What it does |
| --- | --- |
| Check for updates | Asks the release server. |
| Install and restart | Downloads, installs and restarts the application. |

## Worth knowing

- Updates are signed. The app verifies the package against a public key built into it; an unverified package is not installed.
- If the build has no public key in it, update checking is off and the card says so.
- Installing closes the application. Your running containers are unaffected.
