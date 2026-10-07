---
format: aep.planning-md/3
id: story:a-view-filter-follows-a-reference
kind: story
status: draft
title: A view filter follows a declared references relation one hop
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#492
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Gap 4 of https://github.com/beyond10x/ess/issues/492: `exists: {in: param.queues, as: q, that:
queue.uuid == q}` over a `references` relation of cardinality one validates, and an `exists` over
the related rows for a many-relation. Today `ESS-VIEW-003` refuses it ("not a declared observable
root"). Gap 3 comes first: it states the same rows once the caller resolves the uuids.

Spec first; fit review owed.
