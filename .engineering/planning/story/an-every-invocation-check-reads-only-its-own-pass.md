---
format: aep.planning-md/3
id: story:an-every-invocation-check-reads-only-its-own-pass
kind: story
status: draft
title: An every-invocation check in a swapped-caller pass reads only its own pass
summary: 'Inferred from the 0.72.0 gap Q fix: the forced-outcome cell''s every-invocation check spans both passes of a swapped caller.'
tags:
- ess-0.74.0
relations:
- informed_by: approval-record:ess-0720-hold-for-gap-q
revision: 1
---
## Problem

Found while fixing the 0.72.0 swapped-caller count (`approval-record:ess-0720-hold-for-gap-q`).
That fix makes the swapped pass's `expect_invocation` count cumulative over the scenario, because
the runners count every invocation since the scenario started.

The forced-outcome cell in `synthesize/binding_effects.rs` (`ExpectEveryInvocation` with an empty
`selecting` and the mapped `input`) has the same window: it checks every invocation of the binding
since the scenario started. If that scenario is appended to a swapped-caller pass, the second
pass's check also reads the first pass's invocation. Where the two passes map different input
values, the check would fail on the interpreted target. This is inferred from the code only; no
fixture has shown it yet.

## Acceptance

- `an_every_invocation_check_reads_only_its_own_pass`: a neutral fixture whose binding has a
  forced-outcome cell under a swapped caller, with input values that differ between the passes,
  either synthesizes a scenario that passes the interpreted target or shows that no such suite is
  synthesized (in which case the story closes with that test as its evidence).
