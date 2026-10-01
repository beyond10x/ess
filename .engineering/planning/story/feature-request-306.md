---
format: aep.planning-md/3
id: story:feature-request-306
kind: story
status: active
title: Committed generated output regenerates in another checkout
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#306
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T16:38:14Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-01T16:38:15Z", actor: "human:timo", revision: 3}
---
## Outcome

Committed generated output regenerates in any checkout of the repository.

## Acceptance

- An Idle output state is rebound to a moved or second checkout, and generation runs.
- A non-Idle state still refuses a mismatched root, naming the recorded path.

## Origin

beyond10x/ess#306, reported downstream on 0.48.0.

## Fit review

- Class: defect. Committed `.ess-output` is a documented layout (`website/docs/start/runners/go.md:59`), but it works in one checkout only. No new authored surface.

## Decisions

- **accept, redesigned** (coordinator, 2026-10-01): rebind an Idle checkpoint; keep the strict binding for in-flight transactions. Priority 1 (base defect); ships in the next release.
