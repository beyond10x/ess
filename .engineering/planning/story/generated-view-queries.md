---
format: aep.planning-md/3
id: story:generated-view-queries
kind: story
status: active
title: A fully declared view's query is generated over the storage port
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T21:26:32Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T21:26:33Z", actor: "human:timo", revision: 3}
---
## Outcome

A view whose rows the specification fully declares gets a generated query over the storage port,
instead of a `…Query` obligation: plain projections of a source entity, `filter:` views and
`aggregation:` views, each with the conformance interpreter's semantics.

## Acceptance

- The storage port gains a method listing an entity's rows (the generator chooses its shape);
  generated queries read only through the port.
- Projection views (source entity, own fields plus `state`), `filter:` views and `aggregation:` views
  (`group_by`, sum, count, count_distinct, and every aggregate the compiler accepts) are generated
  when the interpreter evaluates them; ordering, paging and absent handling (an absent group key,
  count_distinct over absent values) follow the interpreter exactly; a view it does not evaluate
  stays an obligation naming the construct.
- A fixture's synthesized suite passes against the generated queries with an in-memory port, with
  no hand-written query.
- A downstream specification with 47 views (42 projections, 1 filter, 4 aggregations) reports how
  many are generated.
- Models whose views stay obligations keep their bytes.
