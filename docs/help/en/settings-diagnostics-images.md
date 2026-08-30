# Images this app pulls

The containers StackVo runs that it did not build: the tunnel providers, the landing page, the tunnel guard and the performance helper.

## Why this list exists

StackVo refuses to install a **package** whose version is a moving tag — `latest`, `stable`, `edge`, `main`, `master` — because an image that changes underneath a fixed manifest has no version you can go back to.

Six of the images on this list are on `latest`. The rule was applied to everybody except this application. If the publisher of one of them ships a broken build, it arrives on your machine the next time a container starts, without you changing anything.

This list is the first half of fixing that: you can now see which of them move.

## Pinning one

An administrator's policy file can fix any of them, keyed by repository:

```json
{
  "schemaVersion": 1,
  "imagePins": {
    "cloudflare/cloudflared": "cloudflare/cloudflared:2024.8.2",
    "nginx": "nginx@sha256:…"
  }
}
```

- **A digest is the strongest form**; a fixed tag is the ordinary one. Both are accepted, because refusing a tag would refuse the answer most people can actually produce.
- **The pin has to name the same repository.** `"nginx": "alpine:3"` is a typo, not a pin, and it is refused and reported rather than applied — a pin that silently ran something else would be worse than the moving tag it was meant to fix.
- **The pin is applied before the registry prefix**, so pinning still works on a machine that mirrors Docker Hub.

## Worth knowing

- The tags are not pinned in the app itself, on purpose. Choosing a pin means naming a version that exists, which is something to check against a registry at release time — not something the source can assert.
- **Moving tag** means the reference that will actually run is still on one. A row you have pinned stops being flagged.
- Where these come from is the other half, and it is the same file: `registryPrefix` points them at your organisation's mirror.
