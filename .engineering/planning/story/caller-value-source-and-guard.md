---
format: aep.planning-md/3
id: story:caller-value-source-and-guard
kind: story
status: implemented
title: The authenticated caller is a value source and a guard operand
refs:
- provider: github
  reference: beyond10x/ess#168
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/actor.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/web.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T19:16:19Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T19:19:27Z", actor: "human:timo", revision: 6, imported: true}
- {from: "active", to: "implemented", at: "2026-09-28T04:35:24Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Scope

Actor fields, a `{caller: <field>}` source, and guards comparing a caller field with an input or a stored field. Needs the conformance adapter to supply the caller.

## Acceptance

The #168 repro validates; scenarios run as two callers and require the 403 for the one that is not the record's agent.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain` (actor declaration, value sources, guard admission) — cited (#168 quotes these parsers)
- **Actor:** `ess-domain/src/actor.rs` `ActorSpec` (47), `RawActorSpec` (`deny_unknown_fields`) gain `attributes` — cited
- **Source:** `command.rs` `PayloadSource` (~870-940), keywords (1057), raw parse (~1150-1210) — cited; `command/value_expression.rs` gate (122) — inferred
- **Guard operand:** `command/subject_fact.rs`, `command/subject_state.rs`, `ess-primitives/src/predicate.rs` (334) — inferred
- **Compiler/IR:** `resolve.rs`, `ir.rs` (`ResolvedActor`), `expression.rs` — inferred
- **Adapter protocol (new suite format needed):** `scenario.rs` `ExecuteCommand { actor }` (1864) carries only a name; `target.rs` `SemanticCommandRequest.actor` (471); `runner.rs` (1104, 1343); `coverage_build.rs` (482) — cited symbols, change inferred
- **Runtimes/web:** Go `runtime.go` (1453, 908), TS `runtime.ts`, `web.rs` (207), `web_replay.rs` (270) — inferred
- **Generators/diff:** `ess-entity-runtime`, `ess-gen/src/{http,openapi,docs}.rs`, `ess-synth/src/plan.rs` (851), `ess-diff/src/{diff,change}.rs` — inferred
- **Registries:** source + suite format bump, `models/toolchain/domains/specify.yaml` — cited
- **Note:** #157 (input vs stored field in `when_subject`) shipped in 0.37.0, so the stored-field comparison has a host
- **Confidence:** medium
- **Would collide with:** `PayloadSource` parsing, guard admission, the scenario/target protocol, every format registry
