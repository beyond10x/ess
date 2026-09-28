---
format: aep.planning-md/3
id: story:bounded-retry-bindings
kind: story
status: implemented
title: A binding can state an attempt bound and which failures are final
refs:
- provider: github
  reference: beyond10x/ess#165
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/accessor_cli.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: docs/design/binding-delivery-guarantees.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T19:06:13Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T19:08:17Z", actor: "human:timo", revision: 6, imported: true}
- {from: "active", to: "implemented", at: "2026-09-28T04:35:25Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Scope

Revisits `binding-delivery-guarantees.md` (no retry count) for bounds that are component behaviour: max attempts and final-failure classes on `on_failure:`.

## Acceptance

With a scripted receiver, the suite requires exactly N attempts on 5xx and one on a final 4xx.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (on-failure synthesis) + `crates/specify/ess-domain` (`on_failure:` parsing) — cited
- **Domain:** `ess-domain/src/binding.rs` `RawBindingSpec::on_failure` (215), `Failure` (294), `RawFailure` (386, hand-written Deserialize 408) — cited
- **IR:** `ess-compiler/src/ir.rs` `ResolvedFailure` (1642) — cited
- **Synthesis:** `synthesize.rs` `on_failure` (7274), `Retry` arm (7323), `BindingGap::PolicySilent` = ESS-SYNTH-010 (1059, 7285) — cited
- **Steps:** `scenario.rs` `ConfigureExternalOutcome` (1859, next outcome only), `ExpectInvocation` (2025, no count) — cited; a repeated forced failure and an invocation count are needed — inferred
- **Runners/targets:** `runner.rs`, `target.rs`, `reference.rs`, `faulty.rs`, Go/TS runtimes — inferred
- **Generated runtimes:** `ess-synth/src/{rust,go}/system.rs` (retry has no bound), `rust/feasibility.rs`, `plan.rs` — inferred
- **Also likely:** `ess-gen/src/{docs,asyncapi,graph}.rs`, `ess-synth/src/web/*`, `ess-diff`, `ess-entity-runtime` — inferred
- **Tests:** `tests/synthesis.rs:2238`, `ess-cli/tests/accessor_cli.rs:47` pin ESS-SYNTH-010 — cited
- **Registries:** source + suite format bump — cited pattern
- **Documents:** `docs/design/binding-delivery-guarantees.md` ("No retry count") — cited
- **Open:** the model has outcomes, not HTTP codes: "5xx retried, 4xx final" maps to outcome classes
- **Confidence:** medium
- **Would collide with:** `synthesize.rs`, `scenario.rs` steps, source/suite format bumps, `binding.rs`
