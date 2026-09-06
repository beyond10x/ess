---
format: aep.planning-md/1
id: story:model-binary64-fields
kind: story
status: implemented
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
  path: crates/edge/ess-cli/tests/binary64_adversary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/binary64_count_adversary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/binary64_publication.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/count_writer_pass1.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/count_writer_pass2.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/go_conformance.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-synth/src/clap/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/feasibility.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/eval.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/execute.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_condition.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_numeric.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/eval.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/execute.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/go_condition.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/go_numeric.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/numeric.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4/go_runtime.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4/recipe.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/numeric.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/binary64_adversary.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/binary64_adversary.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/binary64_adversary_go.go.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/binary64_adversary_rust.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_binary64.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_binary64_go_tests.go.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_binary64_rust_tests.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_legacy_maps.json
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_raw.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_binary64.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_binary64_targets.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_go.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_legacy_bytes.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/binary64.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/lib.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/expression_validation.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/billing.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binary64_count_adversary.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/elapsed.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/execution.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/faults.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/halt.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/suite.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: crates/verify/ess-diff/tests/review_adversary_f01.rs
- confidence: cited
  path: docs/design/model-binary64.md
- confidence: cited
  path: docs/design/review-conformance-coverage.md
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: examples/billing-realization/tests/conformance.rs
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 14
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

The implementation handoff is frozen at bf16e504ccad68b2ee67607ba39606aadf07f627.
Its complete 82-path content/mode manifest was verified by the coordinator before
the bot commit; manifest SHA-256 is
edcef1f19ee47dfcad2b0f6b9b26ae19fcf6355ba8b28aa02604a24c9cc9f68e.
This replaces the provisional write scope with observed writes. Adversary test
additions and any routed correction will be recorded separately.

The implementor confirmed the shared domain Number predicate policy, compiler
metadata and synthesis/conformance admission boundaries. Actual caller discovery
also required test-only Result handling in ess-diff and billing-realization.
The same-command whole-base test count was not measured; no such count is inferred.

Previously scoped paths that remained read-only or were replaced by concrete
helper/test placement are retained here as corrections to the earlier inference:
- `crates/generate/ess-openapi/src/accounting.rs` — no write in this unit commit.
- `crates/generate/ess-openapi/src/lib.rs` — no write in this unit commit.
- `crates/generate/schema-contract/src/realize/go.rs` — no write in this unit commit.
- `crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt` — no write in this unit commit.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4` — no write in this unit commit.
- `crates/generate/schema-contract/src/realize/normalize/source.rs` — no write in this unit commit.
- `crates/generate/schema-contract/src/realize/rust.rs` — no write in this unit commit.
- `crates/generate/schema-contract/src/realize/ts.rs` — no write in this unit commit.
- `crates/generate/schema-contract/tests/normalization_binary64_model.rs` — no write in this unit commit.
- `crates/specify/ess-compiler/src/expression.rs` — no write in this unit commit.
- `crates/specify/ess-compiler/src/resolve.rs` — no write in this unit commit.
- `crates/specify/ess-domain/src/command.rs` — no write in this unit commit.
- `crates/specify/ess-domain/src/entity.rs` — no write in this unit commit.
- `crates/specify/ess-domain/src/view.rs` — no write in this unit commit.
- `crates/specify/ess-domain/src/wire.rs` — no write in this unit commit.
- `crates/specify/ess-domain/tests/binary64.rs` — no write in this unit commit.
- `crates/specify/ess-domain/tests/expression.rs` — no write in this unit commit.
- `crates/specify/ess-primitives/src/facts.rs` — no write in this unit commit.
- `crates/verify/ess-conformance/src/go/runtime.go` — no write in this unit commit.
- `crates/verify/ess-conformance/tests/adversary_expression_pass1.rs` — no write in this unit commit.

Confirmed implementation write paths (cited from the frozen manifest):
- `CHANGELOG.md`.
- `crates/edge/ess-cli/src/main.rs`.
- `crates/edge/ess-cli/tests/binary64_publication.rs`.
- `crates/edge/ess-cli/tests/go_conformance.rs`.
- `crates/edge/ess-cli/tests/normalization.rs`.
- `crates/generate/ess-gen/src/model_types.rs`.
- `crates/generate/ess-gen/src/types.rs`.
- `crates/generate/ess-synth/src/clap/mod.rs`.
- `crates/generate/ess-synth/src/failure.rs`.
- `crates/generate/ess-synth/src/go/http.rs`.
- `crates/generate/ess-synth/src/go/layout.rs`.
- `crates/generate/ess-synth/src/go/mod.rs`.
- `crates/generate/ess-synth/src/lib.rs`.
- `crates/generate/ess-synth/src/rust/feasibility.rs`.
- `crates/generate/ess-synth/src/rust/layout.rs`.
- `crates/generate/ess-synth/src/rust/mod.rs`.
- `crates/generate/ess-synth/src/rust/wire.rs`.
- `crates/generate/ess-synth/src/web/mod.rs`.
- `crates/generate/ess-synth/tests/feasibility.rs`.
- `crates/generate/schema-contract/src/realize.rs`.
- `crates/generate/schema-contract/src/realize/normalize.rs`.
- `crates/generate/schema-contract/src/realize/normalize/check.rs`.
- `crates/generate/schema-contract/src/realize/normalize/eval.rs`.
- `crates/generate/schema-contract/src/realize/normalize/execute.rs`.
- `crates/generate/schema-contract/src/realize/normalize/go_condition.go.txt`.
- `crates/generate/schema-contract/src/realize/normalize/go_numeric.go.txt`.
- `crates/generate/schema-contract/src/realize/normalize/go_target.rs`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/eval.rs.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/execute.rs.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/go_condition.go.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/go_numeric.go.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v4/numeric.rs.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v4/go_runtime.go.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v4/recipe.rs.txt`.
- `crates/generate/schema-contract/src/realize/normalize/legacy_v4/rust_runtime.rs.txt`.
- `crates/generate/schema-contract/src/realize/normalize/numeric.rs`.
- `crates/generate/schema-contract/src/realize/normalize/recipe.rs`.
- `crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt`.
- `crates/generate/schema-contract/src/realize/normalize/target.rs`.
- `crates/generate/schema-contract/tests/fixtures/normalization_binary64.rs`.
- `crates/generate/schema-contract/tests/fixtures/normalization_binary64_go_tests.go.txt`.
- `crates/generate/schema-contract/tests/fixtures/normalization_binary64_rust_tests.rs.txt`.
- `crates/generate/schema-contract/tests/fixtures/normalization_legacy_maps.json`.
- `crates/generate/schema-contract/tests/fixtures/normalization_raw.rs`.
- `crates/generate/schema-contract/tests/normalization_binary64.rs`.
- `crates/generate/schema-contract/tests/normalization_binary64_targets.rs`.
- `crates/generate/schema-contract/tests/normalization_go.rs`.
- `crates/generate/schema-contract/tests/normalization_legacy_bytes.rs`.
- `crates/specify/ess-compiler/src/binary64.rs`.
- `crates/specify/ess-compiler/src/lib.rs`.
- `crates/specify/ess-compiler/tests/expression_validation.rs`.
- `crates/specify/ess-domain/src/expression.rs`.
- `crates/specify/ess-domain/src/lib.rs`.
- `crates/specify/ess-domain/src/primitive_admission.rs`.
- `crates/specify/ess-domain/src/spec.rs`.
- `crates/specify/ess-domain/src/system.rs`.
- `crates/specify/ess-domain/src/types.rs`.
- `crates/specify/ess-domain/tests/billing.rs`.
- `crates/verify/ess-conformance/src/admission.rs`.
- `crates/verify/ess-conformance/src/authored.rs`.
- `crates/verify/ess-conformance/src/go/mod.rs`.
- `crates/verify/ess-conformance/src/input.rs`.
- `crates/verify/ess-conformance/src/lib.rs`.
- `crates/verify/ess-conformance/src/runner.rs`.
- `crates/verify/ess-conformance/src/scenario.rs`.
- `crates/verify/ess-conformance/src/synthesize.rs`.
- `crates/verify/ess-conformance/src/web.rs`.
- `crates/verify/ess-conformance/src/witness.rs`.
- `crates/verify/ess-conformance/tests/authored.rs`.
- `crates/verify/ess-conformance/tests/elapsed.rs`.
- `crates/verify/ess-conformance/tests/execution.rs`.
- `crates/verify/ess-conformance/tests/faults.rs`.
- `crates/verify/ess-conformance/tests/halt.rs`.
- `crates/verify/ess-conformance/tests/suite.rs`.
- `crates/verify/ess-conformance/tests/synthesis.rs`.
- `crates/verify/ess-diff/tests/review_adversary_f01.rs`.
- `docs/design/model-binary64.md`.
- `docs/design/source-pinned-data-normalization.md`.
- `examples/billing-realization/tests/conformance.rs`.
- `website/docs/guides/generate-artifacts.md`.
- `website/docs/guides/write-a-specification.md`.
- `website/docs/reference/formats.md`.

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

## Integrated verification

The Binary64 model/normalization boundary is implemented and integrated with the published conformance count writer. verification-report:normalization-binary64-integrated records source identities, both immutable no-product-findings reviews, the corrected default Go test harness, and literal full gates: 1,945 passing tests, zero failed/ignored, plus the successful site build. Structural native codecs, positional arrays, TypeScript normalization and final release remain separate follow-ups. No released-version adoption is claimed here.
