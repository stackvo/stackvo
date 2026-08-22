# Images

The number of Docker images on this machine and their total size.

## Worth knowing

- The count covers every image on this machine, not only the ones StackVo built.
- Every rebuild of a project creates new image layers. Old ones accumulate; if disk is short, `docker image prune` clears them.
