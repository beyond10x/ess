---
format: aep.planning-md/2
id: story:interpreted-bindings-and-unmet-obligations
kind: story
status: draft
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
revision: 3
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
