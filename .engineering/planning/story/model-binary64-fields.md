---
format: aep.planning-md/1
id: story:model-binary64-fields
kind: story
status: draft
title: Represent finite binary64 fields in compiler-owned models
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
revision: 3
---
## Evidence

Concrete decoded input records can contain finite IEEE-754 binary64 fields whose
rounded values feed ordered scaling and integer conversion. Such a source member
is neither an exact decimal nor an integer. Existing normalization supports
explicit binary64 admission and rounded operations for qualified numeric schema
roots, but the authored model cannot declare the corresponding numeric member.

`crates/specify/ess-domain/src/types.rs:42-88` enumerates String, Boolean, Integer,
Decimal, Timestamp, Duration, Uuid and Bytes; Decimal is explicitly an exact
decimal, never a float. `crates/generate/ess-gen/src/types.rs:490-497` projects
Decimal as a patterned string. `crates/generate/schema-contract/src/realize/normalize/recipe.rs`
defines binary64 input paths/operations, while `normalize/check.rs::input_numbers`
requires a numeric leaf. A model-owned string-encoded Decimal cannot supply that leaf.

## Outcome

Give an explicitly declared model-owned finite binary64 field a truthful typed
representation and checked normalization semantics without changing Decimal or
making binary64 decoding implicit for unrelated fields.

## Required design and acceptance

- Bind the domain declaration, numeric wire representation, source identity and
  format compatibility before implementation. Decide whether it is a concrete
  primitive or another narrowly typed construct; do not smuggle it through an
  annotation, exact decimal, string or opaque byte field.
- Establish finite JSON admission and binary64 rounding, including negative zero,
  subnormal values and overflow refusal. NaN/infinities do not become admitted JSON
  values. Distinguish already-decoded values from original token provenance.
- Preserve typed model projection/source accounting and allow the selected modeled
  numeric field to reach the existing checked binary64 normalization operations.
  Range policy at a subsequent float-to-integer conversion remains separately explicit.
- Preserve existing exact-integer/Decimal semantics and old canonical artifacts,
  or make any necessary format migration explicit. Unselected fields do not acquire
  hidden floating conversion.
- Reference and supported generated targets implement the same declared semantics
  or refuse before successful partial output. No global arbitrary JSON property
  bag or source-language decoder emulator is introduced.

## Scope candidates

Primary: `crates/specify/ess-domain/src/types.rs`,
`crates/generate/ess-gen/src/types.rs`, and model-owned normalization admission in
`crates/generate/schema-contract/src/realize/normalize/`.
Typed realization surfaces under `crates/generate/ess-synth` and structural
`schema-contract` realization require scoping once the declaration/wire design is bound.

This is a generic type-system/normalization gap, distinct from lexical JSON capture,
exact decimal arithmetic, source-language permissiveness and platform-specific
out-of-range casts.

## Bound design

The coordinator selected docs/design/model-binary64.md for the next serial unit
following raw JSON format 4 integration. It introduces authored ess/2 Binary64,
finite numeric wire policy with signed-zero retention, compiler-owned model type
metadata, explicit branch numeric input paths and version-5 floating literal/value
operations for defaults and computed output. Old model/recipe/generated file maps
remain stable; selected unsupported synthesis/conformance routes refuse before
publication. No generic JSON property bag, implicit numeric cast, map-key float
support or ess-ir/2 is introduced. Full synthesis codecs are a distinct boundary.

The design is bound but no implementation, gate or release is claimed. Raw JSON
and this unit overlap; they run serially. Typed scope will be refreshed from the
final raw source before dispatch. The operator's original full-gap continuation
remains the task scope absent a response narrowing the outstanding wrap-up question.
