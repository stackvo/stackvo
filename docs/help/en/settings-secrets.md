# Where credentials are kept

Database passwords, tokens and server credentials can live in this machine's keystore instead of `.env`.

## Controls

| Control | What it does |
| --- | --- |
| Move | Saves the value into Keychain, Credential Manager or Secret Service and leaves a reference in `.env`. |
| Restore | Writes the value back into `.env`. |

## What it buys and what it does not

Moving takes the value out of the file that gets backed up, synced and pasted into support threads.

The value is still written into `generated/docker-compose.dynamic.yml`, because that is where Compose reads it. So this takes the password out of `.env`, not off the disk.

## Worth knowing

- The `stackvo.sh` command-line tool cannot read the keystore. If you use it against this workspace too, leave credentials in `.env`.
- If this machine has no keystore the app can reach, nothing can be moved and the card says so.
- If a credential points at the keystore and the keystore does not answer, file generation is blocked. Unlock your keychain, or restore the value.
