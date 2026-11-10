---
format: aep.planning-md/3
id: review-result:zi-w1-lane5-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
relations:
- reviews: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
revision: 1
---
unit: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding (099b6a15d7 plus an uncommitted test file in zi-t1-adv)
verdict: broken
new failing cases: 3 (binding_trigger_row_predicate.rs::adversary_a_creator_branch_sharing_the_publisher_name_does_not_pass_for_the_trigger, ::adversary_a_refusing_sibling_guard_names_the_trigger_predicate, ::adversary_an_input_reading_predicate_is_not_called_the_only_publisher)
mutants survived: none (no mutation run)

- **Diff and runs:** the only changed file is `crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs` (+253, tests only). The 4 new cases alone gave 1 passed, 3 failed, exit 101 (`adv/red1.txt`). The whole binary then gave `10 passed; 3 failed`, exit 101, slot B (`adv/suite.txt`). Executed cases went from 9 to 13.
- **Blocker, the creator case.** With this commit's code, synthesis writes a `binding/refusal/not-eligible` scenario that the honest interpreted model fails. The fixture is a trigger `Touch` that copies `{subject: tier_id}`. Its creator `Open` refuses an ineligible tier on a branch named `ok`, the same name as `Touch`'s publishing branch. The run fails at `:799`: `outcome demo.r.Open/opened … expected ["outcome = opened"], observed ["outcome = ok"]`.
  - Cause: on the stored path, `binding_effects.rs:1908` compares the creator's branch name with the trigger's `published_by.name`. It should compare with the creator branch that the trigger's setup expects (`taken` in `stored_creation`).
  - At base, `keeps_branch_sent` refused `Holds` guards, so this path stayed refused. That comes from reading the base code; I did not run it at base.
  - What reaches it: two commands that share a branch name. I built that fixture; no real model was shown to have it.
- **Warning, the sibling case.** The trigger refuses an ineligible tier on an earlier sibling (`blocked`, `eligible == false`, error) and publishes on its default branch. The refusal keeps the old help, "synthesis arranges one itself where …", whose two conditions both hold (`:615`).
  - This breaks Design step 4 of the story. Its acceptance only names the variant where the default branch is dropped.
  - Cause: `unarranged` (`refusal_policy.rs:597`) only looks for a `Holds` guard on the publishing branch itself.
- **Warning, the input-predicate case (kept case 1).** The refusal now says "`demo.r.Refresh/eligible` publishes the event only where its `when_related:` predicate `eligible == input.flag` holds". That is false, because `Refresh/not-eligible` also publishes the event (`:649`). The sentence is new text from this commit (`refusal_policy.rs:607`).
- **Green case kept:** `adversary_the_trigger_is_expected_on_the_branch_the_row_sends_it_down` checks the acceptance clause "trigger expected on `not-eligible`" (and on `eligible` for flow and delivery). The unit's tests did not assert it.
- **Judgement:** the unit's `contains("when_related")` check is satisfied by the old help text too, so it cannot fail. The control test checks scenario counts only, not the suite bytes the acceptance asks for.
- **Could not break:** declaration order with the default branch last, and the two-row ambiguity (`several()` rules it out).
- **Written outside the worktree:** `<scratch>/adv/append.rs`, `adv/red1.txt`, `adv/red1.txt.touch`, `adv/suite.txt`.

```findings
- file: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
  line: 1908
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: on the stored path trigger_publishes compares the creator's branch name with the trigger's publishing branch, so a creator refusal named like it is accepted and the written refusal scenario fails on the honest model
- file: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
  line: 597
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: a trigger whose row predicate sits on a refusing sibling keeps the old help whose two conditions hold, against Design step 4
- file: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
  line: 607
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the new refusal text says the trigger publishes only where its predicate holds when an unguarded branch of the same command also publishes the event
```
