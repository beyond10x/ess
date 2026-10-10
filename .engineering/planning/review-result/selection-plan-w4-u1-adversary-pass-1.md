---
format: aep.planning-md/3
id: review-result:selection-plan-w4-u1-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 4 unit U1 (interpreter reads the plan)
relations:
- reviews: story:interpreter-reads-selection-plan
revision: 1
---
```
unit: story:interpreter-reads-selection-plan, uncommitted tree at base 373df4ecba
verdict: NEEDS-CHANGE
cases: executed 205→207, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/ess-selection-plan/w4-u1-scratch/adversary-1/
needs-coordinator: whether "every exchanged phase pair moves the answer" binds this story
```

Added `adversary_selection_plan_w4_u1_pass1.rs`: InputRefusal exchanged with HeldState should answer `stale` (got `too-big`); InputRefusal exchanged with Existence should answer `unknown` (got `too-big`). Real-plan controls pass.

Not broken: synthesized suites base vs unit, 186 models / 450 commands / 14,546 lines, `diff` exit 0; mutation audits, 148 models / 1,313 lines, `diff` exit 0; RelatedRow↔InputRefusal and PresentRelated↔Default exchanges move the answer; the deleted helpers leave no behaviour behind; stopping rules unchanged.

```findings
[
  {"file": "crates/verify/ess-conformance/src/interpret/execute.rs", "line": 531, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "refused_by_input answers step 2 before select regardless of the plan's order, so exchanging InputRefusal with HeldState (or any phase select reads) does not move the answer"},
  {"file": "crates/verify/ess-conformance/src/interpret/execute.rs", "line": 613, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "unknown_instance, placed in the plan's Existence phase, is answered by the held-subject loop after refused_by_input, so ordering Existence before InputRefusal does not move it"}
]
```
