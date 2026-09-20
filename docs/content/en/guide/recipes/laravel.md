# Laravel

A Laravel project from nothing to `https://shop.loc`, with a database, the
queue worker running and every mail it sends caught in the window.

## 1. Create it

**Projects → +**, then **Laravel** under the framework templates. Name it
`shop`; the domain `shop.loc` follows. The framework's own installer runs in
a throwaway container and the result is adopted: PHP, nginx and the `public/`
document root are read from what it wrote, not guessed.

!!! tip "The first template takes a few minutes"

    The installer's image is downloaded once. The next Laravel project is
    quick.

**Create** builds the image, writes the hosts entry after showing you the
diff, and opens the project. `https://shop.loc` answers with the welcome
page and no certificate warning.

## 2. Give it a database

The project's `.env` says `DB_CONNECTION=mysql`, and **Project → Services
this project needs** has read it: MySQL appears as *suggested*. If no MySQL
instance is running yet, **Catalogue → Available** installs one; pick the
version. Then **Write to stackvo.json** turns the suggestion into a
declaration, so a colleague who clones the repository gets the same.

The card shows the connection values, the password on a click. Put them in
`.env` — `DB_HOST`, `DB_PORT`, `DB_USERNAME`, `DB_PASSWORD` — and migrate
from the project's folder:

```bash
stackvo artisan migrate
```

`stackvo artisan` runs inside the container, on the container's PHP. Your
host needs none.

## 3. Queue and schedule

**Project → Workers** lists the processes the project's files call for: a
queue worker if a queue is configured, the scheduler if `routes/console.php`
schedules anything. **Start** each; they run beside the container and stop
with it. **Project → Scheduled jobs** is the table of individual jobs, with
each one's last run and log.

## 4. Mail

**Mail → Enable Mailpit** starts the catcher. Point the mailer at it in
`.env` — the Mail page names the host and port — and every message the
project sends lands in the inbox on the **Mail** page, with the HTML, the
text and the headers.

## 5. Redis, Horizon, Telescope

Add Redis the way MySQL was added: **Catalogue → Available**, then declare
it on the services card, then `REDIS_HOST` in `.env`. Installing the package
also gives you `redis-cli` in the project's commands.

Horizon, Telescope and Pulse need no setup: `https://shop.loc/horizon` works
the moment the package is installed, because the project already answers on
its own domain over a trusted certificate. **Project → Telescope, Horizon and
Pulse** says why each one is empty when it is.

## 6. Octane

**Project → Project settings → Web server**: `swoole` or `roadrunner`. Both
*are* the HTTP server, so the proxy points at 8000 instead of 80. **Project
→ Octane reload** restarts the workers when you change code Octane has
loaded.

## Debugging

- **Project → Xdebug → Enabled** puts the extension in the image and shows
  the values your IDE needs. [Debugging](../debugging.md) has the rest.
- **Project → Debug signals** catches `dump()` and `dd()` output in the
  window instead of the page.
- **Project → Browser tests (Dusk)** runs Dusk against the project's own
  domain, with the CA trusted inside the container.

## Every day

```bash
stackvo artisan tinker
stackvo composer require laravel/horizon
stackvo npm run dev
stackvo shell
```

All from the project's folder, all inside the container.
