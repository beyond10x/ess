---
format: aep.planning-md/3
id: story:a-binding-flow-awaits-the-event-its-trigger-causes
kind: story
status: draft
title: A binding flow awaits the event its trigger causes
summary: A bare eventually_event is met by an event the arrangement published before the trigger, so an unsteered set-guard flow still passes (turn-2 adversary F3, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-setguard-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

A binding flow scenario ends in a bare `eventually_event <Event>` with no payload. Where the
trigger's own arrangement already publishes that event (an arranged `Leave` fires the binding and
publishes `Kept` before the trigger runs), the step is met by that earlier event, so the interpreter
cannot tell a flow whose creating command was steered from one that was not. Found by the turn-2
adversary of `story:a-set-guard-fed-by-a-subject-payload-is-arranged`
(`review-result:zi-t2-setguard-adv-1`, finding F3): with the creator left unsteered the flow still
passes. The set-guard unit now checks the steering structurally (`binding_set_guard`
`the_creator_is_sent_the_value_the_rows_were_arranged_against`), but the emitted suite does not.

## Acceptance

- `renamed_creator_is_steered_and_the_unsteered_mutant_is_caught` in
  `crates/verify/ess-conformance/tests/adv_set_guard_subject_payload.rs` runs (no `#[ignore]`) and
  passes: the flow with the creator left unsteered fails on the interpreter.
- A binding flow whose arrangement publishes the awaited event before the trigger either awaits
  the event the trigger causes (by payload or by order) or names why it cannot.
- Byte-pin rows that move are named with this cause.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs` (the binding flow's eventually step)
- `crates/verify/ess-conformance/src/synthesize/binding_effects.rs`
- `crates/verify/ess-conformance/tests/adv_set_guard_subject_payload.rs`
