# Node.js

A Next.js, Nuxt, SvelteKit, Astro, Vite or NestJS project at
`https://app.loc`, with the dev server running against your source and the
production build available with one switch.

## 1. Create it

**Projects → +**, then the framework under the templates — **Next.js**,
**Nuxt**, **Vue (Vite)**, **React (Vite)**, **SvelteKit**, **Astro**,
**Angular**, **NestJS** — or **Empty project** with the **Node** runtime for
anything else. Name it `app`.

For an empty project the form asks for three commands and a package manager:

| Field | Typical value |
| --- | --- |
| Package manager | `npm`, `pnpm`, `yarn` or `bun`. Enables Corepack, so `packageManager` in `package.json` pins the version. |
| Install command | `npm ci` |
| Build command | `npm run build` — optional |
| Start command | `npm run start` |

A template fills them in from what the framework's installer wrote.

## 2. The dev server

The image carries a copy of your code taken at build time, so editing a file
changes nothing until you rebuild. **Project → Dev server**, switched on, mounts your
source into the container live and runs the dev command instead of the
production one — `npm run dev` — with hot reload.

The card also reads your framework's configuration and says whether two
things are covered:

- **The hostname.** Vite returns 403 for a host it does not know; `app.loc`
  has to be in its allowed list.
- **The hot-reload port.** Behind the proxy the browser is on 443, not on
  the dev server's own port, and the client has to be told.

The card shows the lines to add. It shows them rather than writing them,
because the file is yours.

## 3. A database, an API

A Node project needs services the same way a PHP one does: **Catalogue →
Available** installs PostgreSQL or MongoDB or Redis, **Project → Services
this project needs** declares it, and the card shows the connection values
for your `.env`. A monorepo with `api/` and `web/` is one project: see
[Projects](../projects.md).

## 4. Production

**Project → Dev server** switched off and a rebuild runs the build command and the
start command, which is what a server would run. **Project → Production
image** builds the image you would ship, from the same manifest.

## Every day

```bash
stackvo npm install
stackvo pnpm test
stackvo node -v
stackvo shell
```

From the project's folder, inside the container — the host needs no Node.
