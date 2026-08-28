# Getting help

Four places, and which one you want depends on what kind of thing you have.

| You have | Go to |
| --- | --- |
| A question about using it | [Discussions](https://github.com/stackvo/stackvo/discussions) |
| Something that behaves wrongly | [Open an issue](https://github.com/stackvo/stackvo/issues/new/choose) |
| A security vulnerability | **Not an issue** — see [SECURITY.md](SECURITY.md) |
| An idea for a feature | [Open a feature request](https://github.com/stackvo/stackvo/issues/new/choose) |

## Before opening an issue

Two of the app's own tools answer most of what a maintainer would ask you for, and running them first usually shortens the round trip to nothing:

- **Settings → Doctor** names what is wrong and the repair beside it. If it names your problem, the fix is on screen.
- **Settings → Application log → Save a diagnostics bundle** writes the log, the preflight checks, the doctor report and any crash reports into one archive. Passwords and tokens are redacted as the log is written, and the archive is plain text inside — have a look before you send it.

Attach the bundle. The version number alone is rarely enough, because most of what goes wrong here is about the machine rather than about the code: which Docker, which socket, which port was already held and by what.

## What this project can promise

It is free, MIT-licensed, and maintained by one person. That is the whole model, stated plainly because the alternative is letting somebody guess:

- **There is no company behind it and no paid tier.** Nothing here is gated, and nothing is going to be.
- **There is no support contract and no response-time commitment.** Issues are read and most get answered, but "most" is the honest word.
- **It is not funded.** No sponsorship, no foundation, no full-time maintainer. If that matters for your organisation's decision, it should — several tools in this category are backed by sponsorship or a company, and this one is not.

That last row is the one worth reading twice if you are deciding whether to adopt this somewhere it will be hard to leave. The MIT licence means the code stays available whatever happens to the maintainer; it does not mean somebody will be there to fix it.

## Contributing

If you want to fix the thing you found rather than report it, [CONTRIBUTING.md](CONTRIBUTING.md) is the shorter path. The test suite is unusually opinionated — claims in the documentation are checked against the tree, so a number you change in a document will fail the build until the code agrees with it. That is deliberate and it is explained where it happens.
