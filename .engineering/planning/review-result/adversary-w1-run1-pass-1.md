---
format: aep.planning-md/3
id: review-result:adversary-w1-run1-pass-1
kind: review-result
status: active
title: 'W1 RUN1 adversary pass 1: authored success asserts no error'
summary: 'NEEDS-CHANGE: six frozen coverage-producer oracles lack expect_no_error (introduced); one pre-existing item to the backlog'
relations:
- reviews: story:authored-success-step-asserts-no-error
revision: 1
---
unit: W1 RUN1 (story:authored-success-step-asserts-no-error), 029c8529d and 819b869f1 merged as 5a5ee4797, reviewed in tree ess-w1-run1 at 819b869f1
verdict: NEEDS-CHANGE (1 blocker)
cases: executed 324→328, red 3
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: the wave scratch directory run1/adversary-scratch (cargo log)
needs-coordinator: yes (frozen coverage-producer oracles)

**Findings.**
- **F1** (blocker, introduced): `crates/edge/ess-cli/tests/support/coverage_producer.rs:355` (`check_structure`)
  compares fresh authored scenario definitions in full with six frozen `legacy-observations` oracles
  (`author-single`, `author-rank`, `author-accepted-duplicate`, `author-refused-then-accepted`,
  `billing-combined`, `billing-repeated-gap-combined`, `-suite4.json`) that lack the new `expect_no_error`,
  so the three `coverage_producers` tests fail. Red cases
  `the_cli_coverage_producer_oracle_for_the_single_authored_act_still_matches` and `..._rank_...`; a green
  control without the step pins the cause. Coordinator: the oracles move; back to the implementor.
- **F2** (note, pre-existing, out of scope): a success act that also claims `error:` is not refused
  (`src/authored.rs:4261`) and now compiles to a scenario no target can pass. Filed as backlog draft
  `story:an-authored-success-act-claiming-an-error-is-refused` (D398).

**Attacked, not broken.** Refusing acts keep their shape; awaited acts read the awaited answer; every CLI
suite builder selects the format; other committed authored JSON is frozen input never recompiled. Peak
target/ 8G, lowest / 106G.
