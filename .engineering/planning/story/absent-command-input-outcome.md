---
format: aep.planning-md/2
id: story:absent-command-input-outcome
kind: story
status: draft
title: A command can declare the outcome for an absent input as a whole
refs:
- provider: github
  reference: beyond10x/ess#170
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

An outcome selected by the absence of the whole command input (no request body), distinct from `{}`. Also: `validate` refusing `not defined(f)` over a non-Optional input that synthesis cannot witness.

## Acceptance

The #170 repro synthesizes a scenario that sends no input and requires the declared error.
