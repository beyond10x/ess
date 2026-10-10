---
format: aep.planning-md/3
id: review-result:selection-plan-w5-u3-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 5 unit U3 (subject_fact reads the plan)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: wave 5 U3, story:synthesis-reads-selection-plan; uncommitted tree on base 3f0e8e1539
verdict: CONFIRMED (none blocks U3; the 3 red cases are outside U3's file and only fail under a test-only phase order)
cases: executed 93→96, red 3
origin: introduced 0 / pre-existing 0 / undecided 2
wrote-outside-worktree: ~/.cache/ess-selection-plan/w5-u3-scratch/adversary-1/ (probe and logs, no build)
needs-coordinator: which story owns the existence/input-refusal sends; mutant and base runs not done (disk rule)
```

Cases (with InputRefusal and HeldState exchanged under `with_phase_order`, the interpreter under the same order): RotateSecret `too-short` (no declared outcome reached), RenewLease `blank-note` (observed `unknown-lease`), Bind `too-short` (observed `already-bound`). All in synthesis code outside `subject_fact.rs`. Parked in `~/.cache/ess-selection-plan/w5-u3-scratch/adversary_selection_plan_w5_u3_pass1.parked.rs` for `story:existence-family-sends-read-selection-plan`.

Not broken: formats (validation refuses `when_related` beside `when_subject*`, so the two format choices never disagree); `first_read` vs old `FirstDeclared` (identical under the real order by #489 and the #486 rule); `leaves_external`'s list; a sweep of 17 `when_subject` fixtures under 4 orders. Mutants not run under the disk rule; by reading, each named mutant is caught by one of the four new cases.

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 6053, "category": "property", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "is_state_input_refusal sends a state input refusal with no arranged row, assuming it answers before Existence; with InputRefusal and HeldState exchanged the interpreter answers existence first"},
  {"file": "crates/verify/ess-conformance/tests/fixtures/explore-stored-rows.yaml", "line": 1, "category": "property", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "the creation family sends Bind too-short with a key already bound, assuming the input refusal answers before existing_instance; with InputRefusal read after Existence the interpreter answers already-bound"}
]
```
