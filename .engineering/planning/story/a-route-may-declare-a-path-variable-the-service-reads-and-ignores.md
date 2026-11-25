---
format: aep.planning-md/3
id: story:a-route-may-declare-a-path-variable-the-service-reads-and-ignores
kind: story
status: draft
title: A route may declare a path variable that the service accepts and ignores, as the model reads no parameter from it
summary: ESS-HTTP-008 refuses a variable no source reads and ESS-VIEW-003 a parameter no filter reads (shape 8); the filter on the parent is the stricter model; HB-D3
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
revision: 2
---
## Summary

A real service routes `/parents/{parentId}/items/{id}` and reads only `id`: any `parentId` answers the
same rows. An `ess-http/1` route must map every variable of its path to a source the model reads, so
the route cannot be bound, and a view parameter no filter reads is refused. A downstream application
asks for a way to declare a path segment as accepted and ignored. The honest model is a filter on the
parent, which the service does not check; the story records both ways and the decision HB-D3 between
them. The route key is recommended, because a specification of a shipped service must describe what it
does.

## Facts (tag 0.99.1 read; the first refusal run with `ess` 0.98.0 on a toy model)

1. Every variable maps once. The route `get /parents/{parent_id}/items/{item_id}` with only
   `parameters: {item_id: param.item_id}` is refused `ESS-HTTP-008` ("variable `parent_id` of
   `/parents/{parent_id}/items/{item_id}` is not mapped; each variable of the path reads one source
   under `parameters`", `crates/specify/ess-http/src/lib.rs:5456`; table row at
   `docs/design/http-presentation-binding.md`, "What compiling checks"). A mapped source is
   `input.<field>` or `param.<name>` of a scalar a path segment can carry (`ESS-HTTP-009`).
2. A view parameter no filter reads is refused `ESS-VIEW-003` ("declares the parameter `parentId` and no
   filter reads it, so every value of it returns the same rows"), as the reporter shows. The rule is
   the model's: a parameter that selects nothing is a statement the model never makes.
3. Two URLs answering the same rows is not a conflict ESS refuses elsewhere: a request path matches
   one route (`ESS-HTTP-010` and `-011` decide which operations may share a template), and a segment
   that no filter reads is the same as a free text segment.
4. The ignoring precedent is on the request body. A JSON request may be declared to ignore undeclared
   members and echoed path fields (`story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields`,
   active; `request: {unknown: ignore}` on a form route, `crates/generate/ess-synth/tests/fixtures/http-binding-shop.routes.yaml`
   at 0.99.1). A path variable has no such word.
5. The modelling form: the item carries its parent (a reference or a parent field) and the view filters
   by both (`parent_id == param.parent_id` and `item_id == param.item_id`). Then `parentId` is read,
   the route validates, and a decoy-parent read is a scenario the suite can send: a service that
   ignores the parent answers the item and the scenario fails. That failure is the divergence between
   the specification and the shipped service, reported by the suite. It is the right model for a
   service the specification is meant to constrain, and the wrong one for a specification that records
   what a shipped service does.

## Acceptance

1. Red first (after HB-D3 B): `a_path_variable_may_be_declared_ignored`, a new module of `ess-http`'s
   integration tests. `parameters: {parent_id: ignored}` beside `{item_id: param.item_id}` validates and
   compiles. The IR writes the variable as ignored, only when declared, so every existing binding
   compiles to its bytes; a binding with the key is read by no earlier reader, which refuses it as an
   unknown value (no format version: optional, absent when undeclared).
2. Refusals, one new code (the next free `ESS-HTTP-0NN`): `ignored` for a name the path has no
   variable of (as `ESS-HTTP-008` refuses any mapping of it), and `ignored` for every variable of a
   route (nothing then names a row). The reserved word `ignored` collides with no source namespace
   (`input.` and `param.`).
3. OpenAPI writes the variable as a required `in: path` string parameter with `x-ess-ignored: true`
   and a description saying the service accepts any value; `ess generate --kind openapi` pins it in
   `crates/generate/ess-gen/tests/openapi_http_presentation.rs`.
4. The generated Rust and Go servers match the route with any non-empty segment at that position, read
   no value from it, and answer as the route without it; a request with an empty segment is not that
   route. `the_go_and_rust_servers_answer_one_binding_alike` gains one case.
5. Conformance: the suite sends one fixed value for the ignored segment and does not witness a read of
   it. The compiled IR lists every ignored variable, and `ess specify http validate` names each beside
   its count of open routes, so the statement is visible and not buried in the binding.
6. Documentation: `docs/design/http-presentation-binding.md` § "Path templates" states the key and the
   model that loses it (Fact 5: the filter on the parent is stricter and a suite can tell the service
   from the specification, an ignored segment is not); the `ESS-HTTP-008` text for an unmapped
   variable says the two ways.
7. Each test runs within 60 s and each test step within 120 s.

## Fit review

1. Need: bind a route whose path holds a segment the service does not read.
2. Class: a gap in `ess-http/1` if the specification may record a shipped service's behaviour; the
   model answer exists (Fact 5) and is stricter than the service.
3. Covers: shape 8 of the twentieth report. The reporter's wording ("accepted and ignored") is evidence,
   not the design.
4. Borders: `story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields` (the same
   statement for body members), `story:a-view-selects-rows-by-a-caller-attribute` (a tenant segment
   that a service ignores in favour of the credential is the case this key must not be used for: the
   decision of the caller story, HC-D1), `story:an-optional-input-may-select-between-an-operations-routes`
   (a path variable present on one route and absent on another). A search of the store and the open
   issues found no story on an ignored path variable.
5. A second adopter: a retrofit of any service whose nested route keeps a parent segment for URL
   shape only.

## Decisions

- HB-D3 (shape 8): how a path variable the model does not read is bound.
  - A. Decline: the model reads it, as a filter on the parent (Fact 5). The suite then tells a service
    that ignores the parent from the specification. Cost: a specification of a shipped service that
    ignores the parent fails its own conformance run until the service or the specification changes;
    the retrofit states a behaviour the service lacks.
  - B. A mapping word `ignored` for a variable of the path (Acceptance 1 to 6): recorded in the IR and
    OpenAPI, accepted by the servers, named by `validate`. Cost: one optional key and one
    code, a validate line, and a statement of a behaviour the model does not constrain; a reader may
    take the segment for a scope it is not (the caller story's case, HC-D1, stays refused: a segment
    that carries a tenant is never `ignored` for the credential's sake, which no code can decide, so
    the documentation says it).
  - C. B with a required reason text per ignored variable, carried to the IR and the OpenAPI
    description. Cost: B's, a longer binding, and a text the validator cannot check.
  - Recommended B. A specification records what a shipped service does, `ess:retrofitting` cites a
    source for every declaration, and `validate` names the ignored segment where a filter on
    the parent would claim a check the service does not make.

## Scope

- `crates/specify/ess-http/src/lib.rs` (the `parameters` check 5440-5490, the IR writer)
- `crates/specify/ess-http/tests/` (a new module in the existing integration-test binary)
- `crates/generate/ess-gen/src/openapi.rs`, `crates/generate/ess-gen/tests/openapi_http_presentation.rs`
- `crates/generate/ess-synth/src/rust/http.rs`, `src/go/http.rs`, `tests/http_binding/go.rs`
- `docs/design/http-presentation-binding.md`, `website/docs/reference/diagnostics.md`, `CHANGELOG.md`
