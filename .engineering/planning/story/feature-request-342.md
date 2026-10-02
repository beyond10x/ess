---
format: aep.planning-md/3
id: story:feature-request-342
kind: story
status: implemented
title: Guarded deletion leaves no subject in synthesized assertions
refs:
- provider: github
  reference: beyond10x/ess#342
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/outcome_shapes.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:12:15Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:12:15Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-02T13:29:01Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":2}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A deleting outcome with a when_subject refusal sibling synthesizes executable absence checks, without later asserting the deleted row or its invariant.

## Fit review

1. Need: deleting a cart after checking its revision must leave it absent. The minimal cart/revision model in issue #342 reproduces the reported contradictory assertions; no new syntax is proposed.
2. Class: defect. docs/design/outcome-shapes.md:77-81 explicitly requires absence and unknown-instance behavior after deletion.
3. Existing expression: deletes already expresses removal. Dropping when_subject loses revision semantics and is not a suitable idiom.
4. Fit: ordinary preparation returns None for ResolvedEffect::Deletes (synthesize.rs:3442), but stored-field preparation unconditionally sets after: Some(after) (synthesize/subject_fact.rs:4072-4086). Invariant synthesis consumes run.after (synthesize.rs:10349). Correct the shared setup, preserving independent absence/event assertions and all existing source vocabulary; no target-specific new semantics.
5. Second adopter: revoke an expired credential only when the caller's revision matches, then observe absence. This uses the same existing deletion contract.
6. Cost: no new syntax, scenario step or persisted format. Only invalid generated assertions change. Review all Setup.after consumers.
7. Alternatives: change nothing leaves impossible suites; special-casing invariants misses the closed-scenario symptom; correcting the stored-field setup addresses the common cause.

## Decisions

Accept as proposed. Correct deletion state in stored-field witness arrangement. User's full-backlog goal authorizes this bounded correction in the ongoing synthesis batch. This is not an ESS evolution task.

## Acceptance

- issue_342_guarded_delete_passes_an_honest_cart_target: minimal guarded deletion/invariant model executes against an honest Rust target and passes, including subsequent missing-instance refusal.
- issue_342_guarded_delete_never_asserts_the_removed_subject: no later contains or invariant assertion requires the deleted identity.
- Control without when_subject remains valid; a target retaining the row still fails the existing absence assertion.
- Preserve unrelated event expectations, stored guard semantics and existing deletion scenarios.

## Scope

Cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs; tests/outcome_shapes.rs; docs/design/outcome-shapes.md.
Cited read seam, possible edit inferred: crates/verify/ess-conformance/src/synthesize.rs (ordinary setup and deletion/invariant consumers).
Confidence: high for cause; execution must establish red before implementation.
