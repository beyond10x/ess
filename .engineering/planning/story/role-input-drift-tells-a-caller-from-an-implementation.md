---
format: aep.planning-md/3
id: story:role-input-drift-tells-a-caller-from-an-implementation
kind: story
status: draft
title: Role input drift tells a caller from an implementation
summary: 'Adopter report N12: role_input_drift on a producer sending a subset (ess#666).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#666
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

An adopter gets `role_input_drift` on 13 fields with `stands: implements` where "the producer sends a subset". Under the `implements` rule (`crates/specify/ess-composition/src/roles.rs:1-15`) a local command missing a contract input is drift. If the local command is the contract's caller, no stance fits it. Adopter report N12, ess 0.68.0. Needs a minimal repro first.

## Acceptance

- `an_implementation_missing_a_contract_input_drifts`: unchanged behaviour, pinned as a fixture.
- `a_caller_sending_a_subset_composes`: if the repro shows the caller case, a stance for a caller sending a subset of the contract's inputs composes, and a required contract input the caller omits is reported.
- `the_drift_message_names_the_direction`: the refusal says which side lacks the field.
