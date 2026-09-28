---
format: aep.planning-md/3
id: story:optional-input-narrowed-after-refusal
kind: story
status: draft
title: An Optional input is narrowed after an outcome refuses its absence
refs:
- provider: github
  reference: beyond10x/ess#169
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

The type checker reads an earlier `when: not defined(x)` refusing branch and treats `x` as `T` in later branches, under a new source format.

## Acceptance

The #169 repro validates without a conversion; a scenario sends the absent input and requires the refusal.
