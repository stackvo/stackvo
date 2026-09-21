# Requirements

StackVo runs on your machine and puts each project in a Docker container.
So it needs three things, and the third is small.

| Requirement | Detail |
| --- | --- |
| **Docker** | Docker Desktop on macOS and Windows, Docker Engine on Linux. Podman, Colima and OrbStack are recognised too. |
| **Operating system** | macOS 10.15 or later, Windows 10 or later, Linux on x86_64 or aarch64. |
| **Disk** | About 27 MB for the app. Docker images take more — count on a few gigabytes over time. |

## What you do not need

- **No PHP, Node, Python, Go, Ruby or Rust on your machine.** Every runtime
  lives inside a project's container. A host with no PHP at all runs a PHP
  project fine.
- **No package manager.** Nothing is installed through Homebrew, apt or
  Chocolatey.
- **No permanent admin rights.** The one elevated action is writing a line
  into your hosts file so `shop.loc` resolves. The app shows the exact change
  first and asks once.

## Docker, by platform

=== "macOS"

    Install [Docker Desktop](https://www.docker.com/products/docker-desktop/),
    or one of the alternatives StackVo recognises: OrbStack, Colima. Start it
    once; StackVo finds the socket on its own.

=== "Windows"

    Install [Docker Desktop](https://www.docker.com/products/docker-desktop/)
    and start it. StackVo connects through Docker's named pipe.

    !!! note "Windows is the least tested platform"
        The logic is tested on every platform and Windows is in the CI
        matrix, but the hosts-file write through UAC, the pipe against a real
        Docker Desktop and domain resolution in a browser have not been
        verified on a real machine yet. If something is off, please
        [open an issue](https://github.com/stackvo/stackvo/issues/new/choose).

=== "Linux"

    Install [Docker Engine](https://docs.docker.com/engine/install/) from your
    distribution, or Podman. With Podman, the rootless socket is looked for
    first.

!!! tip "If Docker is not running"
    The app still opens. It says so on the dashboard and offers to start
    Docker for you. Nothing else works until the engine is up, but you never
    stare at a blank window.

## Ports

Projects are served through Traefik, so ports 80 and 443 should be free.
Something else holding one of them is the commonest first-run problem;
**Settings → Doctor** names the port and the program that has it.

Next: [Installation](installation.md).
