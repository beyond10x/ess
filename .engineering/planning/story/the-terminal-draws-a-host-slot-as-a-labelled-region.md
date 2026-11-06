---
format: aep.planning-md/3
id: story:the-terminal-draws-a-host-slot-as-a-labelled-region
kind: story
status: draft
title: The terminal draws a host slot as a labelled region
summary: 'ADR Effect (terminal): a placeholder region that ess-ui-test can address; steps inside it are refused'
tags:
- backlog
- queue-0.90.0
- rank-7
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
revision: 1
---
## Need

The accepted ADR on host components, Effect (terminal): the terminal renderer cannot draw a host
component, so it marks the slot's place. The design is design:ui-host-slots, section Generators.

## Acceptance

- The terminal renderer draws a labelled region at each slot's path, showing `host` and `reason`.
- `ess-ui-test/1` can address that region by its path.
- An `ess-ui-test/1` step that acts inside the region is refused by name.

## Scope

- inferred: `crates/ui/ess-ui-tui/src/`, `crates/ui/ess-ui-test/src/runner.rs`.
