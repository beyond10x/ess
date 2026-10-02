---
format: aep.planning-md/3
id: story:interpreted-bindings-and-unmet-obligations
kind: story
status: active
title: A binding reacts under the interpreter, and an unmet obligation stays one
summary: Bindings react with their mapping, delivery and on_failure; an unmet obligation is never a delivery failure
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
- depends_on: story:interpreted-command-execution
scope:
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T16:54:45Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T16:54:45Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
---
# Story: a binding reacts under the interpreter, and an unmet obligation stays one

## Outcome

A binding reacts to the event the model says it reacts to, invokes the command with the mapping the
model declares, and honours its `delivery` and its `on_failure`.

An unmet obligation is reported as an unmet obligation and never as a delivery failure. The emitted
Rust pump draws that line already (`crates/generate/ess-synth/src/rust/system.rs:23-27`): a port
refusing because its behaviour is owed is a fact about an unfinished workspace, not about a
delivery, and escalating it would publish a domain event for a defect no provider caused —
manufactured evidence. The interpreter has no unfilled ports, so it is the implementation most
tempted to drop the distinction.

## Acceptance

Every binding scenario in `examples/billing`'s committed suite reports the same result under
`--target interpreted` as under `--target billing`, and a scenario whose external outcome is never
supplied reports an unsatisfied obligation with no escalation event published.

## Scope

The interpreter's binding dispatch, delivery handling and obligation reporting.

## Complete runtime batch authorization and ownership

The operator's explicit mandate, “EVERY conformance test must support all the features,” authorizes completing this existing interpreter work as part of the grouped consumer-runtime batch. The native audit identified actual implementation gaps, not intrinsic model or consumer-adapter limits: bindings/redelivery/invocation observation, ordinary typed responses, view ordering/paging/aggregates and scenario-supplied facts. The original named acceptance remains required; a passing native Runner using another Target does not deliver this story.

Implementation owner: server_corrections in managed tree ess-backlog-one-time-response-20261002, after its explicit partial checkpoint. Its production ownership is interpret.rs and interpret/**, dedicated interpreter tests, and any reviewed observer/recording corrections. Coordinator owns synthesize/disclosure.rs, one_time_response/produce.rs and producer tests in carrier ess-backlog-next-20261002 after checkpoint handoff. Shared authored::compile_one policy attachment is frozen in that checkpoint; later changes require coordination. Go and TypeScript workers retain their own runtime paths. The interpreter trust comparison and fault matrix must run before claiming complete support. All work remains in the grouped PR; no additional remote gate is scheduled for intermediate checkpoints.
