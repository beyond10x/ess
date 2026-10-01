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
revision: 1
---
## Outcome

A command with a `when_related` refusal and a `wrong_state` outcome validates, with a stated precedence between them.

## Acceptance

- The two reductions in #282 validate, and synthesis witnesses both branches.
- The precedence is written in `docs/design/input-guard-overlap-precedence.md`.

## Origin

beyond10x/ess#282, reported downstream on 0.48.0.

## Fit review

Pending (`.agents/skills/assessing-external-requests/SKILL.md`).
