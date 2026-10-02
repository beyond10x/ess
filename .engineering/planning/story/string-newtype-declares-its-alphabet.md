---
format: aep.planning-md/3
id: story:string-newtype-declares-its-alphabet
kind: story
status: active
title: A String newtype can declare its character set
refs:
- provider: github
  reference: beyond10x/ess#103
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:35:28Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:36:25Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

A String newtype can declare its character set.

## Why

GitHub issue beyond10x/ess#103; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#103 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
