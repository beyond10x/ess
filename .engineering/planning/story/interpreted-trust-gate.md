---
format: aep.planning-md/3
id: story:interpreted-trust-gate
kind: story
status: active
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
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T16:54:46Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T16:54:46Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
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

## Complete runtime batch authorization and ownership

The operator's explicit mandate, “EVERY conformance test must support all the features,” authorizes completing this existing interpreter work as part of the grouped consumer-runtime batch. The native audit identified actual implementation gaps, not intrinsic model or consumer-adapter limits: bindings/redelivery/invocation observation, ordinary typed responses, view ordering/paging/aggregates and scenario-supplied facts. The original named acceptance remains required; a passing native Runner using another Target does not deliver this story.

Implementation owner: server_corrections in managed tree ess-backlog-one-time-response-20261002, after its explicit partial checkpoint. Its production ownership is interpret.rs and interpret/**, dedicated interpreter tests, and any reviewed observer/recording corrections. Coordinator owns synthesize/disclosure.rs, one_time_response/produce.rs and producer tests in carrier ess-backlog-next-20261002 after checkpoint handoff. Shared authored::compile_one policy attachment is frozen in that checkpoint; later changes require coordination. Go and TypeScript workers retain their own runtime paths. The interpreter trust comparison and fault matrix must run before claiming complete support. All work remains in the grouped PR; no additional remote gate is scheduled for intermediate checkpoints.

## Explicit example facts correction, 2026-10-02

The native comparison exposed 8/33 billing and 4/34 Oracle mismatches after callback work, with independently confirmed absent source mappings. Oracle already supplies contact but stores only weight_grams. Billing declares issued_at ordering but IssueInvoice supplies no timestamp and stores none; authored timeline instants do not reach the execution target. Neither elapsed-time hooks nor generated:true establishes a monotonically increasing stored timestamp. The independent read-only reconciliation is retained as interpreted-trust-source-reconciliation.md, SHA256 0001953e05777a3811616c8658bab0ddc1d6513b2830a93469fc35dff1f2e92c; reviewer executions zero.

Decision within the operator's full-feature delivery mandate: preserve the original parity and fault-matrix acceptance and supply explicit facts using existing source vocabulary. Oracle stores contact from input.contact. Billing IssueInvoice gains an explicit required issued_at Timestamp input and stores it; authored witnesses supply distinct literal values preserving the newer-first assertion. This is an intentional example API change, requiring reference target, applicable realization, documentation and generated-artifact updates. No generic clock-to-field semantics, new source format or interpreter-populated arbitrary stored values is authorized by this correction. All 33 billing and 34 Oracle scenario identities and their assertions remain required, along with the independent fault matrix. Results remain incomplete until measured after integration.

Ownership: scope_boolean implements examples/billing, examples/oracle-fixture, affected authored witnesses/reference target and associated regenerated artifacts in its managed tree; server_corrections retains Interpreter production and its new tests. Root owns integration and final evidence. The existing story scope already names examples, reference.rs and suites; exact shared paths are coordinated before edits. No additional remote gate is scheduled.
