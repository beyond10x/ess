---
format: aep.planning-md/3
id: story:derived-measures-and-interval-union
kind: story
status: draft
title: A view derives figures from measures (uptime ratio over the period) and unions intervals clipped to a period
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#562
- provider: github
  reference: beyond10x/ess#563
relations:
- serves: vision:O2
- decomposes: epic:generic-aggregates
- depends_on: story:expression-arithmetic-and-time
revision: 2
---
## Outcome

A view derives figures from its measures: arithmetic over measures with a declared rounding and a
zero-denominator rule, with the period length of the view's own parameters as an operand (strict
uptime `1 - downtime / span`), and an interval-union measure clipped to a period, so overlapping
windows count once and a window across two periods counts in each.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 3, derived
measures and library functions. Parent: `epic:generic-aggregates`.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/562 and
https://github.com/beyond10x/ess/issues/563 (a downstream incident-ledger specification, ess 0.59.0,
`ess/24`).

1. **Need.** Downtime per environment as the length of the union of `[start, end)` intervals clipped
   to `[param.from, param.to)` (a consumer cannot derive it from counts or sums; it needs the rows),
   and uptime as `1 - downtime / (to - from)`. Refusals: `unknown variant union` and `unknown variant
   divide`, "expected one of count, count_distinct, sum, min, max, avg". Requester's syntax (theirs):
   `aggregate: {union: {from, to, clip}}`, `aggregate: {ratio: {numerator, denominator: {param_span}}}`.
2. **Class.** Gap. https://github.com/beyond10x/ess/issues/563 reopens
   https://github.com/beyond10x/ess/issues/441 (decision 3, `docs/design/aggregate-views.md:1031`;
   idiom in `docs/design/read-api-view-idioms.md:26-54`): the idiom moves the figure out of
   conformance.
3. **Already expressible?** No for the union. The ratio's operands are, the figure is not.
4. **Fit.** The ratio is the shape `story:feature-request-441` recorded as alternative (c) (Decimal,
   half-even, absent on a zero denominator), generalised to arithmetic over measures with
   `param.to - param.from` from the expression language. The interval union is a library measure over
   two expressions and a clip, not a new key family. Depends on the expression arithmetic
   (`story:expression-arithmetic-and-time`). Targets: validation, interpreter, synthesis (overlapping,
   adjacent, spanning and outside windows; a zero-length period), generated queries, diff.
5. **Second adopter.** Room occupancy per day from overlapping bookings; conversion rate per
   campaign (https://github.com/beyond10x/ess/issues/441).
6. **Cost.** A source format version; a suite pair carrying computed Decimal values; diff classes.
7. **Alternatives.** (a) Change nothing. (b) The requester's two keys. (c) Arithmetic over measures
   plus one library measure (chosen).

## Decisions

Accept, redesigned. Decision 3 of `docs/design/aggregate-views.md` is reopened by the 2026-10-13
decision to build the four generic features.

## Acceptance

- `interval_union_counts_overlap_once`: two overlapping windows of different incidents add their
  union, not their sum.
- `interval_union_clips_to_the_period`: a window across two periods counts its part in each.
- `uptime_ratio_over_the_parameter_span`: uptime is witnessed with a half-even tie and a zero-length
  period answering absent.
