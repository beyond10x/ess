---
format: aep.planning-md/2
id: story:aggregates-over-optional-fields
kind: story
status: implemented
title: Aggregate over and group by an Optional field
relations:
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/aggregate_views_ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/aggregate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: docs/design/aggregate-views.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 8
---
## Scope

- #148: `sum`/`avg`/`count_distinct` over an `Optional` field skip absent values
  (`skip_absent: true`); a `group_by` key of `Optional<T>` makes the absent value its own group.

## Acceptance

Both repro views in #148 validate and synthesize scenarios with a row that lacks the value, and
the expected aggregate matches the SQL treatment the issue names.

## Derived scope

Derived 2026-09-27 by `aep:story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (view validation) — cited, both refusals in #148 are raised there
- **Primary surface:** `crates/verify/ess-conformance` (aggregate synthesis and expected values) — cited
- **Files:** `crates/specify/ess-domain/src/view.rs:1052-1068` (V9) and `:1192-1212` (V11) — cited
- **Files:** `crates/specify/ess-domain/src/view.rs:443-456` (`RawAggregate`) and `:353` (`Aggregate`) — cited
- **Files:** `crates/verify/ess-conformance/src/synthesize/aggregate.rs` (`leaf` :79; `scopable` :507-516) — cited
- **Files:** `crates/verify/ess-conformance/src/aggregate.rs:92` (`evaluate`) — cited
- **Also likely:** `crates/specify/ess-compiler/src/ir.rs:1286`, `src/resolve.rs:2754` (`ResolvedAggregation`) — inferred
- **Also likely:** `crates/specify/ess-domain/src/primitive_admission.rs:254-270`, `src/system.rs:98` (a format gate) — inferred
- **Also likely:** `crates/verify/ess-conformance/src/witness.rs` rule 1 — inferred
- **Also likely:** `crates/verify/ess-diff/src/diff.rs:1636-1667` — inferred
- **Also likely (tests):** `crates/specify/ess-compiler/tests/aggregate_views_ir.rs:146,153` — cited; `ess-domain/tests/aggregate_views.rs`, `ess-conformance/tests/aggregate_*.rs` — inferred
- **Documents:** `docs/design/aggregate-views.md:161-167,246-259` — cited; guide, schema, change fragment — inferred
- **Confidence:** medium — refusal sites cited; synthesis, IR and format reach inferred
- **Would collide with:** any unit touching `view.rs`, `synthesize/aggregate.rs` or `witness.rs`; with a new format, any unit bumping the source format
- **Not established:** whether `skip_absent` needs a new format; where it is written; whether `witness.rs` must change; how an absent-key group is scoped; `min`/`max` over optional operands.
