---
format: aep.planning-md/3
id: review-result:adversary-d-counter-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit counter
relations:
- reviews: story:a-counter-at-its-limit-is-witnessed
revision: 1
---
unit: counter-limit (#226), working tree ~/.local/state/worktree/trees/b10x/ess/ess-d-counter (uncommitted diff on db92e9fa9)
verdict: NEEDS-CHANGE
cases: executed 1652→1659, red 4
origin: introduced 3 / pre-existing 0 / undecided 1
wrote-outside-worktree: 6 paths under ~/.cache/ess-wave-n2/counter/adv1/ plus build dir ~/.cache/b10x-target/ess-d-counter
needs-coordinator: none

## 1. Diff stat (tracked; the only new untracked file is mine plus the implementor's counter_limit.rs)

```
 crates/verify/ess-conformance/src/synthesize.rs    | 132 ++++--
 .../src/synthesize/related_guard.rs                |  22 +-
 .../ess-conformance/src/synthesize/subject_fact.rs | 521 ++++++++++++++++++++-
 3 files changed, 606 insertions(+), 69 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_counter_limit_pass1.rs   (mine, test only)
?? crates/verify/ess-conformance/tests/counter_limit.rs                   (implementor's)
```
No implementation file touched by me.

## 2. Cases added: crates/verify/ess-conformance/tests/adversary_counter_limit_pass1.rs

Every model: `RetryTask` raises `retries` (unguarded), `StartTask` reads the limit, so only synthesized rows decide it.

| case (line) | asserts | now |
|---|---|---|
| an_even_step_toward_an_odd_limit_witnesses_the_reached_row_short_of_it :313 | `{increment: 2}` from 0, `retries >= 3`: clean synthesis, mutants `>=2`/`>=1` killed | RED |
| a_guarded_sibling_answering_one_step_short_is_witnessed :419 | `over: retries >= 5`, `blocked: 3 <= retries < 5`: clean, mutant `over >= 4` killed | RED |
| a_limit_under_not_is_witnessed_one_step_short :454 | `{not: retries < 3}`: mutant `>= 2` killed | RED |
| two_counters_in_one_guard_are_reached_or_refused_naming_the_bound :470 | `{all: [retries >= 5, restarts >= 5]}`: clean, or refused naming the counter bound | RED |
| an_equality_limit_read_by_another_command_witnesses_the_row_past_it :334 | `== 3`, mutants `>=3`, `==2`, `==4` killed | green |
| every_operator_read_by_another_command_is_decided_on_both_sides :358 | `>`, `<`, `<=`, `!=` each vs both off-by-one literals | green |
| an_owner_equality_limit_witnesses_the_row_past_it :727 | owner `open_cards == 3` via when_related, board reading `>= 3` fails | green |

Red output, the file run alone (`cargo test -p ess-conformance --test adversary_counter_limit_pass1 --no-fail-fast`), verbatim:

```
test an_even_step_toward_an_odd_limit_witnesses_the_reached_row_short_of_it ... FAILED
test a_limit_under_not_is_witnessed_one_step_short ... FAILED
test an_owner_equality_limit_witnesses_the_row_past_it ... ok
test a_guarded_sibling_answering_one_step_short_is_witnessed ... FAILED
test an_equality_limit_read_by_another_command_witnesses_the_row_past_it ... ok
test two_counters_in_one_guard_are_reached_or_refused_naming_the_bound ... FAILED
test every_operator_read_by_another_command_is_decided_on_both_sides ... ok
    "ESS-SYNTH-003: refusal[ESS-SYNTH-003]: outcome work.tasks.StartTask/blocked has no scenario `work.tasks.StartTask/outcome/blocked`\n  no candidate of the 11 tried satisfies ``work.tasks.Task` row for `work.tasks.StartTask/blocked` with [] false and [retries >= 3, retries == 3] true, one side of a stored counter's limit: `work.tasks.Task` stored boundary selecting this branch, over the rows 11 bounded arrangements left; a stored counter (`retries`) is followed only within 16 of each literal its guard compares it with`\n  help: write the branch's condition over values a candidate can carry, or supply a fixture for it",
    "ESS-SYNTH-003: refusal[ESS-SYNTH-003]: outcome work.tasks.StartTask/started has no scenario `work.tasks.StartTask/outcome/started`\n  no candidate of the 11 tried satisfies ``work.tasks.Task` row for `work.tasks.StartTask/started` with [retries >= 3] false and [retries == 1] true, one side of a stored counter's limit: `work.tasks.Task` stored boundary selecting this branch, over the rows 11 bounded arrangements left; a stored counter (`retries`) is followed only within 16 of each literal its guard compares it with`\n  help: write the branch's condition over values a candidate can carry, or supply a fixture for it",
mutants survived: ["limit >= 2"]
    "ESS-SYNTH-003: refusal[ESS-SYNTH-003]: outcome work.tasks.StartTask/started has no scenario `work.tasks.StartTask/outcome/started`\n  no candidate of the 23 tried satisfies ``work.tasks.Task` row for `work.tasks.StartTask/started` with [retries >= 5] false and [retries == 4] true, one side of a stored counter's limit: `work.tasks.Task` stored boundary selecting this branch, over the rows 23 bounded arrangements left; a stored counter (`retries`) is followed only within 16 of each literal its guard compares it with`\n  help: write the branch's condition over values a candidate can carry, or supply a fixture for it",
    "ESS-SYNTH-001: refusal[ESS-SYNTH-001]: outcome work.tasks.StartTask/blocked has no scenario `work.tasks.StartTask/outcome/blocked`\n  no witness: `work.tasks.Task.restarts,retries` is `work.tasks.Task`, which subject fact arrangement exceeds 64 lifecycle/fact combinations\n  help: give the field a type that has a finite value, or drop it from the command's input",
test result: FAILED. 3 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

(A first draft carried a `3 <= retries` case; the parser refuses literal-left comparisons, so it was a typo, removed before the recorded run.)

## 3. Suite run (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast` — summed over 231 `test result` lines: passed 1655, failed 4, ignored 3, executed 1659. Only failing binary: `adversary_counter_limit_pass1` (4 of 7). Final lines verbatim:

```
error: 1 target failed:
    `-p ess-conformance --test adversary_counter_limit_pass1`
EXIT=101
```
`executed 1652` = 1659 minus my file's 7 cases (its own binary; no other binary changed).
`cargo xtask generate --check`: `projections are up to date`, EXIT=0.
`cargo clippy -p ess-conformance --test adversary_counter_limit_pass1 -- -D warnings`: EXIT=0 (file-level allow `single_match_else`); rustfmt --check clean.

## 4. Findings

| file:line | sev | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| subject_fact.rs:1585 (`limit_edges`), :2081 | blocker | NEEDS-CHANGE | introduced | `{increment: 2}` from 0 vs `retries >= 3`: the side rows are pinned at `literal ± step` (1, 3), which no run holds; both `blocked` (reachable at 4) and `started` (witnessed at 0 before this diff) are refused, citing "followed only within 16" on a limit 3 away | pin the nearest value the counter can hold on each side (from start + multiples of steps), or treat a parity-unreachable pin as "adds none" |
| subject_fact.rs:2902 (`counter_goals`) | blocker | NEEDS-CHANGE | introduced | disjoint siblings `over >= 5` / `blocked 3..4`: the default gets a side row for `over` at 4, a row `blocked` answers; `started` is refused wholesale with the same false bound claim. The admitted gap "no side row" is a refusal of the default | witness that row under the sibling answering it (the `answering` scheme `related_boundaries` uses), or skip goals another guarded sibling decides |
| subject_fact.rs:2081 (`beyond`) | warning | CONFIRMED | introduced | `beyond` is set whenever any explored row's counter lies more than 16 from every literal. When the raise is unbounded, which covers every model whose raiser is not the guarded command, an exhaustive search always sets it. So every unreachable side row is refused with the bound message, and "unreachable adds none" holds only when the guard stops the raise. This is the shared cause of rows 1 and 2 | set `beyond` only when the goal's pinned value itself lies past the reach |
| subject_fact.rs:2829 (`limit_goals`) | warning | CONFIRMED | introduced | `{not: retries < 3}`: `blocked` now synthesizes clean (before this diff it was unreachable). No short row exists for `started`, so a target blocking from 2 passes the suite. Before, a refusal showed the gap; now the suite is clean and the defect goes undetected | recurse through `not:` (flip `holds`), or refuse the unhandled shape, never a clean suite |
| subject_fact.rs:44 / :2089 | warning | CONFIRMED | undecided | two counters raised by separate commands, `{all: [retries >= 5, restarts >= 5]}` (10 raises, well within reach): refused ESS-SYNTH-001 "exceeds 64 lifecycle/fact combinations" with help "give the field a type that has a finite value". The per-counter key (33 values per literal) is multiplied against MAX_NODES=64. Base behaviour not run | scale the node budget with followed counters, or refuse ESS-SYNTH-003 naming both bounds |
| subject_fact.rs:1531 (`counter_leaf`) | note | INFEASIBLE | introduced | the literal-left arm (`3 <= retries`) is unreachable from source: the parser refuses a literal on the left. Dead, untested code | drop the arm, or cover it if the IR can produce it |

What reaches rows 1, 2 and 4: an ordinary authored model. The counter is raised by a command other than the one reading it, which is the `StartTask/blocked` shape of the issue itself. `{increment: 2}` and two error branches at different limits are plain modelling choices. Row 5 needs two counters in one guard. I found nothing showing a committed model uses these shapes; `generate --check` is clean.

## 5. Attacked, not broken

- `>`, `<`, `<=`, `!=`, `==` read by a non-raising command, each against both off-by-one literals: all killed.
- `== 3` mutated to `>= 3` (row past the limit), subject and owner (`when_related`): killed.
- Models without counter guards: `cargo xtask generate --check` is clean, so no bytes changed.

Not attacked (disk at 11G): linkguard or #201 state-scoped interaction, `sets:` literal reset, `increment` of a counter created undetermined.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/counter/adv1/board_part.rs (extracted harness scratch)
- ~/.cache/ess-wave-n2/counter/adv1/case-run.log
- ~/.cache/ess-wave-n2/counter/adv1/suite-run.log
- ~/.cache/ess-wave-n2/counter/adv1/generate.log
- ~/.cache/ess-wave-n2/counter/adv1/clippy.log
- ~/.cache/ess-wave-n2/counter/adv1/review.md
- build dir ~/.cache/b10x-target/ess-d-counter (assigned; shared with the implementor)

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 1585
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a step of 2 toward an odd limit pins side rows no run holds, and both branches of a reachable model are refused with a false bound claim, including a default witnessed before the diff
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2902
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the default's side row for a sibling limit that another guarded sibling answers is unreachable as the default, so the default branch is refused wholesale
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2081
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: beyond is set by any explored row past the reach, so with an unbounded raise every unreachable side row is refused as past the 16 bound instead of adding none
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2829
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a limit under not is now synthesized clean with no short row, so a target with the limit one lower passes the suite
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 44
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: two counters within reach exhaust the 64-node budget and are refused ESS-SYNTH-001 with a finite-type repair hint instead of being witnessed or refused naming the counter bound
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 1531
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the literal-left arm of counter_leaf is unreachable because the parser refuses a literal on the left of a comparison
```
