---
format: aep.planning-md/3
id: story:task-ci-lint-ends-within-the-120-second-step-or-splits-into-steps-that-each-do
kind: story
status: draft
title: task ci-lint ends within the 120 second step, or splits into steps that each do
summary: 'A step-budget defect: task ci-lint took 177 s in a gate run of 2026-11-24; its compile and run are separated in the log and each step is held to 120 s'
tags:
- defect
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- informed_by: story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step
revision: 1
---
## Summary

`task ci-lint` took 177 s in a gate run of 2026-11-24 (a batch's gate log, a single reading, load
not recorded), over the 120 s step budget of `AGENTS.md` § Tests stay fast. CI's `checks` job runs
it as one of its steps beside `ci-smoke`, `test-feature-off-doc`, `test-xtask` and `fuzz-check`.
Whether the time is compile or run is not known; the first step is to split the two in the log.

## Acceptance

1. `task ci-lint` ends within 120 s on a warm build at a 1-minute load of 16 or less, or it is
   split into steps that each do, and the report names each step's time.
2. Its compile and its run are separate commands in the log (as
   `story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step`
   does for `test-xtask`), so a slow step names its phase.
3. `ci_lanes.rs` still finds every step of `check` in some job after a split.

## Fit review

1. **Class.** A step-budget defect, one task. `task test-xtask`'s 172 to 187 s is the same
   shape and is held by the border story, amended on 2026-11-25 with those figures.
2. **Not covered.** No story names `ci-lint`'s wall time.
3. **Borders.** The parent `story:the-test-step-meets-the-test-budget`; the `test-xtask` story above.

## Decisions

- TB-D3, default A: when the time is compile, A builds once through the shared compile cache and
  runs separately (the border story's form); B splits `ci-lint` into two tasks that CI runs as two
  steps. Recommended A. Not architecture and not a security boundary; taken on default if
  unanswered.

## Scope

- `Taskfile.yml` (`ci-lint`), `.github/workflows/ci.yml` only if a step is split,
  `crates/edge/ess-xtask/tests/ci_lanes.rs`.
