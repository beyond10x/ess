---
format: aep.planning-md/3
id: review-result:zi-w1-lane3-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
relations:
- reviews: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
revision: 2
---
unit: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed (beyond10x/ess#727), tree zi-t1-adv at 763faf488b
verdict: broken
new failing cases: 3 (adversary_t1_lane3_narrowed_input_relation.rs::adv_an_identity_input_is_not_read_as_its_instance_name, ::adv_acceptance5_only_reader_list_form_catches_a_target_ignoring_the_subject_guard, ::adv_scalar_form_with_a_generated_uuid_identity_catches_the_ignoring_target)
mutants survived: 2 (a target that ignores only `when_subject:` passes the whole list-form "only reader" suite; it also passes the whole scalar suite when the identity is a generated `Uuid`)

cases: executed 31→36, red 3. The 31 is the sum of the unit's targets: 5 + 7 + 17 + 2, the same tests as part 3 without my file.
wrote-outside-worktree: 2 (`adv/r1.txt` and `adv/r2.txt` under the story's scratch directory)
needs-coordinator: yes. The blocker below goes back to the implementor.

1. **Changed files:** the tracked diff is empty. One untracked test file is left uncommitted: `crates/verify/ess-conformance/tests/adversary_t1_lane3_narrowed_input_relation.rs`. Nothing outside tests changed.

2. **My test file run alone:** 2 passed, 3 failed, `exit 101`, slot B ran 60s. Output: `adv/r1.txt`.
   - **Blocker, introduced:** `adv_an_identity_input_is_not_read_as_its_instance_name` fails at line 297. In this test the stored field `origin` holds `input.id`, and the guard is `origin == input.id`. That guard is true on every row a run can hold.
     - Synthesis now reads `id` as the scenario's instance name, `"rollout"`, but the target holds `"id"`. It decides the guard is false and forces `refused` in the `admitted` scenario.
     - The interpreted target correctly answers `refused`. Two scenarios fail: `DecideAdmission/outcome/admitted` and `Rollout/transition/admit/by/DecideAdmission/admitted`.
     - Before this commit, an instance input was dropped and the guard stayed undecided, so no force was added.
     - The cause is `synthesize.rs:4387`. Acceptance 3 requires the interpreted target to pass every scenario, so this breaks a guarantee the acceptance relies on.
     - Suggested fix: leave instance fields unbound, as `input::replay_facts` does (`Completeness::Partial`). Reading the instance name as text is wrong. Even the `instance:<name>` form that `token()` uses would still be wrong here.
     - Reachability: I built this model, and storing a row's own identity is unusual. But any narrowed guard that reads `input.<identity>` gets this wrong verdict.
   - **Blocker by the acceptance text, already disclosed:** `adv_acceptance5_only_reader_list_form_...` fails at line 265. Acceptance 5 names the suite of 3, which includes the list form, and a target ignoring only the subject guard passes all of it. No scenario forces `refused` with a non-empty `missing` the row does not require.
   - **Backlog, beside acceptance:** `adv_scalar_form_with_a_generated_uuid_identity_...` fails at line 265. With the common identity shape, a generated `Uuid`, the instance name is not a valid `Uuid`. The input cannot be read, the guard stays undecided, and no scenario forces the branch. So the ignoring target passes the scalar suite here. The fix above would cover this too.
   - **Green, kept:** `adv_acceptance5_with_sibling_catches_a_target_keeping_its_own_when`. The unit's mutant also drops the branch's own `when:`; this test confirms the suite of 1 still catches the narrower mutant that keeps it.
   - **Green, kept:** `adv_a_quantifier_over_the_stored_list_is_witnessed`.

3. **Suite after my file existed:** I ran the unit's three targets, `synthesis_suite_bytes_table` and my file in one run with `--no-fail-fast`: `exit 101`, slot B ran 60s. Output: `adv/r2.txt`.

| target | result |
|---|---|
| external_beside_held_guard | ok, 17 passed |
| external_narrowed_by_input_relation | ok, 5 passed |
| external_narrowed_by_subject | ok, 7 passed |
| synthesis_suite_bytes_table | ok, 2 passed |
| adversary_t1_lane3_narrowed_input_relation | FAILED, 2 passed, 3 failed |

4. **Judgement:** the code comment at `synthesize.rs:4385` says the instance input "is no value a guard compares". The `origin == input.id` case shows a guard can compare it.

5. **Attacked and could not break:**
   - The narrowed `refused` scenarios in all five of my models were witnessed on rows and inputs where the guard holds; the interpreted target passed every one.
   - The commands that are narrowed and also use subject guards (the version with a sibling) held.
   - Byte pins: the only repository model with a narrowed external branch is `external-narrowed-by-subject.yaml`, and its guards do not read the input. No pin should have moved, and the table is green.

6. **Files outside the worktree:** `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed/adv/r1.txt` and `.../adv/r2.txt`.

7. Findings:
```findings
[
  {
    "file": "crates/verify/ess-conformance/src/synthesize.rs",
    "line": 4387,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "an instance input is read as its instance name, so a narrowed guard that reads the identity input is judged false where the target finds it true, and the sibling's scenario forces a branch a correct target takes"
  },
  {
    "file": "crates/verify/ess-conformance/tests/adversary_t1_lane3_narrowed_input_relation.rs",
    "line": 265,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "acceptance 5 fails for the list-form only-reader suite of acceptance 3; a target ignoring only the subject guard passes all of it"
  },
  {
    "file": "crates/verify/ess-conformance/src/synthesize.rs",
    "line": 4387,
    "category": "boundary",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "with a generated Uuid identity the instance name is not a valid Uuid, the guard stays undecided, and a target ignoring the narrowing passes the scalar suite"
  }
]
```
