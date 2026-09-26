---
format: aep.planning-md/2
id: story:execution-recovery-indeterminate-case-holds-under-load
kind: story
status: draft
title: execution_recovery indeterminate-effect case passes under machine load
relations:
- serves: vision:O2
revision: 1
---
# execution_recovery indeterminate-effect case passes under machine load

## Outcome

`an_effect_before_failure_and_a_lost_acknowledgement_are_both_indeterminate` (`crates/edge/ess-cli/tests/execution_recovery.rs`) passes on every run at the load a shared build machine sees.

## Why

On 2026-09-26 it failed once in a full `task check` at load 8-24: the child exit status was `None` where the test expected `Some(1)`. Rerun alone three times it passed 118/118. A flake on main blocks the bot merge queue.

## Acceptance

- The mechanism that yields `None` is named and removed, not hidden by a longer timeout.
- 200 runs under load, 0 failures; the test still fails when the behaviour it guards is broken.
