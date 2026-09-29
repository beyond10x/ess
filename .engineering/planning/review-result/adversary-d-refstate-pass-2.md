---
format: aep.planning-md/3
id: review-result:adversary-d-refstate-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit refstate
relations:
- reviews: story:an-input-refusal-sits-beside-a-subject-state-branch
revision: 1
---
unit: refusal-beside-state (#227) after correction 1, worktree ess-d-refstate, uncommitted diff on db92e9fa9 plus untracked test files
verdict: NEEDS-CHANGE
cases: executed 1665→1670 (ess-conformance; before = the correction-1 run, corr1/conformance.log), red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 10 paths (list in part 6)
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat`: 11 files changed, 679 insertions(+), 67 deletions(-). This is the unit's own tracked diff, unchanged by this pass. My only addition is one untracked test file, `crates/verify/ess-conformance/tests/adversary_refstate_pass2.rs`. No non-test path was touched.

## 2. Cases added (`adversary_refstate_pass2.rs`)

| test | asserts | now |
|---|---|---|
| `:534` `the_missing_row_witness_still_carries_the_accepting_branch_input_beside_a_refusal` | sign-in model with `blocked: when: client == "blocked"` (refusal) and `fast-path: when: client == "console"` (accepting). The `no-configuration` scenario sends a missing row with client `blocked` and also with client `console` | RED |
| `:552` `a_target_taking_the_accepting_branch_before_the_missing_row_fails` | same model, with every InitiateSignIn branch filed. A correct target passes all scenarios; a target answering `fast-path` before reading the configuration fails at least one | RED |
| `:383` `validation_synthesis_and_the_interpreter_answer_every_combination_alike` | a table of 8 models: when_related+refusal; when_related+refusal+accepting; when_related+existing_instance+refusal; refusal+unknown_instance; refusal+held state; two refusals (held); two refusals (plain); refusal+accepting+external. Each validates, each is synthesized, and each is run against `Interpreted`. No scenario may end Failed or Error, and at least one outcome check per row must agree (the existing_instance row is exempt: the interpreter declines it by design) | green |
| `:451` `a_refusal_every_input_of_which_an_earlier_refusal_claims_is_named` | a later refusal fully shadowed by an earlier one (held-state `frozen-again`, plain `count < 1` inside `count < 2`) is filed or refused by name | green (ESS-SYNTH-003 "…an input-guarded refusal declared before it, which answers first") |
| `:474` `an_accepting_branch_every_input_of_which_a_refusal_claims_is_named` | an accepting held-state branch that a refusal fully shadows is named | green (ESS-SYNTH-003) |

Red output, verbatim, from the file run alone (`cargo test -p ess-conformance --test adversary_refstate_pass2`), before the suite ran:

```
thread 'the_missing_row_witness_still_carries_the_accepting_branch_input_beside_a_refusal' (2498292) panicked at crates/verify/ess-conformance/tests/adversary_refstate_pass2.rs:543:5:
the accepting branch's input is not sent for a missing row: [Some(Literal { value: Text("blocked") })]
```
```
thread 'a_target_taking_the_accepting_branch_before_the_missing_row_fails' (2498290) panicked at crates/verify/ess-conformance/tests/adversary_refstate_pass2.rs:582:5:
a target answering `fast-path` for client "console" before reading the configuration passes every InitiateSignIn scenario
```
The file was run through rustfmt after this run, so the panic lines are now at :545 and :585.

## 3. Suite and gates (after the cases existed)

Every command set `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-d-refstate CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo test -p ess-conformance --no-fail-fast`: EXIT=101, 1668 passed / 2 failed. Both failures are my red cases, and every committed test passed. Final lines: `error: 1 target failed:` / `` `-p ess-conformance --test adversary_refstate_pass2` ``.
- `cargo xtask generate --check`: EXIT=0, `projections are up to date`.
- `cargo clippy -p ess-conformance --test adversary_refstate_pass2 -- -D warnings`: EXIT=0, `Finished`.

## 4. Findings

| file:line | severity | verdict | origin | finding | what reaches it | fix |
|---|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/related_guard.rs:612 (`without_row`, `.next()` at :642) | blocker | NEEDS-CHANGE | introduced | Correction 1 puts refusal guards ahead of accepting guards, and `without_row` still keeps only the first candidate. The #211 F3/F4 witness (a missing row sent with the input an accepting branch takes) is lost once any input refusal is declared. Class 3 (only the first witnessed): a target taking an accepting input branch before reading the related row now passes the whole suite. At the base, the list held only the accepting guards, so `console` was sent | Any `when_related` command with an input refusal and an accepting `when:` branch. That is the sign-in shape plus a blank-client check, and the second adopter's comment on #227 asks for it | Send one missing-row request per guard (each refusal and each accepting guard), each requiring `exists: false`. Do not stop at the first |

## 5. Attacked and could not break

- The interpreter and synthesis agree on all 8 combinations: no Failed or Error. Unsupported results come only from the interpreted target's view reads, and the refusal outcome checks inside them pass.
- Refusals are answered first-declared everywhere I looked: the interpreter (`refused_by_input`), validation (`subject_state.rs:355` `min()`), synthesis (`selected_in_state`, `related_guard::selects`, `sibling_refusals`). The pass-1 case and the #227 held-state case for this are green.
- Switching overlapping refusals from "conflict" to "first declared" drops no requirement. A fully shadowed later refusal, and an accepting branch fully shadowed by a refusal, are refused by synthesis under their scenario id (ESS-SYNTH-003), as #217 does for a shadowed external. Validation admits them, which matches #217's rule.
- `subject_state.rs:289` fallback: only undecidable refusals leave the proof. Held-state models beside the undecidable `too-short` validate and synthesize without refusals.
- On when_related + existing_instance + refusal, the input refusal `bad-note` is refused by name (ESS-SYNTH-008 `arrange_related_row`), not dropped, and the interpreter declines the command as documented.
- Byte stability: `generate --check` is clean.
- Entity Runtime: not re-attacked. `when_related` is not lowered, and the ER pin test is the unit's.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/refstate/adv2/review.md
- ~/.cache/ess-wave-n2/refstate/adv2/red-alone.log
- ~/.cache/ess-wave-n2/refstate/adv2/red-alone-2.log
- ~/.cache/ess-wave-n2/refstate/adv2/red-alone-final.log
- ~/.cache/ess-wave-n2/refstate/adv2/table.log
- ~/.cache/ess-wave-n2/refstate/adv2/conformance-suite.log
- ~/.cache/ess-wave-n2/refstate/adv2/generate-check.log
- ~/.cache/ess-wave-n2/refstate/adv2/clippy.log
- ~/.cache/claude-tmp/claude-1000/-home-timo-beyond10x/c8c080b7-d6ba-44b3-a1f1-53496d986908/tasks/blm0b43ct.output and bctlum9o0.output (harness background-task output)
- The build went into the assigned ~/.cache/b10x-target/ess-d-refstate; no new directory was created.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/related_guard.rs
  line: 612
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: without_row keeps only the first guard's witness and now prefers a refusal's, so beside an input refusal the accepting branch's missing-row overlap (#211 F3/F4) is no longer sent and a target taking the accepting branch before reading the related row passes
```
