# Addresses

How the addresses your projects and services answer on are built. Every hostname sits under this suffix, which is what lets one certificate cover them all.

## Controls

| Control | What it does |
| --- | --- |
| Namespace | Gathers every address under one parent name. Optional; leave it empty to use the extension alone. |
| Extension | The addresses' extension: `.loc`, `.test` and so on. |

The card previews what your addresses will look like.

## Choosing an extension

| Extension | Status |
| --- | --- |
| `.test`, `.localhost` | Reserved for local use. Safe. |
| `.loc` | Not a registered TLD, and widely used for this. |
| `.dev` | A real TLD, and on browsers' HSTS preload list. Nothing under it opens over plain HTTP and the warning cannot be clicked through. Turn HTTPS on first. |

## Worth knowing

- Changing the suffix needs a new certificate. Look at the Certificates card after saving.
- Existing projects keep the domain in their own `stackvo.json`. A new suffix only affects projects created afterwards.
- Saving is not enough: regenerate so the routing labels pick the suffix up. Until then the stack answers with the old ones.
