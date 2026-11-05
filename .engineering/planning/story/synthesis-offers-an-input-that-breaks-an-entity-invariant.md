---
format: aep.planning-md/3
id: story:synthesis-offers-an-input-that-breaks-an-entity-invariant
kind: story
status: draft
title: Synthesis offers an input that would break an entity invariant and expects it refused
summary: Synthesis writes invariant scenarios only with inputs inside the invariant, so an implementation that never checks invariants passes (beyond10x/ess#738)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#738
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

For an entity invariant, synthesis writes one scenario per state-changing branch with an input inside the invariant. It never offers an input outside it and expects the step refused with nothing changed, so an implementation that commits whatever a sets: source supplies passes every scenario.

Reported in https://github.com/beyond10x/ess/issues/738 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept, design first: a new scenario family and refusal expectation, likely a suite-format change; scope it before a wave takes it.
