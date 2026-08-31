# What can leave this machine

Two questions with the same shape, and neither had an answer before: **which of your containers can reach the internet**, and **where did each one's image actually come from**.

Nothing else in this category can answer either, for a reason that is not a shortcoming: a local binary has no containers, so it has no network namespaces to separate one program's traffic from another's.

## "Can reach out" is a fact, not a guess

It is a property of the Docker network, asked of the daemon. A network created with `internal: true` gets no gateway installed — a container whose every network is internal **cannot route out**, and that is provable rather than inferred from behaviour.

The column is deliberately asymmetric:

| Answer | When |
| --- | --- |
| **Yes** | At least one of its networks has a gateway. One way out is a way out. |
| **No** | Every network it is on was created internal. |
| **Cannot tell** | A network the daemon would not describe. |

Nothing short of *every* network being **known** internal earns a "No". A containment claim resting on a lookup that failed is the one wrong answer a report like this must not give, so a network it could not read leaves the row at "Cannot tell".

## Where the image came from

Every container names the reference it was created from, and its registry host is the first component of that reference under Docker's own rule. A reference with no host — `mysql:8.0` — is shown as `docker.io`, because that is where it is pulled from; saying "none" would omit the one host most worth naming.

**If an administrator has set a registry mirror**, this is the follow-up question that had no answer: which containers did not come through it. A row marked *Not from the mirror* was created before the policy arrived, or from a reference the mirror leaves alone. On a machine where the mirror holds, the summary line lists exactly one registry.

## It does not say where anything connected to

Docker keeps no connection log. Answering *"which host did this container talk to"* needs either a packet capture inside the container's network namespace or a proxy standing in front of it, and this app will not install either on your machine to fill in a report.

That is stated here rather than left as a gap, so nothing on this page reads as a list of everywhere your containers have been.

## The byte counts

They are Docker's per-interface counters since each container started, and they include **all** traffic — the StackVo network between your own containers as well as anything that left the machine. So read them as *"did anything leave this container at all"*, which is genuinely useful, and not as internet usage, which they are not.

A stopped container shows nothing rather than zero: Docker has no counters for it, and a zero would be a claim about traffic instead of the absence of a measurement.
