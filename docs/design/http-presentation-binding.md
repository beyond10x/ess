# HTTP presentation binding (`ess-http/1`)

Status: binding for https://github.com/beyond10x/ess/issues/493; stage 1 is built in wave
2026-10-11j (2026-10-11). The scope is the operator's decision on
`decision-blocker:http-surface-scope` (option (b), the staged presentation binding). This page
covers the whole format before any stage is built, so stage 1 fixes no key that stage 2 would have
to bump. Stage 1 is planned for 0.60.0, stage 2 for the release after it, and stage 3 after that
(`decision-blocker:ess-060-wave-scope`). Each section below says which stage it belongs to.

## Problem

ESS chooses every HTTP wire shape it projects:

- the route, `POST /{domain wire}/commands/{command wire}` and `GET /{domain wire}/views/{view wire}`
  (`crates/generate/ess-gen/src/http.rs:448-473`);
- two methods only, `GET` and `POST` (`http.rs:320-332`);
- one media type, `application/json` (`crates/generate/ess-gen/src/openapi.rs:223-225`);
- a response envelope: `outcome`, `published` and `response` for a command
  (`openapi.rs:57-70`, `openapi.rs:231-234`), and `{rows: [...]}` for a view (`openapi.rs:1080-1091`);
- one status per kind of outcome (`http.rs:173-239`).

A published HTTP protocol fixes these itself. RFC 8414 §3 fixes
`GET /.well-known/oauth-authorization-server`, OpenID Connect Discovery 1.0 §4 fixes
`/.well-known/openid-configuration`, and both answer with one bare JSON object. RFC 6749 leaves
the token endpoint's path to each server (§3.2), takes a form-encoded request (§4.1.3), and answers
`400 {"error": …}` (§5.2). RFC 6750 §3 answers a refused bearer token with `401` and a
`WWW-Authenticate` challenge whose `error` attribute names the refusal.

None of this can be stated today. A command refuses an `http:` key
(`` unknown field `http` ``, reproduced in the issue). The model holds no path or verb
(`crates/specify/ess-domain/src/component.rs:259-261`, "No variant names a wire, a port, a path or a
verb"). A realization entrypoint carries one base URL, not a route per operation
(`crates/specify/ess-realization/src/lib.rs:350-364`). So an external implementation's conformance
adapter holds the operation-to-URL mapping in code that nothing checks against the model.

## Decision

A sibling document, `ess-http/1`, binds the commands and views of **one exact ESS** to an HTTP
surface. It is to a command or view what `ess-transport/1` is to an event: authored separately,
pinned to the specification's source digest, compiled against the resolved IR into a deterministic
`ess-http-ir/1`, and never part of `ess/N` (`docs/design/event-transport-binding.md:17-23`). The
model does not change when the same command is served at another path; the binding changes
instead. It follows `ess-cli/1` in owning the whole presentation of an operation, not only its
location (`docs/design/cli-presentation-binding.md:276-294`), and differs from it in pinning the
specification, which `ess-cli/1` does not (`crates/specify/ess-cli-contract/src/lib.rs:12-22`).

Without a binding, every artifact keeps its bytes. That is the rule `--transport` already keeps for
AsyncAPI (`event-transport-binding.md:100-102`).

| stage | adds | release |
|---|---|---|
| 1 | the document, its envelope, the method and path of a command or view, path parameters, refinement, the OpenAPI projection | 0.60.0 |
| 2 | request encoding, response body per outcome, status per outcome, response headers including `WWW-Authenticate`, redirects, overrides for the standard refusals, the Rust and Go servers' template dispatch | the release after 0.60.0 |
| 3 | an HTTP conformance target that reaches an implementation through the binding | after stage 2 |

## Stage 1: the document and the route

### Envelope

```yaml
type: ess-http/1
specification:
  system: catalog
  version: v1
  source_digest: sha256:<hex>
refines: sha256:<hex>          # optional; the digest of the binding this one fills in
routes:
  - view: catalog.discovery.ServerMetadata
    method: get
    path: /.well-known/oauth-authorization-server
  - command: catalog.orders.PlaceOrder
    method: post               # no path: each server chooses it
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
```

| key | meaning |
|---|---|
| `type` | exactly `ess-http/1` |
| `specification` | `system`, `version` and `source_digest` of the IR, the same closed object `ess-transport` reads (`crates/specify/ess-transport/src/lib.rs:34-44`) and checks (`lib.rs:514-533`) |
| `refines` | `sha256:<hex>` of the bytes of the binding this one refines. Absent, the document refines nothing |
| `routes[].command` / `routes[].view` | exactly one of the two: a command's or a view's qualified name |
| `routes[].method` | the closed lower-case set the OpenAPI import already reads: `get`, `put`, `post`, `delete`, `options`, `head`, `patch`, `trace` (`crates/generate/ess-openapi/src/lib.rs:132-152`) |
| `routes[].path` | a path template; absent, the route is open (below) |
| `routes[].parameters` | template variable to source: `input.<field>` for a command, `param.<name>` for a view |

Every object rejects unknown fields. The parameter spelling is the binding-mapping source spelling
that `ess-transport/2` reused for subject tokens (`docs/design/parameterized-event-channel-addresses.md:56-58`),
and `param.<name>` is what a view's `filter:` already reads. There is no expression language: a
source names one field.

### Two levels in one format

The transport format already splits what a contract fixes from what each deployment chooses:
`brokers[].host` is declared "when the contract fixes it" (`event-transport-binding.md:61`). Routes
split the same way:

- A **protocol binding** is published with the specification. It declares a path where the protocol
  fixes one, and leaves `path` out where the protocol leaves it to the server. A route without a
  path is **open**.
- A **server binding** names the protocol binding in `refines:` and fills in every open path. It
  restates each route of the refined binding. It may not change a method or a path the refined
  binding fixed. It may add routes for operations the refined binding does not name.

```yaml
type: ess-http/1
specification: {system: catalog, version: v1, source_digest: sha256:<hex>}
refines: sha256:<digest of the protocol binding above>
routes:
  - {view: catalog.discovery.ServerMetadata, method: get, path: /.well-known/oauth-authorization-server}
  - {command: catalog.orders.PlaceOrder, method: post, path: /api/orders}
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
```

The server binding belongs to the implementation, not to an environment: the path a server's code
serves is the same everywhere, and the base URL is what an environment changes
(`crates/generate/ess-deployment/src/runtime.rs` lowers ports and base endpoints; the realization
entrypoint carries the base `url`, `ess-realization/src/lib.rs:358-363`). A refinement refines one
document; a refinement of a refinement is refused in `/1`.

### Path templates

- starts with `/`; the root `/` alone is a path;
- `/`-separated segments, none empty, so no trailing `/` except the root;
- no `?`, no `#`, no percent-encoding, and no segment that is `.` or `..`. That is the same reason the
  specification already refuses such wire names (`crates/specify/ess-domain/src/wire.rs:95-116`): a
  client that removes dot-segments (RFC 3986 §5.2.4) asks for another route;
- a literal segment is RFC 3986 `pchar` text without `%`;
- a variable is a **whole** segment, `{name}`, where `name` is an identifier. `/items/{id}.json` is
  refused: a partial-segment template needs an escaping rule this format does not state.

Each variable is mapped exactly once in `parameters`, and each mapping names a variable of the path.
A command's mapped input field leaves the request body. A view's mapped parameter leaves the query
string; its other parameters stay query parameters exactly as today (`openapi.rs:1770-1815`). A
mapped source is a scalar a path segment can carry: `String`, `Integer`, an enum, or an unconstrained
newtype of one of them. It is never `Optional`, `List`, a struct or `Json`, because a path segment is
always present and holds one value.

### Which operations a binding may name

- A command a component `accepts`. Every such command has an OpenAPI route today (`http.rs:392-411`).
- A view of a domain owned by a `reached_by: network` component. A view has no route anywhere else
  (`http.rs:413`), and binding one would invent a query surface the specification did not declare.
- A view is bound to `get` only. A command is bound to `post`, `put`, `patch` or `delete`, or to
  `get` only when every input field is mapped to the path (stage 2 adds `request.encoding: query`).
  `head`, `options` and `trace` are refused for both in `/1`.

An operation the binding does not name keeps its derived route. Collision checks run over declared
and derived routes together, per component, because each component is one surface and one OpenAPI
document (`openapi.rs:500-510`).

### What compiling checks (stage 1)

Every refusal is returned at once, with a stable code and the path of the offending key, as
`ess-transport` does (`ess-transport/src/lib.rs:465-512`).

| code | refused | names |
|---|---|---|
| `ESS-HTTP-001` | `type` is not `ess-http/1` | `type` and the value found |
| `ESS-HTTP-002` | `specification` disagrees with the IR | `specification.<key>`, the value found and the IR's |
| `ESS-HTTP-003` | `refines` given and the refined file not supplied, its digest differs, or the refined file itself refines | `refines`, both digests |
| `ESS-HTTP-004` | a route names no command or view, names both, or names one that does not exist | `routes[i]` and the name |
| `ESS-HTTP-005` | one operation routed twice in one document | both `routes[i]` paths and the operation |
| `ESS-HTTP-006` | a method outside the closed set, or not admitted for that kind of operation | `routes[i].method`, the value, the kind |
| `ESS-HTTP-007` | a path that breaks the template grammar | `routes[i].path` and the offending segment |
| `ESS-HTTP-008` | a variable not mapped, mapped twice, or a mapping for no variable | `routes[i].parameters.<name>` |
| `ESS-HTTP-009` | a source in the wrong namespace, naming no field, or of a type a segment cannot carry | `routes[i].parameters.<name>`, the source, its type |
| `ESS-HTTP-010` | two operations of one component on one method and template (variable names ignored), declared or derived | both operations; the message reuses the import's wording, "two operations claim the same method and path" (`ess-openapi/src/lib.rs:76-80`) |
| `ESS-HTTP-011` | two templates of one component and one method that overlap: a literal and a variable at the same position after an equal prefix (`/items/current` beside `/items/{item_id}`) | both operations and the segment |
| `ESS-HTTP-012` | a refinement that changes a fixed method or path, or drops a route of the refined binding | `routes[i]`, the refined value and the new one |
| `ESS-HTTP-013` | an operation that has no HTTP surface: a command no component accepts, a view no network component serves | the operation |
| `ESS-HTTP-014` | a command on `get` with an input field that is neither mapped to the path nor (stage 2) query-encoded | the command and the field |

A stage-1 reader refuses every stage-2 key as an unknown field naming it. `ess-cli/1` took the same
route when the `trailing` kind arrived in 0.54.0 without a format change: an older reader refuses it
by name (`cli-presentation-binding.md:256-260`).

Validation does not refuse an open route. A document with an open route is valid and compiles; the
artifacts that need a path refuse it (below).

### Command surface (stage 1)

Mirroring `ess specify transport` (`crates/edge/ess-cli/src/main.rs:211-215`):

```console
ess specify http validate --path MODEL --binding catalog-http.yaml [--refines protocol-http.yaml]
ess specify http compile  --path MODEL --binding catalog-http.yaml [--refines …] --out FILE
ess generate --path MODEL --kind openapi --http catalog-http.yaml [--refines …] --out DIR
```

`--http` is refused with any other `--kind`, as `--transport` is refused without `--kind asyncapi`
(`main.rs:1626-1627`). `ess generate synthesize` accepts `--http` for the Rust and Go targets only,
from stage 2: 0.60.0 ships stage 1 without it, so the servers' template dispatch below lands with
stage 2 and `ess generate synthesize` has no `--http` flag until then.

### What generated artifacts change (stage 1)

`ess_gen::http::routes` is the one route table every HTTP artifact reads (`http.rs:1-12`,
`http.rs:387`). Stage 1 gives it an optional compiled binding. Without one it returns what it returns
today, so every reader below keeps its bytes.

| artifact | today | with a binding |
|---|---|---|
| OpenAPI | `routes()` becomes `paths` (`openapi.rs:505`) | declared methods and paths; path parameters as `in: path`, `required: true`; a mapped command field leaves `requestBody`. Each bound operation carries `x-ess-route-source: binding`, as a bound channel carries `x-ess-address-source: transport` (`event-transport-binding.md:91`). A binding with an open route is refused, naming the operation |
| Rust server | exact `match request.path.as_str()` and one method per path (`crates/generate/ess-synth/src/rust/http.rs:824-828`) | template dispatch and several methods on one path; `/openapi.json` and `/docs` (`http.rs:42-45`, served at `rust/http.rs:831-834`) stay reserved, and a route claiming either is refused at synthesis |
| Go server | `switch request.URL.Path` (`crates/generate/ess-synth/src/go/http.rs:1132`, `go/http.rs:1300`) | the same as Rust |
| view queries, view-grant synthesis | read `routes()` (`crates/generate/ess-synth/src/view_query.rs:187`, `crates/verify/ess-conformance/src/synthesize/view_grant.rs:64`) | follow `routes()` |
| UI binding and React adapter | build from `routes()` (`crates/ui/ess-ui-check/src/model.rs:1605`, `crates/ui/ess-ui/src/binding.rs:7`) | `ess ui` refuses `--http` by name until a story admits it |
| emitted conformance runners, web bridge, interpreter | no route (`crates/generate/ess-synth/src/web/bridge.rs:1-10`) | untouched |
| generated documentation | prints no route | untouched |

### Formats and determinism (stage 1)

- New: `ess-http/1` (authored) and `ess-http-ir/1` (compiled, canonical JSON).
- The IR orders routes by path, then method, the order `routes()` already uses (`http.rs:440-444`),
  and holds every map as a `BTreeMap`. It records the refined binding's digest and the derived routes
  it was checked against, so a reader sees the whole surface without recomputing it.
- Unchanged: `ess/N` (the `http:` key on a command or view stays an unknown field), `ess-realization/1`
  and `/2`, `ess-transport/1` and `/2`, `ess-cli/1` and `/2` with their plans, every conformance
  document, and every artifact generated without `--http`.

## Stage 2: encoding, bodies, statuses, headers and redirects

Stage 2 adds keys to a route. A route with none of them keeps ESS's own wire shape: JSON request,
the envelope, the derived statuses.

```yaml
type: ess-http/1
specification: {system: catalog, version: v1, source_digest: sha256:<hex>}
refines: sha256:<hex>
refusals:
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    request:
      encoding: form            # json | form | query
      unknown: ignore           # refuse | ignore
    outcomes:
      placed:
        status: 200
        body: response
        headers:
          Cache-Control: {literal: no-store}
      out-of-stock:
        status: 400
        body: {error: error}    # {"error": <error wire name>, …the error's own fields}
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
    body: single
    empty: {status: 404}
  - command: catalog.items.ReadPrivateItem
    method: get
    path: /private/items/{item_id}
    parameters: {item_id: input.item_id}
    outcomes:
      token-refused:
        status: 401
        body: {error: error}
        headers:
          WWW-Authenticate:
            challenge:
              scheme: Bearer
              params:
                realm: {literal: catalog}
                error: {source: error.wire}
  - command: catalog.orders.ApproveOrder
    method: get
    path: /approve
    request: {encoding: query}
    outcomes:
      approved:
        status: 302
        redirect:
          location: input.return_to
          mode: query           # query | fragment
          parameters:
            code: response.code
            state: input.state
```

### Request encoding

| `encoding` | request |
|---|---|
| `json` (default) | today's body, `application/json` (`openapi.rs:223-225`), less any path-mapped field |
| `form` | `application/x-www-form-urlencoded`, UTF-8 (RFC 6749 §4.1.3, Appendix B) |
| `query` | no body; every unmapped input field in the query string. The only encoding a `get` command may use |

A `form` or `query` field is a scalar, an `Optional` scalar or a `List` of scalars. A `List` repeats
its key once per element, the rule view parameters already follow (`openapi.rs:1776-1778`). A key a
scalar field receives twice is a malformed request (RFC 6749 §3.1). Booleans are `true` and `false`,
integers exact decimal text. Keys are the fields' wire names. `unknown: ignore` admits a request
parameter no field names (RFC 6749 §3.1, "MUST ignore unrecognized request parameters"); the
default, `refuse`, keeps ESS's closed bodies.

### Response body per outcome

| `body` | for | wire |
|---|---|---|
| `envelope` (default) | any outcome | today's body: `outcome`, `error`, `payload`, `published`, `response` (`openapi.rs:57-70`) |
| `response` | a `returns: true` outcome without `error` | the command's declared response object, bare |
| `{<member>: error}` | an outcome with `error` | an object whose `<member>` is the error's wire name (`naming.wire`, falling back to the local name, `crates/specify/ess-domain/src/name.rs:240-242`), beside the error's own fields under their wire names |
| `{<member>: {literal: <text>}}` | a standard refusal (below) | an object with one fixed member |
| `none` | any outcome | no body; required with `204`, `304` and every redirect |
| `rows` (default) | a view | today's `{rows: [...]}` |
| `single` | a view | the one row, bare; with `empty: {status}` answering a view that holds none |

A bare body carries no `outcome` and no `published`. A caller of such a route sees the branch only
through status, body shape and headers, and sees no published event. That is what the protocol
says, and stage 3 counts what it costs.

### Status per outcome

`status` is an integer. An outcome the binding does not list keeps `outcome_status`
(`http.rs:233-239`). Refused:

- `1xx` for anything;
- `2xx` for an outcome with `error`, and `4xx` or `5xx` for one without. `3xx` is admitted for both,
  because RFC 6749 §4.1.2.1 answers an authorization error with a redirect;
- two outcomes of one operation on one status unless the bodies tell them apart: both `envelope`
  (the `outcome` member does), or both `{<member>: error}` on one member with distinct error wire
  names. Otherwise a reader, and the stage-3 target, cannot say which branch was taken.

### Response headers

`headers:` maps a header name (an RFC 9110 token) to one value:

| value | wire |
|---|---|
| `{literal: <text>}` | the text, which must be a valid field value |
| `{source: <path>}` | `response.<field>`, `error.<field>`, `error.wire` or `input.<field>`, a scalar, written as text; an absent optional omits the header |
| `{challenge: {scheme, params}}` | one RFC 9110 §11.6.1 challenge: `scheme` a token, each param a `literal` or a `source`, serialised as a quoted string by the generator |

`WWW-Authenticate` with its `error` attribute (RFC 6750 §3) is the third form with
`error: {source: error.wire}`. The generator writes the quoting, so an error wire name cannot break
out of the challenge. `Content-Type`, `Content-Length` and `Transfer-Encoding` belong to the
encoding, and `Location` belongs to `redirect`. Declaring any of them is refused.

This states a header. It does not add authentication to the model: `security` and `securitySchemes`
stay omitted for the reason `openapi.rs:164` and `openapi.rs:171-191` give. A `401` with a challenge
is the answer a declared outcome gives; how a caller proves who it is remains the realization's.

### Redirects

`redirect` belongs to a `3xx` status (`301`, `302`, `303`, `307`, `308`), and each of those requires
it. `location` names a `String` source holding an absolute URI. `mode: query` appends `parameters`
to its query, keeping a query it already has (RFC 6749 §3.1.2); `mode: fragment` writes them as the
fragment. Parameter values are scalar sources; an absent optional is omitted, which is how RFC 6749
§4.1.2 returns `state` only when the request carried it. The body is `none`.

A redirect to a URI taken from input is an open redirect unless the specification's guards compare
that input with something stored. ESS does not add that check; the outcome that redirects must be
guarded by the specification.

### Standard refusals

The served surface refuses some requests without a declared outcome: an actor no grant admits
(`403`, `http.rs:88-109`), an unfinished realization (`501`, `http.rs:141-148`), a request with no
input where the command declares that answer (`400`, `http.rs:83-86`), and a malformed request or a
path nothing declares (`rust/http.rs:1566-1579`). A protocol states its own answer for some of them
(RFC 6749 §5.2, `invalid_request`, `invalid_client`). Top-level `refusals:` takes `not_granted`,
`unfinished` and `malformed`, each with `status`, a literal body and headers. A refusal not listed
keeps today's answer.

### What compiling checks (stage 2)

| code | refused | names |
|---|---|---|
| `ESS-HTTP-020` | an encoding outside the set; `form` or `json` on `get`; `query` on anything but `get` | `routes[i].request.encoding` |
| `ESS-HTTP-021` | a `form` or `query` field whose type has no flat spelling (struct, nested list, map, `Json`) | the field and its type |
| `ESS-HTTP-022` | an `outcomes:` key that is not an outcome of the command, or `outcomes:` on a view | `routes[i].outcomes.<name>` |
| `ESS-HTTP-023` | a status outside `100..=599`, or in a class the outcome may not use | the outcome, the status, the class |
| `ESS-HTTP-024` | a body form not admitted for the outcome or operation (`response` on an outcome without `returns: true`, `{…: error}` without `error`, a body on `204` or `304`, `single` on a command) | the outcome and the form |
| `ESS-HTTP-025` | two outcomes on one status that the bodies cannot tell apart | both outcomes and the status |
| `ESS-HTTP-026` | a header name that is not a token, a header the encoding or `redirect` owns, a header declared twice ignoring case, a source that names no field or names a non-scalar | the header and the source |
| `ESS-HTTP-027` | a challenge whose scheme or param name is not a token, or a param declared twice | the header, the param |
| `ESS-HTTP-028` | `redirect` without a `3xx` status, a `3xx` without `redirect`, a `location` that is not a `String`, a `mode` outside the set, a parameter declared twice | the outcome and the key |
| `ESS-HTTP-029` | a `refusals:` key outside the set, or a status in the wrong class for it | `refusals.<key>` |
| `ESS-HTTP-030` | `body: single` without `empty`, or `empty` without `single` | `routes[i]` |

A refinement may not change a refined route's stage-2 keys either; that is `ESS-HTTP-012`.

### What generated artifacts change (stage 2)

- **OpenAPI.** `requestBody` takes the declared media type. Each declared status gets its own
  response: the bare schema, the error object, or no content; headers as response `headers`, with a
  redirect's `Location`. The `outcome` discriminator is gone from a bare body, so each response
  carries `x-ess-outcomes` naming the outcomes it answers.
- **Rust and Go servers.** Form and query decoders; status, header, challenge and redirect writers.
  The status a server writes today comes from `http::outcome_status` (`rust/http.rs:1244`,
  `go/http.rs:1546-1557`); a binding's status replaces it in the same place, so the document and the
  servers still read one answer.
- Untouched: the transport-free entry points, emitted conformance runners, the web bridge and the
  interpreter, none of which speaks HTTP.

### Formats and determinism (stage 2)

The format stays `ess-http/1` and `ess-http-ir/1`. A document that uses no stage-2 key compiles to
the same IR bytes under a stage-2 reader: absent keys are omitted, never written as defaults. An
OpenAPI document generated from such a binding keeps its stage-1 bytes. Header and parameter maps
are `BTreeMap`s, so the emitted order is the name order, not the authored order.

## Stage 3: an HTTP conformance target

Today the runner is semantic only. `ConformanceTarget::execute_command` takes a
`SemanticCommandRequest` (`crates/verify/ess-conformance/src/target.rs:183-186`) and `query_view` a
`SemanticViewRequest` (`target.rs:225`). `--target` takes `billing`, `oracle-fixture` and
`interpreted` (`main.rs:1089-1097`). The out-of-process adapter is "the named open gap"
(`docs/design/running-a-suite.md:71-80`).

Stage 3 adds a target that implements that trait over HTTP:

```console
ess verify conform run --path MODEL --target http --http server-http.yaml \
  [--refines protocol-http.yaml] --base-url https://catalog.example
```

- **Command.** Encode the input per the route, send it to the declared method and path, and map
  status, body and headers back to the outcome. Stage 2's distinguishability rule
  (`ESS-HTTP-025`) is what makes that mapping total.
- **View.** Send the parameters as path segments and a query string, and read `rows` or `single`.
- **What a bare body cannot show.** Events a branch published, and a response the protocol does
  not return. A scenario that asserts one of these is counted as unobservable on this target, not as
  passed.
- **Refusals by name.** An open route; a binding whose specification digest differs from the
  suite's; an operation the suite exercises that the binding cannot reach.

Formats: the `ess-conformance/*` documents do not change. `--target` gains one value, and
`ess-http-ir/1` is the target's only new input.

## Still refused

- The `http:` key on a command or view in `ess/N`. The binding replaces it.
- Any key in `ess/N`, `ess-realization` or `ess-transport` that carries a route.
- Non-JSON response bodies, and partial-segment path templates, in `ess-http/1`.
- `--http` with an artifact that cannot honour it: `ess ui`, the clap, web and TypeScript targets,
  and any `--kind` other than `openapi`. Each refuses it by name rather than ignoring it.
- Authentication: the binding states a status and a challenge, not how a caller proves who it is.

## Decided for stage 1 (2026-10-11, `decision-blocker:ess-060-wave-scope`)

1. The OpenAPI projection refuses a binding with an open route, naming the operation. Emitting a
   marked server-chosen path stays possible later as an addition.
2. A binding cannot require that it routes every operation in stage 1; an unnamed operation keeps
   its derived route.
3. A literal segment does not take precedence over a variable: `ESS-HTTP-011` refuses the overlap.
4. `refines:` is the digest of the refined file's bytes, as the specification's source pin is.
   Reformatting a refined file is a deliberate re-pin.
5. `ess verify diff` does not compare two bindings in stage 1; that is one unit of stage 2.
6. A view is bound to `get` only in stage 1.

## Open questions

These could not be settled from the sources. Each needs a decision before the stage it names.

7. **Stage 2.** What does a `single` view answer when it holds more than one row?
8. **Stage 2.** Should a binding be able to declare `securitySchemes` once a challenge is declared,
   or does that wait for an authentication construct in the model?
9. **Stage 2.** Which error fields does `{<member>: error}` admit: the error's fields under their
   wire names unchanged, or does `error_description` (RFC 6749 §5.2) need its own mapping?
10. **Stage 3.** How does the HTTP target begin each scenario on an isolated state
    (`target.rs:175-180` leaves "how" to the target) and seed a scenario's initial records, against
    a server it does not control?
11. **Stage 3.** How does the target present itself as a given actor, so grant and `caller.`
    scenarios run? The credentials belong to the realization, which this format does not read.
12. **Stage 3.** Does the conformance run report name its target in a way that admits `http`
    without a format change?
13. **All stages.** The fit review reopens `story:a-resource-has-an-http-shape-of-its-own` and
    `story:a-conformance-suite-runs-against-any-realization`, excluded on 2026-09-15. Which of their
    acceptance items this page now owns, and which stay archived, is not settled here.
