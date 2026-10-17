---
format: aep.planning-md/3
id: epic:generic-expression-language
kind: epic
status: draft
title: One typed expression language with arithmetic, time and string functions, usable everywhere
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#555
- provider: github
  reference: beyond10x/ess#556
- provider: github
  reference: beyond10x/ess#560
- provider: github
  reference: beyond10x/ess#567
relations:
- serves: vision:O2
- informed_by: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

One generic capability: **one typed expression language over facts**, with arithmetic
(`+ - * /` over Integer and Decimal), time (`Timestamp - Timestamp -> Duration`,
`Timestamp ± Duration`, `now`), string functions, and a narrowing `defined(x)`, over the roots
`input`, `subject` (with `state`), `caller`, `response`, `param`, `event`, `context`, `now` and
bound names. One grammar for guards, invariants, view filters, binding conditions, `sets:`,
`payload:`, response fields and computed view fields, and one position table that says which roots
a position admits. Today's mapping forms lower to it, so existing documents keep their IR.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 2.

## Children

| story | covers |
|---|---|
| `story:expression-roots-a-branch-establishes-are-readable` | https://github.com/beyond10x/ess/issues/555, https://github.com/beyond10x/ess/issues/556 |
| `story:expression-arithmetic-and-time` | https://github.com/beyond10x/ess/issues/560, https://github.com/beyond10x/ess/issues/567 |

Visibility issue: https://github.com/beyond10x/ess/issues/560.

## Earlier requests in the same family (cited, not moved here)

- Arithmetic: https://github.com/beyond10x/ess/issues/233 (one constant offset,
  `docs/design/expression-family-source22.md:29-33`), https://github.com/beyond10x/ess/issues/441
  and https://github.com/beyond10x/ess/issues/442 (declined with idioms,
  `docs/design/read-api-view-idioms.md:26-89`).
- Current time: https://github.com/beyond10x/ess/issues/171, https://github.com/beyond10x/ess/issues/244,
  https://github.com/beyond10x/ess/issues/439.
- Value roots: https://github.com/beyond10x/ess/issues/498 (`story:stored-field-equals-returned-response-value`),
  https://github.com/beyond10x/ess/issues/506 (`story:response-field-declares-its-value-source`),
  https://github.com/beyond10x/ess/issues/532 and https://github.com/beyond10x/ess/issues/545 (open,
  in passes of their own; not edited here).

## Acceptance

Each child's named scenarios pass; the position table is generated from the implementation and
drift-checked, and every refusal of a root in a position names the table's row.
