---
format: aep.planning-md/3
id: story:feature-request-286
kind: story
status: draft
title: A view declares which actors may read it
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#286
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A specification declares which actors may read a view, and conformance checks it.

## Acceptance

- A view readable by one of two actors validates.
- Served components refuse the other actor's read.
- A synthesized scenario witnesses that refusal.

## Origin

beyond10x/ess#286, reported downstream on 0.48.0. Read-side counterpart of #265.

## Fit review

Pending.
