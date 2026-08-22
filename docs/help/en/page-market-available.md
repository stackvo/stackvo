# Available

The packages the source publishes, and which versions are installed here.

## Controls

| Control | What it does |
| --- | --- |
| Install | Downloads the version to this machine. |
| Uninstall | Deletes an installed version. |
| Add an instance | Creates a running service instance from that version. |
| Show end-of-life versions | Adds the hidden older versions to the list. |

## Support status

| Label | What it means |
| --- | --- |
| Supported | The vendor is still patching it. |
| Deprecated | The end-of-support date is approaching. |
| End of life | The vendor no longer patches it. |

An end-of-life version keeps working. Not being patched is not the same as being broken.

## Worth knowing

- End-of-life versions are held by the lists, not the catalogue. A workspace whose `.env` names that version has to be able to migrate, and an index that can drop a version is one where somebody loses the source of a running service.
- Before uninstalling a version, check whether an instance is using it; the card says so.
- Installing only downloads the files. Nothing starts running.
