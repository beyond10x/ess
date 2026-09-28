---
format: aep.planning-md/3
id: story:report-carries-passed-failed-skipped
kind: story
status: active
title: The conformance report carries passed, failed and skipped counts
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:45:34Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:46:11Z", actor: "human:timo", revision: 3, imported: true}
---
## Outcome

The conformance report carries passed, failed and skipped counts.

## Why

GitHub issue beyond10x/ess#110; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#110 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.
