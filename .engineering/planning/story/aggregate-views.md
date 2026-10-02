---
format: aep.planning-md/3
id: story:aggregate-views
kind: story
status: implemented
title: A view can return aggregates over one entity's rows
refs:
- provider: github
  reference: beyond10x/ess#96
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T21:44:43Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T21:45:12Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-02T09:41:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
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

## Acceptance reconciliation 2026-10-02

The finalized binding design explicitly resolves avg to Optional<Decimal>, absent over zero rows (docs/design/aggregate-views.md:178/:209), correcting the older Acceptance shorthand avg -> Decimal. It retains six-place half-even rounding. This is the already-shipped design, not a newly narrowed implementation target.

Design commit152fdd066 precedes implementation d95b4581f, which is an ancestor of public0.51.0. Domain tests aggregate_views.rs:202/:311/:379/:427/:532 cover group fields, types and ess/10 admission. Compiler aggregate_views_ir, generator aggregate_views and synth aggregate_views tests cover required projections. Conformance aggregate_views.rs:176 and aggregate_views_mutants.rs:367-398 execute honest rows and independently catch ignored filter, either key, omitted rows, empty-group errors, wrong inputs and truncating averages; aggregate_semantics.rs:28 pins rounding. Retained d2-scratch/final/ess-conformance.log completed EXIT=0; current full conformance package lanes also passed.

The original capability is delivered. Later defects309/361/362 remain separate, explicitly open work and are not erased by correcting this stale lifecycle.
