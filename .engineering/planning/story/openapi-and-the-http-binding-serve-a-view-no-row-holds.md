---
format: aep.planning-md/3
id: story:openapi-and-the-http-binding-serve-a-view-no-row-holds
kind: story
status: draft
title: OpenAPI and the HTTP binding serve a view no row holds
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#674
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-may-answer-values-no-row-holds
revision: 2
---
## Need

A view with no source (`story:a-view-may-answer-values-no-row-holds`) must appear in the projected
documents and be bindable to a route, as a sourced single-row view is: a limits endpoint is a `GET`
that answers one object. Until this story, `ess-gen` and `ess-http` refuse such a view by name.

## Acceptance

- `ess generate --path model --kind openapi` on the reproduction's `Limits` view emits its row
  schema `{max_tags: integer, api_level: integer}`, both required; `--kind schema` emits the same row
  schema.
- `ess specify http validate` accepts `{view: example.library.Limits, method: get, path: /limits,
  body: single}` (exit 0); `body: list` and `empty:` on a sourceless view are refused by name (it
  always answers exactly one row); OpenAPI shows the `200` with the row object.
- A view with a `setting.` field over a source projects that field with the field's declared type.
- Specifications and bindings without such views keep their bytes.
- Tests in `crates/generate/ess-gen/tests/` and `crates/specify/ess-http/tests/` (one file each).

## Scope

- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/src/schema.rs (row schema of views)
- cited: crates/specify/ess-http/src/lib.rs (view routes, `body`, `empty`; `EmptySpec` 781)
- inferred: crates/specify/ess-http/tests/sourceless_view.rs (new)
confidence: medium

## Fit review

1. **Need.** Publish and route a view no row holds.
2. **Class: gap** (follows from the construct).
3. **Already expressible?** No.
4. **Fit.** Reuses `body: single` of view routes; the row schema is the field list as for every
   view. Refuses the keys that assume rows (`empty:`, `body: list`) by name.
5. **Second adopter.** A capabilities endpoint served over HTTP.
6. **Cost.** No format: `ess-http/1` gains no key, only refusals on a new kind of view.
7. **Alternatives.** (a) Leave projections refusing: the endpoint the issue asks for is unservable.
   (b) Chosen.

## Decisions

**accept, redesigned**, as the parent story (Amendment 1 of ADR
`a-command-guard-reads-a-component-setting`).

## Source

github:beyond10x/ess#674
