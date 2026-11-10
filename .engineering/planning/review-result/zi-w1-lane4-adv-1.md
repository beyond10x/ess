---
format: aep.planning-md/3
id: review-result:zi-w1-lane4-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-multi-state-wrong-state-error-branch-is-synthesized
relations:
- reviews: story:a-multi-state-wrong-state-error-branch-is-synthesized
revision: 2
---
```
unit: story:a-multi-state-wrong-state-error-branch-is-synthesized, commit c80b97ab17 (tree zi-t1-adv, plus one uncommitted test file)
verdict: NEEDS-CHANGE
cases: executed 74→79, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: 6 paths, all under <scratch>/adv/
needs-coordinator: origin of the finding (no base tree to run it against)
```
```
unit: story:a-multi-state-wrong-state-error-branch-is-synthesized
verdict: broken
new failing cases: 1 (adversary_listed_state_refusal_pass1.rs::reproduction_with_prose_not_first_passes_wrong_state)
mutants survived: none
```

**1. Diff stat.** `git status --short` shows only `?? crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass1.rs`. That is one new test file and no tracked file changed. `git diff --stat` is empty because the file is untracked.

**2. Cases added** in `~/.local/state/worktree/trees/b10x/ess/zi-t1-adv/crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass1.rs`. They use the story's own reproduction instead of the unit's smaller model. That means keeping the `not-prose` refusal (`when_subject: {predicate: source != Prose}`) declared before `wrong-state`, adding a create with `into: Accepted`, and adding a view that publishes `state` (the story's `b`).
- `reproduction_with_prose_not_first_passes_wrong_state` is **red**. The enum is `[Table, Prose]`. Synthesis returns no refusal, but the interpreted run fails the scenarios it built. First run, of this target alone: `build.sh <scratch>/adv/run1.txt <tree> test -p ess-conformance --locked --test adversary_listed_state_refusal_pass1`, `exit 101`, `slot B waited 1s ran 1370s` (paused on the disk floor). The failure, verbatim:
  ```
  panicked at crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass1.rs:197:5:
      "example.catalogue.Item/state/Rejected/refuses/example.catalogue.SetLevel": Failed,
      "example.catalogue.Item/state/Stale/refuses/example.catalogue.SetLevel": Failed,
      "example.catalogue.SetLevel/outcome/not-prose": Passed,
      "example.catalogue.SetLevel/outcome/set": Passed,
      "example.catalogue.SetLevel/outcome/unknown": Passed,
      "example.catalogue.SetLevel/outcome/wrong-state": Failed,
  ```
- Four cases are green and stay in place, because they pin shapes the unit's single model does not:
  - `reproduction_with_prose_first_passes_wrong_state`: the same reproduction with `[Prose, Table]`.
  - `a_one_element_list_reached_by_two_moves_passes`: `[Stale]`.
  - `a_list_naming_the_initial_state_passes`: `[Extracted, Stale]`.
  - `a_list_beside_an_input_guard_passes`: `[Rejected, Stale]` plus `when: level == High`.

**3. Suite run**, after the case existed: the unit's gate targets plus the lane targets plus mine, `build.sh <scratch>/adv/suite2.txt <tree> test -p ess-conformance --locked --no-fail-fast --test …` (12 named targets). Every target is ok except `adversary_listed_state_refusal_pass1`, which shows `4 passed; 1 failed`. The tail is `error: 1 target failed:`, `exit 101`, `slot B waited 91s ran 60s`. The earlier run without `--no-fail-fast` (`adv/suite.txt`) stopped at the same red binary. The "before" count of 74 is the sum of the implementor's `gate-*.txt` result lines.

**4. Finding: the arranged row sets off the earlier subject-predicate sibling.**
- **What was measured:** `crates/verify/ess-conformance/src/synthesize.rs:6217`. `run_state_refusal` arranges the row with `arrange_first(.., &[])`, which takes no field constraints. `selected_in_state` skips `SubjectPredicate` branches. So the row is created with the first enum variant (`Table`), the correct target answers `not-prose`, and synthesis has still emitted a `wrong-state` scenario and the per-state scenarios. The result is a suite that fails a correct target, with no refusal to warn.
- **Acceptance:** this is in acceptance bullet 2 ("`b` … no refusal, and `ess verify conform run --target interpreted` passes it"), so it is a **blocker**. It holds only when `Prose` is not the first variant, and the story does not fix the variant order.
- **What reaches it:** any adopter whose enum is not ordered with `Prose` first. The issue's real `rfcspec` model has the `not-prose` and `other-text` subject predicates declared before `wrong-state`.
- **Origin: undecided.** The new filter only removes states in which `wrong-state` was never selected, so the base very likely built the same failing scenario. The `state/*/refuses` scenarios come from `state_refusals` at `:6323`, which the diff does not touch. I could not run it at base.
- **Suggested fix (not applied):** have `run_state_refusal` arrange the row so that subject-guarded siblings read earlier are falsified, as the default `wrong_state` path already does through `subject_fact::selects` (#192). Where that is not possible, refuse the scenario.

**5. Attacked and not broken:**
- **Empty or unknown state list:** validation refuses an empty list (`command.rs:7553`), so the `expect` at `synthesize.rs:4835` cannot hit an empty filter.
- **One-element list, list naming the initial state, list beside an input guard:** all green.
- **Nesting under `all:`/`any:`/`not:`:** has no `SubjectState` form, and `is_state_refusal` does not route it.
- **Mutants:** reverting the filter turns 2 of the unit's 3 tests red (its `red.txt`). Negating it breaks the unit's third test.
- **Byte pins and neighbouring targets:** green.
- **Go and TypeScript drift:** not attacked. The defect is in what synthesis emits, so it reaches every target.

**6. Paths written outside the worktree**, all under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-multi-state-wrong-state-error-branch-is-synthesized/adv/`: `issue.json`, `run1.txt`, `suite.txt`, `suite.txt.touch`, `suite2.txt`, `suite2.txt.touch`. `build.sh` also writes into its shared slot B target dir, which is the existing shared one.

**7.**
```findings
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 6217
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: a listed-state refusal declared after a subject-predicate refusal is arranged on a row that sets off that sibling, so with Prose not the first variant the synthesized wrong-state and state/*/refuses scenarios fail the interpreted target while synthesis reports no refusal
```
