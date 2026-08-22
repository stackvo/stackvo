# Inbox

The mail that was caught. Pick a message on the left, read it on the right.

## The list

| Control | What it does |
| --- | --- |
| Search | Filters the messages. You can type `from:ali@example.com` or `subject:"invoice"`. |
| Refresh | Reads the list again. |
| Empty the inbox | Deletes every caught message. |

The list header says how many messages were caught, how many are unread, and how many your search matched.

## Message tabs

| Tab | What it shows |
| --- | --- |
| Preview | The HTML body, in a sandboxed frame. |
| Text | The plain text body. |
| Source | The raw message. |
| Headers | Every header. |
| Attachments | The attached files; you can save them. |
| Compatibility | Whether the HTML and CSS used are supported by mail clients. Green is full support, orange partial, red unsupported. |
| Links | The links in the message. **Check links** fetches each one. |

## Release

Forwards a message to a real address. A copy stays in the catcher.

This needs a relay configured. The relay only sends the message you release; what your application sends is still caught.

## Worth knowing

- The HTML body renders in a sandboxed frame. A caught mail is whatever the code you are testing decided to send.
- **Check links** leaves your machine: every link is really fetched.
- The relay password is kept in the OS keystore and never shown again.
- Leaving the relay's "only send to" list empty means anywhere. That is one typo away.
