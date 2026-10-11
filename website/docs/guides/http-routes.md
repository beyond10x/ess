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

A binding with an open route is refused, naming each operation whose path it leaves open, and
nothing is written: a published document cannot carry a route nobody can send a request to. Give the
server binding that fills it in, with the protocol binding as `--refines`.

`--http` belongs to `--kind openapi` and is refused with any other `--kind`, including none, as
`--transport` is refused without `--kind asyncapi`. `--refines` requires `--http`. The synthesized
servers, `ess ui` and the other generated artifacts do not read a binding yet.
