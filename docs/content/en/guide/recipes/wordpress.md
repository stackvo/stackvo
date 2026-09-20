# WordPress

A WordPress site at `https://blog.loc`, with MySQL, WP-CLI from any
terminal, and the plugin or theme you are working on in the folder where it
belongs.

## 1. Create it

**Projects → +**, then **WordPress** under the framework templates. Name it
`blog`. The template downloads WordPress and adopts the result: PHP, nginx
and the document root — the project root, not `public/` — are read from what
arrived.

**Create** builds the image and opens the project. `https://blog.loc` shows
the installer, which wants a database.

## 2. Give it a database

**Catalogue → Available** installs MySQL if no instance is running; **Project
→ Services this project needs** declares it for the project. The card shows
the host, the port and, on a click, the password. Enter them in the WordPress
installer, or write `wp-config.php` yourself:

```bash
stackvo wp config create --dbname=blog --dbuser=... --dbpass=... --dbhost=...
stackvo wp core install --url=https://blog.loc --title=Blog --admin_user=admin --admin_email=you@example.com
```

`stackvo wp` is WP-CLI inside the container, from the project's folder.

## 3. Work on a plugin or a theme

The project folder is the WordPress root, so `wp-content/plugins/yours/` and
`wp-content/themes/yours/` are where they always are. Edit in your editor;
the container sees the file at once. If the plugin is its own repository,
clone it into `wp-content/plugins/` — **Project → The rest of this
repository** is for the case where the *site* is the repository.

## 4. Mail

WordPress sends mail through `wp_mail()`. **Mail → Enable Mailpit** starts
the catcher; an SMTP plugin pointed at the host and port the Mail page names
puts every message in the inbox on the **Mail** page. Nothing leaves the
machine.

## 5. Uploads, limits, PHP

Large uploads and long imports hit PHP's limits first. **Settings → Web
servers → Request limits** sets the upload size and the execution time for
every PHP project; **Project → PHP settings** overrides them for this one.
The PHP version and extensions are the project's, in **Project → Project
settings** — an older site on PHP 7.4 and a new one on 8.4 run side by side.

## Every day

```bash
stackvo wp plugin list
stackvo wp search-replace 'https://blog.example.com' 'https://blog.loc'
stackvo wp db export
stackvo shell
```

A database dump of a production site restores the same way: **Project →
Snapshots** takes a named copy first, so the import can be undone by name.
