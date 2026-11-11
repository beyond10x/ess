---
format: aep.planning-md/3
id: review-result:zi-t2-lane5-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row
relations:
- reviews: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row
revision: 1
---
```
unit: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row (HEAD 1ddccb0f83, tree zi-t2-12, plus one untracked test file)
verdict: weak
new failing cases: none
mutants survived: 1 (`creators_leave_unset` always returning false passes every test in the crate; found by reading the tests, not by running it; my new tests now catch it)
cases: executed 8→18, red 0
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 7 paths (part 6)
needs-coordinator: none
```

The change holds against everything I tried. "Weak" refers only to one help-text branch that nothing tested.

**1. Diff.** `git diff --stat` is empty. `git status --short` shows only the new test file, so no implementation file was touched:
```
?? crates/verify/ess-conformance/tests/adv_existing_instance_own_run.rs
```

**2. Cases added.** All 10 are in `~/.local/state/worktree/trees/b10x/ess/zi-t2-12/crates/verify/ess-conformance/tests/adv_existing_instance_own_run.rs`. All 10 are green now, so there is no red output to quote. The first run (`adv/run1.txt`, exit 101) failed to compile because of a type mismatch in my file; I fixed it, and the second run (`adv/run2.txt`) gave `10 passed; 0 failed`, `exit 0`.
- Seven tests check that the existence scenario is built with no refusal and that every scenario passes on the interpreter:
  - the related row is two state changes away (Current to Reviewed to Stale);
  - the state change it needs is itself guarded on a row of another entity, so two prior rows of two entities;
  - two commands create the related row, one of them setting `item` from a literal;
  - the command has two creating branches (one with `when: priority > 5`);
  - the story's second-adopter shape (a booking whose schedule must be `Published`);
  - the same with the schedule owning its bookings (`kind: owns`);
  - an external refusal answered before the row (`before_row: true`) beside the guard, which builds a segment through `own_run`.
- Three tests check the help text:
  - the creator sets the field from a literal: the help asks for `sets:`;
  - one of two creators sets it from a literal: the help asks;
  - the creator sets it from an input with a different name: the help does not ask.
- I am leaving all ten in place:
  - The first two help tests are the only tests that pin the `unset: true` branch at `synthesize.rs:3111`. `grep` finds no other test asserting that help text.
  - The seven scenario tests pin shapes the unit's own tests do not cover.

**3. Suite run, after the cases existed.** All six targets below passed, `exit 0`, `slot A waited 50s ran 422s`, output in `adv/suite.txt`. Command:
`build.sh adv/suite.txt <tree> test -p ess-conformance --locked --test existing_instance_beside_related_guards --test adv_existing_instance_own_run --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test adversary_related_values_all_bytes --test external_beside_held_guard`

| Target | Result |
|---|---|
| adv_existing_instance_own_run | 10 passed |
| existing_instance_beside_related_guards | 8 passed |
| synthesis_suite_bytes_table | 2 passed |
| synthesis_suite_bytes_table_cli_read | 1 passed |
| adversary_related_values_all_bytes | 1 passed |
| external_beside_held_guard | 17 passed |

The four byte pins are unchanged.

**4. Findings (none of them a blocker).**
- **`existence.rs:1897` (mutant, warning).** Nothing in the crate checks the `sets: <field>: input.<field>` help, so `creators_leave_unset` could always return false and every test would still pass. Found by `grep`; I did not run the mutant.
  - What reaches it: any refused existence scenario where a creator of the related row sets a compared field from a literal.
  - Now caught by `help_asks_for_sets_where_the_creator_sets_a_literal` and `help_asks_for_sets_where_one_of_two_creators_sets_a_literal`.
- **`existence.rs:1805` and `:1857` (judgement, note).** When `segment` runs without row-set rows, `own_run` makes the same `prepare_at_in` call, at the same distinction, that `creating_input` already made successfully.
  - So `own_run` cannot fail there, which makes the `Unsteered` refusal and the `created_state` change inside `stored_first` effectively unreachable.
  - Consequence: reverting line 1805 to `lifecycle.initial` would pass every test. The unit's `PICK` test exercises only the `creating_input` path, at line 2108.
  - What reaches it: only a command that mixes a row-set guard and a via guard, and I did not construct one.

**5. Attacked and could not break.**
- `own_run` scenarios failing on the interpreter: tried with several prior rows, two creators of the related row, two creating branches, a self-related entity (the unit's `CONFIRM` and my two-branch variant), the owned case, and the external before-row segment.
- The `unset` help when the creator sets the field from a literal or from an input with another name: correct in both.
- Dropping the `own_run` fallback, or reversing the order so the related-row setup comes after the creation: both are caught by `existence_reuses_the_creating_branchs_own_run`.
- The four byte pins: unchanged.
- Not tried: the `unset` help when the creator computes the field from its input, `created_state` with several `into:` creations, and whether `own_run` runs before the old search (that would show up only as changed bytes).

**6. Paths written outside the worktree.** All in `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row/adv/`: `external.rs.part`, `run1.txt`, `run1.txt.touch`, `run2.txt`, `run2.txt.touch`, `suite.txt`, `suite.txt.touch`.

I used 3 of the 4 cargo runs. My lease (`zi-t2-adv-s5`) is released.

**7. Findings block.**
```findings
- file: crates/verify/ess-conformance/src/synthesize/existence.rs
  line: 1897
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "no test asserted the unset-true help arm, so creators_leave_unset returning false always survived the crate's tests; now pinned by two cases in adv_existing_instance_own_run.rs"
- file: crates/verify/ess-conformance/src/synthesize/existence.rs
  line: 1805
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "own_run repeats the prepare_at_in call creating_input already passed at the same distinction, so Unsteered and the stored_first created_state change are effectively unreachable and a revert of line 1805 survives every test"
```
