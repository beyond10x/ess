---
format: aep.planning-md/2
id: story:absent-command-input-outcome
kind: story
status: active
title: A command can declare the outcome for an absent input as a whole
refs:
- provider: github
  reference: beyond10x/ess#170
relations:
- decomposes: epic:retrofit-findings-round-3
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/outcome_shapes.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/outcome-shapes.md
revision: 6
---
## Scope

An outcome selected by the absence of the whole command input (no request body), distinct from `{}`. Also: `validate` refusing `not defined(f)` over a non-Optional input that synthesis cannot witness.

## Acceptance

The #170 repro synthesizes a scenario that sends no input and requires the declared error.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (outcome condition + validate) + `crates/verify/ess-conformance` (synthesis + suite step) — cited
- **Declaration:** `ess-domain/src/command.rs` `OutcomeCondition` (385; `UnknownInstance` 471 is the precedent), `RawOutcome` (4063), `CommandSpec.input` (1794) — cited
- **Marker checks:** `ess-domain/src/command/outcome_shapes.rs` — inferred
- **validate/synthesize disagreement:** `CommandSpec::validate_guard` (`command.rs:2268`) or `Predicate::Defined` (`expression.rs:1153`) — cited site, choice inferred
- **Format gate:** `primitive_admission.rs`, `system.rs` — cited
- **Suite step:** `scenario.rs` `ScenarioStep::ExecuteCommand` (1864): empty input skipped, so absent and `{}` serialise the same today — cited
- **Synthesis/runner:** `synthesize.rs` (ESS-SYNTH-003, 634), `runner.rs` `execute_command` (1102), `target.rs` (181), `reference.rs`, `faulty.rs`, `interpret.rs` — cited/inferred
- **Suite formats:** `outcome_shapes.rs` precedent, `coverage_build.rs`, `coverage.rs`, `admission.rs`; Go/TS version lists — cited/inferred
- **Also likely:** `ess-gen/src/openapi.rs` (`requestBody` not required), `ess-compiler`, `ess-entity-runtime`, `ess-synth/src/rust/{entity,feasibility}.rs`, `ess-diff` — inferred (files `2549c11a7` touched)
- **Confidence:** medium
- **Would collide with:** `command.rs` outcome conditions, `primitive_admission.rs`, any source/suite format bump, `ScenarioStep::ExecuteCommand`, `synthesize.rs` branch witnessing
