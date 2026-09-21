# Requesting a change

Something it should do and does not. A good request describes a situation;
the feature is what the discussion arrives at.

## Before you ask

- **Search first.** The
  [requests](https://github.com/stackvo/stackvo/issues?q=is%3Aissue+label%3Aenhancement)
  and the [discussions](https://github.com/stackvo/stackvo/discussions),
  closed ones included. Add to an existing request rather than opening its
  twin.
- **Check it is not already there.** [Everyday use](../../guide/everyday-use.md),
  the [FAQ](../../reference/faq.md) and the **?** on the closest card are quick
  to check, and a fair number of requests turn out to be a setting nobody
  found.
- **Know what was turned down, and why.** Some obvious requests have been
  measured and declined with a reason — a bundled Ollama, a fifth database,
  Mutagen. The reasons live in the code beside what they explain. A request
  that answers one of them is far more interesting than one that does not
  know it exists.
- **Not sure it is a feature?** Ask in
  [Discussions](https://github.com/stackvo/stackvo/discussions) first.

## Write the request

Open a [feature request](https://github.com/stackvo/stackvo/issues/new/choose).
The form has four fields, ordered by how much they help.

### What are you trying to do

The situation, not the feature. "I keep having to X" says more than "add a
button for X": the button may not be the best answer, and nobody can judge
that without knowing what it is for.

### What you do today instead

The workaround, however ugly. This is the single most useful field on the
form: it says how much the gap actually costs, which is the one thing a
feature description never carries.

### Does something else do this

Name it — Herd, DDEV, Lando, Laragon, ServBay, anything. Not to copy it, but
because a working example settles in a minute an argument about whether a
thing is possible that would otherwise take a week.

### Which part of the app

Projects, services, terminals and logs, profiling, certificates and DNS,
settings, the CLI, the MCP server. It routes the request, and it tells you
something too: a change that touches three parts is three requests.

## What happens next

Requests are read and labelled. Some are done, some wait for a related
change, and some are declined with the reason written down so the next
person finds it. A declined request is not a closed door; it is a recorded
answer, and an answer can be argued with.

If you want to build it yourself, say so in the request before you start.
Ten minutes on the shape of it saves a rewritten pull request, and
[Making a pull request](making-a-pull-request.md) says the rest.
