---
format: aep.planning-md/3
id: story:typed-literals-in-sets-and-unknown-instances
kind: story
status: active
title: 'sets: accepts typed literals and an unknown instance has a declared answer'
refs:
- provider: github
  reference: beyond10x/ess#113
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:53:20Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:54:00Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

sets: accepts typed literals and an unknown instance has a declared answer.

## Why

GitHub issue beyond10x/ess#113; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#113 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
