---
format: aep.planning-md/1
id: story:model-binary64-fields
kind: story
status: active
title: Represent finite binary64 fields in compiler-owned models
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
- depends_on: story:raw-json-normalization-provenance
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-openapi/src/accounting.rs
- confidence: cited
  path: crates/generate/ess-openapi/src/lib.rs
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/go.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/eval.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_numeric.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/numeric.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/source.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/ts.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_binary64_model.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/expression.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/expression_validation.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/src/wire.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/binary64.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/expression.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_expression_pass1.rs
- confidence: cited
  path: docs/design/model-binary64.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 9
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

## Scope

Bound against combined main1667d02 plus raw unit3e2eb52, now integrated as6c78676.
This is one serial cross-crate primitive/model/normalization unit. New Primitive
admission necessarily reaches the domain, compiler, generation and conformance
boundaries; splitting off only the enum would leave falsely successful consumers.
OpenAPI import/accounting stays unchanged and supplies no new numeric authority.

The cited scope covers existing implementation and required qualification surfaces;
not every cited file needs an edit. Concrete new helpers/tests and additional frozen
format1–4 templates are inferred until the implementor returns their actual paths.
Confidence: high for required boundaries, medium for exact new helper placement.

- `CHANGELOG.md` (cited).
- `crates/edge/ess-cli/src/main.rs` (cited).
- `crates/generate/ess-gen/src/model_types.rs` (cited).
- `crates/generate/ess-gen/src/types.rs` (cited).
- `crates/generate/ess-openapi/src/accounting.rs` (cited).
- `crates/generate/ess-openapi/src/lib.rs` (cited).
- `crates/generate/ess-synth/src/failure.rs` (cited).
- `crates/generate/ess-synth/src/lib.rs` (cited).
- `crates/generate/schema-contract/src/realize.rs` (cited).
- `crates/generate/schema-contract/src/realize/go.rs` (cited).
- `crates/generate/schema-contract/src/realize/normalize.rs` (cited).
- `crates/generate/schema-contract/src/realize/normalize/check.rs` (cited).
- `crates/generate/schema-contract/src/realize/normalize/eval.rs` (cited).
- `crates/generate/schema-contract/src/realize/normalize/go_numeric.go.txt` (cited).
- `crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt` (cited).
- `crates/generate/schema-contract/src/realize/normalize/numeric.rs` (cited).
- `crates/generate/schema-contract/src/realize/normalize/source.rs` (cited).
- `crates/generate/schema-contract/src/realize/rust.rs` (cited).
- `crates/generate/schema-contract/src/realize/ts.rs` (cited).
- `crates/specify/ess-compiler/src/expression.rs` (cited).
- `crates/specify/ess-compiler/src/resolve.rs` (cited).
- `crates/specify/ess-compiler/tests/expression_validation.rs` (cited).
- `crates/specify/ess-domain/src/command.rs` (cited).
- `crates/specify/ess-domain/src/entity.rs` (cited).
- `crates/specify/ess-domain/src/expression.rs` (cited).
- `crates/specify/ess-domain/src/spec.rs` (cited).
- `crates/specify/ess-domain/src/system.rs` (cited).
- `crates/specify/ess-domain/src/types.rs` (cited).
- `crates/specify/ess-domain/src/view.rs` (cited).
- `crates/specify/ess-domain/src/wire.rs` (cited).
- `crates/specify/ess-domain/tests/expression.rs` (cited).
- `crates/specify/ess-primitives/src/facts.rs` (cited).
- `crates/verify/ess-conformance/src/authored.rs` (cited).
- `crates/verify/ess-conformance/src/go/mod.rs` (cited).
- `crates/verify/ess-conformance/src/go/runtime.go` (cited).
- `crates/verify/ess-conformance/src/input.rs` (cited).
- `crates/verify/ess-conformance/src/runner.rs` (cited).
- `crates/verify/ess-conformance/src/scenario.rs` (cited).
- `crates/verify/ess-conformance/src/synthesize.rs` (cited).
- `crates/verify/ess-conformance/src/web.rs` (cited).
- `crates/verify/ess-conformance/tests/adversary_expression_pass1.rs` (cited).
- `docs/design/model-binary64.md` (cited).
- `website/docs/guides/generate-artifacts.md` (cited).
- `website/docs/reference/formats.md` (cited).

## Dispatch

The unit begins in managed treewt-9d0ef745088a, branch impl/normalization-binary64,
base6c78676. It owns its own target/ with four serial jobs, incremental compilation
off and development/test debug information0; preserve an8GiB filesystem reserve.
The complete pre-dispatch page is docs/plan/2026-09-06-model-binary64.md. Its final
computed output contains8 candidate waves,93 collisions, no unassessed ids or cycles.
Only this scoped story is selected; other normalization units run afterward.

The final binding separates authored predicate Number comparisons from strict nominal
assignment and normalization5 typed Binary64 equality. The implementor must qualify
actual conformance CLI/direct emitter/reader/runner publication boundaries and preserve
old generated file maps. Unsupported synthesis/conformance routes may refuse explicitly
before producing output. The coordinator alone owns AEP and Git integration.
