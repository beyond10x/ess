---
format: aep.planning-md/3
id: story:a-host-slot-names-the-generator-item-it-stands-in-for
kind: story
status: implemented
title: A host slot names the generator item it stands in for
summary: 'ADR section 4: stands_in_for is checked, and none slots are listed as generator backlog'
tags:
- backlog
- queue-0.90.0
- rank-3
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:54Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:55Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-07T04:10:56Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Need

The accepted ADR on host components, section 4: each slot names the generator item it stands in
for. The gap then reads in the generator's own terms, and a later release can take the slot back.
The design is design:ui-host-slots, section Check.

## Acceptance

- `stands_in_for` accepts any of `COMPOSITE_KINDS`, a primitive, or `none`. `ess ui check`
  refuses any other value by name.
- `stands_in_for: none` with no `reason` is refused.
- The text report and the JSON report list the `none` slots, each with its path and reason.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (`COMPOSITE_KINDS`), `crates/ui/ess-ui-check/src/`.
