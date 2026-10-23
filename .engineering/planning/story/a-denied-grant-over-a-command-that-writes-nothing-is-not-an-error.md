---
format: aep.planning-md/3
id: story:a-denied-grant-over-a-command-that-writes-nothing-is-not-an-error
kind: story
status: draft
title: A denied grant over a command that writes nothing is not an error
summary: 'Adopter report S7: grant/denied over a command that writes nothing errors (ess#655).'
refs:
- provider: github
  reference: beyond10x/ess#655
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

For a command with `accepts: nothing` beside a `read_your_writes` view, the synthesized `grant/denied` scenario emits `expect_view_unchanged` with no consistent query before it, and the run reports `ESS-CF-SUITE` errors (3 of 169 in an adopter's model). Adopter report S7, ess 0.68.0. Recheck against the merged denied-grant read-token fix (commit 9bd955683) before starting.

## Acceptance

- `a_denied_grant_over_a_command_writing_nothing_passes`: the S7 reproduction's `Check/grant/denied` passes on the interpreted target; the run reports 4 passed, 0 errors.
- `a_denied_grant_over_a_writing_command_still_reads_the_view`: `CreateNote/grant/denied` keeps its before-and-after view check.
- `no_suite_step_reads_an_unqueried_view`: no synthesized suite in the fixture table contains `expect_view_unchanged` without a preceding consistent query.
