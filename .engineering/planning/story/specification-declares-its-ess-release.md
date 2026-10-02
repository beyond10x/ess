---
format: aep.planning-md/3
id: story:specification-declares-its-ess-release
kind: story
status: active
title: A specification declares the ess release it is maintained with
refs:
- provider: github
  reference: beyond10x/ess#106
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:42:42Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:43:26Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

A specification declares the ess release it is maintained with.

## Why

GitHub issue beyond10x/ess#106; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#106 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
