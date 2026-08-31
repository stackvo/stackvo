# What this project depends on

StackVo verifies every file of every **service package** against a digest before it runs, refuses a moving tag, and checks a signature over the index it came from.

Meanwhile the project beside it pulls four hundred libraries out of `composer.lock` and `package-lock.json`, and until now nothing here had ever looked at them. That is the wrong way round: the service packages are a catalogue this project publishes and can vouch for. The dependencies are somebody else's code, in far greater quantity, running with your permissions.

## Read the lock files — nothing leaves your machine

Three things the lock file already says, and each is worth saying out loud:

| Finding | Why it is one |
| --- | --- |
| **Fetched over plain HTTP** | Whoever is on the network path chooses what arrives. StackVo refuses `http://` for its *own* catalogue for exactly this reason; a project doing it for four hundred libraries is the same hole, larger. Named package by package. |
| **No integrity hash** | Nothing verifies those bytes. Reported as a count — on a lock file written by an older tool this can be every package, and four hundred identical rows is a screen nobody reads. Regenerating the lock with a current package manager usually adds them. |
| **From another index** | Not a fault. A private mirror is an ordinary thing to have. But it is a supply chain, and one nobody has written down is one nobody is watching. |

**Direct and transitive are kept apart**, and it is the distinction the whole card turns on: a direct dependency is a version *you* choose, and a transitive one is a version somebody else chooses for you. `composer.lock` does not record which is which — that fact only exists in `composer.json`, which is where it is read from.

**Dev dependencies are included** and are not marked apart. They are installed on this machine and run in the same container, and *"it is only a dev dependency"* is a sentence that has introduced real incidents.

## Check for advisories — this one leaves your machine

The second button sends **the names and versions of these packages** to `api.osv.dev`, the public vulnerability database.

Nothing else goes with it: no identifier, no project name, no path, no file contents. It is still a real disclosure — the list says which libraries you use and at which versions — which is why it is a separate button with that sentence above it rather than something folded into the report, and why `PRIVACY.md` says it in the same words.

What comes back is advisory **ids** — `GHSA-…`, `CVE-…`. An id is what you search for. A severity word derived here would be a judgement this app is in no position to make.

A failed query is an error, never an empty result. *"Nothing was found"* and *"I could not ask"* must not look the same on a screen like this one.

## Worth knowing

- **No lock file is not a clean project.** If neither file is here, the card says so rather than reporting nothing wrong.
- Only `composer.lock` and `package-lock.json` are read: the two this app's own runtimes reach for most, and the two that are JSON. `yarn.lock`, `pnpm-lock.yaml`, `go.sum` and `Cargo.lock` are each another format, and a parser written from memory against a format nobody measured is how a report starts quietly missing half a project.
- Lockfile v2 carries both the flat `packages` map and the old nested tree. The flat one is read when it is there, so nothing is counted twice.
