---
format: aep.planning-md/3
id: story:mutation-audit-and-model-runner
kind: story
status: active
title: ess audits a suite by mutation and explores sequences against the IR
refs:
- provider: github
  reference: beyond10x/ess#114
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:55:49Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:56:24Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

ess audits a suite by mutation and explores sequences against the IR.

## Why

GitHub issue beyond10x/ess#114; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#114 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
