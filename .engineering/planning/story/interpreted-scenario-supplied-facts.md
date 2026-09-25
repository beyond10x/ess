---
format: aep.planning-md/2
id: story:interpreted-scenario-supplied-facts
kind: story
status: draft
title: A scenario supplies what the model does not determine
summary: External outcomes, time and minted identities come from the scenario, never from the interpreter
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
- depends_on: story:interpreted-target-selection
scope:
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests
revision: 3
---
# Story: a scenario supplies what the model does not determine

## Outcome

Four things the model does not decide come from the scenario, through the surfaces that already
exist for them: an outcome declared `external:` through `configure_external_outcome`, the current
time through `mark_instant` and `observe_elapsed`, an identity nothing supplies by the interpreter
minting one and the scenario binding it by name with `capture:`, and how a projection is stored by
it not being observable at all.

The interpreter refuses to invent any of them. A scenario supplying what the model does not is what
a scenario is for.

## Acceptance

A scenario that scripts an `external:` outcome, claims a length of time, and binds a minted identity
with `capture:` reports the same result under `--target interpreted` as under `--target billing`,
and a scenario that scripts none of them reports an unsatisfied obligation rather than an invented
value.

## Scope

The interpreter's `ConformanceTarget` hooks: `configure_external_outcome`, `mark_instant`,
`observe_elapsed`, and identity minting. No new suite vocabulary.
