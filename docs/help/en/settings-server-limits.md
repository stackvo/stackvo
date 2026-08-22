# Request limits

What the web server in front of PHP will accept. Written into the generated server configuration.

## Fields

| Field | What it does |
| --- | --- |
| Max body size | The largest request body accepted. A number with an optional `k`, `m` or `g`. |
| Client body timeout | How long the body has to arrive. |
| KeepAlive timeout | How long a connection is held open. |
| FastCGI connect / send / read timeout | The times allowed when talking to PHP-FPM. On long requests the read timeout is the one that matters. |
| TCP nodelay | Sends small packets without holding them back. |
| Gzip, level, types | Response compression. Types are space-separated MIME types; left empty, the server's own list stands. |

A field left at its default writes nothing.

## PHP has its own limits

An upload is refused at the lowest limit in the chain. PHP has `upload_max_filesize`, `post_max_size` and `memory_limit`, and those are per project, on the project's PHP settings card.

Raising the web server's limit and forgetting PHP's leaves the upload refused anyway.

## Worth knowing

- Changes take effect after a regenerate.
- These limits are workspace-wide, not per project.
