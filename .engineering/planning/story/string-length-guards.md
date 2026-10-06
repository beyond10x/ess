---
format: aep.planning-md/3
id: story:string-length-guards
kind: story
status: implemented
title: A guard can test the length of a String
refs:
- provider: github
  reference: beyond10x/ess#104
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:38:22Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:38:56Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:46Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A guard can test the length of a String.

## Why

GitHub issue beyond10x/ess#104; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#104 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
