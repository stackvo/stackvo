# Performance layer

Moves heavy directories off the host filesystem and into a Docker volume. This is what makes Docker feel slow on macOS and Windows.

## Why a list and not one switch

The win depends on which directory you move. Measured:

| Moved | Framework boot | Request writes |
| --- | --- | --- |
| Nothing (bind mount) | 1.47s | 1.14s |
| `vendor` | 0.39s (3.8x) | unchanged |
| `vendor` + `storage/framework` | 0.40s | 0.41s (2.8x) |

`vendor` buys the boot and does nothing for writes. `storage/framework` is what buys the writes.

## Controls

| Control | What it does |
| --- | --- |
| Directory switch | Moves that directory into a volume, or back onto the host. |
| Export to host | Copies a snapshot of the volume's contents back to the host. |
| Delete the volume | Removes the volume and what is in it. |

Changes do not take effect until they are applied to the container; the card says so.

## Worth knowing

- Your own code always stays where your editor can see it. Only directories written by tools inside the container are moved.
- Your editor cannot see a moved directory. If you need to look inside, take a snapshot with **Export to host**. It is a copy; the container keeps writing to the volume.
- A directory that does not exist in the project yet can still be listed. Tools inside the container will create it.
