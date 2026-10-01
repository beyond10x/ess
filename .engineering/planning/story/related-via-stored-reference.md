---
format: aep.planning-md/3
id: story:related-via-stored-reference
kind: story
status: active
title: when_related reads through a stored field of the subject, Optional included (ess/21)
refs:
- provider: github
  reference: beyond10x/ess#304
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:feature-request-282
- depends_on: story:related-via-optional-input
- informed_by: story:related-guard-vocabulary-aligns
scope:
- confidence: cited
  path: crates/generate/ess-synth/tests/related_guard_obligation.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/related_guard_ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_command_execution.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_stored_reference.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:54Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:55Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

A `when_related` guard reads through a stored field of the addressed subject (`via: blocked_by`), Optional included: completing a task is refused while the task its stored `blocked_by` names is not Done (ess/21, story 2 of the design in beyond10x/ess#304, requested by uilab).

## Fit review

One design, one review: see story:related-via-optional-input, `## Fit review`.

## Decisions

Accept, redesigned: the subject via of that design, Optional included; it takes over acceptance line 1 of story:related-guard-vocabulary-aligns; it answers at the precedence step story:feature-request-282 adds. The interpreter executes the subject via (lifting the decline at `crates/verify/ess-conformance/src/interpret/execute.rs:524-528`), so `determined.rs:3-4` ("expressible with the interpreter's semantics") holds when story:related-guard-behaviour generates it.

## Acceptance

- `crates/specify/ess-domain/tests/related_guard.rs`: `a_stored_reference_via_validates_under_ess_21`, `a_stored_reference_via_is_refused_on_creates_and_without_a_subject`
- `crates/specify/ess-compiler/tests/related_guard_ir.rs`: `a_stored_reference_lowers_to_resolved_related_via_subject`
- `crates/verify/ess-conformance/tests/interpreted_command_execution.rs`: `the_interpreter_executes_a_stored_reference_guard`, `the_interpreter_reads_no_row_for_an_absent_stored_reference`
- `crates/verify/ess-conformance/tests/related_guard_stored_reference.rs` (new): `a_blocked_task_is_refused_until_its_blocker_is_done`, `a_target_reading_another_rows_state_fails`, `a_target_reading_the_subjects_own_state_fails`, `a_target_refusing_whenever_a_blocker_is_set_fails`, `a_dangling_reference_is_witnessed_or_noted_unreachable`
- `crates/generate/ess-synth/tests/related_guard_obligation.rs` (new; not `declared_behaviour.rs`, which story:go-generated-behaviour edits): `a_stored_reference_guard_stays_an_obligation_with_its_contract` (removed by story:related-guard-behaviour, which generates it)

## Scope

As story:related-via-optional-input, plus `ess-conformance/src/synthesize/related.rs` (subject-via arrangement), `ess-conformance/src/interpret/execute.rs:524-528` (subject via executed), `ess-compiler/tests/related_guard_ir.rs`, `ess-synth/tests/related_guard_obligation.rs` (new). story:the-interpreter-executes-stored-field-guards also edits `execute.rs`; whichever lands second rebases.

## Sequencing

After story:feature-request-282, #287 and story:related-via-optional-input (shares `related_guard.rs` in `ess-domain` and `ess-conformance`); inside the ess/21 bundle. Takes over acceptance line 1 of story:related-guard-vocabulary-aligns (relation `informed_by`). `CHANGELOG.md` is a merge-time edit (epic).
