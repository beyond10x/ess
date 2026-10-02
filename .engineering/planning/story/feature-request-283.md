---
format: aep.planning-md/3
id: story:feature-request-283
kind: story
status: draft
title: 'A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004)'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#283
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 6
---
## Outcome

A command guards on more than one related row, each with its own `exists: false` answer.

## Acceptance

- A command with two `via` paths and one `exists: false` branch each validates.
- Synthesis witnesses each branch with the other row arranged to pass.

## Origin

beyond10x/ess#283, reported downstream on 0.48.0. The `{related:}` copy part is #270.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: one command guarding on two related rows. Class: gap. Existing idiom: none; ESS-COMMAND-004 keeps one `exists: false`. Fit: generalises `when_related` per `via` path; no new construct.

## Decisions

- **accept, redesigned:** one `exists: false` branch per distinct `via` path; across related rows, step 1 and the new related-predicate step of the precedence order apply in declaration order. From the next format version (ess/21: ess/20 ships alone in 0.49.0). Synthesis arranges the other related rows to pass when witnessing one row's branch. Scheduled after #282, which owns the precedence step.

## Source version allocation, 2026-10-02

The operator explicitly prioritized issue389 for the fast lane. The coordinator allocates the next unshipped source major, ess/21, to its one-time response disclosure contract. The previously planned coordinated downstream syntax bundle moves together to ess/22; its accepted behavior, dependency ordering and requirement to ship as one bundle are unchanged. Prior mentions of ess/21 in this artifact record the earlier allocation, not the current implementation target. No released source/IR/suite meaning is rewritten by this planning change.

This allocation must be reflected in binding designs and compatibility tests before implementation. The389 design independently names its new IR and ordinary/coverage suite versions from actual current source; those numbers are not inferred from the source-format number.
