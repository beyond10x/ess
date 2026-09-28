---
format: aep.planning-md/3
id: story:set-effects-over-filtered-instances
kind: story
status: active
title: An outcome can change every instance that matches a filter
refs:
- provider: github
  reference: beyond10x/ess#167
- provider: github
  reference: beyond10x/ess#175
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/outcome_shapes.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/set-effects-over-filtered-instances.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T21:20:27Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T21:21:56Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

#167: a command whose effect moves or updates every row matching a field filter, with a count. #175: an outcome's secondary effect on other instances selected from the subject's relations or a filter (`affects:`).

## Acceptance

Synthesis arranges matching and non-matching rows and fails an implementation that changes a non-matching row, skips a matching one, or reports a wrong count.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (outcome subject and instance addressing) — cited (#167: ESS-COMMAND-005)
- **Domain:** `command.rs` `RawOutcome` (4063, new `instances:`/`affects:` keys), `subject_of` (4608, refusal 4657), `Effect` (646), `Subject` (708, one `instance`), `PayloadSource` (852, a count source) — cited
- **Predicate for `where:`:** `command/subject_fact.rs` entity-field predicate with `input.` operands (ess/15 gate 266-278) — cited
- **Format gate:** `command/outcome_shapes.rs` `validate` (194) pattern; `system.rs` — cited precedent
- **IR:** `ess-compiler/src/ir.rs` `ResolvedInstance` (~716, only `Supplied`/`Observed`), `ResolvedSubject` (758), `ResolvedOutcome` (804); `resolve.rs` — cited types, new variant inferred
- **Synthesis:** `synthesize.rs` (2463, 5136, 5410); `synthesize/aggregate.rs` is the only multi-row arrangement today (precedent) — inferred
- **Suite:** `scenario.rs` `ScenarioStep` (1775), `outcome_shapes.rs` suite-major pattern, `admission.rs`, `runner.rs`; existing `ExpectView`/`Excludes`/`SnapshotView`/`ExpectViewUnchanged` may suffice — inferred
- **Entity Runtime:** `ess-entity-runtime/src/lib.rs` `LoweringCode` (396-430) refusal, `lower_outcome` (1738) — cited
- **Also likely:** exhaustive matches in `ess-gen/src/{docs,openapi,asyncapi}.rs`, `ess-synth/src/{plan.rs,web/catalog.rs}`, `ess-compiler/src/graph.rs`, `ess-diff`, `ess-conformance/src/{replay,web}.rs` — inferred
- **Registries:** source-format bump, `ess.schema.json` — cited
- **Documents:** new `docs/design/set-effects-over-filtered-instances.md` — inferred
- **Confidence:** medium
- **Would collide with:** `command.rs` outcome subjects/`PayloadSource`; multi-row synthesis (shared with upsert); format bumps; `LoweringCode`
