---
format: aep.planning-md/3
id: story:feature-request-312
kind: story
status: active
title: Suites state their empty-target assumption and act across callers
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#312
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: docs/design/scenario-initial-state-and-cross-caller-witnesses.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T19:07:02Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}}
- {from: "proposed", to: "active", at: "2026-10-03T19:07:02Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"approval":1}}}
---
## Outcome

Suites either hold on a shared target or say they need an empty one, and some scenario acts on a row as a different caller than arranged it.

## Origin

beyond10x/ess#312, found in the #287 adversary pass; pre-existing on 0.49.0.

## Fit review

The accepted canonical intake record at consumer-backlog commit `acb88e97d3c99587c3b0a501314ce35d73fa4f3b` and its retained continuation classifies this as a defect in suite claims and cross-caller evidence. The suite must declare an empty logical modeled namespace before each scenario and must arrange and act on the same row as different declared callers where the source permits it. No new source keyword or implicit physical database deletion is introduced. Actor/caller authority, source refusal precedence and existing composition restrictions remain intact. The binding implementation contract is `docs/design/scenario-initial-state-and-cross-caller-witnesses.md`.

The coordinator accepted initial-state provenance in the unreleased suite34/35 pair; formats1-33 retain unspecified initial-state semantics and their old bytes. CountReport/2 remains closed. Fresh ordinary/coverage synthesis selects34/35. Historical format-specific tests must explicitly construct legitimate legacy documents without new provenance when testing legacy admission, while current-emission tests assert34/35 and typed Empty. A report/1 writer cannot accept a newly synthesized suite34; modern tests must use exact-suite-bound report/2 without rewriting genuine Failed/Skipped/Unsupported categories.

The current bundle imports the production source, but final combined verification remains incomplete. This continuation reconciles the stale draft mirror with the accepted canonical active story; it does not close312 or turn prior partial evidence into current success.
