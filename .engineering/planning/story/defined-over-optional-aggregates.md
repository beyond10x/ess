---
format: aep.planning-md/3
id: story:defined-over-optional-aggregates
kind: story
status: draft
title: defined() admits an Optional struct, list or map
refs:
- provider: github
  reference: beyond10x/ess#176
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

`defined(x)` over any `Optional<T>`, including aggregate `T`, in guards and invariants.

## Acceptance

The #176 invariant `any: [state == Paused, {not: defined(metrics)}]` validates, and synthesis refutes it with an outcome that leaves `Paused` without clearing `metrics`.
