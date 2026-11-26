---
title: Bind commands and views to HTTP routes
description: State the method and path a protocol fixes for a command or a view in an ess-http/1 binding beside the specification, check it, and publish it in the OpenAPI projection.
---

# Bind commands and views to HTTP routes

ESS derives an HTTP route for every operation it projects: `POST /{domain}/commands/{command}` for
a command a component accepts, and `GET /{domain}/views/{view}` for a view of a component declared
`reached_by: network`. A published HTTP protocol often fixes its routes itself, so the specification
is not where they are chosen. An `ess-http/1` binding states them: a document beside the
specification, pinned to its source digest the way an `ess-transport/1` document is, that gives a
method and a path for each operation it binds. Serving the same command at another path changes the
binding while the specification stays the same.

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
- `method` is lower case. A view is bound to `get`, and a command to `post`, `put`, `patch` or
  `delete`, or to `get` when its path carries every input field.
- `path` is a template: it starts with `/`, and a variable is a whole segment, `{name}`.
- `parameters` maps every variable to the one value it carries: `input.<field>` of a command,
  `param.<name>` of a view. That field is a `String`, `Integer`, `Boolean`, `Decimal`, `Timestamp`,
  `Duration` or `Uuid`, an enum, or an unconstrained newtype of one of them, because a path segment
  is always present and holds one value with one text spelling. `Bytes`, `Binary64` and `Json` are
  refused, and so are `Optional`, `List`, a struct and a constrained newtype.
- A variable the service accepts with any value and reads none of is mapped `ignored`:
  `parameters: {parent_id: ignored, item_id: param.item_id}` on `/parents/{parent_id}/items/{item_id}`.
  OpenAPI marks it `x-ess-ignored: true`, the generated servers answer any non-empty segment there
  alike, and `validate` names it. At least one variable of the route reads a value
  (`ESS-HTTP-039`). Where the service does check the parent, filter the view on it instead: a
  conformance run then tells the two apart.

Check a binding against the specification, or write its compiled form:

```bash
ess specify http validate --path MODEL --binding catalog-http.yaml
ess specify http compile  --path MODEL --binding catalog-http.yaml --out catalog-http.json
```

Every refusal is reported at once, with an `ESS-HTTP-` code and the path of the key it is about. Two
operations on one method and path of a component, declared or derived, are refused, variable names
ignored.

Two paths one request can match are admitted where one is more specific, holding the
literal at every segment they differ: `/items/current` beside `/items/{item_id}` serves
`/items/current` by its own route and every other item id by `/items/{item_id}`. Two that cross,
each holding a literal where the other holds a variable, such as `/{tenant}/tasks` beside
`/tasks/{id}` (both match `/tasks/tasks`), are refused: neither takes precedence.

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

- One path may carry several methods, one per operation.
- Each path variable is a required parameter `in: path`, named after the variable, with the schema
  of the input field or view parameter it carries.
- A command's field carried in the path is not in its request body: `{command}.Input` describes the
  body, and a command whose every field is in the path has no request body.
- A view's parameter carried in the path is not a query parameter.
- Each bound operation carries `x-ess-route-source: binding`, so a reader tells the protocol's
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
- `body: input.<field>` beside `encoding: json` (or no `encoding`) sends that one input field as
  the whole request body, bare: the request body's schema is the field's own, a list
  `{type: array, items: …}`, and no `{command}.Input` is written. The field may not be `Optional`,
  and the path must carry every other input field.
- `unknown: ignore` beside `json`, `form` or `query` ignores a member or key no input field names,
  for a client that sends a whole object back, and `{command}.Input` is written with
  `additionalProperties: true`. A member the path carries is admitted when it equals the path's
  value and malformed otherwise; under the default, `refuse`, both are malformed. A struct inside
  the body stays as closed as it is declared, and a bare `body` takes no `unknown: ignore`:

  ```yaml
  - command: catalog.shelves.RelabelShelf
    method: put
    path: /shelves/{shelf_id}/label
    parameters: {shelf_id: input.shelf_id}
    request: {unknown: ignore}        # {"shelf_id": "s-1", "label": "A", "updated_at": "…"}
  ```
- `encoding: multipart` takes a `multipart/form-data` upload: one part per input field the path
  does not carry, a `Bytes` field a file part holding the raw bytes, every other field a text part
  spelled as `form` spells it. `parts` reads a file part's `filename` and `media_type` into
  `String` input fields and lists the `media_types` it may carry. A `get` command takes no
  `multipart`. The document publishes the body as `multipart/form-data`, a file part as
  `{type: string, contentMediaType: application/octet-stream}` with its `media_types` as the part's
  `encoding.contentType`. Both generated servers read it:

  ```yaml
  - command: demo.reports.ImportRows
    method: post
    path: /reports/rows/import
    request:
      encoding: multipart
      parts:
        file:
          filename: input.file_name
          media_types: [text/csv, application/vnd.ms-excel]
  ```
- `query: [input.<field>, …]` reads the fields it lists from the query string beside the body of
  a command on `post`, `put`, `patch` or `delete`, each under its wire name as `encoding: query`
  spells it, and the document publishes each as an `in: query` parameter outside the body. Both
  generated servers read it there:

  ```yaml
  - command: demo.reports.ImportRows
    method: post
    path: /reports/rows/import
    request: {encoding: multipart, query: [input.dry_run]}   # …/import?dry_run=true
  ```
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
- `encoding: text` beside `body: response.<field>` naming a `String` field answers that string
  alone, `text/plain; charset=utf-8`, and the document describes it under `text/plain`.
- `alike: [[fetched, inline]]` on a command route declares that the service answers those
  outcomes the same way: their declared answers must be equal, and the document writes one response
  for the status, its `x-ess-outcomes` listing both, with one schema. Without it, two outcomes whose
  bodies do not tell them apart on one status are refused.
- Declared headers are response `headers`, a `WWW-Authenticate` challenge among them, and a
  `redirect` answers a required `Location` header and no content.
- A `single` view answers its one row at `200` and, at its `empty` status, no content or the
  literal object `empty: {status: 404, body: {error: {literal: not_found}}}` gives.
- `body: array` on a view that is not paged answers its rows as a bare JSON array, `[]` when it
  holds none.
- `body: array.<field>` answers each row as that one field's value, `["vip","sales"]`, and
  `body: single.<field>` the one row's value. Every other field of the view must be one the
  route's path carries (the view filters it by a parameter a path variable maps); an `Optional`,
  struct or union field is refused (`ESS-HTTP-024`).
- A member around a bare form answers the same value under that one name: `body: {order: single}`
  answers `{"order": {…}}` (with `empty` as for `single`), `body: {items: array}` answers
  `{"items": […]}`, and on a command `body: {data: response}` or `{id: response.order_id}` answers
  the response, or the one field, under that member. Each is refused wherever its bare form is.
- A `refusals:` entry replaces the standard answer for that refusal on every operation the binding
  names. A route's own `refusals:` replaces it on that route alone, so one endpoint can answer
  `malformed` with its protocol's error while the others keep the binding's.
- A bare body names no branch, so every response of such a command carries `x-ess-outcomes`, the
  outcomes it answers. A challenge is a header only: the document declares no `securitySchemes`.

### Answer a view as CSV

A view route may answer its rows as CSV when the query holds a value you choose, and as JSON
otherwise:

```yaml
routes:
  - view: demo.reports.Rows
    method: get
    path: /reports/rows
    body: array
    representations:
      - query: {format: csv}
        encoding: {csv: {separator: ";", header: true}}
        headers:
          Content-Disposition: {literal: attachment}
```

`GET /reports/rows` answers the JSON array; `GET /reports/rows?format=csv` answers
`text/csv; charset=utf-8`: a header line of the fields' wire names, then one line per row, `;`
between fields and CRLF at each line end. Another value of `format` is `malformed`. `separator`
is `","` (the default), `";"` or `"\t"`, and `header: false` leaves the header line out. Every
field of the view is a scalar or an `Optional` scalar; an absent value is an empty field. A
representation is refused (`ESS-HTTP-035`) on a command, a `single` view and a paged view, and its
headers are literals only.

### Limit the request body

A binding may state the largest request body its routes read, and the answer a larger one gets:

```yaml
limits: {request_bytes: 64}
refusals:
  too_large: {status: 418, body: {error: {literal: too_big}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
  - command: catalog.orders.NoteOrder
    method: post
    path: /api/orders/{order_id}/note
    parameters: {order_id: input.order_id}
    limits: {request_bytes: 256}
```

- It applies to a route the binding names that reads a body. A view, a `get` command, a
  `query`-encoded command and a route the binding does not name keep the fixed 1 MiB bound.
- `request_bytes` is at least 1 and at most 16777216 (16 MiB). A route's own `limits` replaces the
  binding's on that route; a limit on a route that reads no body is refused (`ESS-HTTP-034`).
- A binding that declares nothing keeps the 1 MiB bound, unchanged.
- `refusals.too_large` takes a `4xx` status. Its body and headers are written as for the other
  refusals; without it the answer is `413` with `{"refused": ...}`. A route's own `too_large`
  replaces the binding's on that route.
- Both servers refuse a body whose `Content-Length` is larger from the request head, before the
  body is read. A chunked request is answered `411` by the Rust server; the Go server reads a
  chunked body up to the bound plus one byte and then refuses it. Send `Content-Length`.
- The OpenAPI document names the bound of each such operation under `x-ess-request-max-bytes` and
  lists the answer as one more response.

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
  with letters, digits, `_`, `.` and `-`. It is unique in the binding and is not another
  operation's qualified name, the `operationId` that operation keeps (`ESS-HTTP-032`). A route
  without one, and every operation the binding does not name, keeps its qualified name.
- `servers` becomes that component's document `servers`, after `info`, in the order written, so the
  first is the default. Only a component declared `reached_by: network` takes servers. Each URL is
  absolute `http://` or `https://` with a host, or a path starting with `/`, without whitespace,
  `{`, `}`, a query, a fragment or credentials, and is listed once (`ESS-HTTP-033`).
- A server binding keeps every `operation_id` its protocol binding fixes, and may add one where the
  protocol binding has none. It states its own `servers`.

A binding with an open route is refused, and the refusal lists each operation whose path it leaves
open. Nothing is written, because a published document cannot carry a route nobody can send a
request to. Give the server binding that fills it in, with the protocol binding as `--refines`.

`--http` is read by `--kind openapi`, `ess generate synthesize --target rust|go`,
`ess generate types`, `ess generate ui` and `ess ui run`. `ess generate` refuses it with any other
`--kind`, including none, as `--transport` is refused without `--kind asyncapi`. `--refines`
requires `--http`.
