---
format: aep.planning-md/3
id: review-result:adversary-d-refstate-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit refstate
relations:
- reviews: story:an-input-refusal-sits-beside-a-subject-state-branch
revision: 1
---
unit: refusal-beside-state (#227), worktree ess-d-refstate, uncommitted diff on db92e9fa9 plus 2 untracked adversary test files
verdict: NEEDS-CHANGE
cases: executed 2697→2705 (ess-domain 1039→1044, ess-conformance 1658→1661), red 3
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: 4 paths (list in part 6)
needs-coordinator: whether two overlapping input refusals are answered first-declared (Entity Runtime does this) or both (the interpreter and the unit's doc do this)

## 1. Diff stat

`git --no-pager diff --stat` is the unit's own tracked diff, unchanged: 7 files, +460/−21. My additions are untracked test files only:

- `crates/specify/ess-domain/tests/adversary_refstate_pass1.rs`
- `crates/verify/ess-conformance/tests/adversary_refstate_pass1.rs`

No non-test path was touched.

## 2. Cases added (each run alone before its suite)

| file:test | asserts | now |
|---|---|---|
| ess-conformance `adversary_refstate_pass1.rs:65` `the_interpreter_does_not_answer_an_input_refusal_before_a_missing_related_row` | sign-in fixture plus `blank-client: when: client == ""`, unconfigured tenant, blank client: interpreter answers `no-configuration` or declines, never `blank-client` | RED |
| ess-conformance `:134` `the_interpreter_answers_the_first_declared_of_two_overlapping_refusals_as_entity_runtime_does` | held-state model with `frozen` and `frozen-again` (both `mode == Freeze`): one answer, `frozen` | RED |
| ess-conformance `the_blank_client_scenario_passes_a_target_answering_a_missing_row_first` | synthesized `blank-client` scenario passes a target answering in #211 order | green (synthesis agrees with #211; only the interpreter disagrees) |
| ess-domain `adversary_refstate_pass1.rs:65` `an_undecidable_refusal_beside_them_does_not_hide_that_conflict` | `too-short` (undecidable) beside `frozen` and `frozen-again` is still `conflicting_declaration` | RED |
| ess-domain `control_two_decided_refusals_claiming_one_input_are_refused` | control without `too-short`: refused | green |
| ess-domain `a_subjectless_refusal_reading_state_through_when_is_still_refused`, `..._stored_only_field_...` | `when: state == …` and `when:` over a stored-only field are still refused | green |
| ess-domain `a_state_gap_outside_the_refusal_is_still_refused` | with no default, both the undecidable and the decidable refusal leave `Pending`/`Revoked` uncovered and are refused | green |

Red output, verbatim, from each case run alone:

```
thread 'the_interpreter_does_not_answer_an_input_refusal_before_a_missing_related_row' (675108) panicked at crates/verify/ess-conformance/tests/adversary_refstate_pass1.rs:91:9:
the interpreter answered ["demo.signin.InitiateSignIn/blank-client"] for a tenant with no configuration; #211 precedence says `no-configuration` answers a missing row whatever the input
```
```
thread 'the_interpreter_answers_the_first_declared_of_two_overlapping_refusals_as_entity_runtime_does' (698337) panicked at crates/verify/ess-conformance/tests/adversary_refstate_pass1.rs:165:5:
assertion `left == right` failed: ["demo.secrets.RotateSecret/frozen", "demo.secrets.RotateSecret/frozen-again"]
  left: ["demo.secrets.RotateSecret/frozen", "demo.secrets.RotateSecret/frozen-again"]
 right: ["demo.secrets.RotateSecret/frozen"]
```
```
thread 'an_undecidable_refusal_beside_them_does_not_hide_that_conflict' (650362) panicked at crates/specify/ess-domain/tests/adversary_refstate_pass1.rs:72:18:
validated: `frozen` and `frozen-again` both claim `mode == Freeze`, and the undecidable `too-short` made the prover drop both from the partition
```

## 3. Suite runs (after the cases existed)

Each command ran with `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-d-refstate CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo test -p ess-domain --no-fail-fast`: exit 101, 1042 passed / 1 failed. The failure is `an_undecidable_refusal_beside_them_does_not_hide_that_conflict`. This run predates `a_state_gap_outside_the_refusal_is_still_refused`; that file re-run alone gives `test result: FAILED. 4 passed; 1 failed`. Final lines: `error: 1 target failed:` / `` `-p ess-domain --test adversary_refstate_pass1` ``.
- `cargo test -p ess-conformance --no-fail-fast`: exit 101, 1659 passed / 2 failed. Both failures are mine. Every committed test passed, so no existing suite's verdict changed because of the interpreter change. Final lines: `error: 1 target failed:` / `` `-p ess-conformance --test adversary_refstate_pass1` ``.
- `cargo xtask generate --check`: exit 0, `projections are up to date`.
- `cargo clippy -p ess-domain --test adversary_refstate_pass1 -- -D warnings` and the same for `-p ess-conformance`: `Finished`, no warnings.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/interpret/execute.rs:342 | blocker | NEEDS-CHANGE | introduced | `refused_by_input` runs before `interpretable` on every command, including a `when_related:` command. There it answers an input refusal for a missing related row. The #211 precedence (cross-record-and-stored-field-guards.md:527, decisions #211 pass 1) says `exists: false` answers first, "whatever the input". Before this unit that command was `NotInterpreted`. Synthesis follows #211 (green case), so the interpreter now contradicts both the doc and the suite | in `refused_by_input`, return `None` when the command has a `Related` branch, or answer `exists: false` first. The narrowest fix is to apply the early answer only to conditions `interpretable` refuses that the precedence places after input refusals (held state, stored fields, existence) |
| crates/specify/ess-domain/src/command/subject_state.rs:283 | warning | NEEDS-CHANGE | introduced | The `analyze_partition` fallback drops **every** input refusal once any one is undecidable. Two decidable refusals claiming one input are refused alone (the unit's own `two_refusals_the_same_input_selects_are_still_refused`) but validate when an unrelated `secret.count < 12` refusal sits beside them. outcome-shapes.md:298 ("two refusals claiming one input are still two answers") is then not enforced | on fallback, drop only the refusals the prover cannot decide and keep the decidable ones in `rest`. Retry with the undecidable ones removed one at a time if needed |
| crates/verify/ess-conformance/src/interpret/execute.rs:438 | warning | CONFIRMED | introduced | Two overlapping input refusals: the interpreter returns both ("the model orders neither first"), while Entity Runtime sorts refusals by declaration and takes the first (`ess-entity-runtime/src/lib.rs:1476-1491`), which is `frozen`. The brief states the #217 precedence as "refusals first, then declaration order". What reaches it: the held-state model above validates only because of finding 2; on plain commands, overlapping decidable refusals already validate (probed, see part 5) | coordinator decides. If first-declared, `refused_by_input` returns the first refusal whose guard holds, and outcome-shapes.md:298 and input-guard-overlap-precedence.md:110 change with it |
| docs/design/cross-record-and-stored-field-guards.md:527 | note | CONFIRMED | pre-existing | The documented precedences form a cycle. An input refusal answers before `existing_instance` (#209, outcome-shapes.md:268). `existing_instance` answers before `exists: false` (#211). `exists: false` answers before "an input-guarded branch" (#211). A command with all three has no documented order. The documents are dated before this unit | state which one wins in one place |

## 5. Attacked and could not break

- `when_related` beside `when_subject_state`: still refused. `related_guard::validate_shape` refuses it on its own, before `subject_state`.
- `wrong_state:` beside `when_subject_state`: still `conflicting_declaration` (`subject_state.rs` validate_shape, unchanged).
- A subjectless `when:` reading `state` or a stored-only field: still refused (green cases).
- Coverage gap through the fallback: none. The rest must cover every input alone, and the joint path still reports a gap in any state with no default (green case).
- Interpreter change on plain commands: all 1658 committed ess-conformance tests pass. The change turns a refusal overlapping an external branch from "forced external wins" into "refusal wins", which matches input-guard-overlap-precedence.md:33.
- Synthesis for the issue model and the state-change, stored-field and guarded-sibling variants is covered by the unit's mutant targets (StateFirst, ExistenceFirst, LaxIn per state, SiblingFirst, WritesOnRefusal, EmitsOnRefusal). Reading the code, I found no dropped state and no reused witness.
- Byte stability: `cargo xtask generate --check` is clean.
- Origin probe, not kept as a case: on a plain command, `frozen` and `frozen-again` validate with or without `too-short`. That output was `plain command validated with frozen/frozen-again overlapping` from both probes. So refusal-vs-refusal overlap is admitted there before this unit.
- Not attacked: `input_absent` beside an input refusal. The interpreter has no representation of an absent body that I could find.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/refstate/adv1/review.md
- ~/.cache/ess-wave-n2/refstate/adv1/domain-suite.log
- ~/.cache/ess-wave-n2/refstate/adv1/conformance-suite.log
- ~/.cache/ess-wave-n2/refstate/adv1/generate-check.log
- The build output went into the assigned `~/.cache/b10x-target/ess-d-refstate`; no new directory was created.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 342
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: refused_by_input answers an input refusal before a missing related row on a when_related command, against the #211 precedence that exists-false answers first whatever the input, where the interpreter declined before
- file: crates/specify/ess-domain/src/command/subject_state.rs
  line: 283
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the partition fallback drops decidable input refusals together with an undecidable one, so two refusals claiming one input validate once an unrelated undecidable refusal is declared beside them
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 438
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: two overlapping input refusals are both returned by the interpreter while Entity Runtime takes the first declared
- file: docs/design/cross-record-and-stored-field-guards.md
  line: 527
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: the #209 and #211 precedences form a cycle between input refusal, existing_instance and exists-false with no documented winner
```
