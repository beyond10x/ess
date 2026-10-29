---
title: Bind commands and views to HTTP routes
description: State the method and path a protocol fixes for a command or a view in an ess-http/1 binding beside the specification, check it, and publish it in the OpenAPI projection.
---

# Bind commands and views to HTTP routes

ESS derives an HTTP route for every operation it projects: `POST /{domain}/commands/{command}` for
a command a component accepts, and `GET /{domain}/views/{view}` for a view of a component declared
`reached_by: network`. A published HTTP protocol often fixes its routes itself, so the specification
cannot be the place that chooses them. An `ess-http/1` binding is where they are stated: a document
beside the specification, pinned to its source digest the way an `ess-transport/1` document is, that
names a method and a path for each operation it binds. The specification does not change when the
same command is served at another path; the binding does.

An operation the binding does not name keeps the route ESS derives for it.

## The binding

```yaml
type: ess-http/1
specification:
  system: catalog
  version: v1
  source_digest: sha256:<the specification's digest>
routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    method: put
    path: /items/{item_id}
    parameters:
      item_id: input.item_id
```

- Each route names exactly one `command` or `view` by its qualified name.
- `method` is lower case. A view is bound to `get`; a command to `post`, `put`, `patch` or
  `delete`, or to `get` when its path carries every input field.
- `path` is a template: it starts with `/`, and a variable is a whole segment, `{name}`.
- `parameters` maps every variable to the one value it carries: `input.<field>` of a command,
  `param.<name>` of a view. That field is a `String`, an `Integer`, an enum or an unconstrained
  newtype of one of them, because a path segment is always present and holds one value.

Check a binding against the specification, or write its compiled form:

```bash
ess specify http validate --path MODEL --binding catalog-http.yaml
ess specify http compile  --path MODEL --binding catalog-http.yaml --out catalog-http.json
```

Every refusal is reported at once, with an `ESS-HTTP-` code and the path of the key it is about. Two
operations on one method and path of a component, declared or derived, are refused, and so are two
paths one request can match, such as `/items/current` beside `/items/{item_id}`: neither takes
precedence.

## Protocol and server bindings

A protocol sometimes fixes one route and leaves another to each server. A route without `path` is
**open**: the protocol binding states the method and leaves the path out.

```yaml
type: ess-http/1
specification: {system: catalog, version: v1, source_digest: sha256:<the specification's digest>}
routes:
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
  - {command: catalog.items.ListItem, method: post}
```

A server binding names the protocol binding by the digest of its file's bytes, restates its routes
and fills in every open path:

```yaml
type: ess-http/1
specification: {system: catalog, version: v1, source_digest: sha256:<the specification's digest>}
refines: sha256:<the digest of the protocol binding's bytes>
routes:
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
  - {command: catalog.items.ListItem, method: post, path: /api/items}
```

```bash
ess specify http validate --path MODEL --binding server-http.yaml --refines protocol-http.yaml
```

A server binding may not change a method or a path the protocol binding fixed, or drop one of its
routes. It may route operations the protocol binding does not name.

## In the OpenAPI projection

```bash
ess generate --path MODEL --kind openapi --http server-http.yaml --refines protocol-http.yaml --out generated
```

The binding is compiled exactly as `ess specify http compile` compiles it, with the same refusals,
and each document then serves every operation the binding routes at its method and path:

- one path may carry several methods, one per operation;
- each path variable is a required parameter `in: path`, named after the variable, with the schema
  of the input field or view parameter it carries;
- a command's field carried in the path is not in its request body: `{command}.Input` describes the
  body, and a command whose every field is in the path has no request body;
- a view's parameter carried in the path is not a query parameter;
- each bound operation carries `x-ess-route-source: binding`, so a reader tells the protocol's
  routes from the derived ones.

An operation the binding does not name keeps its derived route, byte for byte, and every projection
generated without `--http` is unchanged.

### Encoding, statuses, headers and redirects

A route may also state how its request is encoded and how each outcome is answered:

```yaml
refusals:
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    request: {encoding: form, unknown: ignore}
    outcomes:
      placed:
        status: 200
        body: response
        headers:
          Cache-Control: {literal: no-store}
      out-of-stock: {status: 400, body: {error: error}}
  - command: catalog.orders.CancelOrder
    method: post
    path: /api/orders/{order_id}/cancel
    parameters: {order_id: input.order_id}
    outcomes:
      cancelled: {status: 200, body: response.order_id}
    errors: {status: 409, body: {error: error}}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404}
```

The document follows those keys on the routes that use them:

- `encoding: form` makes the request body `application/x-www-form-urlencoded`. `encoding: query`,
  on a `get` or a `delete` command, removes the request body, and each input field the path does
  not carry becomes an `in: query` parameter under its wire name (a `List` repeats its key,
  `style: form`, `explode: true`, and is not required: an absent key is the empty list).
- A view's parameter the path does not carry is an `in: query` parameter under the parameter's flat
  `wire:` when it declares one, and both generated servers read it under that name only.
- Each declared status is its own response. `body: response` answers the command's response
  object bare, `body: response.<field>` that one response field's value alone (the field may not
  be `Optional`), `{error: error}` an object whose `error` is the error's wire name beside the
  error's own fields, and `none` no content. An outcome the route does not list keeps its derived
  status and ESS's envelope.
- `errors: {status: 400, body: {error: error}}` on a command route answers every outcome with
  `error` that `outcomes:` does not list, `external:` refusals among them, exactly as listing it
  under each one would. A listed outcome keeps its own answer, and the document describes each
  outcome `errors` covers at that status.
- Declared headers are response `headers`, a `WWW-Authenticate` challenge among them, and a
  `redirect` answers a required `Location` header and no content.
- A `single` view answers its one row at `200` and no content at its `empty` status.
- A `refusals:` entry replaces the standard answer for that refusal on every operation the binding
  names.
- A bare body names no branch, so every response of such a command carries `x-ess-outcomes`, the
  outcomes it answers. A challenge is a header only: the document declares no `securitySchemes`.

### Operation ids and servers

A route may name the `operationId` a protocol or a client generator expects, and a binding may name
the base URLs of each component reached over the network:

```yaml
servers:
  orders-api:
    - {url: 'https://api.example.test/v2', description: Production}
    - {url: /v2}
routes:
  - command: catalog.orders.PlaceOrder
    operation_id: placeOrder
    method: post
    path: /orders
```

- `operation_id` becomes the operation's `operationId`. It starts with a letter or `_` and continues
  with letters, digits, `_`, `.` and `-`; it is unique in the binding and is not another
  operation's qualified name, the `operationId` that operation keeps (`ESS-HTTP-032`). A route
  without one, and every operation the binding does not name, keeps its qualified name.
- `servers` becomes that component's document `servers`, after `info`, in the order written, so the
  first is the default. Only a component declared `reached_by: network` takes servers. Each URL is
  absolute `http://` or `https://` with a host, or a path starting with `/`, without whitespace,
  `{`, `}`, a query, a fragment or credentials, and is listed once (`ESS-HTTP-033`).
- A server binding keeps every `operation_id` its protocol binding fixes, and may add one where the
  protocol binding has none. It states its own `servers`.

A binding with an open route is refused, naming each operation whose path it leaves open, and
nothing is written: a published document cannot carry a route nobody can send a request to. Give the
server binding that fills it in, with the protocol binding as `--refines`.

`--http` belongs to `--kind openapi` and is refused with any other `--kind`, including none, as
`--transport` is refused without `--kind asyncapi`. `--refines` requires `--http`. The synthesized
servers, `ess ui` and the other generated artifacts do not read a binding yet.
