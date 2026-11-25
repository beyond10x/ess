---
format: aep.planning-md/3
id: story:the-targeted-gate-of-a-patch-release-ends-within-its-job-limit-whichever-crates-the-change-selects
kind: story
status: draft
title: The targeted gate of a patch release ends within its job limit whichever crates the change selects
summary: 0.99.3's Hotfix tests job ran 2,734 of 8,685 tests in 46 minutes and was cancelled at its 60-minute limit; the scope script selected ess-cli, ess-conformance and ess-synth
tags:
- ci
- release
relations:
- serves: vision:O2
- informed_by: story:tests-that-run-over-60-s-on-main-without-a-branch-of-the-week-run-within-60-s-or-move-to-the-slow-lane
revision: 1
---
## Finding

The first patch release that ran the targeted gate (0.99.3, a change in `ess-conformance`) had its
`Hotfix tests` job cancelled at `timeout-minutes: 60` (`.github/workflows/release.yml`, job
`hotfix-tests`). `.github/scripts/hotfix-scope.sh` selected `billing-realization ess-cli
ess-conformance ess-diff ess-synth ess-xtask gatepass-realization` (the changed crate and its
direct dependents). The job found no compile cache, started 8,685 tests in 1,188 binaries on one
runner 14 minutes after the step began, and had run 2,734 of them when the limit ended it 46
minutes later; no test had failed. The same job was skipped in 0.98.2, 0.97.4 and 0.97.3 (their
gates were reused or full), so the limit was never met before. The full gate deals the same tests
over 20 shards.

## Outcome

A patch release whose change selects any set of crates gets a gate that ends inside its job
limits, or the full gate.

## Acceptance

1. The scope script's test names both sides: a change that stays targeted, and a change in
   `ess-conformance`, `ess-cli` or `ess-synth` that does not.
2. `ci_lanes.rs` holds the rule.
3. `AGENTS.md` § Hotfix says which changes take the full gate.
4. The measured time of the targeted job for the largest set that stays targeted is in this
   story.

## Decisions

- HG-D1 (open, the coordinator's):
  - A: the scope script answers `full` when the selected set holds `ess-cli`, `ess-conformance`
    or `ess-synth` (smallest change; such a patch release costs a full run).
  - B: the targeted tests are dealt over several shards from one archive like the full gate (more
    workflow, keeps the narrower set).
  - C: a longer job limit (not a fix under the test-budget rule).
  - Recommended: A.

## Not covered

The compile cache of a tag run (tags save none by design).
