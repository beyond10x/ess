---
format: aep.planning-md/3
id: story:a-view-field-nests-source-fields-under-a-struct-member
kind: story
status: draft
title: 'A view field nests source fields under a struct member (field: nested mapping)'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#760
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/from.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/navigation.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/view_nested_field.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: docs/design/value-expressions.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/guides/specify/values-and-views.md
revision: 4
---
## Need

A read answers a row that nests some of its flat source fields under one member,
`{"content": "…", "metadata": {"id": "a1", "name": "report"}}`, while the stored record is flat
(`id`, `name`, `content`). A view field names one source field, one navigated field
(`field: {related: {via, field}}`, `ess/28`) or one scalar expression (`expr:`, `ess/27`)
(`crates/specify/ess-domain/src/view.rs:917-943`). No view field builds a struct from several
source fields. `payload:` and `sets:` already do: a nested mapping fills a struct target, "every
struct field has a source" (`website/docs/guides/specify/values-and-views.md:43`;
`docs/design/value-expressions.md` § E5).

Minimal reproduction (0.92.0, `ess/36`): struct `example.files.Meta` (`id`, `name`), entity
`example.files.File` (identity `id: String`, fields `name`, `content`), view
`example.files.FileDetail` with fields `content` and `metadata: example.files.Meta`
(`repro/run.txt`):

```text
{name: metadata, type: example.files.Meta}
  -> ESS-VIEW-001 `example.files.File` has no field `metadata`, so `example.files.FileDetail` promises an observation nothing produces
{name: metadata, type: example.files.Meta, expr: "{id: id, name: name}"}
  -> ESS-SPEC-012 `{` is no part of an expression
{name: metadata, type: example.files.Meta, from: {id: id, name: name}}
  -> ESS-VIEW-009 `metadata` of `example.files.FileDetail` writes `from:`, which is not a key of a view field
{name: metadata, type: example.files.Meta, field: {id: id, name: name}}
  -> unknown field `id`, expected `related`
```

This story is the model half: the key, validation, IR, diff and the reference.

## Acceptance

- At the new source format, `{name: metadata, type: example.files.Meta, field: {id: id, name:
  name}}` validates. `field:` on a struct-typed view field takes a nested mapping, and each member
  of the struct takes one source: a source field by name, `{related: {via, field}}`, `{expr: …}`,
  or a nested mapping for a struct member, as `sets:` does.
- Every declared member needs a source, unless it is `Optional`, and then it is absent. A member the
  struct does not declare is `undeclared_reference`. A source of another type is `type_mismatch`
  naming the member. `{related: …}` with a member named `related` reads as the navigation, as in a
  payload (`values-and-views.md:132`). Below the new format the mapping keeps today's refusal, its
  hint naming the format.
- An aggregate view refuses a nested field by name, as it refuses navigation.
- `ess specify compile` writes the mapping on the view field, omitted when absent; a model without
  it keeps its IR bytes; `schemas/generated/ess.schema.json` is regenerated. The OpenAPI and JSON
  Schema projections already write the field as the struct's schema and need no change: a test pins
  that the reproduction's `metadata` is the `example.files.Meta` object.
- `ess verify diff` reports a moved member source as the view field's source change, under the kind
  `field: {related}` uses.
- `website/docs/guides/specify/values-and-views.md` documents the form under views.

## Scope

- cited: crates/specify/ess-domain/src/view.rs
- cited: crates/specify/ess-domain/src/view/navigation.rs
- cited: crates/specify/ess-domain/src/view/from.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-diff/src/change.rs
- inferred: crates/specify/ess-domain/tests/view_nested_field.rs
- cited: website/docs/guides/specify/values-and-views.md
- inferred: docs/design/value-expressions.md
- inferred: schemas/generated/ess.schema.json
confidence: medium

## Fit review

1. **Need.** A read row nests flat stored fields under a struct member. Requester's syntax (theirs):
   `from: {id: id, name: name}` on the view field.
2. **Class.** Gap. A view field has three sources, each one value (`view.rs:917-943`). The two
   workarounds misstate the record or the wire (issue text).
3. **Already expressible?** No (`run.txt`, all four variants refused). Flattening the row moves the
   wire shape into a comment, which no suite checks (policy 1, `generic-language-features.md:588-593`).
4. **Fit.** It reuses the nested mapping that `payload:` and `sets:` take for a struct target
   (`value-expressions.md` § E5) and the view field's existing `field:` key. A view field then takes
   the value forms a payload field takes, member by member. `from:` is already refused on a view
   field, with the repair to `field:` or `expr:` (`view/from.rs:1-8`). Adopting it would revive a
   spelling deliberately refused. Grid R row "view field" (`language-grids.md:32`): one source field
   per member, no computation beyond `expr:`, so the row's sources do not widen. A struct literal in
   `expr:` stays out; G1 decides no struct constructor.
5. **Second adopter.** A user read answering `{"id": …, "address": {"street": …, "city": …}}` from
   a flat `users` record; an order read answering `{"total": {"amount": …, "currency": …}}` from two
   columns.
6. **Cost.** The next open source format; no new key (`field:` takes one more form); no new
   diagnostic code; an IR shape on the view field omitted when absent.
7. **Alternatives.** (a) Change nothing: flattened row or a fake stored struct (issue text).
   (b) The requester's `from:`: refused, a second key for a source, and already refused with a
   repair. (c) A struct constructor in `expr:`: refused, the grammar would gain literals that only
   this position reads. (d) The payload's nested mapping under `field:`: chosen.

## Decisions

accept, redesigned. The request's `from:` mapping is spelled as the existing nested mapping under
the view field's `field:` key. Conformance and generated code:
`story:conformance-asserts-a-nested-view-field-member-by-member`,
`story:generated-rust-and-go-view-queries-answer-a-nested-view-field`.

## Source

github:beyond10x/ess#760
