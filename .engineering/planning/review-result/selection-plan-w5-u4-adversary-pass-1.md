---
format: aep.planning-md/3
id: review-result:selection-plan-w5-u4-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 5 unit U4 (related_guard reads the plan)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: wave 5 U4 story:synthesis-reads-selection-plan; uncommitted tree on base 3f0e8e1539
verdict: CONFIRMED (red cases; none shown introduced by the diff)
cases: executed 72→87, red 6
origin: introduced 2 / pre-existing 1 / undecided 3
wrote-outside-worktree: ~/.cache/ess-selection-plan/w5-u4-scratch/adversary-1/ (8 logs)
needs-coordinator: run a4b and a2 against base 3f0e8e1539
```

Coordinator base run: the same file against the integration tree's `related_guard.rs` (unchanged since 3f0e8e1539) fails the same 6 cases (a2, a2c, a3 stored reference, a4, a4b, a4c), so F1, F2 and F4 are pre-existing. F3 removed and F6 recorded in the CHANGELOG in a correction round the coordinator verified (own test 6, adversary green file 9, open file 6 ignored and failing under `--ignored`, bytes table 2). The 6 red cases are committed as `#[ignore]` in `adversary_selection_plan_w5_u4_pass1_open.rs`; F1 is `story:related-guard-wrong-state-beside-external`.

Not broken: `answering_refusal` vs the old precedence block on every validating composition; `orders_present_related_refusal` under the real order; 7 exchanged orders on five models apart from F4; the `rfind` mutant is caught.

```findings
[
  {"file": "crates/verify/ess-conformance/tests/adversary_selection_plan_w5_u4_pass1.rs", "line": 536, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "On a validating ess/22 command with when_related, wrong_state and an external branch, the synthesized wrong-state scenario arranges no pick and no list, and the interpreter answers no-list."},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 462, "category": "property", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "Several-rows searches ask selects with a one-row projection whose plan reads the predicate refusal among the accepting branches instead of at step 5, so no scenario proves the step-5 order."},
  {"file": "crates/verify/ess-conformance/tests/synthesis_related_guard_plan.rs", "line": 214, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "The exact-message tolerance for no-candidate matches no failure in this tree, so it only hides a future regression."},
  {"file": "crates/verify/ess-conformance/tests/adversary_selection_plan_w5_u4_pass1.rs", "line": 297, "category": "property", "severity": "note", "verdict": "INFEASIBLE", "origin": "undecided", "message": "With input_refusal and present_related exchanged on a stored reference, synthesis claims blocked while the interpreter answers completed."},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 978, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "orders_present_related_refusal returns true for a stored reference or several rows with no related refusal declared, an arrangement rule rather than an order the plan makes."},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 314, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "leaves_external changes the external witness on a validating ess/20 model (desk_stale_after) without a declared byte exception, although the interpreter accepts both witnesses."}
]
```
