---
format: aep.planning-md/3
id: story:feature-request-266
kind: story
status: proposed
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
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:15Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Synthesized scenarios describe the system with its bindings running: a step that publishes an event a binding reacts to is followed, in arrangement and expectation, by the state the bound command leaves.

## Acceptance

- After a command whose event triggers a state-moving binding, synthesized expectations read the bound state, not the pre-binding state.
- No synthesized step sends the bound command explicitly as though the binding had not run.
- A downstream-shaped fixture (event -> command moving state) passes against an implementation that runs its bindings, and fails against one that does not.

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

- **defer:** `decision-blocker:bound-state-observation` blocks this story until the observation semantics are decided and a minimal ESS reproduction exists.
