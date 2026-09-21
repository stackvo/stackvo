# Coming from another tool

StackVo imports from seven local environments, and it never touches the one
it imports from.

| Source | What it reads | Source | What it reads |
| --- | --- | --- | --- |
| **XAMPP** | `htdocs` | **Laravel Sail** | `docker-compose.yml` |
| **Laragon** | `www` | **Laravel Herd** | the site list |
| **MAMP** | `htdocs` | **DDEV** | `.ddev/config.yaml` |
| **Laravel Valet** | parked and linked sites | | |

## How it works

1. **Projects → ⋮** opens the panel. It finds what is installed on this
   machine and **shows you what it found** before doing anything.
2. Pick a site and press **Adopt**. What it is — runtime, web server, document
   root — is detected from the files in the folder. **Adopt all** takes every
   site in one pass, with one password prompt for the hosts file, not one per
   site.
3. The site is **copied** into your workspace. With **Move instead of copy**,
   the original is deleted once the copy is complete and the other tool stops
   serving it.

## Three rules

- **Nothing is written into the other tool.** No PATH edits, no disabled
  services, no changed config. Nothing to undo if you change your mind.
- **Copy by default.** Your original stays where it was until you say
  otherwise.
- **You see it first.** For a project derived from a compose file, the
  `stackvo.json` that would be written — and where each value came from — is
  shown before it is written. Services with no StackVo equivalent are listed
  separately; those are yours to handle.

## After importing

Each imported site is a project like any other: its own container, its own
runtime version, its own address under your suffix. Start it from the
project row and open the domain. If the old tool is still running on ports 80
or 443, stop it or let it go — two servers cannot hold one port.

Next: [Everyday use](../guide/everyday-use.md).
