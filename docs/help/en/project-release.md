# Production image

A shippable image derived from the one this project already runs: same PHP version, same extensions, same web server.

It is not a copy of the development image. That one has no application code in it (your source is mounted from disk) and carries Xdebug.

## Controls

| Control | What it does |
| --- | --- |
| Image tag | The name of the image to build. |
| Build | Builds the image. |
| Check | Verifies the image before it is pushed. |
| Push | Sends a verified image to a registry. |
| Deployment recipe | Gives you a compose file that runs the image. |
| Load a package | Reads a saved `.tar` back into this machine's Docker. |

## What the card shows

- **Excluded** — files kept out of the image.
- **Dockerfile used** — what the production image builds from.
- **What the image actually contains** — the contents of the built image.

## Worth knowing

- StackVo only pushes a verified image, and only to a tag carrying a registry name.
- A registry keeps layers. Deleting the tag afterwards does not remove what is inside it.
- Load needs neither a project nor a plan. It is the receiving end of an air-gapped transfer.
