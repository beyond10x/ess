---
format: aep.planning-md/3
id: story:an-http-route-may-read-an-input-field-or-a-view-parameter-from-a-request-header
kind: story
status: draft
title: An HTTP route feeds an input field or a view parameter from a request header that its protocol document declares
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-http-binding-declares-how-a-caller-authenticates
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
- depends_on: story:an-ess-protospec-1-reply-answers-on-the-exchange-of-the-message-it-received-and-a-credential-header-is-named
- depends_on: story:an-ess-http-1-binding-names-the-protocol-document-its-routes-read-by-its-compiled-digest
revision: 3
---
**Revised on 2026-11-24 after the owner's note.** A header is a protocol concept. The protocol
document declares the request header (name, type, presence). The binding only says which input
field or view parameter it feeds. A credential header may be named by the protocol document but is
never fed to an input: it stays the authentication port's (AC-D3).

## Need

A real HTTP service reads some request values from headers: a request id (`X-Request-Id`), an
idempotency key, a tenant, an SSE client's `Last-Event-ID`. An `ess-http/1` route places an input
only in the path (`parameters`), the query (`request.encoding: query`) or the body (`request.body`
or the JSON object). No header is read anywhere.

- `RequestSpec` has `encoding`, `unknown` and `body` only, and `deny_unknown_fields`
  (`crates/specify/ess-http/src/lib.rs:274-288`). `RouteSpec` has no request-header key
  (`lib.rs:1561-1610`).
- The design page states headers only on the response (`docs/design/http-presentation-binding.md:485-507`).
  The generated Rust server keeps request headers only for the realization's shell
  (`crates/generate/ess-synth/src/rust/http.rs:1864-1869`).

The reporter's case was a bearer token in `Authorization` (third report, finding 3). Under AC-D3
(default A, still open) that header is not an input. How a caller proves who it is belongs to the
binding's authentication port, which hands the model an actor and caller attributes, never the
credential (`architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`,
accepted; `story:an-http-binding-declares-how-a-caller-authenticates`, active). This story is the
generic part: every other header.

What exists, and is not restated here: the command's input fields and the view's parameters (the
semantics), and the route's path, query and body (the binding). The header's name, type, presence
and text encoding are protocol facts under the note. They come from `pr0-protocol-message-structure`,
`pr0-protocol-reply-exchange-and-credential-header` (credential headers) and
`pr0-http-binding-names-a-protocol` (the link).

Minimal reproduction (written fresh, not run). Command `demo.orders.PlaceOrder(request_id: String,
item: String)`. The protocol document `orders-api` declares:

```yaml
messages:
  - name: place_order
    headers:
      - {name: X-Request-Id, type: String}
      - {name: Authorization, type: String, credential: true}
    application: body
```

The binding:

```yaml
protocol: "sha256:<compiled digest of orders-api>"
routes:
  - command: demo.orders.PlaceOrder
    method: post
    path: /orders
    authentication: [bearer_auth]
    request:
      message: place_order
      headers: {X-Request-Id: input.request_id}
```

```console
$ ess specify http validate --path spec --binding orders.http.yaml --protocol orders-api.yaml
error: reading orders.http.yaml as HTTP binding YAML: routes[0].request: unknown field `message`, expected one of `encoding`, `unknown`, `body` …
```

## Outcome

- **What the route names.** `request.message` names the protocol message whose headers this route's
  request carries. `request.headers` maps each of its non-credential headers to `input.<field>` on
  a command route, or to `param.<name>` on a view route.
- **What stays in the binding.** Path, query, the body's encoding and response headers stay in
  `ess-http/1`. The message's application slot is the body the route's encoding writes. On HTTP the
  message declares no body `fields`.
- **The field.** A field fed from a header leaves the body and the query, as a path-mapped field
  does. A `required: false` header feeds an `Optional` field and is absent when the field is. A
  required header that is missing is `malformed`. The header is written as text by the
  protocol-structure rule (`Integer` in decimal, `Boolean` as `true` or `false`).
- **The security boundary.** A header with `credential: true` is never mapped. It must be read by
  one of the route's `authentication` schemes: `bearer` and `basic` read `Authorization`, and
  `api_key` with `in: header` reads its `name`. The generated servers hand it to the
  authentication port and never decode it into model data. Any other mapped header is
  caller-supplied data, with the trust of a body field. A value the system trusts as identity (a
  tenant asserted by a gateway, a user id) reaches the model only through the authentication port
  as caller attributes, and never through this key. HTTP's own headers stay refused as owned by the
  server, the encoding or authentication: `Authorization`, `Proxy-Authorization`, `Cookie`, `Host`,
  `Connection`, `Content-Type`, `Content-Length`, `Content-Encoding`, `Transfer-Encoding`, `Expect`.
  The protocol document declares only `Authorization` among them, and only as a credential.
- **Projections.** The OpenAPI document writes an `in: header` parameter per mapped header. The
  Rust and Go servers read the header, and the generated UIs send it.
- **Conformance.** The suite is unchanged: no scenario reads a header, and the input field is the
  same field. Running it over HTTP is a `--target process` adapter, or the stage-3 HTTP target once
  it exists (PR-D7 A).

## Acceptance

Unit a (binding, IR, OpenAPI):

- **Red first.** With `--protocol orders-api.yaml`, the binding above validates and compiles.
  `ess-http-ir/1` writes `request.message` and the resolved `request.headers` (name, field, type,
  required) only when declared, so every existing binding compiles to the same bytes. The cases are
  in `crates/specify/ess-http/tests/request_encoding.rs` (existing binary) and
  `crates/edge/ess-cli/tests/http_presentation.rs`.
- **View routes.** `request: {message, headers}` is admitted on a view route, and every other
  `request` key stays refused there (`ESS-HTTP-020` narrowed).
- **Refusals.** Each is refused with one new code (the next free `ESS-HTTP-0NN` when the unit
  lands), naming the key:
  - a `request.message` the protocol does not declare, or one with body `fields`;
  - a mapped name that is not a header of that message;
  - a non-credential header of the message left unmapped;
  - a credential header mapped to an input or parameter, with a message that points at
    `authentication`;
  - a credential header that no `authentication` scheme of the route reads;
  - a header name HTTP owns (the list above) declared without `credential`, or `Authorization`
    declared without it;
  - a source of another namespace, or one naming no field;
  - a field whose type does not lower to the header's scalar (`String`, `Integer`, `Boolean`; an
    enum or newtype lowers to its base);
  - a required header feeding an `Optional` field, or the reverse;
  - a field the path or a bare `request.body` already carries.
- **OpenAPI.** `ess generate --kind openapi --http … --protocol …` writes each mapped header as
  `{in: header, name, required, schema}`, leaves the field out of the request body schema, and
  writes no parameter for a credential header (the security scheme carries it). A binding without
  `request.message` keeps its bytes. The case is in
  `crates/generate/ess-gen/tests/openapi_http_presentation.rs`.
- **Docs.** `docs/design/http-presentation-binding.md` gains § "Request headers", which states the
  boundary above and PR-D6, and `website/docs/reference/formats.md` lists the keys.

Unit b (servers):

- The generated Rust server reads each mapped header from the IR. A value that does not decode to
  the field's type, the header given twice, or a required header missing is `malformed` (the
  binding's `refusals.malformed` where declared). The value is never echoed into an error body. A
  credential header is passed to the authentication port only.
- The Go server answers each of these cases with the same status, headers and body bytes
  (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases). `http_binding_rust` holds
  both.

Unit c (generated UIs):

- The React and terminal UIs that call a bound route send the field's value in its header and not
  in the body. A UI whose `bind` gives no value for an `Optional` header field sends no header.

Each test runs within 60 s, and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RequestSpec` 274-288, `RouteSpec` 1561-1610)
- cited: `crates/specify/ess-http/tests/request_encoding.rs`, `crates/edge/ess-cli/tests/http_presentation.rs`
- cited: `crates/generate/ess-gen/src/openapi.rs`, `crates/generate/ess-gen/tests/openapi_http_presentation.rs`
- cited: `crates/generate/ess-synth/src/rust/http.rs` (request headers 1864-1869), `crates/generate/ess-synth/src/go/http.rs`,
  `crates/generate/ess-synth/tests/http_binding_rust.rs` (unit b)
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs` (unit b)
- inferred: the request builders of `crates/ui/ess-ui-react` and `crates/ui/ess-ui-tui` (unit c)
- cited: `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: unit a about 480 changed lines (M), unit b about 400 (M), unit c about 150 (S).

## Gate

- `task fmt-check`
- `cargo clippy -p ess-http -p ess-cli -p ess-gen --all-targets --locked -- -D warnings` (unit a);
  add `-p ess-synth` (unit b), and `-p ess-ui-react -p ess-ui-tui` (unit c)
- `cargo test -p ess-http --locked --test request_encoding`
- `cargo test -p ess-cli --locked --test http_presentation`
- `cargo test -p ess-gen --locked --test openapi_http_presentation`
- `cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6` (byte pin)
- unit b: `cargo test -p ess-synth --locked --test http_binding_rust` and `--test http`
- unit c: `cargo test -p ess-ui-react --locked` and `cargo test -p ess-ui-tui --locked`, one
  `--test` at a time where a step would pass 120 s
- `task ci-lint`
- `task test-xtask` (new code, docs)

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `pr0-protocol-message-structure`, `pr0-protocol-reply-exchange-and-credential-header` and
  `pr0-http-binding-names-a-protocol`.
- PR-D1, PR-D2, PR-D3 and PR-D5 (architecture). PR-D6.
- `story:an-http-binding-declares-how-a-caller-authenticates`: the credential rule reads its
  schemes. If unit a lands first, the rule "a credential header that no scheme reads" refuses every
  credential header, and it relaxes when the schemes land.
- Units b and c come after unit a.
- It shares `crates/specify/ess-http/src/lib.rs` with the SSE binding's unit a, `sl01-member-body`
  and the authentication story.

## Fit review

1. **Need.** A request value carried in a header cannot be bound to the model. Reproduction above.
   Report: third report, finding 3 (the generic part). The reporter asked for `Authorization` as an
   input. That is decided under AC-D3, and this story does not change it.
2. **Class: gap.** Path, query and body are bound. Headers are not (`lib.rs:274-288`).
3. **Already expressible?** No. Moving the value into the body or the query changes the contract.
4. **Fit.**
   - Under the note, the header is declared by the protocol kind and fed by the binding.
   - It reuses the `parameters` shape (`input.<field>` or `param.<name>`), the owned-header rule of
     response headers (`ESS-HTTP-026`), and the flat-scalar rule of `form` and `query` fields
     (`ESS-HTTP-021`).
   - Siblings: command and view routes, since a tenant parameter in a header is the same question.
     The SSE binding's `Last-Event-ID`, when a resume is designed, is a header of its request
     message.
   - Bindings are outside the language grids.
5. **Second adopter.** A payment API whose `Idempotency-Key` header becomes
   `CreatePayment.idempotency_key`, guarded as unique by the model.
6. **Cost.** Two optional keys in `ess-http/1` and `ess-http-ir/1`, absent when undeclared, so no
   format version. One new code. A reader per mapped header in each generated server.
7. **Considered.**
   - (a) Change nothing.
   - (b) `request.headers` naming the header in `ess-http/1` (the earlier body, PR-D5 B).
   - (c) Response headers move too (PR-D5 C, `ess-http/2`).
   - (d) A header place in the model (`ess/N`): wire places stay out of the model.
   - (e) Admit `Authorization` as an input: AC-D3 B.
   - Chosen: PR-D5 A.

## Decisions

**Accept, redesigned again.**

- PR-D5 (architecture, default A): the header is declared in the protocol document.
- PR-D6 (default A): response headers stay in `ess-http/1`.
- PR-D1, PR-D2 and PR-D3 (architecture).
- AC-D3 (credential headers are never inputs, default A) is unchanged and still open. AC-D4 and
  AC-D5 are untouched and still open.

## Source

A downstream application, third report, finding 3 (ESS 0.98.0).
