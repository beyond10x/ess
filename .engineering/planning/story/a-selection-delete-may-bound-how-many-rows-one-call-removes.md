---
format: aep.planning-md/3
id: story:a-selection-delete-may-bound-how-many-rows-one-call-removes
kind: story
status: draft
title: A selection delete may bound how many rows one call removes
summary: 'instances: takes only where, so a bounded batch delete (at most N rows, in a declared order) cannot be declared (adopter item 14, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (recheck batch item 14, the second team's L7, relayed by the orchestrator; adopter names
kept out). A cleanup command deletes the rows a filter selects, but at most N of them per call (a bounded batch), so a large
backlog is drained over several calls. `deletes:` with `instances: {where: ...}` (format ess/23, beyond10x/ess#452) deletes
every selected row and takes no bound and no order.

Needed: a selection-effect `instances:` that may carry a row bound (and the order that decides which rows fall inside it), with
`{count: changed}` reporting at most the bound; or a recorded decision that a bounded batch delete is out of scope, with the
idiom named.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Item {item_id, owner}`, `AddItem` creating it,
`PurgeItems(owner)` with `deletes: demo.server.Item`, `emits: [ItemsPurged]`, payload `removed: {count: changed}`.
`ess specify validate --path spec.yaml`, identical output on 0.68.0 and 0.67.0:

| `instances:` | result |
|---|---|
| `{where: owner == input.owner}` (control) | valid; interpreted conformance 2 scenarios, 2 passed, both versions, byte-identical suites |
| `{where: owner == input.owner, limit: 100}` | `unknown field `limit`, expected `where`` |
| `{where: owner == input.owner, order_by: [item_id asc], limit: 100}` | `unknown field `order_by`, expected `where`` |

Same refusals with `format: ess/30` on 0.68.0.

## Where it lives

- `crates/specify/ess-domain/src/command/set_effects.rs:92` (`RawInstances`): `deny_unknown_fields`, one field `filter`
  serialized as `where`. No bound and no order.
- `story:feature-request-452` (implemented) delivered the unbounded `deletes:` + `instances:` form; its fit review names no
  bound.

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0 and 0.68.0. Commits `071b98401`, `c4d3237c7` and `082e59982` in
`0.67.0..52bd110dd` touch `set_effects.rs` (ess/30 selector work, a Clippy fix) and leave `RawInstances` at `where` only. Gap. Does not block 0.68.0.
