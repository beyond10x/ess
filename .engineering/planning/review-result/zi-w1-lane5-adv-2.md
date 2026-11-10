---
format: aep.planning-md/3
id: review-result:zi-w1-lane5-adv-2
kind: review-result
status: active
title: Turn 1 adversary pass 2 on story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
relations:
- reviews: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
revision: 1
---
unit: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding (tree zi-t1-adv at 4f00151ee1, plus one untracked test file)
verdict: weak
new failing cases: 1 (adversary_t1_lane5_pass2.rs::adv2_a_creator_that_cannot_make_the_refused_row_is_not_blamed_on_the_trigger)
mutants survived: none (no mutation run)

- **Diff:** `git status --short` shows only `?? crates/verify/ess-conformance/tests/adversary_t1_lane5_pass2.rs` (408 lines, a test file; `git diff --stat` is empty because the file is untracked). No implementation file was touched. Executed cases went from 13 to 16, with 1 red.
- **Runs:** All four cargo runs of the budget are used. Runs 1 to 3 (`adv2/red1.txt`, `red2.txt`, `red3.txt`) failed the two stored-path cases on fixture validation (my typos, not findings), and showed the misattribution case red at the same assertion each time. Run 4 (`adv2/run4.txt`) was `test -p ess-conformance --locked --no-fail-fast --test adversary_t1_lane5_pass2 --test binding_trigger_row_predicate`. Its summaries were `2 passed; 1 failed` and `13 passed; 0 failed`, ending `exit 101`. With the budget gone, that last run doubled as the first valid run of the stored-path cases and as the suite run.
- **Warning, the refusal blames the wrong command (beside the acceptance, against Design step 4).** I changed the unit's fixture so that `DefineTier` sets `eligible: true` with no input, so no ineligible tier can ever be created. The refusal still says "…before the trigger: `demo.r.Refresh` chooses its branch by `when_related:` predicates over that row (`eligible == true`)". The help that named the real cause, "takes each field the predicate reads as input", is dropped. That is false as a cause: `Refresh/not-eligible` publishes the event for any row.
  - Failing assertion: `adversary_t1_lane5_pass2.rs:259`, the `!contains("chooses its branch …")` check.
  - Cause: `unarranged` (`refusal_policy.rs:598`) names the first publisher that has any predicate over the entity. It never checks that this publisher was dropped because of that predicate (by `trigger_publishes`).
  - Fix: name the trigger only when `related_arranged` dropped every candidate at `trigger_publishes`. Otherwise keep the old gap.
  - What reaches it: a tier created eligible and suspended later by an update command, which is an ordinary model.
- **Green cases, kept:**
  - `adv2_a_stored_trigger_guarded_by_its_own_row_is_not_sent_for_a_refused_row`: the trigger `Touch` chooses its publishing branch by `when_related: {via: tier_id, predicate: …}`, and the creator `Open` carries a predicate on another field (`frozen == true`).
  - `adv2_a_stored_trigger_with_an_unguarded_creator_is_not_sent_for_a_refused_row`: the same with no predicate on the creator.
  - Both pass the honest interpreted model. I did not observe whether `refusal/not-eligible` was written or refused, so they may pass vacuously. I kept them because they pin a shape the suite does not cover.
- **Could not break:**
  - the stored path with a trigger guarded by its own stored-field predicate;
  - the creator branch compared by identity (the pass-1 case stays green);
  - the control's digest pin;
  - `Unknown` on an unwritten Optional field or under `not`, which drops the candidate rather than writing a wrong scenario (read from the code: `row_truth_with` with an empty `unwritten`).
- **Not tried, budget spent:** two publishing commands where `another_publisher` picks the wrong one.
- **Written outside the worktree** (all in `<scratch>/adv2/`): `head.rs`, `red1.txt`, `red1.txt.touch`, `red2.txt`, `red3.txt`, `run4.txt`.

```findings
- file: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
  line: 598
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: unarranged names the first publisher that has any predicate over the refused entity as the cause, even when that publisher's default branch publishes for every row and the creator cannot make the refused row, and it drops the help naming the condition that actually fails
```
