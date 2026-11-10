---
format: aep.planning-md/3
id: review-result:zi-w1-lane3-adv-2
kind: review-result
status: active
title: Turn 1 adversary pass 2 on story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
relations:
- reviews: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
revision: 2
---
unit: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed (beyond10x/ess#727), tree zi-t1-adv at bc8ce31cec (also covers 763faf488b)
verdict: broken
new failing cases: 2 (adversary_t1_lane3_pass2_near_miss.rs::adv2_near_miss_sends_an_input_another_branchs_subject_guard_selects, ::adv2_a_guard_decided_false_beside_an_identity_read_is_still_forced)
mutants survived: 1 (a target that ignores the narrowing passes the whole suite when the guard is `all: [requires == input.missing, origin == input.id]`)
cases: executed 36→40, red 2. The 36 is the five named targets (5+5+7+17+2), the same as part 3 without my file.
origin: introduced 1 / pre-existing 0 / undecided 1
wrote-outside-worktree: 3 (`adv2/r1.txt`, `adv2/r2.txt`, `adv2/r3.txt`)
needs-coordinator: yes. Finding 1 goes back to the implementor.

1. **Diff:** `git diff --stat` is empty. The only change is one untracked test file, `crates/verify/ess-conformance/tests/adversary_t1_lane3_pass2_near_miss.rs`. No non-test path was touched.

2. **My file run alone:**
   - First run (`adv2/r1.txt`, exit 101, slot B ran 421s): the identity case was red as claimed. The another-branch case failed on a typo in my model, `unknown_capability` is not a valid outcome name, so that failure is not a finding. I fixed the name.
   - Second run (`adv2/r2.txt`, exit 101, slot B ran 15s): 1 passed, 2 failed.
   - **`adv2_near_miss_sends_an_input_another_branchs_subject_guard_selects`** (red, :222). The model is the story's only-reader spec plus a refusal `unknown-capability` whose subject guard is true when some element of `input.missing` is not in `requires`.
     - In the admitted scenario, near_miss sends `missing: ["missing.0"]` against `requires: ["requires.0"]` and forces `refused`.
     - A correct target then takes `unknown-capability`. Failing: `DecideAdmission/outcome/admitted` and `Rollout/transition/admit/by/DecideAdmission/admitted`.
     - Cause: `subject_fact.rs:535-539`. near_miss checks only the other branches' input guards, never another branch's subject guard (the row part) when it reads `input.`.
     - The control case `adv2_control_without_the_near_miss_the_correct_target_passes` removes the own `when:`, so near_miss cannot run, and it is green. That pins the failure on near_miss, not on the arrangement.
     - Suggested fix: also require every other guarded branch's `guard_truth_with` to be False on the changed input.
     - Beside acceptance, but it is a sound-suite break from the new code: a correct target fails the suite. What reaches it: I built the model. Pairing a refusal with the negation of the narrowing is a natural spec shape.
   - **`adv2_a_guard_decided_false_beside_an_identity_read_is_still_forced`** (red, :265).
     - The narrowed guard is `all: [requires == input.missing, origin == input.id]`. On the admitted row, `requires` differs from `missing`, so the guard is false whatever the id is, yet no scenario forces `refused`.
     - Cause: the skip at `synthesize.rs:4392-4401` drops any guard that reads an unbound identity, before three-valued evaluation could decide it. A target ignoring the narrowing passes all 13 scenarios.
     - This is a backlog item beside acceptance. I built the model, and nobody was shown to reach it.
     - Root cause: `unknown_by_absence` (`subject_fact.rs:584`) reads a field `flatten_partial` left unbound as absent. That is why the skip and near_miss's `bound` check exist. Fixing that would let the guard be evaluated instead of skipped.
     - Origin is undecided: I did not run it at base 2a03419cd7. Reading the base code, it did not force here either.
   - **Green, kept:** `adv2_near_miss_keeps_the_siblings_stored_copy_of_the_input_consistent`. The sibling's `sets: {granted: input.missing}` is read back by a view, and the near miss still fires. This pins the effect-reads-changed-input angle, which the suite did not cover.

3. **Suite after my file existed** (`adv2/r3.txt`, `--no-fail-fast`, exit 101, slot B ran 45s):

| target | result |
|---|---|
| adversary_t1_lane3_narrowed_input_relation | ok, 5 passed |
| adversary_t1_lane3_pass2_near_miss | FAILED, 2 passed, 2 failed |
| external_beside_held_guard | ok, 17 passed |
| external_narrowed_by_input_relation | ok, 5 passed |
| external_narrowed_by_subject | ok, 7 passed |
| synthesis_suite_bytes_table | ok, 2 passed |

4. **Judgement:** near_miss's `sourced` test matches `"field"` in the Debug text of the payload. `FactPath` Debug prints without quotes (`FactPath(input.missing)`), so an `{expr: …}` payload source is never seen as sourced. This is harmless today, because payload expectations are recomputed from the updated `run.input`. Not a case.

5. **Attacked and could not break:**
   - a view-observed `sets:` of the changed field;
   - an any/not connective over an unbound leaf in the force path (the skip makes it unreachable there);
   - byte pins: green.

6. **Files outside the worktree:**
   - `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed/adv2/r1.txt`
   - `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed/adv2/r2.txt`
   - `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed/adv2/r3.txt`

7. Findings:
```findings
[
  {
    "file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs",
    "line": 535,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "near_miss checks only other branches' input guards, so it sends an input on which another branch's subject guard reading input. is true and a correct target fails the admitted scenario"
  },
  {
    "file": "crates/verify/ess-conformance/src/synthesize.rs",
    "line": 4392,
    "category": "mutant",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "undecided",
    "message": "the force step skips any narrowed guard reading an unbound identity input even where another conjunct decides it false, so a target ignoring the narrowing passes the whole suite"
  }
]
```
