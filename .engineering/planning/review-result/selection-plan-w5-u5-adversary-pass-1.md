---
format: aep.planning-md/3
id: review-result:selection-plan-w5-u5-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 5 unit U5 (row_set reads the plan)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: story:synthesis-reads-selection-plan U5, uncommitted tree on base 3f0e8e1539
verdict: INFEASIBLE (1 red case, reached only through the with_phase_order test seam); no blocker found
cases: executed 42→47, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: ~/.cache/ess-selection-plan/w5-u5-scratch/adversary-1/ (logs only)
needs-coordinator: mutant and base comparisons not run (DEC-20261008-29, / under 40G)
```

Cases in `adversary_selection_plan_w5_u5_pass1.rs`: a1 the changed shape (a compensating `external:` refusal after an accepting `when:` on a row-set command with `unknown_instance:`) passes the interpreter; a2 three exchanges over four models; a3 a history under PresentRelated↔Accepting is Linearizable; a4 `Gate` scenarios under four orders; a5 under Accepting↔Default the `fast` and `claimed` witnesses fail the interpreter (red).

Not broken: `exists: false` cannot sit on a row-set command (validation); no row-set phase depends on the format; `consistent`'s declaration-order read hides no order decision under the real order.

Coordinator correction: `consistent` refuses a witness when an unrefutable member (`otherwise:`) answers first; a5 green, coordinator ran the adversary file (5 passed), clippy and fmt (exit 0).

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize/row_set.rs", "line": 1, "category": "property", "severity": "note", "verdict": "INFEASIBLE", "origin": "undecided", "message": "input_for and consistent drop plan members before the expected branch that are neither a when: nor a row set, so an otherwise: read first (Accepting<->Default via with_phase_order only) is neither refuted nor made a refusal, and the fast and claimed witnesses fail the interpreter"}
]
```
