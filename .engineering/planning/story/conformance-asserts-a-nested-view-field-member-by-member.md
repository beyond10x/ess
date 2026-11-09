---
format: aep.planning-md/3
id: story:conformance-asserts-a-nested-view-field-member-by-member
kind: story
status: draft
title: Conformance asserts a nested view field member by member
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#760
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-field-nests-source-fields-under-a-struct-member
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/view_nested_field.rs
revision: 4
---
## Need

A view field whose struct members are filled from source fields
(`story:a-view-field-nests-source-fields-under-a-struct-member`) needs a suite that checks each member
against the source field it reads. Otherwise a target that swaps `id` and `name`, or answers the
row flat, passes. View expectations already assert a struct leaf by its dotted path (suite/26,
`website/docs/reference/formats.md`, the `ess-conformance/26` row: "an event payload or view row
expectation may name one leaf of a struct field by its dotted path").

## Acceptance

- `ess verify conform synthesize` on the reproduction model (with the command that stores a file
  added: `example.files.StoreFile` sets `id`, `name`, `content` from input) writes the view
  expectation with `metadata.id` and `metadata.name` as dotted leaves. Each holds the value the
  scenario sent for the source field, and the scenario sends `id` and `name` distinct values, so a
  target that swaps them fails.
- `ess verify conform run --target interpreted` passes: the interpreted target builds the member
  from the row (`crates/verify/ess-conformance/src/interpret/views.rs`).
- A target answering the row flat (`id`, `name` at the top level, no `metadata`) fails the
  scenario. `crates/verify/ess-conformance/tests/` gains such a target.
- No suite format moves: dotted leaves are suite/26 vocabulary. A model without a nested view field
  keeps its suite bytes.

## Scope

- cited: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/view_nested_field.rs
confidence: medium

## Fit review

1. **Need.** The suite checks the nested member's leaves. Requester's syntax (theirs): "asserted
   member by member in conformance, as dotted leaves already are for payloads".
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No (parent story).
4. **Fit.** It reuses the dotted-leaf expectation of suite/26 and the forward evaluation of values
   (`docs/design/generic-language-features.md`, "a value is evaluated forwards"). No search is added.
5. **Second adopter.** The address read of the parent story.
6. **Cost.** No suite format; synthesis and interpreter code only.
7. **Alternatives.** (a) Compare the member whole: refused, a whole-struct comparison fails on any
   member the scenario cannot determine, where leaves let the rest be checked. (b) Dotted leaves:
   chosen. That is the requester's proposal.

## Decisions

accept, redesigned, as the parent story. The assertion is the requester's (dotted leaves).

## Source

github:beyond10x/ess#760
