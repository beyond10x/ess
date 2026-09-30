---
format: aep.planning-md/3
id: epic:downstream-reported-gaps
kind: epic
status: active
title: Downstream-reported synthesis, binding and grant gaps are closed
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T13:04:29Z", actor: "human:timo", revision: 3}
---
## Outcome

A downstream specification synthesizes, runs and records conformance without refusals that come from ESS rather than from the model: related-row guards and values, aggregate group keys, bindings that move state, authorization, and the binding language gaps it reported are closed.

## Scope

Issues beyond10x/ess#229 (state inside when_related), #257, #265, #266, #267, #268, #269, #270, #271, #272. Each story names its conformance scenarios; the downstream mini specifications under the reporter's scratch reproduce #270–#272 and are not copied into this repository.

## Order

Scheduled by `aep plan artifact waves` from each story's scope; the synthesis stories that share `synthesize/aggregate.rs` or the relation-guard arrangement run in sequence.
