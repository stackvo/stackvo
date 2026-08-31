# Configuration

What this project's `stackvo.json` says it is. The fields are read-only; use the **Configure** button on the card to change them.

## Fields

| Field | What it means |
| --- | --- |
| Domain | The name the project opens at in a browser. |
| Aliases | Extra names that reach the same project. An alias starting with `*.` is a wildcard: it goes into the certificate and the router but cannot go into the hosts file, so it does not resolve on its own. |
| PHP / Node version | The version the container runs. |
| Container path | Where your code sits inside the container. Always `/var/www/html`. |
| Access URL · HTTP / HTTPS | The addresses the project answers on. |
| SSL status | Whether a certificate has been issued. |
| Server | nginx, Apache or Swoole. |
| Host path | The project's folder on this machine. |
| Type | The project's template. |
| Document root | The subfolder the web server publishes. `public` on Laravel. |

## Controls

| Control | What it does |
| --- | --- |
| Configure | Opens the project settings panel. Most fields here are changed there. |
| Copy | Puts the value on the clipboard. |
| Clicking an address | Opens it in your browser. |

## PHP extensions

The extensions compiled into the container. Adding one changes the image, so the project has to be rebuilt.

## Problems section

Anything in `stackvo.json` that violates the contract is listed here: the error code, the path in the file, and what is wrong. Warnings do not stop the project from running; errors do.

## Worth knowing

- If the domain does not resolve, this card shows a warning and a button that adds the hosts entry.
- Changing most of these values needs a rebuild. A restart is not enough.

## Does this machine match?

The repository declares what the project needs — its services, its domain, its manifest — and **Check my setup** answers whether this machine has it, line by line. It is the other half of onboarding: every tool in this category helps you *set things up*, and none of them answers the question you actually have an hour after cloning, which is *"I did set it up; why does it still not work?"*

Nothing new is measured. Four of the five facts are the ones the project list already computes — whether the manifest validates, whether the image was ever built here, whether the generated tree is older than `stackvo.json`, whether the domain is in the hosts file — and the fifth is the service table.

A declared service can fail in three ways and they are three different sentences:

| What you see | What it means |
| --- | --- |
| Missing | The service is in the catalogue and is not installed here. Install it from the Market. |
| Different | It is installed and switched **off** — and the versions you do have are shown on the right, because "install it" would be the wrong instruction. |
| Unknown | This build has never heard of that name. Either it is a typo, or the published catalogue is newer than this app. |

**Unknown does not fail the project.** A check the app declined to make is not evidence that something is wrong, and a verifier that says "not ready" for a question it did not ask is one people learn to override.

Every line is shown, including the ones that passed. A result that appeared only when something was broken would leave you unable to tell "it checked and I am fine" from "it did not check".

Without a lock file it cannot say that a *version* is wrong. If the declaration names `redis` without pinning a version, any installed Redis satisfies it and the version found is printed beside the line rather than judged. **Write stackvo.lock** is what changes that.

### The Laravel half: the PHP your project asks for

`composer.json` states what the project needs of the **platform** — `"php": "^8.3"`, and a list of `ext-*` requirements. `stackvo.json` states what the image gives it. Nothing had ever compared the two after adoption, and the failure that produces is frequent and expensive:

`composer.json` says `^8.3`. `stackvo.json` says `8.2`. The image builds without a complaint. Then `composer install` dies **inside the container** with a platform requirement error — which names PHP, and does not name the file that has to change. You are looking at a composer error and the fix is one line of a manifest.

So two more lines are checked, and neither measures anything new:

| Line | What it holds against what |
| --- | --- |
| The PHP `composer.json` asks for | the constraint's first `major.minor`, against `php.version` in your manifest |
| Each `ext-*` it requires | against `php.extensions` — one line per missing extension, because the repair is per extension and the name is the whole of it |

**`require-dev` is not read.** A dev requirement is a tool for the test suite, and failing a project's readiness on one would call a working installation broken.

**A constraint this cannot read is `Unknown`, not a failure.** `*` and a bare `^8` have no `major.minor` in them, and StackVo says so rather than guessing — the same rule that decides everything else on this card.

And a project with no `php` block in its manifest gets none of these lines, whatever its `composer.json` happens to say.

The same answer is available as `stackvo verify <project>`.

## stackvo.lock

`stackvo.json` says which services; `stackvo.lock` says which **versions**, and it belongs in the repository beside it. It is the same division every ecosystem settled on: the manifest is intent, the lock is fact, and the second is what makes the first reproducible.

```json
{
  "lockVersion": 1,
  "at": "2026-08-30T09:14:02Z",
  "services": [
    { "service": "redis", "version": "7.2", "source": "official", "sha256": "9f2c…" }
  ]
}
```

**`sha256` is what makes it a lock rather than a version list.** It is the digest of the package manifest as the catalogue stated it when the service was installed. The same version number can be published twice, and with the digest "redis 7.2" out of somebody else's catalogue is a different answer from "redis 7.2" out of the official one — which is precisely the substitution a version list cannot see.

Once the file exists, the check above gains three answers it could not give before:

| What you see | What it means |
| --- | --- |
| A different version | The lock says 7.2 and this machine runs 7.0. Both numbers are printed, because one of them alone is not something you can act on. |
| A different package | The version matches and the digest does not. Reinstall from the catalogue the lock names, or re-lock if this machine is now the reference. |
| No longer declared | The lock names a service `stackvo.json` has dropped. Nothing to install — re-lock. |

### Written only when you press it

Nothing refreshes this file on its own, and that is deliberate. A lock the app updated silently would record whatever the machine had drifted to, so it would always agree with the machine and could never disagree with it — and a check that cannot fail is worse than no check.

### What it does not lock

- **The runtime and the web server.** `stackvo.json` already carries `php.version` and `server`. A second copy of a fact is how two copies come to disagree.
- **The images StackVo itself pulls** — the tunnel runners, the landing page. Those belong to the machine, not to any one project: one `cloudflared` serves ten projects. They have their own pinning, in an administrator's policy file under `imagePins`.
- **Anything not installed, or installed and switched off.** It says which and why rather than writing an entry it made up — a lock that quietly covers three of your five services is one you believe covers five.

Also available as `stackvo lock <project>`, which is the form that belongs in a CI script.
