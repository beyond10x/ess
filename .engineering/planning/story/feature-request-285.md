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
revision: 1
---
## Outcome

A created row copies a value behind an Optional reference or two references away, so a view can group by it.

## Acceptance

- The #285 reduction validates, and `CostPerOutcome` synthesizes with no ESS-SYNTH-017.
- An absent reference yields an absent copied value, witnessed by a scenario.

## Origin

beyond10x/ess#285, reported downstream on 0.48.0. May overlap #257 for the one-hop Optional group key.

## Fit review

Pending; three alternative forms are offered in the issue.
