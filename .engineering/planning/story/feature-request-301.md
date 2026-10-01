---
format: aep.planning-md/3
id: story:feature-request-301
kind: story
status: draft
title: synthesize is 20-30x slower since 0.40.0 on one specification (8 s to 4-7 min, scenarios +15%)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#301
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Synthesis time is proportional to the scenarios it writes again.

## Acceptance

- The #301 reference specification synthesizes within 2x of 0.40.0's time.
- A synthesis timing guard runs in CI.

## Origin

beyond10x/ess#301, reported downstream (8.3 s on 0.40.0 vs 4–7 min on main).

## Fit review

- Class: defect (performance regression). No surface. **accept**: bisect 0.40.0..0.46.1 with a debug-symbol build on the reporter's copy, then fix. Priority 1.
