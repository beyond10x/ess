---
format: aep.planning-md/3
id: story:slow-probe-244b-follows-intended-moves
kind: story
status: draft
title: 'The #244 window-free byte probe admits suite moves a later release intends'
tags:
- ci
- ess-0.54.0
revision: 1
---
# The #244 window-free byte probe admits suite moves a later release intends

## Acceptance

`task test-slow-probes` passes on the 0.54.0 candidate: the
`adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a` probe either re-pins
its base digests to the 0.53.0 tag or names each intended move with the issue that made it, and a
suite move no issue names still fails it.

## Evidence

Local 0.54.0 gate on `6baa31d5d`, step `slow-probes` (exit 101): three entries
moved, IR, refusals and mutants unchanged, suite bytes changed:

| fixture | base suite bytes | 0.54.0 suite bytes | scenarios that changed |
|---|---|---|---|
| `arrangement-input-refusal.yaml` (and `@ess22`) | 29824 | 32088 | `vault.acct.Configure/outcome/id-required`, `…/secret-too-short` |
| `now-stored-rows.yaml` | 108332 | 109803 | `demo.leases.RenewLease/outcome/blank-note` |

Installed `ess 0.53.0` reproduces the base sizes; the gate's 0.54.0 build reproduces the new
ones. The added steps are input-refusal arrangements for an unknown identity and for two
overlapping input refusals, which the 0.54.0 changelog attributes to beyond10x/ess#454 and
beyond10x/ess#455. The probe is `#[ignore]` and runs only in `task test-slow-probes`, so the
0.54.0 integration repair (`e9f6fcf2f`) did not see it.

## Milestone

`release-plan:ess-054`.
