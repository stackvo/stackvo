# Disk I/O

Current disk read and write rates, with the recent history underneath.

## Worth knowing

- This is the whole machine's disk traffic, not just the containers'.
- On macOS and Windows, bind-mounted directories give the disk far more work. If you see sustained writes, look at the project's Performance layer card: moving directories like `vendor` and `storage/framework` into a Docker volume makes a measurable difference.
