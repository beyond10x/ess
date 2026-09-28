---
format: aep.planning-md/3
id: story:view-paging-and-caller-filters
kind: story
status: draft
title: A view can declare paging and a caller-supplied filter
refs:
- provider: github
  reference: beyond10x/ess#174
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

`page`/`size` parameters that slice the declared order with a total, and a caller filter parameter, without the unobservable-parameter refusal.

## Acceptance

The #174 repro validates; a scenario with more rows than `size` requires the page length, slice and total.
