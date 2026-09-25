---
format: aep.planning-md/2
id: story:interpreted-trust-gate
kind: story
status: draft
title: The interpreter is checked against both hand-written targets
summary: Billing and oracle suites as green as hand-written, and the fault matrix still discriminates
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
- depends_on: story:interpreted-bindings-and-unmet-obligations
- depends_on: story:interpreted-eventual-views
- depends_on: story:interpreted-scenario-supplied-facts
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/faulty.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests
- confidence: inferred
  path: crates/verify/ess-conformance/tests/faults.rs
- confidence: cited
  path: examples/billing
- confidence: cited
  path: examples/oracle-fixture
- confidence: cited
  path: suites/generated/billing/suite.json
- confidence: cited
  path: suites/generated/oracle-fixture/suite.json
revision: 3
---
# Story: the interpreter is checked against both hand-written targets

## Outcome

The interpreter is trusted because two independently-derived implementations of the same document
agree, not because its own tests pass. Where it and a hand-written target agree, the agreement is
worth more than either alone; where they disagree, one of them is wrong and the specification says
which.

`faulty.rs` exists because a fault must leave unrelated scenarios green. Run against the interpreter
too: a fault that reddens everything means the interpreter has coupled things the specification
keeps apart.

## Acceptance

`examples/billing`'s and `examples/oracle-fixture`'s committed suites are as green under `--target
interpreted` as under their hand-written targets, and the `faulty.rs` fault matrix discriminates the
same scenarios under the interpreter as under them.

## Scope

A gate test running both suites under both target families and comparing, plus the fault matrix run.
It adds no capability; it is the evidence the epic's other stories are believed on.
