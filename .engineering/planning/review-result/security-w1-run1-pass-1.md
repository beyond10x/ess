---
format: aep.planning-md/3
id: review-result:security-w1-run1-pass-1
kind: review-result
status: active
title: 'W1 RUN1 security pass 1: authored success asserts no error'
summary: 'NEEDS-CHANGE: the CHANGELOG remedy must also name run_admitted (introduced); one pre-existing item to the backlog'
relations:
- reviews: story:authored-success-step-asserts-no-error
revision: 1
---
unit: W1 RUN1 (story:authored-success-step-asserts-no-error), 029c8529d and 819b869f1 merged as 5a5ee4797, reviewed in tree ess-w1-run1 at 819b869f1
verdict: NEEDS-CHANGE (one CHANGELOG line)
cases: executed 314→320, red 2
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: the wave scratch directory run1/security-scratch (cargo and suite logs)
needs-coordinator: none

**Findings.**
- **S1** (warning, introduced, contract drift): `CHANGELOG.md:7` tells a library caller building an authored
  suite by hand to call `select_fresh_format_for`; a `Runner::run` caller is then refused at `/34`
  (`runner.rs:391`, `UnsupportedReportFormat`), so the remedy must also name `run_admitted`. Red case
  `w1run1_security_the_stated_remedy_keeps_a_legacy_runner_caller_running`. Back to the implementor.
- **S2** (note, pre-existing, out of scope): the same success-act-with-error item as the adversary's F2;
  backlog draft `story:an-authored-success-act-claiming-an-error-is-refused`.

**Attacked, not broken.** An awaited success and an external success answered with an error both fail on
`no error`; at `/4` the new step is refused loudly by `from_suite` and `Runner::run`; a refusing act still
runs at `/4`; every CLI path selects the format. Peak target/ 8G, lowest / 106G.
