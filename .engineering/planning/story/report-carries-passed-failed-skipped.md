---
format: aep.planning-md/3
id: story:report-carries-passed-failed-skipped
kind: story
status: active
title: The conformance report carries passed, failed and skipped counts
refs:
- provider: github
  reference: beyond10x/ess#110
relations:
- serves: vision:O2
revision: 5
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

## Reconciliation 2026-10-02

Released via documented report/2 design, commit69abcd25a contained in0.51.0. The practical arithmetic/counting need is met, but report/1 did not gain the requested scenarios_passed/scenarios_skipped fields. report/2 carries counts.passed/failed/skipped and separate outcomes.failed/skipped. docs/design/truthful-conformance-counts.md:27-35 explicitly preserves report/1; website/docs/guides/verify/runners.md:115-131 maps fields and ESS_REPORT_FORMAT=2 for Go/TypeScript. TypeScript runtime.test.ts:729 asserts1passed/1failed/1skipped and incomplete-run refusal;:672 and ess-cli/tests/go_conformance.rs:1251 hold the once-only migration notice.

Preserve this already-shipped versioned-schema decision rather than adding incompatible report/1 fields to satisfy a literal old proposal. Record the original request as fulfilled through this documented alternative. Original red-first output was not located, so do not claim its chronology proven; lifecycle remains active for that evidence qualification, not missing count behavior.
