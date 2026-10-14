---
format: aep.planning-md/3
id: epic:generic-aggregates
kind: epic
status: draft
title: 'Aggregates over expressions: measures, group keys and arithmetic over measures'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#561
- provider: github
  reference: beyond10x/ess#562
- provider: github
  reference: beyond10x/ess#563
- provider: github
  reference: beyond10x/ess#565
- provider: github
  reference: beyond10x/ess#566
relations:
- serves: vision:O2
- depends_on: epic:generic-expression-language
revision: 2
---
## Outcome

One generic capability: **aggregates over expressions**. A measure takes an expression operand
(`sum(ended_at - started_at)`, `sum(duration_s * level.weight)`); `min`/`max` work on any ordered
type, an enum with a declared order included; a group key is an expression, including one group per
element of a `List`; arithmetic over measures (`1 - sum(down) / span(param.from, param.to)`)
carries a declared rounding and zero-denominator rule; an interval-union measure clipped to a
period is a library function, not a new key.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 3. Builds on
`epic:generic-expression-language` (arithmetic and time) and `epic:generic-row-queries` (navigation in a
measure's operand).

## Children

| story | covers |
|---|---|
| `story:aggregate-measures-over-expressions` | https://github.com/beyond10x/ess/issues/561, https://github.com/beyond10x/ess/issues/566 |
| `story:aggregate-group-keys-are-expressions` | https://github.com/beyond10x/ess/issues/565 |
| `story:derived-measures-and-interval-union` | https://github.com/beyond10x/ess/issues/562, https://github.com/beyond10x/ess/issues/563 |

Visibility issue: https://github.com/beyond10x/ess/issues/561.

## Earlier requests in the same family (cited)

https://github.com/beyond10x/ess/issues/96 (aggregates, one top-level field),
https://github.com/beyond10x/ess/issues/148, https://github.com/beyond10x/ess/issues/363,
https://github.com/beyond10x/ess/issues/446, and https://github.com/beyond10x/ess/issues/441
(decision 3 of `docs/design/aggregate-views.md:1031`, "ratios and differences between aggregates";
`story:feature-request-441` recorded alternative (c), a `ratio:` measure, as the shape to start
from). `story:feature-request-361` and `story:feature-request-362` (aggregate witnesses) stay where
they are.

## Acceptance

Each child's named scenarios pass; `docs/design/aggregate-views.md` § "Out of scope" is revised with
the binding design.
