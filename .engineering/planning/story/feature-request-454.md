---
format: aep.planning-md/3
id: story:feature-request-454
kind: story
status: draft
title: An input-selected refusal cannot be ordered after unknown-instance and wrong_state
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#454
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/held-state-input-refusal.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/held_state_input_refusal.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: cited
  path: docs/design/outcome-shapes.md
- confidence: cited
  path: website/docs/guides/specify/guards-and-predicates.md
revision: 5
---
## Outcome
Resolve beyond10x/ess#454: An input-selected refusal cannot be ordered after unknown-instance and wrong_state.

## Origin
beyond10x/ess#454, filed 2026-10-05; found 2026-09-27 running a conformance suite against an implementation that checks existence and held state before an input code.

## Fit review
1. Need: a refusal of an invalid input that applies only to an existing record in a state that accepts the command. For an unknown identity the answer is `unknown_instance`; in a wrong state it is `wrong_state`. Requester's ask, theirs: "a way to order an input-selected refusal after the existence and held-state checks, or synthesis that respects the precedence the design states".
2. Class: convenience. The behaviour reported is ESS's documented order, not a defect. "The precedence order" puts input-guarded refusals at step 2, before existence (step 3) and the held state (step 4) (`docs/design/cross-record-and-stored-field-guards.md:706-735`). The adopter guide says "The synthesized suite sends the refused input for an identity nothing stores … A target that reads the held state or looks the record up before it checks the input fails the refusal's scenario" (`website/docs/guides/specify/guards-and-predicates.md:76-81`; also `docs/design/outcome-shapes.md:269-273`). The design the requester cites is their implementation's, not ESS's.
3. Existing idiom: guard the refusal by the held state as well. With `when_subject: {predicate: state == Active}` beside its `when:` (ess/18, `website/docs/reference/spec-versions.md:187-193`), it becomes a held-state branch (step 4). Existence then answers first, and in another state `wrong_state:` answers. Fresh probe `<fit-review scratch>/probe-454/spec2.yaml` on installed ess 0.52.0: "demo v1 — 1 file(s), valid", and synthesis writes 6 scenarios with 0 refusals (`synth2.out`). `invalid-code` is witnessed only on an arranged `Active` row (`code: ""`, `suite2.json`). The unknown identity in its first half is sent a valid code and requires nothing to change. The other spelling, `when_subject_state: Active`, is refused beside `wrong_state:` (`validate.out`, ESS-COMMAND-004). Without `wrong_state:` it is refused as open coverage (ESS-COMMAND-005, `held-state-default.yaml`). So the `when_subject` predicate form is the idiom.
4. Fit: the idiom reuses the existing step-4 rule and needs no new construct. One witness gap remains (probe output). `no-session` is sent only with a valid code, and the wrong-state scenario `Session/state/Paused/refuses/Pause` likewise. A target that checks the code before looking the record up therefore still passes. This is the overlap rule of `docs/design/input-guard-overlap-precedence.md` ("sent again at every overlap point") not yet applied to the step 3/4 boundary. Closing it: the existence-refusal and wrong-state scenarios also send an input a held-state refusal claims, requiring existence or `wrong_state`. That uses existing steps only (`execute_command`, `expect_outcome`). It runs in `existence::refusals_on_a_stored_row`/`held_state_refusals` (`crates/verify/ess-conformance/src/synthesize/existence.rs:506,1157`) and the wrong-state witness in `subject_fact.rs`. Hypothesis: the 0.53.0 head changed both files (`git diff --stat 0.52.0..HEAD`, 162 and 1295 lines) and might already send these overlaps. Not verified; the 0.53.0 binary was not built.
5. Second adopter: a cancellation reason is validated only for an order that exists and is still cancellable. An unknown order is `not-found` whatever the reason.
6. Cost: no format or keyword. One guide paragraph. Suite bytes change only for commands combining a `when_subject` refusal with existence or `wrong_state`. Measure across repository models.
7. Alternatives: (a) change nothing. The idiom exists but is undocumented, and its order is unwitnessed. (b) A per-branch precedence key (`after: [unknown_instance, wrong_state]`). That is new surface, and a second way to say what `when_subject` already says, against the "one precedence order" rule (`cross-record-and-stored-field-guards.md:708-710`). (c) Move step 2 after step 3 globally. That breaks every suite and Entity Runtime's lowering (`:728-733`). (d) Chosen: decline (b)/(c), document the idiom, and witness the idiom's own order.

## Decisions
decline, with the idiom. The story body is the decline record, with the probe output above. An input refusal that holds only for a live record is written with `when_subject: {predicate: state == <accepting state>}` beside its `when:`, which places it after existence and `wrong_state`. This story builds the missing witness, so a code-first target fails: existence and wrong-state scenarios also send an input the held-state refusal claims. No format bump.

This story owns the guide section `## Which branch answers` in `website/docs/guides/specify/guards-and-predicates.md`, placed after `## Select an outcome from the held subject state` (it replaces the one-line statement at :74). It also owns the overlap-witness rule that existence and wrong-state scenarios resend a claimed input. #455 depends on this story (coordinator decision 2026-10-05); it adds its own paragraph to that section and its own cases in its own test file, and builds on `input-guard-overlap-precedence.md` rules 1–2, not on a rule of this story's. #459 depends on this story (edge recorded): it raises the visibility of `identity_views` and `one_row_view` in `synthesize/existence.rs` after this story edits `held_state_refusals` (506) and `refusals_on_a_stored_row` (1157).

## Acceptance
- held_state_input_refusal_answers_after_existence: the committed fixture `crates/verify/ess-conformance/tests/fixtures/held-state-input-refusal.yaml` (the fit-review model, also the guide section's fenced model) validates; `no-session` is also sent `code: ""` for an unknown identity and requires `no-session`.
- held_state_input_refusal_answers_after_wrong_state: the `Paused` wrong-state scenario also sends `code: ""` and requires `not-active`.
- code_first_target_fails: a target checking the code before lookup fails `no-session`; an honest target passes.
- plain_input_refusal_keeps_step_two: a refusal with `when:` alone is still sent for an unknown identity first; existing bytes unchanged.
- which_branch_answers_section_states_the_order: `website/docs/guides/specify/guards-and-predicates.md` has the heading `## Which branch answers`. The section contains "input refusals answer first", "unknown instance", "`wrong_state`", "`when_subject: {predicate: state ==`" and "after existence". Its fenced model is the fixture `crates/verify/ess-conformance/tests/fixtures/held-state-input-refusal.yaml`, compared byte for byte. The case lives in `crates/verify/ess-conformance/tests/held_state_input_refusal.rs`; it reads the page and fails on a missing heading, phrase or model.

## Scope
- docs/design/cross-record-and-stored-field-guards.md  cited — the precedence order :706-735
- website/docs/guides/specify/guards-and-predicates.md  cited — refusal-first statement :74-81; new `## Which branch answers` section
- docs/design/outcome-shapes.md  cited — existence precedence :269-273, :290-310
- crates/verify/ess-conformance/src/synthesize/existence.rs  cited — held-state refusals :506, stored-row refusals :1157
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs  inferred — wrong-state witness
- crates/verify/ess-conformance/tests/held_state_input_refusal.rs  inferred — new regression test and the section case
- crates/verify/ess-conformance/tests/fixtures/held-state-input-refusal.yaml  inferred — the guide's example, validated
