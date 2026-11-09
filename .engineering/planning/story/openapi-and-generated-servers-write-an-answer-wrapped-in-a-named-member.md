---
format: aep.planning-md/3
id: story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member
kind: story
status: draft
title: OpenAPI and generated servers write an answer wrapped in a named member
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#755
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-answer-may-be-wrapped-in-one-named-member
revision: 2
---
## Need

A route whose body is `{<member>: single | array | response | response.<field>}`
(`story:an-answer-may-be-wrapped-in-one-named-member`) must be published and served as that
object. Today OpenAPI writes the bare row, array or response schema for the bare forms
(`crates/generate/ess-gen/src/openapi.rs`, § "What OpenAPI writes (k4)" of
`docs/design/http-presentation-binding.md`), and the Rust and Go servers write the bare value
(`crates/generate/ess-synth/src/rust/http/bound.rs:1266-1288` for views, `outcome_answer` for
outcomes; the Go twin in `crates/generate/ess-synth/src/go/http/bound.rs`).

Neutral example: view `demo.apps.AppById(uuid, name)` on `get /apps/one` with `body: {app: single}`,
`empty: {status: 404}`; view `demo.apps.Apps` on `get /apps` with `body: {items: array}`.

## Acceptance

- OpenAPI: the `200` response of `get /apps/one` is `{type: object, required: [app], properties:
  {app: <the row schema>}, additionalProperties: false}`, its `404` as `empty` says; `get /apps` is
  the same object over `items` with the array schema; an outcome's `{data: response}` wraps
  `{command}.{outcome}.Response`. A binding without a wrapped form keeps its OpenAPI bytes.
- Rust server: `GET /apps/one` answers `{"app":{"uuid":"…","name":"…"}}` and, holding no row, the
  `empty` answer; `GET /apps` answers `{"items":[…]}`, `{"items":[]}` when empty; an outcome with
  `{data: response}` answers `{"data":{…}}`.
- The Go server answers each request with the Rust server's status, headers and body bytes
  (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases); a binding without a wrapped
  form generates byte-identical server source.

## Scope

- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- inferred: crates/generate/ess-synth/tests/http_binding/answer_forms.rs
confidence: medium

## Fit review

1. **Need.** The wrapped answer the binding states must be what the document publishes and the
   servers write. Reproduction: the binding story's (the form is refused today).
2. **Class: gap**, the projection and server half of the wrapper of beyond10x/ess#755.
3. **Expressible today? No.**
4. **Fit.** Each wrapped form reuses its bare form's schema and writer and adds one object around
   it; Rust and Go alike, as k6 requires. Two crates in one story because each side is a few lines
   around an existing writer; split per crate if the implementor finds otherwise.
5. **Second adopter.** The payments API's `{"data": …}` of the binding story.
6. **Cost.** Bytes move only for routes with a wrapped form.
7. **Alternatives.** (a) Projection without servers, or the reverse: one target would silently
   ignore the form.

## Decisions

**Accept, redesigned**, as decided in `story:an-answer-may-be-wrapped-in-one-named-member`.

## Source

github:beyond10x/ess#755
