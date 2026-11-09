---
format: aep.planning-md/3
id: story:unknown-by-absence-in-falsified-and-leaves-external
kind: story
status: draft
title: 'Unknown by absence in falsified and leaves_external follows the wrong-state witness, or #234 is restated'
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Finding

Found by U3 of wave D350 (out of scope there; `review-result:adversary-d350-u3-pass-1`) and unchanged on main at
0.92.0. Two synthesis paths in `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` read a sibling's stored
guard that is Unknown only because a field the row leaves absent through `guard_truth_with` (:497), which turns such an
Unknown into `False` (`evaluate_row` :652, `unknown_by_absence` :511), per the beyond10x/ess#234 ruling ("only true
takes it", doc at :488-496):

- `falsified` (closure at :3264), used by the wrong-state witness to decide that a sibling's stored guard is refuted on
  the arranged row (:3473, :3498, :3557, :3572, :3599);
- `leaves_external` (:2994), whose loop over `answered_before` branches `continue`s on `Truth::False` (:3046) and so
  sends the external witness past an earlier stored guard the row leaves Unknown by absence.

D350 U3 made the wrong-state witness's `selection` closure (:3305-3400) treat the same absence as stopping selection
(`OutcomeUnobservable`), because a target in Entity Runtime's order (three-valued guards, an Unknown stops selection)
fails a scenario that assumes the guard was not taken. The interpreter also stops on an Unknown subject guard
(`crates/verify/ess-conformance/src/interpret/execute/subject.rs:128`, `Undetermined::Undecidable`). The two paths
above still follow #234, so one synthesized suite applies two rules to the same guard on the same row. No design page
states either rule beside the other (`docs/design` has no #234 text).

## Decision needed

Reversing #234 for these two functions is a ruling, not a fix: (a) both follow the D350 rule (Unknown by absence
stops selection), or (b) #234 stands for them and the design text says why the wrong-state `selection` reads absence
differently. Until a person decides, this stays a draft.

## Acceptance

- (a) A model whose earlier sibling's stored guard reads an `Optional` field no creator writes (`note == "x"`) beside
  an `external:` branch, and one beside a wrong-state witness: synthesis does not send either witness to that row
  (it picks another row or refuses by name), and an Entity-Runtime-ordered test target passes the suite. Test:
  `crates/verify/ess-conformance/tests/unknown_by_absence_witnesses.rs`, cases
  `an_external_witness_skips_a_row_an_earlier_guard_leaves_unknown_by_absence` and
  `a_falsified_sibling_is_not_one_left_unknown_by_absence`, red before the change.
- or (b) The rule is written in `docs/design/cross-record-and-stored-field-guards.md` (§ The precedence order) beside
  the D350 rule, naming `falsified` and `leaves_external`, and this story is closed with that commit.
- Either way, every existing case in `adversary_d350_u3_pass1.rs` and `adversary_mixed_guard_pass2.rs` keeps its
  verdict.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- cited: crates/verify/ess-conformance/src/interpret/execute/subject.rs
- inferred: docs/design/cross-record-and-stored-field-guards.md
- inferred: crates/verify/ess-conformance/tests/unknown_by_absence_witnesses.rs

confidence: medium
