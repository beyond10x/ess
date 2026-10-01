---
format: aep.planning-md/3
id: story:feature-request-284
kind: story
status: draft
title: ess ui check does not check that a page actor is granted the commands it binds
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#284
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess ui check` refuses a page binding to a command its actor is not granted.

## Acceptance

- The #284 reduction fails `ess ui check`, and the error names the page, actor and command.

## Origin

beyond10x/ess#284, reported downstream on 0.48.0.

## Fit review

Pending.
