---
format: aep.planning-md/3
id: story:related-record-value-source
kind: story
status: draft
title: 'A payload or sets: value can read a field of a record the subject references'
refs:
- provider: github
  reference: beyond10x/ess#166
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

A value source through a declared `relations:` reference of the subject (e.g. the shipment's customer's region). Listed under *Not in this design* in `value-expressions.md`.

## Acceptance

The #166 repro validates; the scenario arranges two customers and fails an implementation that emits the wrong customer's region.
