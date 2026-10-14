---
format: aep.planning-md/3
id: epic:generic-row-queries
kind: epic
status: draft
title: 'Queries over rows: select, order and follow references in every position'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#557
- provider: github
  reference: beyond10x/ess#558
- provider: github
  reference: beyond10x/ess#564
relations:
- serves: vision:O2
- informed_by: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

One generic capability: **queries over rows**. A selection of rows (`rows(E where p order_by k)`)
with `exists`, `count`, `forall`, `first` and `each`, and navigation through a declared reference
(`subject.incident.state`, every element of a `List` of references), usable in every position that
reads a value: guards, `sets:`, `payload:`, response fields, `affects:` filters, view sources, view
fields and view filters, and binding conditions. Today each position gained one root or one
selector per request; this feature closes the row instead of the cell.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 1.

## Children

| story | covers |
|---|---|
| `story:row-queries-navigate-a-reference` | https://github.com/beyond10x/ess/issues/557, https://github.com/beyond10x/ess/issues/564 |
| `story:row-queries-quantify-over-a-list-of-references` | https://github.com/beyond10x/ess/issues/558 |

Visibility issue: https://github.com/beyond10x/ess/issues/564.

## Earlier requests in the same family (cited, not moved here)

- https://github.com/beyond10x/ess/issues/447 and gap 4 of https://github.com/beyond10x/ess/issues/492:
  a live one-hop join, declined with idioms (`docs/design/read-api-view-idioms.md:174-196`,
  `story:a-view-filter-follows-a-reference`). https://github.com/beyond10x/ess/issues/564 is the
  third adopter and states why the idioms do not serve: they take the figure out of conformance.
- https://github.com/beyond10x/ess/issues/505 (`story:related-identity-readable-in-when-related`).
- https://github.com/beyond10x/ess/issues/531, https://github.com/beyond10x/ess/issues/543,
  https://github.com/beyond10x/ess/issues/547 (`story:set-effect-filters-read-the-identity`, held by
  another branch), https://github.com/beyond10x/ess/issues/549 and
  https://github.com/beyond10x/ess/issues/551: open, in passes of their own; this feature is where
  their selectors converge, and their stories are not edited here.
- https://github.com/beyond10x/ess/issues/473 is a row-set selector beside an identity-addressed one;
  its story answers which guard answers first, so it sits under `epic:generic-guard-precedence`.

## Acceptance

Each child's named scenarios pass; the binding design's position table lists every position this
feature admits, and a position it does not admit is refused by name.
