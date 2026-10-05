---
title: Aggregate views
sidebar_position: 5
description: Views with group_by that count, sum and bound one entity's rows, and conditional measures.
---

# Aggregate views

A read API that reports counts, sums and extremes over one entity's rows is a view with `group_by:`
and a field-level `aggregate:`. It needs `format: ess/10`.

```yaml
views:
  - name: metrics.session.TalkTimeByAgent
    source: metrics.session.Session
    consistency: eventual
    filter: state == Completed
    group_by: [agent_id]
    fields:
      - {name: agent_id, type: String}
      - {name: sessions, type: Integer, aggregate: {count: {}}}
      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}
      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}
      - {name: distinct_callers, type: Integer, aggregate: {count_distinct: caller}}
      - {name: mean_talk, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}
```

The filter runs on each source row first, the admitted rows are grouped by the `group_by` fields,
and each aggregate is computed per group. A group with no admitted row is absent. A view without
`group_by` returns exactly one row: `count` is `0` and `min`, `max` and `avg` are absent when no row
passes the filter. Every field without `aggregate:` must be listed in `group_by`.

Each field declares its result type exactly, and validation names the one it expects: `Integer`
for `count`, `count_distinct` and `sum` of an `Integer`; `Decimal` for `sum` of a `Decimal`;
`Optional<T>` for `min` and `max`, keeping a newtype; `Optional<Decimal>` for `avg`, which is
rounded to 6 fractional digits, ties to even. An aggregate reads one top-level field of the source
that every row holds, so an `Optional` argument or group key is refused, and so is grouping by a
`Timestamp` or ranking an aggregate view with `order_by:`.

Conformance creates the rows itself, through the declared creating outcome, and asserts every
group's exact numbers. Because a target may be shared, the rows are kept apart from every other
scenario's by a group key or a parameter compared with one (`queue_id == param.queue_id`) that is a
`String` or `Uuid` the creating command sets from its input. An ungrouped view with no parameter
is over every row, including other scenarios' rows, so conformance reads it before creating its rows
and asserts only how much each `count` and `sum` changed; its other aggregates are not asserted. A
grouped view with neither, or an ungrouped one with no `count` or `sum`, gets no scenario and the
refusal `ESS-SYNTH-016`.

### Conditional measures

From `format: ess/22` a measure may read only some rows of its group: `where:` beside the function
is a condition over one source row and the view's parameters, written as a filter is.

```yaml
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: total, type: Integer, aggregate: {count: {}}}
      - {name: completed, type: Integer, aggregate: {count: {}, where: state == Completed}}
      - {name: escalated_cost, type: Integer, aggregate: {sum: cents, where: escalated == true}}
```

The filter still decides which rows exist and which groups they form; each measure then reads the
rows of its group its own condition holds for. A group stays in the result when a condition selects
none of its rows: that measure's `count`, `count_distinct` and `sum` are `0`, its `min`, `max` and
`avg` absent, and a `sum` with `skip_absent: true` absent. A condition reads the source row, never a
result or another measure, and never `now`. A condition whose truth is unknown for some row — it
compares an absent value — makes the whole read undetermined rather than counting the row out.
`where: true` is a condition, not an omitted one; `null`, empty text and an empty list or map are
refused. A parameter read only by a condition is a parameter of the view; a paging parameter may not
be read by one.

Conformance arranges, in one group, rows its condition admits and rows it refuses — varying the
fields it reads that the creating command sets and the lifecycle state — so that each measure's
exact value would change if its condition were dropped or inverted, and refuses the view
(`ESS-SYNTH-017`) where no arrangement does. Such a suite is written as `ess-conformance/38` or
`/39`. A view a binding or precondition changes the rows of is refused by name.
