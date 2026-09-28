---
format: aep.planning-md/3
id: story:set-effects-over-filtered-instances
kind: story
status: draft
title: An outcome can change every instance that matches a filter
refs:
- provider: github
  reference: beyond10x/ess#167
- provider: github
  reference: beyond10x/ess#175
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

#167: a command whose effect moves or updates every row matching a field filter, with a count. #175: an outcome's secondary effect on other instances selected from the subject's relations or a filter (`affects:`).

## Acceptance

Synthesis arranges matching and non-matching rows and fails an implementation that changes a non-matching row, skips a matching one, or reports a wrong count.
