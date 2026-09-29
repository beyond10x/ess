---
format: aep.planning-md/3
id: review-result:adversary-d-linkguard-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit linkguard
relations:
- reviews: story:a-guard-compares-a-link-field-with-an-input
revision: 1
---
unit: linkguard (#193 guard case), pass 2, working tree ess-c-linkguard on b19d255b4 + uncommitted subject_fact.rs diff (correction 1)
verdict: NEEDS-CHANGE
cases: executed 1634→1641, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths kept, 4 deleted (part 6)
needs-coordinator: none

## 1. Diff stat

```
 .../ess-conformance/src/synthesize/subject_fact.rs | 731 +++++++++++++++++++--   (implementor's, untouched: +672/-59 as briefed)
?? crates/verify/ess-conformance/tests/adversary_linkguard_pass1.rs                (pass 1)
?? crates/verify/ess-conformance/tests/adversary_linkguard_pass2.rs                (mine, new, untracked)
?? crates/verify/ess-conformance/tests/link_field_guard.rs                          (implementor's)
```
The only path I wrote in the tree is a new test file.

## 2. Cases (tests/adversary_linkguard_pass2.rs, 7 cases)

The fixture is a ledger written in the test. Shapes: `TransferEntry` (`same-account` on `account_id == input.account_id`, `moved` writes the link), `AmendEntry` with a link guard, and an opt-in voidable `Entry` (Posted→Voided) for the #204 `state` half.

| case | asserts | now |
|---|---|---|
| `under_cardinality_one_the_moved_branch_names_an_account_holding_no_entry` | under `cardinality: one`, the `moved` scenario does not move the entry into an account that already holds one | **red** |
| `a_ledger_keeping_cardinality_one_passes_the_transfer_suite` | a ledger that keeps `cardinality: one` passes the suite | **red** |
| `a_link_guard_beside_a_reachable_state_half_kills_a_target_reading_the_link_only` | `all: [state == Posted, account_id != input.account_id]` with Voided reachable: the correct ledger passes; guard-ignoring, refuse-all and state-ignoring mutants fail | **red** (the correct ledger fails) |
| `no_scenario_of_a_link_guard_captures_one_name_twice` | no scenario of the 4 link shapes captures an instance name twice; 2 syntheses give the same bytes | **red** |
| `control_the_transfer_under_cardinality_many_passes_and_kills_the_guard_mutants` | control | green |
| `a_link_guard_beside_an_input_comparison_kills_a_target_reading_the_link_only` | `all: [account_id != input.account_id, memo != input.memo]` (#157 beside the link): the correct ledger passes; guard-ignoring and link-only mutants fail | green |
| `control_no_scenario_of_the_same_guard_without_a_link_captures_one_name_twice` | the same question for `state == Posted` beside `memo == "locked"`, `memo != input.memo` and `memo != "locked"` | green |

Red output, from running this file alone before the suite (`red.log`):
```
thread 'under_cardinality_one_the_moved_branch_names_an_account_holding_no_entry' (354322) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass2.rs:511:5:
the entry is moved into an account that already holds one, under `cardinality: one`: filed under [Instance { instance: InstanceName("account") }, Instance { instance: InstanceName("account-66") }], moved to Instance { instance: InstanceName("account-66") }
thread 'a_ledger_keeping_cardinality_one_passes_the_transfer_suite' (354316) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass2.rs:523:5:
a ledger keeping `cardinality: one` fails the suite synthesized for it: {
    "ledger.book.TransferEntry/outcome/moved": "[Diagnostic { code: Outcome, ... expected: [\"outcome = moved\"], observed: [\"no declared outcome was reached; the target refused for a reason the specification does not model\"] } ...
thread 'a_link_guard_beside_a_reachable_state_half_kills_a_target_reading_the_link_only' (354317) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass2.rs:558:5:
assertion `left == right` failed
  left: {"ledger.book.AmendEntry/outcome/amended": "[Diagnostic { code: View, ... expected: [\"ledger.book.EntryAccounts holds a row where account_id = \\\"00000000-0000-4000-8000-000000000003\\\", id = \\\"00000000-0000-4000-8000-000000000004\\\", memo = \\\"memo\\\", state = \\\"Posted\\\"\"], observed: [... {account_id = ...03, id = ...04, memo = memo, state = Voided} ...
thread 'no_scenario_of_a_link_guard_captures_one_name_twice' (354321) panicked at crates/verify/ess-conformance/tests/adversary_linkguard_pass2.rs:637:5:
[
    "ledger.book.AmendEntry/outcome/amended: [\"account-2\", \"entry-2\"]",
test result: FAILED. 3 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

Origin run: the same file on a `git archive b19d255b4` copy, built in its own target dir (`base-run.log`). All 6 link-shape cases fail there with `ESS-SYNTH-003` (`AmendEntry/amended`, `AmendEntry/other-account`, `TransferEntry/moved`, `TransferEntry/same-account` have no scenario). The no-link control is green on the base. So the base refuses these shapes, and this unit now synthesizes them wrong.

Mechanism, read from a dump of the `amended` scenario (voidable shape; since deleted):
- The plain row posts `memo` and the amendment writes `memo`. That write is unchanged, so `prepare`'s fresh-witness loop (subject_fact.rs:1996, `FRESH_WITNESSES` = 3) moves the branch's own row to `further(1)`, which is `account-2`/`entry-2`.
- `boundaries` then numbers its first row `further(rows + 1)` = `further(1)` (:2672), which is `account-2`/`entry-2` again. That row is voided and named `account-66`.
- The closing observation of the branch's own row (`entry-2`, memo `memo`, Posted) therefore reads the rebound, voided entry, and a correct target fails.
- In the same scenario, `account-66`/`entry-66` are captured exactly once, so the `present` flag held.

## 3. Suite, after the cases

`cargo test -p ess-conformance --no-fail-fast` (`suite.log`): 229 result lines, 1637 passed, 4 failed, EXIT=101. Every failure is in my file:
```
test result: FAILED. 3 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
error: test failed, to rerun pass `-p ess-conformance --test adversary_linkguard_pass2`
error: 1 target failed:
    `-p ess-conformance --test adversary_linkguard_pass2`
```
Before: 1634 passed, 0 failed, from the implementor's correction-1 `c1/suite.log`.

`cargo xtask generate --check` (`generate.log`): `projections are up to date`, EXIT=0.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:953 | blocker | NEEDS-CHANGE | introduced | `row_under` gives the second owner an entry, and the doc at :952 says `cardinality: one` admits it. But when the branch writes the link (`moved`: `sets: {account_id: input.account_id}`), the scenario then moves the entry into that owner. Under `cardinality: one` that is an account holding two entries, and a ledger keeping its declared cardinality fails `moved`. Before correction 1 the owner held no row. Measured at tests/adversary_linkguard_pass2.rs:511 and :523. | Do not arrange `row_under` on a scenario whose sent branch writes the owner link from the compared input, or skip it under `cardinality: one` on such a branch. |
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2672 | blocker | NEEDS-CHANGE | introduced | `boundaries` numbers further rows from `further(1)`, but `prepare` may already have put the branch's own row at `further(1..=3)` (:1996). On a link shape the plain amendment leaves `memo` unchanged, so the own row moves to `further(1)`, the first boundary row captures `account-2`/`entry-2` a second time, and the branch's own closing observation reads the wrong row. A correct target fails `AmendEntry/amended`. The no-link controls do not reach it, in this tree or on the base. Measured at :558 and :637. | Number further rows past every distinction the scenario already holds, i.e. skip one whose instance name equals `setup.instance` or a name captured in the setup's steps. The same applies to `overlaps` at :2968. |
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:470 | note | CONFIRMED | introduced | The `compares_link` doc (:470), the `overlaps` comment (:2944) and the proposed changelog say a goal over a link comparison that no bounded arrangement reaches is "refused, never dropped". The code drops it silently when no candidate left it undecided (:2716, :3000), which is the admitted gap. A target ignoring that half is indistinguishable within the search bound, which is the #157-class limit every guard has. No failing case: I built no shape whose half is reachable only past the bound. What reaches it: nothing found. | Reword the doc, comment and changelog: refused where an input naming an owner left the goal undecided; a goal no bounded row meets adds no row, as for every guard. |

The third red case, `a_link_guard_beside_a_reachable_state_half_…`, also asks whether a state-ignoring mutant is killed. That is unanswered: the correct ledger fails first on finding 2.

What reaches F1: an authored `cardinality: one` owner relation plus a command that re-files a row under the named owner and refuses the same owner. Reassigning a single child to another parent is the ordinary form. What reaches F2: any link-guard command whose accepting branch has a conjunct goal (an `all:` sibling guard), with a write that the plain witness leaves unchanged. Here that is a plain `sets: {memo: input.memo}` with no memo literal in the guard.

## 5. Attacked, not broken

- #157 input comparison beside the link in `all:`: synthesized; the correct ledger passes; the guard-ignoring and link-only mutants are killed.
- Transfer under `cardinality: many`: the correct ledger passes; ignore and refuse-all mutants are killed on the right scenarios.
- #201 `when_subject_state:` beside `when_subject`: the compiler refuses it (`conflicting_declaration`: "subject fact and lifecycle guards cannot be combined in one command"), so the two do not interact.
- #211 `when_related`: conflicting with `when_subject*` by decision; not re-run.
- Aggregates: witnessed in their own scenarios (aggregate.rs `observe_change`), and command-scenario `Counts` are floors only (synthesize.rs:4817). The extra row cannot lower a floor. Not run.
- #202 `sources_apart`: runs in `prepare` before `bind_links` and not in `boundaries`. Not run.
- `present`: `account-66`/`entry-66` are captured once in every scenario of 4 link shapes. The only doubled names are finding 2's.
- Determinism: 2 syntheses give identical bytes for 4 link shapes and 3 controls; `generate --check` is up to date.
- (6) Pass-1 lint allow: the file differs from `c1/adversary_linkguard_pass1.rs.orig` by one added line, `#![allow(clippy::struct_excessive_bools, clippy::too_many_lines)]`. No assertion changed.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/linkguard/adv2/review.md
- ~/.cache/ess-wave-n2/linkguard/adv2/red.log
- ~/.cache/ess-wave-n2/linkguard/adv2/suite.log
- ~/.cache/ess-wave-n2/linkguard/adv2/generate.log
- ~/.cache/ess-wave-n2/linkguard/adv2/base-run.log
- ~/.cache/ess-wave-n2/linkguard/adv2/base/ (git archive of b19d255b4 plus my test file, 72M; deleted)
- ~/.cache/b10x-target/ess-c-linkguard-adv2-base (base build dir, 322M; deleted)
- ~/.cache/ess-wave-n2/linkguard/adv2/dump-{1,2,3}.json (scenario dumps from a temporary case, since removed from the test file; deleted)
- ~/.cache/ess-wave-n2/linkguard/adv2/head.rs (transient, moved over the test file; gone)
- Built into the assigned ~/.cache/b10x-target/ess-c-linkguard.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 953
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the second owner's own row makes the moved branch file a second entry under it, so under cardinality one a ledger keeping its declared cardinality fails the synthesized TransferEntry/moved scenario
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2672
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: boundaries numbers further rows from further(1) while prepare may already hold the branch's own row there, so a link-guard scenario captures account-2 and entry-2 twice and a correct target fails AmendEntry/amended
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 470
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the doc, the overlaps comment and the changelog promise a link goal no bounded row reaches is never dropped, but a goal no candidate left undecided is dropped silently
```
