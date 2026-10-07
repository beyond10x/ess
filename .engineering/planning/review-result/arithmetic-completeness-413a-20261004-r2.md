---
format: aep.planning-md/3
id: review-result:arithmetic-completeness-413a-20261004-r2
kind: review-result
status: active
title: 'Final arithmetic review: green, two weak author tests'
relations:
- reviews: story:counter-reachability-arithmetic-completeness
revision: 1
---
unit: 413a-arithmetic-completeness
verdict: green
cases: executed 17→59, red 0
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: review scratch only (append.rs, env.sh, run1–run4 logs) plus test binaries in the unit's build directory
needs-coordinator: yes — two untracked review test files each carried a full copy of counter_limit.rs; the report file write was refused by the harness, so the agent's final message is the report

Publication copy of the second and final whole review of the #413A arithmetic unit. Workstation path
prefixes are replaced by `<worktrees>` and `<cache>`; nothing else is changed.

Tree reviewed: `<worktrees>/ess/wt-5794b43839af`, HEAD `ff17161b4`, plus the uncommitted two-file unit
diff. Final hashes: subject_fact.rs `cb3b9fb0…dc20`, counter_limit.rs `26e00ff6…bc44` (both equal the
refresh freeze).

## Cases added (written before anything ran)

| case | asserts | now |
|---|---|---|
| probe file (SHA `96346e1e…7436`, byte-identical to pass 1's): `independent_extreme_counter_refusals_do_not_claim_a_complete_nearest_value` | at MAX (+1) and MIN (−1): some refusal, no "nearest value it holds" claim, no EstablishEntity, RETRIED kept, healthy target passes | green |
| `review2_ordinary_and_shared_related_digests_equal_the_recorded_baseline` | canonical sha256 equals the story's literals `6bf91d8f…c26c5e` and `8f56c982…c96958` | green |
| `review2_decrement_away_from_max_claims_only_values_it_holds` | step −1 counter with guard `>= MAX`: every nearest-value claim is ≤ 0, RETRIED kept, healthy target passes | green |
| `review2_decrement_away_from_max_does_not_blame_the_reach` | the at-limit refusal does not say "within 16" | green |
| `review2_increment_away_from_min_*` (2 cases) | the MIN mirror of the two cases above | green |
| `review2_literal_past_i64_makes_no_completeness_claim` | literal `2^63`: no nearest-value claim, no seed, healthy target passes | green |
| `review2_related_owner_counter_at_max_makes_no_completeness_claim` | BOARD with `open_cards >= MAX`: `AddCard/full` refused, no nearest-value claim, healthy Board passes | green |

## Runs (brief's environment, `--locked --offline`)

| run | target | exit | counts |
|---|---|---|---|
| run1 | adversary file | 101 | compile error in the reviewer's own `&dyn` usage — not a finding |
| run2 | adversary file, `review2_` filter | 0 | 7 passed / 0 failed / 17 filtered out |
| run3 | probe file, the one case `--exact` | 0 | 1 passed; printed `upper: refused=2 scenarios=2`, `lower: refused=2 scenarios=2` |
| run4 | `counter_limit` + the two new targets, unfiltered, `--no-fail-fast` | 0 | 17 + 24 + 18 = 59 passed / 0 failed / 0 ignored |

## Findings

F1. `crates/verify/ess-conformance/tests/counter_limit.rs:1187` — the canonical-bytes control only
compares two compilations by the same code and prints the hash; it stays green under any deterministic
change to the suite bytes. The reviewer's literal-digest test is green today. No mutated copy was run.

F2. `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:5954` — the widest-negative test was
already green on the base and refuses through padding overflow. `negated` never returns `None`
(`decimal_literal` re-reads both number texts, `facts.rs:283-298`), so the `collect::<Option<_>>` change
at :2540 is unobservable; reverting it to `filter_map` leaves every test green. What reaches it: nothing
found. Harmless as a defensive measure.

## Attacked, could not break

MAX and MIN counters moving toward the limit; counters moving away from MAX/MIN (the base returned a
complete and correct `{-1,0}`, the new code takes the fallback path and stays honest); literal `2^63`
(base `reachable` also returns `None` there, read from `git show ff17161b4`, not run); shared related
counter at MAX; padding overflow, start at MAX, zero step and the frontier `filter_map` at :2570
(already correct on the base).

```findings
[
  {
    "file": "crates/verify/ess-conformance/tests/counter_limit.rs",
    "line": 1187,
    "category": "mutant",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "The canonical-bytes control only compares two compilations by the same code and prints the hash, so it stays green under any deterministic change to the ordinary/shared-related suite bytes; the recorded baseline digests are not asserted."
  },
  {
    "file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs",
    "line": 5954,
    "category": "mutant",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "The widest-negative-magnitude test passes through padding overflow and was green on base; negated never returns None, so the collect::<Option<_>> change at :2540 is unobservable and reverting it to filter_map leaves every test green."
  }
]
```
