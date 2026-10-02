---
format: aep.planning-md/3
id: story:suite-pins-transitions-updates-and-boundaries
kind: story
status: active
title: Generated suites pin transition targets, update values, every source and guard boundaries
refs:
- provider: github
  reference: beyond10x/ess#111
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:49:01Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:49:39Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

Generated suites pin transition targets, update values, every source and guard boundaries.

## Why

GitHub issue beyond10x/ess#111; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#111 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
