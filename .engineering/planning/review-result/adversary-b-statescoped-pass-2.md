---
format: aep.planning-md/3
id: review-result:adversary-b-statescoped-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit state-scoped
relations:
- reviews: story:wrong-state-scoped-to-subject-states
- reviews: story:state-inside-a-when-subject-predicate
revision: 1
---
unit: state-scoped (beyond10x/ess#201, #204), uncommitted working tree on base 1675910146 in ess-b-statescoped (correction 1 applied)
verdict: NEEDS-CHANGE
cases: executed 108→118, red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (~/.cache/ess-wave-n2/statescoped/adv2/{review.md,red-conformance.log,red-runtime.log,suite.log}); build dir ~/.cache/b10x-target/ess-b-statescoped (assigned)
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat`: `24 files changed, 990 insertions(+), 105 deletions(-)`, unchanged from the tree as handed over. No tracked file touched. New untracked files, both tests:

- crates/verify/ess-conformance/tests/adversary_state_scoped_pass2.rs
- crates/generate/ess-entity-runtime/tests/adversary_state_scoped_pass2.rs

## 2. Cases added (each run alone before the suite)

| case | asserts | now |
|---|---|---|
| conformance `a_state_inequality_refusal_is_witnessed_in_every_state_it_answers` | `gone: when_subject: {predicate: state != Placed}` + error, no `wrong_state:`. Synthesis is clean, the faithful target passes. Targets that accept in `Shipped`, in `Delivered`, and in `Cancelled` must each fail a ShipOrder scenario | red |
| conformance `a_negated_state_refusal_is_witnessed_in_every_state_it_answers` | same with `not: state == Placed` | red |
| conformance `state_guards_split_by_the_input_covering_every_wrong_state_synthesize` | two refusals `state != Placed` with `when: reason == Late` / `when: reason == Lost`, no `wrong_state:`. No synthesis refusals | red |
| conformance `a_state_or_input_guard_beside_wrong_state_synthesizes` | `any: [state == Cancelled, input.reason == Lost]` refusal beside `wrong_state:`: no refusals | green |
| conformance `a_target_answering_the_wrong_refusal_of_one_state_fails` | F2 shape. Three mutants are killed: always `gone`, `gone`/`gone-quietly` swapped in `Cancelled` only, and accepting `Delivered`+`Lost` | green |
| conformance `split_refusals_synthesize_deterministically` | two syntheses of the F2 shape are equal | green |
| conformance `a_listed_accepting_guard_of_one_state_or_every_wrong_state_synthesizes` | `[Shipped]`, and `[Shipped, Delivered, Cancelled]` on `preserves:`: no refusals | green |
| runtime `a_state_inequality_refusal_decides_every_state_at_runtime` | `all: [state != Draft, state != Issued]` refusal, no `wrong_state:`. Paid and Cancelled refuse `settled` on both channels; Draft and Issued take `cancelled` | green |
| runtime `listed_refusals_split_by_the_input_decide_per_input_at_runtime` | `[Paid, Cancelled]` + `via == Post` / `via != Post`: each input takes its own refusal | green |
| runtime `a_state_or_field_refusal_beside_wrong_state_decides_at_runtime` | `any: [state == Cancelled, channel == Post]` beside `wrong_state:`: guarded first (#192), then `wrong-state`, then `cancelled` | green |

Two fixture mistakes were fixed before any finding was recorded. A refusal with `preserves:` is refused by the model (`refusal_mutated_state`), so the "subject refusal beside a subjectless one" shape cannot be written, and that case was dropped. `Channel` has three variants, so `via == Email` became `via != Post`.

Red output, verbatim:

```
thread 'a_state_inequality_refusal_is_witnessed_in_every_state_it_answers' (2410746) panicked at crates/verify/ess-conformance/tests/adversary_state_scoped_pass2.rs:316:5:
mutants passing every ShipOrder scenario: ["accepts_in_shipped", "accepts_in_delivered"]; scenarios: [
    "demo.ship.Order/transition/ship/by/demo.ship.ShipOrder/shipped",
    "demo.ship.ShipOrder/outcome/gone",
    "demo.ship.ShipOrder/outcome/shipped",
]
```
```
thread 'a_negated_state_refusal_is_witnessed_in_every_state_it_answers' (2410852) panicked at crates/verify/ess-conformance/tests/adversary_state_scoped_pass2.rs:344:5:
mutants passing every ShipOrder scenario: ["accepts_in_shipped", "accepts_in_delivered"]; scenarios: [
    "demo.ship.Order/transition/ship/by/demo.ship.ShipOrder/shipped",
    "demo.ship.ShipOrder/outcome/gone",
    "demo.ship.ShipOrder/outcome/shipped",
]
```
```
thread 'state_guards_split_by_the_input_covering_every_wrong_state_synthesize' (2410908) panicked at crates/verify/ess-conformance/tests/adversary_state_scoped_pass2.rs:369:5:
assertion `left == right` failed
  left: ["ESS-SYNTH-003: no candidate of the 2 tried satisfies `none of: reason == Late, reason == Lost, on a `demo.ship.Order` row in this state refuting every one of: state != Placed, state != Placed`", (same, twice more)]
 right: []
```

## 3. Suite run (after the cases existed)

```
== cargo test -p ess-domain --test state_scoped_refusals --test stored_field_guards --test adversary_state_scoped_pass1 --no-fail-fast
adversary_state_scoped_pass1: test result: ok. 2 passed; 0 failed
state_scoped_refusals:        test result: ok. 18 passed; 0 failed
stored_field_guards:          test result: ok. 18 passed; 0 failed
exit=0
== cargo test -p ess-conformance --test state_scoped_refusals --test mixed_guard_wrong_state --test adversary_mixed_guard_pass1 --test adversary_state_scoped_pass1 --test adversary_state_scoped_pass2 --no-fail-fast
adversary_mixed_guard_pass1:  test result: ok. 7 passed; 0 failed
adversary_state_scoped_pass1: test result: ok. 5 passed; 0 failed
adversary_state_scoped_pass2: test result: FAILED. 4 passed; 3 failed
mixed_guard_wrong_state:      test result: ok. 8 passed; 0 failed
state_scoped_refusals:        test result: ok. 11 passed; 0 failed
exit=101
== cargo test -p ess-entity-runtime --test lowering --test adversary_state_scoped_pass1 --test adversary_state_scoped_pass2 --no-fail-fast
adversary_state_scoped_pass1: test result: ok. 1 passed; 0 failed
adversary_state_scoped_pass2: test result: ok. 3 passed; 0 failed
lowering:                     test result: ok. 38 passed; 0 failed
exit=0
```

Counts are from this run. Without the two pass-2 files these targets ran 108 cases (2+18+18+7+5+8+11+1+38). With them they ran 118. All pass-1 cases are now green.

## 4. Findings

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| A | crates/verify/ess-conformance/src/synthesize.rs:6344 | NEEDS-CHANGE | introduced | The F3 skip removes every wrong state that a `state` guard answers from the wrong-state family. The comment says "that branch's own scenario witnesses it", but that scenario witnesses one state (`Cancelled`). With `state != Placed` (or `not: state == Placed`), a target that accepts in `Shipped` or in `Delivered` passes every ShipOrder scenario. The suite has 3 ShipOrder ids and no per-state witness. Before the correction, the same model was refused with ESS-SYNTH-003 (loud). Now it is under-witnessed (silent). The listed spelling `[Shipped, Delivered, Cancelled]` is witnessed per state; the #204 spelling of the same model is not. `docs/design/cross-record-and-stored-field-guards.md` ("the wrong-state family writes no scenario for it") has the same gap. | The issue's own fourth spelling (`when_subject: {predicate: state != Shipped}` on a refusal), valid under `ess/18` with no `wrong_state:` branch. The `any:` disjunction spelling is covered by boundary rows (pass 1), so only non-disjunctive forms are affected: `!=`, `not`, and conjunctions of `!=`. |
| B | crates/verify/ess-conformance/src/synthesize/subject_fact.rs:276 | NEEDS-CHANGE | introduced | `state_answers` asks one branch at a time and excludes any branch with an input half. Two `state != Placed` refusals split by `when: reason == Late` / `== Lost` together answer every wrong state for every input, and synthesis still refuses the valid model with ESS-SYNTH-003 ×3. This is the #204 spelling of the F2 shape that the #201 spelling now handles. | `ess/18`: `state`-reading refusals split by the input, with no `wrong_state:` branch. There is a workaround: the `when_subject_state:` list spelling synthesizes. |

Suggested fixes (not applied):
- A: witness each wrong state that the F3 skip removes. For example, keep the `<entity>/state/<S>/refuses/<command>` scenario and arrange it through the guarded branch, as `state_refusals` does for listed refusals. Or, for every state the predicate is true in with `state` alone, give the branch a further row, as `Witness::Listed` does.
- B: decide coverage over the union of the guarded branches: a wrong state is answered when no input and no row in it falls through to the family. For example, treat `GuardUnsatisfiable` from the family's search as "no wrong-state row", as `state_refusals` does.

## 5. Attacked and could not break

- `state_answers` with `or` mixing `state` and an input (`any: [state == Cancelled, input.reason == Lost]`) beside `wrong_state:`: synthesizes clean.
- F2 mutants: always the first refusal, swapped refusals in one state only, and accepting one state+input cell are all killed. `ExpectOutcome` pins the branch.
- F2 determinism: two syntheses are equal.
- `Listed` of length 1 and of every non-move state on `preserves:`: clean. A listed state that no move reaches cannot be written: the domain refuses unreachable states (`entity.rs:595`).
- A subject-bearing refusal beside a subjectless one cannot be written: `refusal_mutated_state`.
- Entity Runtime parity: the conjunction of `!=` with no `wrong_state:`, listed refusals split by the input, and `or` of `state` with a stored field beside `wrong_state:` all select as ESS does.
- F4: the schema now carries `minItems: 1` and `uniqueItems: true`, and the pass-1 case is green. I did not re-derive the `8bfa604c…` shape hash; it comes from the implementor's `cargo xtask` gate.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/statescoped/adv2/review.md
- ~/.cache/ess-wave-n2/statescoped/adv2/red-conformance.log
- ~/.cache/ess-wave-n2/statescoped/adv2/red-runtime.log
- ~/.cache/ess-wave-n2/statescoped/adv2/suite.log
- build output under ~/.cache/b10x-target/ess-b-statescoped (the assigned build dir)

## 7. Findings block

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 6344, "category": "mutant", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "wrong states skipped because a non-disjunctive state guard (state != X, not) answers them are witnessed in only one state, so a target accepting in any other skipped state passes every scenario"},
  {"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 276, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "state-reading refusals that cover a wrong state only together (split by the input) are not recognised by state_answers, and synthesis refuses the valid model with ESS-SYNTH-003"}
]
```
