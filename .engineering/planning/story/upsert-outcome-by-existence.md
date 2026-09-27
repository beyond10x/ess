---
format: aep.planning-md/2
id: story:upsert-outcome-by-existence
kind: story
status: active
title: An outcome can be selected by whether the addressed record exists
refs:
- provider: github
  reference: beyond10x/ess#164
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/unknown_instance.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/outcome_group.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/decision.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 6
---
## Scope

Two accepted branches, update when the row exists and create when it does not, selected by existence (builds on `unknown_instance:` from 0.37.0).

## Acceptance

The #164 repro validates; the second call with the same identity is required to update, not duplicate or refuse.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

Two cases, one mechanism: (a) create-or-update selected by existence; (b) create-or-refuse, `already_exists: true` with `error:` (#164 follow-up comment, `docs/design/cross-record-and-stored-field-guards.md:395-409`). Both copy the `unknown_instance:` pattern (0.37.0).

- **Primary surface:** `crates/specify/ess-domain` (outcome conditions, validation) + `crates/verify/ess-conformance` (two-call witness) — cited
- **Domain:** `command.rs` `OutcomeCondition` (385), raw `unknown_instance` (4107), exclusivity (4407-4436), the unconditional-outcomes check emitting the #164 error (2428-2463, cited), `command/subject_fact.rs:65-83` duplicate check (cited), `command/outcome_shapes.rs` `validate_command` (147-189) and format gate (194-215), `command/subject_state.rs`, `outcome_group.rs:134` — inferred
- **Compiler/IR:** `ess-compiler/src/ir.rs:645`, `resolve.rs`, `expression.rs` — inferred
- **Synthesis:** `synthesize.rs` `unknown_instances`/`unknown_instance` (1280-1302, 5165-5370); `SnapshotSubject`/`ExpectSubjectUnchanged` (`scenario.rs:1891-1903`) look sufficient, so probably no suite format — inferred
- **Generators:** `ess-gen/src/{unknown_instance,openapi,http,docs}.rs`, `ess-synth/src/{plan.rs,rust,go}`, `ess-entity-runtime/src/lib.rs` (`OutcomeShapeUnsupported` 428, 1836), `ess-diff` — inferred
- **Registries:** source-format bump (`system.rs`, `ess-xtask/src/docs.rs`, `ess.schema.json`, `entry-classifications.json`, `formats.md`, `spec-versions.md`, `write-a-specification.md`) — cited
- **Open:** one key or two; whether generated Rust/Go dispatch by existence or refuse
- **Confidence:** medium
- **Would collide with:** `command.rs`/`outcome_shapes.rs`; source-format bump; `synthesize.rs` (multi-row witness shared with set-effects)

## Acceptance, widened 2026-09-27

From the #164 follow-up comment: an existence-selected branch may also be an `error:` outcome (create, or refuse when the record exists; sketched as `already_exists: true` in `docs/design/cross-record-and-stored-field-guards.md`). A second create with the same identity is required to answer the declared error, publish no event and leave the row unchanged.
