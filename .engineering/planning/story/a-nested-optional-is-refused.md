---
format: aep.planning-md/3
id: story:a-nested-optional-is-refused
kind: story
status: draft
title: A nested optional is refused from ess/37
tags:
- defect
- ess-d368-close
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/nested_optional.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: cited
  path: docs/design/binding-mapping-bounded-accessor.md
- confidence: cited
  path: website/docs/guides/verify/observations.md
revision: 10
---
## Finding

Found by wave D368 U1's review passes, pre-existing and out of that story's Acceptance: the validator admits `Optional<Optional<T>>` (`TypeRef::parse`, `crates/specify/ess-domain/src/types.rs`), whose `null` cannot be told from absent on the wire.

Reproduced on 0.92.0 (`ess/35`): a model whose entity field, command input and view field are each
`Optional<Optional<String>>` (`note`) validates (exit 0), synthesizes 8 scenarios with 0 refusals,
and passes the interpreted run 8 of 8. No scenario sends or observes `note` as JSON `null` beside
`note` omitted, so the three readings (absent, present `null`, present value) are neither refused
nor told apart.

What is already written down:

- `docs/design/binding-mapping-bounded-accessor.md` ("Target Optional construction" table) admits
  nested Optional binding targets on purpose: outer `None` omits the member, inner `None` emits
  `null`, and the decoder reads both missing and `null` as outer `None`, so the round trip is not
  injective.
- `website/docs/guides/verify/observations.md` ("Native Rust and Go values can distinguish nested
  Optional states that serialize to the same JSON `null`"): conformance refuses such an ambiguous
  observation for a mapping, and only there.
- Committed users of the type: `crates/generate/ess-synth/tests/fixtures/bounded-accessor.yaml`
  (`format: ess/3`; `Optional<Optional<String>>` and `Optional<Optional<Optional<String>>>`), and
  the tests that read it (`crates/verify/ess-conformance/tests/bounded_accessor.rs`,
  `union_unit_variants_accessor.rs`, `crates/generate/ess-synth/tests/union_unit_variants.rs`), plus
  `crates/verify/ess-conformance/tests/support_explore_optional_unknown/matrix.rs` and
  `docs/design/explorer-optional-unknown.md`.

The bounded-accessor fixture declares the nested types as command input fields
(`projection.core.Receive` inputs `nested` and `deeper`), and that design's table assigns a
binding into them. Because the refusal below is gated on `ess/37`, that fixture keeps validating
and the table rows stay true for sources below `ess/37`.

## Decisions

- Refuse, do not document three readings: the generated decoders read a missing member and `null`
  alike as outer `None` (design page above), so no wire reading can tell the three apart and a
  scenario per reading would have nothing to observe.
- The refusal is gated on `ess/37`, the epic's source format: a source below it validates as it
  does today, with no warning, because the validator reports one code (`implied_relation`) as a
  warning and a second warning code is a contract of its own.
- The refusal has a code of its own, `nested_optional`, so a consumer can match it apart from a
  type that does not parse.

## Acceptance

- `cargo test -p ess-domain --locked --test nested_optional` passes, asserting on a model with
  `format: ess/37` that `ess specify validate` refuses with code `nested_optional`, citing the file,
  line and path of the declaration, each of: an entity field, a command input, an event field, an
  error field, a view field and a struct field declared `Optional<Optional<String>>`; a
  `List<Optional<Optional<String>>>` element; a `Map<String, Optional<Optional<String>>>` value; and
  an `Optional<Optional<Optional<String>>>` field (one diagnostic for that field, not two).
- The same test asserts that the same model with `format: ess/36` validates with exit 0 and no
  diagnostic.
- `ess specify validate --path crates/generate/ess-synth/tests/fixtures/bounded-accessor.yaml`
  exits 0, and `cargo test -p ess-conformance --locked --test bounded_accessor` and
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` pass with no row of
  that fixture moved.
- `docs/design/binding-mapping-bounded-accessor.md` ("Target Optional construction") and
  `website/docs/guides/verify/observations.md` state that from `ess/37` a nested Optional is
  refused as `nested_optional`, and that the nested rows of the table hold below `ess/37` only.

## Scope

- cited: crates/specify/ess-domain/src/types.rs
- cited: crates/specify/ess-primitives/src/error.rs
- cited: docs/design/binding-mapping-bounded-accessor.md
- cited: website/docs/guides/verify/observations.md
- inferred: crates/specify/ess-domain/tests/nested_optional.rs (new)

confidence: medium

Shared surfaces moved: the `ess/37` `FORMAT_HISTORY` sentence and `CHANGELOG.md`; it moves no
byte-pin row.
