---
format: aep.planning-md/3
id: review-result:adversary-d-linkguard-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit linkguard
relations:
- reviews: story:a-guard-compares-a-link-field-with-an-input
revision: 1
---
unit: linkguard (#193 guard case), working tree ess-c-linkguard on b19d255b4 + uncommitted subject_fact.rs diff
verdict: NEEDS-CHANGE
cases: executed 1616→1634, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 8 paths, 2 since deleted (part 6)
needs-coordinator: none

## 1. Diff stat

```
 .../ess-conformance/src/synthesize/subject_fact.rs | 316 ++++++++++++++++++++-   (implementor's, untouched)
?? crates/verify/ess-conformance/tests/adversary_linkguard_pass1.rs                (mine, 813 lines, untracked)
?? crates/verify/ess-conformance/tests/link_field_guard.rs                          (implementor's, untouched)
```
The only path I wrote is a test file.

## 2. Cases (tests/adversary_linkguard_pass1.rs, 18 cases)

Fixture: a ledger written in the test, with one defect switched in at a time. The unit's shape is the control, and the fixture passes it (green).

| case | asserts | now |
|---|---|---|
| `a_target_checking_the_named_account_holds_any_entry_fails_the_suite` | a target that refuses only when the named account holds no entry at all fails the suite | **red** |
| `a_target_reading_only_the_link_half_of_the_guard_fails_the_suite` | under `all: [account_id != input.account_id, memo == "locked"]`, a target that ignores the memo conjunct fails the suite | **red** |
| `under_any_a_target_reading_only_the_memo_half_fails_the_suite` | under `any: [account_id != input.account_id, memo == "locked"]`, a target that **ignores the link** fails the suite | **red** |
| 15 others | control; `all:` memo-only mutant; `any:` link-only mutant; payload carrying `input.account_id`; String identity (no token leak, runs green); `==` transfer that writes the link (refusal and accepting side); `not ==`; Org→Account→Entry chain; `defined()` + `!=`; `state == Posted` + `!=`; `==` on the accepting branch with the refusal as default; two synthesis runs giving the same bytes | green |

Red output, from running this file alone before the suite (`~/.cache/ess-wave-n2/linkguard/adv1/red.log`):
```
thread 'a_target_checking_the_named_account_holds_any_entry_fails_the_suite' (3080397) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass1.rs:553:5:
a target refusing only accounts that hold no entry passed every scenario
thread 'a_target_reading_only_the_link_half_of_the_guard_fails_the_suite' (3080398) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass1.rs:589:5:
a target refusing every other account, whatever the memo, passed every scenario
thread 'under_any_a_target_reading_only_the_memo_half_fails_the_suite' (3080409) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass1.rs:761:5:
a target refusing only locked entries, whichever account is named, passed every scenario
test result: FAILED. 15 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

Origin run: the same file against a `git archive b19d255b4` copy in its own target dir (`base-run.log`). All 18 fail there. Each of the 3 red shapes was refused `ESS-SYNTH-003: outcome ledger.book.AmendEntry/other-account has no scenario …`. So the unit takes a branch the base refused outright and synthesizes it with a coverage gap that nothing reports.

Note: a first base run on the shared `ess-c-linkguard` target reused the worktree's test binary (`adversary_linkguard_pass1-27aac6e4e73ba82a`, no recompile) and was discarded.

## 3. Suite, after the cases

`cargo test -p ess-conformance --no-fail-fast` (log `suite.log`): 228 result lines, 1631 passed, 3 failed, EXIT=101. Every failure is in my file:
```
test result: FAILED. 15 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
error: test failed, to rerun pass `-p ess-conformance --test adversary_linkguard_pass1`
```
Before: 1616 passed, from the implementor's `test-final.log`.

`cargo xtask generate --check` (`generate.log`): `projections are up to date`, EXIT=0.

## 4. Findings

| file:line | severity | verdict | finding | fix |
|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2524 | blocker | NEEDS-CHANGE | Under `any: [link != input, other]`, the disjunct goal "only the link true" is searched with `row_truth` and no input, so the link leaf is `Unknown` and the goal is dropped without a refusal. The refusal is witnessed only through the other disjunct, so **a target that ignores the link passes**. That breaks the story's acceptance on a shape the unit now admits and the base refused (SYNTH-003). Measured at tests/adversary_linkguard_pass1.rs:761. | In `boundaries`, evaluate goals with `row_truth_with` over `linked_inputs` and send through `bind_links`. Or refuse the branch as before when a goal holds a link leaf the search cannot decide. |
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2355 (goal from `conjunct_goals`, dropped by the search at :2524) | warning | NEEDS-CHANGE | Same cause for `all:`. No default row is arranged with "other account, memo not locked", so a target that refuses every other account whatever the memo passes. This breaks the per-conjunct promise in the `boundaries` doc comment (:2477). The implementor note says boundary rows keep "link guard there still refused", but nothing is refused: the goal is dropped silently. Measured at :589. | Same fix as above. |
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:847 | warning | CONFIRMED | `bind_links` arranges the second owner in its initial state and holding no row. A target whose check is "the named account owns some entry" rather than "owns this entry" passes both scenarios. Measured at :553. | Arrange one row of the guarded entity under `OTHER_OWNER` as well, so the refusal names an owner that holds rows. |

What reaches them: an authored `when_subject` predicate combining the link comparison with another stored leaf under `any:`/`all:` (F1, F2). Both forms are ordinary ESS predicates, but I found no in-repo model that uses them with a link. F3 needs only the plain shape from the unit's own test, plus a target with that defect.

## 5. Attacked, not broken

- The token did not leak into the suite: canonical JSON for a Uuid identity, a String identity, a payload carrying `input.account_id`, and a transfer writing the link holds no `instance:` text and no `uuid_of("instance:…")`.
- `==` form on the refusal and on the accepting branch, `not ==`, and reversed operand order: each side was witnessed and the guard mutants were killed.
- Owner of an owner (Org→Account→Entry): the second account got its own org and nothing was named twice.
- `defined(account_id)` beside the comparison, and `state == Posted` (#204) beside it: synthesized and mutants killed.
- An `Optional` owner link is refused by the compiler (`type_mismatch`), so that path cannot be reached.
- `when_related` beside `when_subject` on one command is `conflicting_declaration` by design (#211 revision), so no interaction exists.
- Determinism: two syntheses of each shape give the same bytes, and `cargo xtask generate --check` shows committed suites unchanged.
- `OTHER_OWNER` = further(65), which names `account-66`. `related_guard::retry(1)` also yields further(65), but for input witnesses, not instance names. I found no instance-naming path that reaches 65. Aggregate views grouped by the link were not attacked.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/linkguard/adv1/review.md
- ~/.cache/ess-wave-n2/linkguard/adv1/red.log
- ~/.cache/ess-wave-n2/linkguard/adv1/suite.log
- ~/.cache/ess-wave-n2/linkguard/adv1/generate.log
- ~/.cache/ess-wave-n2/linkguard/adv1/base-run.log
- ~/.cache/ess-wave-n2/linkguard/adv1/base/ (git archive of b19d255b4 plus my test file, 72M)
- ~/.cache/b10x-target/ess-c-linkguard-adv1-base (base build dir, 322M; already deleted by me)
- ~/.cache/ess-wave-n2/linkguard/adv1/append.rs (scratch, already deleted)

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2524
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: under any over a link comparison the link-only disjunct goal is searched without the input and dropped silently, so a target ignoring the link passes a suite the base refused as ESS-SYNTH-003
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2355
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: under all over a link comparison and another leaf the boundary row refuting the other leaf is never arranged, so a target ignoring that conjunct passes, contrary to the boundaries doc and to the "still refused" implementor note
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 847
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the second owner holds no row, so a target refusing only accounts that own no entry at all passes both sides of the guard
```
