# A supervisord to probe against

The `check_probe` and `reach_probe` examples need a real `supervisord` to talk
to. This is it.

It is a **fixture, not a service**: nothing in StackVo needs it, no user has one,
and it is not started with the app. It exists because the pure tests cannot
answer the questions that matter here — whether `supervisorctl status` looks the
way the parser thinks it does, whether a container without the socket refuses
the way the classifier expects. Every bug those probes found was one no fixture
could have contained:

* `docker exec` without `-i`, where a command reading standard input reads
  nothing — successfully
* a pipe shut down but not dropped, where the far side never sees the end of it
* `supervisorctl` writing its refusal to stdout, not stderr

## Running it

```sh
docker build -t stackvo-supd-test src-tauri/tests/fixtures/supervisord
docker run -d --name svsupd -p 9001:9001 stackvo-supd-test
```

Then, from `src-tauri`:

```sh
cargo run --example check_probe          # health probes
cargo run --example reach_probe          # why a container has no process table
```

Take it down with `docker rm -f svsupd`. Nothing depends on it staying up.

## What is in it

Four programs, chosen for what they let a probe see rather than for realism:

| Program | Why |
| --- | --- |
| `steady` | Prints a line every five seconds — something with a log to tail. |
| `flaky` | Exits a second after it starts, for ever. This is `BACKOFF`, and it is what the restart counter and the flapping detector are measured against. |
| `sleeper` | Two of them under a group, so group control and `group:name` addressing have something to address. |

The TCP port is published for `check_probe` rather than for StackVo: nothing in
the app speaks to supervisord over a port any more, but a health probe needs
something that really is listening, and `9001` answers 401 without credentials —
which is also how the "a health endpoint behind auth is working" case gets
exercised.
