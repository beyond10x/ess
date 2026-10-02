---
format: aep.planning-md/3
id: story:feature-request-285
kind: story
status: draft
title: '{related:} reads through an Optional reference or across two references'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#285
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
---
## Outcome

A created row copies a value behind an Optional reference or two references away, so a view can group by it.

## Acceptance

- The #285 reduction validates, and `CostPerOutcome` synthesizes with no ESS-SYNTH-017.
- An absent reference yields an absent copied value, witnessed by a scenario.

## Origin

beyond10x/ess#285, reported downstream on 0.48.0. May overlap #257 for the one-hop Optional group key.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: copy a value behind an Optional reference or two references away, so a view can group by it. Class: gap. Three forms were offered. A join in a view adds a concept views do not have (a view reads one entity); `{related:}` through an Optional reference and a chained `via` extend the existing read.

## Decisions

- **accept, redesigned:** `{related: …}` through an Optional reference yields an Optional value (absent when the reference is absent), and `via` accepts a list for a chained read, each hop Optional-aware. From the next format version (ess/21: ess/20 ships alone in 0.49.0). **Decline** the view join. Check #257's merged fix against the one-hop Optional group key (ESS-SYNTH-017) first; that part may already be closed.

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- The one-hop Optional group key (ESS-SYNTH-017) may already be fixed by #257 (merged on `integrate/gaps-w1`). Check it before scheduling.

## Source version allocation, 2026-10-02

The operator explicitly prioritized issue389 for the fast lane. The coordinator allocates the next unshipped source major, ess/21, to its one-time response disclosure contract. The previously planned coordinated downstream syntax bundle moves together to ess/22; its accepted behavior, dependency ordering and requirement to ship as one bundle are unchanged. Prior mentions of ess/21 in this artifact record the earlier allocation, not the current implementation target. No released source/IR/suite meaning is rewritten by this planning change.

This allocation must be reflected in binding designs and compatibility tests before implementation. The389 design independently names its new IR and ordinary/coverage suite versions from actual current source; those numbers are not inferred from the source-format number.
