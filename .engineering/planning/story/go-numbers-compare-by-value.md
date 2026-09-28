---
format: aep.planning-md/3
id: story:go-numbers-compare-by-value
kind: story
status: active
title: Go conformance compares JSON numbers by value
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:30:42Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:31:36Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

Go conformance compares JSON numbers by value.

## Why

GitHub issue beyond10x/ess#101; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#101 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
