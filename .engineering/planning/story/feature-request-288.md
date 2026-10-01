---
format: aep.planning-md/3
id: story:feature-request-288
kind: story
status: draft
title: 'An affects: filter over subject identity is witnessed'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#288
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

An `affects:` filter over `subject.<identity>` is witnessed as one over `input.<identity>` is.

## Acceptance

- The #288 reproduction synthesizes with no ESS-SYNTH-001.

## Origin

beyond10x/ess#288, found in the 2026-10-01 fit review.

## Fit review

- Class: defect (validate admits what synthesis cannot witness). No new surface.

## Decisions

- **accept as proposed** (coordinator, 2026-10-01).
