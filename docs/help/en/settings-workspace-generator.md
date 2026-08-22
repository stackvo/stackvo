# Generator (drift check)

Compares the generated files on disk with what the generator would write now.

## Controls

| Control | What it does |
| --- | --- |
| Verify the generator now | Runs the comparison and lists the differences. |

## What it is for

Generated files may have been edited by hand, or left over from an older version. This says whether the running configuration still matches your settings.

## Worth knowing

- The check fixes nothing; it shows the difference. Regenerate to fix it.
- No difference means the files on disk match your settings exactly.
