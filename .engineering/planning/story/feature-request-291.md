---
format: aep.planning-md/3
id: story:feature-request-291
kind: story
status: implemented
title: conform run --target interpreted answers wrong_state for an unknown identity where synthesis expects the declared not-found refusal
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#291
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:46:21Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:46:21Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:46:21Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Resolve beyond10x/ess#291: conform run --target interpreted answers wrong_state for an unknown identity where synthesis expects the declared not-found refusal.

## Origin

beyond10x/ess#291, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 3, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Pending.
