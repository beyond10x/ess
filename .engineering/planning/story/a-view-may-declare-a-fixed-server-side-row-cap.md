---
format: aep.planning-md/3
id: story:a-view-may-declare-a-fixed-server-side-row-cap
kind: story
status: draft
title: A view may declare a fixed server-side row cap beside its order
summary: 'limit: is an unknown view field and paging: sizes only from a caller parameter, so "newest 100 rows" cannot be declared (adopter item 15, 2026-10-22); same on 0.67.0.'
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

Reported by an adopter on 2026-10-22 (recheck batch item 15, the second team's L8, relayed by the orchestrator; adopter names
kept out). A list endpoint answers the newest 100 rows: a fixed server-side cap, not a page the caller chooses. `paging:`
(format ess/16, beyond10x/ess#174) covers a caller-chosen page and size; nothing declares a cap the server applies whatever the
caller sends.

Needed: a view may declare a fixed row cap beside its `order_by:` (the first N rows in that order), checked by conformance as
"at most N rows, and they are the first N"; or a recorded decision that the cap is expressed another way, with the idiom named.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Item {item_id, added_at: Timestamp}`, `AddItem` creating
it, view `NewestItems` over `Item`, `consistency: read_your_writes`, `order_by: [added_at desc]`.
`ess specify validate --path spec.yaml`, identical output on 0.68.0 and 0.67.0:

| view declares | result |
|---|---|
| nothing beyond `order_by:` (control) | valid; interpreted conformance 1/1 passed, both versions |
| `params: [page, size]` and `paging: {page: page, size: size}` (control) | valid; 1/1 passed, both versions, byte-identical suites |
| `limit: 100` | `unknown field `limit`, expected one of `name`, `source`, `shape`, `fields`, `params`, `filter`, `group_by`, `order_by`, `paging`, `absent`, `consistency`, `naming`` |
| `paging: {size: 100}` | `invalid type: integer `100`, expected a string` (`size` names a parameter) |

Same refusal for `limit:` with `format: ess/30` on 0.68.0.

## Where it lives

- `crates/specify/ess-domain/src/view.rs:2006` (`RawViewSpec`): the field list in the refusal; `paging` at `view.rs:2032`.
- `crates/specify/ess-domain/src/view/paging.rs:39` (`Paging`): `page` and `size` are parameter names (`String`), so a
  constant size cannot be written.
- `story:view-paging-and-caller-filters` (implemented) delivered caller paging and declined caller free-form filters; it names
  no server-side cap.

## Regression status

Not a 0.68.0 regression: identical refusals on 0.67.0 and 0.68.0. Commit `082e59982` in `0.67.0..52bd110dd` touches `view.rs`
(a view `source:` list refused by name) and adds no cap. Gap. Does not block 0.68.0.
