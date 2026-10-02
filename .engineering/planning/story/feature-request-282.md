---
format: aep.planning-md/3
id: story:feature-request-282
kind: story
status: draft
title: ESS-COMMAND-004 refuses a when_related refusal beside a wrong_state outcome; no precedence is stated
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#282
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 6
---
## Outcome

A command with a `when_related` refusal and a `wrong_state` outcome validates, with a stated precedence between them.

## Acceptance

- The two reductions in #282 validate, and synthesis witnesses both branches.
- The precedence is written in `docs/design/input-guard-overlap-precedence.md`.

## Origin

beyond10x/ess#282, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: a command refusing on a related row's fields beside its own `wrong_state`. Class: gap. Existing idiom: none; the precedence order (`docs/design/cross-record-and-stored-field-guards.md#the-precedence-order`) has no step for a `when_related` predicate refusal, which is why ESS-COMMAND-004 refuses the pair. Fit: adding one step to that single order, not a per-command declaration.

## Decisions

- **accept, redesigned:** no author-declared precedence. The order gains one step: a `when_related` predicate refusal answers after the held state (step 4) and before accepting branches (step 5), so the addressed row's own lifecycle answers first. Admitted from ess/21 only (ess/20 ships alone in 0.49.0; this bundles with family F); validation, synthesis and the interpreter follow the order, and the design doc states it.

## Source version allocation, 2026-10-02

The operator explicitly prioritized issue389 for the fast lane. The coordinator allocates the next unshipped source major, ess/21, to its one-time response disclosure contract. The previously planned coordinated downstream syntax bundle moves together to ess/22; its accepted behavior, dependency ordering and requirement to ship as one bundle are unchanged. Prior mentions of ess/21 in this artifact record the earlier allocation, not the current implementation target. No released source/IR/suite meaning is rewritten by this planning change.

This allocation must be reflected in binding designs and compatibility tests before implementation. The389 design independently names its new IR and ordinary/coverage suite versions from actual current source; those numbers are not inferred from the source-format number.
