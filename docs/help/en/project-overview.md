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

It cannot yet say that a *version* is wrong. If the declaration names `redis` without pinning a version, any installed Redis satisfies it and the version found is printed beside the line rather than judged — saying which one should be there needs a lock file.

The same answer is available as `stackvo verify <project>`.
