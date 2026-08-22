# PHP and web server

The starting values for a new PHP project. Existing projects keep the version in their own `stackvo.json`.

## Controls

| Control | What it does |
| --- | --- |
| PHP version | Pre-selected in the new project form. Each project can still pick its own. |
| Web server | What serves PHP projects: nginx, Apache, Caddy, FrankenPHP or Swoole. |

## Worth knowing

- A change here only affects projects created afterwards. To change an existing project's version, go to that project's page.
- Other runtimes run their own dev server; this setting does not affect them.
