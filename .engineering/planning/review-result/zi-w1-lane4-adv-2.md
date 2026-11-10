---
format: aep.planning-md/3
id: review-result:zi-w1-lane4-adv-2
kind: review-result
status: active
title: Turn 1 adversary pass 2 on story:a-multi-state-wrong-state-error-branch-is-synthesized
relations:
- reviews: story:a-multi-state-wrong-state-error-branch-is-synthesized
revision: 2
---
```
unit: story:a-multi-state-wrong-state-error-branch-is-synthesized
verdict: broken
new failing cases: 2 (adversary_listed_state_refusal_pass2.rs::a_later_sibling_pair_no_row_misses_leaves_wrong_state_witnessed, adversary_listed_state_refusal_pass2.rs::a_target_reading_the_later_subject_guard_first_fails_some_scenario)
mutants survived: 1 (a target that answers `when_subject:` before a `when_subject_state:` declared ahead of it passes every scenario of the suite)
```
- **Cause.** Correction E sends every listed-state refusal through `subject_fact::refusal_witness`. Its `refusal_input_from` (`subject_fact.rs` about 3345–3355, `row_only`/`halves`) has the row miss every stored-guard sibling, including those declared after the refusal. Under the one rule (ess/27+, step 3, declaration order) the refusal declared first answers on any row in a listed state. I did not run c80b97ab17 (not allowed to check it out). Both reds are E's doing by reading the code: before E the row was the unconstrained `arrange_first` one.
- **Red 1, blocker (acceptance bullets 2 and 3).** `wrong-state` is declared first, followed by `is-prose: source == Prose` and `not-prose: source != Prose`. Synthesis now refuses a branch that any Rejected row witnesses: `ESS-SYNTH-003 … no candidate of the 1 tried satisfies 'a Item row in this state refuting every one of: source == Prose, source != Prose'`.
  - Command: `build.sh <scratch>/adv2/run2.txt <tree> test -p ess-conformance --locked --test adversary_listed_state_refusal_pass2`, `exit 101`, `slot A waited 10s ran 15s`. The panic is at line 199.
- **Red 2, the surviving mutant (blocker-adjacent).** Model: `wrong-state` first, then `is-prose`, enum `[Prose, Table]`. Every scenario now sits on a `Table` row, so a mutant target with `is-prose` declared before `wrong-state` passes all 22 scenarios. The panic is at line 236, same outfile.
- **The re-pin.** The scenarios of `guard-pair-corpus.yaml` still pass the interpreted target, but the re-pin is not right. `<scratch>/adv2/run1.txt`, `exit 101`, `slot A waited 10s ran 120s`; it fails only on my file, and each corpus test reports ok:
  - `precedence_plan_answers::guard_pair_corpus_answers_by_the_rule`;
  - `explore_guards_first`, all 3, including the vip-first catch;
  - `mutation_arms::guard_pair_corpus_mutant_dropping_either_guard_turns_a_scenario_red`.

  The byte change has the same cause as the reds. In `SubjectBesideState`, `vip` is declared after `released`, so missing it is not required. The change goes against bullet 3 ("keeps its bytes") and should revert once the fix lands.
- **Suggested fix.** For a `when_subject_state:` refusal that is not the default, refute only the stored-guard siblings declared before it. Keep the full refutation for the default wrong-state refusal.
- **Held: four cases, green and left in place.** None was covered by the existing tests.
  - Two earlier stored-guard siblings: both missed, passes.
  - An earlier sibling that reads the row and the input together: missed through the input, passes.
  - Two earlier siblings that between them hold on every row: refused with a cause met in a listed state, no scenario for it, every scenario passes.
  - A target narrowed to `[Rejected]` or `[Stale]`: caught by the `state/X/refuses` scenarios.
- **Not re-tested.** A command that reads no stored row: it keeps the unit's original path, and that path's tests were green in `corr-c/fullE2.txt`.

The new test file is `~/.local/state/worktree/trees/b10x/ess/zi-t1-adv/crates/verify/ess-conformance/tests/adversary_listed_state_refusal_pass2.rs`, uncommitted. The outfiles are in `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-multi-state-wrong-state-error-branch-is-synthesized/adv2/`. I used 2 of the 4 cargo runs.
