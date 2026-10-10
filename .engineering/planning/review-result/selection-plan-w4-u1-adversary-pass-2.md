---
format: aep.planning-md/3
id: review-result:selection-plan-w4-u1-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave 4 unit U1 (interpreter reads the plan)
relations:
- reviews: story:interpreter-reads-selection-plan
revision: 1
---
```
unit: story:interpreter-reads-selection-plan, pass 2, uncommitted tree at base 373df4ecba
verdict: NEEDS-CHANGE
cases: executed 207→209, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/ess-selection-plan/w4-u1-scratch/adversary-1/ (pass2/)
needs-coordinator: whether the exchanged-pair contract binds this story
```

Added `adversary_selection_plan_w4_u1_pass2.rs`: InputAbsent↔Accepting exchanged should still answer `too-big` (got `unknown`); RelatedRow↔Existence exchanged on `related-guard-multiple.yaml` `StartRun` should answer `no-such-capability` (got Undetermined).

Real order: suite probe and mutation audit identical to the base. 28 pairs × 5 requests: 128 agree, 4 covered by the fixed-before-select statement, 8 not (the two findings).

```findings
[
  {"file": "crates/verify/ess-conformance/src/interpret/execute.rs", "line": 536, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "once the walk passes any select-read phase the input-refusal step is deferred, so Existence and RelatedRow answer before InputRefusal even where the plan reads InputRefusal first"},
  {"file": "crates/verify/ess-conformance/src/interpret/execute.rs", "line": 709, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "held_rows reads input-named related rows at the Existence phase, so Existence read before RelatedRow turns a missing related row into Undetermined instead of its exists-false branch"}
]
```
