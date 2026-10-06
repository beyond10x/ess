---
format: aep.planning-md/3
id: story:feature-request-446
kind: story
status: implemented
title: Order-dependent aggregates and nested grouped results
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#446
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-441
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: cited
  path: docs/design/aggregate-views.md
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/latest.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:26Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:26Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:38Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#446: Order-dependent aggregates and nested grouped results.

## Origin
beyond10x/ess#446, filed 2026-10-05; an adopter's metrics read API, where two reads return the last row per group in query order and one returns groups nested inside an outer group.

## Fit review
1. Need, in two parts:
   - (a) per group, a value taken from one row chosen by an order: the latest row by an instant, an arg-max;
   - (b) one response whose rows for an outer key each hold a list of inner groups.
   Minimal reproduction, written fresh (`<fit-review scratch>/probe-447/`, shared with #447). The requester's syntax, not adopted: aggregate `last`/`first` ordered by a declared field, and a nested-group view shape.
2. Class:
   - (a) is partly a convenience. Where the wanted value is the order key itself, `max` states it. Where it is another field of the latest row, it is a gap at the aggregate level but expressible per group (question 3).
   - "Last in query order" with no declared order is not a domain fact. ESS's own rule is that "a slice of an unordered view names no particular rows" (values-and-views.md:359-361).
   - (b) is presentation of a flat grouping, so local policy.
3. Existing idiom, all validating on the 0.53.0 debug build (probe-447):
   - latest instant per group: `group_by: [queue_id]`, `{max: started_at}` with type `Optional<Timestamp>`. `min`/`max` order Timestamps by instant (aggregate-views.md:186-187).
   - latest row of one group: a row view `filter: queue_id == param.queue_id`, `order_by: [started_at desc]`, `paging: {page, size}`, read with size 1 (values-and-views.md:333-361). Synthesis on that minimal model refused `ESS-SYNTH-014` only because its one-state lifecycle cannot make two rows to order. That is a property of the probe, not of the idiom.
   - nested groups: `group_by: [outer, inner]` returns one flat row per pair, as the aggregate-views fixture's `SessionsByAgentChannel` does (`crates/verify/ess-conformance/tests/fixtures/aggregate-views.yaml`). The client nests them.
   - `{last: …}` is refused, "unknown variant `last`, expected one of `count`, `count_distinct`, `sum`, `min`, `max`, `avg`" (probe-447 n2).
   - `order_by:` on an aggregate view is refused as `ESS-VIEW-009`, "ranking aggregate rows is a window, which is not in this cut" (probe-447 n3; aggregate-views.md:262, :1034).
4. Fit:
   - `first`/`last` with a `by:` would be the first aggregate whose result depends on a second field and on a tie rule. Every target (interpreter, generated Rust and Go queries, Go and TypeScript runners, Entity Runtime lowering) would need the same tie-break, and synthesis would need rows that tie and rows that differ.
   - Its sibling, ranking aggregate rows, was put out of scope as a window (aggregate-views.md:1034). Adding arg-max without windows splits that line.
   - A nested shape would be the first view whose row is not flat. `shape:` and `fields:` are row types (values-and-views.md:304-307), and every projection, the conformance row comparison and the UI read are built on flat rows (inferred from the same section).
5. Second adopter: "current status per device" is an arg-max many readers want; a per-device parameterized read with size 1 serves it today. No second unrelated case of nested groups that a flat grouping does not answer.
6. Cost if accepted: two new aggregate functions with an order field and a tie rule, or a nested view shape; ess/23; a suite pair; diff classes; every target. Cost of the idiom: one section of the shared idioms note and its example.
7. Alternatives:
   - (a) Change nothing.
   - (b) The requester's `last`/`first` + nested shape.
   - (c) An `arg_max: {field, by}` aggregate with ties broken by identity: the smallest form of (a), recorded for when a second adopter needs it inside one grouped result.
   - Chosen: decline with idioms, keeping windows and ranking out together.

## Decisions
decline, with the idiom — The story body is the decline record. The section `## An order-dependent value needs a declared order` in `docs/design/read-api-view-idioms.md` documents three idioms, with their models in `docs/design/read-api-view-idioms.example/latest.yaml`, domain `idioms.latest`: entity `idioms.latest.Event` (`outer`, `inner`, `at`), and the views `idioms.latest.LatestAt` (`max: at`), `idioms.latest.LatestEvent` (parameterised, ordered by `at`, paged) and `idioms.latest.ByOuterInner` (`group_by: [outer, inner]`):
- `max`/`min` for the latest or earliest instant;
- a parameterized, ordered, paged row view read with size 1 for the latest row of one group;
- a flat `group_by: [outer, inner]` the consumer nests.

It states that an order-dependent aggregate needs a declared order and a tie rule, and that both are out of scope with windows (aggregate-views.md:1034). It records `arg_max: {field, by}`, ties broken by identity, as the starting shape if reopened. Depends on #441 (edge recorded), which creates the note, the example directory and the test. No format bump.

## Acceptance
- order_dependent_section_states_the_idiom: `docs/design/read-api-view-idioms.md` has the heading `## An order-dependent value needs a declared order`. The section contains "`max`", "`order_by`", "size 1", "`group_by: [outer, inner]`", "tie rule", "windows" and "`arg_max`". The case reads the note and fails on a missing heading or phrase.
- order_dependent_idiom_examples_validate: the example directory with `latest.yaml` validates, holding `idioms.latest.LatestAt`, `idioms.latest.LatestEvent` and `idioms.latest.ByOuterInner`.
- order_dependent_constructs_stay_refused: `{last: …}` is refused naming the six functions, and `order_by:` on an aggregate view refuses `ESS-VIEW-009`.
- latest_row_view_idiom_synthesizes: `idioms.latest.LatestEvent`, in a model whose creating command can make two rows, synthesizes its ordered-view scenarios with no `ESS-SYNTH-014`.

All four cases live in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs`.

## Scope
- docs/design/read-api-view-idioms.md  inferred — adds `## An order-dependent value needs a declared order` (file created by #441)
- docs/design/read-api-view-idioms.example/latest.yaml  inferred — domain `idioms.latest`: the three idioms (directory created by #441)
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — section, validation, refusals, latest-row synthesis
- docs/design/aggregate-views.md  cited — windows and ranking out of scope, 262, 1034
