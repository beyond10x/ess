---
format: aep.planning-md/3
id: story:a-nested-optional-is-refused
kind: story
status: draft
title: A nested Optional is refused or its three readings are told apart
tags:
- defect
- ess-d368-close
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Finding

Found by wave D368 U1's review passes, pre-existing and out of that story's Acceptance: the validator admits `Optional<Optional<T>>` (`TypeRef::parse`, `crates/specify/ess-domain/src/types.rs`), whose `null` cannot be told from absent on the wire.

Reproduced on 0.92.0 (`ess/35`): a model whose entity field, command input and view field are each
`Optional<Optional<String>>` (`note`) validates (exit 0), synthesizes 8 scenarios with 0 refusals,
and passes the interpreted run 8 of 8. No scenario sends or observes `note` as JSON `null` beside
`note` omitted, so the three readings (absent, present `null`, present value) are neither refused
nor told apart.

What is already written down:

- `docs/design/binding-mapping-bounded-accessor.md` ("Assignment plan" table) admits nested
  Optional binding targets on purpose: outer `None` omits the member, inner `None` emits `null`,
  and the decoder reads both missing and `null` as outer `None`, so the round trip is not
  injective.
- `website/docs/guides/verify/observations.md` ("Native Rust and Go values can distinguish nested
  Optional states that serialize to the same JSON `null`"): conformance refuses such an ambiguous
  observation for a mapping, and only there.
- Committed users of the type: `crates/generate/ess-synth/tests/fixtures/bounded-accessor.yaml`
  (`Optional<Optional<String>>` and `Optional<Optional<Optional<String>>>`), and the tests that
  read it (`crates/verify/ess-conformance/tests/bounded_accessor.rs`,
  `union_unit_variants_accessor.rs`, `crates/generate/ess-synth/tests/union_unit_variants.rs`), plus
  `crates/verify/ess-conformance/tests/support_explore_optional_unknown/matrix.rs` and
  `docs/design/explorer-optional-unknown.md`.

The bounded-accessor fixture declares the nested types as command input fields
(`projection.core.Receive` inputs `nested` and `deeper`), and that design's table assigns a
binding into them. A refusal of command-input fields therefore refuses that fixture and retires
those table rows; the decision has to say so.

## Acceptance

- `ess specify validate` refuses an entity, command-input, event, error or view field declared
  `Optional<Optional<T>>` (at any depth, including inside `List`/`Map`/struct fields) with a named
  diagnostic citing the file, the line and the declaration, or the specification documents what each
  of absent, `null` and a present `null` means and synthesis tells them apart (a scenario per
  reading, and the interpreted run fails an implementation that conflates two of them).
- If refused: gated on the next source format, with a warning under the current one naming the
  same field; `crates/generate/ess-synth/tests/fixtures/bounded-accessor.yaml` (`ess/3`) keeps
  validating and keeps its suite bytes, and the binding design page states the new rule.
- Committed models that use the type are named in the change (the list above, re-measured).

## Scope

- cited: crates/specify/ess-domain/src/types.rs
- cited: docs/design/binding-mapping-bounded-accessor.md
- cited: website/docs/guides/verify/observations.md
- cited: crates/generate/ess-synth/tests/fixtures/bounded-accessor.yaml
- inferred: crates/specify/ess-domain/src/system.rs (format gate)
- inferred: crates/specify/ess-domain/tests/

confidence: medium

## Decisions

None yet: refuse (where, and behind which format) or document and synthesize the three readings.
