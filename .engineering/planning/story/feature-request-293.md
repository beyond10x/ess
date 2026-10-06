---
format: aep.planning-md/3
id: story:feature-request-293
kind: story
status: implemented
title: Explorer excludes every command with an Optional input (and every command with an unknown_instance branch)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#293
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:43:15Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:43:15Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:15Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Resolve beyond10x/ess#293: Explorer excludes every command with an Optional input (and every command with an unknown_instance branch).

## Origin

beyond10x/ess#293, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 5, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Pending.
