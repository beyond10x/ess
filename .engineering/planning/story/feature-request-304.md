---
format: aep.planning-md/3
id: story:feature-request-304
kind: story
status: draft
title: 'when_related through an Optional input: an absent reference takes the not-found branch instead of skipping the guard'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#304
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A `when_related` guard through an Optional input applies only when the input is present.

## Acceptance

- Absent input: accepted, no not-found branch taken.
- Present and in the state: accepted. Present in another state, or missing: refused.
- Synthesis witnesses all three.

## Origin

beyond10x/ess#304, reported downstream on 0.48.0.

## Fit review

Pending. Likely accept as a defect: the not-found branch should not answer an absent Optional.
