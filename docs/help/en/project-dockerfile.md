# Dockerfile

The file this project's image builds from. It is generated from `stackvo.json`, not written by hand. The card is closed by default.

## Controls

| Control | What it does |
| --- | --- |
| Heading | Opens and closes the file. |
| Mode | How the file is generated. See below. |

## The two modes

| Mode | What it does |
| --- | --- |
| Generated | What the generator actually writes. Extensions that cannot be installed are skipped silently. |
| Strict | Refuses to generate if an extension cannot be installed, and names it. |

## The badge

The badge in the card's heading says whether the generated file on disk is still current. If it says out of date, rebuild the project.

The badge only appears in Generated mode. Strict output differs by design, so comparing it to disk would mean nothing.

## Worth knowing

- You cannot edit this file. To change what is in it, change `stackvo.json`.
- It is regenerated every time the project is rebuilt.
