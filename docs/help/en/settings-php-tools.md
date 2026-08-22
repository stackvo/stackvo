# PHP build

What a new PHP container is built with.

## Controls

| Control | What it does |
| --- | --- |
| Composer version | The Composer installed into the PHP image. `latest` follows whatever is current at build time. |
| Node.js version | For asset builds inside the PHP container. Separate from the Node project runtime. |
| Tools | Extra tools installed alongside PHP. Type to add, click the cross to remove. |
| System packages | Packages installed with `apt` inside the container. |

## Worth knowing

- Changes affect projects generated afterwards. To apply them to an existing project, rebuild it.
- System packages add to the image size and the build time. Add only what you actually need.
