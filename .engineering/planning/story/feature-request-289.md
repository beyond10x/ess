---
format: aep.planning-md/3
id: story:feature-request-289
kind: story
status: draft
title: A quantifier binder on the right of a comparison means the binder
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#289
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A quantifier binder on the right of a comparison means the binder.

## Acceptance

- In the #289 reproduction, `a != b` compares the two binders.
- A bare word naming no field, input or binder in scope is refused, never read as text.

## Origin

beyond10x/ess#289, found in the 2026-10-01 fit review.

## Fit review

- Class: defect (a silent misread). It is the binder case of family F's A1 (bare right-hand root).

## Decisions

- **accept as proposed** (coordinator, 2026-10-01). It is a defect fix, so it lands before family F.
