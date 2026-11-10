---
format: aep.planning-md/3
id: story:reader-true-refused-for-closed-readers
kind: story
status: active
title: '`reader: true` is refused where the reader is closed'
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: cited
  path: crates/specify/ess-composition/tests/adversary_reader_conformance.rs
- confidence: cited
  path: website/docs/reference/diagnostics.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:39Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:40Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A composition that marks a field subset `reader: true` against an ESS-generated closed type is refused, because that reader rejects unknown keys (review S1).

## Problem

Still true on main for 0.92.0. `reader: true` (`ess-composition/3`, closed #191) asserts that the
consumer's reader ignores keys it does not declare, and admits a local struct that reads a subset
of the imported struct's fields. ESS-generated readers are closed (`additionalProperties: false`
in the published schema, `deny_unknown_fields` in generated Rust), so a consumer that reads through
one rejects every message carrying an imported field its local type left out. The caveat is
written down (CHANGELOG 0.40.0; `website/docs/reference/formats.md`, `ess-composition/3` row;
`website/docs/reference/spec-versions.md`, `ess-composition/3`) but not enforced:

- `crates/specify/ess-composition/src/conformance.rs` module doc: "The subset assumes a tolerant
  reader that ignores keys it does not declare; ESS-generated closed types do not, and that is the
  author's assertion, not something this comparison can see."
- `crates/specify/ess-composition/tests/adversary_reader_conformance.rs::control_reader_admits_a_plain_field_subset`
  asserts that the subset composes with no diagnostic.

The comparison cannot see it today because nothing in a composition says whether the consumer
reads through ESS-generated code. Every ESS-declared struct generates a closed reader, so "refuse
when the consumer's type is generated closed" would refuse every field-subset reader unless the
composition or the consumer's type can declare a tolerant reader.

## Acceptance

- `cargo test -p ess-composition --locked --test adversary_reader_conformance` passes a new test
  `a_closed_reader_of_a_field_subset_is_refused`: an `ess-composition/6` document whose
  `conformances` entry has `reader: true` and no `tolerant:`, with the local
  `adv.consumer.read.TaggedSubset` against `adv.owner.wire.Tagged`, is refused with one
  `reader_subset_closed` diagnostic naming the local type, the imported type and each omitted
  imported field.
- The same target passes `a_tolerant_reader_of_a_field_subset_composes`: the same entry with
  `reader: true` and `tolerant: true` under `ess-composition/6` composes with no diagnostic.
- The same target passes `control_reader_admits_a_plain_field_subset` unchanged (`ess-composition/3`),
  and `cargo test -p ess-composition --locked` passes with no `/3`, `/4` or `/5` assertion changed.
- The same target passes `a_closed_reader_without_a_subset_composes`: an `ess-composition/6`
  `reader: true` entry with no `tolerant:` and no field subset (a newtype read as its primitive,
  an enum read as `String`, the same field set) composes with no diagnostic.
- The same target passes `tolerant_is_refused_before_format_six`: a `tolerant:` key under
  `ess-composition/5` is refused as an unknown field naming `/6`, and `tolerant: true` without
  `reader: true` under `/6` is refused when read, the error naming `reader`.
- The `reader_subset_closed` help text says that a consumer reading through ESS-generated closed
  types (`additionalProperties: false`, `deny_unknown_fields`) rejects the omitted keys, and that
  `tolerant: true` asserts a reader that ignores them; `website/docs/reference/diagnostics.md` lists
  the code with that text, and `website/docs/reference/formats.md` gains the `ess-composition/6`
  authored and compiled rows.

## Scope

- cited: crates/specify/ess-composition/src/conformance.rs
- cited: crates/specify/ess-composition/src/lib.rs (format constants and `SUPPORTED_COMPOSITION_FORMATS`, `TypeConformance::reader`, `CompositionCode`)
- cited: crates/specify/ess-composition/tests/adversary_reader_conformance.rs
- cited: website/docs/reference/formats.md
- cited: website/docs/reference/diagnostics.md

confidence: medium

The story moves the generated `website/docs/reference/spec-versions.md` (the `ess-composition/6`
entry) and proposes its `CHANGELOG.md` sentence.

## Fit review

Not an adopter request: finding S1 of the repository's own review of external requests. #191, which
introduced `reader: true`, is closed; this story only enforces the caveat that shipped with it.
Class: gap (an assertion validation accepts and generated code breaks).

## Decisions

- The composition learns the consumer's reader from a new entry key `tolerant: true` in a new
  composition format `ess-composition/6`, and a `/6` `reader: true` field subset without it is
  refused: refusing every subset would take back #191's fifth widening from a consumer that does
  read tolerantly, and only a stated marker tells the two apart.
- The refusal starts at `ess-composition/6` and `/3`-`/5` documents keep their meaning and bytes,
  because the composition formats refuse a new rule under an earlier marker
  (`formats.md`, each row's "earlier-format documents keep their meaning and bytes"), and the
  composition diagnostics carry no warning severity to warn with under `/5`. The composition
  format family is not one `ess/37` holds, so this story opens `/6` itself.

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (reader-true-refused-for-closed-readers). Drafted, not scheduled; a fit review precedes dispatch.
