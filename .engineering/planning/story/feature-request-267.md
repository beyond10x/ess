---
format: aep.planning-md/3
id: story:feature-request-267
kind: story
status: implemented
title: Binding flow, delivery and drop are synthesized into commands with wrong_state
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#267
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:feature-request-266
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/accessor_cli.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner/bounded_retry.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner/delivery_context.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts
- confidence: cited
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: docs/design/binding-arrangement-and-drop.md
- confidence: cited
  path: docs/design/binding-delivery-guarantees.md
- confidence: cited
  path: models/toolchain/README.md
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:15Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T13:32:48Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:59Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"test_result":1,"review_outcome":6,"verification":1}}}
---
## Outcome

`binding/flow` and `binding/delivery` are synthesized for a binding into a command that has a `wrong_state` branch, and `binding/on-failure` for `drop` says what it observes.

## Acceptance

- Flow and delivery scenarios arrange the exact mapped destination identity in an eligible state before the trigger; no ESS-SYNTH-010 merely because a wrong_state branch exists.
- Drop forces one declared refusal after arrangement, observes every attempt's mapped input with ExpectEveryInvocation (empty selecting), then exactly one total attempt with ExpectInvocation (empty input, count1) throughout its eventual window. It reads the unchanged subject afterward. Zero delivery, an extra correct retry, an extra malformed-input retry and a success-effect mutant each fail.
- ExpectQuiet remains an event observation and is not accepted as evidence of absent command attempts. The historical zero-invocation wording was incompatible with forcing a refusal on the next invocation; this resolves that contradiction in the accepted drop intent.
- Named fixture scenarios and required native/generated Rust/generated Go execution and native/Go/TypeScript runner controls are bound in docs/design/binding-arrangement-and-drop.md. Existing suite instruction meanings/format remain unchanged. Independent design review is pending before implementation dispatch.

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

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: flow/delivery scenarios for bindings into commands with `wrong_state`. Class: defect. Fit: synthesis only. The `drop` acceptance was an either/or that could not be checked.

## Decisions

- **accept, tightened:** for `drop`, the scenario asserts that no command is invoked within the eventual window (`ExpectQuiet`); the "or the refusal states an inherent limit" alternative is removed from the acceptance.

## Current coordinated binding contract

The 2026-10-03 remaining-bundle execution uses docs/design/binding-arrangement-and-drop.md as the concrete contract for #266/#267. Arrangement tracks eventual binding effects and never races a binding with an explicit route command. Drop needs one forced failed attempt and no retry, not an empty event log or zero attempted delivery. Exact total attempt count uses empty input plus the independent every-invocation mapped-input check; unsupported observation cannot pass. Required healthy/faulty controls are named in that document. This is coordinator resolution before implementation, pending independent design review; historical fit/scope prose remains evidence of intake, not current execution authority.

## Design revision 2

The four findings in review-result:binding-arrangement-drop-design-20261003-r1 are fixed in docs/design/binding-arrangement-and-drop.md: reconcile obsolete mapping-only tracing documentation with shipped retry/every-invocation semantics; specify cumulative non-consuming per-correlation snapshots and adapter controls; require pre-trigger mapped destination identity authority with DestinationIdentityUnavailable for post-trigger-only values; and pin QueryView/SnapshotSubject before the trigger plus QueryView/ExpectSubjectUnchanged after count observation. These are prospective source/adapter tests, not completed execution. Independent final design review remains due.

## Current design disposition

Final independent design reviews at aec396fe6 approved the arrangement/drop contract and the conditional/per-refusal contract (review-result:binding-arrangement-drop-design-20261003-r2 and review-result:conditional-binding-design-20261003-r2). All four and three first-round findings, respectively, were fixed. The matching docs/design pages now bind implementation. Prior pending-design wording is historical; implementation, decisive target controls and independent source review are still required. Serial #266 -> #267 -> #268/#194 -> #269 order and the one bundle PR remain unchanged.
