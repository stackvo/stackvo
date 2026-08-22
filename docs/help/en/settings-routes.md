# Custom routes

Points a name at something StackVo did not start: a dev server you run yourself, a service in another tool, or a staging address.

## Controls

| Control | What it does |
| --- | --- |
| Name | The domain to route. |
| Goes to | The target address. |
| Enabled | Turns the route on or off. |
| Add a route | Adds a row. |
| Remove | Deletes the route. |
| Save and apply | Writes the routes and applies them to the router. |

## If you type localhost

Type `http://localhost:3000` and StackVo corrects it. Inside the proxy's container "localhost" is the proxy itself, which without the correction is a 502 with no explanation.

## Worth knowing

- A route does not check whether the target is up. If it is not, the address returns an error.
- The name has to be under the workspace's suffix for the certificate to cover it.
