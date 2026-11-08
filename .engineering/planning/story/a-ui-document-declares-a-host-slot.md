---
format: aep.planning-md/3
id: story:a-ui-document-declares-a-host-slot
kind: story
status: implemented
title: A UI document declares a host slot
summary: 'ADR section 1: the host slot shape, its schema and reference page, and the declaration refusals'
tags:
- backlog
- queue-0.90.0
- rank-2
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:53Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:53Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-07T04:10:55Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Need

The accepted ADR on host components, section 1: every host slot is declared in the UI document.
Today, a hand-written part of a page lives in a patched runtime file or in a list kept beside the
document, and no ESS command reads either. The design is design:ui-host-slots, section Shape.

## Acceptance

- An `ess-ui/1` document may hold a node that has `host`, `reason` and `stands_in_for` keys, at a
  page, section, overlay, tab or field position. `ess ui check` accepts it.
- `ess ui check` refuses each of these by name:
  - a slot with no `reason`, or an empty one;
  - two slots with one `host` name;
  - `host` beside `component` or a primitive key.
- `ess ui check --hosts <file>` refuses a registered name that no slot declares.
- `schemas/ui/ess-ui.schema.yaml` and the `ess-ui/1` reference page describe the shape. The schema
  drift check stays green.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (node union), `crates/ui/ess-ui-check/src/`,
  `schemas/ui/ess-ui.schema.yaml`, `website/docs/reference/ess-ui.md`.
