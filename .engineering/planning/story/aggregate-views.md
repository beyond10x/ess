---
format: aep.planning-md/2
id: story:aggregate-views
kind: story
status: active
title: A view can return aggregates over one entity's rows
relations:
- serves: vision:O2
revision: 3
---
## Outcome

A read API that returns aggregates over one entity's rows (calls per queue, total talk time per
agent) is specified and checked by conformance instead of being `UNMAPPED:`.

## Why

GitHub issue beyond10x/ess#96. `ViewSpec` (`crates/specify/ess-domain/src/view.rs:293-334`) has no
aggregate; `aggregate:` on a view field is `unknown field`. No design exists.

## Decisions (operator default, 2026-09-25) for the issue's open questions

1. An aggregate is a **view** (`group_by:` + field `aggregate:`), not a new declaration kind.
2. No time bucketing in the first cut.
3. No ratios or differences between aggregates; left to the consumer.
4. Empty groups are **absent** from the result; a view without `group_by` returns one row, with
   `count` 0 and `min`/`max` absent when no row passes the filter.

## Acceptance

- Design page `docs/design/aggregate-views.md` first, recording the four decisions above, result
  types (`count`/`count_distinct` → Integer; `sum` of Integer → Integer; `min`/`max` →
  `Optional<T>`; `avg` → Decimal with the rounding rule written down), `filter:` before grouping,
  and the conformance strategy.
- Domain, IR, validation (`group_by` keys must be view fields; an aggregate's input must be a
  source field of a compatible type), projections that the repository's projection checks require,
  and a source/format version bump.
- Conformance: create N rows through the declared creating command with distinct known values (two
  groups, one row excluded by the filter), assert each group's aggregate with `eventually`; a
  mutant that ignores the filter, a group key or one row fails. Depends on `creates` copying input
  into fields (delivered with #75's story).
