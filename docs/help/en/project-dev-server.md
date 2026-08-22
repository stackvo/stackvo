# Dev server

For Node projects. Runs the project's dev server with your source mounted live, instead of the production build baked into the image.

With this off the container carries a copy of your code taken at build time. Editing a file changes nothing.

## Controls

| Control | What it does |
| --- | --- |
| On / Off | Turns the source mount and the dev server on or off. |
| Dev command | The command to run. It replaces the production command, and the card says which one it replaced. |

## Your project needs something too

The lower half of the card shows the configuration you need in your own repository. It is shown, not written, because that file is yours.

Two things typically go wrong:

- Vite returns 403 for a hostname its configuration does not know. Add the domain to its allowed list.
- The hot-reload client has to be told which port the browser is actually on. Behind the proxy that is 443, not the dev server's own port.

The card reads your configuration and says whether both are covered.

## Worth knowing

- If `package.json` has no Vite, Nuxt or Next, there is no advice to give. The source mount still works.
- If dev mode is on but the container was created without the source mount, the card says so; bring the project up again.
