---
format: aep.planning-md/3
id: review-result:related-via-stored-reference-20261004-r1
kind: review-result
status: active
title: 'Stored-reference via adversary pass 1: wrong_state composition, required witness, old refusal'
relations:
- reviews: story:related-via-stored-reference
revision: 1
---
unit: W2-5 #304 slice 2 (stored-field via), pass 1
verdict: NEEDS-CHANGE
cases: executed 57→68, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (probe, dumps, logs); two build dirs cleaned
needs-coordinator: none

Publication copy of the only adversary pass on the #304 slice-2 unit (worktree `<worktrees>/ess/ess-w2-304-stored-reference-20261004`, base `9d2e45355`, uncommitted diff of 19 files). The reviewer added `crates/verify/ess-conformance/tests/adversary_related_via_stored_pass1.rs` and `crates/specify/ess-domain/tests/adversary_related_via_stored_pass1.rs`; no production file edited. Byte drift: 53 single-file fixtures synthesized at the unit and at base are identical except the new stored-reference fixture; committed suites unchanged.

F1 (blocker) `crates/specify/ess-domain/src/command/related_guard.rs:606`: a stored via admits `wrong_state` beside an accepting `when_related` branch that moves the subject (the same shape with an input via is refused). The interpreter's held-state step (`execute.rs:1095-1105`) selects with related branches skipped, so for a Cancelled task it answers `blocker-missing`/`blocked` where the synthesized state-refusal scenarios expect `wrong-state`; the interpreter fails its own suite (`adversary_related_via_stored_pass1.rs:660`).

F2 (warning) `crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs:185`: for an admitted required stored reference no arranging run can drive the subject through the command, so `wrong_state` is never witnessed, and the refusal claims `completed` is reached by no input (`:861`).

F3 (warning) `related_guard.rs:754`: below ess/22 a bare via naming a stored non-identity field is refused `unsupported_format_version` naming ess/22 instead of its old `type_mismatch`, although ess/22 refuses it too (ess-domain adversary file `:50`).

Note: ESS-SYNTH-008 text on a stored-via command says the row is one "the input names".

Attacked and not broken: pre-branch read when the branch sets the field; earlier rewrite and `cleared`; held state against open or dangling blocker; both declaration orders with and without wrong_state; unchecked later writer not called unreachable; faulty targets reading the reference before held state or the post-branch value are caught.

```findings
[
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 606, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a stored via admits wrong_state beside an accepting when_related move, and the interpreter (execute.rs:1095 held-state step skips related branches) answers blocker-missing/blocked where the synthesized state-refusal scenarios expect wrong-state, so the interpreter fails its own suite"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs", "line": 185, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "for an admitted Required stored reference no arranging run can drive the subject through the command, so wrong_state is never witnessed and the refusal claims completed is reached by no input"},
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 754, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "below ess/22 a bare via naming a stored non-identity field is refused unsupported_format_version naming ess/22 instead of its old type_mismatch, although ess/22 refuses it too"}
]
```
