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

## A list parameter, an empty list and a default

A read that selects the rows whose key is in a list the caller sends declares the parameter as a
`List` and asks for membership with a quantifier: `exists: {in: param.queues, as: q, that: queue_id
== q}` holds for a row whose `queue_id` is one of the values sent, and a value no row holds selects
nothing. Where an empty list means every queue, a second disjunct says so: `param.queues.count ==
0`. A switch the caller may leave out is an `Optional<Boolean>` parameter, and what it means when
absent is written as a disjunct over `not defined(param.abandoned)`.

```yaml
format: ess/22
system: metrics
version: v1
domain: metrics.calls
events:
  - name: metrics.calls.CallRecorded
    fields: [{name: call_id, type: Uuid}]
entities:
  - name: metrics.calls.Call
    identity: {name: call_id, type: Uuid}
    fields:
      - {name: queue_id, type: Integer}
      - {name: abandoned, type: Boolean}
    lifecycle: {initial: Recorded, states: [Recorded], terminal: [Recorded], transitions: []}
commands:
  - name: metrics.calls.RecordCall
    input:
      - {name: queue_id, type: Integer}
      - {name: abandoned, type: Boolean}
    outcomes:
      - name: recorded
        creates: metrics.calls.Call
        instance: call_id
        sets: {queue_id: input.queue_id, abandoned: input.abandoned}
        emits: [metrics.calls.CallRecorded]
        payload: {metrics.calls.CallRecorded: {call_id: {generated: true}}}
views:
  - name: metrics.calls.InQueues
    source: metrics.calls.Call
    consistency: read_your_writes
    params: [{name: queues, type: List<Integer>}]
    filter:
      any:
        - param.queues.count == 0
        - {exists: {in: param.queues, as: q, that: queue_id == q}}
    group_by: [queue_id]
    fields:
      - {name: queue_id, type: Integer}
      - {name: calls, type: Integer, aggregate: {count: {}}}
  - name: metrics.calls.ByAbandonment
    source: metrics.calls.Call
    consistency: read_your_writes
    params: [{name: abandoned, type: Optional<Boolean>}]
    filter:
      any:
        - all: ["not defined(param.abandoned)", abandoned == false]
        - abandoned == param.abandoned
    group_by: [queue_id]
    fields:
      - {name: queue_id, type: Integer}
      - {name: calls, type: Integer, aggregate: {count: {}}}
```

`default:` on a parameter and `{in: param.queues}` are not admitted. `default:` is an unknown
field. `in`, `any_of`, `one_of`, `not_in` and `none_of` hold literal values only, so `queue_id: {in:
param.queues}` would compare with the text `param.queues`; it is refused as `type_mismatch`, naming
the `exists` form, and so is a command input written the same way (`{in: input.allowed}`).

Conformance sends a list parameter over a group key one arranged key at a time, as a list of one,
and asserts that group exactly and every other arranged group absent. It then sends every arranged
key in one list, the first of them not the first group's, so a target that reads only the first
element answers fewer groups; and, where the `param.queues.count == 0` disjunct is written, `[]`,
asserting every group. Like any group selection this needs the suite's empty initial state. The
switch is not witnessed: its parameter is read twice, and the view keeps the refusal
`ESS-SYNTH-017`. The `OpenAPI` document states a list parameter as its query key repeated,
`queues=1&queues=2` (`style: form`, `explode: true`).

## What an aggregate view does not compute

A view returns the numbers a suite can check exactly, and the consumer computes what follows from
them. There is no ratio or difference between two measures: return both operands as measures of one
row, `queued` as `{count: {}}` beside `abandoned` as `{count: {}, where: abandoned == true}`, and
let the consumer divide, round and decide what a zero denominator reads as. A filter moves a
parameter by one constant at most (`duration_ms > param.limit_ms + 1000`); declare the parameter in
the unit the field is stored in, and convert at the adapter. The same applies to a latest row per
group, a unit, a total kept per key and a field of another entity: each has an idiom with the
constructs above, collected with validated examples in the
[read-API view idioms](https://github.com/beyond10x/ess/blob/main/docs/design/read-api-view-idioms.md).
