# Import a preset

Applies an exported preset to this workspace.

## Controls

| Control | What it does |
| --- | --- |
| Choose a file | Reads the preset and shows what would change. |
| Apply N changes | Writes the listed changes. |

## What you see before it writes

A full comparison before anything is written: what, from, to. Lines that cannot be applied are listed separately.

## Worth knowing

- Your passwords and ports are untouched.
- Enabling a service changes the generator's output. After applying, regenerate and bring the stack up.
- If the stack already matches the preset, the card says so and writes nothing.
