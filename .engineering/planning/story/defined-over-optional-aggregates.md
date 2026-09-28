---
format: aep.planning-md/3
id: story:defined-over-optional-aggregates
kind: story
status: implemented
title: defined() admits an Optional struct, list or map
refs:
- provider: github
  reference: beyond10x/ess#176
relations:
- decomposes: epic:retrofit-findings-round-3
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T20:07:45Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T20:09:42Z", actor: "human:timo", revision: 6, imported: true}
- {from: "active", to: "implemented", at: "2026-09-28T04:35:20Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Scope

`defined(x)` over any `Optional<T>`, including aggregate `T`, in guards and invariants.

## Acceptance

The #176 invariant `any: [state == Paused, {not: defined(metrics)}]` validates, and synthesis refutes it with an outcome that leaves `Paused` without clearing `metrics`.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` predicate type check — cited
- **Refusal:** `ess-domain/src/expression.rs:1153` (`Truthy | Defined` refuses non-scalar), `:785` — cited
- **Evaluation:** `ess-primitives/src/predicate.rs:684` (`observe(path).is_some()`), `facts.rs:1147` (bound leaves only, so a present struct path is unbound) — cited
- **Fact flattening:** `ess-conformance/src/input.rs:885`, `runner.rs:2934`; Go `predicate.go` (80, 642, 828); TS `predicate.ts` (108, 262, 91) — cited: all bind children only, so `defined(metrics)` would read false when present
- **Also likely:** `witness.rs:687` `omissions`, `synthesize.rs:6421` `invariants`; `ess-entity-runtime/src/lib.rs:3393` (`Condition::Exists`, external entity-core) — inferred
- **Documents:** `website/docs/reference/predicates.md:166` — inferred
- **Open:** presence marker for a present but empty struct/map; whether a format gate is needed
- **Confidence:** medium
- **Would collide with:** `expression.rs` predicate typing; `ess-primitives` facts/predicate; fact flattening in `input.rs`/`runner.rs`; Go/TS predicate evaluators
