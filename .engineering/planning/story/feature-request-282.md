---
format: aep.planning-md/3
id: story:feature-request-282
kind: story
status: active
title: ESS-COMMAND-004 refuses a when_related refusal beside a wrong_state outcome; no precedence is stated
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#282
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/interpreted_command_execution.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/related_guard_moves.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: cited
  path: docs/design/input-guard-overlap-precedence.md
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T22:51:33Z", actor: "human:timo", revision: 19, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-10-02T22:51:33Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

A command with a `when_related` refusal and a `wrong_state` outcome validates, with a stated precedence between them.

## Acceptance

The two reported #282 shapes must validate and synthesize witnesses under ess/21: an objective guarded by a paused related switch, and a deployment guarded by release approval. The precedence is documented in docs/design/input-guard-overlap-precedence.md and the authoritative cross-record-and-stored-field-guards order.

Named conformance controls to implement (prospective names, not claims of existing coverage):

- `issue_282_related_refusal_and_wrong_state_validate_under_ess_21`: both reductions validate with declaration order reversed as well.
- `issue_282_overlap_below_ess_21_keeps_its_refusal`: older supported formats retain their existing refusal/meaning; no silent widening.
- `issue_282_wrong_state_precedes_related_predicate_refusal`: where both predicates hold for a moving command, interpreter answers wrong_state, with no success event or storage mutation.
- `issue_282_related_predicate_refuses_from_an_allowed_state`: correct own state plus a disallowed related row selects the related refusal; allowed related row reaches acceptance.
- `issue_282_missing_related_row_keeps_its_existing_precedence`: the earlier missing-row branch remains distinguishable from the new predicate step.
- `issue_282_nonmoving_acceptance_keeps_its_state_independence`: do not impose a moving branch's held-state test on a nonmoving acceptance.
- `issue_282_synthesis_witnesses_both_lifecycle_and_related_refusals`: retained suite contains and executes both branch families against an honest interpreter.
- `issue_282_wrong_precedence_target_fails_its_suite`: a controlled target that swaps the overlapping refusal order fails the named scenario.

Source test homes are the scoped related_guard, related_guard_moves and interpreted_command_execution targets; implementor confirms exact placement. Real present-related predicate execution is necessary on current main, which declines that form; no simulated success or skip counts as conformance. The ess/21 bundle merge boundary remains subject to the existing coordinated format decision.

## Origin

beyond10x/ess#282, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: a command refusing on a related row's fields beside its own `wrong_state`. Class: gap. Existing idiom: none; the precedence order (`docs/design/cross-record-and-stored-field-guards.md#the-precedence-order`) has no step for a `when_related` predicate refusal, which is why ESS-COMMAND-004 refuses the pair. Fit: adding one step to that single order, not a per-command declaration.

## Decisions

- **accept, redesigned:** no author-declared precedence. The order gains one step: a `when_related` predicate refusal answers after the held state (step 4) and before accepting branches (step 5), so the addressed row's own lifecycle answers first. Admitted from ess/21 only (ess/20 ships alone in 0.49.0; this bundles with family F); validation, synthesis and the interpreter follow the order, and the design doc states it.

## Scope

Read-only story-scoper inspected current main on 2026-10-03. Confidence high for implementation surfaces, medium for test placement and format plumbing. Cited: crates/specify/ess-domain/src/command/related_guard.rs:262 (WrongState conflict and partition validation); crates/verify/ess-conformance/src/synthesize/related_guard.rs:135 (arrangement and selection); crates/verify/ess-conformance/src/synthesize.rs:4809 (related-command arrangement refusal; wrong_state_scenario at8165); crates/verify/ess-conformance/src/interpret/execute.rs:343,660 (current ordering and unsupported related predicates); crates/generate/ess-synth/src/plan.rs:709 (obligation precedence contract); docs/design/input-guard-overlap-precedence.md (acceptance citation); docs/design/cross-record-and-stored-field-guards.md:605 (authoritative order).

Inferred: crates/specify/ess-domain/src/command.rs:2705 for source-format admission context; crates/specify/ess-domain/src/system.rs:117 for ess/21 bundle constants; tests in crates/specify/ess-domain/tests/related_guard.rs, crates/verify/ess-conformance/tests/related_guard_moves.rs and crates/verify/ess-conformance/tests/interpreted_command_execution.rs. Compiler IR and schemas are not established as necessary for #282 alone; report evidence before extending scope.

Both Optional-via and stored-via stories overlap validation, related-row arrangement and interpreter files. Keep them serial. Preserve the early missing-related-row refusal and distinguish moving branches from nonmoving acceptance; only predicate-refusal precedence moves after held-state. This is cited code inspection, not executed correctness evidence. Related predicates are currently declined by the interpreter, so implementation requires real predicate execution as well as branch ordering. Existing candidate batch/consumer-interpreted-related-20261002 is being inspected as a possible reusable prerequisite; it is not silently considered landed.

## Current source allocation and resumed implementation

The accepted source allocation in the consumer backlog and current integration runbook assigns one-time responses to ess/21 and the coordinated syntax bundle to ess/22. Therefore every historical ess/21 admission boundary and prospective test name in this story now means ess/22; older sources through ess/21 retain the existing refusal. Update binding designs before implementation. This preserves the accepted precedence semantics and delivery boundary; it does not broaden scope.

The full frozen runtime handoff has been applied as local bot integration commit48d5cc77b38f919381d04830475ff6ea4959289f after current released main. The present-related interpreter prerequisite is now present; do not duplicate it or import old carrier ancestry. A read-only scope refresh confirms the named eight controls and narrow files, including interpret/execute/related.rs. Root delegates only FormatVersion::V22 plus its supported-source admission/compatibility plumbing to this serial unit; other format/default/schema projections remain coordinator-owned unless a measured need is reported.

Next unit: ess-serial-282-20261003 from48d5cc77b, branch unit/serial-related-precedence-20261003, own target and assigned scratch. The292 source remains in its own tree and integrates serially. Its implementation and tests must finish; source-read/preparation may proceed independently. Final combined checks and issue closure remain pending.
