---
format: aep.planning-md/2
id: story:upsert-outcome-by-existence
kind: story
status: draft
title: An outcome can be selected by whether the addressed record exists
refs:
- provider: github
  reference: beyond10x/ess#164
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

Two accepted branches, update when the row exists and create when it does not, selected by existence (builds on `unknown_instance:` from 0.37.0).

## Acceptance

The #164 repro validates; the second call with the same identity is required to update, not duplicate or refuse.
