---
format: aep.planning-md/3
id: story:feature-request-297
kind: story
status: active
title: Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#297
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T13:32:48Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T13:32:48Z", actor: "human:timo", revision: 3}
---
## Outcome

Resolve beyond10x/ess#297: Conformance and exploration have no process restart, so identities minted from a counter that resets on restart go undetected.

## Origin

beyond10x/ess#297, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 8d, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

Pending.
