---
format: aep.planning-md/2
id: story:aggregates-over-optional-fields
kind: story
status: draft
title: Aggregate over and group by an Optional field
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 2
---
## Scope

- #148: `sum`/`avg`/`count_distinct` over an `Optional` field skip absent values
  (`skip_absent: true`); a `group_by` key of `Optional<T>` makes the absent value its own group.

## Acceptance

Both repro views in #148 validate and synthesize scenarios with a row that lacks the value, and
the expected aggregate matches the SQL treatment the issue names.
