---
format: aep.planning-md/3
id: review-result:zi-t2-setguard-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:a-set-guard-fed-by-a-subject-payload-is-arranged
relations:
- reviews: story:a-set-guard-fed-by-a-subject-payload-is-arranged
revision: 1
---
unit: story:a-set-guard-fed-by-a-subject-payload-is-arranged (attacked commit e000e23866, HEAD^1 of the catch-up merge 8846f91c3e in zi-t2-4)
verdict: weak
new failing cases: 3 (adv_set_guard_subject_payload.rs::renamed_trigger_guarding_the_stored_field_passes, ::the_fixture_steers_the_creator_to_a_value_it_would_not_send_unsteered, ::renamed_creator_is_steered_and_the_unsteered_mutant_is_caught)
mutants survived: 1 (deleting `input.insert(..)` in `steer_creator`, set_guard.rs:307, leaves the committed fixture's suite byte-identical, and the interpreter still passes the flow on a renamed-input fixture)

- **Diff:** `git status --short` shows one untracked file, `crates/verify/ess-conformance/tests/adv_set_guard_subject_payload.rs`. It is a test file and no implementation file was touched. `git diff --stat` is empty because the file is untracked.
- **Cases:** 25 ran before, 45 after, 3 red. The case-alone runs are in `adv/run1.txt` to `adv/run3.txt`. The suite run is `adv/suite.txt`, exit 101, with only my file red.
- **F1, the emitted suite is wrong (introduced, beside the acceptance):** the trigger `Leave` refuses as `not-theirs` when `member_id != input.member_id`, and the creator stores `member_id: input.who`. `steer_creator` changes Join's `who` from `fc19…` to `5f8c…`, but Leave is still sent `member_id = fc19…`. So `/flow`, `/delivery`, `/mapping` and `/on-failure` fail on the interpreter with `expected outcome = removed, observed outcome = not-theirs`. `decided_after` does not re-check the trigger's own branch against the new stored value. At base this binding was refused, so the change turned a refusal into scenarios that fail a correct implementation. Proposed fix: refuse the steer where any later step reads the stored field, or re-decide the trigger's send after steering.
- **F2, a test that cannot fail (introduced):** UUID witnesses are hashed from the fact path (`witness::uuid`), and `Join.member_id` and `Reconcile.member_id` are both `input.member_id`. So the steered and unsteered values are the same: `00000000-0000-4000-8000-5f8c7101f399` in both the flow and `Leave/outcome/removed` (`adv/run1.txt`, `assert_ne` at :121). The `if` body of `steer_creator` never runs in `binding_set_guard.rs:208`.
- **F3, the interpreter does not catch the mutant even when steering happens (introduced in the test's evidence, cause not confirmed):** on the renamed fixture the flow steers `fc19…` to `5f8c…` and ends in Kept. Setting the creator back to `fc19…` in the JSON still passes on the interpreter (`adv/run3.txt`: `the unsteered flow fails: {}`). My guess is that the Leave steps in the arranged rows fire the binding and publish Kept before the trigger runs, so a bare `eventually_event` is met by that earlier event. I did not confirm this.
- **Could not break (each passes on the interpreter, and is arranged where the test checks it):**
  - `exists: false`, both with and without `state == Active` (plain and renamed)
  - two subject conjuncts (`eligible == input.flag`, plain and renamed)
  - a creator that guards the steered field
  - a creator storing the field under another input name; its flow ends in Kept
  - two creators of the trigger's row
  - a creator that publishes the steered field
  - a trigger that resends the stored value
  - the acceptance's negative case: 9 scenarios, 4 refusals, three `ESS-SYNTH-010`
  - all four byte pins green in `adv/suite.txt`
- **Not tried:** the subject row created by an earlier binding, and the CLI `ess verify conform run --target interpreted`. The in-test `Interpreted` runner covers the same check.
- **Written outside the worktree:** `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-set-guard-fed-by-a-subject-payload-is-arranged/adv/` holds `run1.txt`, `run2.txt`, `run3.txt`, `suite.txt` (plus `.touch` files) and `head.rs`, `tail.rs`. The builds used the shared slot A target directory.

```findings
- file: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs
  line: 245
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "steering the creator's stored field is not re-decided against the trigger's own branch, so a trigger comparing that stored field with its own input takes its refusal and four emitted binding scenarios fail on the interpreter where the binding used to be refused"
- file: crates/verify/ess-conformance/tests/binding_set_guard.rs
  line: 208
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the committed fixture's creator and invoked inputs share the fact path input.member_id, so the steered and unsteered UUIDs are identical, steer_creator's branch never runs, and a mutant removing it emits byte-identical suites"
- file: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs
  line: 307
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "with the creator's input renamed so steering does happen, the flow with the creator left unsteered still passes on the interpreter, so the interpreter check in the test does not observe steering (cause not confirmed: probably a Kept published by the arranged Leave steps)"
```
