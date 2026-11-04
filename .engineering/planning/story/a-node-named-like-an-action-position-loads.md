---
format: aep.planning-md/3
id: story:a-node-named-like-an-action-position-loads
kind: story
status: draft
title: A state or region named like an action position loads
summary: The shorthand expander reads a named map entry called action or form as that construct (UI-adopter wave 1 U4 implementor note, pre-existing, backlog)
tags:
- backlog
- from-finding
relations:
- informed_by: story:a-refused-node-is-reported-at-its-own-path
revision: 1
---
## Need

The loader's local-shorthand expander (`crates/ui/ess-ui/src/expand.rs`, `Local::mapping`) reads
a map entry by its key: a state, a region or another named entry called `action`, `form` or another
action position, and holding a map, is expanded as that construct, so a valid document is refused.
Found by the load-error-path unit of UI-adopter wave 1 (story:a-refused-node-is-reported-at-its-own-path),
whose fix in `locate.rs` derives named-node maps from the schema's `containers` and so reports the
right path, but leaves the expander's own name-based reading in place; pre-existing at 83391ebaeb.

## Acceptance

- A document whose state, region or other named map entry is called like an action position
  (`action`, `form`, ...) loads and expands as the same document with another name does.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
