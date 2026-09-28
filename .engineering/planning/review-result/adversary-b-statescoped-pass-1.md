---
format: aep.planning-md/3
id: review-result:adversary-b-statescoped-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit state-scoped
relations:
- reviews: story:wrong-state-scoped-to-subject-states
- reviews: story:state-inside-a-when-subject-predicate
revision: 1
---
unit: state-scoped (beyond10x/ess#201, #204), uncommitted working tree on base 1675910146 in ess-b-statescoped
verdict: NEEDS-CHANGE
cases: executed 99→107, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (~/.cache/ess-wave-n2/statescoped/adv1/{review.md,red-conformance.log,probe-ir.log,suite.log,suite-domain.log}); build dir ~/.cache/b10x-target/ess-b-statescoped (assigned)
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat`: `24 files changed, 821 insertions(+), 73 deletions(-)`. That matches the tree as it was handed over, so no tracked file changed. New untracked files, all tests:

- crates/specify/ess-domain/tests/adversary_state_scoped_pass1.rs
- crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs
- crates/generate/ess-entity-runtime/tests/adversary_state_scoped_pass1.rs

## 2. Cases added (red output captured by running each case alone)

| case | asserts | now |
|---|---|---|
| conformance `a_listed_accepting_guard_is_witnessed_in_every_state_it_lists` | `already-shipped: when_subject_state: [Shipped, Delivered]` (preserves). The faithful target passes; a target that refuses in `Shipped`, and one that refuses in `Delivered`, each fail a ShipOrder scenario | red |
| conformance `two_refusals_of_one_held_state_split_by_the_input_both_synthesize` | two subjectless listed refusals, split by an enum input `when:`, produce no synthesis refusals | red |
| conformance `state_guards_covering_every_wrong_state_leave_no_wrong_state_refusal` | #204 spelling: `any: [state == Shipped, state == Delivered]` preserves, and `state == Cancelled` refusal; no `wrong_state:` branch; no synthesis refusals | red |
| domain `the_published_schema_refuses_what_the_model_refuses` | the list branch of `HeldStates` in the generated schema carries `minItems: 1` and `uniqueItems: true` | red |
| conformance `a_state_disjunction_in_when_subject_is_witnessed_in_every_state_it_names` | the same mutants are killed under the #204 spelling | green |
| conformance `a_listed_guard_compiles_to_one_ir_whatever_order_it_is_written_in` | `[Delivered, Cancelled]` and `[Cancelled, Delivered]` give identical canonical IR; a scalar stays a string | green |
| domain `json_sources_are_gated_like_yaml_ones` | JSON-form sources at ess/17 are refused `unsupported_format_version` for the list form and for `state` in `when_subject` | green |
| runtime `a_state_reading_refusal_in_a_wrong_state_is_selected_before_wrong_state_at_runtime` | lowered CancelInvoice: Paid/Post takes `posted-paid`, Paid/Email and Cancelled/Post take `wrong-state`, Issued/Post takes `cancelled` | green |

The first runs of the input-split case failed on my own fixture mistakes (`Bool`, then a Boolean guard the partition does not decide). I fixed the fixture before recording the red run below.

Red output, verbatim:

```
thread 'a_listed_accepting_guard_is_witnessed_in_every_state_it_lists' panicked at crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs:268:9:
misses_shipped passes every ShipOrder scenario: a state `when_subject_state: [Shipped, Delivered]` lists is never arranged for `already-shipped`
```
```
thread 'two_refusals_of_one_held_state_split_by_the_input_both_synthesize' panicked at crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs:294:5:
  left: ["ESS-SYNTH-007: a second scenario claimed this id (Refusal { ... state: StateName(Cancelled), command: CommandRef(QualifiedName(demo.ship.ShipOrder)), refuses: true }), cause: DuplicateScenario })", "ESS-SYNTH-007: a second scenario claimed this id (... state: StateName(Delivered) ... DuplicateScenario })"]
 right: []
```
```
thread 'a_state_disjunction_in_when_subject_is_witnessed_in_every_state_it_names' panicked at crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs:310:5:
  left: ["ESS-SYNTH-003: no candidate of the 1 tried satisfies `a `demo.ship.Order` row in this state refuting every one of: (state == Shipped or state == Delivered), state == Cancelled` (... state: StateName(Cancelled) ...)", "... StateName(Delivered) ...", "... StateName(Shipped) ..."]
 right: []
```
(The refusal assertion was later split out into `state_guards_covering_every_wrong_state_leave_no_wrong_state_refusal`. The output is the same.)
```
thread 'the_published_schema_refuses_what_the_model_refuses' panicked at crates/specify/ess-domain/tests/adversary_state_scoped_pass1.rs:69:5:
assertion `left == right` failed: `when_subject_state: []` passes the schema and is refused by the model: {"anyOf":[{"$ref":"#/definitions/StateName"},{"items":{"$ref":"#/definitions/StateName"},"type":"array"}],"description":"One held lifecycle state, or a nonempty list of them (ess/18)."}
  left: Null
 right: Number(1)
```

## 3. Suite run (after the cases existed)

```
== cargo test -p ess-domain --test state_scoped_refusals --test stored_field_guards --test adversary_state_scoped_pass1 --no-fail-fast
adversary_state_scoped_pass1: test result: FAILED. 1 passed; 1 failed
state_scoped_refusals:        test result: ok. 18 passed; 0 failed
stored_field_guards:          test result: ok. 18 passed; 0 failed
exit=101
== cargo test -p ess-conformance --test state_scoped_refusals --test mixed_guard_wrong_state --test adversary_mixed_guard_pass1 --test adversary_state_scoped_pass1 --no-fail-fast
adversary_mixed_guard_pass1:  test result: ok. 7 passed; 0 failed
adversary_state_scoped_pass1: test result: FAILED. 2 passed; 3 failed
mixed_guard_wrong_state:      test result: ok. 8 passed; 0 failed
state_scoped_refusals:        test result: ok. 10 passed; 0 failed
exit=101
== cargo test -p ess-entity-runtime --test lowering --test adversary_state_scoped_pass1 --no-fail-fast
adversary_state_scoped_pass1: test result: ok. 1 passed; 0 failed
lowering:                     test result: ok. 38 passed; 0 failed
exit=0
```

How the counts were read: without my files these targets ran 99 cases (18+18+7+8+10+38). With them, 107. There was no implementor `cases:` line to take the first number from.

## 4. Findings

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| F1 | crates/verify/ess-conformance/src/synthesize.rs:3676 | NEEDS-CHANGE | introduced | `reach` witnesses a listed guard only in the first state that can be reached. A target that refuses in `Shipped` passes every ShipOrder scenario when `already-shipped` lists `[Shipped, Delivered]`. The split spelling witnesses both states. | Any `ess/18` document that puts a list on an accepting branch (`preserves`/`updates`/`moves`). The domain admits it (`a_listed_state_on_a_branch_with_a_subject_needs_its_move_to_start_in_each`). |
| F2 | crates/verify/ess-conformance/src/synthesize.rs:2270 | NEEDS-CHANGE | introduced | The `<entity>/state/<S>/refuses/<command>` id is keyed by state only. Two subjectless held-state refusals of one state, split by the input, collide and give ESS-SYNTH-007 on a valid model. | `ess/18`: `when_subject_state:` plus `when:` on subjectless refusals. At base a command could have at most one subjectless (default) refusal. |
| F3 | crates/verify/ess-conformance/src/synthesize.rs:6281 | NEEDS-CHANGE | introduced | The wrong-state family still tries every state in `ir.wrong_states` and cannot find a row that refutes the `state`-reading guards. It refuses with ESS-SYNTH-003 on a valid model. The `selects` change (subject_fact.rs:495) covers the branch scenarios but not this family. | The issue's own fourth spelling (`when_subject: {predicate: state != Shipped}` on a refusal): any `ess/18` command whose `state` guards answer every wrong state and which has no `wrong_state:` branch. |
| F4 | crates/specify/ess-domain/src/entity.rs:239 | CONFIRMED | introduced | The `HeldStates` schema's array branch has no `minItems: 1` and no `uniqueItems: true`. `[]` and `[A, A]` pass the schema and are refused by the model. Its own description says "nonempty". | Editor or CI validation against `schemas/generated/ess.schema.json`. |

Suggested fixes (not applied):
- F1: witness each listed state, one outcome scenario per state or per-state `state/<S>/…` ids, as refusals already get.
- F2: put the branch in the id when more than one refusal claims a state, or refuse the shape at validation.
- F3: skip a wrong state that the `state`-reading guards cover completely, treating `GuardUnsatisfiable` there as "no wrong-state row", as `state_refusals` does.
- F4: emit `minItems: 1` and `uniqueItems: true`.

## 5. What I attacked and could not break

- Format gate below ess/18 blocks all three new spellings: list, subjectless scalar refusal, `state` in `when_subject`. Checked in YAML and JSON.
- The list is kept in name order. Written order does not change the canonical IR, and a scalar keeps its string bytes.
- Empty and repeated lists are refused (unit tests). `when_subject_state` plus `when_subject` on one branch is still refused.
- Entity Runtime parity for #192 precedence: a `state`-reading refusal in a wrong state is selected before `wrong_state`. `$from_state` lowering is correct.
- #204 disjunction mutants: both "misses one disjunct" targets are killed by the boundary scenarios.
- Changed expectation in `stored_field_guards.rs::it_reads_the_entity_fields_and_nothing_else`: it still asserts a refusal at the same key, now `unsupported_format_version` naming ess/18. It is not weakened.
- Go/TS runners: they read scenarios, and the scenario vocabulary and ids are unchanged, so nothing new reaches them.
- The ess-gen, ess-diff and openapi renderings of `HeldStates` use `Display` ("Cancelled or Delivered"). No drift found.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/statescoped/adv1/review.md
- ~/.cache/ess-wave-n2/statescoped/adv1/red-conformance.log
- ~/.cache/ess-wave-n2/statescoped/adv1/probe-ir.log
- ~/.cache/ess-wave-n2/statescoped/adv1/suite.log
- ~/.cache/ess-wave-n2/statescoped/adv1/suite-domain.log
- build output under ~/.cache/b10x-target/ess-b-statescoped (the assigned build dir)

## 7. Findings block

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 3676, "category": "mutant", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a listed held-state guard on an accepting branch is witnessed only in its first reachable state, so a target that mishandles any other listed state passes the suite"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 2270, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "two subjectless held-state refusals of one state split by the input collide on the per-state refusal id and synthesis refuses a valid model with ESS-SYNTH-007"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 6281, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the wrong-state family refuses with ESS-SYNTH-003 for every wrong state that state-reading when_subject guards fully answer, so the #204 spelling of the #201 shape does not synthesize"},
  {"file": "crates/specify/ess-domain/src/entity.rs", "line": 239, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the generated HeldStates schema admits an empty or repeated state list that the model refuses, despite describing the list as nonempty"}
]
```
