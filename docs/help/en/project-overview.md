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
