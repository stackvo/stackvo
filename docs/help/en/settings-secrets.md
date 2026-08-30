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

## Scanning for the ones nobody moved

The list above is the direction you take **after** you know there is a credential in `.env`. **Scan for credentials** is the other one: it finds the ones nobody moved, and — worse, and the reason it exists — the ones sitting in a file git is tracking.

It matches the **value**, not only the key's name. Masking matches names by suffix (`PASSWORD`, `TOKEN`, `KEY`), which is right for masking and not enough here: a variable called `MY_FAVOURITE_THING` can hold an AWS key just as well. So every rule is a shape its issuer publishes — `AKIA…`, `ghp_…`, `xoxb-…`, `sk_live_…`, a PEM private-key header — and the name rule is kept as a second, independent net.

There is deliberately **no** "long random-looking string" rule. That kind of rule fires on minified JavaScript, on a hash in a lockfile and on a base64 image, and a scanner people learn to ignore is worse than no scanner at all: a miss costs one finding, a false positive costs the feature.

| What it looks at | Why |
| --- | --- |
| `.env` | The values on this machine. A key already in the keystore is not a finding — you did the thing the app asked for. |
| Files git is **tracking** | What is tracked is exactly what leaves the machine. `node_modules` and build output are not read, because nobody pushes them. |

### What a finding carries instead of the value

A report that quotes the secret is a second copy of it, on a screen people photograph and paste into chat windows. But "never print it" on its own leaves you with a row you cannot act on — two lines saying *AWS access key* do not tell you whether that is one key in two places or two keys. So each finding carries what every scanner in this field carries:

| | What it is | What it is for |
| --- | --- | --- |
| Preview | `AKIA…MPLE` — the first and last four characters | Recognising *which* key it is, among the four in your password manager |
| Fingerprint | Twelve hex characters of the value's sha256 | Two rows with one fingerprint are **one secret in two places** — the difference between rotating one key and rotating two |

A value shorter than sixteen characters is masked whole, because four characters at each end of a short password is the password.

### Was it ever committed?

Asked **by path** — `git log --all -- <path>` — and never by value. `git log -S<secret>` would put the secret in a command line, where every process on the machine can read it out of `ps`. The path answer is also the stronger one: a file that was committed and later deleted is still in the history with everything that was in it, which a value search would miss the moment somebody rotated half of it.

That is why there are two separate answers about `.env`: **tracked now**, and **committed at some point**. The second is the one people get wrong. Untracking the file today does not take it out of the history.

### Taking `.env` out of git

When a project is named and its `.env` is tracked, the card offers the repair, and it does the standard thing in the standard order:

1. **`git rm --cached`** — untracks it and leaves it on disk. Not `git rm`, which would delete the configuration your stack is running on.
2. **`.gitignore`** — asked with `git check-ignore` rather than guessed, because `.gitignore`, `.git/info/exclude` and a global excludes file all take part. The line is added only if the answer is no.
3. **`.env.example`** — the file that should have been the tracked one all along: the same keys, no values, with your comments and grouping kept. Written only if there is none; overwriting yours with a generated one would throw away what you wrote for the next person.

Two things it does **not** do, and says so:

- The removal is **staged**, not committed. Until you commit and push it, nothing has left this machine.
- It cannot rewrite history. If the file was ever committed, every value that was in it is still in the repository — **rotate them.** That is the step people skip, and it is the only one that actually closes the hole.

The scan is bounded — 2,000 files, half a megabyte each — and says how many it skipped. A scan that passed over four hundred files and said nothing would read as a clean repository.

## Worth knowing

- The `stackvo.sh` command-line tool cannot read the keystore. If you use it against this workspace too, leave credentials in `.env`.
- If this machine has no keystore the app can reach, nothing can be moved and the card says so.
- If a credential points at the keystore and the keystore does not answer, file generation is blocked. Unlock your keychain, or restore the value.
