---
format: aep.planning-md/3
id: story:feature-request-266
kind: story
status: implemented
title: Synthesized scenarios account for bindings that move state
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#266
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:15Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T13:32:48Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:59Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

Synthesized scenarios describe the system with its bindings running: a step that publishes an event a binding reacts to is followed, in arrangement and expectation, by the state the bound command leaves.

## Acceptance

- On the minimal reproduction attached to beyond10x/ess#266 (entity `Job` New→Started, `Create` emits `Created`, binding `Created -> Start`), no synthesized scenario sends `Start` to a job `Create` just made, and `Start/outcome/started` is either arranged without the binding or named in coverage.
- No view expectation after `Create` asserts state `New`; an eventual expectation asserts `Started`.
- A target that runs its bindings passes the suite; one that does not fails the binding flow scenario.

## Origin

beyond10x/ess#266; downstream 225 scenarios / 492 expectations contradicted bindings on 0.48.0.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the tree, **inferred** = a reading. `CHANGELOG.md` and `changes/` are the coordinator's at merge and are not scope entries.

- **Files:** `crates/verify/ess-conformance/src/synthesize.rs` — cited: `advance` :3447 sets `next.state = transition.to` (:3504) with no step for a binding the step triggers; `route_from` :3926 builds edges from `EssIr::drivers()` (`ir.rs:2456`), which includes binding-invoked commands, so a route can send the bound command explicitly; `prepare_subject` `after` :3177 is the branch's own `transition.to`; `written_elsewhere` :3264 covers fields only
- **Also likely:** `synthesize/subject_fact.rs` (`step` :3244, `reach_state` :3276), `synthesize/existence.rs` (:486, :832), `synthesize/related_guard.rs` (:526); `lifecycle`/`wrong_state_scenario` :7601/:7713; `reachable_branch` :10588 — inferred
- **Tests:** a new test and fixture with a target that runs bindings (the interpreter does not, `interpret.rs:250,306`) — inferred
- **Confidence:** medium
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 229 on `ess-conformance/src/synthesize/subject_fact.rs`; 265 on `ess-conformance/src/synthesize.rs`; 267 on `ess-conformance/src/synthesize.rs`; 268 on `ess-conformance/src/synthesize.rs`; 269 on `ess-conformance/src/synthesize.rs`; 271 on `ess-conformance/src/synthesize/subject_fact.rs`; 272 on `ess-conformance/src/synthesize/subject_fact.rs`; 273 on `ess-conformance/src/synthesize.rs`
- **Safety fact:** a binding-invoked command reaches route search only through `EssIr::drivers()` (`synthesize.rs:3393`, :7608), so filtering there changes arrangement and wrong-state enumeration only — unproven

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: synthesized expectations that account for state-moving bindings. Class: defect, unconfirmed: not reproduced in ESS; the interpreter runs no bindings (`ess-conformance/src/interpret.rs:247-255`); whether the bound state is observed immediately or eventually is undecided.

## Decisions

- **accept, redesigned (coordinator, 2026-09-30; blocker cleared by a minimal reproduction):** bindings are eventual, so no synthesized step races one. Synthesis never sends a bound command explicitly to a subject a triggering step's binding will move; where every route to such a subject triggers the binding, the binding's own flow scenario is the witness and the direct scenario is named in coverage as not arrangeable without racing the binding. View expectations after a triggering step assert only what the binding leaves unchanged, or the settled state after the eventual window.

## Current coordinated binding contract

The 2026-10-03 remaining-bundle execution uses docs/design/binding-arrangement-and-drop.md as the concrete contract for #266/#267. Arrangement tracks eventual binding effects and never races a binding with an explicit route command. Drop needs one forced failed attempt and no retry, not an empty event log or zero attempted delivery. Exact total attempt count uses empty input plus the independent every-invocation mapped-input check; unsupported observation cannot pass. Required healthy/faulty controls are named in that document. This is coordinator resolution before implementation, pending independent design review; historical fit/scope prose remains evidence of intake, not current execution authority.

## Current design disposition

Final independent design reviews at aec396fe6 approved the arrangement/drop contract and the conditional/per-refusal contract (review-result:binding-arrangement-drop-design-20261003-r2 and review-result:conditional-binding-design-20261003-r2). All four and three first-round findings, respectively, were fixed. The matching docs/design pages now bind implementation. Prior pending-design wording is historical; implementation, decisive target controls and independent source review are still required. Serial #266 -> #267 -> #268/#194 -> #269 order and the one bundle PR remain unchanged.
