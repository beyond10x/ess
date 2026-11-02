---
format: aep.planning-md/3
id: review-result:adversary-d368-u1-pass-1
kind: review-result
status: active
title: 'D368 U1 adversary pass 1: an optional element appended as null'
relations:
- reviews: story:an-optional-element-is-appended-or-refused-consistently
revision: 1
---
unit: D368 U1 (story:an-optional-element-is-appended-or-refused-consistently), 295d0b617 on unit/ess-d368-u1, reviewed against 86d3d194d in tree ess-d368-u1-adv at 7e0d2e880
verdict: APPROVE against the Acceptance (no in-scope finding); 3 out-of-scope rows
cases: executed 0→11 (review file alone; the suite run was refused by the free-space precheck at 09:02:17Z), red 2
origin: introduced 0 / pre-existing 1 / undecided 1
wrote-outside-worktree: the wave scratch directory u1/adversary-scratch (cargo log only)
needs-coordinator: none

- **Cases.** `tests/adversary_d368_u1_pass1.rs`, uncommitted review file. Nine were green:
  - a no-mutation control;
  - the suite fails a target that drops the null, and one that appends the text "null";
  - `Optional<List<Optional<String>>>` held absent reads `[null]`, then `[null, "b"]`;
  - an element read through an absent or null Optional struct appends null;
  - a Bool, list or map element is refused and the row is unchanged;
  - a required String element over `List<Optional<String>>` is refused by the domain.
- **Two red cases**, both outside the Acceptance:
  - a target that always appends null passes the synthesized suite;
  - `Optional<Optional<String>>` is admitted as an element.
- **Out-of-scope rows, decided at wave close:**
  - the suite never sends a present Optional element;
  - doubly Optional elements are admitted, so null cannot be told from absent;
  - the ess/31 history sentence at system.rs:335 still says an Optional read is refused.
- **Attacked, not broken:**
  - the null append when the outer list is Optional;
  - an element read through an input path;
  - ill-typed elements;
  - `required()` recursion is bounded by MAX_TYPE_DEPTH 32.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":10571,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"out-of-scope: the suite for an append over List<Optional<T>> never sends a present element, so a target that always appends null passes it (red: a_target_ignoring_a_present_optional_element_fails_the_synthesized_suite)"},
{"file":"crates/specify/ess-domain/src/types.rs","line":163,"category":"judgement","severity":"note","verdict":"INFEASIBLE","origin":"pre-existing","message":"out-of-scope: the domain admits List<Optional<Optional<T>>>, where the interpreter's null cannot be told from absent; no model is known to write it"},
{"file":"crates/specify/ess-domain/src/system.rs","line":335,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"out-of-scope: the ess/31 history still says an Optional read is refused as type_mismatch, though over List<Optional<T>> it is admitted and now appends null"}
]
```
