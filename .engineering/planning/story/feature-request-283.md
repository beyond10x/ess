---
format: aep.planning-md/3
id: story:feature-request-283
kind: story
status: draft
title: 'A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004)'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#283
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A command guards on more than one related row, each with its own `exists: false` answer.

## Acceptance

- A command with two `via` paths and one `exists: false` branch each validates.
- Synthesis witnesses each branch with the other row arranged to pass.

## Origin

beyond10x/ess#283, reported downstream on 0.48.0. The `{related:}` copy part is #270.

## Fit review

Pending.
