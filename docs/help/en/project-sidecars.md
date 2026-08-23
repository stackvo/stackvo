# Containers this repository brought with it

Extra containers declared in `stackvo.json` under `sidecars`. They are rendered into this project's own compose block, and they come up and go down with the project.

## A sidecar is not a service

A **service** is a catalogue id — a need this machine satisfies from a package, shared by every project that asks for it. A **sidecar** is a container this repository brought along, and it belongs to this project alone.

That difference is why two clones of the same repository under different project names get two separate containers and two separate volumes, and cannot collide: every name is derived from the project's name rather than declared in the file.

## Reaching one

Only from inside this project's network, by the container name shown on the row:

```
QDRANT_HOST=stackvo-shop-vectors
QDRANT_PORT=6333
```

There is no host port and no host path, and that is by construction rather than an omission. The argument that lets `stackvo php` run in the project's container — the container already runs this repository's code — does not carry over to a **new** image the file names, so anything reaching out of the project's own network is refused until there is a gate to approve it behind.

The practical effect: the application can use it, and you cannot open it in a browser.

## Declaring one

```json
"sidecars": {
  "vectors": {
    "image": "qdrant/qdrant:v1.19.0",
    "about": "Vector search for the recommendations page",
    "env": { "QDRANT__SERVICE__API_KEY": "local-only" },
    "volumes": [{ "name": "storage", "path": "/qdrant/storage" }]
  }
}
```

`image` must carry a tag. An untagged image moves under whoever pulled it last month, which is the same reason a package version is pinned.

`env` is committed to the repository, so it is for configuration and not for secrets — it is the file everybody on the team can read.

`volumes` are Docker named volumes. A host bind mount is the thing this format refuses.

## When to write one instead of asking for a package

When the container is a detail of this repository rather than something the machine offers. A vector database one project indexes into, a mock of a third-party API, a worker image the team maintains — none of those belong in a catalogue everybody shares.
