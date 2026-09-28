---
format: aep.planning-md/3
id: story:interpreted-command-execution
kind: story
status: implemented
title: A command's declared outcome is executed from the IR
summary: 'Outcome selection, transition, sets: writes, emitted events and payload mappings, derived rather than decided'
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
- depends_on: story:interpreted-target-selection
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/interpreted_command_execution.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpreted.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_command_execution.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T21:46:29Z", actor: "human:timo", revision: 4, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T21:47:34Z", actor: "human:timo", revision: 5, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T23:22:04Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}, imported: true}
---
# Story: a command's declared outcome is executed from the IR

## Outcome

Executing a command against the interpreter produces exactly what the model determined: which
outcome it takes and from which states that is legal, the transition it makes, the field writes
`sets:` declares from `input.<field>` or a literal, the events it emits and each payload's mapping,
whether a `wrong_state` branch refuses or accepts, and what `invariants:` require of an instance at
rest.

Nothing here is chosen by the interpreter. A refusal the model does not declare is not available to
it: a target that answers `wrong-state` whenever it is inconvenient turns a suite green by making
every branch reachable.

## Acceptance

Every scenario in `examples/billing`'s committed suite that exercises only command execution,
transitions, `sets:` writes, emitted events and declared refusals reports the same result under
`--target interpreted` as under `--target billing`.

## Scope

The interpreter's command path, its instance store and its invariant check. Reads the compiled IR;
writes no code and emits nothing.
