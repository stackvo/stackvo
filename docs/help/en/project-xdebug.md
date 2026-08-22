# Xdebug

Step debugging for this project.

## Controls

| Control | What it does |
| --- | --- |
| Enabled / Disabled | Turns Xdebug on and off. |

## The first time is different

Turning it on for the first time puts the extension in the image and needs a **rebuild**. Every time after that only restarts the container: the extension stays in the image and costs nothing while it is off.

The second switch being much faster than the first is normal.

## IDE settings

The card lists the values to put into your IDE:

| Field | What for |
| --- | --- |
| Port | The port Xdebug connects on. |
| IDE key | The key that identifies the session. |
| Server name | The `PHP_IDE_CONFIG` value. |
| Path mapping | Which path in the container matches which path on your machine. Without it breakpoints do not bind. |
| Xdebug version | The installed version. |

## Worth knowing

- If the card says the running container is not carrying the Xdebug settings, restart the project.
- `stackvo up` from the command line does not layer this configuration and recreates the container without it.
- Xdebug and the profiler are two modes of one extension. They cannot both be on.
