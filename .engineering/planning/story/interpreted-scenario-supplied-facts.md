---
format: aep.planning-md/3
id: story:interpreted-scenario-supplied-facts
kind: story
status: active
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
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T16:54:46Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T16:54:46Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
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

## Complete runtime batch authorization and ownership

The operator's explicit mandate, “EVERY conformance test must support all the features,” authorizes completing this existing interpreter work as part of the grouped consumer-runtime batch. The native audit identified actual implementation gaps, not intrinsic model or consumer-adapter limits: bindings/redelivery/invocation observation, ordinary typed responses, view ordering/paging/aggregates and scenario-supplied facts. The original named acceptance remains required; a passing native Runner using another Target does not deliver this story.

Implementation owner: server_corrections in managed tree ess-backlog-one-time-response-20261002, after its explicit partial checkpoint. Its production ownership is interpret.rs and interpret/**, dedicated interpreter tests, and any reviewed observer/recording corrections. Coordinator owns synthesize/disclosure.rs, one_time_response/produce.rs and producer tests in carrier ess-backlog-next-20261002 after checkpoint handoff. Shared authored::compile_one policy attachment is frozen in that checkpoint; later changes require coordination. Go and TypeScript workers retain their own runtime paths. The interpreter trust comparison and fault matrix must run before claiming complete support. All work remains in the grouped PR; no additional remote gate is scheduled for intermediate checkpoints.
