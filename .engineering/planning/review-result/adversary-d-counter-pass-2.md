---
format: aep.planning-md/3
id: review-result:adversary-d-counter-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit counter
relations:
- reviews: story:a-counter-at-its-limit-is-witnessed
revision: 1
---
unit: counter-limit (#226) after correction 1, working tree ~/.local/state/worktree/trees/b10x/ess/ess-d-counter (uncommitted diff on db92e9fa9)
verdict: NEEDS-CHANGE
cases: executed 1663→1669, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths under ~/.cache/ess-wave-n2/counter/adv2/ plus build dir ~/.cache/b10x-target/ess-d-counter
needs-coordinator: none

## 1. Diff stat

```
 crates/verify/ess-conformance/src/synthesize.rs    | 170 ++--
 .../src/synthesize/related_guard.rs                |  25 +-
 .../ess-conformance/src/synthesize/subject_fact.rs | 879 ++++++++++++++++++++-
 3 files changed, 996 insertions(+), 78 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_counter_limit_pass1.rs   (adversary pass 1)
?? crates/verify/ess-conformance/tests/adversary_counter_limit_pass2.rs   (mine, test only)
?? crates/verify/ess-conformance/tests/counter_limit.rs                   (implementor's)
```
I touched no implementation file. The tracked diff is the implementor's.

## 2. Cases added: crates/verify/ess-conformance/tests/adversary_counter_limit_pass2.rs

| case (line) | asserts | now |
|---|---|---|
| a_second_creation_past_the_limit_does_not_hide_the_bound :378 | two creators, `AImportTask` at 40 and `CreateTask` at 0, step 1, `retries >= 30`. Must either be refused naming the bound, or kill the mutants `>=29` and `>=31` | RED |
| a_counter_started_from_an_input_is_decided_on_both_sides | `retries: input.start` (one-step fallback), `>= 3`: kills `>=2` and `>=4` | green |
| a_reset_to_a_literal_mid_run_pins_the_rows_it_reaches | step 3 plus reset to 1, `>= 5`: kills `>=4` and `>=7` | green |
| two_raising_commands_with_different_steps_are_decided_on_both_sides | +3 and +1, `>= 10`: kills `>=9` and `>=11` | green |
| an_increment_and_a_decrement_on_one_counter_are_decided_on_both_sides | +2 and -1, `>= 5`: kills `>=4` and `>=6` | green |
| a_nested_not_mixing_a_counter_with_another_leaf_is_decided_on_both_sides | `{all: [{not: {any: [retries < 3, label == "x"]}}]}`: kills `>=2` and `>=4` | green |

Red output, from running the file alone (`cargo test -p ess-conformance --test adversary_counter_limit_pass2 --no-fail-fast`), verbatim:

```
test a_second_creation_past_the_limit_does_not_hide_the_bound ... FAILED
thread 'a_second_creation_past_the_limit_does_not_hide_the_bound' (808688) panicked at crates/verify/ess-conformance/tests/adversary_counter_limit_pass2.rs:400:5:
neither refused naming the bound nor decided: survived ["limit >= 29", "limit >= 31"], refusals []
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
EXIT=101
```

## 3. Suite run (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast`, summed over `test result` lines: passed 1668, failed 1, ignored 3, executed 1669. The only failure is the pass-2 case above, and all 7 pass-1 cases are now green. Final lines, verbatim:

```
error: 1 target failed:
    `-p ess-conformance --test adversary_counter_limit_pass2`
EXIT=101
```
`executed 1663` is 1669 minus this file's 6 cases (its own binary).
`cargo xtask generate --check`: `projections are up to date`, EXIT=0.
`cargo clippy -p ess-conformance --test adversary_counter_limit_pass2 -- -D warnings`: EXIT=0 (file-level allow `format_push_string`). `rustfmt --check`: clean.

## 4. Findings

| file:line | sev | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| subject_fact.rs:2238 (`search_within`), consumed at :3259 (`left_unwitnessed`) | blocker | NEEDS-CHANGE | introduced | Setup: two creators, and the one searched first (by command name) starts past the limit. Its search ends in a plain GuardUnsatisfiable. The creator that would need more than 16 raises sets `beyond`, but `search_within` returns the first creator's cause. `left_unwitnessed` then reads "every row left, none holds it", so both side rows (29 and 30) are dropped silently. The suite is clean, and targets off by one either way pass. With `CreateTask` alone the same limit is refused naming the bound, so renaming a command changes a refusal into a silent drop (class 2). | Prefer a beyond-reach cause over any other when choosing among creators' refusals (or refuse the limit row whenever any creator's search was beyond reach). |

What was measured: `adversary_counter_limit_pass2.rs:400`. Both mutants survived, with 0 refusals.
What reaches it: an ordinary model with a second creating command (an import or a clone) whose literal start lies past the limit. I did not find a committed model with this shape; `generate --check` is clean.

## 5. Attacked, not broken

- `reachable` with two raising commands (+3/+1), a reset to a literal mid-run (+3, reset 1), increment plus decrement (+2/-1), and a start from an input (one-step fallback): all off-by-one mutants killed.
- `positive` over `not: any:` inside `all:`, mixing the counter with a String leaf: both mutants killed.
- Every pass-1 red case (odd limit with step 2, sibling band, `not:` limit, two counters) is now green.
- `== 3` with step 2 (branch dead): refused under `StartTask/blocked` without claiming the bound, and the side-past-reach refusal keeps the branch's own scenario. Both are covered by the implementor's `counter_limit.rs:532/:560`, which I read and did not duplicate.
- Byte stability: `generate --check` is clean.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/counter/adv2/case-run.log
- ~/.cache/ess-wave-n2/counter/adv2/suite-run.log
- ~/.cache/ess-wave-n2/counter/adv2/generate.log
- ~/.cache/ess-wave-n2/counter/adv2/clippy.log
- ~/.cache/ess-wave-n2/counter/adv2/review.md (this file)
- build dir ~/.cache/b10x-target/ess-d-counter (assigned, shared)

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2238
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: with two creators, the first creator's non-bound refusal hides a later creator's beyond-reach search, so both side rows of a limit past the reach are dropped silently and off-by-one targets pass
```
