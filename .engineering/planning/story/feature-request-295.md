---
format: aep.planning-md/3
id: story:feature-request-295
kind: story
status: draft
title: 'mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#295
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Resolve beyond10x/ess#295: mutate: emit-drop is stillborn on every outcome that emits one event, so single-event emission is never audited.

## Origin

beyond10x/ess#295, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 6b, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Pending.
