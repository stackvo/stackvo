# Every project

The viewer that shows every project's output in one stream.

## The toolbar

| Control | What it does |
| --- | --- |
| Project picker | Which projects to watch. Empty means all of them. |
| Search | Filters the visible lines; the match is highlighted inside the line. |
| Regular expression | Reads the search as a regular expression. |
| Level filter | Shows only the levels you pick. |
| Copy | Puts the visible lines on the clipboard. |
| Follow | Scrolls to the bottom as lines arrive. |
| Pause | Holds the stream; resuming delivers what was waiting. |

## The lines

Each line starts with the project it came from. The name is a fixed width so the text starts on one column, which is what makes an interleaved stream readable.

## Worth knowing

- The project selection is applied in the backend. That is what stops sixty watchers running for eight lines.
- A container path in a stack frame is clickable and opens the file in your editor.
