# Octane reload

Octane boots your application **once** and keeps it in memory. That is the whole point of it — and it is also the thing that catches everybody the first week.

You add a route to `routes/web.php`. You reload the page. You get a 404. The route is there in the file, and it does not exist in the running server, because the server has not read that file since it started. So you go looking for a typo in your own code.

## Why not `octane:start --watch`

That is Laravel's own answer, and it works. Its price is **installing Node and chokidar into your image**: a second file watcher, running inside a container, polling a bind mount that StackVo is already watching from the host — which on macOS and Windows is the expensive kind of polling.

StackVo already watches your project directory, and already runs commands inside your project's container. So the answer here is a single action:

```
php artisan octane:reload
```

It adds **nothing to your image**. That makes it strictly better than the documented route rather than merely different from it.

`octane:reload` is not a restart. The server replaces its workers and keeps its socket open, so nothing outside notices except the requests that were already in flight.

## The switch is off, and it stays off until you say otherwise

A reload that arrives while a request is being served **kills that request**. That is a trade some people want and some people do not, and a default cannot decide it for them. So the automatic reload is off per project, and turning it on is a decision with the consequence written next to it.

It is a **preference, not a manifest setting**. Your `stackvo.json` travels with the repository and describes what the project *is*; "reload my workers when I save" is what one developer wants on one machine, and a colleague should not inherit it from a `git pull`.

## What counts as a save

Only paths Octane itself watches: `app`, `bootstrap`, `config`, `database`, `public`, `resources`, `routes`, `composer.lock` and `.env`.

Anything inside `node_modules`, `vendor`, `public/build`, `public/hot` or `.git` is ignored at any depth, and so are editor swap files. Without that, running Vite would be a reload loop — a front-end build writes several hundred files into `public/build`, and none of them changed your application.

**It is debounced by two seconds**, which is much longer than the debounce on the manifest watcher. Those two answer different questions: that one asks *"did my editor write this file three times"*, which is about one save, and this one asks *"has the developer stopped changing things"*, which is about a whole operation. A `composer install` touches thousands of files, and one reload per file is a server that never finishes booting.

## Where it does not apply

If your project is served by nginx, Apache or Caddy, it runs through PHP-FPM, which reads the file on every request. There is nothing in memory to replace, so this card says so instead of offering a button that would do nothing.
