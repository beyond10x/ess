---
format: aep.planning-md/3
id: story:current-time-guard-operand
kind: story
status: active
title: A Timestamp guard can compare with the current time and a tolerance
refs:
- provider: github
  reference: beyond10x/ess#171
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/time.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/target.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: docs/design/timestamp-clock-provenance.md
- confidence: inferred
  path: website/docs/reference/predicates.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T19:54:46Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T19:55:52Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

A `now` operand with duration arithmetic in guards, with a clock the conformance adapter controls (see `timestamp-clock-provenance.md`).

## Acceptance

The #171 repro validates; scenarios at now-61s and now-59s require refusal and acceptance.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (guard typing) + `crates/verify/ess-conformance` (synthesis, suite value, clock) — inferred
- **Refusal site:** `ess-domain/src/expression.rs:861` `text_literal` (`not an RFC 3339 instant`, :902), :1007 — cited
- **Grammar/evaluation:** `ess-primitives/src/predicate.rs` `Operand` (334), `evaluate_compare` (721); `ess-primitives/src/time.rs` — inferred
- **IR:** `ess-compiler/src/expression.rs`, `ir.rs` — inferred
- **Synthesis:** `synthesize.rs:5965` instant boundaries, `witness.rs:1613` `Leaf::Timestamp` — inferred
- **Suite value / clock:** `scenario.rs` `ScenarioValue` (1416), `runner.rs` `Clock` (104), `target.rs` `ConformanceTarget` (100) — inferred
- **Runtimes:** `ess-entity-runtime/src/lib.rs` `Ordering::Instant` (3468); Go/TS `runtime`/`predicate` — inferred
- **Registries:** source + suite format bump (`system.rs`, `scenario.rs`, `admission.rs`, `ess-xtask/src/docs.rs`, `entry-classifications.json`, `ess.schema.json`) — inferred
- **Documents:** `docs/design/timestamp-clock-provenance.md` — cited (it rules out a current-time fallback, so a design decision is needed); `website/docs/reference/predicates.md` — inferred
- **Confidence:** medium
- **Would collide with:** source/suite format bumps, guard operand changes (`predicate.rs`, `expression.rs`), `ScenarioValue`/`ConformanceTarget`, Go/TS runtimes
