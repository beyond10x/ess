---
format: aep.planning-md/3
id: story:a-scenario-may-publish-a-trigger-event-through-a-test-adapter
kind: story
status: draft
title: A scenario may publish a trigger event through a test adapter, so a refusal its only emitter guards can be arranged
summary: 'Split from the 0.69.0 refusal-policy defect: a suite step that publishes a published event with chosen fields; format decision open.'
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-refusal-policy-scenario-is-not-set-up-through-a-trigger-that-needs-the-row
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/target.rs
revision: 2
---
## Need

Split from `story:a-refusal-policy-scenario-is-not-set-up-through-a-trigger-that-needs-the-row` on 2026-10-22. A binding's
`binding/refusal/<outcome>` scenario for a missing related row cannot be arranged when every emitter of the trigger event
refuses while that row is absent: no sequence of the model's own commands delivers the event with the row absent. 0.69.0
refuses that scenario by name (ESS-SYNTH-010). This story would let the suite publish the trigger's event itself, so the
scenario runs.

## Proposal

A test-adapter control step that publishes a specified event with fields the suite chooses (here the related identity fresh,
never stored), in the manner of `ConfigureExternalOutcome` (§12, `crates/verify/ess-conformance/src/scenario.rs:2321`) and
shaped like `DeliverEvent` (`scenario.rs:2585-2603`, which today takes only events nothing in the specification publishes).

## Where it would land (inferred)

- `crates/verify/ess-conformance/src/scenario.rs` (`ScenarioStep`), `src/runner.rs` (beside the `DeliverEvent` arm, :713),
  `src/target.rs` (target capability), `src/interpret/` (interpreted target), the Go scenario player's version gate.
- `crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1300` (`related_missing`): emit the step instead of the
  ESS-SYNTH-010 refusal.

## Decisions

Open (2026-10-22): whether to add a suite format version for this step, or to widen `DeliverEvent`'s contract, or to decline.
Every target adapter would have to implement the step, and a player below the new version must refuse it by name.

## Acceptance

Owed once the format decision is taken: the d2-shaped model's `binding/refusal/role-missing` runs and passes on the interpreted
target, a target that retries `role-missing` fails it, and an older player refuses the suite by name before any target is
reached.
