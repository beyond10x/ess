---
format: aep.planning-md/3
id: story:feature-request-267
kind: story
status: proposed
title: Binding flow, delivery and drop are synthesized into commands with wrong_state
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#267
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/accessor_cli.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: docs/design/binding-delivery-guarantees.md
- confidence: cited
  path: models/toolchain/README.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:15Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`binding/flow` and `binding/delivery` are synthesized for a binding into a command that has a `wrong_state` branch, and `binding/on-failure` for `drop` says what it observes.

## Acceptance

- Flow and delivery scenarios arrange the bound command's entity in a state it accepts; no ESS-SYNTH-010 for them.
- For `drop`, a scenario observes that nothing changed and nothing further was published, or the refusal states it is an inherent limit.

## Origin

beyond10x/ess#267, reported downstream on 0.48.0.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the tree, **inferred** = a reading. `CHANGELOG.md` and `changes/` are the coordinator's at merge and are not scope entries.

- **Cause:** `crates/verify/ess-conformance/src/synthesize.rs:10588` `reachable_branch` drops only `InjectFault` branches; a `wrong_state` branch (`TestStrategy::ArrangeState`) beside the default leaves two candidates → `BindingGap::BranchUndecided` → ESS-SYNTH-010 — cited
- **Files:** `synthesize.rs` `bindings` :10205, `flow` :10297, `delivery` :10432, `on_failure` :10493, `BindingGap` :1204/:1300-1395 — cited
- **Also likely:** `synthesize/delivery_context.rs` (:594, :698), state helpers `run` :2274, `run_state_refusal` :2607, `complete_wrong_state` :8532; `scenario.rs` `ExpectQuiet`/`ExpectSubjectUnchanged` used, not changed — inferred
- **Tests:** `crates/verify/ess-conformance/tests/synthesis.rs:2200-2305`, `crates/edge/ess-cli/tests/accessor_cli.rs:47` — cited
- **Generated:** `suites/generated/oracle-fixture/suite.json`, `suites/generated/README.md:84` — cited
- **Documents:** `docs/design/binding-delivery-guarantees.md:66,158`, `website/docs/reference/diagnostics.md:188`, `models/toolchain/README.md:131-134` (names this exact `check-on-tag` refusal) — cited
- **Confidence:** high for flow/delivery; medium for `drop`
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 265 on `ess-conformance/src/synthesize.rs`; 266 on `ess-conformance/src/synthesize.rs`; 268 on `ess-conformance/src/synthesize.rs`, `docs/design/binding-delivery-guarantees.md`; 269 on `ess-conformance/src/synthesize/delivery_context.rs`, `ess-conformance/src/synthesize.rs`, `docs/design/binding-delivery-guarantees.md`; 273 on `ess-conformance/src/synthesize.rs`
- **Safety fact:** `reachable_branch` has two callers (`synthesize.rs`, `delivery_context.rs:594`), so narrowing it changes no other family — git grep, unproven
