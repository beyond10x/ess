---
format: aep.planning-md/3
id: review-result:zi-t2-lane3-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:an-arrangement-refusal-names-the-cause-of-every-creator-tried
relations:
- reviews: story:an-arrangement-refusal-names-the-cause-of-every-creator-tried
revision: 1
---
```
unit: story:an-arrangement-refusal-names-the-cause-of-every-creator-tried
verdict: weak
new failing cases: 1 (tests/arrangement_refusal_names_every_creator.rs::adv728_a_creator_whose_rows_miss_the_subject_guard_is_named_too)
mutants survived: 2 (not run: the unit's test passes if Display writes the first cause beside every creator; nothing before this pass reached the merge in across_targets, since no byte-pin row has a list of creators); both are killed by the new green cases
```
cases: executed 24→29, red 1. Tree: zi-t2-11 at HEAD a7cee9c8d8 plus my uncommitted test additions. Base 93ec319d3e. Origin: introduced 0 / pre-existing 1 / undecided 0. Nothing written outside the worktree except files in the adv/ scratch folder. The coordinator does not need to settle anything.

1. `git diff --stat`: one file changed, `crates/verify/ess-conformance/tests/arrangement_refusal_names_every_creator.rs`, 228 lines added. Only a test file changed, nothing else.

2. **Red case.** `adv728_a_creator_whose_rows_miss_the_subject_guard_is_named_too` (run alone, `adv/run1-cases.txt`, exit 101). The model is the unit's own, with Record's `incomplete` branch removed. Record can now create a Decision, but only with kind Erratum, and Annotate needs one with kind Accept. Confirm still has no input that reaches its branch. Fixing either command would make the scenario reachable, but the refusal names only Confirm:
   `ESS-SYNTH-004 demo.review.Annotate/outcome/annotated: it needs an instance of `demo.review.Decision` to change without moving, and the route runs through `demo.review.Confirm/confirmed`, which no input reaches`
   - **Cause:** in `subject_fact.rs:5734`, `search_under` only lists all creators when every creator failed with `InstanceRequired`. Record fails with `GuardUnsatisfiable`, so the code falls back to naming the first creator.
   - **Who hits it:** any author whose creators fail for causes of different kinds. This is the same misdirection as #728.
   - **Status:** outside acceptance item 1 (that item is about m4 only), so it goes to the backlog rather than blocking. The base code at 93ec319d3e also keeps only the first cause, so it is pre-existing.

3. **Green cases, kept because nothing else in the suite checked these:**
   - `adv728_each_creator_is_paired_with_its_own_cause`: each creator's cause names its own branch.
   - `adv728_the_plain_arrange_route_names_every_creator`: the same check on the route without `when_subject`.
   - `adv728_declaration_order_does_not_move_the_tried_list`: declaring Record before Confirm gives the same bytes.
   - `adv728_every_target_state_contributes_its_creators_causes` (`adv/run3-target-merge.txt`, exit 0): two target states each produce a creators list, and the refusal names all four blocked moves. An earlier run (`adv/run2-target-merge.txt`) failed because my model was invalid (Draft could not be reached), not because of a defect; I fixed the model.

4. **Suite run** (`adv/run4-suite.txt`, exit 101): the unit's target plus `when_subject_witness`, `existing_instance_beside_related_guards` and `adversary_whensubj_pass1`. 28 passed, 1 failed, the red case above.

5. **Minor issue in the code comment** (`synthesize.rs:3032`): the comment on `across_targets` says each creator is named once. In the run3 output Begin and Prepare each appear twice, and the text does not say which target state each entry is for. This is a note, not a defect.

6. **Attacked and could not break:**
   - Determinism: the order is by command name, and declaration order does not change it.
   - A reversed or truncated list fails the unit's own test.
   - A creator refused with a nested ESS-SYNTH-004 cannot grow without bound: `of_creators` flattens nested lists.
   - The ESS-SYNTH-004 row in `website/docs/reference/diagnostics.md` uses explanation text the diff did not change. I did not run `cargo xtask docs`.
   - Byte pins: the implementor's gate shows them green. They hash the printed refusals, so a mutant that changed the text for a single creator would move a row.

7. **Files written outside the worktree:** `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/an-arrangement-refusal-names-the-cause-of-every-creator-tried/adv/` holding run1-cases.txt, run2-target-merge.txt, run3-target-merge.txt and run4-suite.txt.

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 5734
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "when one creator fails with InstanceRequired and another with GuardUnsatisfiable, search_under falls back to the first creator's cause, so the ESS-SYNTH-004 refusal still names only Confirm and hides Record's repair (adv728_a_creator_whose_rows_miss_the_subject_guard_is_named_too)."
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 3032
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "across_targets merges per-target lists by (creator, cause), so a creator blocked differently in two target states is named twice, and the text does not name the target states, against its doc comment \"each creator is named once\"."
```
