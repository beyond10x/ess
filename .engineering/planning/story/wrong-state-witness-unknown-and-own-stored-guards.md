---
format: aep.planning-md/3
id: story:wrong-state-witness-unknown-and-own-stored-guards
kind: story
status: implemented
title: A wrong-state witness handles unknown sibling guards and the moving branch's own stored guard
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T13:05:44Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-01T13:05:44Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-01T15:20:30Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
# Story: a wrong-state witness handles unknown sibling guards and the moving branch's own stored guard

## Outcome

The wrong-state witness row never lets a non-moving default answer, and never stops selection on an
unknown guard: the moving branch's own `when_subject:` holds on the row, and a moving sibling is
exempt only while its stored guard decides true or false there.

## Why

Adversary pass 2 on the #192 unit (`review-result:adversary-n-mixedguard-pass-2`), both
pre-existing (the cases fail on base `a4b422e7e` too), measured against a target ordered as the
Entity Runtime lowering (`ess-entity-runtime/src/lib.rs:1450`, entity-core `select_outcome` /
`admit_state`):

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:698`: the witness checks only the
  moving branch's input half; a row its `when_subject:` rejects lets a non-moving default answer
  instead of wrong_state.
- `subject_fact.rs:660` (`answered_by_state`): a moving sibling is exempt whatever its guard
  evaluates to; a guard unknown on the row (optional field left absent) stops entity-core selection
  with `OutcomeUnobservable` before wrong_state.

Reached only by the adversary's fixtures; nothing found in committed examples.

## Acceptance

- The three cases of the pass-2 adversary file (kept at
  `~/.local/state/worktree/archives/ess/mixedguard-adv2/adversary_mixed_guard_pass2.rs`) pass:
  `adversary2_the_moving_branchs_own_subject_guard_must_hold_on_the_row`,
  `adversary2_a_row_only_moving_branch_beside_a_preserving_default`,
  `adversary2_an_exempt_moving_sibling_whose_guard_is_unknown_on_the_row`.
- Where no row satisfies both, synthesis refuses with ESS-SYNTH-003.

## Scope

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` — cited
- `docs/design/cross-record-and-stored-field-guards.md` (Wrong state) — cited

## Delivered

D350, 2026-11-01: unit a886ef467 (merged bd5b4296e), fix round b67d7661a with the adversary's cases (merged 13982222c). `subject_fact.rs` `refusal_input_from` and `selection`: the wrong-state witness is a row where the moving branch's own guard holds and every exempt sibling's guard is known; a guard that reads a field the row leaves empty and that the known leaves do not decide stops selection; otherwise ESS-SYNTH-003. The three pass-2 cases pass, case 3 as reworded by the coordinator (written and passing, or refused ESS-SYNTH-003 naming the guard), since no correct synthesis can satisfy the archived assertion on that model. Design text § Wrong state updated. Scope: both lines cited, confirmed by the baseline run. Out of scope: `falsified` and `leaves_external` keep the #234 ruling, now `story:unknown-by-absence-in-falsified-and-leaves-external` (needs a decision).
