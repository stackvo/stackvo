# PHP settings

`php.ini` values for this project. They are written to `.stackvo/php.ini` and mounted read-only into PHP's `conf.d` directory. PHP reads it after its own `php.ini`, so what is here wins.

## Fields

| Field | What it does |
| --- | --- |
| Memory limit | The most memory one request may use. A number with `K`, `M` or `G`. `-1` for unlimited. |
| Max upload size | The cap on a single uploaded file. |
| Max POST size | The cap on the whole body. It must be at least the upload size; the smaller of the two wins. |
| Max execution time | Seconds a request may run for. `0` for unlimited. |

The values in the fields are PHP's current values in the running container.

## Controls

| Control | What it does |
| --- | --- |
| Save | Writes the file. |
| Remove the file | Deletes `.stackvo/php.ini`; settings go back to the image's defaults. |

Clearing a field removes that directive from the file.

## Worth knowing

- PHP reads configuration at startup. Restart the project after saving.
- If the file is on disk but not mounted into the running container, the card says so; bring the project up again.
- Editing the file by hand and committing it are both safe. Directives the card does not know about are listed separately and preserved.
- `stackvo up` from the command line does not layer this file.
