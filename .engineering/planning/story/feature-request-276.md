---
format: aep.planning-md/3
id: story:feature-request-276
kind: story
status: draft
title: Declarations added or removed leave no residual in the diff
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#276
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

`ess verify diff` reports no `unclassified-changed` for a declaration added or removed on one side, and actors' `attributes` are compared or named.

## Acceptance

- An added actor with `attributes`, and an added view with an `aggregation`, each produce only `<family>/<name>/added` (no `system/<system>/unclassified-changed`).
- An actor whose `attributes` change on both sides produces a classified change, or the residual names `attributes` explicitly.

## Origin

beyond10x/ess#276, reported downstream on 0.48.0; same class as #256.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect, diff only, no authored surface.

## Decisions

- **accept**, fixed for the class (declarations on one side leave the residual), not for one key.
