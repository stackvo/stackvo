# Source

Where service packages are pulled from, and whether that address works.

## Controls

| Control | What it does |
| --- | --- |
| Catalogue address | An `https://` address or a folder. A GitHub repository URL is translated to where the files are actually served. |
| Test | Tries the address before pulling and says how many packages it found. |
| Choose a folder | Picks a local catalogue folder. |
| Pull and use | Downloads the catalogue and puts it into effect. |

## Worth knowing

- StackVo carries no services inside itself. Until a catalogue is pulled, nothing is available.
- An administrator can pin the source. Then the address here is ignored, and the card says what it was pinned to.
- If this machine requires a signed catalogue and no signing key has been published, the pull is refused rather than falling back to unsigned.
