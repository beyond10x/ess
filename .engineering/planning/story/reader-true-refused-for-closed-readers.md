---
format: aep.planning-md/3
id: story:reader-true-refused-for-closed-readers
kind: story
status: draft
title: '`reader: true` is refused where the reader is closed'
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
- decomposes: epic:zero-open-issues-2026-11
revision: 3
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

- `ess specify compose` on an `ess-composition/3`+ document whose `conformances` entry has
  `reader: true` and a local struct reading a strict subset of the imported struct's fields, where
  the consumer's type is read through an ESS-generated reader, is refused with a named code naming
  the local type, the imported type and the omitted imported fields (or warns under the current
  composition format and refuses from the next).
- The help text carries the caveat now in the CHANGELOG 0.40.0 entry (CL:943-947 at the review):
  a consumer reading through ESS-generated closed types must not use `reader` for a field subset.
- A `reader: true` entry with no field subset (newtype as primitive, enum as `String`, same field
  set) still composes; `control_reader_admits_a_plain_field_subset` is kept for a consumer declared
  tolerant, or replaced by the refusal case if the decision is to refuse every subset.

## Scope

- cited: crates/specify/ess-composition/src/conformance.rs
- cited: crates/specify/ess-composition/src/lib.rs
- cited: crates/specify/ess-composition/tests/adversary_reader_conformance.rs
- cited: website/docs/reference/formats.md
- cited: website/docs/reference/spec-versions.md

confidence: medium

## Fit review

Not an adopter request: finding S1 of the repository's own review of external requests. #191, which
introduced `reader: true`, is closed; this story only enforces the caveat that shipped with it.
Class: gap (an assertion validation accepts and generated code breaks).

## Decisions

None yet. Open: how the composition learns that the consumer reads through ESS-generated closed
code (a tolerant-reader marker on the entry or the type, or refusing every field subset), and
whether the refusal waits for the next composition format.

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (reader-true-refused-for-closed-readers). Drafted, not scheduled; a fit review precedes dispatch.
