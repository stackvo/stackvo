# Manifest

The project's `stackvo.json`, as text. The card is closed by default; click the heading to open it.

## Controls

| Control | What it does |
| --- | --- |
| Heading | Opens and closes the editor. Closing it does not lose what you typed. |
| Save | Writes the text to the file. Key order is corrected to satisfy the contract. |
| Bring up via compose | Generates the compose files from the saved manifest and starts the stack. |

## Worth knowing

- Every field on the Configuration card above is written in this file. Changing it there is safer; this is for seeing the file itself.
- Saving validates. A file that violates the contract is refused, and the offending key is named.
- A change made here reloads the rest of the page.
