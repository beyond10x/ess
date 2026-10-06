---
format: aep.planning-md/3
id: story:feature-request-300
kind: story
status: implemented
title: 'ess-ui/1: widget expansion is exponential in nesting depth; a valid document can hang ess ui check'
tags:
- feature-request
- ui-spec
refs:
- provider: github
  reference: beyond10x/ess#300
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:46:22Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:46:22Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:46:23Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Resolve beyond10x/ess#300: ess-ui/1: widget expansion is exponential in nesting depth; a valid document can hang ess ui check.

## Origin

beyond10x/ess#300.

## Priority

UI spec: lower priority than every base-spec story (operator, 2026-10-01).

## Fit review

Pending.
