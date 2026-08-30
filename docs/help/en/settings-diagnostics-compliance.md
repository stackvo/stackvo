# Is the policy actually holding?

This card appears only on a machine where an administrator has pushed a policy file. It reads that file's clauses one at a time and reports what this machine currently **is** — not what the file says.

## Why those are different

A policy arrives on a machine that was already set up, and most of what this finds is nobody breaking a rule. It is a rule with work left to do:

| Clause | When it is enforced | What that leaves behind |
| --- | --- | --- |
| `registryPrefix` | As files are **generated** | A project nobody regenerated since Tuesday still pulls from Docker Hub |
| `market.allowedPackages` | As a package is **installed** | A service installed last month stays installed when the list that would have refused it lands today |
| `market.requireSignature` | On the **next** refresh | The index already in the cache was accepted under whatever rule was in force then |
| `market.allowOverrides` | As an override is **created** | The files already on disk keep being read in front of the published package |

Everything above has the same repair: regenerate, uninstall, refresh, delete. The report exists so you know which one, and where.

## The four states

| State | Means |
| --- | --- |
| **Holding** | Measured, and this machine is inside the clause |
| **Bypassed** | Measured, and something here is outside it |
| **No opinion** | The policy says nothing on this subject |
| **Unmeasured** | No evidence either way — the reason is always written beside it |

**"No opinion" is never a pass.** Every list in the `market` block means *no opinion* when it is empty, never "none" — so a report that folded silence into a green tick would score a machine with no policy at all as fully compliant, which is the most misleading thing a compliance report can do.

**"Unmeasured" is never a pass either.** It covers two things that look different and are the same here: a fact the app cannot see (the generated tree would not read, a package manifest would not load), and a clause with nothing to apply to — an `imagePins` entry naming a repository this build never runs is a line that does nothing. Neither is evidence of compliance.

## "Nothing unaccounted for", and what it does not mean

The chip at the top is green only when nothing is bypassed **and** nothing is unmeasured.

It is deliberately not called *compliant*. The policy layer is not a security boundary — the file, and the `STACKVO_POLICY_FILE` variable that redirects it, are usually within reach of whoever holds the machine. This card reports what was measured here. It says nothing about what somebody could change, and it is not a certificate anybody can sign on the strength of.

## Worth knowing

- Every fact comes off the disk: the `.env` as written, the generated tree, the package directory, the remembered catalogue source, the override files, the project manifests. Docker is never asked — a report you cannot run with the engine stopped is one you cannot run when you most need it.
- The `.env` is read as written rather than as the app resolves it. The app itself always resolves a locked key to the administrator's value; the file is where the two can disagree, and anything reading the file directly — `docker compose` from a terminal, a script — gets what the file says.
- The mirror question is answered by the mirror itself: if re-applying it to a file's own bytes would change them, it never reached that file.
- Hooks and providers cannot be bypassed — they are checked as they run — so those rows report how much the rule is actually stopping. A refusal that stops nothing and one that stops forty steps are both "holding", and only one of them is worth your attention.
