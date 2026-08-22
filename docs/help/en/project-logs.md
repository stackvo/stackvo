# Logs

The container's own output and the log files the project writes.

## The toolbar

| Control | What it does |
| --- | --- |
| Source picker | Which stream to read: the container's output, or one of the project's log files. |
| Search | Filters the visible lines. The match is highlighted inside the line. |
| Regular expression | Reads the search as a regular expression. |
| Level filter | Shows only the levels you pick. |
| Copy | Puts the visible lines on the clipboard. |
| Follow | Scrolls to the bottom as lines arrive. Turn it off and your place stays put. |
| Pause | Holds the stream. Resuming delivers what was waiting. |

## Worth knowing

- The container's output only carries stdout and stderr. Anything your application writes to its own log file is not there; pick the file in the source picker.
- Log files only exist once the project has been built.
- A container path in a stack frame is clickable: it opens the file in your editor.
- The "live from here" rule marks the moment the stream opened. Lines above it were already in the file.
