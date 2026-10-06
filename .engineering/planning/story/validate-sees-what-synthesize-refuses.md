---
format: aep.planning-md/3
id: story:validate-sees-what-synthesize-refuses
kind: story
status: implemented
title: validate reports authored-scenario and unset-field problems synthesize would hit
refs:
- provider: github
  reference: beyond10x/ess#112
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:51:13Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:51:44Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:51Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

validate reports authored-scenario and unset-field problems synthesize would hit.

## Why

GitHub issue beyond10x/ess#112; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#112 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
