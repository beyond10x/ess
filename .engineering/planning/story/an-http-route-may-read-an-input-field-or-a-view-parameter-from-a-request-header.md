---
format: aep.planning-md/3
id: story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header
kind: story
status: draft
title: An HTTP route may read an input field or a view parameter from a request header
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-http-binding-declares-how-a-caller-authenticates
revision: 1
---
## Need

A real HTTP service reads some request values from headers: a request id (`X-Request-Id`), an
idempotency key, a tenant, an SSE client's `Last-Event-ID`. An `ess-http/1` route can place an
input only in the path (`parameters`), the query (`request.encoding: query`) or the body
(`request.body`, the default JSON object). A header is read nowhere:

- `RequestSpec` has `encoding`, `unknown` and `body` only, with `deny_unknown_fields`
  (`crates/specify/ess-http/src/lib.rs:274-288`). `RouteSpec` has no request-header key
  (`lib.rs:1561-1610`).
- The design page states headers only on the response (`docs/design/http-presentation-binding.md`
  § "Response headers"). The generated Rust server keeps request headers only for the
  realization's shell: "the model declares no header, so routing never looks at one"
  (`crates/generate/ess-synth/src/rust/http.rs:1864-1869`).

The reporter's case was a bearer token in `Authorization` (third report, finding 3). That header
is deliberately not an input, and stays so under **AC-D3** (default A): how a caller proves who it
is belongs to the binding's authentication port, which hands the model an actor and caller
attributes, never the credential
(`architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`, accepted;
`story:an-http-binding-declares-how-a-caller-authenticates`, active). This story is the generic
part of the finding: every other header.

Minimal reproduction (written fresh; not run): command `demo.orders.PlaceOrder(request_id:
String, item: String)`.

```yaml
routes:
  - command: demo.orders.PlaceOrder
    method: post
    path: /orders
    request: {headers: {X-Request-Id: input.request_id}}
```

```console
$ ess specify http validate --path spec --binding orders.http.yaml
error: reading orders.http.yaml as HTTP binding YAML: routes[0].request: unknown field `headers`, expected one of `encoding`, `unknown`, `body` …
```

## Outcome

`request.headers` maps a request header name to `input.<field>` on a command route and to
`param.<name>` on a view route. A field read from a header leaves the body and the query, as a
path-mapped field does. An absent header leaves an `Optional` field absent, and is `malformed`
for a required field. The OpenAPI document writes an `in: header` parameter. The Rust and Go servers
read the header, and the generated UIs send it.

## Acceptance

Unit a (binding, IR, OpenAPI):

- The binding above validates and compiles. `ess-http-ir/1` writes `request.headers` only when
  declared, so every existing binding compiles to the same bytes. `crates/specify/ess-http/tests/request_headers.rs`
  (new) starts from the refused document above.
- `request: {headers: …}` is admitted on a view route, and every other `request` key stays refused
  there (`ESS-HTTP-020` narrowed).
- Refused with one new code (the next free `ESS-HTTP-0NN` when the unit lands; `034` is taken by
  `limits`), each naming the key:
  - a header name that is not a token, or one declared twice ignoring case;
  - a header the server, the encoding or authentication owns: `Authorization`,
    `Proxy-Authorization`, `Cookie`, `Host`, `Connection`, `Content-Type`, `Content-Length`,
    `Content-Encoding`, `Transfer-Encoding`, `Expect`, and the header an `api_key` scheme of the
    binding names. For `Authorization` the message points at `authentication`;
  - a source of another namespace, one naming no field, or a field whose type is not a scalar or an
    `Optional` scalar (`String`, `Integer`, `Boolean`, an enum, or a newtype of one);
  - a field the path or a bare `request.body` already carries.
- `ess generate --kind openapi --http` writes each as `{in: header, name, required, schema}` and
  leaves the field out of the request body schema. A binding without it keeps its bytes.
- `docs/design/http-presentation-binding.md` gains § "Request headers", and `website/docs/reference/formats.md`
  lists the key.

Unit b (servers):

- The generated Rust server reads each declared header: a value that does not decode to the field's
  type, the header given twice, or a required header missing is `malformed` (the binding's
  `refusals.malformed` where declared). The value is never echoed into an error body.
- The Go server answers each of these cases with the same status, headers and body bytes
  (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases). `http_binding_rust` holds
  both.

Unit c (generated UIs):

- The React and terminal UIs that call a bound route send the field's value in its header and not
  in the body. A UI whose `bind` gives no value for an `Optional` header field sends no header.

Each test runs under 60 s, and each test step within 120 s.

## Fit review

1. **Need.** A request value carried in a header cannot be bound to the model. Reproduction above.
   Report: third report, finding 3 (the generic part). The reporter asked for `Authorization` as an
   input, which is decided under AC-D3.
2. **Class: gap.** Path, query and body are bound. Headers are not (`lib.rs:274-288`).
3. **Already expressible?** No. Moving the value into the body or query changes the contract, and
   the suite would check a request the real service never accepts.
4. **Fit.** Not a source-language cell (bindings are outside the grids,
   `docs/design/language-grids.md`). Reuses the `parameters` shape (`<wire place>: input.<field>`
   or `param.<name>`), the response headers' token and owned-header rules (`ESS-HTTP-026`), and the
   flat-scalar rule of `form`/`query` fields (`ESS-HTTP-021`). Siblings: command and view routes
   both, since a view parameter in a header (a tenant) is the same question. `ess-cli/2` has
   no headers. The SSE story's `Last-Event-ID`, which "waits for an exchange that states one"
   (`story:an-event-channel-may-be-bound-to-server-sent-events`), can use this key when its
   exchange states one.
5. **Second adopter.** A payment API whose `Idempotency-Key` header becomes
   `CreatePayment.idempotency_key`, guarded as unique by the model.
6. **Cost.** One optional key in `ess-http/1` and `ess-http-ir/1`, absent when undeclared, so no
   format version. This follows the precedent of `request.body`, `operation_id`, `servers` and
   `limits` (design page: "No format version: a reader without the key refuses … as an unknown
   field"). One new diagnostic code. Generated servers gain a reader per declared header.
7. **Considered.** (a) Change nothing. (b) Chosen: `request.headers`. (c) A header place in the model
   (`ess/N`). This was rejected: wire places live only in the binding (design page § "Still refused").
   (d) Admit `Authorization`. Put to AC-D3.

## Decisions

Accept, redesigned. The generic header binding is adopted. `Authorization` and the other credential
headers stay out under AC-D3 (default A). No other open question.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RequestSpec` 274-288, `RouteSpec` 1561-1610)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- cited: `crates/generate/ess-synth/src/rust/http.rs` (request headers 1864-1869), `crates/generate/ess-synth/src/go/http.rs`
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`
- inferred: `crates/ui/ess-ui-react` and `crates/ui/ess-ui-tui` request builders (unit c)
- cited: `docs/design/http-presentation-binding.md`
- inferred: `crates/specify/ess-http/tests/request_headers.rs` (new)
- Size: unit a about 450 lines, unit b about 400, unit c about 150. Each is M or S.

## Depends on

- `story:an-http-binding-declares-how-a-caller-authenticates`: the owned-header list reads its
  `api_key` header name. If unit a lands first, the clause is added when the scheme lands.
- Units b and c after unit a.

## Source

A downstream application, third report, finding 3 (ESS 0.98.0).
