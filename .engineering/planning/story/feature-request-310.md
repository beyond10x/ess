---
format: aep.planning-md/3
id: story:feature-request-310
kind: story
status: draft
title: Generated code targets select a branch by whether the record exists
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#310
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Generated Rust, Go and TypeScript servers select a branch by whether the addressed record exists.

## Acceptance

- A specification with an `existing_instance:` refusal generates for every code target.
- Its existence scenarios pass against the generated server.

## Origin

beyond10x/ess#310, reported downstream on 0.49.0. It blocks a downstream release.

## Fit review

- Class: defect. ess/16 admits the form, but no code target carries it. No new surface. **accept as proposed** (coordinator, 2026-10-01). Priority 1, Rust first.
