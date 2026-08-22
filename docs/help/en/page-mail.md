# Mail

Mail your projects send, caught before it leaves the machine.

## When the catcher is off

The whole page is an offer to turn it on. **Enable** writes `.env`, regenerates the configuration and starts the container. The first run downloads the image and can take a minute.

The app never touches `.env` because a page was opened. Opening this page changes nothing.

## Worth knowing

- While the catcher runs, nothing your application sends leaves the machine. It all stops here.
- To forward a message to a real address you need a relay configured; without one, releasing is refused.
